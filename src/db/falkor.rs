use super::{HistoryDatabase, SearchQuery, SearchResult};
use crate::browser::HistoryEntry;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest;
use serde_json::json;
use std::collections::HashMap;
use url::Url;

pub struct FalkorDB {
    client: reqwest::Client,
    base_url: String,
    graph_name: String,
}

impl FalkorDB {
    pub fn new(host: &str, port: u16, graph_name: &str) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: format!("http://{}:{}", host, port),
            graph_name: graph_name.to_string(),
        })
    }

    async fn execute_query(&self, cypher: &str, params: HashMap<String, serde_json::Value>) -> Result<serde_json::Value> {
        let response = self.client
            .post(format!("{}/graph/{}/query", self.base_url, self.graph_name))
            .json(&json!({
                "query": cypher,
                "params": params
            }))
            .send()
            .await
            .context("Failed to execute FalkorDB query")?;

        let result = response.json::<serde_json::Value>().await?;
        Ok(result)
    }

    fn extract_domain(url: &str) -> String {
        Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(String::from))
            .unwrap_or_else(|| "unknown".to_string())
    }

    async fn create_indexes(&self) -> Result<()> {
        // Create indexes for better performance
        let indexes = vec![
            "CREATE INDEX ON :URL(url)",
            "CREATE INDEX ON :URL(visit_time)",
            "CREATE INDEX ON :Domain(name)",
            "CREATE INDEX ON :Browser(name)",
        ];

        for index in indexes {
            let _ = self.execute_query(index, HashMap::new()).await;
        }

        Ok(())
    }
}

#[async_trait]
impl HistoryDatabase for FalkorDB {
    async fn insert_entries(&self, entries: Vec<HistoryEntry>, browser: String) -> Result<()> {
        // Batch insert entries into FalkorDB
        for chunk in entries.chunks(100) {
            let mut cypher = String::from("UNWIND $entries AS entry ");
            cypher.push_str("MERGE (u:URL {url: entry.url}) ");
            cypher.push_str("SET u.title = entry.title, u.last_visit = entry.visit_time, ");
            cypher.push_str("u.visit_count = entry.visit_count ");
            cypher.push_str("MERGE (b:Browser {name: $browser}) ");
            cypher.push_str("MERGE (u)-[:VISITED_WITH]->(b) ");

            // Extract and link domains
            cypher.push_str("WITH u, entry ");
            cypher.push_str("MERGE (d:Domain {name: entry.domain}) ");
            cypher.push_str("MERGE (u)-[:BELONGS_TO]->(d) ");

            // Create temporal relationships
            cypher.push_str("WITH u ");
            cypher.push_str("MATCH (prev:URL) ");
            cypher.push_str("WHERE prev.last_visit < u.last_visit ");
            cypher.push_str("AND NOT EXISTS((prev)-[:FOLLOWED_BY]->(u)) ");
            cypher.push_str("WITH prev, u ");
            cypher.push_str("ORDER BY prev.last_visit DESC ");
            cypher.push_str("LIMIT 1 ");
            cypher.push_str("CREATE (prev)-[:FOLLOWED_BY {browser: $browser}]->(u)");

            let entries_json: Vec<serde_json::Value> = chunk
                .iter()
                .map(|e| json!({
                    "url": e.url,
                    "title": e.title,
                    "visit_time": e.visit_time.to_rfc3339(),
                    "visit_count": e.visit_count,
                    "domain": Self::extract_domain(&e.url)
                }))
                .collect();

            let mut params = HashMap::new();
            params.insert("entries".to_string(), json!(entries_json));
            params.insert("browser".to_string(), json!(browser));

            self.execute_query(&cypher, params).await?;
        }

        Ok(())
    }

    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        let mut cypher = String::from("MATCH (u:URL) ");
        let mut where_clauses = Vec::new();
        let mut params = HashMap::new();

        // Text search condition
        if !query.query.is_empty() {
            where_clauses.push("(u.url CONTAINS $query OR u.title CONTAINS $query)");
            params.insert("query".to_string(), json!(query.query));
        }

        // Browser filter
        if let Some(browsers) = &query.browsers {
            cypher.push_str("MATCH (u)-[:VISITED_WITH]->(b:Browser) ");
            where_clauses.push("b.name IN $browsers");
            params.insert("browsers".to_string(), json!(browsers));
        }

        // Date range filter
        if let Some(date_from) = query.date_from {
            where_clauses.push("u.last_visit >= $date_from");
            params.insert("date_from".to_string(), json!(date_from.to_rfc3339()));
        }

        if let Some(date_to) = query.date_to {
            where_clauses.push("u.last_visit <= $date_to");
            params.insert("date_to".to_string(), json!(date_to.to_rfc3339()));
        }

        // Domain filter
        if let Some(domains) = &query.domains {
            cypher.push_str("MATCH (u)-[:BELONGS_TO]->(d:Domain) ");
            where_clauses.push("d.name IN $domains");
            params.insert("domains".to_string(), json!(domains));
        }

        // Add WHERE clause if there are conditions
        if !where_clauses.is_empty() {
            cypher.push_str(&format!("WHERE {} ", where_clauses.join(" AND ")));
        }

        // Add related URLs using graph traversal
        cypher.push_str("OPTIONAL MATCH (u)-[:FOLLOWED_BY*1..3]-(related:URL) ");
        cypher.push_str("WITH u, COLLECT(DISTINCT related.url) AS related_urls ");

        // Return results with ordering and pagination
        cypher.push_str("RETURN u.url AS url, u.title AS title, u.last_visit AS visit_time, ");
        cypher.push_str("u.visit_count AS visit_count, related_urls ");
        cypher.push_str("ORDER BY u.last_visit DESC ");
        cypher.push_str(&format!("SKIP {} LIMIT {}", query.offset, query.limit));

        params.insert("limit".to_string(), json!(query.limit));
        params.insert("offset".to_string(), json!(query.offset));

        let result = self.execute_query(&cypher, params).await?;

        // Parse results
        let results = Self::parse_search_results(result)?;
        Ok(results)
    }

    async fn get_domains(&self) -> Result<Vec<String>> {
        let cypher = "MATCH (d:Domain) RETURN d.name AS domain ORDER BY domain";
        let result = self.execute_query(cypher, HashMap::new()).await?;

        // Parse domain results
        let domains = Self::parse_domain_results(result)?;
        Ok(domains)
    }

    async fn get_related_urls(&self, url: &str, limit: usize) -> Result<Vec<String>> {
        let cypher = "MATCH (u:URL {url: $url})-[:FOLLOWED_BY*1..3]-(related:URL)
                      RETURN DISTINCT related.url AS url
                      LIMIT $limit";

        let mut params = HashMap::new();
        params.insert("url".to_string(), json!(url));
        params.insert("limit".to_string(), json!(limit));

        let result = self.execute_query(cypher, params).await?;

        // Parse related URLs
        let urls = Self::parse_url_results(result)?;
        Ok(urls)
    }
}

impl FalkorDB {
    fn parse_search_results(result: serde_json::Value) -> Result<Vec<SearchResult>> {
        // Parse FalkorDB response into SearchResult objects
        // This will depend on the exact format FalkorDB returns
        let mut results = Vec::new();

        if let Some(rows) = result.get("results").and_then(|r| r.as_array()) {
            for row in rows {
                if let Some(values) = row.as_array() {
                    let search_result = SearchResult {
                        url: values.get(0).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        title: values.get(1).and_then(|v| v.as_str()).map(String::from),
                        visit_time: values.get(2)
                            .and_then(|v| v.as_str())
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(chrono::Utc::now),
                        visit_count: values.get(3).and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                        related_urls: values.get(4)
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect())
                            .unwrap_or_default(),
                        relevance_score: 1.0, // Calculate based on your scoring logic
                        browser_source: "".to_string(), // Add from query results
                        domain: Self::extract_domain(
                            values.get(0).and_then(|v| v.as_str()).unwrap_or("")
                        ),
                        // New semantic fields
                        clean_site_name: None,
                        site_category: None,
                        key_topics: None,
                        summary: None,
                    };
                    results.push(search_result);
                }
            }
        }

        Ok(results)
    }

    fn parse_domain_results(result: serde_json::Value) -> Result<Vec<String>> {
        let mut domains = Vec::new();

        if let Some(rows) = result.get("results").and_then(|r| r.as_array()) {
            for row in rows {
                if let Some(domain) = row.get(0).and_then(|v| v.as_str()) {
                    domains.push(domain.to_string());
                }
            }
        }

        Ok(domains)
    }

    fn parse_url_results(result: serde_json::Value) -> Result<Vec<String>> {
        let mut urls = Vec::new();

        if let Some(rows) = result.get("results").and_then(|r| r.as_array()) {
            for row in rows {
                if let Some(url) = row.get(0).and_then(|v| v.as_str()) {
                    urls.push(url.to_string());
                }
            }
        }

        Ok(urls)
    }
}