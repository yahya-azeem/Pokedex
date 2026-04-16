//! # Pokedex Swarm
//!
//! Multi-agent orchestration engine for Pokedex. A single user prompt generates
//! a team of specialized AI agents ("Pokeballs") that collaborate towards a
//! shared goal, with real-time event broadcasting for frontend visualization.

pub mod db;
pub mod events;
pub mod llm;
pub mod orchestrator;
pub mod persona_loader;
pub mod pokeball;

// Re-export primary types
pub use events::{AgentOutputType, SwarmEvent};
pub use llm::{MultiProviderClient, ModelTier};
pub use orchestrator::{
    AgentRoleBlueprint, Swarm, SwarmManifest, SwarmOrchestrator, SwarmPhase,
};
pub use persona_loader::{Persona, PersonaCatalog};
pub use pokeball::{Pokeball, PokeballOutput, PokeballStatus};
