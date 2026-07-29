//! RAG (Retrieval-Augmented Generation) local sur les notes et pièces du dossier.
//!
//! Architecture :
//! 1. Chunking des notes / observations en segments de texte.
//! 2. Embedding de chaque chunk via le backend IA (Ollama par défaut).
//! 3. Stockage des vecteurs en mémoire pour la démo ; sqlite-vec pour la prod.
//! 4. Requête : embedding de la question + recherche des k plus proches voisins.
//! 5. Prompt augmenté avec le contexte récupéré, envoyé au LLM.

use crate::backend::EmbeddingBatch;
use crate::service::AiService;
use serde::{Deserialize, Serialize};

/// Segment de texte indexé pour le RAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagChunk {
    pub id: String,
    pub source_type: String, // "note", "observation", "entity", etc.
    pub source_id: String,
    pub text: String,
    pub embedding: Option<Vec<f32>>,
}

/// Index in-mémoire de chunks pour le RAG.
#[derive(Debug, Clone, Default)]
pub struct RagIndex {
    chunks: Vec<RagChunk>,
}

impl RagIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute des chunks sans embeddings (à encoder ensuite).
    pub fn add_chunks(&mut self, chunks: Vec<RagChunk>) {
        self.chunks.extend(chunks);
    }

    /// Encode tous les chunks manquants via le backend IA.
    pub async fn encode(
        &mut self, svc: &AiService) -> Result<(), crate::AiError> {
        let pending: Vec<usize> = self
            .chunks
            .iter()
            .enumerate()
            .filter(|(_, c)| c.embedding.is_none())
            .map(|(i, _)| i)
            .collect();

        if pending.is_empty() {
            return Ok(());
        }

        let texts: Vec<&str> = pending
            .iter()
            .map(|&i| self.chunks[i].text.as_str())
            .collect();

        let batch = svc.embed(&texts).await?;

        for (idx, vec) in pending.into_iter().zip(batch.vectors.into_iter()) {
            self.chunks[idx].embedding = Some(vec);
        }
        Ok(())
    }

    /// Recherche les k chunks les plus similaires à la requête.
    pub fn search(&self, query_vec: &[f32], k: usize) -> Vec<(&RagChunk, f32)> {
        let mut scored: Vec<(&RagChunk, f32)> = self
            .chunks
            .iter()
            .filter_map(|c| {
                let emb = c.embedding.as_ref()?;
                Some((c, cosine_similarity(query_vec, emb)))
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    /// Vide l'index.
    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

/// Résultat d'une requête RAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagResult {
    pub answer: String,
    pub sources: Vec<RagSource>,
}

/// Source ayant contribué au contexte RAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagSource {
    pub source_type: String,
    pub source_id: String,
    pub text: String,
    pub score: f32,
}

/// Service RAG de haut niveau.
impl AiService {
    /// Indexe les textes fournis (notes, observations…) pour le RAG.
    pub async fn index_for_rag(
        &self,
        index: &mut RagIndex,
        texts: Vec<RagChunk>,
    ) -> Result<(), crate::AiError> {
        index.add_chunks(texts);
        index.encode(self).await
    }

    /// Pose une question au dossier via RAG.
    pub async fn rag_query(
        &self,
        index: &RagIndex,
        question: &str,
        k: usize,
    ) -> Result<RagResult, crate::AiError> {
        if index.is_empty() {
            return Err(crate::AiError::Empty);
        }

        // 1. Embedding de la question
        let query_batch = self.embed(&[question]).await?;
        let query_vec = query_batch
            .vectors
            .into_iter()
            .next()
            .ok_or(crate::AiError::Empty)?;

        // 2. Recherche des k plus proches voisins
        let neighbors = index.search(&query_vec, k);
        if neighbors.is_empty() {
            return Err(crate::AiError::Empty);
        }

        // 3. Construction du contexte
        let mut context = String::from("Contexte du dossier :\n");
        for (chunk, score) in &neighbors {
            context.push_str(&format!(
                "\n--- [{} | {} | score {:.3}] ---\n{}\n",
                chunk.source_type, chunk.source_id, score, chunk.text
            ));
        }

        // 4. Prompt augmenté
        let prompt = format!(
            "{context}\n\n---\n\nQuestion : {question}\n\nRéponds en te basant UNIQUEMENT sur le contexte ci-dessus. Si tu ne trouves pas la réponse, dis-le explicitement."
        );

        let answer = self.complete(&prompt).await?;

        let sources = neighbors
            .into_iter()
            .map(|(chunk, score)| RagSource {
                source_type: chunk.source_type.clone(),
                source_id: chunk.source_id.clone(),
                text: chunk.text.clone(),
                score,
            })
            .collect();

        Ok(RagResult { answer, sources })
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{AiBackend, EmbeddingBatch};
    use async_trait::async_trait;
    use std::sync::Arc;

    #[derive(Debug, Clone)]
    struct MockEmbedBackend;

    #[async_trait]
    impl AiBackend for MockEmbedBackend {
        async fn is_available(&self) -> bool { true }
        async fn complete(&self, _: &str) -> Result<String, crate::AiError> {
            Ok("Réponse fictive".to_string())
        }
        async fn embed(&self, texts: &[&str]) -> Result<EmbeddingBatch, crate::AiError> {
            // Embeddings artificiels : chaque texte a un vecteur distinct
            let vectors: Vec<Vec<f32>> = texts
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let mut v = vec![0.0_f32; 4];
                    v[i % 4] = 1.0;
                    v
                })
                .collect();
            Ok(EmbeddingBatch { vectors })
        }
    }

    #[tokio::test]
    async fn rag_query_finds_answer() {
        let svc = AiService::with_backend(Arc::new(MockEmbedBackend));
        let mut index = RagIndex::new();

        let chunks = vec![
            RagChunk {
                id: "c1".to_string(),
                source_type: "note".to_string(),
                source_id: "note-1".to_string(),
                text: "Alice Dupont habite au 12 rue de Paris.".to_string(),
                embedding: None,
            },
            RagChunk {
                id: "c2".to_string(),
                source_type: "observation".to_string(),
                source_id: "obs-1".to_string(),
                text: "Le suspect a utilisé le pseudonyme dark_knight_42.".to_string(),
                embedding: None,
            },
        ];

        svc.index_for_rag(&mut index, chunks).await.unwrap();
        assert_eq!(index.len(), 2);

        let res = svc.rag_query(&index, "Où habite Alice ?", 2).await.unwrap();
        assert!(!res.answer.is_empty());
        assert_eq!(res.sources.len(), 2);
        // Le premier source devrait être celui qui mentionne Alice
        assert!(res.sources[0].text.contains("Alice"));
    }

    #[tokio::test]
    async fn rag_empty_index_returns_error() {
        let svc = AiService::with_backend(Arc::new(MockEmbedBackend));
        let index = RagIndex::new();
        let res = svc.rag_query(&index, "question", 2).await;
        assert!(matches!(res, Err(crate::AiError::Empty)));
    }
}
