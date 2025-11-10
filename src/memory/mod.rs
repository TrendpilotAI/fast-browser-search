use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

pub struct MemoryManager {
    zep_client: ZepClient,
    graphiti_client: GraphitiClient,
}

impl MemoryManager {
    pub fn new(zep_url: &str, graphiti_url: &str, api_key: Option<String>) -> Result<Self> {
        Ok(Self {
            zep_client: ZepClient::new(zep_url, api_key.clone())?,
            graphiti_client: GraphitiClient::new(graphiti_url, api_key)?,
        })
    }

    pub async fn add_search_memory(&self, session_id: &str, query: &str, results: &[String]) -> Result<()> {
        // Store search interaction in Zep
        self.zep_client.add_memory(session_id, query, results).await?;

        // Update knowledge graph in Graphiti
        self.graphiti_client.update_graph(session_id, query, results).await?;

        Ok(())
    }

    pub async fn get_context(&self, session_id: &str, query: &str) -> Result<ConversationContext> {
        let zep_context = self.zep_client.get_context(session_id).await?;
        let graph_insights = self.graphiti_client.get_insights(session_id, query).await?;

        Ok(ConversationContext {
            recent_searches: zep_context.recent_searches,
            related_topics: graph_insights.related_topics,
            user_preferences: zep_context.preferences,
            suggested_queries: graph_insights.suggested_queries,
        })
    }
}

// Zep Memory Client
pub struct ZepClient {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl ZepClient {
    pub fn new(base_url: &str, api_key: Option<String>) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            api_key,
        })
    }

    pub async fn add_memory(&self, session_id: &str, query: &str, results: &[String]) -> Result<()> {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(key) = &self.api_key {
            headers.insert("Authorization", format!("Bearer {}", key).parse()?);
        }

        let memory = json!({
            "session_id": session_id,
            "messages": [
                {
                    "role": "user",
                    "content": query,
                    "metadata": {
                        "type": "search_query"
                    }
                },
                {
                    "role": "assistant",
                    "content": format!("Found {} results", results.len()),
                    "metadata": {
                        "type": "search_results",
                        "urls": results
                    }
                }
            ]
        });

        self.client
            .post(format!("{}/sessions/{}/memory", self.base_url, session_id))
            .headers(headers)
            .json(&memory)
            .send()
            .await
            .context("Failed to add memory to Zep")?;

        Ok(())
    }

    pub async fn get_context(&self, session_id: &str) -> Result<ZepContext> {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(key) = &self.api_key {
            headers.insert("Authorization", format!("Bearer {}", key).parse()?);
        }

        let response = self.client
            .get(format!("{}/sessions/{}/memory", self.base_url, session_id))
            .headers(headers)
            .send()
            .await
            .context("Failed to get context from Zep")?;

        let context: ZepContext = response.json().await?;
        Ok(context)
    }
}

// Graphiti Knowledge Graph Client
pub struct GraphitiClient {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl GraphitiClient {
    pub fn new(base_url: &str, api_key: Option<String>) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            api_key,
        })
    }

    pub async fn update_graph(&self, session_id: &str, query: &str, results: &[String]) -> Result<()> {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(key) = &self.api_key {
            headers.insert("Authorization", format!("Bearer {}", key).parse()?);
        }

        let graph_update = json!({
            "session_id": session_id,
            "entities": [
                {
                    "type": "search_query",
                    "value": query,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            ],
            "relationships": results.iter().map(|url| {
                json!({
                    "from": query,
                    "to": url,
                    "type": "searched_for"
                })
            }).collect::<Vec<_>>()
        });

        self.client
            .post(format!("{}/graph/update", self.base_url))
            .headers(headers)
            .json(&graph_update)
            .send()
            .await
            .context("Failed to update Graphiti graph")?;

        Ok(())
    }

    pub async fn get_insights(&self, session_id: &str, query: &str) -> Result<GraphInsights> {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(key) = &self.api_key {
            headers.insert("Authorization", format!("Bearer {}", key).parse()?);
        }

        let request = json!({
            "session_id": session_id,
            "query": query,
            "depth": 3
        });

        let response = self.client
            .post(format!("{}/graph/insights", self.base_url))
            .headers(headers)
            .json(&request)
            .send()
            .await
            .context("Failed to get insights from Graphiti")?;

        let insights: GraphInsights = response.json().await?;
        Ok(insights)
    }
}

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub recent_searches: Vec<String>,
    pub related_topics: Vec<String>,
    pub user_preferences: HashMap<String, String>,
    pub suggested_queries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZepContext {
    pub recent_searches: Vec<String>,
    pub preferences: HashMap<String, String>,
    pub session_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphInsights {
    pub related_topics: Vec<String>,
    pub suggested_queries: Vec<String>,
    pub entity_connections: Vec<EntityConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityConnection {
    pub from: String,
    pub to: String,
    pub relationship: String,
    pub strength: f32,
}