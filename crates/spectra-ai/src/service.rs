//! Service IA de haut niveau — dégradation gracieuse garantie.

use crate::backend::AiBackend;
use spectra_core::{Entity, EntityKind, Observation, Provenance};
use std::sync::Arc;

/// Suggestion de pivot issue de l'IA.
#[derive(Debug, Clone)]
pub struct AiSuggestion {
    /// Description lisible de la suggestion.
    pub description: String,
    /// Type d'entité cible suggéré.
    pub suggested_kind: EntityKind,
    /// Score de confiance brut (0–1).
    pub confidence: f64,
}

/// Service IA encapsulant un backend optionnel.
#[derive(Clone)]
pub struct AiService {
    backend: Option<Arc<dyn AiBackend>>,
}

impl std::fmt::Debug for AiService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiService")
            .field("available", &self.backend.is_some())
            .finish()
    }
}

impl AiService {
    /// Crée un service sans backend (mode dégradé).
    pub fn offline() -> Self {
        Self { backend: None }
    }

    /// Crée un service avec un backend.
    pub fn with_backend(backend: Arc<dyn AiBackend>) -> Self {
        Self {
            backend: Some(backend),
        }
    }

    /// Détecte automatiquement le meilleur backend disponible.
    pub async fn new_auto_detect() -> Self {
        if let Some(backend) = crate::ollama::OllamaBackend::detect().await {
            tracing::info!("backend Ollama détecté");
            return Self::with_backend(Arc::new(backend));
        }
        tracing::warn!("aucun backend IA détecté — fonctionnalités IA indisponibles");
        Self::offline()
    }

    fn require_backend(&self) -> Result<&Arc<dyn AiBackend>, crate::AiError> {
        self.backend.as_ref().ok_or(crate::AiError::Unavailable)
    }

    /// Vrai si un backend est actuellement connecté.
    pub fn is_available(&self) -> bool {
        self.backend.is_some()
    }

    /// Calcule les embeddings d'une liste de textes.
    pub async fn embed(&self, texts: &[&str]) -> Result<crate::backend::EmbeddingBatch, crate::AiError> {
        let backend = self.require_backend()?;
        backend.embed(texts).await
    }

    /// Génère un texte à partir d'un prompt brut.
    pub async fn complete(&self, prompt: &str) -> Result<String, crate::AiError> {
        let backend = self.require_backend()?;
        backend.complete(prompt).await
    }

    /// Résumé d'un texte libre.
    pub async fn summarize(&self,
        text: &str,
    ) -> Result<String, crate::AiError> {
        let backend = self.require_backend()?;
        let prompt = format!(
            "Résume le texte suivant en français de manière concise (2-3 phrases maximum) :\n\n{}",
            text
        );
        backend.complete(&prompt).await
    }

    /// Extraction d'entités (NER) depuis un texte.
    ///
    /// Retourne une liste d'observations marquées `INFERRED`.
    pub async fn extract_entities(
        &self,
        text: &str,
        case_id: &str,
    ) -> Result<Vec<Observation>, crate::AiError> {
        let backend = self.require_backend()?;
        let prompt = format!(
            "Extrait les entités du texte suivant. Réponds UNIQUEMENT sous forme de JSON array d'objets avec les champs 'type' (person, organization, location, email, phone, username, domain) et 'value'. Texte :\n\n{}",
            text
        );
        let raw = backend.complete(&prompt).await?;
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&raw)
            .or_else(|_| {
                // fallback : tente d'extraire un bloc JSON
                let start = raw.find('[').unwrap_or(0);
                let end = raw.rfind(']').map(|i| i + 1).unwrap_or(raw.len());
                serde_json::from_str(&raw[start..end])
            })
            .unwrap_or_default();

        let mut observations = Vec::new();
        for item in parsed {
            let kind_str = item.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
            let value = item.get("value").and_then(|v| v.as_str()).unwrap_or("");
            if value.is_empty() {
                continue;
            }
            let _kind = parse_kind(kind_str);
            observations.push(Observation {
                id: format!("ai-ner-{}", uuid::Uuid::new_v4()),
                subject: format!("ai-extracted-{}", case_id),
                predicate: "extracted_entity".to_string(),
                value: spectra_core::PropertyValue::String(value.to_string()),
                source: "ai:ner".to_string(),
                method: "llm_extraction".to_string(),
                observed_at: chrono::Utc::now().to_rfc3339(),
                valid_from: None,
                valid_to: None,
                confidence: spectra_core::AdmiraltyCode::from_str("F6").unwrap_or(spectra_core::AdmiraltyCode {
                    source_reliability: spectra_core::SourceReliability::F,
                    information_credibility: spectra_core::InformationCredibility::V6,
                }),
                provenance: Provenance::Inferred,
                raw_hash: None,
                operator: "ai-service".to_string(),
            });
        }
        Ok(observations)
    }

    /// Suggestions de pivots pour une entité donnée.
    pub async fn suggest_pivots(
        &self,
        entity: &Entity,
        context: &str,
    ) -> Result<Vec<AiSuggestion>, crate::AiError> {
        let backend = self.require_backend()?;
        let prompt = format!(
            "Pour l'entité '{}' (type {:?}) dans le contexte suivant, suggère 3 pistes d'investigation OSINT concrètes. Réponds par une liste numérotée de 3 éléments courts (10 mots max chacun).\n\nContexte : {}\n",
            entity.display_label, entity.kind, context
        );
        let raw = backend.complete(&prompt).await?;
        let suggestions: Vec<AiSuggestion> = raw
            .lines()
            .filter(|l| !l.trim().is_empty())
            .take(3)
            .map(|line| {
                let cleaned = line.trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ')' || c == '-')
                    .trim()
                    .to_string();
                AiSuggestion {
                    description: cleaned,
                    suggested_kind: entity.kind.clone(),
                    confidence: 0.5,
                }
            })
            .collect();
        Ok(suggestions)
    }

    /// Détection de doublons sémantiques par similarité cosinus.
    ///
    /// Retourne les paires d'entités dont la similarité dépasse le seuil.
    pub async fn detect_duplicates(
        &self,
        entities: &[Entity],
        threshold: f32,
    ) -> Result<Vec<(EntityId, EntityId, f32)>, crate::AiError> {
        let backend = self.require_backend()?;
        let texts: Vec<String> = entities
            .iter()
            .map(|e| format!("{} {} {:?}", e.display_label, e.canonical_value, e.kind))
            .collect();
        let slices: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        let batch = backend.embed(&slices).await?;

        let mut duplicates = Vec::new();
        for i in 0..batch.vectors.len() {
            for j in (i + 1)..batch.vectors.len() {
                let sim = cosine_similarity(&batch.vectors[i], &batch.vectors[j]);
                if sim >= threshold {
                    duplicates.push((entities[i].id.clone(), entities[j].id.clone(), sim));
                }
            }
        }
        Ok(duplicates)
    }
}

fn parse_kind(s: &str) -> EntityKind {
    match s.to_lowercase().as_str() {
        "person" => EntityKind::Person,
        "organization" => EntityKind::Organization,
        "location" => EntityKind::Location,
        "email" => EntityKind::EmailAddress,
        "phone" => EntityKind::PhoneNumber,
        "username" => EntityKind::Username,
        "domain" => EntityKind::Domain,
        _ => EntityKind::Alias,
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

type EntityId = String;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{AiBackend, EmbeddingBatch};
    use async_trait::async_trait;

    /// Backend factice pour les tests unitaires.
    #[derive(Debug, Clone)]
    struct MockBackend;

    #[async_trait]
    impl AiBackend for MockBackend {
        async fn is_available(&self) -> bool {
            true
        }

        async fn complete(&self, prompt: &str) -> Result<String, crate::AiError> {
            if prompt.contains("JSON array") {
                Ok(r#"[{"type":"person","value":"Alice"},{"type":"email","value":"alice@test.com"}]"#.to_string())
            } else if prompt.contains("pistes d'investigation") {
                Ok("1. Vérifier les réseaux sociaux\n2. Consulter les registres publics\n3. Analyser les fuites de données".to_string())
            } else {
                Ok("Résumé fictif".to_string())
            }
        }

        async fn embed(&self, _texts: &[&str]) -> Result<EmbeddingBatch, crate::AiError> {
            let n = _texts.len();
            let vectors = (0..n).map(|_| vec![0.1_f32; 384]).collect();
            Ok(EmbeddingBatch { vectors })
        }
    }

    #[tokio::test]
    async fn offline_service_returns_unavailable() {
        let svc = AiService::offline();
        assert!(!svc.is_available());
        assert!(matches!(svc.summarize("test").await, Err(crate::AiError::Unavailable)));
        assert!(matches!(svc.extract_entities("test", "case-1").await, Err(crate::AiError::Unavailable)));
    }

    #[tokio::test]
    async fn summarize_with_mock_backend() {
        let svc = AiService::with_backend(Arc::new(MockBackend));
        let res = svc.summarize("bla bla").await.unwrap();
        assert_eq!(res, "Résumé fictif");
    }

    #[tokio::test]
    async fn extract_entities_parses_json() {
        let svc = AiService::with_backend(Arc::new(MockBackend));
        let obs = svc.extract_entities("Alice est une personne.", "case-1").await.unwrap();
        assert_eq!(obs.len(), 2);
        assert_eq!(obs[0].provenance, Provenance::Inferred);
        assert_eq!(obs[0].source, "ai:ner");
    }

    #[tokio::test]
    async fn suggest_pivots_returns_three() {
        let entity = Entity {
            id: "e1".to_string(),
            kind: EntityKind::Person,
            canonical_value: "Alice".to_string(),
            display_label: "Alice".to_string(),
            properties: Default::default(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            merged_from: vec![],
        };
        let svc = AiService::with_backend(Arc::new(MockBackend));
        let suggestions = svc.suggest_pivots(&entity, "contexte").await.unwrap();
        assert_eq!(suggestions.len(), 3);
        assert_eq!(suggestions[0].suggested_kind, EntityKind::Person);
    }

    #[tokio::test]
    async fn detect_duplicates_finds_similar() {
        let entities = vec![
            Entity {
                id: "e1".to_string(),
                kind: EntityKind::Person,
                canonical_value: "Alice".to_string(),
                display_label: "Alice".to_string(),
                properties: Default::default(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                merged_from: vec![],
            },
            Entity {
                id: "e2".to_string(),
                kind: EntityKind::Person,
                canonical_value: "Alice".to_string(),
                display_label: "Alice Dupont".to_string(),
                properties: Default::default(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                merged_from: vec![],
            },
        ];
        let svc = AiService::with_backend(Arc::new(MockBackend));
        let dups = svc.detect_duplicates(&entities, 0.99).await.unwrap();
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].0, "e1");
        assert_eq!(dups[0].1, "e2");
    }
}
