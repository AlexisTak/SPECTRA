//! Instantanés d'un dossier.
//!
//! Un instantané capture **l'état complet** du dossier : métadonnées, sujets,
//! preuves, événements. Le tout est sérialisé, haché, stocké dans un fichier.
//!
//! # Garantie probatoire
//!
//! L'empreinte SHA-256 porte sur le contenu capturé, pas sur une déclaration de
//! l'appelant. Vérifier un instantané, c'est recalculer l'empreinte sur le
//! fichier stocké et comparer.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::command;
use crate::database::{generate_uuid, AppState, sha256_hash_bytes};
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

/// Métadonnées renvoyées après création.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotMetadata {
    pub id: String,
    pub case_id: String,
    pub nom: String,
    pub description: Option<String>,
    /// Empreinte du **contenu capturé**, pas d'une déclaration.
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

/// Capture l'état complet d'un dossier dans un fichier chiffrable.
///
/// L'instantané contient : métadonnées du dossier, sujets, preuves, événements.
/// Le fichier est stocké dans le magasin de preuves, et son empreinte SHA-256
/// est calculée sur le contenu sérialisé — pas sur une déclaration de l'appelant.
#[command]
pub async fn take_snapshot(
    state: tauri::State<'_, AppState>,
    input: CreateSnapshotInput,
) -> AppResult<SnapshotMetadata> {
    let conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();
    let id = generate_uuid();

    // Vérifie que le dossier existe et récupère ses métadonnées
    let case_meta: Option<(String, String, Option<String>)> = conn
        .query_row(
            "SELECT reference, titre, description FROM cases WHERE id = ?",
            rusqlite::params![&input.case_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok();

    let Some((reference, titre, description)) = case_meta else {
        return Err(AppError::msg(format!("Dossier introuvable : {}", input.case_id)));
    };

    // Capture des sujets
    let subjects: Vec<serde_json::Value> = conn
        .prepare(
            "SELECT id, nom, prenom, statut, dateNaissance, lieuNaissance, nationalite, telephone, email, adresse, description \
             FROM subjects WHERE caseId = ? ORDER BY id",
        )?
        .query_map(rusqlite::params![&input.case_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "nom": row.get::<_, Option<String>>(1)?,
                "prenom": row.get::<_, Option<String>>(2)?,
                "statut": row.get::<_, String>(3)?,
                "date_naissance": row.get::<_, Option<String>>(4)?,
                "lieu_naissance": row.get::<_, Option<String>>(5)?,
                "nationalite": row.get::<_, Option<String>>(6)?,
                "telephone": row.get::<_, Option<String>>(7)?,
                "email": row.get::<_, Option<String>>(8)?,
                "adresse": row.get::<_, Option<String>>(9)?,
                "description": row.get::<_, Option<String>>(10)?,
            }))
        })?
        .filter_map(Result::ok)
        .collect();

    // Capture des preuves (seulement métadonnées, pas le contenu des fichiers)
    let evidence: Vec<serde_json::Value> = conn
        .prepare(
            "SELECT id, type, nom, description, hash_sha256, taille, dateAjout, statut \
             FROM evidence WHERE caseId = ? ORDER BY id",
        )?
        .query_map(rusqlite::params![&input.case_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "type": row.get::<_, String>(1)?,
                "nom": row.get::<_, Option<String>>(2)?,
                "description": row.get::<_, Option<String>>(3)?,
                "hash_sha256": row.get::<_, Option<String>>(4)?,
                "taille": row.get::<_, Option<i64>>(5)?,
                "date_ajout": row.get::<_, String>(6)?,
                "statut": row.get::<_, String>(7)?,
            }))
        })?
        .filter_map(Result::ok)
        .collect();

    // Capture des événements du dossier
    let events: Vec<serde_json::Value> = conn
        .prepare(
            "SELECT id, type, titre, description, timestamp, actor \
             FROM case_events WHERE caseId = ? ORDER BY timestamp",
        )?
        .query_map(rusqlite::params![&input.case_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "type": row.get::<_, String>(1)?,
                "titre": row.get::<_, Option<String>>(2)?,
                "description": row.get::<_, Option<String>>(3)?,
                "timestamp": row.get::<_, String>(4)?,
                "actor": row.get::<_, Option<String>>(5)?,
            }))
        })?
        .filter_map(Result::ok)
        .collect();

    // Sérialisation complète
    let snapshot_content = serde_json::json!({
        "id": &id,
        "case_id": &input.case_id,
        "reference": &reference,
        "titre": &titre,
        "description": &description,
        "nom": &input.nom,
        "description_snapshot": &input.description,
        "timestamp": &now,
        "subjects": subjects,
        "evidence": evidence,
        "events": events,
    });

    let json_bytes = serde_json::to_vec(&snapshot_content)?;
    let hash_bytes = sha256_hash_bytes(&json_bytes);

    // Stockage dans le magasin
    let shard = &id[..2];
    let dest_dir = state.storage_root().join("snapshots").join(shard);
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| AppError::msg(format!("Magasin inaccessible : {e}")))?;

    let dest_path = dest_dir.join(format!("{id}.json"));
    std::fs::write(&dest_path, &json_bytes)
        .map_err(|e| AppError::msg(format!("Écriture impossible : {e}")))?;

    // Chaînage avec le précédent instantané
    let hash_precedent: Option<String> = conn
        .query_row(
            "SELECT hashSha256 FROM snapshots WHERE caseId = ? ORDER BY created_at DESC LIMIT 1",
            rusqlite::params![&input.case_id],
            |row| row.get(0),
        )
        .ok();

    // Enregistrement en base
    conn.execute(
        "INSERT INTO snapshots (id, caseId, nom, description, hashSha256, hashPrecedent, taille, chemin, metadata) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &id,
            &input.case_id,
            &input.nom,
            &input.description,
            &hash_bytes,
            &hash_precedent,
            &(json_bytes.len() as i64),
            &dest_path.to_string_lossy().to_string(),
            &input.metadata,
        ],
    )?;

    // Audit chaîné
    crate::database::append_audit_event(
        &conn,
        &input.case_id,
        "snapshot",
        "snapshot",
        Some(&id),
        "system",
        serde_json::json!({
            "nom": input.nom,
            "sujets": subjects.len(),
            "preuves": evidence.len(),
            "evenements": events.len(),
            "sha256": hash_bytes,
        }),
    )?;

    Ok(SnapshotMetadata {
        id,
        case_id: input.case_id,
        nom: input.nom,
        description: input.description,
        hash_sha256: hash_bytes,
        taille: Some(json_bytes.len() as i64),
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
    let conn = state.get_conn().await;

    let snapshots = conn
        .prepare("SELECT id, caseId, nom, description, hashSha256, taille, created_at FROM snapshots WHERE caseId = ? ORDER BY created_at DESC")?
        .query_map(rusqlite::params![case_id], |row| {
            Ok(SnapshotMetadata {
                id: row.get(0)?,
                case_id: row.get(1)?,
                nom: row.get(2)?,
                description: row.get(3)?,
                hash_sha256: row.get(4)?,
                taille: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();

    Ok(snapshots)
}

// =============================================================================
// GET SNAPSHOT
// =============================================================================

#[command]
pub async fn get_snapshot(
    state: tauri::State<'_, AppState>,
    id: String,
) -> AppResult<Option<Snapshot>> {
    let conn = state.get_conn().await;

    let snapshot_meta: Option<(String, String, String, Option<String>, String, Option<String>, Option<i64>, Option<String>, Option<serde_json::Value>)> = conn
        .query_row(
            "SELECT id, caseId, nom, description, hashSha256, hashPrecedent, taille, chemin, metadata FROM snapshots WHERE id = ?",
            rusqlite::params![&id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                ))
            },
        )
        .ok();

    let Some((id, case_id, nom, description, hash_sha256, hash_precedent, taille, chemin, metadata)) = snapshot_meta else {
        return Ok(None);
    };

    let contents: Vec<SnapshotContent> = conn
        .prepare("SELECT id, nom, kind, contenu, chemin, hashSha256, metadata FROM snapshot_contents WHERE snapshotId = ?")?
        .query_map(rusqlite::params![&id], |row| {
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
        })?
        .filter_map(Result::ok)
        .collect();

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
        contents: Some(contents),
    }))
}

// =============================================================================
// VERIFY SNAPSHOT INTEGRITY
// =============================================================================

/// Vérifie l'intégrité d'un instantané en recalculant l'empreinte sur le fichier.
#[command]
pub async fn verify_snapshot_integrity(
    state: tauri::State<'_, AppState>,
    id: String,
) -> AppResult<serde_json::Value> {
    let conn = state.get_conn().await;

    let (expected_hash, chemin): (String, Option<String>) = conn
        .query_row(
            "SELECT hashSha256, chemin FROM snapshots WHERE id = ?",
            rusqlite::params![&id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

    let Some(chemin) = chemin else {
        return Ok(serde_json::json!({
            "snapshot_id": id,
            "status": "missing",
            "message": "aucun fichier associé",
            "verified_at": Utc::now().to_rfc3339()
        }));
    };

    let Ok(bytes) = std::fs::read(&chemin) else {
        return Ok(serde_json::json!({
            "snapshot_id": id,
            "status": "missing",
            "message": "fichier illisible",
            "verified_at": Utc::now().to_rfc3339()
        }));
    };

    let actual_hash = sha256_hash_bytes(&bytes);

    if actual_hash == expected_hash {
        Ok(serde_json::json!({
            "snapshot_id": id,
            "status": "intact",
            "message": "empreinte recalculée conforme",
            "verified_at": Utc::now().to_rfc3339()
        }))
    } else {
        Ok(serde_json::json!({
            "snapshot_id": id,
            "status": "altered",
            "expected_hash": expected_hash,
            "actual_hash": actual_hash,
            "message": "empreinte différente : contenu modifié",
            "verified_at": Utc::now().to_rfc3339()
        }))
    }
}

// =============================================================================
// DELETE SNAPSHOT
// =============================================================================

#[command]
pub async fn delete_snapshot(
    state: tauri::State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    let conn = state.get_conn().await;

    let case_id: String = conn
        .query_row("SELECT caseId FROM snapshots WHERE id = ?", rusqlite::params![&id], |row| row.get(0))?;

    // Supprime le fichier du magasin
    let chemin: Option<String> = conn
        .query_row("SELECT chemin FROM snapshots WHERE id = ?", rusqlite::params![&id], |row| row.get(0))
        .ok();

    if let Some(chemin) = &chemin {
        let _ = std::fs::remove_file(chemin);
    }

    // Audit
    crate::database::append_audit_event(
        &conn,
        &case_id,
        "delete",
        "snapshot",
        Some(&id),
        "system",
        serde_json::json!({"chemin": chemin}),
    )?;

    conn.execute("DELETE FROM snapshot_contents WHERE snapshotId = ?", rusqlite::params![&id])?;
    conn.execute("DELETE FROM snapshots WHERE id = ?", rusqlite::params![&id])?;

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
