use anyhow::{Context, Result};
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::browser::HistoryEntry;

/// Gmail API client for fast email indexing
pub struct GmailClient {
    access_token: Arc<RwLock<Option<String>>>,
    refresh_token: Option<String>,
    client_id: String,
    client_secret: String,
    http_client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmailMessage {
    pub id: String,
    pub thread_id: String,
    pub snippet: String,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub to: Vec<String>,
    pub date: DateTime<Utc>,
    pub body_text: String,
    pub body_html: Option<String>,
    pub labels: Vec<String>,
}

impl GmailClient {
    /// Create a new Gmail client
    pub fn new(client_id: String, client_secret: String, refresh_token: Option<String>) -> Self {
        Self {
            access_token: Arc::new(RwLock::new(None)),
            refresh_token,
            client_id,
            client_secret,
            http_client: reqwest::Client::new(),
        }
    }

    /// Set access token (from OAuth2 flow)
    pub async fn set_access_token(&self, token: String) {
        *self.access_token.write().await = Some(token);
    }

    /// Get valid access token, refreshing if needed
    async fn get_access_token(&self) -> Result<String> {
        // Check if we have a valid token
        if let Some(token) = self.access_token.read().await.as_ref() {
            // TODO: Validate token hasn't expired
            return Ok(token.clone());
        }

        // Refresh token if available
        if let Some(refresh_token) = &self.refresh_token {
            self.refresh_access_token(refresh_token).await?;
            if let Some(token) = self.access_token.read().await.as_ref() {
                return Ok(token.clone());
            }
        }

        anyhow::bail!("No valid access token available. Please authenticate first.");
    }

    /// Refresh access token using refresh token
    async fn refresh_access_token(&self, refresh_token: &str) -> Result<()> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let response = self
            .http_client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await
            .context("Failed to refresh access token")?;

        let token_response: TokenResponse = response
            .json()
            .await
            .context("Failed to parse token response")?;

        self.set_access_token(token_response.access_token).await;
        Ok(())
    }

    /// Fetch messages in batches for ultra-fast indexing
    pub async fn fetch_messages_batch(
        &self,
        max_results: usize,
        query: Option<&str>,
    ) -> Result<Vec<GmailMessage>> {
        let access_token = self.get_access_token().await?;
        
        let mut all_messages = Vec::new();
        let mut page_token: Option<String> = None;
        let batch_size = 500; // Gmail API max is 500 per request

        loop {
            let mut url = format!(
                "https://gmail.googleapis.com/gmail/v1/users/me/messages?maxResults={}",
                batch_size.min(max_results - all_messages.len())
            );

            if let Some(q) = query {
                url.push_str(&format!("&q={}", urlencoding::encode(q)));
            }

            if let Some(ref token) = page_token {
                url.push_str(&format!("&pageToken={}", token));
            }

            let response = self
                .http_client
                .get(&url)
                .bearer_auth(&access_token)
                .send()
                .await
                .context("Failed to fetch message list")?;

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                anyhow::bail!("Gmail API error: {}", error_text);
            }

            let message_list: MessageListResponse = response
                .json()
                .await
                .context("Failed to parse message list")?;

            // Fetch full message details in parallel batches
            let message_ids: Vec<String> = message_list
                .messages
                .iter()
                .map(|m| m.id.clone())
                .collect();

            if message_ids.is_empty() {
                break;
            }

            // Fetch messages in parallel batches of 10
            let message_futures: Vec<_> = message_ids
                .chunks(10)
                .map(|chunk| self.fetch_messages_parallel(chunk, &access_token))
                .collect();

            let batch_results: Vec<Result<Vec<GmailMessage>>> =
                futures::future::join_all(message_futures).await;

            for result in batch_results {
                match result {
                    Ok(messages) => all_messages.extend(messages),
                    Err(e) => tracing::warn!("Failed to fetch message batch: {}", e),
                }
            }

            if all_messages.len() >= max_results {
                break;
            }

            page_token = message_list.next_page_token;
            if page_token.is_none() {
                break;
            }
        }

        Ok(all_messages.into_iter().take(max_results).collect())
    }

    /// Fetch multiple messages in parallel
    async fn fetch_messages_parallel(
        &self,
        message_ids: &[String],
        access_token: &str,
    ) -> Result<Vec<GmailMessage>> {
        let futures: Vec<_> = message_ids
            .iter()
            .map(|id| self.fetch_message(id, access_token))
            .collect();

        let results = futures::future::join_all(futures).await;
        let mut messages = Vec::new();

        for result in results {
            match result {
                Ok(Some(msg)) => messages.push(msg),
                Ok(None) => {}
                Err(e) => tracing::debug!("Failed to fetch message: {}", e),
            }
        }

        Ok(messages)
    }

    /// Fetch a single message by ID
    async fn fetch_message(
        &self,
        message_id: &str,
        access_token: &str,
    ) -> Result<Option<GmailMessage>> {
        let url = format!(
            "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}?format=full",
            message_id
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("Failed to fetch message")?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let gmail_msg: GmailApiMessage = response
            .json()
            .await
            .context("Failed to parse message")?;

        Ok(Some(gmail_msg.into()))
    }

    /// Get OAuth2 authorization URL
    pub fn get_authorization_url(&self, redirect_uri: &str) -> String {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("redirect_uri", redirect_uri),
            ("response_type", "code"),
            ("scope", "https://www.googleapis.com/auth/gmail.readonly"),
            ("access_type", "offline"),
            ("prompt", "consent"),
        ];

        let mut url = "https://accounts.google.com/o/oauth2/v2/auth?".to_string();
        let query_string: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect();
        url.push_str(&query_string.join("&"));

        url
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<TokenResponse> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ];

        let response = self
            .http_client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await
            .context("Failed to exchange authorization code")?;

        let token_response: TokenResponse = response
            .json()
            .await
            .context("Failed to parse token response")?;

        self.set_access_token(token_response.access_token.clone()).await;

        Ok(token_response)
    }
}

/// Convert Gmail messages to browser history entries for unified search
impl From<GmailMessage> for HistoryEntry {
    fn from(msg: GmailMessage) -> Self {
        // Create a Gmail URL-like identifier
        let url = format!("gmail://message/{}", msg.id);
        
        // Use subject or snippet as title
        let title = msg.subject.clone().or_else(|| {
            if msg.snippet.len() > 100 {
                Some(format!("{}...", &msg.snippet[..100]))
            } else {
                Some(msg.snippet.clone())
            }
        });

        HistoryEntry {
            id: format!("gmail_{}", msg.id),
            url,
            title,
            visit_time: msg.date,
            visit_count: 1,
            typed_count: 0,
            last_visit_time: msg.date,
            hidden: false,
            favicon_url: Some("https://www.gmail.com/favicon.ico".to_string()),
        }
    }
}

/// Gmail API response types
#[derive(Debug, Deserialize)]
struct MessageListResponse {
    messages: Vec<MessageReference>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MessageReference {
    id: String,
    thread_id: String,
}

#[derive(Debug, Deserialize)]
struct GmailApiMessage {
    id: String,
    thread_id: String,
    snippet: String,
    payload: MessagePayload,
    internal_date: String,
    label_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MessagePayload {
    headers: Vec<Header>,
    parts: Option<Vec<MessagePart>>,
    body: Option<MessageBody>,
}

#[derive(Debug, Deserialize)]
struct Header {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct MessagePart {
    mime_type: String,
    body: MessageBody,
    parts: Option<Vec<MessagePart>>,
}

#[derive(Debug, Deserialize)]
struct MessageBody {
    data: Option<String>,
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "refresh_token")]
    pub refresh_token: Option<String>,
    #[serde(rename = "expires_in")]
    pub expires_in: Option<u64>,
    #[serde(rename = "token_type")]
    pub token_type: Option<String>,
}

impl From<GmailApiMessage> for GmailMessage {
    fn from(api_msg: GmailApiMessage) -> Self {
        let headers_map: HashMap<String, String> = api_msg
            .payload
            .headers
            .iter()
            .map(|h| (h.name.to_lowercase(), h.value.clone()))
            .collect();

        let subject = headers_map.get("subject").cloned();
        let from = headers_map.get("from").cloned();
        let to = headers_map
            .get("to")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let date = headers_map
            .get("date")
            .and_then(|d| {
                // Parse RFC2822 date
                chrono::DateTime::parse_from_rfc2822(d)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            })
            .or_else(|| {
                // Fallback to internal date
                api_msg
                    .internal_date
                    .parse::<i64>()
                    .ok()
                    .map(|ts| DateTime::from_timestamp(ts / 1000, 0).unwrap_or_else(Utc::now))
            })
            .unwrap_or_else(Utc::now);

        // Extract body text
        let body_text = extract_text_from_payload(&api_msg.payload);
        let body_html = extract_html_from_payload(&api_msg.payload);

        GmailMessage {
            id: api_msg.id,
            thread_id: api_msg.thread_id,
            snippet: api_msg.snippet,
            subject,
            from,
            to,
            date,
            body_text,
            body_html,
            labels: api_msg.label_ids,
        }
    }
}

fn extract_text_from_payload(payload: &MessagePayload) -> String {
    if let Some(ref parts) = payload.parts {
        for part in parts {
            if part.mime_type == "text/plain" {
                if let Some(ref body) = part.body.data {
                    let input = body.replace(' ', "+").replace('-', "+").replace('_', "/");
                    if let Ok(bytes) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(input) {
                        if let Ok(text) = String::from_utf8(bytes) {
                            return text;
                        }
                    }
                }
            }
            // Recursively check nested parts
            if let Some(ref nested_parts) = part.parts {
                for nested in nested_parts {
                    if nested.mime_type == "text/plain" {
                        if let Some(ref body) = nested.body.data {
                            if let Ok(decoded) = base64::engine::general_purpose::URL_SAFE_NO_PAD
                                .decode(body.replace(' ', "+"))
                            {
                                return String::from_utf8_lossy(&decoded).to_string();
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to body data
    if let Some(ref body) = payload.body {
        if let Some(ref data) = body.data {
                let input = data.replace(' ', "+").replace('-', "+").replace('_', "/");
                if let Ok(bytes) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(input) {
                    if let Ok(text) = String::from_utf8(bytes) {
                        return text;
                    }
                }
        }
    }

    String::new()
}

fn extract_html_from_payload(payload: &MessagePayload) -> Option<String> {
    if let Some(ref parts) = payload.parts {
        for part in parts {
            if part.mime_type == "text/html" {
                if let Some(ref body) = part.body.data {
                    let input = body.replace(' ', "+").replace('-', "+").replace('_', "/");
                    if let Ok(bytes) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(input) {
                        if let Ok(text) = String::from_utf8(bytes) {
                            return Some(text);
                        }
                    }
                }
            }
        }
    }
    None
}


