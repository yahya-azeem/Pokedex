use reqwest::Client;
use tokio_stream::StreamExt;
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GOOGLE_API_KEY")?;
    let url = format!(
        "https://generativelanguage.googleapis.com/v1/models/gemini-1.5-flash:streamGenerateContent?key={}&alt=sse",
        api_key
    );

    let client = Client::new();
    let body = json!({
        "contents": [{
            "parts": [{"text": "Write a funny 2-line joke."}]
        }]
    });

    println!("Starting raw stream capture from: {}", url);
    let mut stream = client.post(&url)
        .json(&body)
        .send()
        .await?
        .bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        println!("--- NEW CHUNK ({} bytes) ---", chunk.len());
        println!("{}", String::from_utf8_lossy(&chunk));
    }

    Ok(())
}
