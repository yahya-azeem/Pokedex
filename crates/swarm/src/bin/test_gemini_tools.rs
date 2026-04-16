use pokedex_api::MultiProviderClient;
use pokedex_api::client::ClientConfig;
use pokedex_tools::ToolContext;
use pokedex_query::{run_query_loop, QueryConfig};
use pokedex_tools::Tool;
use pokedex_core::cost::CostTracker;
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let google_api_key = std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY must be set");

    let config = ClientConfig {
        google_api_key: Some(google_api_key),
        ..Default::default()
    };
    let client = MultiProviderClient::new(config)?;
    let cost_tracker = Arc::new(CostTracker::default());

    let mut messages = vec![pokedex_core::types::Message::user("Please write a file named 'test.txt' with the content 'hello from gemini' using your tools.".to_string())];
    
    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(pokedex_tools::FileWriteTool),
    ];

    let tool_ctx = ToolContext {
        working_dir: std::env::current_dir()?,
        permission_mode: pokedex_core::config::PermissionMode::BypassPermissions,
        permission_handler: Arc::new(pokedex_core::permissions::AutoPermissionHandler {
            mode: pokedex_core::config::PermissionMode::BypassPermissions,
        }),
        cost_tracker: cost_tracker.clone(),
        session_id: "test-session".to_string(),
        file_history: Arc::new(parking_lot::Mutex::new(pokedex_core::file_history::FileHistory::new())),
        current_turn: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        non_interactive: true,
        mcp_manager: None,
        config: pokedex_core::config::Config::default(),
    };

    let query_config = QueryConfig {
        model: "gemini-2.0-flash".to_string(),
        max_turns: 5,
        ..Default::default()
    };

    let (tx, mut rx) = mpsc::unbounded_channel();
    
    println!("Starting test loop...");
    
    let loop_task = tokio::spawn(async move {
        run_query_loop(
            &client,
            &mut messages,
            &tools,
            &tool_ctx,
            &query_config,
            cost_tracker,
            Some(tx),
            tokio_util::sync::CancellationToken::new(),
            None,
        ).await
    });

    while let Some(event) = rx.recv().await {
        println!("Event: {:?}", event);
    }

    let outcome = loop_task.await?;
    println!("Outcome: {:?}", outcome);

    Ok(())
}
