//! Transform WHOIS — requête vers les serveurs de registre.

use spectra_core::{Entity, EntityKind, Observation, PropertyValue};
use spectra_transform::{
    Transform, TransformContext, TransformError, TransformInput, TransformOutput,
};
use std::future::Future;
use std::pin::Pin;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Transform WHOIS : interroge le serveur WHOIS d'un domaine ou d'une IP.
pub struct WhoisTransform;

impl WhoisTransform {
    pub fn new() -> Self {
        Self
    }

    /// Détermine le serveur WHOIS pour un domaine donné.
    fn guess_server(domain: &str) -> &'static str {
        // Heuristique simple : les TLDs principaux.
        if domain.ends_with(".com") || domain.ends_with(".net") {
            "whois.verisign-grs.com"
        } else if domain.ends_with(".org") {
            "whois.publicinterestregistry.net"
        } else if domain.ends_with(".fr") {
            "whois.nic.fr"
        } else if domain.ends_with(".de") {
            "whois.denic.de"
        } else if domain.ends_with(".uk") {
            "whois.nic.uk"
        } else if domain.ends_with(".io") {
            "whois.nic.io"
        } else {
            "whois.iana.org"
        }
    }
}

impl Transform for WhoisTransform {
    fn id(&self) -> &str {
        "whois:lookup"
    }

    fn display_name(&self) -> &str {
        "Requête WHOIS"
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
            let query = &input.entity.canonical_value;
            let server = Self::guess_server(query);

            let addr = format!("{}:43", server);
            let mut stream = TcpStream::connect(&addr)
                .await
                .map_err(|e| TransformError::NetworkError(format!("TCP WHOIS: {e}")))?;

            stream
                .write_all(format!("{}\r\n", query).as_bytes())
                .await
                .map_err(|e| TransformError::NetworkError(format!("send: {e}")))?;

            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            let timeout = tokio::time::Duration::from_secs(10);

            loop {
                match tokio::time::timeout(timeout, stream.read(&mut chunk)).await {
                    Ok(Ok(0)) => break,
                    Ok(Ok(n)) => buf.extend_from_slice(&chunk[..n]),
                    Ok(Err(e)) => {
                        return Err(TransformError::NetworkError(format!("read: {e}")))
                    }
                    Err(_) => break, // timeout
                }
            }

            let raw = String::from_utf8_lossy(&buf);
            let now = chrono::Utc::now().to_rfc3339();
            let mut output = TransformOutput::default();

            // Extraction naïve de champs courants.
            let mut registrar: Option<String> = None;
            let mut creation_date: Option<String> = None;
            let mut expiry_date: Option<String> = None;

            for line in raw.lines() {
                let lower = line.to_lowercase();
                if lower.starts_with("registrar:") && registrar.is_none() {
                    registrar = Some(line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string());
                }
                if (lower.starts_with("creation date:") || lower.starts_with("created:")) && creation_date.is_none() {
                    creation_date = Some(line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string());
                }
                if (lower.starts_with("registry expiry date:") || lower.starts_with("expires:") || lower.starts_with("expiration date:")) && expiry_date.is_none() {
                    expiry_date = Some(line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string());
                }
            }

            if let Some(v) = registrar {
                output.observations.push(Observation {
                    id: format!("obs-whois-registrar-{}", &query),
                    subject: input.entity.id.clone(),
                    predicate: "whois_registrar".to_string(),
                    value: PropertyValue::String(v),
                    source: "whois:lookup".to_string(),
                    method: "WHOIS_TCP".to_string(),
                    observed_at: now.clone(),
                    valid_from: None,
                    valid_to: None,
                    confidence: spectra_core::AdmiraltyCode::from_str("C3").unwrap_or(spectra_core::AdmiraltyCode {
                        source_reliability: spectra_core::SourceReliability::C,
                        information_credibility: spectra_core::InformationCredibility::V3,
                    }),
                    provenance: spectra_core::Provenance::Collected,
                    raw_hash: None,
                    operator: "SPECTRA".to_string(),
                });
            }
            if let Some(v) = creation_date {
                output.observations.push(Observation {
                    id: format!("obs-whois-created-{}", &query),
                    subject: input.entity.id.clone(),
                    predicate: "whois_creation_date".to_string(),
                    value: PropertyValue::String(v),
                    source: "whois:lookup".to_string(),
                    method: "WHOIS_TCP".to_string(),
                    observed_at: now.clone(),
                    valid_from: None,
                    valid_to: None,
                    confidence: spectra_core::AdmiraltyCode::from_str("C3").unwrap_or(spectra_core::AdmiraltyCode {
                        source_reliability: spectra_core::SourceReliability::C,
                        information_credibility: spectra_core::InformationCredibility::V3,
                    }),
                    provenance: spectra_core::Provenance::Collected,
                    raw_hash: None,
                    operator: "SPECTRA".to_string(),
                });
            }
            if let Some(v) = expiry_date {
                output.observations.push(Observation {
                    id: format!("obs-whois-expiry-{}", &query),
                    subject: input.entity.id.clone(),
                    predicate: "whois_expiry_date".to_string(),
                    value: PropertyValue::String(v),
                    source: "whois:lookup".to_string(),
                    method: "WHOIS_TCP".to_string(),
                    observed_at: now.clone(),
                    valid_from: None,
                    valid_to: None,
                    confidence: spectra_core::AdmiraltyCode::from_str("C3").unwrap_or(spectra_core::AdmiraltyCode {
                        source_reliability: spectra_core::SourceReliability::C,
                        information_credibility: spectra_core::InformationCredibility::V3,
                    }),
                    provenance: spectra_core::Provenance::Collected,
                    raw_hash: None,
                    operator: "SPECTRA".to_string(),
                });
            }

            Ok(output)
        })
    }
}
