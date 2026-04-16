use std::sync::Arc;
use pokedex_swarm::SwarmOrchestrator;
use tokio::sync::broadcast;
use pokedex_swarm::SwarmEvent;

/// Shared application state available to all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub orchestrator: Arc<SwarmOrchestrator>,
    pub event_tx: broadcast::Sender<SwarmEvent>,
}

impl AppState {
    pub fn new(orchestrator: SwarmOrchestrator) -> Self {
        let event_tx = orchestrator.event_tx.clone();
        Self {
            orchestrator: Arc::new(orchestrator),
            event_tx,
        }
    }
}
