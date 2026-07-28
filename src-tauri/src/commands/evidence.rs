//! Evidence commands module
//!
//! Manages evidence items with integrity verification.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::command;
use chrono::Utc;
use crate::database::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Evidence {
    pub id: String,
    pub case_id: String,
    pub r#type: String,
    pub nom: Option<String>,
    pub description: Option<String>,
    pub chemin: Option<String>,
    pub hash_sha256: Option<String>,
    pub hash_md5: Option<String>,
    pub taille: Option<i64>,
    pub date_ajout: String,
    pub statut: String,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IntegrityCheck {
    pub evidence_id: String,
    pub hash_sha256: String,
    pub hash_md5: String,
    pub verified: bool,
    pub message: String,
    pub checked_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CaseIntegrityReport {
    pub case_id: String,
    pub total_evidence: i64,
    pub verified_count: i64,
    pub broken_count: i64,
    pub last_check: Option<String>,
}

// =============================================================================
// GET EVIDENCE
// =============================================================================

#[command]
pub async fn get_evidence(
    state: tauri::State<'_, AppState>,
    case_id: String,
) -> AppResult<Vec<Evidence>> {
    let mut conn = state.get_conn().await;

    let mut stmt = conn.prepare("SELECT id, caseId, type, nom, description, chemin, hash_sha256, hash_md5, taille, dateAjout, statut, source, sourceUrl, metadata FROM evidence WHERE caseId = ? ORDER BY dateAjout DESC")?;

    let evidence = stmt.query_map(rusqlite::params![case_id], |row| {
        Ok(Evidence {
            id: row.get(0)?,
            case_id: row.get(1)?,
            r#type: row.get(2)?,
            nom: row.get(3)?,
            description: row.get(4)?,
            chemin: row.get(5)?,
            hash_sha256: row.get(6)?,
            hash_md5: row.get(7)?,
            taille: row.get(8)?,
            date_ajout: row.get(9)?,
            statut: row.get(10)?,
            source: row.get(11)?,
            source_url: row.get(12)?,
            metadata: row.get(13)?,
        })
    })?;

    let result: Result<Vec<_>, _> = evidence.collect();
    Ok(result?)
}

// =============================================================================
// ADD EVIDENCE
// =============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AddEvidenceInput {
    pub case_id: String,
    pub r#type: String,
    pub nom: Option<String>,
    pub description: Option<String>,
    pub chemin: Option<String>,
    pub hash_sha256: Option<String>,
    pub hash_md5: Option<String>,
    pub taille: Option<i64>,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AddEvidenceOptions {
    pub actor: Option<String>,
    pub source: Option<String>,
    pub reliability: Option<i32>,
    pub qualification: Option<String>,
    pub source_url: Option<String>,
}

#[command]
pub async fn add_evidence(
    state: tauri::State<'_, AppState>,
    case_id: String,
    data: AddEvidenceInput,
    options: Option<AddEvidenceOptions>,
) -> AppResult<Evidence> {
    let mut conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();

    // Check if case exists
    let case_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM cases WHERE id = ?",
        rusqlite::params![case_id],
        |row| row.get(0),
    )?;
    if case_exists == 0 {
        return Err(AppError::msg(format!("Case not found: {}", case_id)));
    }

    let id = crate::database::generate_uuid();

    conn.execute(
        "INSERT INTO evidence (id, caseId, type, nom, description, chemin, hash_sha256, hash_md5, taille, dateAjout, statut, source, sourceUrl, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &id,
            &case_id,
            &data.r#type,
            &data.nom,
            &data.description,
            &data.chemin,
            &data.hash_sha256,
            &data.hash_md5,
            &data.taille,
            &now,
            &"en_attente",
            &data.source,
            &data.source_url,
            &data.metadata,
        ],
    )?;

    // Add to FTS index
    if let Some(nom) = &data.nom {
        conn.execute(
            "INSERT INTO fts_evidence (rowid, nom, description, chemin) VALUES (?, ?, ?, ?)",
            rusqlite::params![&id, nom, &data.description, &data.chemin],
        )?;
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
        entity_kind: "evidence".to_string(),
        entity_id: Some(id.clone()),
        actor: options.as_ref().and_then(|o| o.actor.clone()).unwrap_or_else(|| "system".to_string()),
        sequence: 1,
        payload: serde_json::json!({"type": data.r#type, "nom": data.nom}),
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

    // Re-fetch to get full record
    get_evidence_single(&mut *conn, &id)
}

fn get_evidence_single(conn: &mut rusqlite::Connection, id: &str) -> AppResult<Evidence> {
    let mut stmt = conn.prepare("SELECT id, caseId, type, nom, description, chemin, hash_sha256, hash_md5, taille, dateAjout, statut, source, sourceUrl, metadata FROM evidence WHERE id = ?")?;

    let evidence = stmt.query_row(rusqlite::params![id], |row| {
        Ok(Evidence {
            id: row.get(0)?,
            case_id: row.get(1)?,
            r#type: row.get(2)?,
            nom: row.get(3)?,
            description: row.get(4)?,
            chemin: row.get(5)?,
            hash_sha256: row.get(6)?,
            hash_md5: row.get(7)?,
            taille: row.get(8)?,
            date_ajout: row.get(9)?,
            statut: row.get(10)?,
            source: row.get(11)?,
            source_url: row.get(12)?,
            metadata: row.get(13)?,
        })
    })?;

    Ok(evidence)
}

// =============================================================================
// DELETE EVIDENCE
// =============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeleteEvidenceOptions {
    pub actor: Option<String>,
}

#[command]
pub async fn delete_evidence(
    state: tauri::State<'_, AppState>,
    id: String,
    case_id: String,
    options: Option<DeleteEvidenceOptions>,
) -> AppResult<()> {
    let mut conn = state.get_conn().await;

    // Check evidence exists
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM evidence WHERE id = ?",
        rusqlite::params![id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Err(AppError::msg(format!("Evidence not found: {}", id)));
    }

    // Get evidence details before delete
    let evidence = get_evidence_single(&mut *conn, &id)?;

    // Remove from FTS index
    conn.execute(
        "DELETE FROM fts_evidence WHERE rowid = ?",
        rusqlite::params![&id],
    )?;

    // Delete evidence
    conn.execute(
        "DELETE FROM evidence WHERE id = ?",
        rusqlite::params![&id],
    )?;

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
        entity_kind: "evidence".to_string(),
        entity_id: Some(id.clone()),
        actor: options.as_ref().and_then(|o| o.actor.clone()).unwrap_or_else(|| "system".to_string()),
        sequence: 1,
        payload: serde_json::json!({"type": evidence.r#type, "nom": evidence.nom}),
    };
    let link = spectra_audit::compute_link(&audit_event, &previous_hash);

    conn.execute(
        "INSERT INTO case_events (id, caseId, type, titre, description, timestamp, actor, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &crate::database::generate_uuid(),
            &case_id,
            "evidence_deleted",
            &format!("Evidence deleted: {}", evidence.nom.unwrap_or_else(|| "Unknown".to_string())),
            &format!("Removed {} evidence from case", evidence.r#type),
            &now,
            &options.as_ref().and_then(|o| o.actor.clone()),
            &serde_json::to_string(&options)?,
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

    Ok(())
}
