//! Cekarna — application de bureau d'enquêtes OSINT.
//!
//! Point d'entrée et enregistrement des commandes Tauri.
//!
//! Le shell applicatif reste celui de Cekarna ; les briques de domaine SPECTRA
//! (`spectra-core`, `spectra-store`, `spectra-audit`) sont introduites
//! progressivement par les crates du workspace.

mod commands;
pub mod database;
mod error;

pub use database::AppState;
pub use error::{AppError, AppResult};

use tauri::Manager;

/// Démarre l'application.
///
/// La base est ouverte dans le répertoire de données applicatives fourni par
/// Tauri, et non dans le répertoire courant : deux lancements depuis des
/// répertoires différents doivent voir le même dossier d'enquête
/// (voir `audit.md`, P1-10).
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            // Magasin de preuves : les fichiers sont **copiés** ici à
            // l'ingestion. Conserver un simple chemin vers un fichier externe
            // ne garantit rien — il peut être modifié ou supprimé après
            // enregistrement (`audit.md`, P1-2).
            let storage_root = data_dir.join("storage");
            std::fs::create_dir_all(&storage_root)?;

            let conn = database::init_database(&data_dir.join("casetrack.db"))?;
            app.manage(AppState::new(conn, storage_root));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_cases,
            commands::get_case,
            commands::get_case_stats,
            commands::create_case,
            commands::update_case,
            commands::delete_case,
            commands::get_subjects,
            commands::create_subject,
            commands::update_subject,
            commands::delete_subject,
            commands::get_evidence,
            commands::add_evidence,
            commands::delete_evidence,
            commands::get_events,
            commands::add_note,
            commands::take_snapshot,
            commands::list_snapshots,
            commands::get_snapshot,
            commands::verify_snapshot_integrity,
            commands::delete_snapshot,
            commands::load_snapshot_bundle,
            commands::verify_evidence,
            commands::verify_case_evidence,
            commands::verify_audit_trail,
            commands::list_audit,
            commands::log_access,
            commands::set_claim,
            commands::list_claims,
            commands::get_claim,
            commands::delete_claim,
            commands::get_claim_stats,
            commands::search_action,
            commands::reindex_all,
            commands::get_reports,
            commands::get_report,
            commands::create_report,
            commands::update_report,
            commands::delete_report,
            commands::get_report_stats,
            commands::add_report_content,
            commands::get_report_contents,
            commands::add_report_timeline_event,
            commands::get_report_timeline,
        ])
        .run(tauri::generate_context!())
        .expect("erreur au démarrage de l'application Tauri");
}
