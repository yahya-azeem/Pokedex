use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::events::AgentOutputType;

/// A Pokeball represents an individual AI agent in the swarm.
/// Named after the Pokedex theme — each Pokeball captures a specific persona.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokeball {
    /// Unique identifier
    pub id: Uuid,
    /// Display name of this agent
    pub name: String,
    /// Role description (e.g. "Lead Engineer", "UX Designer")
    pub role: String,
    /// Which agency-agents persona this agent is based on (if any)
    pub persona_path: Option<String>,
    /// The full persona instructions loaded from the library
    pub persona_content: Option<String>,
    /// Category of the persona (e.g. "engineering", "design")
    pub persona_category: Option<String>,
    /// Which LLM model this agent uses
    pub model: String,
    /// Model affinity score (from promptfoo evaluation)
    pub model_score: Option<f64>,
    /// Current agent status
    pub status: PokeballStatus,
    /// Messages this agent has produced
    pub output_history: Vec<PokeballOutput>,
    /// IDs of other agents this one collaborates with
    pub collaboration_targets: Vec<Uuid>,
    /// Assigned task description
    pub task: Option<String>,
    /// When this agent was spawned
    pub created_at: DateTime<Utc>,
    /// When this agent finished (if applicable)
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PokeballStatus {
    /// Waiting to be assigned work
    Idle,
    /// Actively working on a task
    Working,
    /// Communicating with another agent
    Collaborating,
    /// Task complete
    Finished,
    /// Agent encountered an error
    Error,
}

impl std::fmt::Display for PokeballStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Working => write!(f, "working"),
            Self::Collaborating => write!(f, "collaborating"),
            Self::Finished => write!(f, "finished"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// A single output produced by a Pokeball agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokeballOutput {
    pub id: Uuid,
    pub content: String,
    pub output_type: AgentOutputType,
    pub timestamp: DateTime<Utc>,
    /// If this output is directed at a specific agent
    pub target_agent: Option<Uuid>,
}

impl Pokeball {
    /// Create a new Pokeball agent with the given role and model.
    pub fn new(name: String, role: String, model: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            role,
            persona_path: None,
            persona_content: None,
            persona_category: None,
            model,
            model_score: None,
            status: PokeballStatus::Idle,
            output_history: Vec::new(),
            collaboration_targets: Vec::new(),
            task: None,
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    /// Attach a persona from the agency-agents library.
    pub fn with_persona(mut self, path: String, content: String, category: String) -> Self {
        self.persona_path = Some(path);
        self.persona_content = Some(content);
        self.persona_category = Some(category);
        self
    }

    /// Assign a task to this agent.
    pub fn assign_task(&mut self, task: String) {
        self.task = Some(task);
        self.status = PokeballStatus::Working;
    }

    /// Record an output from this agent.
    pub fn add_output(&mut self, content: String, output_type: AgentOutputType) -> PokeballOutput {
        let output = PokeballOutput {
            id: Uuid::new_v4(),
            content,
            output_type,
            timestamp: Utc::now(),
            target_agent: None,
        };
        self.output_history.push(output.clone());
        output
    }

    /// Mark this agent as collaborating with another.
    pub fn collaborate_with(&mut self, target_id: Uuid) {
        if !self.collaboration_targets.contains(&target_id) {
            self.collaboration_targets.push(target_id);
        }
        self.status = PokeballStatus::Collaborating;
    }

    /// Mark this agent as finished.
    pub fn finish(&mut self) {
        self.status = PokeballStatus::Finished;
        self.completed_at = Some(Utc::now());
    }

    /// Build the system prompt for this agent's LLM calls.
    pub fn build_system_prompt(&self) -> String {
        let mut prompt = String::new();

        prompt.push_str(&format!("# Agent: {}\n", self.name));
        prompt.push_str(&format!("## Role: {}\n\n", self.role));

        if let Some(ref persona) = self.persona_content {
            prompt.push_str("## Persona Instructions\n\n");
            prompt.push_str(persona);
            prompt.push_str("\n\n");
        }

        if let Some(ref task) = self.task {
            prompt.push_str("## Current Task\n\n");
            prompt.push_str(task);
            prompt.push_str("\n\n");
        }

        prompt.push_str("## Operating Rules\n\n");
        prompt.push_str("- You are one agent in a collaborative swarm working toward a shared goal.\n");
        prompt.push_str("- Be concise and focused in your outputs.\n");
        prompt.push_str("- When you need input from another agent, clearly state what you need and from whom.\n");
        prompt.push_str("- Produce concrete, actionable deliverables — not vague suggestions.\n");
        prompt.push_str("- Structure your response with THINKING: and DELIVERABLE: sections.\n");

        prompt
    }

    /// Get a summary view suitable for API responses.
    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "name": self.name,
            "role": self.role,
            "persona_category": self.persona_category,
            "model": self.model,
            "status": format!("{}", self.status),
            "task": self.task,
            "output_count": self.output_history.len(),
            "collaboration_targets": self.collaboration_targets,
            "created_at": self.created_at,
        })
    }
}
