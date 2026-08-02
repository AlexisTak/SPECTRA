//! Events commands module
//!
//! Manages case events and notes.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::command;
use crate::database::{generate_uuid, AppState};
use chrono::Utc;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
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
) -> AppResult<Vec<CaseEvent>> {
    let conn = state.get_conn().await;

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
) -> AppResult<CaseEvent> {
    let conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();
    let id = generate_uuid();

    // Check case exists
    let case_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM cases WHERE id = ?",
        rusqlite::params![case_id],
        |row| row.get(0),
    )?;
    if case_exists == 0 {
        return Err(AppError::msg(format!("Case not found: {}", case_id)));
    }

    // Récupère le dernier hash d'audit pour ce dossier (chaînage).
    let previous_hash: Option<String> = conn
        .query_row(
            "SELECT imma FROM audit_events WHERE caseId = ? ORDER BY timestamp DESC LIMIT 1",
            rusqlite::params![&case_id],
            |row| row.get(0),
        )
        .ok();
    let previous_hash = previous_hash.unwrap_or_default();

    // Construit l'événement d'audit et calcule son hash chaîné.
    let audit_event = spectra_audit::AuditEvent {
        id: crate::database::generate_uuid(),
        case_id: case_id.clone(),
        action: "create".to_string(),
        entity_kind: "note".to_string(),
        entity_id: Some(id.clone()),
        actor: "system".to_string(),
        sequence: 1,
        payload: serde_json::json!({"description": description}),
    };
    let link = spectra_audit::compute_link(&audit_event, &previous_hash);

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

    conn.execute(
        "INSERT INTO audit_events (id, caseId, action, entityKind, entityId, imma, imma_precedent, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &audit_event.id,
            &case_id,
            &audit_event.action,
            &audit_event.entity_kind,
            &audit_event.entity_id,
            &link.hash,
            &link.previous_hash,
            &serde_json::to_string(&audit_event.payload)?,
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
