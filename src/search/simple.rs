use crate::browser::{self, BrowserHistory};
use crate::db::{simple_storage::SimpleStorage, HistoryDatabase, SearchQuery, SearchResult};
use anyhow::{Context, Result};
use dashmap::DashMap;
use std::sync::Arc;

pub struct SimpleSearchEngine {
    db: Arc<SimpleStorage>,
    indexed_urls: Arc<DashMap<String, bool>>,
}

impl SimpleSearchEngine {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            db: Arc::new(SimpleStorage::new()),
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

        // Insert into database
        self.db
            .insert_entries(new_entries, browser_name)
            .await
            .context("Failed to insert entries into database")?;

        Ok(())
    }

    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        self.db.search(query).await.context("Failed to search database")
    }

    pub async fn get_suggestions(&self, partial_query: &str) -> Result<Vec<String>> {
        // Simple suggestion based on partial matches
        let results = self.search(SearchQuery {
            query: partial_query.to_string(),
            limit: 10,
            offset: 0,
            browsers: None,
            date_from: None,
            date_to: None,
            domains: None,
        }).await?;

        Ok(results.into_iter().map(|r| r.url).collect())
    }

    pub async fn get_popular_urls(&self, limit: isize) -> Result<Vec<(String, i64)>> {
        // Return top URLs by visit count
        let all_results = self.search(SearchQuery {
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

    pub async fn get_domains(&self) -> Result<Vec<String>> {
        self.db.get_domains().await
    }

    pub async fn get_related_urls(&self, url: &str, limit: usize) -> Result<Vec<String>> {
        self.db.get_related_urls(url, limit).await
    }
}