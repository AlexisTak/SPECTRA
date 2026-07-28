//! Reports commands module
//!
//! Manages reports with timeline and content.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::command;
use crate::database::{generate_report_reference, generate_uuid, AppState};
use chrono::Utc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Report {
    pub id: String,
    pub reference: String,
    pub case_id: String,
    pub titre: String,
    pub description: Option<String>,
    pub statut: String,
    pub date_creation: String,
    pub date_echeance: Option<String>,
    pub date_cloture: Option<String>,
    pub auteur: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateReportInput {
    pub case_id: String,
    pub titre: String,
    pub description: Option<String>,
    pub statut: Option<String>,
    pub date_echeance: Option<String>,
    pub auteur: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateReportInput {
    pub titre: Option<String>,
    pub description: Option<String>,
    pub statut: Option<String>,
    pub date_echeance: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReportContent {
    pub id: String,
    pub report_id: String,
    pub section: String,
    pub contenu: Option<String>,
    pub ordre: i32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReportTimelineEvent {
    pub id: String,
    pub report_id: String,
    pub date_event: String,
    pub description: String,
    pub r#type: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReportStats {
    pub total: i64,
    pub by_status: serde_json::Map<String, i64>,
    pub by_case: serde_json::Map<String, i64>,
}

// =============================================================================
// GET REPORTS
// =============================================================================

#[command]
pub async fn get_reports(
    state: tauri::State<'_, AppState>,
    case_id: Option<String>,
) -> Result<Vec<Report>> {
    let mut conn = state.get_conn().await;

    let query = if let Some(ref cid) = case_id {
        "SELECT id, reference, caseId, titre, description, statut, dateCreation, dateEcheance, dateCloture, auteur, metadata FROM reports WHERE caseId = ? ORDER BY dateCreation DESC"
    } else {
        "SELECT id, reference, caseId, titre, description, statut, dateCreation, dateEcheance, dateCloture, auteur, metadata FROM reports ORDER BY dateCreation DESC"
    };

    let params: Vec<&dyn rusqlite::ToSql> = if case_id.is_some() {
        vec![case_id.as_ref().unwrap()]
    } else {
        vec![]
    };

    let mut stmt = conn.prepare(query)?;
    let reports = stmt.query_map(params, |row| {
        Ok(Report {
            id: row.get(0)?,
            reference: row.get(1)?,
            case_id: row.get(2)?,
            titre: row.get(3)?,
            description: row.get(4)?,
            statut: row.get(5)?,
            date_creation: row.get(6)?,
            date_echeance: row.get(7)?,
            date_cloture: row.get(8)?,
            auteur: row.get(9)?,
            metadata: row.get(10)?,
        })
    })?;

    let result: Result<Vec<_>, _> = reports.collect();
    Ok(result?)
}

// =============================================================================
// GET REPORT
// =============================================================================

#[command]
pub async fn get_report(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Option<Report>> {
    let mut conn = state.get_conn().await;

    let mut stmt = conn.prepare("SELECT id, reference, caseId, titre, description, statut, dateCreation, dateEcheance, dateCloture, auteur, metadata FROM reports WHERE id = ?")?;

    let report = stmt.query_row(rusqlite::params![id], |row| {
        Ok(Report {
            id: row.get(0)?,
            reference: row.get(1)?,
            case_id: row.get(2)?,
            titre: row.get(3)?,
            description: row.get(4)?,
            statut: row.get(5)?,
            date_creation: row.get(6)?,
            date_echeance: row.get(7)?,
            date_cloture: row.get(8)?,
            auteur: row.get(9)?,
            metadata: row.get(10)?,
        })
    }).ok();

    Ok(report)
}

// =============================================================================
// CREATE REPORT
// =============================================================================

#[command]
pub async fn create_report(
    state: tauri::State<'_, AppState>,
    data: CreateReportInput,
) -> Result<Report> {
    let mut conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();
    let id = generate_uuid();
    let reference = generate_report_reference(&mut *conn)?;

    // Check case exists
    let case_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM cases WHERE id = ?",
        rusqlite::params![data.case_id],
        |row| row.get(0),
    )?;
    if case_exists == 0 {
        return Err(anyhow::anyhow!("Case not found: {}", data.case_id));
    }

    conn.execute(
        "INSERT INTO reports (id, reference, caseId, titre, description, statut, dateCreation, dateEcheance, dateCloture, auteur, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &id,
            &reference,
            &data.case_id,
            &data.titre,
            &data.description,
            &data.statut.unwrap_or_else(|| "en_cours".to_string()),
            &now,
            &data.date_echeance,
            Option::<&str>::None,
            &data.auteur,
            &data.metadata,
        ],
    )?;

    // Add to FTS index
    conn.execute(
        "INSERT INTO fts_reports (rowid, reference, titre, description) VALUES (?, ?, ?, ?)",
        rusqlite::params![&id, &reference, &data.titre, &data.description],
    )?;

    // Re-fetch to get full record
    get_report(state, id).await.map(|r| r.ok_or_else(|| anyhow::anyhow!("Failed to retrieve created report")))
}

// =============================================================================
// UPDATE REPORT
// =============================================================================

#[command]
pub async fn update_report(
    state: tauri::State<'_, AppState>,
    id: String,
    data: UpdateReportInput,
) -> Result<Report> {
    let mut conn = state.get_conn().await;

    // Build dynamic UPDATE
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
    if let Some(date_echeance) = &data.date_echeance {
        updates.push("dateEcheance = ?");
        params.push(date_echeance);
    }
    if let Some(metadata) = &data.metadata {
        updates.push("metadata = ?");
        params.push(metadata);
    }

    updates.push("dateMiseJour = ?");
    params.push(&Utc::now().to_rfc3339());

    params.push(&id);

    let query = format!("UPDATE reports SET {} WHERE id = ?", updates.join(", "));
    conn.execute(&query, params)?;

    // Update FTS index
    if let Some(titre) = &data.titre {
        conn.execute(
            "UPDATE fts_reports SET titre = ? WHERE rowid = ?",
            rusqlite::params![titre, &id],
        )?;
    }

    // Re-fetch to get full record
    get_report(state, id).await.map(|r| r.ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated report")))
}

// =============================================================================
// DELETE REPORT
// =============================================================================

#[command]
pub async fn delete_report(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<()> {
    let mut conn = state.get_conn().await;

    // Remove from FTS index
    conn.execute(
        "DELETE FROM fts_reports WHERE rowid = ?",
        rusqlite::params![&id],
    )?;

    // Delete contents
    conn.execute(
        "DELETE FROM report_contents WHERE reportId = ?",
        rusqlite::params![&id],
    )?;

    // Delete timeline
    conn.execute(
        "DELETE FROM report_timeline WHERE reportId = ?",
        rusqlite::params![&id],
    )?;

    // Delete report
    conn.execute(
        "DELETE FROM reports WHERE id = ?",
        rusqlite::params![&id],
    )?;

    Ok(())
}

// =============================================================================
// GET REPORT STATS
// =============================================================================

#[command]
pub async fn get_report_stats(state: tauri::State<'_, AppState>) -> Result<ReportStats> {
    let mut conn = state.get_conn().await;

    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM reports",
        rusqlite::params![],
        |row| row.get(0),
    )?;

    // Count by status
    let by_status: Vec<(String, i64)> = conn.query_map(
        "SELECT statut, COUNT(*) FROM reports GROUP BY statut",
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?.collect::<Result<_, _>>()?;

    let mut by_status_map = serde_json::Map::new();
    for (statut, count) in by_status {
        by_status_map.insert(statut, count);
    }

    // Count by case
    let by_case: Vec<(String, i64)> = conn.query_map(
        "SELECT caseId, COUNT(*) FROM reports GROUP BY caseId",
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?.collect::<Result<_, _>>()?;

    let mut by_case_map = serde_json::Map::new();
    for (cid, count) in by_case {
        by_case_map.insert(cid, count);
    }

    Ok(ReportStats {
        total,
        by_status: by_status_map,
        by_case: by_case_map,
    })
}

// =============================================================================
// ADD REPORT CONTENT
// =============================================================================

#[command]
pub async fn add_report_content(
    state: tauri::State<'_, AppState>,
    report_id: String,
    section: String,
    contenu: String,
    ordre: Option<i32>,
) -> Result<ReportContent> {
    let mut conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();
    let id = generate_uuid();

    // Check report exists
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM reports WHERE id = ?",
        rusqlite::params![report_id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Err(anyhow::anyhow!("Report not found: {}", report_id));
    }

    let ordre = ordre.unwrap_or_else(|| {
        conn.query_row(
            "SELECT COALESCE(MAX(ordre), 0) + 1 FROM report_contents WHERE reportId = ?",
            rusqlite::params![report_id],
            |row| row.get(0),
        ).unwrap_or(1)
    });

    conn.execute(
        "INSERT INTO report_contents (id, reportId, section, contenu, ordre, metadata) VALUES (?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &id,
            &report_id,
            &section,
            &contenu,
            &ordre,
            &serde_json::json!({"created_by": "system", "created_at": now}),
        ],
    )?;

    // Re-fetch to get full record
    let mut stmt = conn.prepare("SELECT id, reportId, section, contenu, ordre, metadata FROM report_contents WHERE id = ?")?;

    let content = stmt.query_row(rusqlite::params![id], |row| {
        Ok(ReportContent {
            id: row.get(0)?,
            report_id: row.get(1)?,
            section: row.get(2)?,
            contenu: row.get(3)?,
            ordre: row.get(4)?,
            metadata: row.get(5)?,
        })
    })?;

    Ok(content)
}

// =============================================================================
// GET REPORT CONTENTS
// =============================================================================

#[command]
pub async fn get_report_contents(
    state: tauri::State<'_, AppState>,
    report_id: String,
) -> Result<Vec<ReportContent>> {
    let mut conn = state.get_conn().await;

    let mut stmt = conn.prepare("SELECT id, reportId, section, contenu, ordre, metadata FROM report_contents WHERE reportId = ? ORDER BY ordre")?;

    let contents = stmt.query_map(rusqlite::params![report_id], |row| {
        Ok(ReportContent {
            id: row.get(0)?,
            report_id: row.get(1)?,
            section: row.get(2)?,
            contenu: row.get(3)?,
            ordre: row.get(4)?,
            metadata: row.get(5)?,
        })
    })?;

    let result: Result<Vec<_>, _> = contents.collect();
    Ok(result?)
}

// =============================================================================
// ADD REPORT TIMELINE EVENT
// =============================================================================

#[command]
pub async fn add_report_timeline_event(
    state: tauri::State<'_, AppState>,
    report_id: String,
    date_event: String,
    description: String,
) -> Result<ReportTimelineEvent> {
    let mut conn = state.get_conn().await;
    let now = Utc::now().to_rfc3339();
    let id = generate_uuid();

    // Check report exists
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM reports WHERE id = ?",
        rusqlite::params![report_id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Err(anyhow::anyhow!("Report not found: {}", report_id));
    }

    conn.execute(
        "INSERT INTO report_timeline (id, reportId, dateEvent, description, type, metadata) VALUES (?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &id,
            &report_id,
            &date_event,
            &description,
            Option::<&str>::None,
            &serde_json::json!({"created_by": "system", "created_at": now}),
        ],
    )?;

    // Re-fetch to get full record
    let mut stmt = conn.prepare("SELECT id, reportId, dateEvent, description, type, metadata FROM report_timeline WHERE id = ?")?;

    let event = stmt.query_row(rusqlite::params![id], |row| {
        Ok(ReportTimelineEvent {
            id: row.get(0)?,
            report_id: row.get(1)?,
            date_event: row.get(2)?,
            description: row.get(3)?,
            r#type: row.get(4)?,
            metadata: row.get(5)?,
        })
    })?;

    Ok(event)
}

// =============================================================================
// GET REPORT TIMELINE
// =============================================================================

#[command]
pub async fn get_report_timeline(
    state: tauri::State<'_, AppState>,
    report_id: String,
) -> Result<Vec<ReportTimelineEvent>> {
    let mut conn = state.get_conn().await;

    let mut stmt = conn.prepare("SELECT id, reportId, dateEvent, description, type, metadata FROM report_timeline WHERE reportId = ? ORDER BY dateEvent DESC")?;

    let events = stmt.query_map(rusqlite::params![report_id], |row| {
        Ok(ReportTimelineEvent {
            id: row.get(0)?,
            report_id: row.get(1)?,
            date_event: row.get(2)?,
            description: row.get(3)?,
            r#type: row.get(4)?,
            metadata: row.get(5)?,
        })
    })?;

    let result: Result<Vec<_>, _> = events.collect();
    Ok(result?)
}
