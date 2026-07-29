//! Type d'erreur des commandes Tauri.
//!
//! Tauri exige que le type d'erreur d'une commande implémente `Serialize` :
//! `anyhow::Error` ne le fait pas, ce qui rendait les 43 commandes de
//! l'application non compilables (voir `audit.md`, P0-1a).
//!
//! Conformément à CLAUDE.md §8, les crates bibliothèque utilisent `thiserror`
//! et non `anyhow`.

use serde::{Serialize, Serializer};

/// Erreur remontée au frontend par une commande Tauri.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Échec d'une opération SQLite.
    #[error("erreur base de données : {0}")]
    Database(#[from] rusqlite::Error),

    /// Échec de (dé)sérialisation JSON.
    #[error("erreur de sérialisation : {0}")]
    Serialization(#[from] serde_json::Error),

    /// Erreur métier : entité introuvable, entrée invalide, invariant violé.
    #[error("{0}")]
    Internal(String),
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl AppError {
    /// Construit une erreur métier à partir d'un message.
    pub fn msg(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

/// Sérialisée en simple chaîne : le frontend reçoit un message, jamais la
/// structure interne de l'erreur.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

/// Alias de retour des commandes et des fonctions internes.
pub type AppResult<T> = std::result::Result<T, AppError>;
