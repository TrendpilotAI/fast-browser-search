use crate::db::{SearchQuery, SearchResult};
use crate::search::SearchEngine;
use anyhow::Result;
use axum::{
    extract::{Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};

pub struct ApiServer {
    search_engine: Arc<SearchEngine>,
    port: u16,
}

impl ApiServer {
    pub fn new(search_engine: Arc<SearchEngine>, port: u16) -> Self {
        Self {
            search_engine,
            port,
        }
    }

    pub async fn run(self) -> Result<()> {
        let app = self.build_router();
        let addr = format!("0.0.0.0:{}", self.port);

        tracing::info!("API server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    fn build_router(self) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Router::new()
            .route("/api/search", post(search_handler))
            .route("/api/suggest", get(suggest_handler))
            .route("/api/popular", get(popular_handler))
            .route("/api/domains", get(domains_handler))
            .route("/api/related", get(related_handler))
            .route("/api/index", post(index_handler))
            .route("/ws", get(websocket_handler))
            .route("/health", get(health_handler))
            .layer(cors)
            .with_state(Arc::new(self.search_engine))
    }
}

// Request/Response types
#[derive(Debug, Deserialize)]
struct SearchRequest {
    query: String,
    limit: Option<usize>,
    offset: Option<usize>,
    browsers: Option<Vec<String>>,
    date_from: Option<String>,
    date_to: Option<String>,
    domains: Option<Vec<String>>,
    session_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
    total: usize,
    query_time_ms: u64,
}

#[derive(Debug, Deserialize)]
struct SuggestRequest {
    query: String,
}

#[derive(Debug, Serialize)]
struct SuggestResponse {
    suggestions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RelatedRequest {
    url: String,
    limit: Option<usize>,
}

// Handlers
async fn search_handler(
    State(engine): State<Arc<Arc<SearchEngine>>>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();

    let query = SearchQuery {
        query: req.query,
        limit: req.limit.unwrap_or(20),
        offset: req.offset.unwrap_or(0),
        browsers: req.browsers,
        date_from: req.date_from.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        date_to: req.date_to.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        domains: req.domains,
    };

    match engine.search(query, req.session_id).await {
        Ok(results) => {
            let response = SearchResponse {
                total: results.len(),
                results,
                query_time_ms: start.elapsed().as_millis() as u64,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Search error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn suggest_handler(
    State(engine): State<Arc<Arc<SearchEngine>>>,
    Query(req): Query<SuggestRequest>,
) -> impl IntoResponse {
    match engine.get_suggestions(&req.query).await {
        Ok(suggestions) => {
            let response = SuggestResponse { suggestions };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Suggest error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn popular_handler(
    State(engine): State<Arc<Arc<SearchEngine>>>,
) -> impl IntoResponse {
    match engine.get_popular_urls(20).await {
        Ok(urls) => {
            let response = json!({
                "popular": urls.into_iter().map(|(url, count)| {
                    json!({ "url": url, "visits": count })
                }).collect::<Vec<_>>()
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Popular URLs error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn domains_handler(
    State(engine): State<Arc<Arc<SearchEngine>>>,
) -> impl IntoResponse {
    match engine.get_domains().await {
        Ok(domains) => {
            (StatusCode::OK, Json(json!({ "domains": domains }))).into_response()
        }
        Err(e) => {
            tracing::error!("Domains error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn related_handler(
    State(engine): State<Arc<Arc<SearchEngine>>>,
    Query(req): Query<RelatedRequest>,
) -> impl IntoResponse {
    let limit = req.limit.unwrap_or(10);

    match engine.get_related_urls(&req.url, limit).await {
        Ok(urls) => {
            (StatusCode::OK, Json(json!({ "related": urls }))).into_response()
        }
        Err(e) => {
            tracing::error!("Related URLs error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn index_handler(
    State(engine): State<Arc<Arc<SearchEngine>>>,
) -> impl IntoResponse {
    tokio::spawn(async move {
        if let Err(e) = engine.index_all_browsers().await {
            tracing::error!("Indexing error: {}", e);
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(json!({ "message": "Indexing started in background" })),
    )
        .into_response()
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "healthy" }))).into_response()
}

// WebSocket handler for real-time search
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(engine): State<Arc<Arc<SearchEngine>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, engine))
}

async fn handle_websocket(mut socket: WebSocket, engine: Arc<Arc<SearchEngine>>) {
    tracing::info!("WebSocket connection established");

    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(text) => {
                    if let Ok(req) = serde_json::from_str::<SearchRequest>(&text) {
                        let query = SearchQuery {
                            query: req.query.clone(),
                            limit: req.limit.unwrap_or(20),
                            offset: req.offset.unwrap_or(0),
                            browsers: req.browsers,
                            date_from: req.date_from.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                                .map(|dt| dt.with_timezone(&chrono::Utc)),
                            date_to: req.date_to.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                                .map(|dt| dt.with_timezone(&chrono::Utc)),
                            domains: req.domains,
                        };

                        match engine.search(query, req.session_id).await {
                            Ok(results) => {
                                let response = json!({
                                    "type": "search_results",
                                    "results": results,
                                });

                                if socket
                                    .send(Message::Text(response.to_string()))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            Err(e) => {
                                let error = json!({
                                    "type": "error",
                                    "message": e.to_string(),
                                });

                                if socket
                                    .send(Message::Text(error.to_string()))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    }

    tracing::info!("WebSocket connection closed");
}