use once_cell::sync::Lazy;
use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};
use std::collections::{HashMap, HashSet};
use unicode_segmentation::UnicodeSegmentation;
use whatlang::detect;

use super::{Entity, EntityType, PageMetadata};
use crate::nlp::site_mapper::SiteMapper;

/// Common stop words to filter out
static STOP_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    vec![
        "the", "be", "to", "of", "and", "a", "in", "that", "have", "i",
        "it", "for", "not", "on", "with", "he", "as", "you", "do", "at",
        "this", "but", "his", "by", "from", "they", "we", "say", "her", "she",
        "or", "an", "will", "my", "one", "all", "would", "there", "their",
        "what", "so", "up", "out", "if", "about", "who", "get", "which", "go",
        "me", "when", "make", "can", "like", "time", "no", "just", "him", "know",
        "take", "people", "into", "year", "your", "good", "some", "could", "them",
        "see", "other", "than", "then", "now", "look", "only", "come", "its", "over",
        "think", "also", "back", "after", "use", "two", "how", "our", "work",
        "first", "well", "way", "even", "new", "want", "because", "any", "these",
        "give", "day", "most", "us", "is", "was", "are", "been", "has", "had",
        "were", "said", "did", "get", "may", "http", "https", "www", "com", "org",
    ]
    .into_iter()
    .collect()
});

/// Technology-related keywords for entity detection
static TECH_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    vec![
        "rust", "python", "javascript", "typescript", "java", "golang", "go", "c++",
        "ruby", "swift", "kotlin", "scala", "haskell", "clojure", "elixir", "erlang",
        "react", "vue", "angular", "svelte", "nextjs", "nodejs", "deno", "bun",
        "docker", "kubernetes", "aws", "azure", "gcp", "terraform", "ansible",
        "git", "github", "gitlab", "bitbucket", "vscode", "vim", "neovim", "emacs",
        "postgresql", "mysql", "mongodb", "redis", "elasticsearch", "kafka", "rabbitmq",
        "linux", "windows", "macos", "ubuntu", "debian", "fedora", "arch",
        "ai", "ml", "machine learning", "deep learning", "neural network", "llm",
        "transformer", "bert", "gpt", "claude", "openai", "anthropic",
        "api", "rest", "graphql", "grpc", "websocket", "microservices", "serverless",
        "frontend", "backend", "fullstack", "devops", "sre", "cicd", "testing",
    ]
    .into_iter()
    .collect()
});

pub struct KeywordExtractor {
    stemmer: Stemmer,
    min_word_length: usize,
    max_keywords: usize,
}

impl KeywordExtractor {
    pub fn new() -> Self {
        Self {
            stemmer: Stemmer::create(Algorithm::English),
            min_word_length: 3,
            max_keywords: 10,
        }
    }

    /// Extract keywords from text using TF-IDF-like scoring
    pub fn extract_keywords(&self, text: &str) -> Vec<String> {
        // Tokenize and clean
        let words = self.tokenize_and_clean(text);

        // Calculate word frequencies
        let mut word_freq: HashMap<String, f32> = HashMap::new();
        let total_words = words.len() as f32;

        for word in &words {
            *word_freq.entry(word.clone()).or_insert(0.0) += 1.0;
        }

        // Normalize frequencies and apply scoring
        let mut scored_words: Vec<(String, f32)> = word_freq
            .into_iter()
            .map(|(word, freq)| {
                let tf = freq / total_words;

                // Boost score for technology keywords
                let boost = if TECH_KEYWORDS.contains(word.to_lowercase().as_str()) {
                    2.0
                } else {
                    1.0
                };

                // Simple scoring: TF * word_length_factor * boost
                let word_len_factor = (word.len() as f32).min(10.0) / 10.0;
                let score = tf * word_len_factor * boost;

                (word, score)
            })
            .collect();

        // Sort by score and return top keywords
        scored_words.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored_words
            .into_iter()
            .take(self.max_keywords)
            .map(|(word, _)| word)
            .collect()
    }

    /// Extract topics based on title and URL
    pub fn extract_topics(&self, title: &str, url: &str) -> Vec<String> {
        let mut topics = Vec::new();

        // Extract keywords from title
        let keywords = self.extract_keywords(title);

        // Add technology keywords found in title
        for keyword in &keywords {
            if TECH_KEYWORDS.contains(keyword.to_lowercase().as_str()) {
                topics.push(keyword.clone());
            }
        }

        // Extract topics from URL path
        let path_segments = SiteMapper::extract_path_segments(url);
        for segment in path_segments {
            let clean_segment = segment.to_lowercase().replace('-', " ").replace('_', " ");
            if clean_segment.len() >= self.min_word_length && !STOP_WORDS.contains(clean_segment.as_str()) {
                // Check if it's a known technology
                if TECH_KEYWORDS.contains(clean_segment.as_str()) {
                    topics.push(clean_segment);
                }
            }
        }

        // Deduplicate while preserving order
        let mut seen = HashSet::new();
        topics.retain(|topic| seen.insert(topic.clone()));

        topics
    }

    /// Extract entities from text (people, organizations, technologies)
    pub fn extract_entities(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();

        // Simple heuristic-based entity extraction
        let words: Vec<&str> = text.split_whitespace().collect();

        // Look for capitalized sequences (potential names/organizations)
        let capitalized_regex = Regex::new(r"\b[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\b").unwrap();
        for cap in capitalized_regex.captures_iter(text) {
            if let Some(match_str) = cap.get(0) {
                let entity_text = match_str.as_str().to_string();

                // Skip if it's a stop word or too short
                if entity_text.len() < 3 || STOP_WORDS.contains(entity_text.to_lowercase().as_str()) {
                    continue;
                }

                // Classify entity type
                let entity_type = if TECH_KEYWORDS.contains(entity_text.to_lowercase().as_str()) {
                    EntityType::Technology
                } else if entity_text.contains(' ') && entity_text.split_whitespace().count() >= 2 {
                    // Multi-word capitalized could be organization or person
                    if entity_text.contains("Inc") || entity_text.contains("LLC") || entity_text.contains("Corp") {
                        EntityType::Organization
                    } else {
                        EntityType::Person
                    }
                } else {
                    EntityType::Other
                };

                entities.push(Entity {
                    text: entity_text,
                    entity_type,
                    confidence: 0.7, // Simple heuristic, so moderate confidence
                });
            }
        }

        // Look for technology mentions
        for word in &words {
            let lower = word.to_lowercase();
            if TECH_KEYWORDS.contains(lower.as_str()) && !entities.iter().any(|e| e.text.to_lowercase() == lower) {
                entities.push(Entity {
                    text: word.to_string(),
                    entity_type: EntityType::Technology,
                    confidence: 0.9,
                });
            }
        }

        entities
    }

    /// Tokenize and clean text
    fn tokenize_and_clean(&self, text: &str) -> Vec<String> {
        text.unicode_words()
            .map(|w| w.to_lowercase())
            .filter(|w| {
                w.len() >= self.min_word_length
                && !STOP_WORDS.contains(w.as_str())
                && w.chars().any(|c| c.is_alphabetic())
            })
            .map(|w| self.stemmer.stem(&w).to_string())
            .collect()
    }

    /// Detect language of text
    pub fn detect_language(&self, text: &str) -> String {
        detect(text)
            .map(|info| info.lang().to_code())
            .unwrap_or("en")
            .to_string()
    }

    /// Create page metadata from URL and title
    pub fn create_metadata(&self, url: &str, title: Option<&str>) -> PageMetadata {
        let (domain, subdomain) = SiteMapper::extract_domain_parts(url);
        let path_segments = SiteMapper::extract_path_segments(url);

        let text = title.unwrap_or("");
        let keywords = self.extract_keywords(text);
        let entities = self.extract_entities(text);
        let word_count = text.unicode_words().count();

        PageMetadata {
            word_count,
            main_keywords: keywords,
            extracted_entities: entities,
            domain,
            subdomain,
            path_segments,
        }
    }
}

impl Default for KeywordExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_extraction() {
        let extractor = KeywordExtractor::new();
        let text = "Rust is a systems programming language that runs blazingly fast and prevents segfaults";
        let keywords = extractor.extract_keywords(text);

        assert!(!keywords.is_empty());
        // Should extract important words like "rust", "programming", "language"
        assert!(keywords.iter().any(|k| k.to_lowercase().contains("rust") || k.to_lowercase().contains("program")));
    }

    #[test]
    fn test_topic_extraction() {
        let extractor = KeywordExtractor::new();
        let title = "Building a REST API with Rust and Actix-Web";
        let url = "https://blog.example.com/rust/web/api-tutorial";

        let topics = extractor.extract_topics(title, url);
        assert!(!topics.is_empty());
        // Should find "rust", "api" as topics
        assert!(topics.iter().any(|t| t.to_lowercase() == "rust"));
    }

    #[test]
    fn test_entity_extraction() {
        let extractor = KeywordExtractor::new();
        let text = "Microsoft Azure announced new Kubernetes features for Docker containers";

        let entities = extractor.extract_entities(text);
        assert!(!entities.is_empty());

        // Should find "Microsoft Azure" as organization and "Kubernetes", "Docker" as technologies
        assert!(entities.iter().any(|e| matches!(e.entity_type, EntityType::Technology)));
    }

    #[test]
    fn test_language_detection() {
        let extractor = KeywordExtractor::new();

        let english = "This is an English text about programming";
        assert_eq!(extractor.detect_language(english), "en");

        let spanish = "Este es un texto en español sobre programación";
        assert_eq!(extractor.detect_language(spanish), "es");
    }
}