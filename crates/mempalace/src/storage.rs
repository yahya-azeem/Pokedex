use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::palace::MemoryEntry;

pub struct MemStorage {
    conn: Arc<Mutex<Connection>>,
}

impl MemStorage {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open MemPalace at {}", path.display()))?;
        
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE IF NOT EXISTS wings (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS halls (
                id TEXT PRIMARY KEY,
                wing_id TEXT NOT NULL,
                name TEXT NOT NULL,
                FOREIGN KEY(wing_id) REFERENCES wings(id)
             );
             CREATE TABLE IF NOT EXISTS rooms (
                id TEXT PRIMARY KEY,
                hall_id TEXT NOT NULL,
                name TEXT NOT NULL,
                FOREIGN KEY(hall_id) REFERENCES halls(id)
             );
             CREATE TABLE IF NOT EXISTS closets (
                id TEXT PRIMARY KEY,
                room_id TEXT NOT NULL,
                name TEXT NOT NULL,
                FOREIGN KEY(room_id) REFERENCES rooms(id)
             );
             CREATE TABLE IF NOT EXISTS drawers (
                id TEXT PRIMARY KEY,
                closet_id TEXT NOT NULL,
                name TEXT NOT NULL,
                FOREIGN KEY(closet_id) REFERENCES closets(id)
             );
             CREATE TABLE IF NOT EXISTS entries (
                id TEXT PRIMARY KEY,
                drawer_id TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT NOT NULL,
                importance REAL NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY(drawer_id) REFERENCES drawers(id)
             );
             -- Knowledge Graph: Triples
             CREATE TABLE IF NOT EXISTS triples (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                subject TEXT NOT NULL,
                predicate TEXT NOT NULL,
                object TEXT NOT NULL,
                context_id TEXT, -- Can be wing_id or entry_id
                timestamp TEXT NOT NULL
             );
             COMMIT;"
        )?;
        Ok(())
    }

    pub fn save_entry(&self, drawer_id: Uuid, entry: &MemoryEntry) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO entries (id, drawer_id, content, metadata, importance, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entry.id.to_string(),
                drawer_id.to_string(),
                entry.content,
                serde_json::to_string(&entry.metadata)?,
                entry.importance,
                entry.timestamp.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn save_triple(&self, subject: &str, predicate: &str, object: &str, context_id: Option<Uuid>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO triples (subject, predicate, object, context_id, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                subject,
                predicate,
                object,
                context_id.map(|u| u.to_string()),
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn ensure_wing(&self, name: &str) -> Result<Uuid> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM wings WHERE name = ?1")?;
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            return Ok(Uuid::parse_str(&id_str)?);
        }
        
        let id = Uuid::new_v4();
        conn.execute("INSERT INTO wings (id, name) VALUES (?1, ?2)", params![id.to_string(), name])?;
        Ok(id)
    }

    pub fn ensure_hall(&self, wing_id: Uuid, name: &str) -> Result<Uuid> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM halls WHERE wing_id = ?1 AND name = ?2")?;
        let mut rows = stmt.query(params![wing_id.to_string(), name])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            return Ok(Uuid::parse_str(&id_str)?);
        }
        let id = Uuid::new_v4();
        conn.execute("INSERT INTO halls (id, wing_id, name) VALUES (?1, ?2, ?3)", params![id.to_string(), wing_id.to_string(), name])?;
        Ok(id)
    }

    pub fn ensure_room(&self, hall_id: Uuid, name: &str) -> Result<Uuid> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM rooms WHERE hall_id = ?1 AND name = ?2")?;
        let mut rows = stmt.query(params![hall_id.to_string(), name])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            return Ok(Uuid::parse_str(&id_str)?);
        }
        let id = Uuid::new_v4();
        conn.execute("INSERT INTO rooms (id, hall_id, name) VALUES (?1, ?2, ?3)", params![id.to_string(), hall_id.to_string(), name])?;
        Ok(id)
    }

    pub fn ensure_closet(&self, room_id: Uuid, name: &str) -> Result<Uuid> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM closets WHERE room_id = ?1 AND name = ?2")?;
        let mut rows = stmt.query(params![room_id.to_string(), name])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            return Ok(Uuid::parse_str(&id_str)?);
        }
        let id = Uuid::new_v4();
        conn.execute("INSERT INTO closets (id, room_id, name) VALUES (?1, ?2, ?3)", params![id.to_string(), room_id.to_string(), name])?;
        Ok(id)
    }

    pub fn ensure_drawer(&self, closet_id: Uuid, name: &str) -> Result<Uuid> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM drawers WHERE closet_id = ?1 AND name = ?2")?;
        let mut rows = stmt.query(params![closet_id.to_string(), name])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            return Ok(Uuid::parse_str(&id_str)?);
        }
        let id = Uuid::new_v4();
        conn.execute("INSERT INTO drawers (id, closet_id, name) VALUES (?1, ?2, ?3)", params![id.to_string(), closet_id.to_string(), name])?;
        Ok(id)
    }

    pub fn list_entries_for_wing(&self, wing_id: Uuid) -> Result<Vec<MemoryEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT e.id, e.content, e.metadata, e.importance, e.timestamp 
             FROM entries e
             JOIN drawers d ON e.drawer_id = d.id
             JOIN closets c ON d.closet_id = c.id
             JOIN rooms r ON c.room_id = r.id
             JOIN halls h ON r.hall_id = h.id
             WHERE h.wing_id = ?1"
        )?;
        let rows = stmt.query_map(params![wing_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let metadata_str: String = row.get(2)?;
            let timestamp_str: String = row.get(4)?;
            Ok(MemoryEntry {
                id: Uuid::parse_str(&id_str).unwrap(),
                content: row.get(1)?,
                metadata: serde_json::from_str(&metadata_str).unwrap(),
                importance: row.get(3)?,
                timestamp: DateTime::parse_from_rfc3339(&timestamp_str).unwrap().with_timezone(&Utc),
            })
        })?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }
}
