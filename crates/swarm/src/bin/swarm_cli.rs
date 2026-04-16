use anyhow::Result;
use clap::Parser;
use colored::*;
use dotenvy::dotenv;
use std::path::PathBuf;
use std::sync::Arc;

use pokedex_swarm::orchestrator::SwarmOrchestrator;
use pokedex_swarm::events::{SwarmEvent, AgentOutputType};
use pokedex_core::cost::CostTracker;
use pokedex_core::Settings;
use pokedex_api::client::ClientConfig;
use pokedex_api::ProviderClient;

#[derive(Parser, Debug)]
#[command(author, version, about = "Pokedex Swarm CLI — MiroFish Multi-Agent Engine")]
struct Args {
    /// The goal for the agent swarm to achieve
    #[arg(short, long, required_unless_present = "resume")]
    goal: Option<String>,

    /// Optional path to the library directory (defaults to ./library)
    #[arg(short, long, default_value = "library")]
    library: String,

    /// Number of agents for scaled simulation mode (enables MiroFish parallel engine)
    #[arg(short, long)]
    agents: Option<usize>,

    /// Resume a previously saved project by name
    #[arg(short, long)]
    resume: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env
    dotenv().ok();

    let args = Args::parse();
    let library_path = PathBuf::from(&args.library);
    let is_scaled = args.agents.is_some();
    let agent_count = args.agents.unwrap_or(0);

    println!("{}", "=== Pokedex Swarm CLI ===".bold().bright_magenta());
    
    if let Some(ref resume_name) = args.resume {
        println!("{} {}", "Resuming Project:".bold().yellow(), resume_name.bright_white());
    } else {
        let goal = args.goal.as_ref().expect("Goal is required when not resuming");
        println!("{} {}", "Goal:".bold().cyan(), goal.bright_white());
    }

    if is_scaled {
        println!("{} {} agents (MiroFish Parallel Engine)", "Scale:".bold().yellow(), agent_count.to_string().bold().bright_yellow());
    }
    println!("{} {}", "Library:".dimmed(), library_path.display());
    println!("{}", "---------------------------".dimmed());

    // Initialize Core API for agentic execution
    let config = Settings::load().await.unwrap_or_default().config;
    let api_config = ClientConfig {
        api_key: config.api_key.clone(),
        google_api_key: std::env::var("GOOGLE_API_KEY").ok(),
        github_token: std::env::var("GITHUB_TOKEN").ok(),
        api_base: config.resolve_api_base(),
        use_bearer_auth: false,
    };
    let core_api = Arc::new(ProviderClient::new(api_config)?);
    let tools = Arc::new(pokedex_tools::all_tools());
    let cost_tracker = CostTracker::new();

    // Initialize orchestrator with agentic power
    let orchestrator = Arc::new(SwarmOrchestrator::new(
        library_path,
        core_api,
        tools,
        cost_tracker,
    ).await?);
    let mut rx = orchestrator.event_tx.subscribe();

    // Start execution in background
    let goal = args.goal.clone();
    let orchestrator_clone = orchestrator.clone();
    
    let handle = tokio::spawn(async move {
        let mut swarm_id = None;

        if let Some(ref project_name) = args.resume {
            // Logic to find and load swarm from project directory
            let normalized_name = project_name.to_lowercase()
                .replace(' ', "_")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>();
            
            let dump_path = std::env::current_dir().unwrap()
                .join("Projects").join(normalized_name).join("Swarm").join("swarm_state.json");
            
            if dump_path.exists() {
                if let Ok(content) = std::fs::read_to_string(dump_path) {
                    if let Ok(swarm) = serde_json::from_str::<pokedex_swarm::orchestrator::Swarm>(&content) {
                        swarm_id = Some(swarm.id);
                        // Force it into the store
                        orchestrator_clone.store.save(&swarm).await.ok();
                        println!("  {} Successfully restored phase: {}", "ðŸ“¦".green(), swarm.phase);
                    }
                }
            } else {
                eprintln!("\n{} Persistence file not found at {}", "â Œ".red(), args.resume.as_ref().unwrap());
                return;
            }
        } else {
            let goal = args.goal.clone().unwrap();
            match orchestrator_clone.create_swarm(goal).await {
                Ok(id) => swarm_id = Some(id),
                Err(e) => {
                    eprintln!("\n{} {}", "Failed to create swarm:".red().bold(), e);
                    return;
                }
            }
        }

        if let Some(id) = swarm_id {
            let result = if is_scaled {
                orchestrator_clone.execute_swarm_scaled(id, agent_count).await
            } else {
                orchestrator_clone.execute_swarm(id).await
            };
            if let Err(e) = result {
                eprintln!("\n{} {}", "Error during execution:".red().bold(), e);
            }
        }
    });

    // Event monitoring loop
    while let Ok(event) = rx.recv().await {
        match event {
            SwarmEvent::ManifestGenerated { agent_count, orchestrator_instructions, .. } => {
                println!("\n{} {}", "ðŸ“‹ Swarm Manifest Generated".bold().bright_magenta(), format!("({} agents planned)", agent_count).dimmed());
                // Truncate plan for large swarms
                let plan_display = if orchestrator_instructions.len() > 500 {
                    format!("{}...", &orchestrator_instructions[..500])
                } else {
                    orchestrator_instructions
                };
                println!("{} {}", "Plan:".bold().cyan(), plan_display.italic());
            }
            SwarmEvent::AgentSpawned { name, role, model, .. } => {
                println!("{} Spawning agent: {} ({}) using {}", "âš¡".yellow(), name.bold().bright_white(), role.cyan(), model.dimmed().italic());
            }
            SwarmEvent::PhaseChanged { new_phase, .. } => {
                let (icon, label) = match new_phase.as_str() {
                    "manifest" => ("ðŸ§ ", "Dynamic Planning".bright_blue()),
                    "populating_agents" => ("ðŸ§¬", "Populating Swarm".bright_yellow()),
                    "collaborating" => ("ðŸ”¥", "Live Collaboration".bright_green()),
                    "report_generation" => ("ðŸ“„", "Generating Final Report".bright_magenta()),
                    "completed" => ("âœ…", "Execution Completed".green()),
                    "failed" => ("âŒ", "Execution Failed".red()),
                    _ => ("ðŸ”„", new_phase.normal()),
                };
                println!("\n{} Phase: {}", icon.bold(), label.bold());
            }
            SwarmEvent::AgentOutput { agent_name, content, output_type, .. } => {
                match output_type {
                    AgentOutputType::Thinking => {
                        if !is_scaled { // Skip verbose thinking in scaled mode
                            println!("\n{} {} is thinking...", "ðŸ§ ".dimmed(), agent_name.bold().bright_white());
                            println!("{}", content.dimmed().italic());
                        }
                    }
                    AgentOutputType::Deliverable => {
                        if !is_scaled { // Show full deliverables only in normal mode
                            println!("\n{} {} produced a deliverable:", "ðŸ“¦".green(), agent_name.bold().bright_white());
                            println!("{}", content.bright_white().bold());
                            println!("{}", "---------------------------".dimmed());
                        }
                    }
                    AgentOutputType::StatusUpdate => {
                        println!("{} {}", "â„¹ï¸".blue(), content.dimmed());
                    }
                    AgentOutputType::Question => {
                        println!("{} {} is asking: {}", "❓".yellow(), agent_name.bold().bright_white(), content.bright_yellow());
                    }
                }
            }
            SwarmEvent::AgentMessage { from_name, to_name, message, .. } => {
                if !is_scaled { // Skip collaboration messages in scaled mode
                    println!("{} {} -> {}: {}", "💬".blue(), from_name.bold().cyan(), to_name.bold().cyan(), message.italic());
                }
            }
            SwarmEvent::ToolExecution { agent_name, tool_name, input_json, .. } => {
                println!("\n{} {} is engaging technical agency: {} ({})", "🛠️".yellow().bold(), agent_name.bold().bright_white(), tool_name.bold().bright_yellow(), input_json.dimmed());
            }
            SwarmEvent::SwarmCompleted { duration_secs, .. } => {
                println!("\n{} Swarm execution completed in {}s", "✅".green().bold(), duration_secs.to_string().bold());
                break;
            }
            SwarmEvent::SwarmError { error, .. } => {
                eprintln!("\n{} {}", "âŒ Swarm Error:".red().bold(), error);
                break;
            }
            SwarmEvent::ToolExecution { agent_name, tool_name, input_json, output, is_error, .. } => {
                if !is_scaled {
                    if let Some(out) = output {
                        if is_error {
                            println!("  {} {} tool {} failed: {}", "âœ–".red(), agent_name.bold(), tool_name.bright_yellow(), out.red());
                        } else {
                            println!("  {} {} tool {} completed", "âœ…".green(), agent_name.bold(), tool_name.bright_yellow());
                        }
                    } else {
                        println!("  {} {} using tool: {} {}", "ðŸ› ï¸ ".cyan(), agent_name.bold(), tool_name.bright_yellow(), input_json.dimmed());
                    }
                }
            }
            _ => {}
        }
    }

    // Ensure the background task has finished
    let _ = handle.await;
    println!("\n{}", "=== Execution Finished ===".bold().bright_magenta());

    Ok(())
}
