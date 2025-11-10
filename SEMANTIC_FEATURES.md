# Semantic Search & NLP Features for Fast Browser Search

## Overview
This document outlines the semantic search and NLP enhancements for the Fast Browser Search tool, enabling intelligent content understanding, metadata extraction, and natural language processing capabilities.

## Recommended Rust NLP Libraries

### Primary Choice: **Candle + Ort (ONNX Runtime)**
- **Candle**: Rust-native deep learning framework for running transformer models
- **Ort**: ONNX Runtime bindings for Rust, excellent for sentence embeddings
- **Why**: Best performance, production-ready, supports modern transformer models

### Additional Libraries:
- **Tantivy**: Full-text search with BM25 scoring (already great for hybrid search)
- **rust-bert**: For BERT-based models (heavier but powerful)
- **tokenizers**: HuggingFace tokenizers in Rust
- **whatlang**: Language detection
- **rust-stemmers**: Text stemming for better matching

## Feature Implementation Plan

### 1. Semantic Embeddings
- Use sentence-transformers model (all-MiniLM-L6-v2) via ONNX
- Generate embeddings for each URL title and content
- Store embeddings in a vector database (in-memory initially)
- Compute cosine similarity for semantic search

### 2. Metadata Extraction
```rust
struct EnrichedHistoryEntry {
    // Original fields
    url: String,
    title: Option<String>,
    visit_time: DateTime<Utc>,

    // New semantic fields
    clean_site_name: String,        // "GitHub" instead of "github.com"
    site_category: String,          // "Developer Tools", "Social Media", etc.
    key_topics: Vec<String>,        // ["rust", "programming", "search"]
    summary: String,                // AI-generated summary
    language: String,               // "en", "es", etc.
    embedding: Vec<f32>,            // Sentence embedding vector
    metadata: PageMetadata,
}

struct PageMetadata {
    word_count: usize,
    read_time_minutes: f32,
    main_keywords: Vec<String>,
    extracted_entities: Vec<Entity>, // People, organizations, locations
    sentiment: f32,                 // -1.0 to 1.0
}
```

### 3. Site Name Beautification
Map common domains to friendly names:
- github.com → "GitHub"
- stackoverflow.com → "Stack Overflow"
- reddit.com → "Reddit"
- youtube.com → "YouTube"
- docs.rust-lang.org → "Rust Documentation"

### 4. Topic Extraction
- Use TF-IDF for keyword extraction from titles
- Named Entity Recognition (NER) for people, places, organizations
- Category classification using predefined rules or lightweight ML

### 5. Semantic Search Ranking
Combine multiple signals:
- Semantic similarity (cosine distance of embeddings)
- Keyword match score (BM25)
- Recency boost
- Visit frequency
- Domain authority

### 6. Smart Summaries
- Extract key sentences from page titles
- Group related visits into sessions
- Generate topic clusters from browsing patterns

## Architecture Changes

### New Modules
```
src/
├── nlp/
│   ├── mod.rs
│   ├── embeddings.rs      # Sentence embeddings via ONNX
│   ├── extractor.rs       # Metadata & keyword extraction
│   ├── site_mapper.rs     # Domain to friendly name mapping
│   └── summarizer.rs      # Content summarization
├── vector/
│   ├── mod.rs
│   └── index.rs           # Vector similarity search
```

### Database Schema Updates
```sql
-- Additional fields for semantic search
ALTER TABLE history_entries ADD COLUMN embedding BLOB;
ALTER TABLE history_entries ADD COLUMN clean_site_name TEXT;
ALTER TABLE history_entries ADD COLUMN key_topics TEXT; -- JSON array
ALTER TABLE history_entries ADD COLUMN summary TEXT;
ALTER TABLE history_entries ADD COLUMN language TEXT;
ALTER TABLE history_entries ADD COLUMN metadata TEXT; -- JSON object
```

## Implementation Phases

### Phase 1: Basic NLP Setup (Current)
- [ ] Add Candle and Ort dependencies
- [ ] Implement site name beautification
- [ ] Basic keyword extraction from titles

### Phase 2: Embeddings & Semantic Search
- [ ] Load ONNX sentence transformer model
- [ ] Generate embeddings for all entries
- [ ] Implement vector similarity search
- [ ] Hybrid ranking (semantic + keyword)

### Phase 3: Advanced Features
- [ ] Topic clustering
- [ ] Session detection
- [ ] Smart summaries
- [ ] Entity extraction

## Performance Considerations
- Embedding generation: ~10-20ms per entry (batch processing recommended)
- Vector search: <5ms for 100k entries with HNSW index
- Memory usage: ~1KB per entry for embeddings (384 dimensions × 4 bytes)
- Initial indexing: ~30-60 seconds for 100k entries

## API Enhancements

### New Endpoints
```
POST /api/semantic-search
{
    "query": "machine learning tutorials",
    "use_semantic": true,
    "extract_topics": true,
    "limit": 20
}

GET /api/topics
Returns extracted topics and their frequencies

GET /api/sites
Returns beautified site names and visit statistics

POST /api/similar/{url}
Find semantically similar pages
```

## Dependencies to Add

```toml
[dependencies]
# Core NLP
candle = "0.3"
candle-nn = "0.3"
candle-transformers = "0.3"
ort = "1.16"
tokenizers = "0.15"

# Text Processing
tantivy = "0.21"
whatlang = "0.16"
rust-stemmers = "1.2"
unicode-segmentation = "1.10"

# Utilities
ordered-float = "4.0"  # For similarity calculations
ndarray = "0.15"       # For vector operations
hnsw = "0.11"          # For fast vector search
```

## Next Steps
1. Install NLP dependencies
2. Create site name mapping configuration
3. Implement basic keyword extraction
4. Set up ONNX model loading
5. Add embedding generation pipeline
6. Update search API to support semantic queries