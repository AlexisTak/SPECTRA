//! Subjects commands module
//!
//! Manages subjects (suspects, victims, witnesses).

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::command;
use crate::database::{generate_uuid, AppState};
use chrono::Utc;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    pub id: String,
    pub case_id: String,
    pub nom: Option<String>,
    pub prenom: Option<String>,
    pub statut: String,
    pub date_naissance: Option<String>,
    pub lieu_naissance: Option<String>,
    pub nationalite: Option<String>,
    pub telephone: Option<String>,
    pub email: Option<String>,
    pub adresse: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubjectInput {
    pub case_id: String,
    pub nom: Option<String>,
    pub prenom: Option<String>,
    pub statut: Option<String>,
    pub date_naissance: Option<String>,
    pub lieu_naissance: Option<String>,
    pub nationalite: Option<String>,
    pub telephone: Option<String>,
    pub email: Option<String>,
    pub adresse: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubjectInput {
    pub nom: Option<String>,
    pub prenom: Option<String>,
    pub statut: Option<String>,
    pub date_naissance: Option<String>,
    pub lieu_naissance: Option<String>,
    pub nationalite: Option<String>,
    pub telephone: Option<String>,
    pub email: Option<String>,
    pub adresse: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// =============================================================================
// GET SUBJECTS
// =============================================================================

#[command]
pub async fn get_subjects(
    state: tauri::State<'_, AppState>,
    case_id: String,
) -> AppResult<Vec<Subject>> {
    let conn = state.get_conn().await;

    let mut stmt = conn.prepare("SELECT id, caseId, nom, prenom, statut, dateNaissance, lieuNaissance, nationalite, telephone, email, adresse, description, metadata FROM subjects WHERE caseId = ? ORDER BY nom")?;

    let subjects = stmt.query_map(rusqlite::params![case_id], |row| {
        Ok(Subject {
            id: row.get(0)?,
            case_id: row.get(1)?,
            nom: row.get(2)?,
            prenom: row.get(3)?,
            statut: row.get(4)?,
            date_naissance: row.get(5)?,
            lieu_naissance: row.get(6)?,
            nationalite: row.get(7)?,
            telephone: row.get(8)?,
            email: row.get(9)?,
            adresse: row.get(10)?,
            description: row.get(11)?,
            metadata: row.get(12)?,
        })
    })?;

    let result: Result<Vec<_>, _> = subjects.collect();
    Ok(result?)
}

// =============================================================================
// CREATE SUBJECT
// =============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubjectOptions {
    pub actor: Option<String>,
}

#[command]
pub async fn create_subject(
    state: tauri::State<'_, AppState>,
    data: CreateSubjectInput,
    _options: Option<CreateSubjectOptions>,
) -> AppResult<Subject> {
    let mut conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();

    // Check case exists
    let case_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM cases WHERE id = ?",
        rusqlite::params![data.case_id],
        |row| row.get(0),
    )?;
    if case_exists == 0 {
        return Err(AppError::msg(format!("Case not found: {}", data.case_id)));
    }

    let id = generate_uuid();
    // Lié une fois : `data.statut` était consommé deux fois (`audit.md`, P0-1j).
    let statut = data
        .statut
        .clone()
        .unwrap_or_else(|| "suspect".to_string());

    conn.execute(
        "INSERT INTO subjects (id, caseId, nom, prenom, statut, dateNaissance, lieuNaissance, nationalite, telephone, email, adresse, description, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &id,
            &data.case_id,
            &data.nom,
            &data.prenom,
            &statut,
            &data.date_naissance,
            &data.lieu_naissance,
            &data.nationalite,
            &data.telephone,
            &data.email,
            &data.adresse,
            &data.description,
            &data.metadata,
        ],
    )?;

    // Add to FTS index
    if let Some(nom) = &data.nom {
        conn.execute(
            "INSERT INTO fts_subjects (rowid, nom, prenom, description, email, telephone) VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![&id, nom, &data.prenom, &data.description, &data.email, &data.telephone],
        )?;
    }

    // Récupère le dernier hash d'audit pour ce dossier (chaînage).
    let previous_hash: Option<String> = conn
        .query_row(
            "SELECT imma FROM audit_events WHERE caseId = ? ORDER BY timestamp DESC LIMIT 1",
            rusqlite::params![&data.case_id],
            |row| row.get(0),
        )
        .ok();
    let previous_hash = previous_hash.unwrap_or_default();

    // Construit l'événement d'audit et calcule son hash chaîné.
    let audit_event = spectra_audit::AuditEvent {
        id: crate::database::generate_uuid(),
        case_id: data.case_id.clone(),
        action: "create".to_string(),
        entity_kind: "subject".to_string(),
        entity_id: Some(id.clone()),
        actor: _options.as_ref().and_then(|o| o.actor.clone()).unwrap_or_else(|| "system".to_string()),
        sequence: 1,
        payload: serde_json::json!({"statut": statut, "nom": data.nom}),
    };
    let link = spectra_audit::compute_link(&audit_event, &previous_hash);

    conn.execute(
        "INSERT INTO case_events (id, caseId, type, titre, description, timestamp, actor, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &crate::database::generate_uuid(),
            &data.case_id,
            "subject_added",
            &format!("Subject added: {}", data.nom.unwrap_or_else(|| "Unknown".to_string())),
            &format!("Added subject with status {}", statut),
            &now,
            &_options.as_ref().and_then(|o| o.actor.clone()),
            &serde_json::to_string(&_options)?,
        ],
    )?;

    conn.execute(
        "INSERT INTO audit_events (id, caseId, action, entityKind, entityId, imma, imma_precedent, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &audit_event.id,
            &data.case_id,
            &audit_event.action,
            &audit_event.entity_kind,
            &audit_event.entity_id,
            &link.hash,
            &link.previous_hash,
            &serde_json::to_string(&audit_event.payload)?,
        ],
    )?;

    // Re-fetch to get full record
    get_subject_single(&mut *conn, &id)
}

fn get_subject_single(conn: &mut rusqlite::Connection, id: &str) -> AppResult<Subject> {
    let mut stmt = conn.prepare("SELECT id, caseId, nom, prenom, statut, dateNaissance, lieuNaissance, nationalite, telephone, email, adresse, description, metadata FROM subjects WHERE id = ?")?;

    let subject = stmt.query_row(rusqlite::params![id], |row| {
        Ok(Subject {
            id: row.get(0)?,
            case_id: row.get(1)?,
            nom: row.get(2)?,
            prenom: row.get(3)?,
            statut: row.get(4)?,
            date_naissance: row.get(5)?,
            lieu_naissance: row.get(6)?,
            nationalite: row.get(7)?,
            telephone: row.get(8)?,
            email: row.get(9)?,
            adresse: row.get(10)?,
            description: row.get(11)?,
            metadata: row.get(12)?,
        })
    })?;

    Ok(subject)
}

// =============================================================================
// UPDATE SUBJECT
// =============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubjectOptions {
    pub actor: Option<String>,
}

#[command]
pub async fn update_subject(
    state: tauri::State<'_, AppState>,
    id: String,
    data: UpdateSubjectInput,
    options: Option<UpdateSubjectOptions>,
) -> AppResult<Subject> {
    let mut conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();

    // Check subject exists
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM subjects WHERE id = ?",
        rusqlite::params![id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Err(AppError::msg(format!("Subject not found: {}", id)));
    }

    // Get case_id before update
    let case_id: String = conn.query_row(
        "SELECT caseId FROM subjects WHERE id = ?",
        rusqlite::params![id],
        |row| row.get(0),
    )?;

    // Build dynamic UPDATE
    let mut updates = Vec::new();
    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();

    if let Some(nom) = &data.nom {
        updates.push("nom = ?");
        params.push(nom);
    }
    if let Some(prenom) = &data.prenom {
        updates.push("prenom = ?");
        params.push(prenom);
    }
    if let Some(statut) = &data.statut {
        updates.push("statut = ?");
        params.push(statut);
    }
    if let Some(date_naissance) = &data.date_naissance {
        updates.push("dateNaissance = ?");
        params.push(date_naissance);
    }
    if let Some(lieu_naissance) = &data.lieu_naissance {
        updates.push("lieuNaissance = ?");
        params.push(lieu_naissance);
    }
    if let Some(nationalite) = &data.nationalite {
        updates.push("nationalite = ?");
        params.push(nationalite);
    }
    if let Some(telephone) = &data.telephone {
        updates.push("telephone = ?");
        params.push(telephone);
    }
    if let Some(email) = &data.email {
        updates.push("email = ?");
        params.push(email);
    }
    if let Some(adresse) = &data.adresse {
        updates.push("adresse = ?");
        params.push(adresse);
    }
    if let Some(description) = &data.description {
        updates.push("description = ?");
        params.push(description);
    }
    if let Some(metadata) = &data.metadata {
        updates.push("metadata = ?");
        params.push(metadata);
    }

    updates.push("updated_at = ?");
    params.push(&now);

    params.push(&id);

    let query = format!("UPDATE subjects SET {} WHERE id = ?", updates.join(", "));
    conn.execute(&query, &params[..])?;

    // Update FTS index
    if let Some(nom) = &data.nom {
        conn.execute(
            "UPDATE fts_subjects SET nom = ? WHERE rowid = ?",
            rusqlite::params![nom, &id],
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
        action: "update".to_string(),
        entity_kind: "subject".to_string(),
        entity_id: Some(id.clone()),
        actor: options.as_ref().and_then(|o| o.actor.clone()).unwrap_or_else(|| "system".to_string()),
        sequence: 1,
        payload: serde_json::json!({"nom": data.nom, "statut": data.statut}),
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
    get_subject_single(&mut *conn, &id)
}

// =============================================================================
// DELETE SUBJECT
// =============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSubjectOptions {
    pub actor: Option<String>,
}

#[command]
pub async fn delete_subject(
    state: tauri::State<'_, AppState>,
    id: String,
    case_id: String,
    options: Option<DeleteSubjectOptions>,
) -> AppResult<()> {
    let mut conn = state.get_conn().await;

    // Check subject exists
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM subjects WHERE id = ?",
        rusqlite::params![id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Err(AppError::msg(format!("Subject not found: {}", id)));
    }

    // Get subject details before delete
    let subject = get_subject_single(&mut *conn, &id)?;

    // Remove from FTS index
    conn.execute(
        "DELETE FROM fts_subjects WHERE rowid = ?",
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
        entity_kind: "subject".to_string(),
        entity_id: Some(id.clone()),
        actor: options.as_ref().and_then(|o| o.actor.clone()).unwrap_or_else(|| "system".to_string()),
        sequence: 1,
        payload: serde_json::json!({"nom": subject.nom, "statut": subject.statut}),
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

    // Delete subject
    conn.execute(
        "DELETE FROM subjects WHERE id = ?",
        rusqlite::params![&id],
    )?;

    // Add case event
    conn.execute(
        "INSERT INTO case_events (id, caseId, type, titre, description, timestamp, actor, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &crate::database::generate_uuid(),
            &case_id,
            "subject_deleted",
            &format!("Subject deleted: {}", subject.nom.unwrap_or_else(|| "Unknown".to_string())),
            &format!("Removed {} from case", subject.statut),
            &now,
            &options.as_ref().and_then(|o| o.actor.clone()),
            &serde_json::to_string(&options)?,
        ],
    )?;

    Ok(())
}
