use super::{HistoryDatabase, SearchQuery, SearchResult};
use crate::browser::HistoryEntry;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use dashmap::DashMap;
use url::Url;

pub struct SimpleStorage {
    entries: Arc<DashMap<String, Vec<HistoryEntry>>>,
}

impl SimpleStorage {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
        }
    }

    fn extract_domain(url: &str) -> String {
        Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(String::from))
            .unwrap_or_else(|| "unknown".to_string())
    }
}

#[async_trait]
impl HistoryDatabase for SimpleStorage {
    async fn insert_entries(&self, entries: Vec<HistoryEntry>, browser: String) -> Result<()> {
        self.entries.insert(browser.clone(), entries);
        tracing::info!("Stored {} entries for browser {}",
                     self.entries.get(&browser).map(|e| e.len()).unwrap_or(0),
                     browser);
        Ok(())
    }

    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let query_lower = query.query.to_lowercase();

        for entry in self.entries.iter() {
            let browser = entry.key();
            let entries = entry.value();

            for history_entry in entries.iter() {
                // Check if URL or title matches the query
                let url_matches = history_entry.url.to_lowercase().contains(&query_lower);
                let title_matches = history_entry.title
                    .as_ref()
                    .map(|t| t.to_lowercase().contains(&query_lower))
                    .unwrap_or(false);

                if url_matches || title_matches {
                    // Apply browser filter if specified
                    if let Some(ref browsers) = query.browsers {
                        if !browsers.contains(browser) {
                            continue;
                        }
                    }

                    // Apply date filters
                    if let Some(date_from) = query.date_from {
                        if history_entry.visit_time < date_from {
                            continue;
                        }
                    }

                    if let Some(date_to) = query.date_to {
                        if history_entry.visit_time > date_to {
                            continue;
                        }
                    }

                    results.push(SearchResult {
                        url: history_entry.url.clone(),
                        title: history_entry.title.clone(),
                        visit_time: history_entry.visit_time,
                        visit_count: history_entry.visit_count,
                        relevance_score: 1.0,
                        browser_source: browser.clone(),
                        domain: Self::extract_domain(&history_entry.url),
                        related_urls: Vec::new(),
                        // New semantic fields (will be populated by semantic search engine)
                        clean_site_name: None,
                        site_category: None,
                        key_topics: None,
                        summary: None,
                    });
                }
            }
        }

        // Sort by visit time (most recent first)
        results.sort_by(|a, b| b.visit_time.cmp(&a.visit_time));

        // Apply pagination
        results = results
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect();

        Ok(results)
    }

    async fn get_domains(&self) -> Result<Vec<String>> {
        let mut domains = std::collections::HashSet::new();

        for entry in self.entries.iter() {
            for history_entry in entry.value().iter() {
                domains.insert(Self::extract_domain(&history_entry.url));
            }
        }

        let mut domain_list: Vec<String> = domains.into_iter().collect();
        domain_list.sort();
        Ok(domain_list)
    }

    async fn get_related_urls(&self, url: &str, limit: usize) -> Result<Vec<String>> {
        let domain = Self::extract_domain(url);
        let mut related = Vec::new();

        for entry in self.entries.iter() {
            for history_entry in entry.value().iter() {
                if history_entry.url != url && Self::extract_domain(&history_entry.url) == domain {
                    related.push(history_entry.url.clone());
                    if related.len() >= limit {
                        return Ok(related);
                    }
                }
            }
        }

        Ok(related)
    }
}