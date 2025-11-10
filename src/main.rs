mod api;
mod browser;
mod db;
mod memory;
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

    tracing::info!("Starting Fast Browser History Search...");

    // Get configuration from environment variables or use defaults
    let falkor_host = std::env::var("FALKOR_HOST").unwrap_or_else(|_| "localhost".to_string());
    let falkor_port: u16 = std::env::var("FALKOR_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(6379);

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6380".to_string());

    let zep_url = std::env::var("ZEP_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let graphiti_url = std::env::var("GRAPHITI_URL").unwrap_or_else(|_| "http://localhost:8001".to_string());

    let api_key = std::env::var("API_KEY").ok();
    let api_port: u16 = std::env::var("API_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    // Initialize search engine
    let search_engine = Arc::new(
        search::SearchEngine::new(
            &falkor_host,
            falkor_port,
            &redis_url,
            &zep_url,
            &graphiti_url,
            api_key,
        )
        .await?,
    );

    // Start initial indexing in the background
    let indexing_engine = search_engine.clone();
    tokio::spawn(async move {
        tracing::info!("Starting initial browser history indexing...");
        if let Err(e) = indexing_engine.index_all_browsers().await {
            tracing::error!("Initial indexing failed: {}", e);
        } else {
            tracing::info!("Initial indexing completed successfully");
        }
    });

    // Start file watcher for auto-updates
    let watcher_engine = search_engine.clone();
    tokio::spawn(async move {
        if let Err(e) = watch_browser_histories(watcher_engine).await {
            tracing::error!("File watcher error: {}", e);
        }
    });

    // Start API server
    let api_server = api::ApiServer::new(search_engine, api_port);

    tracing::info!("Fast Browser History Search is ready!");
    tracing::info!("API Server: http://localhost:{}", api_port);
    tracing::info!("WebSocket: ws://localhost:{}/ws", api_port);

    api_server.run().await?;

    Ok(())
}

async fn watch_browser_histories(engine: Arc<search::SearchEngine>) -> Result<()> {
    use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(10)),
    )?;

    // Get all browser history paths to watch
    let paths = browser::find_browser_history_files();

    for (browser_type, path) in &paths {
        if path.exists() {
            tracing::info!("Watching {:?} history at {:?}", browser_type, path);
            watcher.watch(path.as_path(), RecursiveMode::NonRecursive)?;
        }
    }

    // Handle file change events
    loop {
        if let Ok(event) = rx.recv() {
            match event.kind {
                EventKind::Modify(_) | EventKind::Create(_) => {
                    tracing::debug!("Browser history file changed, re-indexing...");

                    // Wait a moment for the file to be fully written
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    if let Err(e) = engine.index_all_browsers().await {
                        tracing::error!("Re-indexing error: {}", e);
                    }
                }
                _ => {}
            }
        }
    }
}
