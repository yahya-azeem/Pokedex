use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Events broadcast over WebSocket to connected frontend clients.
/// Each variant maps to a visual update in the Svelte dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SwarmEvent {
    /// The swarm has been created and manifest generation begins.
    SwarmCreated {
        swarm_id: Uuid,
        goal: String,
        timestamp: DateTime<Utc>,
    },

    /// The orchestrator generated the manifest (roles, plan).
    ManifestGenerated {
        swarm_id: Uuid,
        orchestrator_instructions: String,
        agent_count: usize,
        timestamp: DateTime<Utc>,
    },

    /// A new Pokeball agent has been spawned into the swarm.
    AgentSpawned {
        swarm_id: Uuid,
        agent_id: Uuid,
        name: String,
        role: String,
        persona_category: Option<String>,
        model: String,
        timestamp: DateTime<Utc>,
    },

    /// An agent's status has changed (Idle â†’ Working, etc.)
    AgentStatusChanged {
        swarm_id: Uuid,
        agent_id: Uuid,
        old_status: String,
        new_status: String,
        timestamp: DateTime<Utc>,
    },

    /// An agent produced an output or thought.
    AgentOutput {
        swarm_id: Uuid,
        agent_id: Uuid,
        agent_name: String,
        content: String,
        output_type: AgentOutputType,
        timestamp: DateTime<Utc>,
    },

    /// An agent sent a message to another agent (collaboration edge).
    AgentMessage {
        swarm_id: Uuid,
        from_agent_id: Uuid,
        to_agent_id: Uuid,
        from_name: String,
        to_name: String,
        message: String,
        timestamp: DateTime<Utc>,
    },

    /// The swarm phase has changed (Manifest â†’ Populating â†’ Simulating, etc.)
    PhaseChanged {
        swarm_id: Uuid,
        old_phase: String,
        new_phase: String,
        timestamp: DateTime<Utc>,
    },

    /// The swarm has completed execution with a final report.
    SwarmCompleted {
        swarm_id: Uuid,
        summary: String,
        total_messages: usize,
        duration_secs: f64,
        timestamp: DateTime<Utc>,
    },

    /// An error occurred in the swarm.
    SwarmError {
        swarm_id: Uuid,
        error: String,
        timestamp: DateTime<Utc>,
    },

    /// Technical tool execution details for visibility and debugging.
    ToolExecution {
        swarm_id: Uuid,
        agent_id: Uuid,
        agent_name: String,
        tool_name: String,
        input_json: String,
        output: Option<String>,
        is_error: bool,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentOutputType {
    /// Internal reasoning / chain-of-thought
    Thinking,
    /// A deliverable or work product
    Deliverable,
    /// A status update or progress report
    StatusUpdate,
    /// A question directed at the orchestrator or another agent
    Question,
}

impl SwarmEvent {
    pub fn swarm_id(&self) -> Uuid {
        match self {
            Self::SwarmCreated { swarm_id, .. }
            | Self::ManifestGenerated { swarm_id, .. }
            | Self::AgentSpawned { swarm_id, .. }
            | Self::AgentStatusChanged { swarm_id, .. }
            | Self::AgentOutput { swarm_id, .. }
            | Self::AgentMessage { swarm_id, .. }
            | Self::PhaseChanged { swarm_id, .. }
            | Self::SwarmCompleted { swarm_id, .. }
            | Self::SwarmError { swarm_id, .. }
            | Self::ToolExecution { swarm_id, .. } => *swarm_id,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::SwarmCreated { timestamp, .. }
            | Self::ManifestGenerated { timestamp, .. }
            | Self::AgentSpawned { timestamp, .. }
            | Self::AgentStatusChanged { timestamp, .. }
            | Self::AgentOutput { timestamp, .. }
            | Self::AgentMessage { timestamp, .. }
            | Self::PhaseChanged { timestamp, .. }
            | Self::SwarmCompleted { timestamp, .. }
            | Self::SwarmError { timestamp, .. }
            | Self::ToolExecution { timestamp, .. } => *timestamp,
        }
    }
}
