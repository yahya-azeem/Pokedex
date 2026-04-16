pub mod palace;
pub mod storage;
pub mod search;

use anyhow::Result;
use std::path::Path;
use uuid::Uuid;
use chrono::Utc;
use serde_json::json;

pub struct MemPalace {
    storage: storage::MemStorage,
    search: search::SearchEngine,
}

impl MemPalace {
    pub fn new(path: &Path) -> Result<Self> {
        Ok(Self {
            storage: storage::MemStorage::new(path)?,
            search: search::SearchEngine::new(),
        })
    }

    /// Store a memory in a specific organizational drawer.
    pub async fn remember(&self, wing_name: &str, room_name: &str, content: &str) -> Result<Uuid> {
        let wing_id = self.storage.ensure_wing(wing_name)?;
        let hall_id = self.storage.ensure_hall(wing_id, "Main")?;
        let room_id = self.storage.ensure_room(hall_id, room_name)?;
        let closet_id = self.storage.ensure_closet(room_id, "General")?;
        let drawer_id = self.storage.ensure_drawer(closet_id, "Default")?;

        let id = Uuid::new_v4();
        let entry = palace::MemoryEntry {
            id,
            content: content.to_string(),
            metadata: json!({ "wing": wing_name, "room": room_name }),
            timestamp: Utc::now(),
            importance: 1.0,
        };

        self.storage.save_entry(drawer_id, &entry)?;
        Ok(id)
    }

    /// Add a knowledge triple (subject-predicate-object) for relational memory.
    pub fn associate(&self, subject: &str, predicate: &str, object: &str, context_id: Option<Uuid>) -> Result<()> {
        self.storage.save_triple(subject, predicate, object, context_id)
    }

    /// Retrieve memories relevant to a query within a specific wing.
    pub async fn recall(&self, wing_name: &str, query: &str, limit: usize) -> Result<Vec<String>> {
        let wing_id = self.storage.ensure_wing(wing_name)?;
        let entries = self.storage.list_entries_for_wing(wing_id)?;
        
        if entries.is_empty() {
            return Ok(Vec::new());
        }

        let documents: Vec<String> = entries.iter().map(|e| e.content.clone()).collect();
        let ranked = self.search.compute_tfidf(query, &documents);
        
        let results = ranked.into_iter()
            .take(limit)
            .map(|(idx, _)| documents[idx].clone())
            .collect();

        Ok(results)
    }
}
