//! Cekarna - Desktop OSINT application for case investigations
//!
//! Main entry point and command registration.

use tauri::Manager;

mod database;
mod commands;

pub use database::AppState;

/// Application configuration
pub const APP_CONFIG: &str = r#"
{
  "app_name": "Cekarna",
  "version": "0.1.0",
  "data_dir": ".cekarna",
  "max_evidence_size_mb": 100,
  "hash_algorithm": "sha256",
  "audit_enabled": true
}
"#;


// =============================================================================
// MAIN - Command registration and app startup
// =============================================================================

fn main() {
    // Initialize database first (to check path and create tables)
    let database_path = std::path::Path::new("casetrack.db");
    let conn = database::init_database(database_path)
        .map_err(|e| format!("Failed to initialize database: {}", e))
        .expect("Failed to initialize database");

    tauri::Builder::default()
        .setup(|app| {
            // Store the connection in app state by replacing the default one
            // We need to use State::inner() to get mutable access
            let mut state = app.state::<AppState>();
            state.inner().set_connection(conn);

            // Initialize CLI arguments if any
            let args: Vec<String> = std::env::args().collect();
            if !args.is_empty() {
                eprintln!("Application started with args: {:?}", &args[1..]);
            }

            Ok(())
        })
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Re-exported commands from commands module
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
        .run()
        .expect("error while running tauri application");
}
