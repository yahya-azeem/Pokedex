use pokedex_tools::{ContainerBashTool, Tool, ToolContext, PermissionLevel};
use pokedex_core::config::{Config, PermissionMode};
use pokedex_core::permissions::AutoPermissionHandler;
use pokedex_core::cost::CostTracker;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::atomic::AtomicUsize;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let working_dir = std::env::current_dir()?;
    println!("Testing ContainerBash with working_dir: {:?}", working_dir);

    let handler = Arc::new(AutoPermissionHandler {
        mode: PermissionMode::BypassPermissions,
    });
    
    let ctx = ToolContext {
        working_dir: working_dir.clone(),
        permission_mode: PermissionMode::BypassPermissions,
        permission_handler: handler,
        cost_tracker: CostTracker::new(),
        session_id: "test-session".to_string(),
        file_history: Arc::new(parking_lot::Mutex::new(pokedex_core::file_history::FileHistory::new())),
        current_turn: Arc::new(AtomicUsize::new(1)),
        non_interactive: true,
        mcp_manager: None,
        config: Config::default(),
    };

    let tool = ContainerBashTool;

    println!("--- Test 0: Basic Execution (uname) ---");
    let input0 = json!({
        "command": "uname -a",
        "description": "Check kernel version"
    });
    let result0 = tool.execute(input0, &ctx).await;
    println!("Result 0 Output:\n{}", result0.stdout);
    if result0.is_error {
        println!("Test 0 failed (Error state)!");
    } else {
        println!("Test 0 passed!");
    }

    println!("\n--- Test 1: ls /workspace ---");
    let input = json!({
        "command": "ls /workspace",
        "description": "List workspace files"
    });
    let result = tool.execute(input, &ctx).await;
    println!("Result 1 Output:\n{}", result.stdout);
    if result.is_error {
        println!("Test 1 failed (Error state)!");
    } else {
        println!("Test 1 passed!");
    }

    println!("\n--- Test 2: Persistence (cd) ---");
    let input2 = json!({
        "command": "cd /workspace/crates && pwd",
        "description": "Change directory and print it"
    });
    let result2 = tool.execute(input2, &ctx).await;
    println!("Result 2:\n{}", result2.stdout);

    let input3 = json!({
        "command": "pwd",
        "description": "Check current directory"
    });
    let result3 = tool.execute(input3, &ctx).await;
    println!("Result 3 (should be /workspace/crates):\n{}", result3.stdout);

    if result3.stdout.contains("/workspace/crates") {
        println!("Test 2 passed!");
    } else {
        println!("Test 2 failed!");
    }

    println!("\n--- Test 3: Environment Persistence ---");
    let input4 = json!({
        "command": "export FOO=BAR",
        "description": "Set environment variable"
    });
    let _ = tool.execute(input4, &ctx).await;

    let input5 = json!({
        "command": "echo $FOO",
        "description": "Check environment variable"
    });
    let result5 = tool.execute(input5, &ctx).await;
    println!("Result 5 (should be BAR):\n{}", result5.stdout);

    if result5.stdout.trim() == "BAR" {
        println!("Test 3 passed!");
    } else {
        println!("Test 3 failed!");
    }

    Ok(())
}
