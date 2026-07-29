//! Transform Certificate Transparency — requête vers crt.sh.

use serde::Deserialize;
use spectra_core::{Entity, EntityKind, Observation, PropertyValue};
use spectra_transform::{
    Transform, TransformContext, TransformError, TransformInput, TransformOutput, Relation,
};
use std::future::Future;
use std::pin::Pin;

/// Transform Certificate Transparency : interroge l'API JSON de crt.sh.
pub struct CtTransform;

impl CtTransform {
    pub fn new() -> Self {
        Self
    }
}

/// Réponse de l'API crt.sh.
#[derive(Debug, Deserialize)]
struct CrtShEntry {
    #[serde(rename = "issuer_name")]
    issuer: Option<String>,
    #[serde(rename = "common_name")]
    common_name: Option<String>,
    #[serde(rename = "name_value")]
    name_value: Option<String>,
    #[serde(rename = "entry_timestamp")]
    entry_timestamp: Option<String>,
}

impl Transform for CtTransform {
    fn id(&self) -> &str {
        "ct:search"
    }

    fn display_name(&self) -> &str {
        "Certificate Transparency (crt.sh)"
    }

    fn input_kinds(&self) -> &[spectra_core::EntityKind] {
        &[EntityKind::Domain]
    }

    fn execute(
        &self,
        ctx: TransformContext,
        input: TransformInput,
    ) -> Pin<Box<dyn Future<Output = Result<TransformOutput, TransformError>> + Send + '_>> {
        Box::pin(async move {
            let domain = &input.entity.canonical_value;
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .user_agent(&ctx.user_agent)
                .build()
                .map_err(|e| TransformError::Internal(format!("client: {e}")))?;

            let resp = client
                .get("https://crt.sh/")
                .query(&[("q", domain.as_str()), ("output", "json")])
                .send()
                .await
                .map_err(|e| TransformError::NetworkError(format!("HTTP: {e}")))?;

            let status = resp.status();
            let body = resp
                .text()
                .await
                .map_err(|e| TransformError::NetworkError(format!("read body: {e}")))?;

            if !status.is_success() {
                return Err(TransformError::NetworkError(format!(
                    "crt.sh répond HTTP {}",
                    status.as_u16()
                )));
            }

            let entries: Vec<CrtShEntry> = serde_json::from_str(&body)
                .map_err(|e| TransformError::Internal(format!("parse JSON: {e}")))?;

            let now = chrono::Utc::now().to_rfc3339();
            let mut output = TransformOutput::default();
            let mut seen = std::collections::HashSet::new();

            for entry in entries {
                let cn = entry.common_name.as_deref().unwrap_or("").to_string();
                if cn.is_empty() || seen.contains(&cn) {
                    continue;
                }
                seen.insert(cn.clone());

                let entity = Entity {
                    id: format!("cert-{}", &cn),
                    kind: EntityKind::Domain,
                    canonical_value: cn.clone(),
                    display_label: cn.clone(),
                    properties: Default::default(),
                    created_at: now.clone(),
                    merged_from: vec![],
                };
                output.entities.push(entity);
                output.relations.push(Relation {
                    source: input.entity.id.clone(),
                    target: format!("cert-{}", &cn),
                    kind: "has_certificate".to_string(),
                    properties: Default::default(),
                });

                if let Some(issuer) = entry.issuer {
                    output.observations.push(Observation {
                        id: format!("obs-ct-issuer-{}", &cn),
                        subject: input.entity.id.clone(),
                        predicate: "ct_issuer".to_string(),
                        value: PropertyValue::String(issuer),
                        source: "ct:search".to_string(),
                        method: "HTTP_GET".to_string(),
                        observed_at: now.clone(),
                        valid_from: None,
                        valid_to: None,
                        confidence: spectra_core::AdmiraltyCode::from_str("B2").unwrap_or(spectra_core::AdmiraltyCode {
                            source_reliability: spectra_core::SourceReliability::B,
                            information_credibility: spectra_core::InformationCredibility::V2,
                        }),
                        provenance: spectra_core::Provenance::Collected,
                        raw_hash: None,
                        operator: "SPECTRA".to_string(),
                    });
                }
            }

            Ok(output)
        })
    }
}
