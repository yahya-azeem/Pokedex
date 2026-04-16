// pokedex-tools: All tool implementations for the Pokedex Rust port.
//
// Each tool maps to a capability the LLM can invoke: running shell commands,
// reading/writing/editing files, searching codebases, fetching web pages, etc.

use async_trait::async_trait;
pub use pokedex_core::config::PermissionMode;
pub use pokedex_core::cost::CostTracker;
pub use pokedex_core::permissions::{PermissionDecision, PermissionHandler, PermissionRequest, PermissionLevel};
pub use pokedex_core::types::{ToolDefinition, ToolResultContent};
pub use pokedex_core::config::Config;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Sub-modules â€“ each contains a full tool implementation.
pub mod ask_user;
pub mod bash;
pub mod brief;
pub mod config_tool;
pub mod cron;
pub mod enter_plan_mode;
pub mod exit_plan_mode;
pub mod file_edit;
pub mod file_read;
pub mod file_write;
pub mod glob_tool;
pub mod grep_tool;
pub mod lsp_tool;
pub mod mcp_resources;
pub mod todo_write;
pub mod notebook_edit;
pub mod powershell;
pub mod send_message;
pub mod bundled_skills;
pub mod skill_tool;
pub mod sleep;
pub mod tasks;
pub mod tool_search;
pub mod web_fetch;
pub mod web_search;
pub mod worktree;
pub mod computer_use;
pub mod mcp_auth_tool;
pub mod repl_tool;
pub mod synthetic_output;
pub mod team_tool;
pub mod remote_trigger;
pub mod wasm_container;
pub mod credential_governance;
pub mod container_bash;
pub mod browser;
pub mod mem_palace_tool;

// Re-exports for convenience.
pub use ask_user::AskUserQuestionTool;
pub use bash::BashTool;
pub use brief::BriefTool;
pub use config_tool::ConfigTool;
pub use cron::{CronCreateTool, CronDeleteTool, CronListTool};
pub use enter_plan_mode::EnterPlanModeTool;
pub use exit_plan_mode::ExitPlanModeTool;
pub use file_edit::FileEditTool;
pub use file_read::FileReadTool;
pub use file_write::FileWriteTool;
pub use glob_tool::GlobTool;
pub use grep_tool::GrepTool;
pub use lsp_tool::LspTool;
pub use mcp_resources::{ListMcpResourcesTool, ReadMcpResourceTool};
pub mod notebook_edit_reexport { pub use crate::notebook_edit::NotebookEditTool; }
pub use notebook_edit_reexport::NotebookEditTool;
pub use powershell::PowerShellTool;
pub use send_message::SendMessageTool;
pub use skill_tool::SkillTool;
pub use sleep::SleepTool;
pub use tasks::{TaskCreateTool, TaskGetTool, TaskListTool, TaskOutputTool, TaskStopTool, TaskUpdateTool};
pub use todo_write::TodoWriteTool;
pub use tool_search::ToolSearchTool;
pub use web_fetch::WebFetchTool;
pub use web_search::WebSearchTool;
pub use worktree::{EnterWorktreeTool, ExitWorktreeTool};
pub use mcp_auth_tool::McpAuthTool;
pub use repl_tool::ReplTool;
pub use team_tool::{TeamCreateTool, TeamDeleteTool};
pub use synthetic_output::SyntheticOutputTool;
pub use remote_trigger::RemoteTriggerTool;
pub use container_bash::ContainerBashTool;
pub use browser::BrowserTool;
pub use mem_palace_tool::MemPalaceTool;

/// Trait to be implemented by all system tools.
// ... (skipping some unchanged lines for readability in thought but need full block in tool call)
// Actually I'll just use smaller chunks to be safe.

// (Re-evaluating chunks)
// I'll do two chunks: one for module/re-exports, one for all_tools().

/// Trait to be implemented by all system tools.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique name used by the LLM to invoke the tool.
    fn name(&self) -> &str;

    /// Human-friendly description of what the tool does.
    fn description(&self) -> &str;

    /// Required permission level to use this tool.
    fn permission_level(&self) -> PermissionLevel;

    /// JSON schema for the tool's input arguments.
    fn input_schema(&self) -> Value;

    /// Execute the tool logic.
    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult;

    /// Convert the tool into a standard definition for LLM prompting.
    fn to_definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: self.input_schema(),
        }
    }
}

/// The result of a tool execution.
pub struct ToolResult {
    pub stdout: String,
    pub stderr: String,
    pub is_error: bool,
}

impl ToolResult {
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            stderr: String::new(),
            is_error: false,
        }
    }

    pub fn error(stderr: impl Into<String>) -> Self {
        Self {
            stdout: String::new(),
            stderr: stderr.into(),
            is_error: true,
        }
    }
    
    /// Compatibility shim.
    pub fn with_metadata(self, _metadata: Value) -> Self {
        self
    }
}

impl std::fmt::Display for ToolResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.stdout.is_empty() {
            write!(f, "{}", self.stdout)?;
        }
        if !self.stderr.is_empty() {
            if !self.stdout.is_empty() {
                write!(f, "\n")?;
            }
            write!(f, "ERROR: {}", self.stderr)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ShellState {
    /// Current working directory as tracked by the shell state.
    pub cwd: Option<PathBuf>,
    /// Environment variable overrides exported by previous commands.
    pub env_vars: HashMap<String, String>,
}

impl ShellState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Process-global registry of shell states keyed by session_id.
static SHELL_STATE_REGISTRY: once_cell::sync::Lazy<dashmap::DashMap<String, Arc<parking_lot::Mutex<ShellState>>>> =
    once_cell::sync::Lazy::new(dashmap::DashMap::new);

/// Return the persistent `ShellState` for the given session, creating one if needed.
pub fn session_shell_state(session_id: &str) -> Arc<parking_lot::Mutex<ShellState>> {
    SHELL_STATE_REGISTRY
        .entry(session_id.to_string())
        .or_insert_with(|| Arc::new(parking_lot::Mutex::new(ShellState::new())))
        .clone()
}

/// Remove the shell state for a session (e.g. when the session ends).
pub fn clear_session_shell_state(session_id: &str) {
    SHELL_STATE_REGISTRY.remove(session_id);
}

/// Shared context passed to every tool invocation.
#[derive(Clone)]
pub struct ToolContext {
    pub working_dir: PathBuf,
    pub permission_mode: PermissionMode,
    pub permission_handler: Arc<dyn PermissionHandler>,
    pub cost_tracker: Arc<CostTracker>,
    pub session_id: String,
    pub file_history: Arc<parking_lot::Mutex<pokedex_core::file_history::FileHistory>>,
    pub current_turn: Arc<AtomicUsize>,
    /// If true, suppress interactive prompts (batch / CI mode).
    pub non_interactive: bool,
    /// Optional MCP manager for ListMcpResources / ReadMcpResource tools.
    pub mcp_manager: Option<Arc<pokedex_mcp::McpManager>>,
    /// Configured event hooks (PreToolUse, PostToolUse, etc.).
    pub config: Config,
}

impl ToolContext {
    /// Resolve a path (possibly relative) against the working directory.
    pub fn resolve_path<P: AsRef<std::path::Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_dir.join(path)
        }
    }

    /// Helper to check permission for a tool operation.
    pub fn check_permission(
        &self,
        tool: &str,
        description: &str,
        is_read_only: bool,
    ) -> Result<(), anyhow::Error> {
        let decision = self.permission_handler.check_permission(&PermissionRequest {
            tool_name: tool.to_string(),
            description: description.to_string(),
            details: None,
            is_read_only,
        });

        match decision {
            PermissionDecision::Allow => Ok(()),
            PermissionDecision::Deny => {
                anyhow::bail!("Permission denied for tool '{}'", tool)
            }
            _ => Ok(()),
        }
    }
    
    /// Compatibility shim. NotebookEditTool calls this with 4 args + self.
    pub fn record_file_change(&self, _path: PathBuf, _old: &[u8], _new: &[u8], _tool: &str) {
        // No-op
    }
}

/// Return all built-in tools (excluding AgentTool, which lives in pokedex-query).
pub fn all_tools() -> Vec<Box<dyn Tool>> {
    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(container_bash::ContainerBashTool),
        Box::new(credential_governance::RequestCredentialTool),
        Box::new(credential_governance::ApproveCredentialTool),
        Box::new(FileReadTool),
        Box::new(FileEditTool),
        Box::new(FileWriteTool),
        Box::new(GlobTool),
        Box::new(GrepTool),
        Box::new(WebFetchTool),
        Box::new(WebSearchTool),
        Box::new(NotebookEditTool),
        Box::new(TaskCreateTool),
        Box::new(TaskGetTool),
        Box::new(TaskUpdateTool),
        Box::new(TaskListTool),
        Box::new(TaskStopTool),
        Box::new(TaskOutputTool),
        Box::new(TodoWriteTool),
        Box::new(AskUserQuestionTool),
        Box::new(EnterPlanModeTool),
        Box::new(ExitPlanModeTool),
        Box::new(SleepTool),
        Box::new(CronCreateTool),
        Box::new(CronDeleteTool),
        Box::new(CronListTool),
        Box::new(EnterWorktreeTool),
        Box::new(ExitWorktreeTool),
        Box::new(ListMcpResourcesTool),
        Box::new(ReadMcpResourceTool),
        Box::new(ToolSearchTool),
        Box::new(BriefTool),
        Box::new(ConfigTool),
        Box::new(SendMessageTool),
        Box::new(team_tool::TeamCreateTool),
        Box::new(team_tool::TeamDeleteTool),
        Box::new(SkillTool),
        Box::new(LspTool),
        Box::new(ReplTool),
        Box::new(SyntheticOutputTool),
        Box::new(McpAuthTool),
        Box::new(RemoteTriggerTool),
        Box::new(BrowserTool),
        Box::new(MemPalaceTool),
    ];

    tools
}

/// Find a tool by name (case-sensitive).
pub fn find_tool(name: &str) -> Option<Box<dyn Tool>> {
    all_tools().into_iter().find(|t| t.name() == name)
}
