pub mod client;
pub mod streaming;

pub use client::{MultiProviderClient, ClientConfig};
pub type ProviderClient = MultiProviderClient;
pub use streaming::{StreamAccumulator, StreamEvent, StreamHandler};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelTier {
    Expert,    // Pro, Ultra, gpt-4
    Standard,  // Flash, mini, gemma-7b
    Base,      // gemma-2b, smaller-models
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: Option<String>,
    pub created_at: Option<i64>,
}

impl ModelInfo {
    pub fn tier(&self) -> ModelTier {
        let id_lower = self.id.to_lowercase();
        if id_lower.contains("pro") || id_lower.contains("ultra") || id_lower.contains("gpt-4") || id_lower.contains("o1") {
            ModelTier::Expert
        } else if id_lower.contains("flash") || id_lower.contains("mini") || id_lower.contains("sonnet") || id_lower.contains("gemma-7b") {
            ModelTier::Standard
        } else {
            ModelTier::Base
        }
    }
}

use pokedex_core::types::{Message, ToolDefinition};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMessage {
    pub role: String,
    pub content: serde_json::Value,
}

impl From<&Message> for ApiMessage {
    fn from(m: &Message) -> Self {
        Self {
            role: match m.role {
                pokedex_core::types::Role::User => "user".to_string(),
                pokedex_core::types::Role::Assistant => "assistant".to_string(),
            },
            content: serde_json::to_value(&m.content).unwrap_or(serde_json::Value::Null),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl From<&ToolDefinition> for ApiToolDefinition {
    fn from(d: &ToolDefinition) -> Self {
        Self {
            name: d.name.clone(),
            description: d.description.clone(),
            input_schema: d.input_schema.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    Text(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    pub budget_tokens: u32,
}

impl ThinkingConfig {
    pub fn enabled(budget: u32) -> Self {
        Self { budget_tokens: budget }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, derive_builder::Builder)]
#[builder(setter(into, strip_option))]
pub struct CreateMessageRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<ApiMessage>,
    #[builder(default)]
    pub system: Option<SystemPrompt>,
    #[builder(default)]
    pub tools: Vec<ApiToolDefinition>,
    #[builder(default)]
    pub thinking: Option<ThinkingConfig>,
    #[builder(default)]
    pub temperature: Option<f32>,
}

impl CreateMessageRequest {
    pub fn builder(model: &str, max_tokens: u32) -> CreateMessageRequestBuilder {
        let mut b = CreateMessageRequestBuilder::default();
        b.model(model.to_string());
        b.max_tokens(max_tokens);
        b
    }

    pub fn build(builder: CreateMessageRequestBuilder) -> Self {
        builder.build().unwrap()
    }
}
