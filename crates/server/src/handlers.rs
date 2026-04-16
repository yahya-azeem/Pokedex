use axum::{
    extract::{Path, State, ws::{WebSocket, WebSocketUpgrade, Message}},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;


use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateSwarmRequest {
    pub goal: String,
}

// ---------------------------------------------------------------------------
// REST Handlers
// ---------------------------------------------------------------------------

/// GET /health
pub async fn health_check() -> &'static str {
    "OK"
}

/// POST /api/swarm — Create and execute a new swarm
pub async fn create_swarm(
    State(state): State<AppState>,
    Json(payload): Json<CreateSwarmRequest>,
) -> impl IntoResponse {
    tracing::info!("Creating swarm with goal: {}", payload.goal);

    match state.orchestrator.create_swarm(payload.goal).await {
        Ok(swarm_id) => {
            // Spawn the execution pipeline in the background
            let orchestrator = state.orchestrator.clone();
            tokio::spawn(async move {
                if let Err(e) = orchestrator.execute_swarm(swarm_id).await {
                    tracing::error!("Swarm execution failed: {}", e);
                }
            });

            Json(json!({
                "status": "success",
                "swarm_id": swarm_id,
                "message": "Swarm creation started"
            }))
        }
        Err(e) => Json(json!({
            "status": "error",
            "message": format!("Failed to create swarm: {}", e)
        })),
    }
}

/// GET /api/swarm/:id — Get current swarm state
pub async fn get_swarm(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.orchestrator.get_swarm(id).await {
        Some(swarm) => Json(json!({
            "status": "success",
            "swarm": {
                "id": swarm.id,
                "goal": swarm.goal,
                "phase": format!("{}", swarm.phase),
                "agents": swarm.agents.iter().map(|a| a.summary()).collect::<Vec<_>>(),
                "manifest": swarm.manifest,
                "report": swarm.report,
                "created_at": swarm.created_at,
                "completed_at": swarm.completed_at,
            }
        })),
        None => Json(json!({
            "status": "error",
            "message": "Swarm not found"
        })),
    }
}

/// GET /api/swarms — List all swarms
pub async fn list_swarms(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let swarms = state.orchestrator.list_swarms().await;
    Json(json!({
        "status": "success",
        "swarms": swarms
    }))
}

/// DELETE /api/swarm/:id — Cancel a running swarm
pub async fn cancel_swarm(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.orchestrator.cancel_swarm(id).await {
        Ok(_) => Json(json!({
            "status": "success",
            "message": "Swarm cancelled"
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": format!("Failed to cancel swarm: {}", e)
        })),
    }
}

/// GET /api/personas — List all available personas from the catalog
pub async fn list_personas(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let catalog = &state.orchestrator.catalog;
    let personas: Vec<serde_json::Value> = catalog.personas.iter().map(|p| {
        json!({
            "name": p.name,
            "category": p.category,
            "filename": p.filename,
            "description": p.description,
            "keywords": p.keywords,
        })
    }).collect();

    Json(json!({
        "status": "success",
        "count": catalog.count,
        "categories": catalog.categories(),
        "personas": personas,
    }))
}

// ---------------------------------------------------------------------------
// WebSocket Handler
// ---------------------------------------------------------------------------

/// GET /ws — Upgrade to WebSocket for real-time swarm events
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    tracing::info!("New WebSocket connection established");

    let mut rx = state.event_tx.subscribe();

    // Send a welcome message
    let welcome = json!({
        "type": "connected",
        "data": { "message": "Connected to Pokedex Swarm" }
    });
    if socket.send(Message::Text(serde_json::to_string(&welcome).unwrap().into())).await.is_err() {
        return;
    }

    // Forward all broadcast events to the WebSocket client
    loop {
        tokio::select! {
            // Receive events from the broadcast channel and send to client
            event = rx.recv() => {
                match event {
                    Ok(swarm_event) => {
                        let json = serde_json::to_string(&swarm_event).unwrap_or_default();
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            tracing::info!("WebSocket client disconnected");
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("WebSocket client lagged {} events", n);
                    }
                    Err(_) => break,
                }
            }
            // Handle incoming messages from client (for future use)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        tracing::debug!("Received client message: {}", text);
                        // Future: handle client commands (e.g., cancel swarm, request status)
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!("WebSocket client disconnected");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}
