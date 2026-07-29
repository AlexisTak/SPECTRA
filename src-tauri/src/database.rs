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
    /// Racine du magasin de preuves, sous le répertoire de données applicatives.
    storage_root: std::path::PathBuf,
    /// Service IA (Ollama ou offline).
    pub ai_service: tokio::sync::RwLock<spectra_ai::AiService>,
}

impl AppState {
    /// Construit l'état à partir d'une connexion déjà initialisée.
    pub fn new(conn: Connection, storage_root: std::path::PathBuf, ai_service: spectra_ai::AiService) -> Self {
        Self {
            conn: Mutex::new(conn),
            storage_root,
            ai_service: tokio::sync::RwLock::new(ai_service),
        }
    }

    /// Prend le verrou sur la connexion.
    pub async fn get_conn(&self) -> tokio::sync::MutexGuard<'_, Connection> {
        self.conn.lock().await
    }

    /// Racine du magasin de preuves.
    pub fn storage_root(&self) -> &std::path::Path {
        &self.storage_root
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

/// Journal d'audit hash-chaîné.
///
/// `sequence` est **persisté** et non reconstruit : il entre dans le calcul du
/// hachage, donc le recalcul à la vérification doit retrouver exactement la
/// valeur utilisée à l'écriture. La version d'origine le dérivait des chiffres
/// de l'UUID, ce qui rendait toute vérification impossible.
///
/// Il fournit aussi l'ordre total que l'horodatage ne garantit pas : deux
/// événements de la même seconde sont indiscernables par `timestamp`
/// (`audit.md`, P2-14).
const AUDIT_EVENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS audit_events (
    id TEXT PRIMARY KEY,
    caseId TEXT NOT NULL,
    sequence INTEGER NOT NULL DEFAULT 1,
    action TEXT NOT NULL,
    entityKind TEXT NOT NULL,
    entityId TEXT,
    actor TEXT,
    metadata TEXT,
    imma TEXT NOT NULL,
    imma_precedent TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (caseId) REFERENCES cases(id) ON DELETE CASCADE,
    UNIQUE (caseId, sequence)
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

/// Crée les index de recherche plein texte.
///
/// Les tables sont **autonomes** (pas de `content=`), avec l'identifiant métier
/// stocké dans une colonne `UNINDEXED`.
///
/// Le schéma d'origine déclarait `content = 'cases', content_rowid = 'id'`,
/// or FTS5 exige que `content_rowid` désigne un **entier**. Les identifiants
/// étant des UUID textuels, toute insertion échouait sur `datatype mismatch` —
/// donc toute création de dossier (`audit.md`, P2-2).
///
/// Les anciennes tables sont supprimées : leur schéma est irréparable, et les
/// bases déjà créées resteraient inutilisables.
fn create_fts_tables(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        DROP TABLE IF EXISTS fts_cases;
        DROP TABLE IF EXISTS fts_evidence;
        DROP TABLE IF EXISTS fts_tiktok_videos;
        DROP TABLE IF EXISTS fts_subjects;
        DROP TABLE IF EXISTS fts_reports;
        DROP TABLE IF EXISTS fts_web_archives;
        DROP TABLE IF EXISTS fts_notes;

        CREATE VIRTUAL TABLE IF NOT EXISTS fts_cases USING fts5(
            entity_id UNINDEXED, reference, titre, description
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_evidence USING fts5(
            entity_id UNINDEXED, nom, description, chemin
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_tiktok_videos USING fts5(
            entity_id UNINDEXED, videoId, auteur, description
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_subjects USING fts5(
            entity_id UNINDEXED, nom, prenom, description, email, telephone
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_reports USING fts5(
            entity_id UNINDEXED, reference, titre, description
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_web_archives USING fts5(
            entity_id UNINDEXED, url, title, content
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_notes USING fts5(
            entity_id UNINDEXED, titre, contenu
        );
    "#,
    )?;

    Ok(())
}

/// Maintient les index de recherche synchronisés avec les tables métier.
///
/// Les déclencheurs d'origine inséraient dans `rowid` (entier) un UUID textuel
/// et ne couvraient que 3 entités sur 7 : `subjects`, `reports`,
/// `web_archives` et `notes` n'étaient jamais désindexés à la suppression.
/// Des données personnelles de sujets supprimés restaient donc indéfiniment
/// dans l'index — un défaut d'effacement au sens du RGPD (`audit.md`, P1-9,
/// P2-10).
///
/// Les déclencheurs couvrent désormais insertion, mise à jour et suppression
/// pour les sept entités. Les commandes n'ont plus à indexer manuellement :
/// la double indexation était un autre défaut relevé (P2-9).
fn create_fts_triggers(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        DROP TRIGGER IF EXISTS cases_ai;
        DROP TRIGGER IF EXISTS cases_ad;
        DROP TRIGGER IF EXISTS cases_au;
        DROP TRIGGER IF EXISTS evidence_ai;
        DROP TRIGGER IF EXISTS evidence_ad;
        DROP TRIGGER IF EXISTS evidence_au;
        DROP TRIGGER IF EXISTS tiktok_videos_ai;
        DROP TRIGGER IF EXISTS tiktok_videos_ad;
        DROP TRIGGER IF EXISTS tiktok_videos_au;

        CREATE TRIGGER IF NOT EXISTS cases_ai AFTER INSERT ON cases BEGIN
            INSERT INTO fts_cases(entity_id, reference, titre, description)
            VALUES(new.id, new.reference, new.titre, new.description);
        END;
        CREATE TRIGGER IF NOT EXISTS cases_ad AFTER DELETE ON cases BEGIN
            DELETE FROM fts_cases WHERE entity_id = old.id;
        END;
        CREATE TRIGGER IF NOT EXISTS cases_au AFTER UPDATE ON cases BEGIN
            DELETE FROM fts_cases WHERE entity_id = old.id;
            INSERT INTO fts_cases(entity_id, reference, titre, description)
            VALUES(new.id, new.reference, new.titre, new.description);
        END;

        CREATE TRIGGER IF NOT EXISTS evidence_ai AFTER INSERT ON evidence BEGIN
            INSERT INTO fts_evidence(entity_id, nom, description, chemin)
            VALUES(new.id, new.nom, new.description, new.chemin);
        END;
        CREATE TRIGGER IF NOT EXISTS evidence_ad AFTER DELETE ON evidence BEGIN
            DELETE FROM fts_evidence WHERE entity_id = old.id;
        END;
        CREATE TRIGGER IF NOT EXISTS evidence_au AFTER UPDATE ON evidence BEGIN
            DELETE FROM fts_evidence WHERE entity_id = old.id;
            INSERT INTO fts_evidence(entity_id, nom, description, chemin)
            VALUES(new.id, new.nom, new.description, new.chemin);
        END;

        CREATE TRIGGER IF NOT EXISTS subjects_ai AFTER INSERT ON subjects BEGIN
            INSERT INTO fts_subjects(entity_id, nom, prenom, description, email, telephone)
            VALUES(new.id, new.nom, new.prenom, new.description, new.email, new.telephone);
        END;
        CREATE TRIGGER IF NOT EXISTS subjects_ad AFTER DELETE ON subjects BEGIN
            DELETE FROM fts_subjects WHERE entity_id = old.id;
        END;
        CREATE TRIGGER IF NOT EXISTS subjects_au AFTER UPDATE ON subjects BEGIN
            DELETE FROM fts_subjects WHERE entity_id = old.id;
            INSERT INTO fts_subjects(entity_id, nom, prenom, description, email, telephone)
            VALUES(new.id, new.nom, new.prenom, new.description, new.email, new.telephone);
        END;

        CREATE TRIGGER IF NOT EXISTS reports_ai AFTER INSERT ON reports BEGIN
            INSERT INTO fts_reports(entity_id, reference, titre, description)
            VALUES(new.id, new.reference, new.titre, new.description);
        END;
        CREATE TRIGGER IF NOT EXISTS reports_ad AFTER DELETE ON reports BEGIN
            DELETE FROM fts_reports WHERE entity_id = old.id;
        END;
        CREATE TRIGGER IF NOT EXISTS reports_au AFTER UPDATE ON reports BEGIN
            DELETE FROM fts_reports WHERE entity_id = old.id;
            INSERT INTO fts_reports(entity_id, reference, titre, description)
            VALUES(new.id, new.reference, new.titre, new.description);
        END;

        CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
            INSERT INTO fts_notes(entity_id, titre, contenu)
            VALUES(new.id, new.titre, new.contenu);
        END;
        CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
            DELETE FROM fts_notes WHERE entity_id = old.id;
        END;
        CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
            DELETE FROM fts_notes WHERE entity_id = old.id;
            INSERT INTO fts_notes(entity_id, titre, contenu)
            VALUES(new.id, new.titre, new.contenu);
        END;

        CREATE TRIGGER IF NOT EXISTS web_archives_ai AFTER INSERT ON web_archives BEGIN
            INSERT INTO fts_web_archives(entity_id, url, title, content)
            VALUES(new.id, new.url, new.title, new.content);
        END;
        CREATE TRIGGER IF NOT EXISTS web_archives_ad AFTER DELETE ON web_archives BEGIN
            DELETE FROM fts_web_archives WHERE entity_id = old.id;
        END;

        CREATE TRIGGER IF NOT EXISTS tiktok_videos_ai AFTER INSERT ON tiktok_videos BEGIN
            INSERT INTO fts_tiktok_videos(entity_id, videoId, auteur, description)
            VALUES(new.id, new.videoId, new.auteur, new.description);
        END;
        CREATE TRIGGER IF NOT EXISTS tiktok_videos_ad AFTER DELETE ON tiktok_videos BEGIN
            DELETE FROM fts_tiktok_videos WHERE entity_id = old.id;
        END;
    "#,
    )?;

    Ok(())
}

// =============================================================================
// AUDIT TRAIL TRIGGERS
// =============================================================================

/// Supprime les déclencheurs d'audit SQL hérités.
///
/// Ces déclencheurs appelaient `sha256()`, **qui n'existe pas dans SQLite** :
/// toute création ou modification de dossier échouait sur
/// `no such function: sha256` (`audit.md`, P1-5). Le défaut était invisible en
/// lecture de code et n'est apparu qu'en soumettant le formulaire.
///
/// Ils ne sont pas réparés mais **retirés**, pour deux raisons :
///
/// 1. Le chaînage est désormais calculé en Rust (`spectra_audit::compute_link`)
///    dans chacune des 18 commandes de mutation. Deux implémentations
///    concurrentes du même hachage produiraient des maillons incohérents.
/// 2. Leur génération d'identifiant (`MAX(id) + 1` sur des identifiants
///    **textuels**) renvoyait toujours `1`, donc le deuxième événement de
///    l'année violait la clé primaire.
///
/// Le `DROP` est nécessaire et non seulement le retrait du `CREATE` : les bases
/// déjà créées portent les déclencheurs et resteraient inutilisables.
fn create_audit_triggers(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        DROP TRIGGER IF EXISTS audit_insert;
        DROP TRIGGER IF EXISTS audit_update;
    "#,
    )?;

    Ok(())
}

// =============================================================================
// INTEGRITY TRIGGERS
// =============================================================================

/// Supprime le déclencheur d'intégrité hérité.
///
/// Même défaut de génération d'identifiant que les déclencheurs d'audit :
/// `MAX(id) + 1` appliqué à des identifiants **textuels** (`IL-2026-000001`)
/// renvoie toujours `1`. La deuxième preuve vérifiée de l'année violait donc la
/// clé primaire, faisant échouer la mise à jour métier elle-même.
///
/// Par ailleurs il journalisait « Evidence hash verified » sur simple présence
/// d'une empreinte, sans rien vérifier — exactement le faux positif rassurant
/// que l'audit reproche à cette base de code (`audit.md`, P1-1).
///
/// La journalisation d'intégrité doit être faite en Rust, après relecture
/// effective du fichier et recalcul de son empreinte.
fn create_integrity_triggers(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        DROP TRIGGER IF EXISTS evidence_integrity;
    "#,
    )?;

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
    sha256_hash_bytes(data.as_bytes())
}

/// Empreinte SHA-256 d'un contenu binaire.
///
/// Utilisée à l'ingestion d'une preuve et à chaque vérification : c'est la même
/// fonction des deux côtés, sinon la comparaison n'aurait pas de sens.
pub fn sha256_hash_bytes(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
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

// =============================================================================
// JOURNAL D'AUDIT
// =============================================================================

/// Écrit un maillon de la chaîne d'audit d'un dossier.
///
/// Point de passage **unique** pour toute écriture dans `audit_events`. Chaque
/// commande de mutation répétait auparavant cette logique, avec des variantes
/// subtiles — d'où des maillons incohérents détectés seulement à la
/// vérification.
///
/// Garanties :
///
/// 1. `previous_hash` est lu depuis le dernier maillon **du même dossier**,
///    ordonné par `sequence` (ordre total, contrairement à l'horodatage).
/// 2. `sequence` est calculé ici et **persisté**, car il entre dans le hachage.
/// 3. Le hachage est calculé par `spectra_audit`, jamais fourni par l'appelant.
///
/// L'appelant doit déjà détenir le verrou sur la connexion.
pub fn append_audit_event(
    conn: &Connection,
    case_id: &str,
    action: &str,
    entity_kind: &str,
    entity_id: Option<&str>,
    actor: &str,
    payload: serde_json::Value,
) -> AppResult<()> {
    let (previous_hash, previous_sequence): (String, i64) = conn
        .query_row(
            "SELECT imma, sequence FROM audit_events WHERE caseId = ? \
             ORDER BY sequence DESC LIMIT 1",
            params![case_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap_or_else(|_| (String::new(), 0));

    let sequence = previous_sequence + 1;

    let event = spectra_audit::AuditEvent {
        id: generate_uuid(),
        case_id: case_id.to_string(),
        action: action.to_string(),
        entity_kind: entity_kind.to_string(),
        entity_id: entity_id.map(str::to_string),
        actor: actor.to_string(),
        sequence: sequence as u64,
        payload,
    };

    let link = spectra_audit::compute_link(&event, &previous_hash);

    conn.execute(
        "INSERT INTO audit_events \
         (id, caseId, sequence, action, entityKind, entityId, actor, metadata, imma, imma_precedent) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            &event.id,
            case_id,
            sequence,
            &event.action,
            &event.entity_kind,
            &event.entity_id,
            &event.actor,
            &serde_json::to_string(&event.payload)?,
            &link.hash,
            &link.previous_hash,
        ],
    )?;

    Ok(())
}
