use anyhow::{Context, Result};
use ndarray::Array1;
use ordered_float::OrderedFloat;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Simple text embedding model using TF-IDF-like approach
/// This is a placeholder until we can properly integrate a real embedding model
pub struct EmbeddingModel {
    vocabulary: Arc<RwLock<HashMap<String, usize>>>,
    idf_scores: Arc<RwLock<HashMap<String, f32>>>,
    dimension: usize,
}

impl EmbeddingModel {
    /// Create a new embedding model
    pub async fn new(_model_name: Option<&str>) -> Result<Self> {
        Ok(Self {
            vocabulary: Arc::new(RwLock::new(HashMap::new())),
            idf_scores: Arc::new(RwLock::new(HashMap::new())),
            dimension: 384, // Simulating standard embedding dimension
        })
    }

    /// Generate embeddings for a single text using TF-IDF-like approach
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        // Simple word-based embedding using hash trick
        let mut embedding = vec![0.0f32; self.dimension];

        // Tokenize and create feature vector
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return Ok(embedding);
        }

        for word in words {
            let word_lower = word.to_lowercase();
            // Simple hash function to map words to dimensions
            let hash = self.simple_hash(&word_lower);
            let index = (hash as usize) % self.dimension;
            embedding[index] += 1.0;
        }

        // Normalize the embedding
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in &mut embedding {
                *value /= norm;
            }
        }

        Ok(embedding)
    }

    /// Generate embeddings for multiple texts (batch processing)
    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();

        for text in texts {
            let embedding = self.embed_text(&text).await?;
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    /// Simple hash function for word to index mapping
    fn simple_hash(&self, word: &str) -> u32 {
        word.chars().fold(0u32, |acc, c| {
            acc.wrapping_mul(31).wrapping_add(c as u32)
        })
    }

    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let a_arr = Array1::from_vec(a.to_vec());
        let b_arr = Array1::from_vec(b.to_vec());

        let dot_product = a_arr.dot(&b_arr);
        let norm_a = (a_arr.dot(&a_arr)).sqrt();
        let norm_b = (b_arr.dot(&b_arr)).sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// Find top-k most similar embeddings
    pub fn find_top_k_similar(
        query_embedding: &[f32],
        embeddings: &[Vec<f32>],
        k: usize,
    ) -> Vec<(usize, f32)> {
        let mut similarities: Vec<(usize, OrderedFloat<f32>)> = embeddings
            .iter()
            .enumerate()
            .map(|(idx, emb)| {
                let sim = Self::cosine_similarity(query_embedding, emb);
                (idx, OrderedFloat(sim))
            })
            .collect();

        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.cmp(&a.1));

        // Take top k
        similarities
            .into_iter()
            .take(k)
            .map(|(idx, sim)| (idx, sim.0))
            .collect()
    }

    pub fn get_dimension(&self) -> usize {
        self.dimension
    }
}

/// Vector index for fast similarity search
pub struct VectorIndex {
    embeddings: Vec<Vec<f32>>,
    metadata: Vec<String>, // Store URLs or IDs
}

impl VectorIndex {
    pub fn new() -> Self {
        Self {
            embeddings: Vec::new(),
            metadata: Vec::new(),
        }
    }

    /// Add an embedding to the index
    pub fn add_embedding(&mut self, embedding: Vec<f32>, metadata: String) {
        self.embeddings.push(embedding);
        self.metadata.push(metadata);
    }

    /// Search for similar items
    pub fn search(&self, query_embedding: &[f32], k: usize) -> Vec<(String, f32)> {
        if self.embeddings.is_empty() {
            return Vec::new();
        }

        let top_k = EmbeddingModel::find_top_k_similar(query_embedding, &self.embeddings, k);

        top_k
            .into_iter()
            .map(|(idx, score)| (self.metadata[idx].clone(), score))
            .collect()
    }

    /// Get the number of indexed items
    pub fn len(&self) -> usize {
        self.embeddings.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.embeddings.is_empty()
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.embeddings.clear();
        self.metadata.clear();
    }

    /// Build from a batch of embeddings and metadata
    pub fn build_from_batch(embeddings: Vec<Vec<f32>>, metadata: Vec<String>) -> Self {
        Self {
            embeddings,
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_generation() {
        let model = EmbeddingModel::new(None).await.unwrap();

        let text = "This is a test sentence for embedding generation";
        let embedding = model.embed_text(text).await.unwrap();

        // Check embedding dimension
        assert_eq!(embedding.len(), 384);

        // Check that values are reasonable (normalized)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01 || norm == 0.0); // Either normalized or zero vector
    }

    #[tokio::test]
    async fn test_batch_embedding() {
        let model = EmbeddingModel::new(None).await.unwrap();

        let texts = vec![
            "First test sentence".to_string(),
            "Second test sentence".to_string(),
            "Third test sentence".to_string(),
        ];

        let embeddings = model.embed_batch(texts).await.unwrap();

        assert_eq!(embeddings.len(), 3);
        assert!(embeddings.iter().all(|emb| emb.len() == 384));
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![0.5, 0.5, 0.0];
        let b = vec![0.5, 0.5, 0.0];
        let c = vec![-0.5, -0.5, 0.0];

        // Same vectors should have similarity 1
        let sim_ab = EmbeddingModel::cosine_similarity(&a, &b);
        assert!((sim_ab - 1.0).abs() < 0.001);

        // Opposite vectors should have negative similarity
        let sim_ac = EmbeddingModel::cosine_similarity(&a, &c);
        assert!(sim_ac < 0.0);
    }

    #[test]
    fn test_vector_index() {
        let mut index = VectorIndex::new();

        // Add some embeddings
        index.add_embedding(vec![0.1, 0.2, 0.3], "url1".to_string());
        index.add_embedding(vec![0.2, 0.3, 0.4], "url2".to_string());
        index.add_embedding(vec![0.9, 0.8, 0.7], "url3".to_string());

        // Search
        let query = vec![0.15, 0.25, 0.35];
        let results = index.search(&query, 2);

        assert_eq!(results.len(), 2);
        // First result should be url2 (most similar to query)
        assert_eq!(results[0].0, "url2");
    }
}