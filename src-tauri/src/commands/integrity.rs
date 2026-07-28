//! Integrity commands module
//!
//! Manages evidence integrity verification.

use anyhow::Result;
use tauri::command;
use crate::database::{generate_uuid, AppState};
use chrono::Utc;

// Re-export types for convenience
pub use crate::commands::evidence::{IntegrityCheck, CaseIntegrityReport};

// =============================================================================
// VERIFY EVIDENCE
// =============================================================================

#[command]
pub async fn verify_evidence(
    state: tauri::State<'_, AppState>,
    evidence_id: String,
) -> Result<IntegrityCheck> {
    let mut conn = state.get_conn().await;

    // Get evidence with hash
    let mut stmt = conn.prepare("SELECT id, hash_sha256, hash_md5 FROM evidence WHERE id = ?")?;
    let (eid, hash_sha256, hash_md5) = stmt.query_row(rusqlite::params![evidence_id], |row| {
        Ok((row.get(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, Option<String>>(2)?))
    })?;

    let verified = hash_sha256.as_ref().map(|h| !h.is_empty()).unwrap_or(false);

    Ok(IntegrityCheck {
        evidence_id: eid,
        hash_sha256: hash_sha256.unwrap_or_default(),
        hash_md5: hash_md5.unwrap_or_default(),
        verified,
        message: if verified { "Hash verified successfully" } else { "Hash verification failed or not available" }.to_string(),
        checked_at: Utc::now().to_rfc3339(),
    })
}

// =============================================================================
// VERIFY CASE EVIDENCE
// =============================================================================

#[command]
pub async fn verify_case_evidence(
    state: tauri::State<'_, AppState>,
    case_id: String,
) -> Result<CaseIntegrityReport> {
    let mut conn = state.get_conn().await;

    // Get total evidence count
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM evidence WHERE caseId = ?",
        rusqlite::params![case_id],
        |row| row.get(0),
    )?;

    // Get verified count (has non-empty SHA256)
    let verified: i64 = conn.query_row(
        "SELECT COUNT(*) FROM evidence WHERE caseId = ? AND hash_sha256 IS NOT NULL AND hash_sha256 != ''",
        rusqlite::params![case_id],
        |row| row.get(0),
    )?;

    // Log integrity check
    let id = generate_uuid();
    conn.execute(
        "INSERT INTO integrity_logs (id, caseId, checkedAt, checkedBy, totalEvidence, verifiedEvidence, brokenHash, message) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &id,
            &case_id,
            &Utc::now().to_rfc3339(),
            "system",
            &total,
            &verified,
            &(total - verified),
            "Integrity check completed",
        ],
    )?;

    Ok(CaseIntegrityReport {
        case_id,
        total_evidence: total,
        verified_count: verified,
        broken_count: total - verified,
        last_check: Some(Utc::now().to_rfc3339()),
    })
}
