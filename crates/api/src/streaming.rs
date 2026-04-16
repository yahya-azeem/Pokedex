use serde::{Deserialize, Serialize};
use pokedex_core::types::{Message, UsageInfo};

#[async_trait::async_trait]
pub trait StreamHandler: Send + Sync {
    fn on_event(&self, event: &StreamEvent);
}

pub struct NullStreamHandler;
#[async_trait::async_trait]
impl StreamHandler for NullStreamHandler {
    fn on_event(&self, _event: &StreamEvent) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    MessageStart {
        message: serde_json::Value,
    },
    ContentBlockStart {
        index: usize,
        content_block: serde_json::Value,
    },
    ContentBlockDelta {
        index: usize,
        delta: ContentDelta,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageDelta {
        delta: serde_json::Value,
        usage: serde_json::Value,
    },
    MessageStop {
        stop_reason: Option<String>,
        usage: Option<UsageInfo>,
    },
    Error {
        error_type: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentDelta {
    TextDelta { text: String },
    ThinkingDelta { thinking: String },
    SignatureDelta { signature: String },
}

pub struct StreamAccumulator {
    text: String,
    blocks: Vec<serde_json::Value>,
    stop_reason: Option<String>,
    usage: UsageInfo,
}

impl StreamAccumulator {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            blocks: Vec::new(),
            stop_reason: None,
            usage: UsageInfo::default(),
        }
    }

    pub fn on_event(&mut self, event: &StreamEvent) {
        match event {
            StreamEvent::ContentBlockStart { content_block, .. } => {
                self.blocks.push(content_block.clone());
            }
            StreamEvent::ContentBlockDelta { delta, .. } => {
                match delta {
                    ContentDelta::TextDelta { text } => self.text.push_str(text),
                     ContentDelta::ThinkingDelta { thinking } => {
                          // Add thinking to blocks if not present
                          if !self.blocks.iter().any(|b| b["type"] == "thinking") {
                              self.blocks.push(serde_json::json!({
                                  "type": "thinking",
                                  "thinking": thinking,
                                  "signature": ""
                              }));
                          } else {
                              // Append to existing thinking block (simplified)
                              if let Some(b) = self.blocks.iter_mut().find(|b| b["type"] == "thinking") {
                                  if let Some(t) = b["thinking"].as_str() {
                                      b["thinking"] = serde_json::json!(format!("{}{}", t, thinking));
                                      b["signature"] = serde_json::json!("");
                                  }
                              }
                          }
                     }
                    _ => {}
                }
            }
            StreamEvent::MessageDelta { usage, .. } => {
                if let Ok(u) = serde_json::from_value::<UsageInfo>(usage.clone()) {
                    self.usage.input_tokens += u.input_tokens;
                    self.usage.output_tokens += u.output_tokens;
                    self.usage.cache_creation_input_tokens += u.cache_creation_input_tokens;
                    self.usage.cache_read_input_tokens += u.cache_read_input_tokens;
                }
            }
            StreamEvent::MessageStop { stop_reason, usage } => {
                if let Some(reason) = stop_reason {
                    self.stop_reason = Some(reason.clone());
                }
                if let Some(u) = usage {
                    self.usage = u.clone();
                }
            }
            _ => {}
        }
    }

    pub fn finish(self) -> (Message, UsageInfo, Option<String>) {
        let mut final_blocks = self.blocks;
        if !self.text.is_empty() {
            final_blocks.push(serde_json::json!({
                "type": "text",
                "text": self.text
            }));
        }

        let content = if final_blocks.is_empty() {
             pokedex_core::types::MessageContent::Text(String::new())
        } else {
             let mut blocks = Vec::new();
             for v in final_blocks {
                 match serde_json::from_value::<pokedex_core::types::ContentBlock>(v.clone()) {
                     Ok(b) => blocks.push(b),
                     Err(e) => {
                         println!("  [HANDSHAKE_ERROR] Failed to deserialize block: {}. Error: {}", v, e);
                         tracing::error!("  [GHOST_WORK_DETECTED] Failed to deserialize block: {}. Error: {}", v, e);
                         // Fall back to text if we can't deserialize a complex block
                         blocks.push(pokedex_core::types::ContentBlock::Text {
                             text: format!("[Deserialization Error]: {}", e),
                         });
                     }
                 }
             }
             pokedex_core::types::MessageContent::Blocks(blocks)
        };

        (
            Message {
                role: pokedex_core::types::Role::Assistant,
                content,
                uuid: None,
                cost: None,
            },
            self.usage,
            self.stop_reason,
        )
    }
}
