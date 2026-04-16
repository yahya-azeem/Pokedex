use pokedex_core::types::{ContentBlock, Message, MessageContent};
use serde_json::json;

fn main() {
    let tool_json = json!({
        "type": "tool_use",
        "id": "call_file_write",
        "name": "file_write",
        "input": {
            "file_path": "test.txt",
            "content": "hello world"
        }
    });

    println!("Testing deserialization of: {}", tool_json);

    match serde_json::from_value::<ContentBlock>(tool_json) {
        Ok(block) => {
            println!("SUCCESS: Deserialized block: {:?}", block);
            let msg = Message {
                role: pokedex_core::types::Role::Assistant,
                content: MessageContent::Blocks(vec![block]),
                uuid: None,
                cost: None,
            };
            println!("Tool blocks found: {}", msg.get_all_text());
            println!("Tool count: {}", msg.get_tool_use_blocks().len());
        }
        Err(e) => {
            println!("FAILURE: Could not deserialize: {}", e);
        }
    }
}
