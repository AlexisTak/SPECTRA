//! IA locale pour SPECTRA — optionnelle, jamais bloquante.
//!
//! # Rôle
//!
//! Fournit des capacités d'IA (NER, résumé, suggestions, embeddings) via des
//! backends locaux (Ollama). Toute sortie est marquée `INFERRED` et
//! nécessite une validation humaine avant d'être intégrée au graphe.
//!
//! # Invariants
//!
//! 1. Aucun appel réseau sortant hors de l'hôte local (Ollama).
//! 2. Toute fonction se dégrade gracieusement à « indisponible » si le
//!    backend n'est pas accessible.
//! 3. Les embeddings sont calculés en local ; aucune donnée d'enquête ne
//!    quitte la machine.

pub mod backend;
pub mod ollama;
pub mod rag;
pub mod service;

pub use backend::{AiBackend, EmbeddingBatch};
pub use ollama::OllamaBackend;
pub use rag::{RagChunk, RagIndex, RagResult, RagSource};
pub use service::{AiService, AiSuggestion};

/// Erreur du moteur IA.
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    /// Le backend n'est pas disponible.
    #[error("backend IA indisponible")]
    Unavailable,
    /// Erreur réseau.
    #[error("erreur HTTP : {0}")]
    Http(#[from] reqwest::Error),
    /// Réponse inattendue.
    #[error("réponse inattendue du backend : {0}")]
    Unexpected(String),
    /// Erreur JSON.
    #[error("erreur JSON : {0}")]
    Json(#[from] serde_json::Error),
    /// Aucun résultat.
    #[error("aucun résultat")]
    Empty,
}
