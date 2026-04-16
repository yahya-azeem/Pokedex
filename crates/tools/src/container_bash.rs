use crate::{PermissionLevel, Tool, ToolContext, ToolResult, session_shell_state};
use async_trait::async_trait;
use pokedex_core::bash_classifier::{BashRiskLevel, classify_bash_command};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::debug;
use crate::wasm_container::get_or_create_runner;
use crate::credential_governance::get_approved_credential_mounts;

pub struct ContainerBashTool;

#[derive(Debug, Deserialize)]
struct ContainerBashInput {
    command: String,
    #[serde(default)]
    agent_name: Option<String>,
    #[serde(default)]
    distribution: Option<String>,
    #[serde(default)]
    instance_id: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_timeout")]
    #[allow(dead_code)]
    timeout: u64,
}

fn default_timeout() -> u64 {
    std::env::var("CONTAINER_TIMEOUT_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(120_000)
}

#[async_trait]
impl Tool for ContainerBashTool {
    fn name(&self) -> &str {
        "ContainerBash"
    }

    fn description(&self) -> &str {
        "Executes a bash command in a secure WASM container (Alpine or Kali). \
         Workspace is mounted at /mnt/wasi0. State (cwd/env) is shared by distribution \
         and instance_id within a session. Use 'alpine' for dev and 'kali' for security tasks. \
         To use host credentials (gh, vercel, etc.), first request them via RequestCredential. \
         Always provide your agent_name to ensure proper credential mapping."
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Execute
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The bash command to execute"
                },
                "agent_name": {
                    "type": "string",
                    "description": "Your current persona name (e.g. 'Senior Developer') for credential mapping"
                },
                "distribution": {
                    "type": "string",
                    "enum": ["alpine", "kali", "openbsd"],
                    "description": "Target OS (alpine: dev/default, kali: security/pentesting, openbsd: hardened security)"
                },
                "instance_id": {
                    "type": "string",
                    "description": "Optional instance name for isolation within a session"
                },
                "description": {
                    "type": "string",
                    "description": "Description of the operation"
                },
                "timeout": {
                    "type": "number",
                    "description": "Timeout in ms"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let params: ContainerBashInput = match serde_json::from_value(input) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid input: {}", e)),
        };

        let distro_default = std::env::var("CONTAINER_DEFAULT_DISTRO").unwrap_or_else(|_| "alpine".to_string());
        let distro = params.distribution.clone().unwrap_or(distro_default);
        let instance = params.instance_id.clone().unwrap_or_else(|| "default".to_string());
        let agent_name = params.agent_name.clone().unwrap_or_else(|| "unknown".to_string());
        
        // Dynamic state key: session + distro + instance
        let state_key = format!("{}-{}-{}", ctx.session_id, distro, instance);

        // Permission check
        let desc = params.description.as_deref().unwrap_or(&params.command);
        if let Err(e) = ctx.check_permission(self.name(), desc, false) {
            return ToolResult::error(e.to_string());
        }

        // Security classifier
        if classify_bash_command(&params.command) == BashRiskLevel::Critical {
            return ToolResult::error("Command blocked by security classifier.");
        }

        // Retrieve state
        let shell_state_arc = session_shell_state(&state_key);
        let (_cwd, env_vars) = {
            let state = shell_state_arc.lock();
            (
                state.cwd.clone().unwrap_or_else(|| ctx.working_dir.clone()),
                state.env_vars.clone(),
            )
        };

        // Resolve WASM image path
        let wasm_file = format!("{}-amd64.wasm", distro);
        let image_dir = std::env::var("POKEDEX_IMAGE_DIR").unwrap_or_else(|_| ".pokedex/images".to_string());
        let wasm_path = ctx.working_dir.join(image_dir).join(wasm_file);
        
        if !wasm_path.exists() {
            return ToolResult::error(format!(
                "Container image for '{}' not found. Run setup script for this distro.",
                distro
            ));
        }

        // Fetch approved credential mounts
        let mounts = get_approved_credential_mounts(&ctx.session_id, &agent_name);
        
        // Build a preamble to link credential directories in the guest
        let mut preamble = String::new();
        preamble.push_str("mkdir -p ~/.config && ");
        for (_, guest_path) in &mounts {
            if let Some(tool) = guest_path.split('/').last() {
                // ln -sfn ensures we overwrite existing links/folders if needed
                preamble.push_str(&format!("ln -sfn {} ~/.config/{} && ", guest_path, tool));
            }
        }
        
        let final_command = format!("{}{}", preamble, params.command);

        let runner = match get_or_create_runner(&wasm_path) {
            Ok(r) => r,
            Err(e) => return ToolResult::error(format!("Initialization failed: {}", e)),
        };

        debug!(distro = %distro, state_key = %state_key, "Executing in container with mounts: {:?}", mounts);

        let runner_clone = runner.clone();
        let wd = ctx.working_dir.clone();
        let ev = env_vars.clone();
        let sk = state_key.clone();
        
        let run_result = match tokio::task::spawn_blocking(move || {
             runner_clone.run_command(&final_command, &wd, &mounts, &ev, &sk)
        }).await {
            Ok(res) => res,
            Err(e) => return ToolResult::error(format!("Task error: {}", e)),
        };

        match run_result {
            Ok((stdout, stderr, exit_code)) => {
                let mut output = stdout;
                if !stderr.is_empty() {
                    if !output.is_empty() { output.push('\n'); }
                    output.push_str("STDERR:\n");
                    output.push_str(&stderr);
                }
                if output.is_empty() { output = "(no output)".to_string(); }
                if exit_code != 0 {
                    ToolResult::error(format!("Exited code {}\n{}", exit_code, output))
                } else {
                    ToolResult::success(output)
                }
            }
            Err(e) => ToolResult::error(format!("Execution failed: {}", e)),
        }
    }
}
