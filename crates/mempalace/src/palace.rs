use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Uuid,
    pub content: String,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub importance: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drawer {
    pub id: Uuid,
    pub name: String,
    pub entries: Vec<MemoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Closet {
    pub id: Uuid,
    pub name: String,
    pub drawers: Vec<Drawer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub closets: Vec<Closet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hall {
    pub id: Uuid,
    pub name: String,
    pub rooms: Vec<Room>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wing {
    pub id: Uuid,
    pub name: String,
    pub halls: Vec<Hall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Palace {
    pub wings: Vec<Wing>,
}

impl Palace {
    pub fn new() -> Self {
        Self { wings: Vec::new() }
    }
}
