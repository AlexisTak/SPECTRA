//! Commandes IA — pont entre Tauri et spectra-ai.
//!
//! Toutes les commandes se dégradent proprement à « indisponible » si aucun
//! backend IA n'est détecté (Ollama absent, etc.).

use crate::database::AppState;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::command;

// =============================================================================
// STATUS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    pub available: bool,
    pub backend_name: String,
    pub model: Option<String>,
}

#[command]
pub async fn ai_status(state: tauri::State<'_, AppState>) -> AppResult<AiStatus> {
    let svc = state.ai_service.read().await;
    Ok(AiStatus {
        available: svc.is_available(),
        backend_name: if svc.is_available() {
            "Ollama".to_string()
        } else {
            "offline".to_string()
        },
        model: if svc.is_available() {
            Some("llama3.1".to_string())
        } else {
            None
        },
    })
}

// =============================================================================
// SUMMARIZE
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummarizeInput {
    pub text: String,
}

#[command]
pub async fn ai_summarize(
    state: tauri::State<'_, AppState>,
    input: SummarizeInput,
) -> AppResult<String> {
    let svc = state.ai_service.read().await;
    match svc.summarize(&input.text).await {
        Ok(summary) => Ok(summary),
        Err(spectra_ai::AiError::Unavailable) => {
            Ok("Fonctionnalité IA indisponible (Ollama non détecté).".to_string())
        }
        Err(e) => Err(AppError::msg(format!("erreur IA : {e}"))),
    }
}

// =============================================================================
// NER (EXTRACT ENTITIES)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiExtractInput {
    pub text: String,
    pub case_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiExtractedEntity {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub value: String,
    pub provenance: String,
    pub confidence: String,
}

#[command]
pub async fn ai_extract_entities(
    state: tauri::State<'_, AppState>,
    input: AiExtractInput,
) -> AppResult<Vec<AiExtractedEntity>> {
    let svc = state.ai_service.read().await;
    match svc.extract_entities(&input.text, &input.case_id).await {
        Ok(obs) => {
            let out = obs
                .into_iter()
                .map(|o| AiExtractedEntity {
                    id: o.id,
                    subject: o.subject,
                    predicate: o.predicate,
                    value: serde_json::to_string(&o.value).unwrap_or_default(),
                    provenance: format!("{:?}", o.provenance),
                    confidence: format!("{:?}", o.confidence),
                })
                .collect();
            Ok(out)
        }
        Err(spectra_ai::AiError::Unavailable) => Ok(Vec::new()),
        Err(e) => Err(AppError::msg(format!("erreur IA : {e}"))),
    }
}

// =============================================================================
// SUGGEST PIVOTS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSuggestInput {
    pub entity_id: String,
    pub entity_kind: String,
    pub display_label: String,
    pub canonical_value: String,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSuggestOutput {
    pub description: String,
    pub confidence: f64,
}

#[command]
pub async fn ai_suggest_pivots(
    state: tauri::State<'_, AppState>,
    input: AiSuggestInput,
) -> AppResult<Vec<AiSuggestOutput>> {
    use spectra_core::{Entity, EntityKind};

    let kind = match input.entity_kind.to_lowercase().as_str() {
        "person" => EntityKind::Person,
        "organization" => EntityKind::Organization,
        "location" => EntityKind::Location,
        "emailaddress" => EntityKind::EmailAddress,
        "phonenumber" => EntityKind::PhoneNumber,
        "username" => EntityKind::Username,
        "domain" => EntityKind::Domain,
        _ => EntityKind::Alias,
    };

    let entity = Entity {
        id: input.entity_id,
        kind,
        canonical_value: input.canonical_value,
        display_label: input.display_label,
        properties: Default::default(),
        created_at: chrono::Utc::now().to_rfc3339(),
        merged_from: vec![],
    };

    let svc = state.ai_service.read().await;
    match svc.suggest_pivots(&entity, &input.context).await {
        Ok(suggestions) => {
            let out = suggestions
                .into_iter()
                .map(|s| AiSuggestOutput {
                    description: s.description,
                    confidence: s.confidence,
                })
                .collect();
            Ok(out)
        }
        Err(spectra_ai::AiError::Unavailable) => Ok(Vec::new()),
        Err(e) => Err(AppError::msg(format!("erreur IA : {e}"))),
    }
}

// =============================================================================
// DETECT DUPLICATES
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiDuplicateInput {
    pub entities: Vec<AiDuplicateEntity>,
    pub threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiDuplicateEntity {
    pub id: String,
    pub kind: String,
    pub display_label: String,
    pub canonical_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiDuplicateResult {
    pub id_a: String,
    pub id_b: String,
    pub similarity: f32,
}

#[command]
pub async fn ai_detect_duplicates(
    state: tauri::State<'_, AppState>,
    input: AiDuplicateInput,
) -> AppResult<Vec<AiDuplicateResult>> {
    use spectra_core::EntityKind;

    let entities: Vec<spectra_core::Entity> = input
        .entities
        .into_iter()
        .map(|e| {
            let kind = match e.kind.to_lowercase().as_str() {
                "person" => EntityKind::Person,
                "organization" => EntityKind::Organization,
                "location" => EntityKind::Location,
                "emailaddress" => EntityKind::EmailAddress,
                "phonenumber" => EntityKind::PhoneNumber,
                "username" => EntityKind::Username,
                "domain" => EntityKind::Domain,
                _ => EntityKind::Alias,
            };
            spectra_core::Entity {
                id: e.id,
                kind,
                canonical_value: e.canonical_value,
                display_label: e.display_label,
                properties: Default::default(),
                created_at: chrono::Utc::now().to_rfc3339(),
                merged_from: vec![],
            }
        })
        .collect();

    let svc = state.ai_service.read().await;
    match svc.detect_duplicates(&entities, input.threshold).await {
        Ok(dups) => {
            let out = dups
                .into_iter()
                .map(|(a, b, sim)| AiDuplicateResult {
                    id_a: a,
                    id_b: b,
                    similarity: sim,
                })
                .collect();
            Ok(out)
        }
        Err(spectra_ai::AiError::Unavailable) => Ok(Vec::new()),
        Err(e) => Err(AppError::msg(format!("erreur IA : {e}"))),
    }
}

// =============================================================================
// RAG QUERY
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRagInput {
    pub question: String,
    pub chunks: Vec<AiRagChunk>,
    pub k: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRagChunk {
    pub id: String,
    pub source_type: String,
    pub source_id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRagOutput {
    pub answer: String,
    pub sources: Vec<AiRagSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRagSource {
    pub source_type: String,
    pub source_id: String,
    pub text: String,
    pub score: f32,
}

#[command]
pub async fn ai_rag_query(
    state: tauri::State<'_, AppState>,
    input: AiRagInput,
) -> AppResult<AiRagOutput> {
    use spectra_ai::rag::{RagChunk, RagIndex};

    let mut index = RagIndex::new();
    let chunks: Vec<RagChunk> = input
        .chunks
        .into_iter()
        .map(|c| RagChunk {
            id: c.id,
            source_type: c.source_type,
            source_id: c.source_id,
            text: c.text,
            embedding: None,
        })
        .collect();

    let svc = state.ai_service.read().await;
    match svc.index_for_rag(&mut index, chunks).await {
        Ok(()) => {}
        Err(spectra_ai::AiError::Unavailable) => {
            return Ok(AiRagOutput {
                answer: "IA indisponible — impossible d'indexer les documents.".to_string(),
                sources: vec![],
            });
        }
        Err(e) => return Err(AppError::msg(format!("erreur IA : {e}"))),
    }

    let k = input.k.unwrap_or(3);
    match svc.rag_query(&index, &input.question, k).await {
        Ok(res) => Ok(AiRagOutput {
            answer: res.answer,
            sources: res
                .sources
                .into_iter()
                .map(|s| AiRagSource {
                    source_type: s.source_type,
                    source_id: s.source_id,
                    text: s.text,
                    score: s.score,
                })
                .collect(),
        }),
        Err(spectra_ai::AiError::Unavailable) => Ok(AiRagOutput {
            answer: "IA indisponible — impossible de répondre à la question.".to_string(),
            sources: vec![],
        }),
        Err(e) => Err(AppError::msg(format!("erreur IA : {e}"))),
    }
}
