// MemPalaceTool: Direct integration with the project-local semantic memory system.
// Replaces file-based notes for team collaboration.

use crate::{PermissionLevel, Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use pokedex_mempalace::MemPalace;
use std::sync::Arc;
use std::path::PathBuf;

pub struct MemPalaceTool {
    // We expect the MemPalace instance to be passed in or managed globally.
    // However, Tool trait methods don't easily allow passing state during execution
    // unless it's in the Tool struct or ToolContext.
    // We'll look for a registry or expect the Orchestrator to initialize it.
}

#[derive(Debug, Deserialize)]
struct MemPalaceInput {
    action: String, // "remember" | "recall"
    content: Option<String>,
    query: Option<String>,
    wing: Option<String>, // e.g. "Architecture", "Security", "Testing"
    room: Option<String>, // e.g. "Key Decisions", "Database Schema"
}

/// Global registry for project-specific MemPalace instances to be used by tools.
pub static MEM_PALACE_REGISTRY: once_cell::sync::Lazy<dashmap::DashMap<String, Arc<MemPalace>>> =
    once_cell::sync::Lazy::new(dashmap::DashMap::new);

#[async_trait]
impl Tool for MemPalaceTool {
    fn name(&self) -> &str {
        "mempalace"
    }

    fn description(&self) -> &str {
        "Semantic memory system for project notes. Use 'remember' to save technical decisions, \
         and 'recall' to semantically search your teammates' work. This is the PRIMARY \
         way to share knowledge across the swarm."
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["remember", "recall"],
                    "description": "Whether to store a memory or search for one"
                },
                "content": {
                    "type": "string",
                    "description": "Content to store (for 'remember' action)"
                },
                "query": {
                    "type": "string",
                    "description": "Search query (for 'recall' action)"
                },
                "wing": {
                    "type": "string",
                    "description": "Organizational wing (e.g. 'Engineering', 'Security')"
                },
                "room": {
                    "type": "string",
                    "description": "Specific topic room (e.g. 'Database', 'Authentication')"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let params: MemPalaceInput = match serde_json::from_value(input) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid input: {}", e)),
        };

        // Permission check
        let desc = format!("MemPalace: {}", params.action);
        if let Err(e) = ctx.check_permission(self.name(), &desc, true) {
            return ToolResult::error(e.to_string());
        }

        // Get the palace for this session/project
        let palace = match MEM_PALACE_REGISTRY.get(&ctx.session_id) {
            Some(p) => p.clone(),
            None => {
                // Fallback: search for swarm-wide palace if session_id is specific
                if let Some(pos) = ctx.session_id.rfind('-') {
                    let swarm_id = &ctx.session_id[..pos];
                     match MEM_PALACE_REGISTRY.get(swarm_id) {
                         Some(p) => p.clone(),
                         None => return ToolResult::error("MemPalace not initialized for this project. Use the technical tools to prepare the workspace."),
                     }
                } else {
                    return ToolResult::error("MemPalace session not found.");
                }
            }
        };

        match params.action.as_str() {
            "remember" => {
                let content = params.content.ok_or_else(|| ToolResult::error("Missing content for 'remember'")).unwrap_or_default();
                let wing = params.wing.unwrap_or_else(|| "General".to_string());
                let room = params.room.unwrap_or_else(|| "Common".to_string());
                
                match palace.remember(&wing, &room, &content).await {
                    Ok(id) => ToolResult::success(format!("Memory stored successfully (ID: {}). Your teammates can now recall this.", id)),
                    Err(e) => ToolResult::error(format!("Failed to store memory: {}", e)),
                }
            }
            "recall" => {
                let query = params.query.ok_or_else(|| ToolResult::error("Missing query for 'recall'")).unwrap_or_default();
                let wing = params.wing.unwrap_or_else(|| "General".to_string());
                
                match palace.recall(&wing, &query, 5).await {
                    Ok(results) => {
                        if results.is_empty() {
                            ToolResult::success("No relevant memories found for your query.")
                        } else {
                            let formatted = results.iter()
                                .map(|r| format!("- {}", r))
                                .collect::<Vec<_>>()
                                .join("\n");
                            ToolResult::success(format!("Found {} relevant memories:\n{}", results.len(), formatted))
                        }
                    }
                    Err(e) => ToolResult::error(format!("Failed to recall memories: {}", e)),
                }
            }
            _ => ToolResult::error("Invalid action"),
        }
    }
}
