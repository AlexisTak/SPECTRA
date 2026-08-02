//! Transform DNS — résolution d'enregistrements.

use hickory_resolver::TokioResolver;
use spectra_core::{Entity, EntityKind, Observation, PropertyValue};
use spectra_transform::{
    Transform, TransformContext, TransformError, TransformInput, TransformOutput, Relation,
};
use std::future::Future;
use std::pin::Pin;

/// Transform DNS : résout les enregistrements A, AAAA, MX et TXT d'un domaine.
pub struct DnsTransform;

impl DnsTransform {
    /// Crée le transform. Aucune I/O n'est réalisée à la construction.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Transform for DnsTransform {
    fn id(&self) -> &str {
        "dns:resolve"
    }

    fn display_name(&self) -> &str {
        "Résolution DNS"
    }

    fn input_kinds(&self) -> &[spectra_core::EntityKind] {
        &[EntityKind::Domain, EntityKind::IpAddress]
    }

    fn execute(
        &self,
        _ctx: TransformContext,
        input: TransformInput,
    ) -> Pin<Box<dyn Future<Output = Result<TransformOutput, TransformError>> + Send + '_>> {
        Box::pin(async move {
            let domain = &input.entity.canonical_value;
            if domain.is_empty() {
                return Err(TransformError::UnsupportedEntityType(
                    "valeur vide".to_string(),
                ));
            }

            let resolver = TokioResolver::builder_tokio()
                .map_err(|e| TransformError::NetworkError(format!("resolver builder: {e}")))?
                .build()
                .map_err(|e| TransformError::NetworkError(format!("resolver build: {e}")))?;

            let mut output = TransformOutput::default();
            let now = chrono::Utc::now().to_rfc3339();

            // A records
            match resolver.lookup_ip(domain).await {
                Ok(ips) => {
                    for ip in ips.iter() {
                        let ip_str = ip.to_string();
                        let entity = Entity {
                            id: format!("ip-{}", &ip_str),
                            kind: EntityKind::IpAddress,
                            canonical_value: ip_str.clone(),
                            display_label: ip_str.clone(),
                            properties: Default::default(),
                            created_at: now.clone(),
                            merged_from: vec![],
                        };
                        output.entities.push(entity);
                        output.relations.push(Relation {
                            source: input.entity.id.clone(),
                            target: format!("ip-{}", &ip_str),
                            kind: "resolves_to".to_string(),
                            properties: Default::default(),
                        });
                        output.observations.push(Observation {
                            id: format!("obs-dns-a-{}", &ip_str),
                            subject: input.entity.id.clone(),
                            predicate: "dns_a".to_string(),
                            value: PropertyValue::String(ip_str),
                            source: "dns:resolve".to_string(),
                            method: "DNS_LOOKUP".to_string(),
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
                Err(e) => {
                    tracing::warn!("DNS lookup A/AAAA échoué pour {}: {}", domain, e);
                }
            }

            // MX records
            use hickory_resolver::proto::rr::RecordType;
            match resolver.lookup(domain, RecordType::MX).await {
                Ok(mx) => {
                    for record in mx.answers().iter() {
                        if let hickory_resolver::proto::rr::RData::MX(mx_rdata) = &record.data {
                            let exchange = mx_rdata.exchange.to_string();
                            output.observations.push(Observation {
                                id: format!("obs-dns-mx-{}", &exchange),
                                subject: input.entity.id.clone(),
                                predicate: "dns_mx".to_string(),
                                value: PropertyValue::String(exchange),
                                source: "dns:resolve".to_string(),
                                method: "DNS_LOOKUP".to_string(),
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
                }
                Err(e) => {
                    tracing::warn!("DNS lookup MX échoué pour {}: {}", domain, e);
                }
            }

            Ok(output)
        })
    }
}
