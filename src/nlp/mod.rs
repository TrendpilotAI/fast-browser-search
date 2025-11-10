pub mod site_mapper;
pub mod extractor;
pub mod embeddings;
pub mod summarizer;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedHistoryEntry {
    // Original fields
    pub url: String,
    pub title: Option<String>,
    pub visit_time: chrono::DateTime<chrono::Utc>,
    pub visit_count: i32,
    pub browser_source: String,

    // New semantic fields
    pub clean_site_name: String,
    pub site_category: Option<String>,
    pub key_topics: Vec<String>,
    pub summary: Option<String>,
    pub language: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: PageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetadata {
    pub word_count: usize,
    pub main_keywords: Vec<String>,
    pub extracted_entities: Vec<Entity>,
    pub domain: String,
    pub subdomain: Option<String>,
    pub path_segments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub text: String,
    pub entity_type: EntityType,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Technology,
    Product,
    Other,
}

/// Configuration for NLP processing
#[derive(Debug, Clone)]
pub struct NLPConfig {
    pub enable_embeddings: bool,
    pub embedding_model: String,
    pub max_keywords: usize,
    pub min_keyword_length: usize,
    pub enable_summarization: bool,
}

impl Default for NLPConfig {
    fn default() -> Self {
        Self {
            enable_embeddings: true,
            embedding_model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            max_keywords: 10,
            min_keyword_length: 3,
            enable_summarization: false,
        }
    }
}