//! SPECTRA — application de bureau d'investigation OSINT.
//!
//! Point d'entrée et enregistrement des commandes Tauri.
//!
//! Les briques de domaine SPECTRA (`spectra-core`, `spectra-store`,
//! `spectra-audit`, `spectra-ai`, `spectra-report`) sont introduites
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let storage_root = data_dir.join("storage");
            std::fs::create_dir_all(&storage_root)?;

            let conn = database::init_database(&data_dir.join("casetrack.db"))?;

            // Détection async du backend IA (Ollama) — block_on car setup est sync.
            let ai_service = tauri::async_runtime::block_on(async {
                spectra_ai::AiService::new_auto_detect().await
            });

            app.manage(AppState::new(conn, storage_root, ai_service));
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
            commands::run_osint_campaign,
            commands::load_osint_probes,
            commands::update_osint_datasets,
            commands::ai_status,
            commands::ai_summarize,
            commands::ai_extract_entities,
            commands::ai_suggest_pivots,
            commands::ai_detect_duplicates,
            commands::ai_rag_query,
        ])
        .run(tauri::generate_context!())
        .expect("erreur au démarrage de l'application Tauri");
}
