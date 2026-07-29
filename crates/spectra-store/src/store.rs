//! Gestionnaire de connexion et migrations.

use rusqlite::Connection;
use std::path::Path;
use thiserror::Error;

/// Erreur du store.
#[derive(Debug, Error)]
pub enum StoreError {
    /// Erreur SQLite sous-jacente.
    #[error("erreur SQLite : {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// Erreur de sérialisation/désérialisation JSON.
    #[error("erreur JSON : {0}")]
    Json(#[from] serde_json::Error),
}

/// Store SQLite unique par dossier d'enquête.
pub struct Store {
    pub(crate) conn: Connection,
}

impl Store {
    /// Ouvre un fichier `.spectra` (ou tout chemin SQLite) et exécute les
    /// migrations si nécessaire.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Ouvre une base en mémoire (utile pour les tests).
    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), StoreError> {
        self.conn.execute_batch(SCHEMA)?;
        Ok(())
    }
}

const SCHEMA: &str = include_str!("schema.sql");
