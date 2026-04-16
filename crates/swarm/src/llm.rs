use anyhow::Result;
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION}};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug};
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use std::collections::HashMap;

// â”€â”€â”€ Configuration Defaults (Soft-coded) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
const DEFAULT_GEMINI_BASE: &str = "https://generativelanguage.googleapis.com";
const DEFAULT_GITHUB_BASE: &str = "https://models.github.ai";
const GITHUB_API_VERSION: &str = "2026-03-10";
const CACHE_EXPIRY_SECS: u64 = 86400; // 24 hours
const ERROR_TRUNCATE_LEN: usize = 200;

// â”€â”€â”€ Model Tiers (MiroFish Ideology) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    Simulation,
    General,
    Research,
    Expert,
}

impl ModelTier {
    pub fn from_role(role: &str) -> Self {
        let r = role.to_lowercase();
        if r.contains("simulat") || r.contains("citizen") || r.contains("voter")
            || r.contains("crowd") || r.contains("npc") || r.contains("participant")
            || r.contains("observer") || r.contains("persona")
            || r.contains("general public") || r.contains("consumer")
            || r.contains("end user") || r.contains("layperson")
        {
            ModelTier::Simulation
        } else if r.contains("code") || r.contains("engineer") || r.contains("developer")
            || r.contains("architect") || r.contains("expert") || r.contains("senior")
            || r.contains("swe") || r.contains("pentest") || r.contains("red team")
            || r.contains("offensive") || r.contains("exploit")
        {
            ModelTier::Expert
        } else if r.contains("research") || r.contains("analyst") || r.contains("scientist")
            || r.contains("strateg") || r.contains("investigat")
            || r.contains("soc") || r.contains("blue team") || r.contains("security")
            || r.contains("threat") || r.contains("incident") || r.contains("forensic")
        {
            ModelTier::Research
        } else {
            ModelTier::General
        }
    }

    pub fn to_category(&self) -> &str {
        match self {
            Self::Simulation => "marketing",
            Self::General => "product",
            Self::Research => "strategy",
            Self::Expert => "engineering",
        }
    }
}

// â”€â”€â”€ Provider Metadata and Discovery â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModelDiscovery {
    id: String,
    provider: Provider,
    display_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
enum Provider {
    Gemini,
    Copilot,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelCache {
    updated_at: u64,
    models: Vec<ModelDiscovery>,
}

#[derive(Debug, Deserialize)]
struct ModelScores {
    models: HashMap<String, HashMap<String, f64>>,
}

/// A pool of API keys with round-robin rotation.
pub struct ApiKeyPool {
    keys: Vec<String>,
    counter: AtomicUsize,
}

impl ApiKeyPool {
    pub fn new(keys: Vec<String>) -> Self {
        Self { keys, counter: AtomicUsize::new(0) }
    }
    pub fn next(&self) -> Option<&String> {
        if self.keys.is_empty() { return None; }
        let idx = self.counter.fetch_add(1, Ordering::SeqCst) % self.keys.len();
        self.keys.get(idx)
    }
    pub fn len(&self) -> usize { self.keys.len() }
}

pub struct MultiProviderClient {
    client: Client,
    gemini_pool: ApiKeyPool,
    copilot_pool: ApiKeyPool,
    model_scores: Option<ModelScores>,
    fallback_chains: HashMap<ModelTier, Vec<ModelEndpoint>>,
    config: SwarmLlmConfig,
}

#[derive(Debug, Clone)]
struct ModelEndpoint {
    provider: Provider,
    model: String,
    display_name: String,
}

#[derive(Debug, Clone)]
pub struct SwarmLlmConfig {
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub gemini_base_url: String,
    pub github_base_url: String,
}

impl Default for SwarmLlmConfig {
    fn default() -> Self {
        Self {
            max_retries: env::var("LLM_MAX_RETRIES").ok().and_then(|s| s.parse().ok()).unwrap_or(4),
            retry_delay: Duration::from_secs(env::var("LLM_RETRY_DELAY").ok().and_then(|s| s.parse().ok()).unwrap_or(2)),
            gemini_base_url: env::var("GEMINI_API_BASE").unwrap_or_else(|_| DEFAULT_GEMINI_BASE.to_string()),
            github_base_url: env::var("GITHUB_MODELS_BASE").unwrap_or_else(|_| DEFAULT_GITHUB_BASE.to_string()),
        }
    }
}

// â”€â”€â”€ Discovery API Response Types â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[derive(Deserialize)]
struct GeminiModelsResponse { models: Vec<GeminiModelMetadata> }
#[derive(Deserialize)]
struct GeminiModelMetadata { name: String, displayName: String }

#[derive(Deserialize)]
struct GithubCatalogModel { id: String, name: String }

// â”€â”€â”€ LLM API types â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[derive(Serialize, Debug)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiContent { role: Option<String>, parts: Vec<GeminiPart> }
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiPart { text: String }
#[derive(Deserialize, Debug)]
struct GeminiResponse { candidates: Vec<Candidate> }
#[derive(Deserialize, Debug)]
struct Candidate { content: GeminiContent }

#[derive(Serialize, Debug)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct OpenAIMessage { role: String, content: String }
#[derive(Deserialize, Debug)]
struct OpenAIResponse { choices: Vec<OpenAIChoice> }
#[derive(Deserialize, Debug)]
struct OpenAIChoice { message: OpenAIMessageResponse }
#[derive(Deserialize, Debug)]
struct OpenAIMessageResponse { content: String }

// â”€â”€â”€ Implementation â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

impl MultiProviderClient {
    pub async fn new() -> Self {
        let config = SwarmLlmConfig::default();
        let client = Client::new();

        let gemini_keys = env::var("GOOGLE_API_KEYS").or_else(|_| env::var("GOOGLE_API_KEY"))
            .map(|s| s.split(|c| c == ',' || c == ';').map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).collect())
            .unwrap_or_default();
        
        let copilot_tokens = env::var("GITHUB_TOKENS").or_else(|_| env::var("GITHUB_TOKEN"))
            .map(|s| s.split(|c| c == ',' || c == ';').map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).collect())
            .unwrap_or_default();

        if gemini_keys.is_empty() && copilot_tokens.is_empty() {
             panic!("At least one GOOGLE_API_KEY or GITHUB_TOKEN must be set.");
        }

        let gemini_pool = ApiKeyPool::new(gemini_keys);
        let copilot_pool = ApiKeyPool::new(copilot_tokens);

        // Load Promptfoo scores
        let scores = std::fs::read_to_string("promptfoo/model_scores.json")
            .ok()
            .and_then(|c| serde_json::from_str::<ModelScores>(&c).ok());

        // Perform Dynamic Discovery
        let models = Self::discover_models(&client, &gemini_pool, &copilot_pool, &config).await;

        let fallback_chains = Self::build_optimized_chains(&scores, &models);

        info!("ðŸ Ÿ Swarm LLM Load Balancer initialized (Dynamic Discovery):");
        info!("   Available Models discovered: {}", models.len());
        info!("   Gemini:  âœ… ({} keys)", gemini_pool.len());
        info!("   Copilot: âœ… ({} keys)", copilot_pool.len());

        Self {
            client,
            gemini_pool,
            copilot_pool,
            model_scores: scores,
            fallback_chains,
            config,
        }
    }

    async fn discover_models(client: &Client, gemini: &ApiKeyPool, copilot: &ApiKeyPool, config: &SwarmLlmConfig) -> Vec<ModelDiscovery> {
        let cache_path = dirs::home_dir().map(|h| h.join(".pokedex").join("model_cache.json"));
        
        // Try Cache
        if let Some(ref path) = cache_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(cache) = serde_json::from_str::<ModelCache>(&content) {
                    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                    if now - cache.updated_at < CACHE_EXPIRY_SECS {
                        debug!("Using cached model discovery list (age: {}s)", now - cache.updated_at);
                        return cache.models;
                    }
                }
            }
        }

        let mut discovered = Vec::new();

        // 1. Fetch Gemini
        if let Some(key) = gemini.next() {
            let url = format!("{}/v1beta/models?key={}", config.gemini_base_url, key);
            if let Ok(res) = client.get(&url).send().await {
                if let Ok(data) = res.json::<GeminiModelsResponse>().await {
                    for m in data.models {
                        discovered.push(ModelDiscovery {
                            id: m.name,
                            provider: Provider::Gemini,
                            display_name: m.displayName,
                        });
                    }
                }
            }
        }

        // 2. Fetch GitHub
        if let Some(token) = copilot.next() {
            let url = format!("{}/catalog/models", config.github_base_url);
            let mut headers = HeaderMap::new();
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).unwrap());
            headers.insert("X-GitHub-Api-Version", HeaderValue::from_static(GITHUB_API_VERSION));
            
            if let Ok(res) = client.get(&url).headers(headers).send().await {
                if let Ok(data) = res.json::<Vec<GithubCatalogModel>>().await {
                    for m in data {
                        discovered.push(ModelDiscovery {
                            id: m.id,
                            provider: Provider::Copilot,
                            display_name: format!("Copilot {}", m.name),
                        });
                    }
                }
            }
        }

        // Save to cache
        if !discovered.is_empty() {
             if let Some(ref path) = cache_path {
                 let cache = ModelCache {
                     updated_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
                     models: discovered.clone(),
                 };
                 if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).ok(); }
                 if let Ok(json) = serde_json::to_string_pretty(&cache) {
                     let _ = std::fs::write(path, json);
                 }
             }
        } else {
            // Fallback to minimal sensible defaults if discovery fails COMPLETELY
            warn!("âš ï¸  Dynamic model discovery failed. Using static emergency fallbacks.");
            discovered = vec![
                ModelDiscovery { id: "gemini-2.0-flash".into(), provider: Provider::Gemini, display_name: "Gemini 2.0 Flash (Fallback)".into() },
                ModelDiscovery { id: "gpt-4o-mini".into(), provider: Provider::Copilot, display_name: "Copilot GPT-4o Mini (Fallback)".into() },
            ];
        }

        discovered
    }

    fn build_optimized_chains(scores: &Option<ModelScores>, discovered: &[ModelDiscovery]) -> HashMap<ModelTier, Vec<ModelEndpoint>> {
        let tiers = [ModelTier::Simulation, ModelTier::General, ModelTier::Research, ModelTier::Expert];
        let mut chains = HashMap::new();

        for tier in tiers {
            let category = tier.to_category();
            let mut candidates: Vec<ModelEndpoint> = discovered.iter().map(|d| ModelEndpoint {
                provider: d.provider.clone(),
                model: d.id.clone(),
                display_name: d.display_name.clone(),
            }).collect();

            // Sort by Promptfoo scores
            if let Some(s) = scores {
                candidates.sort_by(|a, b| {
                    let score_a = s.models.get(&a.model).and_then(|m| m.get(category)).cloned().unwrap_or(0.0);
                    let score_b = s.models.get(&b.model).and_then(|m| m.get(category)).cloned().unwrap_or(0.0);
                    score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                });
            } else {
                // Heuristic sort if no scores
                match tier {
                    ModelTier::Simulation => candidates.sort_by_key(|c| !c.model.contains("mini") && !c.model.contains("lite")),
                    ModelTier::Expert => candidates.sort_by_key(|c| c.model.contains("mini") || c.model.contains("lite")),
                    _ => {}
                }
            }

            chains.insert(tier, candidates);
        }
        chains
    }

    pub async fn complete(&self, tier: ModelTier, system_prompt: &str, user_content: &str) -> Result<(String, String)> {
        let chain = self.fallback_chains.get(&tier)
            .ok_or_else(|| anyhow::anyhow!("No fallback chain for tier {:?}", tier))?;

        let mut last_error = "No models available".to_string();

        for endpoint in chain {
            for attempt in 0..self.config.max_retries {
                if attempt > 0 {
                    tokio::time::sleep(self.config.retry_delay * attempt).await;
                    debug!("Retrying {} (attempt {})", endpoint.display_name, attempt + 1);
                }

                let res = match endpoint.provider {
                    Provider::Gemini => self.call_gemini(&endpoint.model, system_prompt, user_content).await,
                    Provider::Copilot => self.call_copilot(&endpoint.model, system_prompt, user_content).await,
                };

                match res {
                    Ok(content) => return Ok((content, endpoint.display_name.clone())),
                    Err(e) => {
                        let err_str = format!("{}", e);
                        let is_rate_limit = err_str.contains("429") || err_str.contains("RESOURCE_EXHAUSTED") || err_str.contains("rate");
                        last_error = if err_str.len() > ERROR_TRUNCATE_LEN { err_str[..ERROR_TRUNCATE_LEN].to_string() } else { err_str };
                        if is_rate_limit { continue; }
                        warn!("âš ï¸  {} failed: {}", endpoint.display_name, &last_error);
                        break; 
                    }
                }
            }
        }
        anyhow::bail!("All discovered models in {:?} tier exhausted. Last error: {}", tier, last_error)
    }

    async fn call_gemini(&self, model: &str, system_prompt: &str, user_content: &str) -> Result<String> {
        let api_key = self.gemini_pool.next().ok_or_else(|| anyhow::anyhow!("No Gemini API keys"))?;
        
        // Clean model name if it already has 'models/' prefix
        let model_path = if model.starts_with("models/") { model.to_string() } else { format!("models/{}", model) };
        let url = format!("{}/v1beta/{}:generateContent?key={}", self.config.gemini_base_url, model_path, api_key);

        let request_body = GeminiRequest {
            system_instruction: Some(GeminiContent {
                role: None,
                parts: vec![GeminiPart { text: system_prompt.to_string() }],
            }),
            contents: vec![GeminiContent {
                role: Some("user".to_string()),
                parts: vec![GeminiPart { text: user_content.to_string() }],
            }],
        };

        let response = self.client.post(&url).header(CONTENT_TYPE, "application/json").json(&request_body).send().await?;
        if !response.status().is_success() {
            anyhow::bail!("Gemini error ({}): {}", response.status(), response.text().await?);
        }

        let completion: GeminiResponse = response.json().await?;
        Ok(completion.candidates.first().and_then(|c| c.content.parts.first()).map(|p| p.text.clone()).unwrap_or_default())
    }

    async fn call_copilot(&self, model: &str, system_prompt: &str, user_content: &str) -> Result<String> {
        let token = self.copilot_pool.next().ok_or_else(|| anyhow::anyhow!("No Copilot tokens"))?;
        let url = format!("{}/chat/completions", self.config.github_base_url);

        let request_body = OpenAIRequest {
            model: model.to_string(),
            messages: vec![
                OpenAIMessage { role: "system".into(), content: system_prompt.to_string() },
                OpenAIMessage { role: "user".into(), content: user_content.to_string() },
            ],
            max_tokens: Some(16_384),
        };

        let response = self.client.post(&url)
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .json(&request_body).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Copilot error ({}): {}", response.status(), response.text().await?);
        }

        let completion: OpenAIResponse = response.json().await?;
        Ok(completion.choices.first().map(|c| c.message.content.clone()).unwrap_or_default())
    }

    pub async fn complete_with_model(&self, model: &str, system_prompt: &str, user_content: &str) -> Result<(String, String)> {
        let res = if model.contains('/') || model.contains("gpt") {
             // Heuristic: models with / are usually GitHub (publisher/model)
             self.call_copilot(model, system_prompt, user_content).await
        } else {
             self.call_gemini(model, system_prompt, user_content).await
        };

        match res {
            Ok(c) => Ok((c, model.to_string())),
            Err(_) => self.complete(ModelTier::from_role(model), system_prompt, user_content).await,
        }
    }
}

pub type GeminiClient = MultiProviderClient;
