// BrowserTool: High-fidelity agentic web browsing using headless_chrome.
// Supports JS rendering, basic interaction, and Markdown conversion.

use crate::{PermissionLevel, Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use headless_chrome::{Browser, LaunchOptions};
use tracing::{debug, info};
use std::time::Duration;

pub struct BrowserTool;

#[derive(Debug, Deserialize)]
struct BrowserInput {
    url: String,
    #[serde(default)]
    click: Option<String>,
    #[serde(default)]
    r#type: Option<Value>, // { "selector": "...", "text": "..." }
    #[serde(default = "default_wait")]
    wait_ms: u64,
}

fn default_wait() -> u64 { 
    std::env::var("BROWSER_WAIT_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(1500)
}

#[async_trait]
impl Tool for BrowserTool {
    fn name(&self) -> &str {
        "browser"
    }

    fn description(&self) -> &str {
        "View and interact with web pages. Renders JavaScript and returns the content as Markdown. \
         Can click elements and type text before capturing the page."
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to navigate to"
                },
                "click": {
                    "type": "string",
                    "description": "Optional CSS selector to click before capturing"
                },
                "type": {
                    "type": "object",
                    "properties": {
                        "selector": { "type": "string" },
                        "text": { "type": "string" }
                    },
                    "description": "Optional { selector, text } to type into an input before capturing"
                },
                "wait_ms": {
                    "type": "integer",
                    "description": "Time to wait for JS rendering in ms (default: 1500)"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let params: BrowserInput = match serde_json::from_value(input) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid browser input: {}", e)),
        };

        // Permission check
        let desc = format!("Browse {}", params.url);
        if let Err(e) = ctx.check_permission(self.name(), &desc, true) {
            return ToolResult::error(e.to_string());
        }

        debug!(url = %params.url, "Launching headless browser");

        let options = LaunchOptions::default_builder()
            .headless(true)
            .build()
            .map_err(|e| ToolResult::error(format!("Failed to build browser options: {}", e)));

        let options = match options {
            Ok(o) => o,
            Err(e) => return e,
        };

        let browser = match Browser::new(options) {
            Ok(b) => b,
            Err(e) => return ToolResult::error(format!("Could not find or launch Chrome/Chromium: {}. Ensure a browser is installed.", e)),
        };

        let tab = match browser.new_tab() {
            Ok(t) => t,
            Err(e) => return ToolResult::error(format!("Failed to open new tab: {}", e)),
        };

        info!("Navigating browser to {}", params.url);
        if let Err(e) = tab.navigate_to(&params.url) {
            return ToolResult::error(format!("Navigation failed: {}", e));
        }

        // Wait for initial load
        if let Err(e) = tab.wait_until_navigated() {
            debug!("Wait until navigated timed out/failed: {}, continuing anyway", e);
        }

        // Interaction: Type
        if let Some(type_data) = params.r#type {
            if let (Some(sel), Some(txt)) = (type_data["selector"].as_str(), type_data["text"].as_str()) {
                debug!("Typing '{}' into '{}'", txt, sel);
                if let Ok(element) = tab.find_element(sel) {
                    let _ = element.click();
                    let _ = element.type_into(txt);
                }
            }
        }

        // Interaction: Click
        if let Some(sel) = params.click {
            debug!("Clicking '{}'", sel);
            if let Ok(element) = tab.find_element(&sel) {
                let _ = element.click();
            }
        }

        // Wait for JS/rendering
        std::thread::sleep(Duration::from_millis(params.wait_ms));

        // Capture content
        let html = match tab.get_content() {
            Ok(h) => h,
            Err(e) => return ToolResult::error(format!("Failed to get page content: {}", e)),
        };

        let title = tab.get_title().unwrap_or_else(|_| "Untitled".to_string());

        // Convert to Markdown
        let markdown = html2md::parse_html(&html);

        // Optional: Capture screenshot (base64)
        let screenshot_b64 = tab.capture_screenshot(headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Jpeg, None, None, true)
            .map(|data| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data))
            .ok();

        let mut output = format!("# {}\nURL: {}\n\n{}", title, params.url, markdown);
        
        // Truncate if extreme
        let max_len: usize = std::env::var("BROWSER_MAX_LEN").ok().and_then(|s| s.parse().ok()).unwrap_or(120_000);
        if output.len() > max_len {
            output = format!("{}\n\n... (truncated)", &output[..max_len]);
        }

        let mut result = ToolResult::success(output);
        if let Some(b64) = screenshot_b64 {
            result = result.with_metadata(json!({
                "screenshot": format!("data:image/jpeg;base64,{}", b64)
            }));
        }

        result
    }
}
