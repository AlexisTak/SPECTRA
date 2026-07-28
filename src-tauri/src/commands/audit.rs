//! Audit commands module
//!
//! Manages audit trail and integrity verification.

use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use tauri::command;
use crate::database::{generate_uuid, AppState};
use chrono::Utc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuditEvent {
    pub id: String,
    pub case_id: String,
    pub action: String,
    pub entity_kind: String,
    pub entity_id: Option<String>,
    pub actor: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub imma: String,
    pub imma_precedent: Option<String>,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuditTrailVerification {
    pub ok: bool,
    pub total: usize,
    pub broken_at_id: Option<String>,
    pub broken_at_index: Option<usize>,
    pub reason: Option<String>,
}

// =============================================================================
// VERIFY AUDIT TRAIL
// =============================================================================

#[command]
pub async fn verify_audit_trail(
    state: tauri::State<'_, AppState>,
    case_id: String,
) -> AppResult<AuditTrailVerification> {
    let conn = state.get_conn().await;

    // Get all audit events for case
    let mut stmt = conn.prepare("SELECT id, caseId, action, entityKind, entityId, actor, metadata, imma, imma_precedent, timestamp FROM audit_events WHERE caseId = ? ORDER BY timestamp ASC")?;
    let db_events: Vec<AuditEvent> = stmt.query_map(rusqlite::params![case_id], |row| {
        Ok(AuditEvent {
            id: row.get(0)?,
            case_id: row.get(1)?,
            action: row.get(2)?,
            entity_kind: row.get(3)?,
            entity_id: row.get(4)?,
            actor: row.get(5)?,
            metadata: row.get(6)?,
            imma: row.get(7)?,
            imma_precedent: row.get(8)?,
            timestamp: row.get(9)?,
        })
    })?.filter_map(|e| e.ok()).collect();

    let total = db_events.len();

    // Convert to spectra_audit::AuditEvent for verification
    let mut events: Vec<spectra_audit::AuditEvent> = Vec::new();
    let mut links: Vec<spectra_audit::ChainLink> = Vec::new();

    for db_event in &db_events {
        let payload = db_event.metadata.clone().unwrap_or_else(|| serde_json::json!({}));
        let sequence: u64 = db_event.id.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0);

        events.push(spectra_audit::AuditEvent {
            id: db_event.id.clone(),
            case_id: db_event.case_id.clone(),
            action: db_event.action.clone(),
            entity_kind: db_event.entity_kind.clone(),
            entity_id: db_event.entity_id.clone(),
            actor: db_event.actor.clone().unwrap_or_else(|| "unknown".to_string()),
            sequence,
            payload,
        });

        links.push(spectra_audit::ChainLink {
            hash: db_event.imma.clone(),
            previous_hash: db_event.imma_precedent.clone().unwrap_or_default(),
        });
    }

    // Use the verified verify_chain function from spectra-audit
    match spectra_audit::verify_chain(&events, &links) {
        Ok(count) => Ok(AuditTrailVerification {
            ok: true,
            total: count,
            broken_at_id: None,
            broken_at_index: None,
            reason: None,
        }),
        Err((index, reason)) => {
            let broken_event_id = events.get(index).map(|e| e.id.clone()).or_else(|| db_events.get(index).map(|e| e.id.clone()));
            Ok(AuditTrailVerification {
                ok: false,
                total,
                broken_at_id: broken_event_id,
                broken_at_index: Some(index),
                reason: Some(reason),
            })
        }
    }
}

// =============================================================================
// LIST AUDIT
// =============================================================================

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ListAuditOptions {
    pub limit: Option<i64>,
    pub action: Option<String>,
    pub entity_kind: Option<String>,
}

#[command]
pub async fn list_audit(
    state: tauri::State<'_, AppState>,
    case_id: String,
    options: Option<ListAuditOptions>,
) -> AppResult<Vec<AuditEvent>> {
    let mut conn = state.get_conn().await;
    let opts = options.unwrap_or_default();

    let mut query = String::from("SELECT id, caseId, action, entityKind, entityId, actor, metadata, imma, imma_precedent, timestamp FROM audit_events WHERE caseId = ?");
    let mut params: Vec<&dyn rusqlite::ToSql> = vec![&case_id];

    if let Some(action) = &opts.action {
        query.push_str(" AND action = ?");
        params.push(action);
    }
    if let Some(entity_kind) = &opts.entity_kind {
        query.push_str(" AND entityKind = ?");
        params.push(entity_kind);
    }

    query.push_str(" ORDER BY timestamp DESC");

    if let Some(limit) = opts.limit {
        query.push_str(&format!(" LIMIT {}", limit));
    }

    let mut stmt = conn.prepare(&query)?;
    let events = stmt.query_map(&params[..], |row| {
        Ok(AuditEvent {
            id: row.get(0)?,
            case_id: row.get(1)?,
            action: row.get(2)?,
            entity_kind: row.get(3)?,
            entity_id: row.get(4)?,
            actor: row.get(5)?,
            metadata: row.get(6)?,
            imma: row.get(7)?,
            imma_precedent: row.get(8)?,
            timestamp: row.get(9)?,
        })
    })?;

    let result: Result<Vec<_>, _> = events.collect();
    Ok(result?)
}

// =============================================================================
// LOG ACCESS
// =============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogAccessInput {
    pub case_id: String,
    pub actor: String,
    pub metadata: Option<serde_json::Value>,
}

#[command]
pub async fn log_access(
    state: tauri::State<'_, AppState>,
    input: LogAccessInput,
) -> AppResult<()> {
    let mut conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();
    let id = generate_uuid();

    // Generate IMMA hash
    let imma_data = format!("access:{}:{}:{}", input.case_id, input.actor, now);
    let imma = crate::database::sha256_hash(&imma_data);

    conn.execute(
        "INSERT INTO audit_events (id, caseId, action, entityKind, entityId, actor, metadata, imma, imma_precedent, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &id,
            &input.case_id,
            "access",
            "case",
            &input.case_id,
            &input.actor,
            &input.metadata,
            &imma,
            "NULL",
            &now,
        ],
    )?;

    Ok(())
}
