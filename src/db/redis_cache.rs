use super::{SearchQuery, SearchResult};
use anyhow::{Context, Result};
use redis::aio::{ConnectionManager, ConnectionManagerConfig};
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct RedisCache {
    conn: ConnectionManager,
    ttl_seconds: u64,
}

impl RedisCache {
    pub async fn new(redis_url: &str, ttl_seconds: u64) -> Result<Self> {
        let client = Client::open(redis_url)
            .context("Failed to create Redis client")?;

        let config = ConnectionManagerConfig::new()
            .set_connection_timeout(Duration::from_secs(5))
            .set_response_timeout(Duration::from_secs(2));

        let conn = ConnectionManager::new_with_config(client, config).await
            .context("Failed to connect to Redis")?;

        Ok(Self {
            conn,
            ttl_seconds,
        })
    }

    pub async fn get_search_results(&mut self, query: &SearchQuery) -> Result<Option<Vec<SearchResult>>> {
        let key = self.generate_cache_key(query);

        let cached: Option<String> = self.conn.get(&key).await?;

        if let Some(data) = cached {
            let results: Vec<SearchResult> = serde_json::from_str(&data)?;
            tracing::debug!("Cache hit for query: {:?}", query.query);
            Ok(Some(results))
        } else {
            tracing::debug!("Cache miss for query: {:?}", query.query);
            Ok(None)
        }
    }

    pub async fn set_search_results(&mut self, query: &SearchQuery, results: &[SearchResult]) -> Result<()> {
        let key = self.generate_cache_key(query);
        let data = serde_json::to_string(results)?;

        self.conn.set_ex(&key, data, self.ttl_seconds).await?;

        // Also store in a list of recent searches
        let recent_key = "recent_searches";
        self.conn.lpush(recent_key, &query.query).await?;
        self.conn.ltrim(recent_key, 0, 99).await?; // Keep only last 100 searches

        Ok(())
    }

    pub async fn get_recent_searches(&mut self) -> Result<Vec<String>> {
        let searches: Vec<String> = self.conn.lrange("recent_searches", 0, -1).await?;
        Ok(searches)
    }

    pub async fn increment_url_visit(&mut self, url: &str) -> Result<()> {
        let key = format!("url_visits:{}", url);
        let _: i64 = self.conn.incr(&key, 1).await?;
        Ok(())
    }

    pub async fn get_popular_urls(&mut self, limit: isize) -> Result<Vec<(String, i64)>> {
        // Get all url_visits:* keys
        let pattern = "url_visits:*";
        let keys: Vec<String> = self.conn.keys(pattern).await?;

        let mut url_scores = Vec::new();
        for key in keys.iter().take(limit as usize) {
            let url = key.strip_prefix("url_visits:").unwrap_or("");
            let score: Option<i64> = self.conn.get(key).await?;
            if let Some(score) = score {
                url_scores.push((url.to_string(), score));
            }
        }

        // Sort by score descending
        url_scores.sort_by(|a, b| b.1.cmp(&a.1));
        url_scores.truncate(limit as usize);

        Ok(url_scores)
    }

    pub async fn invalidate_search_cache(&mut self, pattern: &str) -> Result<()> {
        let keys: Vec<String> = self.conn.keys(format!("search:*{}*", pattern)).await?;
        for key in keys {
            self.conn.del(&key).await?;
        }
        Ok(())
    }

    fn generate_cache_key(&self, query: &SearchQuery) -> String {
        let mut key_parts = vec![
            format!("search:{}", query.query),
            format!("limit:{}", query.limit),
            format!("offset:{}", query.offset),
        ];

        if let Some(browsers) = &query.browsers {
            key_parts.push(format!("browsers:{}", browsers.join(",")));
        }

        if let Some(date_from) = query.date_from {
            key_parts.push(format!("from:{}", date_from.timestamp()));
        }

        if let Some(date_to) = query.date_to {
            key_parts.push(format!("to:{}", date_to.timestamp()));
        }

        if let Some(domains) = &query.domains {
            key_parts.push(format!("domains:{}", domains.join(",")));
        }

        key_parts.join(":")
    }
}

// Session cache for user preferences and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub session_id: String,
    pub last_searches: Vec<String>,
    pub preferred_browsers: Vec<String>,
    pub search_preferences: SearchPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPreferences {
    pub default_limit: usize,
    pub show_related: bool,
    pub group_by_domain: bool,
    pub sort_by_recency: bool,
}

impl Default for SearchPreferences {
    fn default() -> Self {
        Self {
            default_limit: 20,
            show_related: true,
            group_by_domain: false,
            sort_by_recency: true,
        }
    }
}

impl RedisCache {
    pub async fn get_session(&mut self, session_id: &str) -> Result<Option<UserSession>> {
        let key = format!("session:{}", session_id);
        let data: Option<String> = self.conn.get(&key).await?;

        if let Some(json) = data {
            let session: UserSession = serde_json::from_str(&json)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    pub async fn set_session(&mut self, session: &UserSession) -> Result<()> {
        let key = format!("session:{}", session.session_id);
        let data = serde_json::to_string(session)?;

        // Session expires after 24 hours
        self.conn.set_ex(&key, data, 86400).await?;
        Ok(())
    }
}