use crate::{PermissionLevel, Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::info;

/// Process-global registry of approved core credentials.
/// Keyed by (session_id, agent_name).
static CREDENTIAL_APPROVALS: Lazy<DashMap<(String, String), HashSet<String>>> =
    Lazy::new(DashMap::new);

/// Get the list of approved host directories for an agent in a session.
pub fn get_approved_credential_mounts(
    session_id: &str,
    agent_name: &str,
) -> Vec<(PathBuf, String)> {
    let mut mounts = Vec::new();
    let approvals = match CREDENTIAL_APPROVALS.get(&(session_id.to_string(), agent_name.to_string())) {
        Some(a) => a.clone(),
        None => return mounts,
    };

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let appdata = std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| home.join("AppData/Roaming"));

    for cred in approvals {
        match cred.as_str() {
            "gh" => {
                let host_path = appdata.join("gh");
                mounts.push((host_path, "/mnt/creds/gh".to_string()));
            }
            "vercel" => {
                let host_path = appdata.join("com.vercel.cli/Data");
                mounts.push((host_path, "/mnt/creds/vercel".to_string()));
            }
            "gcloud" => {
                let host_path = appdata.join("gcloud");
                mounts.push((host_path, "/mnt/creds/gcloud".to_string()));
            }
            "supabase" => {
                let host_path = home.join(".supabase");
                mounts.push((host_path, "/mnt/creds/supabase".to_string()));
            }
            _ => {}
        }
    }
    mounts
}

// ---------------------------------------------------------------------------
// RequestCredentialTool
// ---------------------------------------------------------------------------

pub struct RequestCredentialTool;

#[derive(Debug, Deserialize)]
struct RequestCredentialInput {
    credential_name: String,
    justification: String,
}

#[async_trait]
impl Tool for RequestCredentialTool {
    fn name(&self) -> &str {
        "RequestCredential"
    }

    fn description(&self) -> &str {
        "Request access to a host-level credential (gh, vercel, gcloud, supabase). \
         You must provide a justification. Access is not granted until a Lead Agent \
         (Orchestrator or Auditor) approves the request."
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "credential_name": {
                    "type": "string",
                    "enum": ["gh", "vercel", "gcloud", "supabase"],
                    "description": "The name of the credential to request"
                },
                "justification": {
                    "type": "string",
                    "description": "Why do you need this credential for your task?"
                }
            },
            "required": ["credential_name", "justification"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let params: RequestCredentialInput = match serde_json::from_value(input) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid input: {}", e)),
        };

        info!(
            session_id = %ctx.session_id,
            credential = %params.credential_name,
            "Credential requested: {}",
            params.justification
        );

        ToolResult::success(format!(
            "Access request for '{}' submitted. Please wait for a Lead Agent (Orchestrator or Auditor) to approve this request via ApproveCredential.",
            params.credential_name
        ))
    }
}

// ---------------------------------------------------------------------------
// ApproveCredentialTool
// ---------------------------------------------------------------------------

pub struct ApproveCredentialTool;

#[derive(Debug, Deserialize)]
struct ApproveCredentialInput {
    agent_name: String,
    credential_name: String,
}

#[async_trait]
impl Tool for ApproveCredentialTool {
    fn name(&self) -> &str {
        "ApproveCredential"
    }

    fn description(&self) -> &str {
        "Authorize an agent to access a specific host-level credential directory. \
         Only Lead Agents (Orchestrator, Auditor) should call this tool. \
         Authorization persists for the duration of the current session."
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Write
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "agent_name": {
                    "type": "string",
                    "description": "The exact name/persona of the agent being authorized"
                },
                "credential_name": {
                    "type": "string",
                    "enum": ["gh", "vercel", "gcloud", "supabase"],
                    "description": "The name of the credential directory to approve"
                }
            },
            "required": ["agent_name", "credential_name"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let params: ApproveCredentialInput = match serde_json::from_value(input) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid input: {}", e)),
        };

        let key = (ctx.session_id.clone(), params.agent_name.clone());
        CREDENTIAL_APPROVALS
            .entry(key)
            .or_insert_with(HashSet::new)
            .insert(params.credential_name.clone());

        info!(
            session_id = %ctx.session_id,
            agent = %params.agent_name,
            credential = %params.credential_name,
            "Credential access APPROVED"
        );

        ToolResult::success(format!(
            "Agent '{}' has been granted access to the '{}' credential directory for this session.",
            params.agent_name, params.credential_name
        ))
    }
}
