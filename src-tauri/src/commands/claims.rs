//! Claims commands module
//!
//! Manages claims about evidence and subjects.

use anyhow::Result;
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
    pub by_qualification: serde_json::Map<String, i64>,
    pub by_source: serde_json::Map<String, i64>,
}

// =============================================================================
// SET CLAIM
// =============================================================================

#[command]
pub async fn set_claim(
    state: tauri::State<'_, AppState>,
    input: SetClaimInput,
) -> Result<Claim> {
    let mut conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();
    let id = generate_uuid();

    // Validate qualification
    if !["preuve", "indice", "hypothese", "non_verifie"].contains(&input.qualification.as_str()) {
        return Err(anyhow::anyhow!("Invalid qualification: {}", input.qualification));
    }

    // Validate fiabilite
    if input.fiabilite < 0 || input.fiabilite > 5 {
        return Err(anyhow::anyhow!("Fiabilite must be between 0 and 5"));
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

fn get_claim_single(conn: &mut rusqlite::Connection, ref_kind: &str, ref_id: &str) -> Result<Claim> {
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
) -> Result<Vec<Claim>> {
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
) -> Result<Option<Claim>> {
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
) -> Result<()> {
    let mut conn = state.get_conn().await;

    // Delete claim
    conn.execute(
        "DELETE FROM claims WHERE id = ?",
        rusqlite::params![id],
    )?;

    // Add case event
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO case_events (id, caseId, type, titre, description, timestamp, actor, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &format!("CE-{}-000001", Utc::now().year()),
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
) -> Result<ClaimStats> {
    let mut conn = state.get_conn().await;

    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM claims WHERE caseId = ?",
        rusqlite::params![case_id],
        |row| row.get(0),
    )?;

    // Count by qualification
    let by_qualification: Vec<(String, i64)> = conn.query_map(
        "SELECT qualification, COUNT(*) FROM claims WHERE caseId = ? GROUP BY qualification",
        rusqlite::params![case_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?.collect::<Result<_, _>>()?;

    let mut by_qual_map = serde_json::Map::new();
    for (qual, count) in by_qualification {
        by_qual_map.insert(qual, count);
    }

    // Count by source
    let by_source: Vec<(String, i64)> = conn.query_map(
        "SELECT source, COUNT(*) FROM claims WHERE caseId = ? GROUP BY source",
        rusqlite::params![case_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?.collect::<Result<_, _>>()?;

    let mut by_source_map = serde_json::Map::new();
    for (src, count) in by_source {
        by_source_map.insert(src, count);
    }

    Ok(ClaimStats {
        total,
        by_qualification: by_qual_map,
        by_source: by_source_map,
    })
}
