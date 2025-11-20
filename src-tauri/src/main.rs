#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use browser_history_search::{
    search::semantic::SemanticSearchEngine,
    db::{SearchQuery, SearchResult},
    gmail::GmailClient,
};
use std::sync::Arc;
use tauri::{Manager, State, Window};
use tokio::sync::Mutex;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenResponse,
    TokenUrl, CsrfToken, Scope, AuthorizationCode,
};
use url::Url;
use tokio::net::TcpListener;
use warp::Filter;

// Wrapper struct to hold the engine
struct SearchState(Arc<SemanticSearchEngine>);

// Wrapper for Gmail Client
struct GmailState(Arc<Mutex<Option<Arc<GmailClient>>>>);

#[tauri::command]
async fn search(
    state: State<'_, SearchState>,
    query: String,
    limit: Option<usize>,
    offset: Option<usize>,
    use_semantic: Option<bool>,
) -> Result<Vec<SearchResult>, String> {
    let engine = &state.0;
    
    let search_query = SearchQuery {
        query,
        limit: limit.unwrap_or(20),
        offset: offset.unwrap_or(0),
        browsers: None,
        date_from: None,
        date_to: None,
        domains: None,
    };

    let use_semantic = use_semantic.unwrap_or(true);

    engine.semantic_search(search_query, use_semantic)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn suggest(
    state: State<'_, SearchState>,
    query: String,
) -> Result<Vec<String>, String> {
    let engine = &state.0;
    engine.get_suggestions(&query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn related(
    state: State<'_, SearchState>,
    url: String,
    limit: Option<usize>,
) -> Result<Vec<String>, String> {
    let engine = &state.0;
    engine.get_related_urls(&url, limit.unwrap_or(10))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn index_history(state: State<'_, SearchState>) -> Result<String, String> {
    let engine = state.0.clone();
    
    // Run indexing in background
    tokio::spawn(async move {
        if let Err(e) = engine.index_all_browsers().await {
            tracing::error!("Indexing failed: {}", e);
        }
    });

    Ok("Indexing started".to_string())
}

#[tauri::command]
async fn get_popular(
    state: State<'_, SearchState>,
    limit: Option<isize>,
) -> Result<Vec<(String, i64)>, String> {
    let engine = &state.0;
    engine.get_popular_urls(limit.unwrap_or(20))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_domains(state: State<'_, SearchState>) -> Result<Vec<String>, String> {
    let engine = &state.0;
    engine.get_domains()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn connect_google(
    window: Window,
    gmail_state: State<'_, GmailState>,
    search_state: State<'_, SearchState>,
) -> Result<String, String> {
    // Load credentials from environment variables
    let client_id_str = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| "GOOGLE_CLIENT_ID environment variable not set".to_string())?;
    let client_secret_str = std::env::var("GOOGLE_CLIENT_SECRET")
        .map_err(|_| "GOOGLE_CLIENT_SECRET environment variable not set".to_string())?;
    
    let client_id = ClientId::new(client_id_str);
    let client_secret = ClientSecret::new(client_secret_str);
    
    let listener = TcpListener::bind("127.0.0.1:0").await.map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let redirect_url = format!("http://localhost:{}/callback", port);

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .expect("Invalid token endpoint URL");

    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(RedirectUrl::new(redirect_url.clone()).expect("Invalid redirect URL"));

    let (authorize_url, _csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("https://www.googleapis.com/auth/gmail.readonly".to_string()))
        .url();

    let gmail_state = gmail_state.0.clone();
    let engine = search_state.0.clone();
    let client_clone = client.clone();

    // Spawn server to handle callback
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let tx = Arc::new(Mutex::new(Some(tx)));
    let tx_filter = warp::any().map(move || tx.clone());

    tokio::spawn(async move {
        let callback = warp::path("callback")
            .and(warp::query::<std::collections::HashMap<String, String>>())
            .and(tx_filter)
            .then(|params: std::collections::HashMap<String, String>, tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<String>>>>| async move {
                if let Some(code) = params.get("code") {
                    if let Some(tx) = tx.lock().await.take() {
                        let _ = tx.send(code.clone());
                    }
                    "Authentication successful! You can close this window."
                } else {
                    "Authentication failed."
                }
            });

        warp::serve(callback).run(([127, 0, 0, 1], port)).await;
    });

    // Spawn task to handle code exchange
    tokio::spawn(async move {
        if let Ok(code) = rx.await {
            tracing::info!("Received auth code, exchanging for token...");
            let token_result = client_clone
                .exchange_code(AuthorizationCode::new(code))
                .request_async(oauth2::reqwest::async_http_client)
                .await;

            match token_result {
                Ok(token) => {
                    let access_token = token.access_token().secret().clone();
                    let refresh_token = token.refresh_token().map(|rt| rt.secret().clone());
                    
                    // Create Gmail client with credentials from environment
                    let client_id_str = std::env::var("GOOGLE_CLIENT_ID")
                        .unwrap_or_else(|_| String::new());
                    let client_secret_str = std::env::var("GOOGLE_CLIENT_SECRET")
                        .unwrap_or_else(|_| String::new());
                    
                    let gmail_client = GmailClient::new(
                        client_id_str,
                        client_secret_str,
                        refresh_token,
                    );
                    let gmail_client_arc = Arc::new(gmail_client);
                    
                    // Set access token
                    gmail_client_arc.set_access_token(access_token).await;
                    
                    // Update state
                    *gmail_state.lock().await = Some(gmail_client_arc.clone());
                    
                    // Update search engine
                    engine.set_gmail_client(gmail_client_arc.clone()).await;
                    
                    // Trigger indexing immediately
                    let engine_clone = engine.clone();
                    tokio::spawn(async move {
                        tracing::info!("Starting initial Gmail indexing...");
                        if let Err(e) = engine_clone.index_all_browsers().await {
                            tracing::error!("Gmail indexing error: {}", e);
                        }
                    });
                    
                    tracing::info!("Gmail connected and indexing started!");
                },
                Err(e) => tracing::error!("Failed to exchange token: {}", e),
            }
        }
    });

    // Open browser
    if let Err(e) = open::that(authorize_url.to_string()) {
        return Err(format!("Failed to open browser: {}", e));
    }

    Ok("Authentication flow started".to_string())
}

#[tauri::command]
async fn gmail_status(
    gmail_state: State<'_, GmailState>,
) -> Result<bool, String> {
    let client = gmail_state.0.lock().await;
    Ok(client.is_some())
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .setup(|app| {
            // Initialize the search engine
            tauri::async_runtime::block_on(async {
                let engine = SemanticSearchEngine::new().await
                    .expect("Failed to initialize search engine");
                
                // Start initial indexing
                let engine = Arc::new(engine);
                let indexing_engine = engine.clone();
                tokio::spawn(async move {
                    if let Err(e) = indexing_engine.index_all_browsers().await {
                        tracing::error!("Initial indexing failed: {}", e);
                    }
                });

                app.manage(SearchState(engine));
                app.manage(GmailState(Arc::new(Mutex::new(None))));
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search,
            suggest,
            related,
            index_history,
            get_popular,
            get_domains,
            connect_google,
            gmail_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
