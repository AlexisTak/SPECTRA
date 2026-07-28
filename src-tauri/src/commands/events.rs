//! Events commands module
//!
//! Manages case events and notes.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::command;
use crate::database::{generate_uuid, AppState};
use chrono::Utc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CaseEvent {
    pub id: String,
    pub case_id: String,
    pub r#type: String,
    pub titre: Option<String>,
    pub description: Option<String>,
    pub timestamp: String,
    pub actor: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// =============================================================================
// GET EVENTS
// =============================================================================

#[command]
pub async fn get_events(
    state: tauri::State<'_, AppState>,
    case_id: String,
) -> Result<Vec<CaseEvent>> {
    let mut conn = state.get_conn().await;

    let mut stmt = conn.prepare("SELECT id, caseId, type, titre, description, timestamp, actor, metadata FROM case_events WHERE caseId = ? ORDER BY timestamp DESC")?;

    let events = stmt.query_map(rusqlite::params![case_id], |row| {
        Ok(CaseEvent {
            id: row.get(0)?,
            case_id: row.get(1)?,
            r#type: row.get(2)?,
            titre: row.get(3)?,
            description: row.get(4)?,
            timestamp: row.get(5)?,
            actor: row.get(6)?,
            metadata: row.get(7)?,
        })
    })?;

    let result: Result<Vec<_>, _> = events.collect();
    Ok(result?)
}

// =============================================================================
// ADD NOTE
// =============================================================================

#[command]
pub async fn add_note(
    state: tauri::State<'_, AppState>,
    case_id: String,
    description: String,
) -> Result<CaseEvent> {
    let mut conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();
    let id = generate_uuid();

    // Check case exists
    let case_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM cases WHERE id = ?",
        rusqlite::params![case_id],
        |row| row.get(0),
    )?;
    if case_exists == 0 {
        return Err(anyhow::anyhow!("Case not found: {}", case_id));
    }

    conn.execute(
        "INSERT INTO case_events (id, caseId, type, titre, description, timestamp, actor, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &id,
            &case_id,
            "note",
            "Note added",
            &description,
            &now,
            "system",
            &serde_json::json!({"type": "note"}),
        ],
    )?;

    // Re-fetch to get full record
    let mut stmt = conn.prepare("SELECT id, caseId, type, titre, description, timestamp, actor, metadata FROM case_events WHERE id = ?")?;
    let event = stmt.query_row(rusqlite::params![id], |row| {
        Ok(CaseEvent {
            id: row.get(0)?,
            case_id: row.get(1)?,
            r#type: row.get(2)?,
            titre: row.get(3)?,
            description: row.get(4)?,
            timestamp: row.get(5)?,
            actor: row.get(6)?,
            metadata: row.get(7)?,
        })
    })?;

    Ok(event)
}
