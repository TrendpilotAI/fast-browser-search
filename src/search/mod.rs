use crate::browser::{self, BrowserHistory};
use crate::db::{falkor::FalkorDB, redis_cache::RedisCache, HistoryDatabase, SearchQuery, SearchResult};
use crate::memory::MemoryManager;
use anyhow::{Context, Result};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SearchEngine {
    falkor_db: Arc<FalkorDB>,
    redis_cache: Arc<RwLock<RedisCache>>,
    memory_manager: Arc<MemoryManager>,
    indexed_urls: Arc<DashMap<String, bool>>,
}

impl SearchEngine {
    pub async fn new(
        falkor_host: &str,
        falkor_port: u16,
        redis_url: &str,
        zep_url: &str,
        graphiti_url: &str,
        api_key: Option<String>,
    ) -> Result<Self> {
        let falkor_db = Arc::new(FalkorDB::new(falkor_host, falkor_port, "browser_history")?);
        let redis_cache = Arc::new(RwLock::new(RedisCache::new(redis_url, 3600).await?));
        let memory_manager = Arc::new(MemoryManager::new(zep_url, graphiti_url, api_key)?);

        Ok(Self {
            falkor_db,
            redis_cache,
            memory_manager,
            indexed_urls: Arc::new(DashMap::new()),
        })
    }

    pub async fn index_all_browsers(&self) -> Result<()> {
        tracing::info!("Starting browser history indexing...");

        let histories = browser::get_all_browser_histories()?;
        let total_entries: usize = histories.iter().map(|h| h.entries.len()).sum();

        tracing::info!(
            "Found {} browser histories with {} total entries",
            histories.len(),
            total_entries
        );

        for history in histories {
            self.index_browser_history(history).await?;
        }

        tracing::info!("Browser history indexing completed");
        Ok(())
    }

    async fn index_browser_history(&self, history: BrowserHistory) -> Result<()> {
        let browser_name = format!("{:?}", history.browser);

        // Filter out already indexed URLs
        let new_entries: Vec<_> = history
            .entries
            .into_iter()
            .filter(|entry| !self.indexed_urls.contains_key(&entry.url))
            .collect();

        if new_entries.is_empty() {
            tracing::debug!("No new entries for {}", browser_name);
            return Ok(());
        }

        tracing::info!(
            "Indexing {} new entries for {} ({})",
            new_entries.len(),
            browser_name,
            history.profile
        );

        // Mark URLs as indexed
        for entry in &new_entries {
            self.indexed_urls.insert(entry.url.clone(), true);
        }

        // Insert into FalkorDB
        self.falkor_db
            .insert_entries(new_entries, browser_name)
            .await
            .context("Failed to insert entries into FalkorDB")?;

        // Invalidate relevant cache entries
        let mut cache = self.redis_cache.write().await;
        cache.invalidate_search_cache("*").await?;

        Ok(())
    }

    pub async fn search(
        &self,
        query: SearchQuery,
        session_id: Option<String>,
    ) -> Result<Vec<SearchResult>> {
        // Check Redis cache first
        let mut cache = self.redis_cache.write().await;

        if let Some(cached_results) = cache.get_search_results(&query).await? {
            tracing::debug!("Returning cached results for query: {}", query.query);
            return Ok(cached_results);
        }

        drop(cache); // Release the write lock

        // Get context from memory manager if session exists
        let context = if let Some(ref sid) = session_id {
            Some(
                self.memory_manager
                    .get_context(sid, &query.query)
                    .await
                    .ok(),
            )
        } else {
            None
        };

        // Enhance query with context
        let enhanced_query = if let Some(Some(ctx)) = context {
            self.enhance_query_with_context(query.clone(), ctx).await
        } else {
            query.clone()
        };

        // Search in FalkorDB
        let results = self
            .falkor_db
            .search(enhanced_query)
            .await
            .context("Failed to search FalkorDB")?;

        // Cache the results
        let mut cache = self.redis_cache.write().await;
        cache.set_search_results(&query, &results).await?;

        // Update memory if session exists
        if let Some(sid) = session_id {
            let urls: Vec<String> = results.iter().map(|r| r.url.clone()).collect();
            self.memory_manager
                .add_search_memory(&sid, &query.query, &urls)
                .await?;
        }

        Ok(results)
    }

    async fn enhance_query_with_context(
        &self,
        mut query: SearchQuery,
        context: crate::memory::ConversationContext,
    ) -> SearchQuery {
        // Add related topics to the search
        if !context.related_topics.is_empty() {
            tracing::debug!("Enhancing query with related topics: {:?}", context.related_topics);
            // You could expand the query here based on context
        }

        // Apply user preferences
        for (key, value) in context.user_preferences {
            match key.as_str() {
                "default_limit" => {
                    if let Ok(limit) = value.parse::<usize>() {
                        query.limit = limit;
                    }
                }
                "preferred_browsers" => {
                    if query.browsers.is_none() {
                        query.browsers = Some(value.split(',').map(String::from).collect());
                    }
                }
                _ => {}
            }
        }

        query
    }

    pub async fn get_suggestions(&self, partial_query: &str) -> Result<Vec<String>> {
        let mut cache = self.redis_cache.write().await;
        let recent = cache.get_recent_searches().await?;

        // Filter recent searches that match the partial query
        let suggestions: Vec<String> = recent
            .into_iter()
            .filter(|s| s.to_lowercase().contains(&partial_query.to_lowercase()))
            .take(10)
            .collect();

        Ok(suggestions)
    }

    pub async fn get_popular_urls(&self, limit: isize) -> Result<Vec<(String, i64)>> {
        let mut cache = self.redis_cache.write().await;
        cache.get_popular_urls(limit).await
    }

    pub async fn get_domains(&self) -> Result<Vec<String>> {
        self.falkor_db.get_domains().await
    }

    pub async fn get_related_urls(&self, url: &str, limit: usize) -> Result<Vec<String>> {
        self.falkor_db.get_related_urls(url, limit).await
    }
}