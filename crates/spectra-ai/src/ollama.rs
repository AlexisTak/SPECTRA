//! Backend Ollama via reqwest.
//!
//! Communique avec l'API HTTP locale d'Ollama (`http://localhost:11434`).
//! Aucun appel sortant : tout reste sur la machine.

use crate::backend::{AiBackend, EmbeddingBatch};
use async_trait::async_trait;
use serde_json::json;

/// Backend Ollama.
#[derive(Debug, Clone)]
pub struct OllamaBackend {
    client: reqwest::Client,
    base_url: String,
    chat_model: String,
    embed_model: String,
}

impl OllamaBackend {
    /// Crée un backend Ollama avec les modèles par défaut.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            chat_model: "llama3.1".to_string(),
            embed_model: "nomic-embed-text".to_string(),
        }
    }

    /// Définit le modèle de chat (génération).
    pub fn with_chat_model(mut self, model: impl Into<String>) -> Self {
        self.chat_model = model.into();
        self
    }

    /// Définit le modèle d'embeddings.
    pub fn with_embed_model(mut self, model: impl Into<String>) -> Self {
        self.embed_model = model.into();
        self
    }

    /// Détecte automatiquement Ollama sur `http://localhost:11434`.
    pub async fn detect() -> Option<Self> {
        let backend = Self::new("http://localhost:11434");
        if backend.is_available().await {
            Some(backend)
        } else {
            None
        }
    }

    async fn post_json(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, crate::AiError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client.post(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(crate::AiError::Unexpected(format!(
                "HTTP {} : {}",
                status, text
            )));
        }
        let json = resp.json().await?;
        Ok(json)
    }
}

#[async_trait]
impl AiBackend for OllamaBackend {
    async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        matches!(self.client.get(&url).send().await, Ok(r) if r.status().is_success())
    }

    async fn complete(&self, prompt: &str) -> Result<String, crate::AiError> {
        let body = json!({
            "model": self.chat_model,
            "prompt": prompt,
            "stream": false,
        });
        let resp = self.post_json("/api/generate", body).await?;
        resp["response"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| crate::AiError::Unexpected("champ 'response' manquant".to_string()))
    }

    async fn embed(&self, texts: &[&str]) -> Result<EmbeddingBatch, crate::AiError> {
        let mut vectors = Vec::with_capacity(texts.len());
        for text in texts {
            let body = json!({
                "model": self.embed_model,
                "prompt": text,
            });
            let resp = self.post_json("/api/embeddings", body).await?;
            let vec = resp["embedding"]
                .as_array()
                .ok_or_else(|| crate::AiError::Unexpected("champ 'embedding' manquant".to_string()))?
                .iter()
                .map(|v| v.as_f64().map(|f| f as f32).unwrap_or(0.0))
                .collect::<Vec<f32>>();
            vectors.push(vec);
        }
        Ok(EmbeddingBatch { vectors })
    }
}
