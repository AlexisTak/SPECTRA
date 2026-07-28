//! Vérification d'intégrité des preuves.
//!
//! # Ce que « vérifier » veut dire ici
//!
//! L'implémentation d'origine considérait une preuve « vérifiée » dès lors que
//! la colonne `hash_sha256` contenait une chaîne non vide, et renvoyait
//! `"Hash verified successfully"` (`audit.md`, P1-1). Le fichier n'était jamais
//! ouvert : une preuve modifiée, remplacée ou supprimée du disque était
//! rapportée intacte.
//!
//! Vérifier, c'est **relire le fichier et recalculer son empreinte**. Tout le
//! reste est une lecture de métadonnées.

use crate::database::AppState;
use crate::error::AppResult;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::command;

/// Résultat de la vérification d'une preuve.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntegrityCheck {
    pub evidence_id: String,
    pub nom: Option<String>,
    /// `intact` | `altered` | `missing` | `no_hash` | `no_path`
    pub status: String,
    pub expected_hash: Option<String>,
    pub actual_hash: Option<String>,
    pub message: String,
    pub checked_at: String,
}

/// Bilan pour un dossier.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CaseIntegrityReport {
    pub case_id: String,
    pub total: i64,
    pub intact: i64,
    /// Empreinte recalculée différente : la preuve a été modifiée.
    pub altered: i64,
    /// Fichier absent du magasin.
    pub missing: i64,
    /// Preuve sans empreinte enregistrée (ingérée avant la correction P1-2).
    pub no_hash: i64,
    pub checks: Vec<IntegrityCheck>,
    pub generated_at: String,
}

/// Vérifie une preuve en relisant son fichier.
#[command]
pub async fn verify_evidence(
    state: tauri::State<'_, AppState>,
    evidence_id: String,
) -> AppResult<IntegrityCheck> {
    let conn = state.get_conn().await;

    let (nom, chemin, expected): (Option<String>, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT nom, chemin, hash_sha256 FROM evidence WHERE id = ?",
            rusqlite::params![evidence_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

    Ok(check_one(
        &evidence_id,
        nom,
        chemin.as_deref(),
        expected.as_deref(),
    ))
}

/// Vérifie toutes les preuves d'un dossier et journalise le contrôle.
#[command]
pub async fn verify_case_evidence(
    state: tauri::State<'_, AppState>,
    case_id: String,
) -> AppResult<CaseIntegrityReport> {
    let conn = state.get_conn().await;

    let mut stmt = conn.prepare(
        "SELECT id, nom, chemin, hash_sha256 FROM evidence WHERE caseId = ? ORDER BY dateAjout",
    )?;

    let rows: Vec<(String, Option<String>, Option<String>, Option<String>)> = stmt
        .query_map(rusqlite::params![case_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .filter_map(Result::ok)
        .collect();
    drop(stmt);

    let checks: Vec<IntegrityCheck> = rows
        .iter()
        .map(|(id, nom, chemin, expected)| {
            check_one(id, nom.clone(), chemin.as_deref(), expected.as_deref())
        })
        .collect();

    let count = |status: &str| checks.iter().filter(|c| c.status == status).count() as i64;
    let intact = count("intact");
    let altered = count("altered");
    let missing = count("missing");
    let no_hash = count("no_hash") + count("no_path");
    let generated_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO integrity_logs \
         (id, caseId, checkedAt, checkedBy, totalEvidence, verifiedEvidence, brokenHash, message) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &crate::database::generate_uuid(),
            &case_id,
            &generated_at,
            "system",
            &(checks.len() as i64),
            &intact,
            // Ne compte que les altérations avérées : une preuve sans empreinte
            // n'est pas corrompue, elle est non vérifiable.
            &(altered + missing),
            &format!(
                "{intact} intactes, {altered} altérées, {missing} manquantes, {no_hash} non vérifiables"
            ),
        ],
    )?;

    // Le contrôle d'intégrité est lui-même un acte d'enquête : il entre dans
    // la chaîne d'audit.
    crate::database::append_audit_event(
        &conn,
        &case_id,
        "verify",
        "evidence",
        None,
        "system",
        serde_json::json!({
            "total": checks.len(),
            "intact": intact,
            "altered": altered,
            "missing": missing,
        }),
    )?;

    Ok(CaseIntegrityReport {
        case_id,
        total: checks.len() as i64,
        intact,
        altered,
        missing,
        no_hash,
        checks,
        generated_at,
    })
}

/// Relit un fichier et compare son empreinte à celle enregistrée.
fn check_one(
    evidence_id: &str,
    nom: Option<String>,
    chemin: Option<&str>,
    expected: Option<&str>,
) -> IntegrityCheck {
    let checked_at = Utc::now().to_rfc3339();

    let make = |status: &str, actual: Option<String>, message: &str| IntegrityCheck {
        evidence_id: evidence_id.to_string(),
        nom: nom.clone(),
        status: status.to_string(),
        expected_hash: expected.map(str::to_string),
        actual_hash: actual,
        message: message.to_string(),
        checked_at: checked_at.clone(),
    };

    let Some(chemin) = chemin else {
        return make("no_path", None, "aucun fichier associé à cette preuve");
    };

    let Some(expected) = expected else {
        return make(
            "no_hash",
            None,
            "aucune empreinte enregistrée : preuve non vérifiable",
        );
    };

    let Ok(bytes) = std::fs::read(std::path::Path::new(chemin)) else {
        return make("missing", None, "fichier absent ou illisible dans le magasin");
    };

    let actual = crate::database::sha256_hash_bytes(&bytes);
    if actual == expected {
        make("intact", Some(actual), "empreinte recalculée conforme")
    } else {
        make(
            "altered",
            Some(actual),
            "empreinte recalculée différente : le fichier a été modifié",
        )
    }
}
