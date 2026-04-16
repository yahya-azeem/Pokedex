mod state;
mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::net::SocketAddr;
use std::path::PathBuf;

use state::AppState;
use pokedex_swarm::SwarmOrchestrator;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables from .env file
    if let Err(e) = dotenvy::dotenv() {
        tracing::warn!("Failed to load .env file: {}. Expecting environment variables to be set manually.", e);
    }

    // Resolve library path (relative to workspace root)
    let library_path = resolve_library_path();
    tracing::info!("Library path: {}", library_path.display());

    // Initialize the swarm orchestrator
    let orchestrator = SwarmOrchestrator::new(library_path).await
        .expect("Failed to initialize SwarmOrchestrator");

    tracing::info!(
        "Loaded {} personas from agency-agents library",
        orchestrator.catalog.count
    );

    let state = AppState::new(orchestrator);

    // Build the router
    let api_routes = Router::new()
        .route("/swarm", post(handlers::create_swarm))
        .route("/swarms", get(handlers::list_swarms))
        .route("/swarm/{id}", get(handlers::get_swarm).delete(handlers::cancel_swarm))
        .route("/personas", get(handlers::list_personas));

    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/ws", get(handlers::ws_handler))
        .nest("/api", api_routes)
        // Serve static frontend files from web/build
        .fallback_service(ServeDir::new(resolve_web_build_path()))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 5001));
    tracing::info!("ðŸ”´ Pokedex Swarm Server listening on http://{}", addr);
    tracing::info!("   API:       http://{}/api/", addr);
    tracing::info!("   WebSocket: ws://{}/ws", addr);
    tracing::info!("   Health:    http://{}/health", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Resolve the library path heuristically.
fn resolve_library_path() -> PathBuf {
    // Check POKEDEX_LIBRARY_PATH env var first
    if let Ok(path) = std::env::var("POKEDEX_LIBRARY_PATH") {
        return PathBuf::from(path);
    }

    // Try relative to current directory (workspace root)
    let cwd = std::env::current_dir().unwrap_or_default();
    let candidates = [
        cwd.join("library"),
        cwd.join("../library"),
        cwd.join("../../library"),
    ];

    for candidate in &candidates {
        if candidate.join("agency-agents").exists() {
            return candidate.clone();
        }
    }

    // Fallback
    cwd.join("library")
}

/// Resolve the web build directory for serving static files.
fn resolve_web_build_path() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_default();
    let candidates = [
        cwd.join("web/build"),
        cwd.join("../web/build"),
        cwd.join("../../web/build"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    cwd.join("web/build")
}
