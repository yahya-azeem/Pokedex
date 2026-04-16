use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::orchestrator::Swarm;

/// MiroFish Graph Edge — represents a directional relationship between two entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub edge_type: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// MiroFish Graph Layer — tracks all relationships in the swarm ecosystem.
/// This sits alongside the Swarm state and provides queryable graph traversal.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MiroFishGraph {
    pub edges: Vec<GraphEdge>,
}

impl MiroFishGraph {
    /// Add a relationship edge to the graph.
    pub fn relate(&mut self, from: Uuid, to: Uuid, edge_type: &str) {
        // Avoid duplicate edges
        if !self.edges.iter().any(|e| e.from_id == from && e.to_id == to && e.edge_type == edge_type) {
            self.edges.push(GraphEdge {
                from_id: from,
                to_id: to,
                edge_type: edge_type.to_string(),
                metadata: None,
                created_at: chrono::Utc::now(),
            });
        }
    }

    /// Query all outgoing edges of a given type from a node.
    pub fn outgoing(&self, from: Uuid, edge_type: &str) -> Vec<&GraphEdge> {
        self.edges.iter()
            .filter(|e| e.from_id == from && e.edge_type == edge_type)
            .collect()
    }

    /// Query all incoming edges of a given type to a node.
    pub fn incoming(&self, to: Uuid, edge_type: &str) -> Vec<&GraphEdge> {
        self.edges.iter()
            .filter(|e| e.to_id == to && e.edge_type == edge_type)
            .collect()
    }

    /// Get the full collaboration network for visualization.
    pub fn collaboration_network(&self) -> serde_json::Value {
        let collaborations: Vec<serde_json::Value> = self.edges.iter()
            .filter(|e| e.edge_type == "COLLABORATES_WITH")
            .map(|e| serde_json::json!({
                "from": e.from_id,
                "to": e.to_id,
                "type": e.edge_type,
            }))
            .collect();

        let productions: Vec<serde_json::Value> = self.edges.iter()
            .filter(|e| e.edge_type == "PRODUCED")
            .map(|e| serde_json::json!({
                "agent": e.from_id,
                "output": e.to_id,
                "type": e.edge_type,
            }))
            .collect();

        serde_json::json!({
            "collaborations": collaborations,
            "productions": productions,
            "total_edges": self.edges.len(),
        })
    }
}

/// Persistent storage for swarms with MiroFish graph relationships.
/// Uses in-memory state with JSON file persistence — lightweight, native Rust, zero external deps.
pub struct SwarmStore {
    storage_path: PathBuf,
    swarms: Arc<RwLock<HashMap<Uuid, Swarm>>>,
    /// The MiroFish graph layer tracking all inter-agent relationships
    pub graph: Arc<RwLock<MiroFishGraph>>,
}

impl SwarmStore {
    /// Create a new store targeting the given directory.
    pub async fn new(storage_path: PathBuf) -> Result<Self> {
        if !storage_path.exists() {
            fs::create_dir_all(&storage_path)?;
        }

        let mut swarms = HashMap::new();
        // Load existing swarms from the directory
        if let Ok(entries) = fs::read_dir(&storage_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json")
                    && !path.file_name().map_or(false, |n| n == "graph.json")
                {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(swarm) = serde_json::from_str::<Swarm>(&content) {
                            swarms.insert(swarm.id, swarm);
                        }
                    }
                }
            }
        }

        // Load the graph if it exists
        let graph_path = storage_path.join("graph.json");
        let graph = if graph_path.exists() {
            let content = fs::read_to_string(&graph_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            MiroFishGraph::default()
        };

        Ok(Self {
            storage_path,
            swarms: Arc::new(RwLock::new(swarms)),
            graph: Arc::new(RwLock::new(graph)),
        })
    }

    /// Save or update a swarm in the store.
    pub async fn save(&self, swarm: &Swarm) -> Result<()> {
        let mut swarms = self.swarms.write().await;
        swarms.insert(swarm.id, swarm.clone());

        let path = self.storage_path.join(format!("{}.json", swarm.id));
        let content = serde_json::to_string_pretty(swarm)?;
        fs::write(path, content)?;

        // Also persist the graph
        let graph = self.graph.read().await;
        let graph_path = self.storage_path.join("graph.json");
        let graph_content = serde_json::to_string_pretty(&*graph)?;
        fs::write(graph_path, graph_content)?;

        Ok(())
    }

    /// Retrieve a swarm by ID.
    pub async fn get(&self, id: Uuid) -> Option<Swarm> {
        let swarms = self.swarms.read().await;
        swarms.get(&id).cloned()
    }

    /// List all swarms.
    pub async fn list(&self) -> Vec<Swarm> {
        let swarms = self.swarms.read().await;
        swarms.values().cloned().collect()
    }

    /// Delete a swarm from storage.
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let mut swarms = self.swarms.write().await;
        swarms.remove(&id);

        let path = self.storage_path.join(format!("{}.json", id));
        if path.exists() {
            fs::remove_file(path)?;
        }

        Ok(())
    }

    /// Establish a graph relationship between two agents (MiroFish link).
    pub async fn relate_agents(&self, from_id: Uuid, to_id: Uuid, rel_type: &str) -> Result<()> {
        let mut graph = self.graph.write().await;
        graph.relate(from_id, to_id, rel_type);
        Ok(())
    }

    /// Get the MiroFish collaboration network for a swarm.
    pub async fn get_collaboration_network(&self, _swarm_id: Uuid) -> Result<serde_json::Value> {
        let graph = self.graph.read().await;
        Ok(graph.collaboration_network())
    }
}
