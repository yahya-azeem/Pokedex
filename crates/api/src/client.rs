use crate::{CreateMessageRequest, ApiMessage};
use crate::streaming::{StreamEvent, StreamHandler};
use pokedex_core::error::{Result, ClaudeError};
use std::sync::Arc;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use tracing::debug;

#[derive(Debug, Clone, Default)]
pub struct ClientConfig {
    pub api_key: Option<String>,
    pub google_api_key: Option<String>,
    pub github_token: Option<String>,
    pub api_base: String,
    pub use_bearer_auth: bool,
}

impl ClientConfig {
    /// Resolve the best available model from the provide pool based on tier.
    pub async fn resolve_best_in_tier(&self, client: &MultiProviderClient, tier: crate::ModelTier) -> String {
        let models = client.fetch_available_models().await.unwrap_or_default();
        models.into_iter()
            .filter(|m| m.tier() == tier)
            .map(|m| m.id)
            .next()
            .unwrap_or_else(|| "gemini-2.0-flash".to_string())
    }

    pub async fn resolve_auto_model(&self, client: &MultiProviderClient, requested: &str) -> String {
        if requested != "auto" && !requested.is_empty() && requested != "expert" && requested != "flash" {
            return requested.to_string();
        }

        let tier = match requested {
            "expert" => crate::ModelTier::Expert,
            "flash" | "auto" | "" => crate::ModelTier::Standard,
            _ => crate::ModelTier::Standard,
        };

        self.resolve_best_in_tier(client, tier).await
    }
}

pub enum Provider {
    Gemini,
    Copilot,
    Anthropic,
}

impl Provider {
    pub fn from_model(model: &str) -> Self {
        if model.contains("gemini") || model.contains("gemma") {
            Provider::Gemini
        } else if model.contains("gpt-") || model.contains("o1") || model.contains("o3") {
            Provider::Copilot
        } else {
            Provider::Anthropic
        }
    }
}

pub struct MultiProviderClient {
    config: ClientConfig,
    http: reqwest::Client,
}

impl MultiProviderClient {
    pub fn new(config: ClientConfig) -> Result<Self> {
        Ok(Self {
            config,
            http: reqwest::Client::new(),
        })
    }

    pub fn from_config(cfg: &pokedex_core::Config) -> Result<Self> {
        let config = ClientConfig {
            api_key: cfg.api_key.clone(),
            google_api_key: cfg.google_api_key.clone(),
            github_token: cfg.github_token.clone(),
            api_base: cfg.resolve_api_base(),
            use_bearer_auth: false,
        };
        Self::new(config)
    }

    pub async fn create_message_stream(
        &self,
        mut request: CreateMessageRequest,
        _handler: Arc<dyn StreamHandler>,
    ) -> Result<mpsc::UnboundedReceiver<StreamEvent>> {
        // Auto-resolve model from available providers
        request.model = self.config.resolve_auto_model(self, &request.model).await;

        let provider = Provider::from_model(&request.model);
        match provider {
            Provider::Gemini => self.call_gemini_stream(request).await,
            Provider::Copilot => self.call_copilot_stream(request).await,
            Provider::Anthropic => {
                let (tx, rx) = mpsc::unbounded_channel();
                let _ = tx.send(StreamEvent::Error {
                    error_type: "provider_unavailable".to_string(),
                    message: "Anthropic provider not available. Configure GOOGLE_API_KEY or GITHUB_TOKEN.".to_string(),
                });
                Ok(rx)
            }
        }
    }

    pub async fn create_message(&self, mut request: CreateMessageRequest) -> Result<crate::ApiMessage> {
        // Auto-resolve model from available providers
        request.model = self.config.resolve_auto_model(self, &request.model).await;

        let provider = Provider::from_model(&request.model);
        match provider {
            Provider::Gemini => self.call_gemini(&request).await,
            Provider::Copilot => self.call_copilot(&request).await,
            Provider::Anthropic => {
                Err(ClaudeError::Other("Anthropic provider not available. Configure GOOGLE_API_KEY or GITHUB_TOKEN.".to_string()))
            }
        }
    }

    // ─── Gemini Implementation ───────────────────────────────────────────

    async fn call_gemini(&self, request: &CreateMessageRequest) -> Result<ApiMessage> {
        let api_key = self.config.google_api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("GOOGLE_API_KEY not set"))?;

        let model_name = request.model.strip_prefix("models/").unwrap_or(&request.model);
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model_name, api_key
        );

        let body = self.map_to_gemini_request(request);
        
        let response = self.http.post(&url).json(&body).send().await
            .map_err(|e| ClaudeError::Api(e.to_string()))?;

        if !response.status().is_success() {
            let err = response.text().await.unwrap_or_default();
            return Err(ClaudeError::Api(format!("Gemini Error: {}", err)));
        }

        let completion: GeminiResponse = response.json().await
            .map_err(|e| ClaudeError::Api(e.to_string()))?;

        let candidate = completion.candidates.as_ref().and_then(|c| c.first())
            .ok_or_else(|| ClaudeError::Api("No candidates in Gemini response".to_string()))?;

        // Handle tool calls vs text response
        let mut content = serde_json::Value::Null;
        if let Some(parts) = candidate.content.as_ref().and_then(|c| c.parts.as_ref()) {
            for part in parts {
                if let Some(text) = &part.text {
                    content = serde_json::Value::String(text.clone());
                } else if let Some(call) = &part.function_call {
                    content = serde_json::json!([{
                        "type": "tool_use",
                        "id": format!("call_{}", uuid::Uuid::new_v4()), 
                        "name": call.name,
                        "input": call.args
                    }]);
                }
            }
        }

        Ok(ApiMessage {
            role: "assistant".to_string(),
            content,
        })
    }

    /// Gemini streaming: uses `?alt=sse` to get Server-Sent Events format.
    /// Without `alt=sse`, Gemini returns plain JSON chunks, NOT SSE.
    async fn call_gemini_stream(&self, request: CreateMessageRequest) -> Result<mpsc::UnboundedReceiver<StreamEvent>> {
        let api_key = self.config.google_api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("GOOGLE_API_KEY not set"))?;

        let model_name = request.model.strip_prefix("models/").unwrap_or(&request.model);
        // CRITICAL FIX: append `alt=sse` so the endpoint returns text/event-stream
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?key={}&alt=sse",
            model_name, api_key
        );

        let body = self.map_to_gemini_request(&request);
        let (tx, rx) = mpsc::unbounded_channel();
        
        let mut last_error = "Unknown error".to_string();
        let mut response = None;

        for attempt in 0..=5 {
            if attempt > 0 {
                let delay = std::time::Duration::from_secs(2u64.pow(attempt) * 8);
                println!("  [429_RETRY] Rate limited. Attempt {}/5. Re-trying in {}s...", attempt, delay.as_secs());
                tokio::time::sleep(delay).await;
            }

            let res = self.http.post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| ClaudeError::Api(e.to_string()))?;

            println!("  [RAW_STREAM] Attempt {} - Response Status: {}", attempt, res.status());

            if res.status().is_success() {
                response = Some(res);
                break;
            }

            let status = res.status();
            last_error = res.text().await.unwrap_or_default();
            
            if status != reqwest::StatusCode::TOO_MANY_REQUESTS && status.as_u16() != 429 {
                return Err(ClaudeError::Api(format!("Gemini Stream Error ({}): {}", status, last_error)));
            }
            
            if attempt == 5 {
                return Err(ClaudeError::Api(format!("Gemini Stream Error (Retries Exhausted): {}", last_error)));
            }
        }

        let response = response.ok_or_else(|| ClaudeError::Api("Failed to initialize stream after retries".to_string()))?;
        let mut response_stream = response.bytes_stream();

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stop_reason = "end_turn".to_string();
            let mut buffer = String::new();
            
            println!("  [RAW_STREAM] Listening for bytes from Gemini v1beta...");

            while let Some(chunk_result) = response_stream.next().await {
                let chunk = match chunk_result {
                    Ok(b) => b,
                    Err(e) => {
                        let _ = tx.send(StreamEvent::Error { error_type: "stream_error".to_string(), message: e.to_string() });
                        return;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                // Process complete SSE data lines
                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].trim().to_string();
                    buffer = buffer[pos + 1..].to_string();

                    if line.starts_with("data: ") {
                        let json_data = &line["data: ".len()..];
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_data) {
                            // DEFENSIVE EXTRACTION: Hunt for functionCall in any part of any candidate
                            if let Some(candidates) = val.get("candidates").and_then(|c| c.as_array()) {
                                for candidate in candidates {
                                    if let Some(parts) = candidate.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                                        for part in parts {
                                            // Handle text
                                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                                let _ = tx.send(StreamEvent::ContentBlockDelta {
                                                    index: 0,
                                                    delta: crate::streaming::ContentDelta::TextDelta { text: text.to_string() },
                                                });
                                            }
                                            // Handle tool call
                                            if let Some(call) = part.get("functionCall") {
                                                let name = call.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                                                let args = call.get("args").cloned().unwrap_or(serde_json::json!({}));
                                                stop_reason = "tool_use".to_string();
                                                tracing::info!("  [Gemini] Technical agency detected: {}", name);
                                                println!("  🛠️ [Gemini] DETECTED TECHNICAL AGENCY: {}", name);
                                                let _ = tx.send(StreamEvent::ContentBlockStart {
                                                    index: 1,
                                                    content_block: serde_json::json!({
                                                        "type": "tool_use",
                                                        "id": format!("call_{}_{}", name, uuid::Uuid::new_v4().to_string()[..8].to_string()),
                                                        "name": name,
                                                        "input": args
                                                    })
                                                });
                                                let _ = tx.send(StreamEvent::ContentBlockStop { index: 1 });
                                            }
                                        }
                                    }
                                    // Extract finish reason if present
                                    if let Some(fr) = candidate.get("finishReason").and_then(|r| r.as_str()) {
                                        let new_reason = match fr {
                                            "STOP" => "end_turn".to_string(),
                                            "MAX_TOKENS" => "max_tokens".to_string(),
                                            "SAFETY" | "RECITATION" | "OTHER" => "error".to_string(),
                                            _ => "end_turn".to_string(),
                                        };
                                        if stop_reason != "tool_use" || new_reason != "end_turn" {
                                            stop_reason = new_reason;
                                        }
                                    }
                                }
                            }
                            
                            // Extract usage metadata
                            if let Some(usage) = val.get("usageMetadata") {
                                let usage_info = pokedex_core::types::UsageInfo {
                                    input_tokens: usage.get("promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0),
                                    output_tokens: usage.get("candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0),
                                    cache_creation_input_tokens: 0,
                                    cache_read_input_tokens: usage.get("cachedContentTokenCount").and_then(|v| v.as_u64()).unwrap_or(0),
                                };
                                let _ = tx.send(StreamEvent::MessageDelta {
                                    delta: serde_json::json!({}),
                                    usage: serde_json::to_value(&usage_info).unwrap_or_default(),
                                });
                            }
                        }
                    }
                }
            }
            
            // Final message-stop event with accumulated stop reason.
            let _ = tx.send(StreamEvent::MessageStop {
                stop_reason: Some(stop_reason),
                usage: None,
            });
        });

        Ok(rx)
    }

    /// Map a CreateMessageRequest to Gemini's JSON format.
    /// CRITICAL FIX: Flatten all content blocks to text-only parts.
    /// Gemini expects `{"text": "..."}` objects in `parts`, not raw ContentBlock arrays.
    fn map_to_gemini_request(&self, request: &CreateMessageRequest) -> serde_json::Value {
        let mut contents: Vec<serde_json::Value> = Vec::new();
        let mut last_role = None;

        for m in &request.messages {
            let role = if m.role == "user" { "user" } else if m.role == "assistant" { "model" } else { continue };
            
            // Strictly enforce alternating roles.
            // If the same role repeats (e.g. multiple assistant turns or injected user messages), 
            // merge them into a single turn for Gemini.
            if Some(role) == last_role {
                if let Some(last_content) = contents.last_mut() {
                    let mut parts = Self::flatten_content_to_structured_parts(&m.content);
                    last_content["parts"].as_array_mut().unwrap().append(&mut parts);
                }
                continue;
            }
            
            // First message must be "user".
            if last_role.is_none() && role != "user" { continue; }

            let parts = Self::flatten_content_to_structured_parts(&m.content);
            if parts.is_empty() { continue; }

            contents.push(serde_json::json!({
                "role": role,
                "parts": parts
            }));
            last_role = Some(role);
        }

        // If we filtered out everything, or history is empty, add a dummy user prompt 
        // if we are sure there is system instruction. Usually there is at least one user prompt.
        if contents.is_empty() {
             contents.push(serde_json::json!({
                 "role": "user",
                 "parts": [{"text": "Hello"}]
             }));
        }

        let mut body = serde_json::json!({
            "contents": contents
        });

        if let Some(s) = &request.system {
            body["system_instruction"] = match s {
                crate::SystemPrompt::Text(t) => serde_json::json!({
                    "parts": [{"text": t}]
                })
            };
        }

        if !request.tools.is_empty() {
            body["tools"] = serde_json::json!([{
                "function_declarations": request.tools.iter().map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.input_schema
                    })
                }).collect::<Vec<_>>()
            }]);
            
            // USE AUTO MODE: Aggressive prompting in the system message will handle forced agency.
            body["tool_config"] = serde_json::json!({
                "function_calling_config": {
                    "mode": "AUTO"
                }
            });
        }

        println!("  [GEMINI_DEBUG] Sending request with {} tools and {} messages", request.tools.len(), request.messages.len());
        body
    }

    /// Map standardized content blocks to Gemini-native structured parts.
    /// Preserves 'functionCall' and 'functionResponse' for proper agentic loops.
    fn flatten_content_to_structured_parts(content: &serde_json::Value) -> Vec<serde_json::Value> {
        match content {
            serde_json::Value::String(s) => vec![serde_json::json!({"text": s})],
            serde_json::Value::Array(arr) => {
                let mut parts = Vec::new();
                for block in arr {
                    let b_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("text");
                    match b_type {
                        "text" => {
                            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                parts.push(serde_json::json!({"text": text}));
                            }
                        }
                        "thinking" => {
                            if let Some(text) = block.get("thinking").and_then(|t| t.as_str()) {
                                // Gemini doesn't have a native 'thinking' part yet, wrap as text.
                                parts.push(serde_json::json!({"text": format!("<thinking>\n{}\n</thinking>", text)}));
                            }
                        }
                        "tool_use" => {
                            let name = block.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                            let input = block.get("input").cloned().unwrap_or(serde_json::json!({}));
                            parts.push(serde_json::json!({
                                "functionCall": {
                                    "name": name,
                                    "args": input
                                }
                            }));
                        }
                        "tool_result" => {
                            let id = block.get("tool_use_id").and_then(|i| i.as_str()).unwrap_or("unknown");
                            let name = block.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                            let content_val = block.get("content").cloned().unwrap_or(serde_json::json!(""));
                            
                            // Native Gemini function_response: MUST include 'id' and 'name'
                            parts.push(serde_json::json!({
                                "functionResponse": {
                                    "id": id,
                                    "name": name,
                                    "response": { "result": content_val }
                                }
                            }));
                        }
                        _ => {
                            // Try to extract anything as text if it failed other tags
                            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                parts.push(serde_json::json!({"text": text}));
                            }
                        }
                    }
                }
                if parts.is_empty() {
                    vec![serde_json::json!({"text": content.to_string()})]
                } else {
                    parts
                }
            }
            serde_json::Value::Null => vec![serde_json::json!({"text": ""})],
            other => vec![serde_json::json!({"text": other.to_string()})],
        }
    }

    // ─── Copilot Implementation ──────────────────────────────────────────

    /// Flatten content for Copilot (OpenAI format expects string content).
    fn flatten_content_to_string(content: &serde_json::Value) -> String {
        match content {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(arr) => {
                arr.iter().filter_map(|block| {
                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                        Some(text.to_string())
                    } else if let Some(thinking) = block.get("thinking").and_then(|t| t.as_str()) {
                        Some(format!("[Thinking: {}]", thinking))
                    } else if block.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                        let c = block.get("content").map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        }).unwrap_or_default();
                        Some(c)
                    } else {
                        None
                    }
                }).collect::<Vec<_>>().join("\n")
            }
            serde_json::Value::Null => String::new(),
            other => other.to_string(),
        }
    }

    async fn call_copilot(&self, request: &CreateMessageRequest) -> Result<ApiMessage> {
        let token = self.config.github_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("GITHUB_TOKEN not set"))?;

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token))?);

        let url = "https://models.inference.ai.azure.com/chat/completions";

        let mut messages = Vec::new();
        if let Some(crate::SystemPrompt::Text(s)) = &request.system {
            messages.push(serde_json::json!({ "role": "system", "content": s }));
        }
        for m in &request.messages {
            // CRITICAL FIX: Flatten content blocks to plain strings for Copilot
            let content_str = Self::flatten_content_to_string(&m.content);
            messages.push(serde_json::json!({
                "role": m.role,
                "content": content_str
            }));
        }

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens
        });

        if !request.tools.is_empty() {
            body["tools"] = serde_json::json!(request.tools.iter().map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.input_schema
                    }
                })
            }).collect::<Vec<_>>());
        }

        let response = self.http.post(url).headers(headers).json(&body).send().await
            .map_err(|e| ClaudeError::Api(e.to_string()))?;

        if !response.status().is_success() {
            let err = response.text().await.unwrap_or_default();
            return Err(ClaudeError::Api(format!("Copilot Error: {}", err)));
        }

        let resp: serde_json::Value = response.json().await
            .map_err(|e| ClaudeError::Api(e.to_string()))?;

        let choice = &resp["choices"][0]["message"];
        
        let mut content = serde_json::Value::Null;
        if let Some(text) = choice["content"].as_str() {
            content = serde_json::Value::String(text.to_string());
        }
        
        if let Some(tool_calls) = choice["tool_calls"].as_array() {
            content = serde_json::json!(tool_calls.iter().map(|tc| {
                serde_json::json!({
                    "type": "tool_use",
                    "id": tc["id"],
                    "name": tc["function"]["name"],
                    "input": serde_json::from_str::<serde_json::Value>(tc["function"]["arguments"].as_str().unwrap_or("{}")).unwrap_or_default()
                })
            }).collect::<Vec<_>>());
        }

        Ok(ApiMessage {
            role: "assistant".to_string(),
            content,
        })
    }

    async fn call_copilot_stream(&self, request: CreateMessageRequest) -> Result<mpsc::UnboundedReceiver<StreamEvent>> {
        let token = self.config.github_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("GITHUB_TOKEN not set"))?;

        let url = "https://models.inference.ai.azure.com/chat/completions";
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token))?);

        let mut messages = Vec::new();
        if let Some(crate::SystemPrompt::Text(s)) = &request.system {
            messages.push(serde_json::json!({ "role": "system", "content": s }));
        }
        for m in &request.messages {
            let content_str = Self::flatten_content_to_string(&m.content);
            messages.push(serde_json::json!({ "role": m.role, "content": content_str }));
        }

        let body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens,
            "stream": true
        });

        let (tx, rx) = mpsc::unbounded_channel();
        let mut event_source = reqwest_eventsource::RequestBuilderExt::eventsource(self.http.post(url).headers(headers).json(&body))
            .map_err(|e| ClaudeError::Api(e.to_string()))?;

        tokio::spawn(async move {
            use reqwest_eventsource::Event;
            while let Some(event) = tokio_stream::StreamExt::next(&mut event_source).await {
                match event {
                    Ok(Event::Message(msg)) => {
                        if msg.data == "[DONE]" { break; }
                        if let Ok(resp) = serde_json::from_str::<serde_json::Value>(&msg.data) {
                            if let Some(choice) = resp["choices"].as_array().and_then(|a| a.first()) {
                                if let Some(text) = choice["delta"]["content"].as_str() {
                                    let _ = tx.send(StreamEvent::ContentBlockDelta { 
                                        index: 0, 
                                        delta: crate::streaming::ContentDelta::TextDelta { text: text.to_string() } 
                                    });
                                }
                                if let Some(tool_calls) = choice["delta"]["tool_calls"].as_array() {
                                    for tc in tool_calls {
                                        let _ = tx.send(StreamEvent::ContentBlockStart {
                                            index: 1,
                                            content_block: serde_json::json!({
                                                "type": "tool_use",
                                                "id": tc["id"],
                                                "name": tc["function"]["name"],
                                                "input": tc["function"]["arguments"]
                                            })
                                        });
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(StreamEvent::Error { error_type: "stream_error".into(), message: e.to_string() });
                        break;
                    }
                    _ => {}
                }
            }
            let _ = tx.send(StreamEvent::MessageStop {
                stop_reason: Some("end_turn".to_string()),
                usage: None,
            });
        });

        Ok(rx)
    }

    pub async fn fetch_available_models(&self) -> Result<Vec<crate::ModelInfo>> {
        let mut models = Vec::new();
        
        // 1. Fetch from Gemini
        if let Some(api_key) = &self.config.google_api_key {
            let url = format!("https://generativelanguage.googleapis.com/v1/models?key={}", api_key);
            if let Ok(resp) = self.http.get(&url).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(models_arr) = json.get("models").and_then(|v| v.as_array()) {
                        for m in models_arr {
                            if let Some(name) = m.get("name").and_then(|v| v.as_str()) {
                                // Strip "models/" prefix
                                let id = name.strip_prefix("models/").unwrap_or(name);
                                // Filter for generateContent support
                                if let Some(methods) = m.get("supportedGenerationMethods").and_then(|v| v.as_array()) {
                                    if methods.iter().any(|v| v.as_str() == Some("generateContent")) {
                                        models.push(crate::ModelInfo {
                                            id: id.to_string(),
                                            display_name: m.get("displayName").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                            created_at: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Fetch from GitHub Copilot (OIDC / Inference API)
        if let Some(github_token) = &self.config.github_token {
            let url = "https://models.inference.ai.azure.com/models";
            let mut headers = HeaderMap::new();
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", github_token))?);
            
            if let Ok(resp) = self.http.get(url).headers(headers).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(models_arr) = json.as_array() {
                        for m in models_arr {
                            if let Some(name) = m.get("name").and_then(|v| v.as_str()) {
                                models.push(crate::ModelInfo {
                                    id: name.to_string(),
                                    display_name: m.get("friendly_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                    created_at: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(models)
    }
}

// ─── Gemini Data Types ───────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GeminiUsageMetadata {
    prompt_token_count: Option<u64>,
    candidates_token_count: Option<u64>,
    total_token_count: Option<u64>,
    cached_content_token_count: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: Option<GeminiContent>,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GeminiContent {
    parts: Option<Vec<GeminiPart>>,
}

#[derive(Deserialize, Debug)]
struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    function_call: Option<GeminiFunctionCall>,
    #[serde(rename = "functionResponse", skip_serializing_if = "Option::is_none")]
    function_response: Option<GeminiFunctionResponse>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeminiFunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeminiFunctionCall {
    name: String,
    args: serde_json::Value,
}
