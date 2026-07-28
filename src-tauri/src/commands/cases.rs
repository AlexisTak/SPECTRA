//! Cases commands module
//!
//! Provides CRUD operations for case management.

use anyhow::Result;
use chrono::Datelike;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::command;
use crate::database::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateCaseInput {
    pub titre: String,
    pub description: Option<String>,
    pub statut: Option<String>,
    pub priorite: Option<String>,
    pub categorie: Option<String>,
    pub tags: Option<Vec<String>>,
    pub meta: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateCaseInput {
    pub titre: Option<String>,
    pub description: Option<String>,
    pub statut: Option<String>,
    pub priorite: Option<String>,
    pub categorie: Option<String>,
    pub tags: Option<Vec<String>>,
    pub meta: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Case {
    pub id: String,
    pub reference: String,
    pub titre: String,
    pub description: Option<String>,
    pub statut: String,
    pub priorite: String,
    pub categorie: Option<String>,
    pub date_creation: String,
    pub date_mise_jour: String,
    pub tags: Option<Vec<String>>,
    pub meta: Option<serde_json::Value>,
}

// =============================================================================
// GET CASES
// =============================================================================

#[command]
pub async fn get_cases(
    state: tauri::State<'_, AppState>,
    statut: Option<String>,
    search: Option<String>,
) -> Result<Vec<Case>> {
    let conn = state.get_conn().await;

    let query = if search.is_some() {
        "SELECT id, reference, titre, description, statut, priorite, categorie, dateCreation, dateMiseJour, tags, meta FROM cases WHERE statut = ? AND (reference LIKE '%' || ? || '%' OR titre LIKE '%' || ? || '%' OR description LIKE '%' || ? || '%')"
    } else if statut.is_some() {
        "SELECT id, reference, titre, description, statut, priorite, categorie, dateCreation, dateMiseJour, tags, meta FROM cases WHERE statut = ?"
    } else {
        "SELECT id, reference, titre, description, statut, priorite, categorie, dateCreation, dateMiseJour, tags, meta FROM cases"
    };

    let statut_val = statut.as_ref().map(|s| s.as_str()).unwrap_or("ouvert");
    let search_val = search.as_ref().map(|s| s.as_str()).unwrap_or("");

    let params = if search.is_some() {
        [&statut_val, &search_val, &search_val, &search_val] as _
    } else if statut.is_some() {
        [&statut_val] as _
    } else {
        &[] as _
    };

    let mut stmt = conn.prepare(query)?;
    let cases = stmt.query_map(params, |row| {
        Ok(Case {
            id: row.get(0)?,
            reference: row.get(1)?,
            titre: row.get(2)?,
            description: row.get(3)?,
            statut: row.get(4)?,
            priorite: row.get(5)?,
            categorie: row.get(6)?,
            date_creation: row.get(7)?,
            date_mise_jour: row.get(8)?,
            tags: row.get::<_, Option<String>>(9)?.map(|s| serde_json::from_str(&s).unwrap_or_default()),
            meta: row.get::<_, Option<String>>(10)?.map(|s| serde_json::from_str(&s).unwrap_or_default()),
        })
    })?;

    let result: Result<Vec<_>, _> = cases.collect();
    Ok(result?)
}

// =============================================================================
// GET CASE BY ID
// =============================================================================

#[command]
pub async fn get_case(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Option<Case>> {
    let conn = state.get_conn().await;

    let mut stmt = conn.prepare("SELECT id, reference, titre, description, statut, priorite, categorie, dateCreation, dateMiseJour, tags, meta FROM cases WHERE id = ?")?;
    let case = stmt.query_row(rusqlite::params![id], |row| {
        Ok(Case {
            id: row.get(0)?,
            reference: row.get(1)?,
            titre: row.get(2)?,
            description: row.get(3)?,
            statut: row.get(4)?,
            priorite: row.get(5)?,
            categorie: row.get(6)?,
            date_creation: row.get(7)?,
            date_mise_jour: row.get(8)?,
            tags: row.get::<_, Option<String>>(9)?.map(|s| serde_json::from_str(&s).unwrap_or_default()),
            meta: row.get::<_, Option<String>>(10)?.map(|s| serde_json::from_str(&s).unwrap_or_default()),
        })
    }).ok();

    Ok(case)
}

// =============================================================================
// GET CASE STATS
// =============================================================================

#[command]
pub async fn get_case_stats(state: tauri::State<'_, AppState>) -> Result<serde_json::Value> {
    let conn = state.get_conn().await;

    let mut stats = serde_json::Map::new();

    let statut_counts: Vec<(String, i64)> = (*conn).query_map(
        "SELECT statut, COUNT(*) FROM cases GROUP BY statut",
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?.collect::<Result<_, _>>()?;

    stats.insert("byStatus".to_string(), serde_json::to_value(statut_counts)?);

    let priorite_counts: Vec<(String, i64)> = (*conn).query_map(
        "SELECT priorite, COUNT(*) FROM cases GROUP BY priorite",
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?.collect::<Result<_, _>>()?;

    stats.insert("byPriority".to_string(), serde_json::to_value(priorite_counts)?);

    Ok(serde_json::Value::Object(stats))
}

// =============================================================================
// CREATE CASE
// =============================================================================

#[command]
pub async fn create_case(
    state: tauri::State<'_, AppState>,
    data: CreateCaseInput,
) -> Result<Case> {
    let conn = state.get_conn().await;
    let now = chrono::Utc::now().to_rfc3339();

    let id = crate::database::generate_uuid();
    let reference = crate::database::generate_case_reference_locked(&state.conn).await?;
    let year = chrono::Utc::now().year();

    conn.execute(
        "INSERT INTO cases (id, reference, titre, description, statut, priorite, categorie, dateCreation, dateMiseJour, tags, meta) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &id,
            &reference,
            &data.titre,
            &data.description,
            &data.statut.unwrap_or_else(|| "ouvert".to_string()),
            &data.priorite.unwrap_or_else(|| "normale".to_string()),
            &data.categorie,
            &now,
            &now,
            &data.tags.map(|t| serde_json::to_string(&t)).transpose()?,
            &data.meta,
        ],
    )?;

    // Add to FTS index
    conn.execute(
        "INSERT INTO fts_cases (rowid, reference, titre, description) VALUES (?, ?, ?, ?)",
        rusqlite::params![&id, &reference, &data.titre, &data.description],
    )?;

    // Log audit event
    conn.execute(
        "INSERT INTO audit_events (id, caseId, action, entityKind, entityId, imma, imma_precedent, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &format!("AE-{}-000001", year),
            &id,
            "create",
            "case",
            &id,
            &format!("hex(sha256('create:case:{}:{}'))", id, now),
            "NULL",
            &serde_json::to_string(&serde_json::json!({"reference": reference}))?,
        ],
    )?;

    get_case(state, id).map(|c| c.ok_or_else(|| anyhow::anyhow!("Failed to retrieve created case")))
}

// =============================================================================
// UPDATE CASE
// =============================================================================

#[command]
pub async fn update_case(
    state: tauri::State<'_, AppState>,
    id: String,
    data: UpdateCaseInput,
) -> Result<Case> {
    let conn = state.get_conn().await;

    let mut updates = Vec::new();
    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();

    if let Some(titre) = &data.titre {
        updates.push("titre = ?");
        params.push(titre);
    }
    if let Some(description) = &data.description {
        updates.push("description = ?");
        params.push(description);
    }
    if let Some(statut) = &data.statut {
        updates.push("statut = ?");
        params.push(statut);
    }
    if let Some(priorite) = &data.priorite {
        updates.push("priorite = ?");
        params.push(priorite);
    }
    if let Some(categorie) = &data.categorie {
        updates.push("categorie = ?");
        params.push(categorie);
    }
    if let Some(tags) = &data.tags {
        updates.push("tags = ?");
        params.push(&serde_json::to_string(tags)?);
    }
    if let Some(meta) = &data.meta {
        updates.push("meta = ?");
        params.push(meta);
    }

    updates.push("dateMiseJour = ?");
    params.push(&chrono::Utc::now().to_rfc3339());

    params.push(&id);

    let query = format!("UPDATE cases SET {} WHERE id = ?", updates.join(", "));
    conn.execute(&query, params)?;

    // Update FTS index
    if let Some(titre) = &data.titre {
        conn.execute(
            "UPDATE fts_cases SET titre = ? WHERE rowid = ?",
            rusqlite::params![titre, &id],
        )?;
    }

    get_case(state, id).map(|c| c.ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated case")))
}

// =============================================================================
// DELETE CASE
// =============================================================================

#[command]
pub async fn delete_case(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<()> {
    let conn = state.get_conn().await;

    conn.execute(
        "DELETE FROM fts_cases WHERE rowid = ?",
        rusqlite::params![&id],
    )?;

    conn.execute("DELETE FROM cases WHERE id = ?", rusqlite::params![&id])?;

    Ok(())
}
