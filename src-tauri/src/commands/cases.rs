//! Cases commands module
//!
//! Provides CRUD operations for case management.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::command;
use crate::database::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
) -> AppResult<Vec<Case>> {
    let conn = state.get_conn().await;

    // Les filtres sont indépendants : une recherche sans filtre de statut ne
    // doit pas retomber implicitement sur `statut = 'ouvert'`, ce qui rendait
    // les dossiers clos introuvables (`audit.md`, P2-6).
    let mut query = String::from(
        "SELECT id, reference, titre, description, statut, priorite, categorie, dateCreation, dateMiseJour, tags, meta FROM cases",
    );
    let mut clauses: Vec<&str> = Vec::new();
    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();

    if let Some(statut) = &statut {
        clauses.push("statut = ?");
        params.push(statut);
    }
    if let Some(search) = &search {
        clauses.push(
            "(reference LIKE '%' || ? || '%' OR titre LIKE '%' || ? || '%' OR description LIKE '%' || ? || '%')",
        );
        params.push(search);
        params.push(search);
        params.push(search);
    }
    if !clauses.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&clauses.join(" AND "));
    }

    let mut stmt = conn.prepare(&query)?;
    let cases = stmt.query_map(&params[..], |row| {
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
) -> AppResult<Option<Case>> {
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
pub async fn get_case_stats(state: tauri::State<'_, AppState>) -> AppResult<serde_json::Value> {
    let conn = state.get_conn().await;

    let mut stats = serde_json::Map::new();

    let statut_counts: Vec<(String, i64)> = conn
        .prepare("SELECT statut, COUNT(*) FROM cases GROUP BY statut")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<_, _>>()?;

    stats.insert("byStatus".to_string(), serde_json::to_value(statut_counts)?);

    let priorite_counts: Vec<(String, i64)> = conn
        .prepare("SELECT priorite, COUNT(*) FROM cases GROUP BY priorite")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<_, _>>()?;

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
) -> AppResult<Case> {
    let conn = state.get_conn().await;
    let now = chrono::Utc::now().to_rfc3339();

    let id = crate::database::generate_uuid();
    // Le verrou est déjà détenu par `conn` : reprendre le mutex ici provoquait
    // un interblocage (`audit.md`, P2-8). On réutilise la connexion verrouillée.
    let reference = crate::database::generate_case_reference(&conn)?;

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

    // L'indexation plein texte est faite par les déclencheurs SQL
    // (`create_fts_triggers`) : l'insertion manuelle produisait un doublon
    // (`audit.md`, P2-9) et échouait de toute façon, `rowid` devant être un
    // entier alors qu'on lui passait un UUID (P2-2).

    // Premier maillon de la chaîne d'audit.
    //
    // La version d'origine stockait la **chaîne littérale**
    // `hex(sha256('create:case:…'))` — du texte, pas un hachage — et `"NULL"`
    // en guise de maillon précédent (`audit.md`, P1-3, P1-4). La vérification
    // signalait donc une rupture dès le premier événement, ce qu'a révélé le
    // parcours de bout en bout.
    crate::database::append_audit_event(
        &conn,
        &id,
        "create",
        "case",
        Some(&id),
        "system",
        serde_json::json!({ "reference": reference }),
    )?;

    drop(conn);
    get_case(state, id)
        .await?
        .ok_or_else(|| AppError::msg("dossier introuvable après création"))
}

// =============================================================================
// UPDATE CASE
// =============================================================================

#[command]
pub async fn update_case(
    state: tauri::State<'_, AppState>,
    id: String,
    data: UpdateCaseInput,
) -> AppResult<Case> {
    let conn = state.get_conn().await;

    // Bloc explicite : `Vec<&dyn ToSql>` n'est pas `Sync`, il ne doit donc pas
    // rester vivant au moment du `.await` final, sinon le futur de la commande
    // n'est plus `Send`.
    //
    // Les valeurs dérivées sont liées ici : les pousser directement dans
    // `params` créait des temporaires libérés avant l'exécution de la requête
    // (`audit.md`, P0-1g).
    {
    let tags_json = data.tags.as_ref().map(serde_json::to_string).transpose()?;
    let now = chrono::Utc::now().to_rfc3339();

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
    if let Some(tags_json) = &tags_json {
        updates.push("tags = ?");
        params.push(tags_json);
    }
    if let Some(meta) = &data.meta {
        updates.push("meta = ?");
        params.push(meta);
    }

    updates.push("dateMiseJour = ?");
    params.push(&now);

    params.push(&id);

    let query = format!("UPDATE cases SET {} WHERE id = ?", updates.join(", "));
    conn.execute(&query, &params[..])?;

    // Update FTS index
    if let Some(titre) = &data.titre {
        conn.execute(
            "UPDATE fts_cases SET titre = ? WHERE rowid = ?",
            rusqlite::params![titre, &id],
        )?;
    }

    // Récupère le dernier hash d'audit pour ce dossier (chaînage).
    let previous_hash: Option<String> = conn
        .query_row(
            "SELECT imma FROM audit_events WHERE caseId = ? ORDER BY timestamp DESC LIMIT 1",
            rusqlite::params![&id],
            |row| row.get(0),
        )
        .ok();
    let previous_hash = previous_hash.unwrap_or_default();

    // Construit l'événement d'audit et calcule son hash chaîné.
    let audit_event = spectra_audit::AuditEvent {
        id: crate::database::generate_uuid(),
        case_id: id.clone(),
        action: "update".to_string(),
        entity_kind: "case".to_string(),
        entity_id: Some(id.clone()),
        actor: "system".to_string(),
        sequence: 1,
        payload: serde_json::json!({"titre": data.titre, "statut": data.statut}),
    };
    let link = spectra_audit::compute_link(&audit_event, &previous_hash);

    conn.execute(
        "INSERT INTO audit_events (id, caseId, action, entityKind, entityId, imma, imma_precedent, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &audit_event.id,
            &id,
            &audit_event.action,
            &audit_event.entity_kind,
            &audit_event.entity_id,
            &link.hash,
            &link.previous_hash,
            &serde_json::to_string(&audit_event.payload)?,
        ],
    )?;
    }

    drop(conn);
    get_case(state, id)
        .await?
        .ok_or_else(|| AppError::msg("dossier introuvable après mise à jour"))
}

// =============================================================================
// DELETE CASE
// =============================================================================

#[command]
pub async fn delete_case(
    state: tauri::State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    let conn = state.get_conn().await;

    // Récupère le dernier hash d'audit pour ce dossier (chaînage).
    let previous_hash: Option<String> = conn
        .query_row(
            "SELECT imma FROM audit_events WHERE caseId = ? ORDER BY timestamp DESC LIMIT 1",
            rusqlite::params![&id],
            |row| row.get(0),
        )
        .ok();
    let previous_hash = previous_hash.unwrap_or_default();

    // Construit l'événement d'audit et calcule son hash chaîné.
    let audit_event = spectra_audit::AuditEvent {
        id: crate::database::generate_uuid(),
        case_id: id.clone(),
        action: "delete".to_string(),
        entity_kind: "case".to_string(),
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
            &id,
            &audit_event.action,
            &audit_event.entity_kind,
            &audit_event.entity_id,
            &link.hash,
            &link.previous_hash,
            &serde_json::to_string(&audit_event.payload)?,
        ],
    )?;

    conn.execute(
        "DELETE FROM fts_cases WHERE rowid = ?",
        rusqlite::params![&id],
    )?;

    conn.execute("DELETE FROM cases WHERE id = ?", rusqlite::params![&id])?;

    Ok(())
}
