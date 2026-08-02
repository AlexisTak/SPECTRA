//! Transform email — vérification MX, SPF, DMARC et patterns de permutation.
//!
//! # Objectif
//!
//! Valider la livraison possible d'un email et collecter les enregistrements
//! DNS liés (MX, TXT SPF, TXT DMARC). La technique "mot de passe oublié"
//! (modèle Holehe) est laissée pour une extension future car elle nécessite
//! des adaptateurs spécifiques par service web.

use hickory_resolver::{
    proto::rr::{RecordType, RData},
    TokioResolver,
};
use spectra_core::{AdmiraltyCode, EntityKind, Observation, PropertyValue, Provenance};
use spectra_transform::{Transform, TransformContext, TransformError, TransformInput, TransformOutput};
use std::collections::BTreeMap;
use std::pin::Pin;

/// Transform natif Email.
pub struct EmailTransform {
    resolver: TokioResolver,
}

impl EmailTransform {
    /// Construit le transform et son résolveur DNS.
    ///
    /// # Errors
    ///
    /// Retourne une erreur si le résolveur système ne peut pas être construit.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let resolver = TokioResolver::builder_tokio()
            .map_err(|e| format!("resolver builder: {e}"))?
            .build()
            .map_err(|e| format!("resolver build: {e}"))?;
        Ok(Self { resolver })
    }

    /// Parse un email en (local_part, domaine).
    fn parse_email(email: &str) -> Option<(&str, &str)> {
        let at = email.rfind('@')?;
        let local = &email[..at];
        let domain = &email[at + 1..];
        if local.is_empty() || domain.is_empty() {
            return None;
        }
        Some((local, domain))
    }

    /// Résout les enregistrements MX d'un domaine.
    async fn mx_records(
        &self,
        domain: &str,
    ) -> Result<Vec<String>, TransformError> {
        let lookup = self
            .resolver
            .lookup(domain, RecordType::MX)
            .await
            .map_err(|e| TransformError::NetworkError(format!("MX lookup {domain}: {e}")))?;

        let mut records = Vec::new();
        for record in lookup.answers().iter() {
            if let RData::MX(mx) = &record.data {
                records.push(mx.exchange.to_string());
            }
        }
        Ok(records)
    }

    /// Résout les enregistrements TXT d'un domaine.
    async fn txt_records(
        &self,
        domain: &str,
    ) -> Result<Vec<String>, TransformError> {
        let lookup = self
            .resolver
            .lookup(domain, RecordType::TXT)
            .await
            .map_err(|e| TransformError::NetworkError(format!("TXT lookup {domain}: {e}")))?;

        let mut records = Vec::new();
        for record in lookup.answers().iter() {
            if let RData::TXT(txt) = &record.data {
                for s in txt.txt_data.iter() {
                    records.push(String::from_utf8_lossy(s).to_string());
                }
            }
        }
        Ok(records)
    }

    /// Génère des permutations courantes d'emails à partir d'un nom/prénom.
    ///
    /// `domain` doit inclure l'arobase (par exemple `@example.com`).
    #[must_use]
    pub fn generate_permutations(
        first: &str,
        last: &str,
        domain: &str,
    ) -> Vec<String> {
        let f = first.to_lowercase();
        let l = last.to_lowercase();
        let patterns = vec![
            format!("{}.{}{}", f, l, domain),
            format!("{}.{}{}", f, l.chars().next().unwrap_or('x'), domain),
            format!("{}{}{}", f, l, domain),
            format!("{}{}{}", f, l.chars().next().unwrap_or('x'), domain),
            format!("{}_{}{}", f, l, domain),
            format!("{}-{}{}", f, l, domain),
        ];
        patterns.into_iter().collect()
    }
}

impl Transform for EmailTransform {
    fn id(&self) -> &str {
        "collect:email"
    }

    fn display_name(&self) -> &str {
        "Email — MX, SPF, DMARC"
    }

    fn input_kinds(&self) -> &[EntityKind] {
        &[EntityKind::EmailAddress]
    }

    fn execute(
        &self,
        _ctx: TransformContext,
        input: TransformInput,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<TransformOutput, TransformError>> + Send + '_>> {
        Box::pin(async move {
            let email = &input.entity.canonical_value;
            let Some((_local, domain)) = Self::parse_email(email) else {
                return Err(TransformError::UnsupportedEntityType(
                    "email malformé".to_string(),
                ));
            };

            let mut observations = Vec::new();
            let mut entities = Vec::new();
            let mut relations = Vec::new();
            let now = chrono::Utc::now().to_rfc3339();

            // MX records
            let mx = self.mx_records(domain).await?;
            observations.push(Observation {
                id: format!("obs-email-mx-{}", email),
                subject: input.entity.id.clone(),
                predicate: "mx_records".to_string(),
                value: PropertyValue::Json(serde_json::to_value(&mx).unwrap_or_default()),
                source: "collect:email".to_string(),
                method: "DNS_MX".to_string(),
                observed_at: now.clone(),
                valid_from: None,
                valid_to: None,
                confidence: AdmiraltyCode::from_str("B2").unwrap_or(AdmiraltyCode {
                    source_reliability: spectra_core::SourceReliability::B,
                    information_credibility: spectra_core::InformationCredibility::V2,
                }),
                provenance: Provenance::Collected,
                raw_hash: None,
                operator: "SPECTRA".to_string(),
            });

            // TXT records (SPF, DMARC)
            let txt = self.txt_records(domain).await?;
            let spf = txt.iter().filter(|r| r.contains("v=spf1")).cloned().collect::<Vec<_>>();
            let dmarc = txt.iter().filter(|r| r.contains("v=DMARC1")).cloned().collect::<Vec<_>>();

            if !spf.is_empty() {
                observations.push(Observation {
                    id: format!("obs-email-spf-{}", email),
                    subject: input.entity.id.clone(),
                    predicate: "spf_record".to_string(),
                    value: PropertyValue::Json(serde_json::to_value(&spf).unwrap_or_default()),
                    source: "collect:email".to_string(),
                    method: "DNS_TXT".to_string(),
                    observed_at: now.clone(),
                    valid_from: None,
                    valid_to: None,
                    confidence: AdmiraltyCode::from_str("B2").unwrap_or(AdmiraltyCode {
                        source_reliability: spectra_core::SourceReliability::B,
                        information_credibility: spectra_core::InformationCredibility::V2,
                    }),
                    provenance: Provenance::Collected,
                    raw_hash: None,
                    operator: "SPECTRA".to_string(),
                });
            }

            if !dmarc.is_empty() {
                observations.push(Observation {
                    id: format!("obs-email-dmarc-{}", email),
                    subject: input.entity.id.clone(),
                    predicate: "dmarc_record".to_string(),
                    value: PropertyValue::Json(serde_json::to_value(&dmarc).unwrap_or_default()),
                    source: "collect:email".to_string(),
                    method: "DNS_TXT".to_string(),
                    observed_at: now.clone(),
                    valid_from: None,
                    valid_to: None,
                    confidence: AdmiraltyCode::from_str("B2").unwrap_or(AdmiraltyCode {
                        source_reliability: spectra_core::SourceReliability::B,
                        information_credibility: spectra_core::InformationCredibility::V2,
                    }),
                    provenance: Provenance::Collected,
                    raw_hash: None,
                    operator: "SPECTRA".to_string(),
                });
            }

            // Entité Domaine associée
            let domain_entity_id = format!("domain-{}", domain);
            entities.push(spectra_core::Entity {
                id: domain_entity_id.clone(),
                kind: EntityKind::Domain,
                canonical_value: domain.to_string(),
                display_label: domain.to_string(),
                properties: BTreeMap::new(),
                created_at: now.clone(),
                merged_from: vec![],
            });
            relations.push(spectra_transform::Relation {
                source: input.entity.id.clone(),
                target: domain_entity_id,
                kind: "hosted_on".to_string(),
                properties: BTreeMap::new(),
            });

            Ok(TransformOutput {
                entities,
                observations,
                relations,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_email() {
        assert_eq!(
            EmailTransform::parse_email("alice@example.com"),
            Some(("alice", "example.com"))
        );
    }

    #[test]
    fn parse_email_with_plus() {
        assert_eq!(
            EmailTransform::parse_email("alice+bob@example.com"),
            Some(("alice+bob", "example.com"))
        );
    }

    #[test]
    fn parse_invalid_email_no_at() {
        assert_eq!(EmailTransform::parse_email("aliceexample.com"), None);
    }

    #[test]
    fn generate_permutations_basic() {
        let perms = EmailTransform::generate_permutations("Jean", "Dupont", "@example.com");
        assert!(perms.contains(&"jean.dupont@example.com".to_string()));
        assert!(perms.contains(&"jeandupont@example.com".to_string()));
        assert!(perms.contains(&"jean_dupont@example.com".to_string()));
        assert!(perms.contains(&"jean-dupont@example.com".to_string()));
    }
}
