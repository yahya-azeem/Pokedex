use anyhow::Result;
use pokedex_swarm::llm::{MultiProviderClient, ModelTier};
use std::env;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: promptfoo_provider <prompt> [tier]");
        std::process::exit(1);
    }

    let prompt = &args[1];
    let tier = match args.get(2).map(|s| s.as_str()) {
        Some("simulation") => ModelTier::Simulation,
        Some("expert") => ModelTier::Expert,
        Some("research") => ModelTier::Research,
        _ => ModelTier::General,
    };

    let client = MultiProviderClient::new().await;
    
    match client.complete(tier, "You are a specialized agent in a swarm testing environment.", prompt).await {
        Ok((output, model_used)) => {
            let result = json!({
                "output": output,
                "model_used": model_used,
            });
            println!("{}", result);
            Ok(())
        }
        Err(e) => {
            let error = json!({
                "error": format!("LLM Error: {}", e)
            });
            println!("{}", error);
            Ok(())
        }
    }
}
