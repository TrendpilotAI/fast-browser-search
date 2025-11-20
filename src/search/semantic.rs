use crate::browser::{self, BrowserHistory, HistoryEntry};
use crate::db::{simple_storage::SimpleStorage, HistoryDatabase, SearchQuery, SearchResult};
use crate::gmail::GmailClient;
use crate::nlp::{
    embeddings::{EmbeddingModel, VectorIndex},
    extractor::KeywordExtractor,
    site_mapper::SiteMapper,
    summarizer::Summarizer,
    EnrichedHistoryEntry,
};
use anyhow::{Context, Result};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Semantic search engine with NLP capabilities
pub struct SemanticSearchEngine {
    db: Arc<SimpleStorage>,
    embedding_model: Arc<RwLock<EmbeddingModel>>,
    vector_index: Arc<RwLock<VectorIndex>>,
    extractor: Arc<KeywordExtractor>,
    indexed_urls: Arc<DashMap<String, bool>>,
    enriched_entries: Arc<DashMap<String, EnrichedHistoryEntry>>,
    gmail_client: Arc<RwLock<Option<Arc<GmailClient>>>>,
}

impl SemanticSearchEngine {
    pub async fn new() -> Result<Self> {
        // Initialize embedding model
        let embedding_model = EmbeddingModel::new(None)
            .await
            .context("Failed to initialize embedding model")?;

        Ok(Self {
            db: Arc::new(SimpleStorage::new()),
            embedding_model: Arc::new(RwLock::new(embedding_model)),
            vector_index: Arc::new(RwLock::new(VectorIndex::new())),
            extractor: Arc::new(KeywordExtractor::new()),
            indexed_urls: Arc::new(DashMap::new()),
            enriched_entries: Arc::new(DashMap::new()),
            gmail_client: Arc::new(RwLock::new(None)),
        })
    }

    /// Set Gmail client for email indexing
    pub async fn set_gmail_client(&self, client: Arc<GmailClient>) {
        *self.gmail_client.write().await = Some(client);
    }

    pub async fn index_all_browsers(&self) -> Result<()> {
        tracing::info!("Starting semantic browser history indexing...");

        let histories = browser::get_all_browser_histories()?;
        let total_entries: usize = histories.iter().map(|h| h.entries.len()).sum();

        tracing::info!(
            "Found {} browser histories with {} total entries",
            histories.len(),
            total_entries
        );

        // First pass: Index basic data
        for history in histories {
            self.index_browser_history(history).await?;
        }

        // Index Gmail if client is configured
        if let Some(ref gmail_client) = *self.gmail_client.read().await {
            tracing::info!("Indexing Gmail messages...");
            if let Err(e) = self.index_gmail(gmail_client, 10000).await {
                tracing::warn!("Gmail indexing failed: {}", e);
            }
        }

        // Second pass: Generate embeddings in batches
        self.generate_all_embeddings().await?;

        tracing::info!("Semantic browser history indexing completed");
        Ok(())
    }

    async fn index_gmail(&self, client: &GmailClient, max_messages: usize) -> Result<()> {
        let messages = client.fetch_messages_batch(max_messages, None).await?;
        let gmail_entries: Vec<HistoryEntry> = messages.into_iter().map(Into::into).collect();
        
        if gmail_entries.is_empty() {
            tracing::info!("No Gmail messages to index");
            return Ok(());
        }

        tracing::info!("Indexing {} Gmail messages", gmail_entries.len());

        // Filter out already indexed messages
        let new_entries: Vec<_> = gmail_entries
            .into_iter()
            .filter(|entry| !self.indexed_urls.contains_key(&entry.url))
            .collect();

        if new_entries.is_empty() {
            tracing::debug!("No new Gmail entries to index");
            return Ok(());
        }

        // Process and enrich entries
        for entry in &new_entries {
            let enriched = self.enrich_entry(entry, "Gmail").await?;
            self.enriched_entries.insert(entry.url.clone(), enriched);
            self.indexed_urls.insert(entry.url.clone(), true);
        }

        // Insert into database
        self.db
            .insert_entries(new_entries, "Gmail".to_string())
            .await
            .context("Failed to insert Gmail entries into database")?;

        tracing::info!("Gmail indexing completed");
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

        // Process and enrich entries
        for entry in &new_entries {
            let enriched = self.enrich_entry(entry, &browser_name).await?;
            self.enriched_entries.insert(entry.url.clone(), enriched);
            self.indexed_urls.insert(entry.url.clone(), true);
        }

        // Insert into database
        self.db
            .insert_entries(new_entries, browser_name)
            .await
            .context("Failed to insert entries into database")?;

        Ok(())
    }

    async fn enrich_entry(&self, entry: &HistoryEntry, browser_source: &str) -> Result<EnrichedHistoryEntry> {
        // Get clean site name
        let clean_site_name = SiteMapper::get_clean_site_name(&entry.url);
        let site_category = SiteMapper::get_site_category(&entry.url);

        // Extract keywords and topics
        let title_str = entry.title.as_deref().unwrap_or("");
        let _keywords = self.extractor.extract_keywords(title_str);
        let topics = self.extractor.extract_topics(title_str, &entry.url);

        // Detect language
        let language = self.extractor.detect_language(title_str);

        // Create metadata
        let metadata = self.extractor.create_metadata(&entry.url, entry.title.as_deref());

        // Generate summary
        let summary = Some(Summarizer::summarize_entry(
            entry.title.as_deref(),
            &entry.url,
            Some(&clean_site_name),
            site_category.as_deref(),
            &topics,
        ));

        Ok(EnrichedHistoryEntry {
            url: entry.url.clone(),
            title: entry.title.clone(),
            visit_time: entry.visit_time,
            visit_count: entry.visit_count,
            browser_source: browser_source.to_string(),
            clean_site_name,
            site_category,
            key_topics: topics,
            summary,
            language,
            embedding: None, // Will be generated in batch
            metadata,
        })
    }

    async fn generate_all_embeddings(&self) -> Result<()> {
        tracing::info!("Generating embeddings for all entries...");

        // Collect all texts to embed
        let mut texts_to_embed = Vec::new();
        let mut urls = Vec::new();

        for entry in self.enriched_entries.iter() {
            let text = if let Some(title) = &entry.title {
                format!("{} {}", title, entry.key_topics.join(" "))
            } else {
                entry.clean_site_name.clone()
            };
            texts_to_embed.push(text);
            urls.push(entry.url.clone());
        }

        if texts_to_embed.is_empty() {
            return Ok(());
        }

        // Generate embeddings in batches
        let batch_size = 100;
        let model = self.embedding_model.read().await;
        let mut all_embeddings = Vec::new();

        for batch in texts_to_embed.chunks(batch_size) {
            tracing::debug!("Processing embedding batch of size {}", batch.len());
            let batch_embeddings = model.embed_batch(batch.to_vec()).await?;
            all_embeddings.extend(batch_embeddings);
        }

        drop(model); // Release the read lock

        // Update entries with embeddings and build index
        let mut index = self.vector_index.write().await;
        for (url, embedding) in urls.iter().zip(all_embeddings.iter()) {
            if let Some(mut entry) = self.enriched_entries.get_mut(url) {
                entry.embedding = Some(embedding.clone());
            }
            index.add_embedding(embedding.clone(), url.clone());
        }

        tracing::info!("Generated {} embeddings", all_embeddings.len());
        Ok(())
    }

    pub async fn semantic_search(&self, query: SearchQuery, use_semantic: bool) -> Result<Vec<SearchResult>> {
        if use_semantic && !query.query.is_empty() {
            // Check if vector index has data
            let index = self.vector_index.read().await;
            let has_indexed_data = !index.is_empty();
            drop(index);

            if has_indexed_data {
                // Generate query embedding
                let model = self.embedding_model.read().await;
                let query_embedding = model.embed_text(&query.query).await?;
                drop(model);

                // Search in vector index
                let index = self.vector_index.read().await;
                let similar_items = index.search(&query_embedding, query.limit * 2); // Get more candidates
                drop(index);

                // Convert to search results
                let mut results = Vec::new();
                for (url, similarity_score) in similar_items.iter().take(query.limit) {
                    if let Some(entry) = self.enriched_entries.get(url) {
                        results.push(SearchResult {
                            url: entry.url.clone(),
                            title: entry.title.clone(),
                            visit_time: entry.visit_time,
                            visit_count: entry.visit_count,
                            relevance_score: *similarity_score,
                            browser_source: entry.browser_source.clone(),
                            related_urls: vec![],
                            domain: entry.metadata.domain.clone(),
                            // New fields from enriched data
                            clean_site_name: Some(entry.clean_site_name.clone()),
                            site_category: entry.site_category.clone(),
                            key_topics: Some(entry.key_topics.clone()),
                            summary: entry.summary.clone(),
                        });
                    }
                }

                // If semantic search found results, return them
                if !results.is_empty() {
                    return Ok(results);
                }
            }
            // Fall through to keyword search if semantic search returned no results
        }
        
        // Fall back to keyword search (or use it directly if semantic is disabled)
        {
            // Fall back to keyword search
            let results = self.db.search(query).await?;

            // Enrich results with NLP data
            let mut enriched_results = Vec::new();
            for mut result in results {
                if let Some(entry) = self.enriched_entries.get(&result.url) {
                    result.clean_site_name = Some(entry.clean_site_name.clone());
                    result.site_category = entry.site_category.clone();
                    result.key_topics = Some(entry.key_topics.clone());
                    result.summary = entry.summary.clone();
                } else {
                    // Basic enrichment if not in cache
                    result.clean_site_name = Some(SiteMapper::get_clean_site_name(&result.url));
                    result.site_category = SiteMapper::get_site_category(&result.url);
                }
                enriched_results.push(result);
            }

            Ok(enriched_results)
        }
    }

    pub async fn get_suggestions(&self, partial_query: &str) -> Result<Vec<String>> {
        // Get keyword-based suggestions
        let results = self.db.search(SearchQuery {
            query: partial_query.to_string(),
            limit: 10,
            offset: 0,
            browsers: None,
            date_from: None,
            date_to: None,
            domains: None,
        }).await?;

        // Return clean site names and topics as suggestions
        let mut suggestions = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for result in results {
            if let Some(entry) = self.enriched_entries.get(&result.url) {
                // Add clean site name
                if seen.insert(entry.clean_site_name.clone()) {
                    suggestions.push(entry.clean_site_name.clone());
                }

                // Add topics
                for topic in &entry.key_topics {
                    if seen.insert(topic.clone()) && suggestions.len() < 10 {
                        suggestions.push(topic.clone());
                    }
                }
            }
        }

        Ok(suggestions)
    }

    pub async fn get_topics(&self) -> Result<Vec<(String, usize)>> {
        let mut topic_counts = std::collections::HashMap::new();

        for entry in self.enriched_entries.iter() {
            for topic in &entry.key_topics {
                *topic_counts.entry(topic.clone()).or_insert(0) += 1;
            }
        }

        let mut topics: Vec<(String, usize)> = topic_counts.into_iter().collect();
        topics.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(topics)
    }

    pub async fn get_sites_summary(&self) -> Result<Vec<SiteSummary>> {
        let mut site_stats: std::collections::HashMap<String, SiteSummary> = std::collections::HashMap::new();

        for entry in self.enriched_entries.iter() {
            let summary = site_stats
                .entry(entry.clean_site_name.clone())
                .or_insert_with(|| SiteSummary {
                    clean_name: entry.clean_site_name.clone(),
                    domain: entry.metadata.domain.clone(),
                    category: entry.site_category.clone(),
                    visit_count: 0,
                    page_count: 0,
                    main_topics: Vec::new(),
                });

            summary.visit_count += entry.visit_count as usize;
            summary.page_count += 1;

            // Add topics
            for topic in &entry.key_topics {
                if !summary.main_topics.contains(topic) && summary.main_topics.len() < 5 {
                    summary.main_topics.push(topic.clone());
                }
            }
        }

        let mut summaries: Vec<SiteSummary> = site_stats.into_values().collect();
        summaries.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));

        Ok(summaries)
    }

    pub async fn find_similar(&self, url: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Get the embedding for the given URL
        if let Some(entry) = self.enriched_entries.get(url) {
            if let Some(embedding) = &entry.embedding {
                let index = self.vector_index.read().await;
                let similar_items = index.search(embedding, limit + 1); // +1 to exclude self

                let mut results = Vec::new();
                for (similar_url, score) in similar_items {
                    if similar_url != url {
                        if let Some(similar_entry) = self.enriched_entries.get(&similar_url) {
                            results.push(SearchResult {
                                url: similar_entry.url.clone(),
                                title: similar_entry.title.clone(),
                                visit_time: similar_entry.visit_time,
                                visit_count: similar_entry.visit_count,
                                relevance_score: score,
                                browser_source: similar_entry.browser_source.clone(),
                                related_urls: vec![],
                                domain: similar_entry.metadata.domain.clone(),
                                clean_site_name: Some(similar_entry.clean_site_name.clone()),
                                site_category: similar_entry.site_category.clone(),
                                key_topics: Some(similar_entry.key_topics.clone()),
                                summary: similar_entry.summary.clone(),
                            });
                        }
                    }
                }

                return Ok(results);
            }
        }

        // Fallback to keyword-based similarity
        self.db.get_related_urls(url, limit).await
            .map(|urls| {
                urls.into_iter()
                    .filter_map(|url| {
                        self.enriched_entries.get(&url).map(|entry| SearchResult {
                            url: entry.url.clone(),
                            title: entry.title.clone(),
                            visit_time: entry.visit_time,
                            visit_count: entry.visit_count,
                            relevance_score: 0.5,
                            browser_source: entry.browser_source.clone(),
                            related_urls: vec![],
                            domain: entry.metadata.domain.clone(),
                            clean_site_name: Some(entry.clean_site_name.clone()),
                            site_category: entry.site_category.clone(),
                            key_topics: Some(entry.key_topics.clone()),
                            summary: entry.summary.clone(),
                        })
                    })
                    .collect()
            })
    }

    // Standard interface methods to match SimpleSearchEngine

    /// Standard search method - uses semantic search by default
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        self.semantic_search(query, true).await
    }

    /// Get popular URLs by visit count
    pub async fn get_popular_urls(&self, limit: isize) -> Result<Vec<(String, i64)>> {
        let all_results = self.db.search(SearchQuery {
            query: String::new(),
            limit: limit as usize,
            offset: 0,
            browsers: None,
            date_from: None,
            date_to: None,
            domains: None,
        }).await?;

        Ok(all_results
            .into_iter()
            .map(|r| (r.url, r.visit_count as i64))
            .collect())
    }

    /// Get all domains from the database
    pub async fn get_domains(&self) -> Result<Vec<String>> {
        self.db.get_domains().await
    }

    /// Get related URLs using semantic similarity
    pub async fn get_related_urls(&self, url: &str, limit: usize) -> Result<Vec<String>> {
        let results = self.find_similar(url, limit).await?;
        Ok(results.into_iter().map(|r| r.url).collect())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SiteSummary {
    pub clean_name: String,
    pub domain: String,
    pub category: Option<String>,
    pub visit_count: usize,
    pub page_count: usize,
    pub main_topics: Vec<String>,
}