//! Evidence commands module
//!
//! Manages evidence items with integrity verification.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::command;
use chrono::Utc;
use crate::database::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct AddEvidenceOptions {
    pub actor: Option<String>,
    pub source: Option<String>,
    pub reliability: Option<i32>,
    pub qualification: Option<String>,
    pub source_url: Option<String>,
}

/// Ingère une preuve : lit le fichier, calcule son empreinte, le copie.
///
/// # Pourquoi le backend lit le fichier lui-même
///
/// La version d'origine se contentait d'enregistrer `hash_sha256` **tel que
/// fourni par le client**, sans jamais ouvrir le fichier (`audit.md`, P1-2).
/// Une empreinte que le système n'a pas observée n'atteste rien : elle
/// certifie ce que l'appelant a bien voulu déclarer.
///
/// Le fichier est en outre **copié** dans un magasin sous le répertoire de
/// données applicatives. Conserver un chemin vers un fichier externe laisse la
/// preuve mutable après enregistrement — l'analyste peut la modifier ou la
/// supprimer sans que rien ne le signale.
#[command]
pub async fn add_evidence(
    state: tauri::State<'_, AppState>,
    case_id: String,
    data: AddEvidenceInput,
    options: Option<AddEvidenceOptions>,
) -> AppResult<Evidence> {
    let source_path = data
        .chemin
        .as_deref()
        .ok_or_else(|| AppError::msg("chemin du fichier requis"))?;

    let source_path = std::path::Path::new(source_path);
    if !source_path.is_file() {
        return Err(AppError::msg(format!(
            "fichier introuvable : {}",
            source_path.display()
        )));
    }

    // Lecture et empreintes calculées ici, avant toute écriture en base.
    let bytes = std::fs::read(source_path)
        .map_err(|e| AppError::msg(format!("lecture impossible : {e}")))?;
    let sha256 = crate::database::sha256_hash_bytes(&bytes);
    let taille = bytes.len() as i64;

    let id = crate::database::generate_uuid();

    // Sous-répertoire par préfixe d'identifiant : évite des dizaines de
    // milliers d'entrées dans un même dossier.
    let shard = &id[..2];
    let dest_dir = state.storage_root().join("evidence").join(shard);
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| AppError::msg(format!("magasin inaccessible : {e}")))?;

    let extension = source_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    let dest_path = dest_dir.join(format!("{id}{extension}"));

    std::fs::write(&dest_path, &bytes)
        .map_err(|e| AppError::msg(format!("copie impossible : {e}")))?;

    let nom = data.nom.clone().or_else(|| {
        source_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_string)
    });

    let conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();

    let case_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM cases WHERE id = ?",
        rusqlite::params![case_id],
        |row| row.get(0),
    )?;
    if case_exists == 0 {
        // Le fichier vient d'être copié : on le retire, sinon le magasin
        // accumule des orphelins à chaque erreur.
        let _ = std::fs::remove_file(&dest_path);
        return Err(AppError::msg(format!("dossier introuvable : {case_id}")));
    }

    // `statut = 'verifie'` : l'empreinte vient d'être calculée sur le contenu
    // effectivement stocké, ce n'est pas une déclaration de l'appelant.
    conn.execute(
        "INSERT INTO evidence (id, caseId, type, nom, description, chemin, hash_sha256, hash_md5, taille, dateAjout, statut, source, sourceUrl, metadata)          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &id,
            &case_id,
            &data.r#type,
            &nom,
            &data.description,
            &dest_path.to_string_lossy().to_string(),
            &sha256,
            Option::<String>::None,
            &taille,
            &now,
            &"verifie",
            &data.source,
            &data.source_url,
            &data.metadata,
        ],
    )?;

    let actor = options
        .as_ref()
        .and_then(|o| o.actor.clone())
        .unwrap_or_else(|| "system".to_string());

    crate::database::append_audit_event(
        &conn,
        &case_id,
        "create",
        "evidence",
        Some(&id),
        &actor,
        serde_json::json!({
            "type": data.r#type,
            "nom": nom,
            "sha256": sha256,
            "taille": taille,
            "source_originale": source_path.to_string_lossy(),
        }),
    )?;

    get_evidence_single(&conn, &id)
}

fn get_evidence_single(conn: &rusqlite::Connection, id: &str) -> AppResult<Evidence> {
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
#[serde(rename_all = "camelCase")]
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
