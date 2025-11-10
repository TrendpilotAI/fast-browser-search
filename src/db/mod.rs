pub mod falkor;
pub mod redis_cache;
pub mod schema;

use anyhow::Result;
use async_trait::async_trait;
use crate::browser::HistoryEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub url: String,
    pub title: Option<String>,
    pub visit_time: chrono::DateTime<chrono::Utc>,
    pub relevance_score: f32,
    pub browser_source: String,
    pub related_urls: Vec<String>,
    pub domain: String,
    pub visit_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub limit: usize,
    pub offset: usize,
    pub browsers: Option<Vec<String>>,
    pub date_from: Option<chrono::DateTime<chrono::Utc>>,
    pub date_to: Option<chrono::DateTime<chrono::Utc>>,
    pub domains: Option<Vec<String>>,
}

#[async_trait]
pub trait HistoryDatabase: Send + Sync {
    async fn insert_entries(&self, entries: Vec<HistoryEntry>, browser: String) -> Result<()>;
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>>;
    async fn get_domains(&self) -> Result<Vec<String>>;
    async fn get_related_urls(&self, url: &str, limit: usize) -> Result<Vec<String>>;
}