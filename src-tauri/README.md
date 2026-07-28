# Cekarna - Tauri Backend

This directory contains the Rust backend for the Cekarna desktop OSINT application.

## Project Structure

```
src-tauri/
├── src/
│   ├── lib.rs          # Main entry point and command registration
│   ├── main.rs         # Tauri application entry point
│   ├── database.rs     # SQLite database management (24 tables)
│   └── commands/
│       ├── cases.rs        # Case CRUD operations
│       ├── evidence.rs     # Evidence management
│       ├── subjects.rs     # Subject management
│       ├── events.rs       # Event and note management
│       ├── snapshots.rs    # Case snapshot management
│       ├── integrity.rs    # Integrity verification
│       ├── audit.rs        # Audit trail management
│       ├── claims.rs       # Claims management
│       ├── search.rs       # FTS5 search
│       ├── reports.rs      # Report management
│       └── settings.rs     # Ollama AI settings
├── Cargo.toml          # Rust dependencies
├── build.rs            # Build script
└── tauri.conf.json     # Tauri configuration
```

## Database Schema

The application uses SQLite with 24 tables:

1. **cases** - Case management
2. **case_events** - Case event timeline
3. **evidence** - Evidence items with hash verification
4. **subjects** - Suspects, victims, witnesses
5. **tiktok_videos** - TikTok video evidence
6. **reports** - Investigation reports
7. **report_contents** - Report content sections
8. **report_timeline** - Report timeline events
9. **claims** - Claims about evidence/subjects
10. **audit_events** - Audit trail with IMMA chaining
11. **snapshots** - Case snapshots with hash verification
12. **snapshot_contents** - Snapshot contents
13. **integrity_logs** - Integrity verification logs
14. **ollama_settings** - AI settings
15. **users** - User management
16. **session_logs** - Session tracking
17. **web_archives** - Web archive evidence
18. **notes** - Free-form notes
19. **evidence_subject_links** - Evidence-subject relationships
20. **case_tags** - Case tag junction
21. **report_tags** - Report tag junction
22. **audit_settings** - Audit configuration
23. **FTS5 tables** - Full-text search indexes
24. **tags** - Tag definitions

## Commands

All Tauri commands are documented in the TypeScript frontend at `lib/tauri-bridge.ts`.

## Building

```bash
cd src-tauri
cargo build --release
```

## Running

```bash
cargo run
```

## Testing

```bash
cargo test
```
