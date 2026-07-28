//! Database module for Cekarna
//!
//! Provides:
//! - SQLite connection management
//! - 24 tables with FTS5 search support
//! - Triggers for audit trail and integrity
//! - WAL mode for concurrent access

use crate::error::AppResult;
use chrono::{Datelike, Utc};
use rusqlite::{Connection, params};
use std::path::Path;
use tokio::sync::Mutex;

/// État applicatif partagé par les commandes.
///
/// `tokio::sync::Mutex` est utilisé car il peut envelopper un type `!Sync`.
/// Le type n'est délibérément **pas** `Clone` : `Mutex<Connection>` ne l'est
/// pas, et dupliquer la connexion à la base d'enquête n'aurait pas de sens.
///
/// Ce mutex n'est **pas réentrant** : une fonction qui détient déjà le verrou ne
/// doit jamais appeler une fonction qui le reprend — cause de deux
/// interblocages dans le code d'origine (`audit.md`, P2-8).
pub struct AppState {
    conn: Mutex<Connection>,
}

impl AppState {
    /// Construit l'état à partir d'une connexion déjà initialisée.
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }

    /// Prend le verrou sur la connexion.
    pub async fn get_conn(&self) -> tokio::sync::MutexGuard<'_, Connection> {
        self.conn.lock().await
    }
}

/// Initialize the database with all tables and triggers
pub fn init_database(db_path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(db_path)?;

    // Enable WAL mode for concurrent access
    conn.pragma_update(None, "journal_mode", "wal")?;

    // Enable foreign keys
    conn.pragma_update(None, "foreign_keys", "on")?;

    // Create all tables and triggers
    create_tables(&conn)?;

    Ok(conn)
}

fn create_tables(conn: &Connection) -> AppResult<()> {
    // 1. Cases table
    conn.execute_batch(CASES_TABLE)?;

    // 2. Case events table
    conn.execute_batch(CASE_EVENTS_TABLE)?;

    // 3. Evidence table
    conn.execute_batch(EVIDENCE_TABLE)?;

    // 4. Subjects table
    conn.execute_batch(SUBJECTS_TABLE)?;

    // 5. Tiktok videos table
    conn.execute_batch(TIKTOK_VIDEOS_TABLE)?;

    // 6. Reports table
    conn.execute_batch(REPORTS_TABLE)?;

    // 7. Report contents table
    conn.execute_batch(REPORT_CONTENTS_TABLE)?;

    // 8. Report timeline events table
    conn.execute_batch(REPORT_TIMELINE_TABLE)?;

    // 9. Claims table
    conn.execute_batch(CLAIMS_TABLE)?;

    // 10. Audit events table
    conn.execute_batch(AUDIT_EVENTS_TABLE)?;

    // 11. Snapshots table
    conn.execute_batch(SNAPSHOTS_TABLE)?;

    // 12. Snapshot contents table
    conn.execute_batch(SNAPSHOT_CONTENTS_TABLE)?;

    // 13. Integrity logs table
    conn.execute_batch(INTEGRITY_LOGS_TABLE)?;

    // 14. Ollama settings table
    conn.execute_batch(OLLAMA_SETTINGS_TABLE)?;

    // 15. FTS5 tables for search
    create_fts_tables(conn)?;

    // 16. Tags table and junction tables
    create_tags_tables(conn)?;

    // 17. Users table
    conn.execute_batch(USERS_TABLE)?;

    // 18. Session logs table
    conn.execute_batch(SESSION_LOGS_TABLE)?;

    // 19. Web archives table
    conn.execute_batch(WEB_ARCHIVES_TABLE)?;

    // 20. Notes table
    conn.execute_batch(NOTES_TABLE)?;

    // 21. Evidence subject links table
    conn.execute_batch(EVIDENCE_SUBJECT_LINKS_TABLE)?;

    // 22. Case tags junction table
    conn.execute_batch(CASE_TAGS_TABLE)?;

    // 23. Report tags junction table
    conn.execute_batch(REPORT_TAGS_TABLE)?;

    // 24. Audit settings table
    conn.execute_batch(AUDIT_SETTINGS_TABLE)?;

    // Create FTS triggers
    create_fts_triggers(conn)?;

    // Create audit triggers
    create_audit_triggers(conn)?;

    // Create integrity triggers
    create_integrity_triggers(conn)?;

    Ok(())
}

// =============================================================================
// TABLE CREATION SQL
// =============================================================================

const CASES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS cases (
    id TEXT PRIMARY KEY,
    reference TEXT UNIQUE NOT NULL,
    titre TEXT NOT NULL,
    description TEXT,
    statut TEXT NOT NULL DEFAULT 'ouvert' CHECK(statut IN ('ouvert', 'en_cours', 'transmis', 'clos')),
    priorite TEXT NOT NULL DEFAULT 'normale' CHECK(priorite IN ('basse', 'normale', 'haute', 'urgente')),
    categorie TEXT,
    dateCreation TEXT NOT NULL,
    dateMiseJour TEXT NOT NULL,
    tags TEXT,
    meta TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

const CASE_EVENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS case_events (
    id TEXT PRIMARY KEY,
    caseId TEXT NOT NULL,
    type TEXT NOT NULL,
    titre TEXT,
    description TEXT,
    timestamp TEXT NOT NULL,
    actor TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (caseId) REFERENCES cases(id) ON DELETE CASCADE
);
"#;

const EVIDENCE_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS evidence (
    id TEXT PRIMARY KEY,
    caseId TEXT NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('file', 'image', 'video', 'text', 'web_archive', 'tiktok')),
    nom TEXT,
    description TEXT,
    chemin TEXT,
    hash_sha256 TEXT,
    hash_md5 TEXT,
    taille INTEGER,
    dateAjout TEXT NOT NULL,
    statut TEXT DEFAULT 'en_attente' CHECK(statut IN ('en_attente', 'verifie', 'non_verifie', 'supprime')),
    source TEXT,
    sourceUrl TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (caseId) REFERENCES cases(id) ON DELETE CASCADE
);
"#;

const SUBJECTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS subjects (
    id TEXT PRIMARY KEY,
    caseId TEXT NOT NULL,
    nom TEXT,
    prenom TEXT,
    statut TEXT NOT NULL DEFAULT 'suspect' CHECK(statut IN ('suspect', 'victime', 'temoin', 'inconnu')),
    dateNaissance TEXT,
    lieuNaissance TEXT,
    nationalite TEXT,
    telephone TEXT,
    email TEXT,
    adresse TEXT,
    description TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (caseId) REFERENCES cases(id) ON DELETE CASCADE
);
"#;

const TIKTOK_VIDEOS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS tiktok_videos (
    id TEXT PRIMARY KEY,
    caseId TEXT NOT NULL,
    videoId TEXT NOT NULL,
    url TEXT NOT NULL,
    auteur TEXT,
    description TEXT,
    datePublication TEXT,
    likes INTEGER DEFAULT 0,
    commentaires INTEGER DEFAULT 0,
    partages INTEGER DEFAULT 0,
    vues INTEGER DEFAULT 0,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (caseId) REFERENCES cases(id) ON DELETE CASCADE
);
"#;

const REPORTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS reports (
    id TEXT PRIMARY KEY,
    reference TEXT UNIQUE NOT NULL,
    caseId TEXT NOT NULL,
    titre TEXT NOT NULL,
    description TEXT,
    statut TEXT NOT NULL DEFAULT 'en_cours' CHECK(statut IN ('en_cours', 'en_revue', 'termine', 'archive')),
    dateCreation TEXT NOT NULL,
    dateEcheance TEXT,
    dateCloture TEXT,
    auteur TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (caseId) REFERENCES cases(id) ON DELETE CASCADE
);
"#;

const REPORT_CONTENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS report_contents (
    id TEXT PRIMARY KEY,
    reportId TEXT NOT NULL,
    section TEXT NOT NULL,
    contenu TEXT,
    ordre INTEGER NOT NULL DEFAULT 0,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (reportId) REFERENCES reports(id) ON DELETE CASCADE
);
"#;

const REPORT_TIMELINE_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS report_timeline (
    id TEXT PRIMARY KEY,
    reportId TEXT NOT NULL,
    dateEvent TEXT NOT NULL,
    description TEXT NOT NULL,
    type TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (reportId) REFERENCES reports(id) ON DELETE CASCADE
);
"#;

const CLAIMS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS claims (
    id TEXT PRIMARY KEY,
    caseId TEXT NOT NULL,
    refKind TEXT NOT NULL CHECK(refKind IN ('evidence', 'subject', 'tiktok_video', 'web_archive', 'note')),
    refId TEXT NOT NULL,
    qualification TEXT NOT NULL CHECK(qualification IN ('preuve', 'indice', 'hypothese', 'non_verifie')),
    fiabilite INTEGER NOT NULL DEFAULT 0 CHECK(fiabilite BETWEEN 0 AND 5),
    source TEXT,
    sourceUrl TEXT,
    takenBy TEXT,
    notes TEXT,
    datePreuve TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (caseId) REFERENCES cases(id) ON DELETE CASCADE
);
"#;

const AUDIT_EVENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS audit_events (
    id TEXT PRIMARY KEY,
    caseId TEXT NOT NULL,
    action TEXT NOT NULL,
    entityKind TEXT NOT NULL,
    entityId TEXT,
    actor TEXT,
    metadata TEXT,
    imma TEXT NOT NULL,
    imma_precedent TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (caseId) REFERENCES cases(id) ON DELETE CASCADE
);
"#;

const SNAPSHOTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS snapshots (
    id TEXT PRIMARY KEY,
    caseId TEXT NOT NULL,
    nom TEXT NOT NULL,
    description TEXT,
    hashSha256 TEXT NOT NULL,
    hashPrecedent TEXT,
    taille INTEGER,
    chemin TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (caseId) REFERENCES cases(id) ON DELETE CASCADE
);
"#;

const SNAPSHOT_CONTENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS snapshot_contents (
    id TEXT PRIMARY KEY,
    snapshotId TEXT NOT NULL,
    nom TEXT NOT NULL,
    kind TEXT NOT NULL,
    contenu TEXT,
    chemin TEXT,
    hashSha256 TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (snapshotId) REFERENCES snapshots(id) ON DELETE CASCADE
);
"#;

const INTEGRITY_LOGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS integrity_logs (
    id TEXT PRIMARY KEY,
    caseId TEXT NOT NULL,
    checkedAt TEXT NOT NULL,
    checkedBy TEXT,
    totalEvidence INTEGER,
    verifiedEvidence INTEGER,
    brokenHash INTEGER,
    message TEXT,
    metadata TEXT,
    FOREIGN KEY (caseId) REFERENCES cases(id) ON DELETE CASCADE
);
"#;

const OLLAMA_SETTINGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS ollama_settings (
    id INTEGER PRIMARY KEY CHECK(id = 1),
    baseUrl TEXT NOT NULL DEFAULT 'http://localhost:11434',
    model TEXT DEFAULT 'llama3:instruct',
    temperature REAL DEFAULT 0.7,
    maxTokens INTEGER DEFAULT 4096,
    timeout INTEGER DEFAULT 30,
    statusTimeoutMs INTEGER DEFAULT 5000,
    enabled BOOLEAN DEFAULT 1,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

const USERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT,
    displayName TEXT,
    role TEXT NOT NULL DEFAULT 'user' CHECK(role IN ('admin', 'user', 'viewer')),
    lastLogin TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

const SESSION_LOGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS session_logs (
    id TEXT PRIMARY KEY,
    userId TEXT,
    action TEXT NOT NULL,
    details TEXT,
    ip TEXT,
    userAgent TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (userId) REFERENCES users(id)
);
"#;

const WEB_ARCHIVES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS web_archives (
    id TEXT PRIMARY KEY,
    caseId TEXT NOT NULL,
    url TEXT NOT NULL,
    title TEXT,
    content TEXT,
    hashSha256 TEXT,
    dateArchive TEXT NOT NULL,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (caseId) REFERENCES cases(id) ON DELETE CASCADE
);
"#;

const NOTES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    caseId TEXT NOT NULL,
    titre TEXT,
    contenu TEXT,
    tags TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (caseId) REFERENCES cases(id) ON DELETE CASCADE
);
"#;

const EVIDENCE_SUBJECT_LINKS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS evidence_subject_links (
    evidenceId TEXT NOT NULL,
    subjectId TEXT NOT NULL,
    relation TEXT NOT NULL DEFAULT 'associé',
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (evidenceId, subjectId),
    FOREIGN KEY (evidenceId) REFERENCES evidence(id) ON DELETE CASCADE,
    FOREIGN KEY (subjectId) REFERENCES subjects(id) ON DELETE CASCADE
);
"#;

const CASE_TAGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS case_tags (
    caseId TEXT NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY (caseId, tag),
    FOREIGN KEY (caseId) REFERENCES cases(id) ON DELETE CASCADE
);
"#;

const REPORT_TAGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS report_tags (
    reportId TEXT NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY (reportId, tag),
    FOREIGN KEY (reportId) REFERENCES reports(id) ON DELETE CASCADE
);
"#;

const AUDIT_SETTINGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS audit_settings (
    id INTEGER PRIMARY KEY CHECK(id = 1),
    enabled BOOLEAN DEFAULT 1,
    retentionDays INTEGER DEFAULT 365,
    maxEntries INTEGER DEFAULT 100000,
    hashAlgorithm TEXT NOT NULL DEFAULT 'sha256',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

// =============================================================================
// FTS5 SEARCH TABLES
// =============================================================================

fn create_fts_tables(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_cases USING fts5(
            reference,
            titre,
            description,
            content = 'cases',
            content_rowid = 'id'
        );
    "#)?;

    conn.execute_batch(r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_evidence USING fts5(
            nom,
            description,
            chemin,
            content = 'evidence',
            content_rowid = 'id'
        );
    "#)?;

    conn.execute_batch(r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_tiktok_videos USING fts5(
            videoId,
            auteur,
            description,
            content = 'tiktok_videos',
            content_rowid = 'id'
        );
    "#)?;

    conn.execute_batch(r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_subjects USING fts5(
            nom,
            prenom,
            description,
            email,
            telephone,
            content = 'subjects',
            content_rowid = 'id'
        );
    "#)?;

    conn.execute_batch(r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_reports USING fts5(
            reference,
            titre,
            description,
            content = 'reports',
            content_rowid = 'id'
        );
    "#)?;

    conn.execute_batch(r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_web_archives USING fts5(
            url,
            title,
            content,
            content = 'web_archives',
            content_rowid = 'id'
        );
    "#)?;

    conn.execute_batch(r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_notes USING fts5(
            titre,
            contenu,
            content = 'notes',
            content_rowid = 'id'
        );
    "#)?;

    Ok(())
}

fn create_fts_triggers(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(r#"
        CREATE TRIGGER IF NOT EXISTS cases_ai AFTER INSERT ON cases BEGIN
            INSERT INTO fts_cases(rowid, reference, titre, description)
            VALUES(new.id, new.reference, new.titre, new.description);
        END;
    "#)?;

    conn.execute_batch(r#"
        CREATE TRIGGER IF NOT EXISTS cases_ad AFTER DELETE ON cases BEGIN
            INSERT INTO fts_cases(fts_cases, rowid, reference, titre, description)
            VALUES('delete', old.id, old.reference, old.titre, old.description);
        END;
    "#)?;

    conn.execute_batch(r#"
        CREATE TRIGGER IF NOT EXISTS cases_au AFTER UPDATE ON cases BEGIN
            INSERT INTO fts_cases(fts_cases, rowid, reference, titre, description)
            VALUES('delete', old.id, old.reference, old.titre, old.description);
            INSERT INTO fts_cases(rowid, reference, titre, description)
            VALUES(new.id, new.reference, new.titre, new.description);
        END;
    "#)?;

    conn.execute_batch(r#"
        CREATE TRIGGER IF NOT EXISTS evidence_ai AFTER INSERT ON evidence BEGIN
            INSERT INTO fts_evidence(rowid, nom, description, chemin)
            VALUES(new.id, new.nom, new.description, new.chemin);
        END;
    "#)?;

    conn.execute_batch(r#"
        CREATE TRIGGER IF NOT EXISTS evidence_ad AFTER DELETE ON evidence BEGIN
            INSERT INTO fts_evidence(fts_evidence, rowid, nom, description, chemin)
            VALUES('delete', old.id, old.nom, old.description, old.chemin);
        END;
    "#)?;

    conn.execute_batch(r#"
        CREATE TRIGGER IF NOT EXISTS evidence_au AFTER UPDATE ON evidence BEGIN
            INSERT INTO fts_evidence(fts_evidence, rowid, nom, description, chemin)
            VALUES('delete', old.id, old.nom, old.description, old.chemin);
            INSERT INTO fts_evidence(rowid, nom, description, chemin)
            VALUES(new.id, new.nom, new.description, new.chemin);
        END;
    "#)?;

    conn.execute_batch(r#"
        CREATE TRIGGER IF NOT EXISTS tiktok_videos_ai AFTER INSERT ON tiktok_videos BEGIN
            INSERT INTO fts_tiktok_videos(rowid, videoId, auteur, description)
            VALUES(new.id, new.videoId, new.auteur, new.description);
        END;
    "#)?;

    conn.execute_batch(r#"
        CREATE TRIGGER IF NOT EXISTS tiktok_videos_ad AFTER DELETE ON tiktok_videos BEGIN
            INSERT INTO fts_tiktok_videos(fts_tiktok_videos, rowid, videoId, auteur, description)
            VALUES('delete', old.id, old.videoId, old.auteur, old.description);
        END;
    "#)?;

    conn.execute_batch(r#"
        CREATE TRIGGER IF NOT EXISTS tiktok_videos_au AFTER UPDATE ON tiktok_videos BEGIN
            INSERT INTO fts_tiktok_videos(fts_tiktok_videos, rowid, videoId, auteur, description)
            VALUES('delete', old.id, old.videoId, old.auteur, old.description);
            INSERT INTO fts_tiktok_videos(rowid, videoId, auteur, description)
            VALUES(new.id, new.videoId, new.auteur, new.description);
        END;
    "#)?;

    Ok(())
}

// =============================================================================
// AUDIT TRAIL TRIGGERS
// =============================================================================

fn create_audit_triggers(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(r#"
        CREATE TRIGGER IF NOT EXISTS audit_insert AFTER INSERT ON cases BEGIN
            INSERT INTO audit_events (id, caseId, action, entityKind, entityId, imma, imma_precedent, metadata)
            VALUES (
                'AE-' || strftime('%Y', 'now') || '-' || printf('%06d', (
                    SELECT COALESCE(MAX(id), 0) + 1 FROM audit_events WHERE entityKind = 'case'
                )),
                new.id,
                'create',
                'case',
                new.id,
                hex(sha256('insert:case:' || new.id || ':' || datetime('now'))),
                NULL,
                json_patch('{}', json_object('reference', new.reference))
            );
        END;
    "#)?;

    conn.execute_batch(r#"
        CREATE TRIGGER IF NOT EXISTS audit_update AFTER UPDATE ON cases BEGIN
            INSERT INTO audit_events (id, caseId, action, entityKind, entityId, imma, imma_precedent, metadata)
            VALUES (
                'AE-' || strftime('%Y', 'now') || '-' || printf('%06d', (
                    SELECT COALESCE(MAX(id), 0) + 1 FROM audit_events
                )),
                new.id,
                'update',
                'case',
                new.id,
                hex(sha256('update:case:' || new.id || ':' || datetime('now') || ':' || old.reference)),
                hex(sha256('update:case:' || old.id || ':' || datetime('now') || ':' || old.reference)),
                json_patch(json_object('before', json_object('reference', old.reference)), json_object('after', json_object('reference', new.reference)))
            );
        END;
    "#)?;

    Ok(())
}

// =============================================================================
// INTEGRITY TRIGGERS
// =============================================================================

fn create_integrity_triggers(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(r#"
        CREATE TRIGGER IF NOT EXISTS evidence_integrity AFTER UPDATE ON evidence
        WHEN new.hash_sha256 IS NOT NULL AND old.hash_sha256 IS NULL BEGIN
            INSERT INTO integrity_logs (id, caseId, checkedAt, checkedBy, totalEvidence, verifiedEvidence, brokenHash, message)
            VALUES (
                'IL-' || strftime('%Y', 'now') || '-' || printf('%06d', (
                    SELECT COALESCE(MAX(id), 0) + 1 FROM integrity_logs
                )),
                new.caseId,
                datetime('now'),
                'system',
                1,
                1,
                0,
                'Evidence hash verified'
            );
        END;
    "#)?;

    Ok(())
}

// =============================================================================
// TAGS TABLES
// =============================================================================

fn create_tags_tables(conn: &Connection) -> AppResult<()> {
    // Le `?` manquait : l'échec de création de la table était silencieusement
    // ignoré (`audit.md`, P3-9).
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            nom TEXT UNIQUE NOT NULL,
            description TEXT,
            color TEXT DEFAULT 'blue',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
    "#,
    )?;

    Ok(())
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

pub fn sha256_hash(data: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn generate_case_reference(conn: &Connection) -> AppResult<String> {
    let year = Utc::now().year();
    let count: i64 = conn.query_row(
        "SELECT COALESCE(MAX(CAST(SUBSTR(reference, -4) AS INTEGER)), 0) FROM cases WHERE reference LIKE ?",
        params![format!("ENQ-{}-%", year)],
        |row| row.get(0),
    )?;

    Ok(format!("ENQ-{}-{:04}", year, count + 1))
}

pub fn generate_report_reference(conn: &Connection) -> AppResult<String> {
    let year = Utc::now().year();
    let count: i64 = conn.query_row(
        "SELECT COALESCE(MAX(CAST(SUBSTR(reference, -4) AS INTEGER)), 0) FROM reports WHERE reference LIKE ?",
        params![format!("RPT-{}-%", year)],
        |row| row.get(0),
    )?;

    Ok(format!("RPT-{}-{:04}", year, count + 1))
}

pub fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
