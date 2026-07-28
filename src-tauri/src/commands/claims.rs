//! Claims commands module
//!
//! Manages claims about evidence and subjects.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::command;
use crate::database::{generate_uuid, AppState};
use chrono::Utc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Claim {
    pub id: String,
    pub case_id: String,
    pub ref_kind: String,
    pub ref_id: String,
    pub qualification: String,
    pub fiabilite: i32,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub taken_by: Option<String>,
    pub notes: Option<String>,
    pub date_preuve: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SetClaimInput {
    pub case_id: String,
    pub ref_kind: String,
    pub ref_id: String,
    pub qualification: String,
    pub fiabilite: i32,
    pub source: String,
    pub source_url: Option<String>,
    pub taken_by: Option<String>,
    pub notes: Option<String>,
    pub date_preuve: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClaimStats {
    pub total: i64,
    // `serde_json::Map` est toujours `Map<String, Value>` : le paramétrer avec
    // `i64` ne compile pas. Un `HashMap` sérialise en objet JSON de la même
    // façon (`audit.md`, P0-1d).
    pub by_qualification: std::collections::HashMap<String, i64>,
    pub by_source: std::collections::HashMap<String, i64>,
}

// =============================================================================
// SET CLAIM
// =============================================================================

#[command]
pub async fn set_claim(
    state: tauri::State<'_, AppState>,
    input: SetClaimInput,
) -> AppResult<Claim> {
    let mut conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();
    let id = generate_uuid();

    // Validate qualification
    if !["preuve", "indice", "hypothese", "non_verifie"].contains(&input.qualification.as_str()) {
        return Err(AppError::msg(format!("Invalid qualification: {}", input.qualification)));
    }

    // Validate fiabilite
    if input.fiabilite < 0 || input.fiabilite > 5 {
        return Err(AppError::msg(format!("Fiabilite must be between 0 and 5")));
    }

    // Check if claim already exists for this ref
    let existing: i64 = conn.query_row(
        "SELECT COUNT(*) FROM claims WHERE refKind = ? AND refId = ?",
        rusqlite::params![input.ref_kind, input.ref_id],
        |row| row.get(0),
    )?;

    if existing > 0 {
        // Update existing claim
        conn.execute(
            "UPDATE claims SET qualification = ?, fiabilite = ?, source = ?, sourceUrl = ?, takenBy = ?, notes = ?, datePreuve = ?, metadata = ?, updated_at = ? WHERE refKind = ? AND refId = ?",
            rusqlite::params![
                &input.qualification,
                &input.fiabilite,
                &input.source,
                &input.source_url,
                &input.taken_by,
                &input.notes,
                &input.date_preuve,
                &input.metadata,
                &now,
                &input.ref_kind,
                &input.ref_id,
            ],
        )?;
    } else {
        // Insert new claim
        conn.execute(
            "INSERT INTO claims (id, caseId, refKind, refId, qualification, fiabilite, source, sourceUrl, takenBy, notes, datePreuve, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                &id,
                &input.case_id,
                &input.ref_kind,
                &input.ref_id,
                &input.qualification,
                &input.fiabilite,
                &input.source,
                &input.source_url,
                &input.taken_by,
                &input.notes,
                &input.date_preuve,
                &input.metadata,
            ],
        )?;
    }

    // Re-fetch to get full record
    get_claim_single(&mut *conn, &input.ref_kind, &input.ref_id)
}

fn get_claim_single(conn: &mut rusqlite::Connection, ref_kind: &str, ref_id: &str) -> AppResult<Claim> {
    let mut stmt = conn.prepare("SELECT id, caseId, refKind, refId, qualification, fiabilite, source, sourceUrl, takenBy, notes, datePreuve, metadata FROM claims WHERE refKind = ? AND refId = ?")?;

    let claim = stmt.query_row(rusqlite::params![ref_kind, ref_id], |row| {
        Ok(Claim {
            id: row.get(0)?,
            case_id: row.get(1)?,
            ref_kind: row.get(2)?,
            ref_id: row.get(3)?,
            qualification: row.get(4)?,
            fiabilite: row.get(5)?,
            source: row.get(6)?,
            source_url: row.get(7)?,
            taken_by: row.get(8)?,
            notes: row.get(9)?,
            date_preuve: row.get(10)?,
            metadata: row.get(11)?,
        })
    })?;

    Ok(claim)
}

// =============================================================================
// LIST CLAIMS
// =============================================================================

#[command]
pub async fn list_claims(
    state: tauri::State<'_, AppState>,
    case_id: String,
) -> AppResult<Vec<Claim>> {
    let mut conn = state.get_conn().await;

    let mut stmt = conn.prepare("SELECT id, caseId, refKind, refId, qualification, fiabilite, source, sourceUrl, takenBy, notes, datePreuve, metadata FROM claims WHERE caseId = ? ORDER BY fiabilite DESC")?;

    let claims = stmt.query_map(rusqlite::params![case_id], |row| {
        Ok(Claim {
            id: row.get(0)?,
            case_id: row.get(1)?,
            ref_kind: row.get(2)?,
            ref_id: row.get(3)?,
            qualification: row.get(4)?,
            fiabilite: row.get(5)?,
            source: row.get(6)?,
            source_url: row.get(7)?,
            taken_by: row.get(8)?,
            notes: row.get(9)?,
            date_preuve: row.get(10)?,
            metadata: row.get(11)?,
        })
    })?;

    let result: Result<Vec<_>, _> = claims.collect();
    Ok(result?)
}

// =============================================================================
// GET CLAIM
// =============================================================================

#[command]
pub async fn get_claim(
    state: tauri::State<'_, AppState>,
    ref_kind: String,
    ref_id: String,
) -> AppResult<Option<Claim>> {
    let mut conn = state.get_conn().await;

    let claim = get_claim_single(&mut *conn, &ref_kind, &ref_id).ok();

    Ok(claim)
}

// =============================================================================
// DELETE CLAIM
// =============================================================================

#[command]
pub async fn delete_claim(
    state: tauri::State<'_, AppState>,
    id: String,
    case_id: String,
) -> AppResult<()> {
    let mut conn = state.get_conn().await;

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
    let now = Utc::now().to_rfc3339();
    let audit_event = spectra_audit::AuditEvent {
        id: crate::database::generate_uuid(),
        case_id: case_id.clone(),
        action: "delete".to_string(),
        entity_kind: "claim".to_string(),
        entity_id: Some(id.clone()),
        actor: "system".to_string(),
        sequence: 1,
        payload: serde_json::json!({"claim_id": id}),
    };
    let link = spectra_audit::compute_link(&audit_event, &previous_hash);

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

    // Delete claim
    conn.execute(
        "DELETE FROM claims WHERE id = ?",
        rusqlite::params![id],
    )?;

    // Add case event
    conn.execute(
        "INSERT INTO case_events (id, caseId, type, titre, description, timestamp, actor, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &crate::database::generate_uuid(),
            &case_id,
            "claim_deleted",
            "Claim deleted",
            &format!("Removed claim {}", id),
            &now,
            "system",
            &serde_json::json!({"claim_id": id}),
        ],
    )?;

    Ok(())
}

// =============================================================================
// GET CLAIM STATS
// =============================================================================

#[command]
pub async fn get_claim_stats(
    state: tauri::State<'_, AppState>,
    case_id: String,
) -> AppResult<ClaimStats> {
    let mut conn = state.get_conn().await;

    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM claims WHERE caseId = ?",
        rusqlite::params![case_id],
        |row| row.get(0),
    )?;

    // Count by qualification
    let by_qual_map: std::collections::HashMap<String, i64> = conn
        .prepare(
            "SELECT qualification, COUNT(*) FROM claims WHERE caseId = ? GROUP BY qualification",
        )?
        .query_map(rusqlite::params![case_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?
        .collect::<Result<_, _>>()?;

    // Count by source
    let by_source_map: std::collections::HashMap<String, i64> = conn
        .prepare("SELECT source, COUNT(*) FROM claims WHERE caseId = ? GROUP BY source")?
        .query_map(rusqlite::params![case_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?
        .collect::<Result<_, _>>()?;

    Ok(ClaimStats {
        total,
        by_qualification: by_qual_map,
        by_source: by_source_map,
    })
}
