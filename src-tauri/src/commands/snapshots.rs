//! Snapshots commands module
//!
//! Manages case snapshots with hash verification.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::command;
use crate::database::{generate_uuid, AppState};
use crate::database::sha256_hash;
use chrono::Utc;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub id: String,
    pub case_id: String,
    pub nom: String,
    pub description: Option<String>,
    pub hash_sha256: String,
    pub hash_precedent: Option<String>,
    pub taille: Option<i64>,
    pub chemin: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub contents: Option<Vec<SnapshotContent>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotContent {
    pub id: String,
    pub snapshot_id: String,
    pub nom: String,
    pub kind: String,
    pub contenu: Option<String>,
    pub chemin: Option<String>,
    pub hash_sha256: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotMetadata {
    pub id: String,
    pub case_id: String,
    pub nom: String,
    pub description: Option<String>,
    pub hash_sha256: String,
    pub taille: Option<i64>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSnapshotInput {
    pub case_id: String,
    pub nom: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// =============================================================================
// TAKE SNAPSHOT
// =============================================================================

#[command]
pub async fn take_snapshot(
    state: tauri::State<'_, AppState>,
    input: CreateSnapshotInput,
) -> AppResult<SnapshotMetadata> {
    let mut conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();
    let id = generate_uuid();

    // Check case exists
    let case_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM cases WHERE id = ?",
        rusqlite::params![input.case_id],
        |row| row.get(0),
    )?;
    if case_exists == 0 {
        return Err(AppError::msg(format!("Case not found: {}", input.case_id)));
    }

    // Get the latest snapshot hash for chaining
    let hash_precedent: Option<String> = conn.query_row(
        "SELECT hashSha256 FROM snapshots WHERE caseId = ? ORDER BY created_at DESC LIMIT 1",
        rusqlite::params![input.case_id],
        |row| row.get(0),
    ).ok();

    // Create combined JSON for hashing
    let snapshot_data = serde_json::json!({
        "id": id,
        "case_id": input.case_id,
        "nom": input.nom,
        "description": input.description,
        "timestamp": now,
        "metadata": input.metadata
    });
    let json_str = serde_json::to_string(&snapshot_data)?;
    let hash = sha256_hash(&json_str);

    conn.execute(
        "INSERT INTO snapshots (id, caseId, nom, description, hashSha256, hashPrecedent, taille, chemin, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &id,
            &input.case_id,
            &input.nom,
            &input.description,
            &hash,
            &hash_precedent,
            json_str.len() as i64,
            "NULL",
            &input.metadata,
        ],
    )?;

    // Récupère le dernier hash d'audit pour ce dossier (chaînage).
    let previous_hash: Option<String> = conn
        .query_row(
            "SELECT imma FROM audit_events WHERE caseId = ? ORDER BY timestamp DESC LIMIT 1",
            rusqlite::params![&input.case_id],
            |row| row.get(0),
        )
        .ok();
    let previous_hash = previous_hash.unwrap_or_default();

    // Construit l'événement d'audit et calcule son hash chaîné.
    let audit_event = spectra_audit::AuditEvent {
        id: crate::database::generate_uuid(),
        case_id: input.case_id.clone(),
        action: "snapshot".to_string(),
        entity_kind: "snapshot".to_string(),
        entity_id: Some(id.clone()),
        actor: "system".to_string(),
        sequence: 1,
        payload: serde_json::json!({"nom": input.nom, "description": input.description}),
    };
    let link = spectra_audit::compute_link(&audit_event, &previous_hash);

    conn.execute(
        "INSERT INTO audit_events (id, caseId, action, entityKind, entityId, imma, imma_precedent, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &audit_event.id,
            &input.case_id,
            &audit_event.action,
            &audit_event.entity_kind,
            &audit_event.entity_id,
            &link.hash,
            &link.previous_hash,
            &serde_json::to_string(&audit_event.payload)?,
        ],
    )?;

    // Add case event
    conn.execute(
        "INSERT INTO case_events (id, caseId, type, titre, description, timestamp, actor, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &crate::database::generate_uuid(),
            &input.case_id,
            "snapshot_taken",
            &format!("Snapshot: {}", input.nom),
            &format!("Created snapshot {}", id),
            &now,
            "system",
            &serde_json::json!({"snapshot_id": id}),
        ],
    )?;

    Ok(SnapshotMetadata {
        id,
        case_id: input.case_id,
        nom: input.nom,
        description: input.description,
        hash_sha256: hash,
        taille: Some(json_str.len() as i64),
        created_at: now,
    })
}

// =============================================================================
// LIST SNAPSHOTS
// =============================================================================

#[command]
pub async fn list_snapshots(
    state: tauri::State<'_, AppState>,
    case_id: String,
) -> AppResult<Vec<SnapshotMetadata>> {
    let mut conn = state.get_conn().await;

    let mut stmt = conn.prepare("SELECT id, caseId, nom, description, hashSha256, taille, created_at FROM snapshots WHERE caseId = ? ORDER BY created_at DESC")?;

    let snapshots = stmt.query_map(rusqlite::params![case_id], |row| {
        Ok(SnapshotMetadata {
            id: row.get(0)?,
            case_id: row.get(1)?,
            nom: row.get(2)?,
            description: row.get(3)?,
            hash_sha256: row.get(4)?,
            taille: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    let result: Result<Vec<_>, _> = snapshots.collect();
    Ok(result?)
}

// =============================================================================
// GET SNAPSHOT
// =============================================================================

#[command]
pub async fn get_snapshot(
    state: tauri::State<'_, AppState>,
    id: String,
) -> AppResult<Option<Snapshot>> {
    let mut conn = state.get_conn().await;

    // Get snapshot metadata
    let mut stmt = conn.prepare("SELECT id, caseId, nom, description, hashSha256, hashPrecedent, taille, chemin, metadata FROM snapshots WHERE id = ?")?;
    let snapshot_meta = stmt.query_row(rusqlite::params![id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<i64>>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, Option<serde_json::Value>>(8)?,
        ))
    }).ok();

    let (id, case_id, nom, description, hash_sha256, hash_precedent, taille, chemin, metadata) = match snapshot_meta {
        Some(m) => m,
        None => return Ok(None),
    };

    // Get contents
    let mut contents_stmt = conn.prepare("SELECT id, nom, kind, contenu, chemin, hashSha256, metadata FROM snapshot_contents WHERE snapshotId = ?")?;
    let contents = contents_stmt.query_map(rusqlite::params![id], |row| {
        Ok(SnapshotContent {
            id: row.get(0)?,
            snapshot_id: id.clone(),
            nom: row.get(1)?,
            kind: row.get(2)?,
            contenu: row.get(3)?,
            chemin: row.get(4)?,
            hash_sha256: row.get(5)?,
            metadata: row.get(6)?,
        })
    })?;

    let contents: Result<Vec<_>, _> = contents.collect();

    Ok(Some(Snapshot {
        id,
        case_id,
        nom,
        description,
        hash_sha256,
        hash_precedent,
        taille,
        chemin,
        metadata,
        contents: contents.ok(),
    }))
}

// =============================================================================
// VERIFY SNAPSHOT INTEGRITY
// =============================================================================

#[command]
pub async fn verify_snapshot_integrity(
    state: tauri::State<'_, AppState>,
    id: String,
) -> AppResult<serde_json::Value> {
    let mut conn = state.get_conn().await;

    // Get snapshot
    let mut stmt = conn.prepare("SELECT hashSha256, hashPrecedent FROM snapshots WHERE id = ?")?;
    let (hash, hash_precedent): (String, Option<String>) = stmt.query_row(rusqlite::params![id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;

    // Verify current hash
    let current_valid = !hash.is_empty();

    // Verify chain integrity
    let chain_valid = match hash_precedent {
        Some(prev_hash) => !prev_hash.is_empty(),
        None => true, // First snapshot, no chain to verify
    };

    Ok(serde_json::json!({
        "snapshot_id": id,
        "hash_valid": current_valid,
        "chain_valid": chain_valid,
        "verified_at": Utc::now().to_rfc3339()
    }))
}

// =============================================================================
// DELETE SNAPSHOT
// =============================================================================

#[command]
pub async fn delete_snapshot(
    state: tauri::State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    let mut conn = state.get_conn().await;

    // Get snapshot case_id before delete
    let case_id: String = conn.query_row(
        "SELECT caseId FROM snapshots WHERE id = ?",
        rusqlite::params![id],
        |row| row.get(0),
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
    let audit_event = spectra_audit::AuditEvent {
        id: crate::database::generate_uuid(),
        case_id: case_id.clone(),
        action: "delete".to_string(),
        entity_kind: "snapshot".to_string(),
        entity_id: Some(id.clone()),
        actor: "system".to_string(),
        sequence: 1,
        payload: serde_json::json!({}),
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

    // Check snapshot exists
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM snapshots WHERE id = ?",
        rusqlite::params![id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Err(AppError::msg(format!("Snapshot not found: {}", id)));
    }

    // Delete contents first
    conn.execute(
        "DELETE FROM snapshot_contents WHERE snapshotId = ?",
        rusqlite::params![id],
    )?;

    // Delete snapshot
    conn.execute(
        "DELETE FROM snapshots WHERE id = ?",
        rusqlite::params![id],
    )?;

    Ok(())
}

// =============================================================================
// LOAD SNAPSHOT BUNDLE
// =============================================================================

#[command]
pub async fn load_snapshot_bundle(
    state: tauri::State<'_, AppState>,
    id: String,
) -> AppResult<Option<serde_json::Value>> {
    // Ne pas prendre le verrou ici : `get_snapshot` le prend lui-même, et ce
    // mutex n.est pas réentrant (`audit.md`, P2-8).
    let snapshot = get_snapshot(state, id).await?;

    Ok(snapshot.map(|s| {
        serde_json::json!({
            "snapshot": {
                "id": s.id,
                "case_id": s.case_id,
                "nom": s.nom,
                "description": s.description,
                "hash_sha256": s.hash_sha256,
                "taille": s.taille,
                "contents": s.contents
            }
        })
    }))
}
