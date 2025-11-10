mod api;
mod browser;
mod db;
mod memory;
mod nlp;
mod search;

use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "browser_history_search=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Fast Browser History Search (Semantic Mode)...");

    let api_port: u16 = std::env::var("API_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    // Initialize semantic search engine
    let search_engine = Arc::new(
        search::semantic::SemanticSearchEngine::new().await?,
    );

    // Start initial indexing in the background
    let indexing_engine = search_engine.clone();
    tokio::spawn(async move {
        tracing::info!("Starting initial browser history indexing with semantic enrichment...");
        if let Err(e) = indexing_engine.index_all_browsers().await {
            tracing::error!("Initial indexing failed: {}", e);
        } else {
            tracing::info!("Initial indexing completed successfully with semantic data");
        }
    });

    // Create semantic API server
    let api_server = SemanticApiServer::new(search_engine, api_port);

    tracing::info!("Fast Browser History Search is ready!");
    tracing::info!("API Server: http://localhost:{}", api_port);
    tracing::info!("WebSocket: ws://localhost:{}/ws", api_port);

    api_server.run().await?;

    Ok(())
}

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

pub struct SemanticApiServer {
    search_engine: Arc<search::semantic::SemanticSearchEngine>,
    port: u16,
}

impl SemanticApiServer {
    pub fn new(search_engine: Arc<search::semantic::SemanticSearchEngine>, port: u16) -> Self {
        Self {
            search_engine,
            port,
        }
    }

    pub async fn run(self) -> Result<()> {
        let port = self.port;
        let app = self.build_router();
        let addr = format!("0.0.0.0:{}", port);

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
            // Standard endpoints (backward compatible)
            .route("/api/search", post(search_handler))
            .route("/api/suggest", get(suggest_handler))
            .route("/api/popular", get(popular_handler))
            .route("/api/domains", get(domains_handler))
            .route("/api/related", get(related_handler))
            .route("/api/index", post(index_handler))
            // Semantic-specific endpoints
            .route("/api/semantic/search", post(semantic_search_handler))
            .route("/api/semantic/similar", get(semantic_similar_handler))
            .route("/api/semantic/topics", get(semantic_topics_handler))
            .route("/api/semantic/sites", get(semantic_sites_handler))
            // Health check
            .route("/health", get(health_handler))
            .layer(cors)
            .with_state(Arc::new(self.search_engine))
    }
}

#[derive(Debug, Deserialize)]
struct SearchRequest {
    query: String,
    limit: Option<usize>,
    offset: Option<usize>,
    browsers: Option<Vec<String>>,
    date_from: Option<String>,
    date_to: Option<String>,
    domains: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    results: Vec<db::SearchResult>,
    total: usize,
    query_time_ms: u64,
}

async fn search_handler(
    State(engine): State<Arc<Arc<search::semantic::SemanticSearchEngine>>>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();

    let query = db::SearchQuery {
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

    match engine.search(query).await {
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

#[derive(Debug, Deserialize)]
struct SuggestRequest {
    query: String,
}

async fn suggest_handler(
    State(engine): State<Arc<Arc<search::semantic::SemanticSearchEngine>>>,
    Query(req): Query<SuggestRequest>,
) -> impl IntoResponse {
    match engine.get_suggestions(&req.query).await {
        Ok(suggestions) => {
            (StatusCode::OK, Json(json!({ "suggestions": suggestions }))).into_response()
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
    State(engine): State<Arc<Arc<search::semantic::SemanticSearchEngine>>>,
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
    State(engine): State<Arc<Arc<search::semantic::SemanticSearchEngine>>>,
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

#[derive(Debug, Deserialize)]
struct RelatedRequest {
    url: String,
    limit: Option<usize>,
}

async fn related_handler(
    State(engine): State<Arc<Arc<search::semantic::SemanticSearchEngine>>>,
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
    State(engine): State<Arc<Arc<search::semantic::SemanticSearchEngine>>>,
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

// Semantic-specific endpoints

#[derive(Debug, Deserialize)]
struct SemanticSearchRequest {
    query: String,
    limit: Option<usize>,
    use_semantic: Option<bool>,
}

async fn semantic_search_handler(
    State(engine): State<Arc<Arc<search::semantic::SemanticSearchEngine>>>,
    Json(req): Json<SemanticSearchRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();

    let query = db::SearchQuery {
        query: req.query,
        limit: req.limit.unwrap_or(20),
        offset: 0,
        browsers: None,
        date_from: None,
        date_to: None,
        domains: None,
    };

    let use_semantic = req.use_semantic.unwrap_or(true);

    match engine.semantic_search(query, use_semantic).await {
        Ok(results) => {
            let response = json!({
                "results": results,
                "total": results.len(),
                "query_time_ms": start.elapsed().as_millis() as u64,
                "semantic": use_semantic,
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Semantic search error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
struct SimilarRequest {
    url: String,
    limit: Option<usize>,
}

async fn semantic_similar_handler(
    State(engine): State<Arc<Arc<search::semantic::SemanticSearchEngine>>>,
    Query(req): Query<SimilarRequest>,
) -> impl IntoResponse {
    let limit = req.limit.unwrap_or(10);

    match engine.find_similar(&req.url, limit).await {
        Ok(results) => {
            (StatusCode::OK, Json(json!({ "similar": results }))).into_response()
        }
        Err(e) => {
            tracing::error!("Semantic similar error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn semantic_topics_handler(
    State(engine): State<Arc<Arc<search::semantic::SemanticSearchEngine>>>,
) -> impl IntoResponse {
    match engine.get_topics().await {
        Ok(topics) => {
            (StatusCode::OK, Json(json!({ "topics": topics }))).into_response()
        }
        Err(e) => {
            tracing::error!("Topics error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn semantic_sites_handler(
    State(engine): State<Arc<Arc<search::semantic::SemanticSearchEngine>>>,
) -> impl IntoResponse {
    match engine.get_sites_summary().await {
        Ok(sites) => {
            (StatusCode::OK, Json(json!({ "sites": sites }))).into_response()
        }
        Err(e) => {
            tracing::error!("Sites summary error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}