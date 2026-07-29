//! Trait abstrait du backend IA.

use async_trait::async_trait;

/// Résultat d'un batch d'embeddings.
#[derive(Debug, Clone)]
pub struct EmbeddingBatch {
    /// Vecteurs de embeddings, dans le même ordre que la requête.
    pub vectors: Vec<Vec<f32>>,
}

/// Capacités d'un backend IA local.
#[async_trait]
pub trait AiBackend: Send + Sync {
    /// Vérifie si le backend est joignable.
    async fn is_available(&self) -> bool;

    /// Génère un texte à partir d'un prompt.
    async fn complete(&self, prompt: &str) -> Result<String, crate::AiError>;

    /// Calcule les embeddings d'une liste de textes.
    async fn embed(&self, texts: &[&str]) -> Result<EmbeddingBatch, crate::AiError>;
}
