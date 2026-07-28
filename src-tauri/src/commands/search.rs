//! Search commands module
//!
//! Provides full-text search using FTS5.

use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use tauri::command;
use crate::database::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub id: String,
    pub kind: String,
    pub case_id: Option<String>,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct _SearchOptions {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub highlight: Option<bool>,
}

// =============================================================================
// SEARCH ACTION
// =============================================================================

#[command]
pub async fn search_action(
    state: tauri::State<'_, AppState>,
    query: String,
    kinds: Option<Vec<String>>,
    case_id: Option<String>,
) -> AppResult<Vec<SearchHit>> {
    let mut conn = state.get_conn().await;
    let opts = kinds.unwrap_or_default();

    let mut results = Vec::new();

    // Search in cases
    if opts.is_empty() || opts.contains(&"case".to_string()) {
        results.extend(search_fts_table(&mut *conn, "fts_cases", "cases", &query, case_id.clone())?);
    }

    // Search in evidence
    if opts.is_empty() || opts.contains(&"evidence".to_string()) {
        results.extend(search_fts_table(&mut *conn, "fts_evidence", "evidence", &query, case_id.clone())?);
    }

    // Search in subjects
    if opts.is_empty() || opts.contains(&"subject".to_string()) {
        results.extend(search_fts_table(&mut *conn, "fts_subjects", "subjects", &query, case_id.clone())?);
    }

    // Search in reports
    if opts.is_empty() || opts.contains(&"report".to_string()) {
        results.extend(search_fts_table(&mut *conn, "fts_reports", "reports", &query, case_id.clone())?);
    }

    // Search in web archives
    if opts.is_empty() || opts.contains(&"web_archive".to_string()) {
        results.extend(search_fts_table(&mut *conn, "fts_web_archives", "web_archives", &query, case_id.clone())?);
    }

    // Search in TikTok videos
    if opts.is_empty() || opts.contains(&"tiktok_video".to_string()) {
        results.extend(search_fts_table(&mut *conn, "fts_tiktok_videos", "tiktok_videos", &query, case_id.clone())?);
    }

    // Search in notes
    if opts.is_empty() || opts.contains(&"note".to_string()) {
        results.extend(search_fts_table(&mut *conn, "fts_notes", "notes", &query, case_id)?);
    }

    // Sort by score and limit
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(50);

    Ok(results)
}

fn search_fts_table(
    conn: &mut rusqlite::Connection,
    fts_table: &str,
    content_table: &str,
    query: &str,
    case_id: Option<String>,
) -> AppResult<Vec<SearchHit>> {
    // Use FTS5 MATCH query
    let mut stmt = conn.prepare(&format!(
        "SELECT rowid, nom, description FROM {} WHERE {} MATCH ? ORDER BY rank",
        fts_table, fts_table
    ))?;

    let hits = stmt.query_map(rusqlite::params![query], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?))
    })?;

    let mut results = Vec::new();
    for (rowid, nom, description) in hits.filter_map(|r| r.ok()).collect::<Vec<_>>() {
        // Get case_id from content table
        let cid: Option<String> = conn.query_row(
            &format!("SELECT caseId FROM {} WHERE id = ?", content_table),
            rusqlite::params![rowid],
            |row| row.get(0),
        ).ok();

        // Filtre par dossier. La version d'origine comparait une variable à
        // elle-même (masquage de nom) : le filtre n'avait aucun effet et la
        // recherche « dans ce dossier » renvoyait tous les dossiers
        // (`audit.md`, P2-4).
        if let Some(wanted) = &case_id {
            if cid.as_deref() != Some(wanted.as_str()) {
                continue;
            }
        }

        results.push(SearchHit {
            id: rowid.to_string(),
            kind: content_table.to_string(),
            case_id: cid,
            title: nom,
            snippet: description.unwrap_or_default(),
            score: 0.0, // FTS5 score would require custom implementation
        });
    }

    Ok(results)
}

// =============================================================================
// REINDEX ALL
// =============================================================================

#[command]
pub async fn reindex_all(state: tauri::State<'_, AppState>) -> AppResult<()> {
    let mut conn = state.get_conn().await;

    // Rebuild FTS indexes
    conn.execute("INSERT INTO fts_cases(fts_cases) VALUES('rebuild')", rusqlite::params!())?;
    conn.execute("INSERT INTO fts_evidence(fts_evidence) VALUES('rebuild')", rusqlite::params!())?;
    conn.execute("INSERT INTO fts_subjects(fts_subjects) VALUES('rebuild')", rusqlite::params!())?;
    conn.execute("INSERT INTO fts_reports(fts_reports) VALUES('rebuild')", rusqlite::params!())?;
    conn.execute("INSERT INTO fts_web_archives(fts_web_archives) VALUES('rebuild')", rusqlite::params!())?;
    conn.execute("INSERT INTO fts_notes(fts_notes) VALUES('rebuild')", rusqlite::params!())?;
    conn.execute("INSERT INTO fts_tiktok_videos(fts_tiktok_videos) VALUES('rebuild')", rusqlite::params!())?;

    Ok(())
}
