//! Transform téléphone — parsing E.164, opérateur, format national.
//!
//! # Objectif
//!
//! Normaliser un numéro de téléphone au format E.164, déterminer l'opérateur
//! à partir des préfixes nationaux embarqués, et produire un objet structuré
//! pour le graphe.
//!
//! # Limites
//!
//! La portabilité et la réputation nécessitent des sources tierces payantes ou
//! des bases locales ; elles sont laissées en TODO pour l'instant.

use spectra_core::{AdmiraltyCode, EntityKind, Observation, PropertyValue, Provenance};
use spectra_transform::{Transform, TransformContext, TransformError, TransformInput, TransformOutput};
use std::collections::BTreeMap;
use std::pin::Pin;

/// Transform natif Téléphone.
pub struct PhoneTransform;

impl PhoneTransform {
    pub fn new() -> Self {
        Self
    }

    /// Tente de normaliser un numéro en E.164 basique.
    /// Accepte +33 6 12 34 56 78, 06 12 34 56 78, etc.
    fn normalize_e164(raw: &str) -> Option<String> {
        let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() < 8 {
            return None;
        }
        if digits.starts_with('0') && digits.len() == 10 {
            // Numéro français national → ajouter +33 et supprimer le 0 initial
            return Some(format!("+33{}", &digits[1..]));
        }
        if digits.starts_with('1') && digits.len() == 11 {
            // Numéro US sans +
            return Some(format!("+{}", digits));
        }
        if digits.starts_with("33") && digits.len() == 11 {
            return Some(format!("+{}", digits));
        }
        if raw.starts_with('+') {
            return Some(format!("+{}", digits));
        }
        None
    }

    /// Devine l'opérateur à partir du préfixe E.164 français.
    fn infer_operator(e164: &str) -> &'static str {
        if let Some(national) = e164.strip_prefix("+33") {
            if national.is_empty() {
                return "Inconnu";
            }
            match national.chars().next().unwrap() {
                '6' | '7' => "Mobile",
                '1' | '2' | '3' | '4' | '5' | '9' => "Fixe",
                _ => "Inconnu",
            }
        } else {
            "Inconnu"
        }
    }
}

impl Transform for PhoneTransform {
    fn id(&self) -> &str {
        "collect:phone"
    }

    fn display_name(&self) -> &str {
        "Téléphone — E.164, opérateur"
    }

    fn input_kinds(&self) -> &[EntityKind] {
        &[EntityKind::PhoneNumber]
    }

    fn execute(
        &self,
        _ctx: TransformContext,
        input: TransformInput,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<TransformOutput, TransformError>> + Send + '_>> {
        Box::pin(async move {
            let raw = input.entity.canonical_value.clone();
            let Some(e164) = Self::normalize_e164(&raw) else {
                return Err(TransformError::UnsupportedEntityType(
                    "numéro de téléphone malformé".to_string(),
                ));
            };

            let operator = Self::infer_operator(&e164);
            let now = chrono::Utc::now().to_rfc3339();

            let mut observations = Vec::new();
            observations.push(Observation {
                id: format!("obs-phone-e164-{}", e164),
                subject: input.entity.id.clone(),
                predicate: "e164".to_string(),
                value: PropertyValue::String(e164.clone()),
                source: "collect:phone".to_string(),
                method: "NORMALIZE".to_string(),
                observed_at: now.clone(),
                valid_from: None,
                valid_to: None,
                confidence: AdmiraltyCode::from_str("A1").unwrap_or(AdmiraltyCode {
                    source_reliability: spectra_core::SourceReliability::A,
                    information_credibility: spectra_core::InformationCredibility::V1,
                }),
                provenance: Provenance::Collected,
                raw_hash: None,
                operator: "SPECTRA".to_string(),
            });

            observations.push(Observation {
                id: format!("obs-phone-op-{}", e164),
                subject: input.entity.id.clone(),
                predicate: "inferred_operator".to_string(),
                value: PropertyValue::String(operator.to_string()),
                source: "collect:phone".to_string(),
                method: "PREFIX_LOOKUP".to_string(),
                observed_at: now.clone(),
                valid_from: None,
                valid_to: None,
                confidence: AdmiraltyCode::from_str("C3").unwrap_or(AdmiraltyCode {
                    source_reliability: spectra_core::SourceReliability::C,
                    information_credibility: spectra_core::InformationCredibility::V3,
                }),
                provenance: Provenance::Inferred,
                raw_hash: None,
                operator: "SPECTRA".to_string(),
            });

            let mut properties = BTreeMap::new();
            properties.insert("e164".to_string(), PropertyValue::String(e164));
            properties.insert("operator".to_string(), PropertyValue::String(operator.to_string()));

            let mut entities = Vec::new();
            entities.push(spectra_core::Entity {
                id: input.entity.id.clone(),
                kind: EntityKind::PhoneNumber,
                canonical_value: raw,
                display_label: input.entity.display_label.clone(),
                properties,
                created_at: now,
                merged_from: vec![],
            });

            Ok(TransformOutput {
                entities,
                observations,
                relations: vec![],
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_french_mobile() {
        assert_eq!(
            PhoneTransform::normalize_e164("06 12 34 56 78"),
            Some("+33612345678".to_string())
        );
    }

    #[test]
    fn normalize_french_mobile_with_plus33() {
        assert_eq!(
            PhoneTransform::normalize_e164("+33 6 12 34 56 78"),
            Some("+33612345678".to_string())
        );
    }

    #[test]
    fn normalize_us_number() {
        assert_eq!(
            PhoneTransform::normalize_e164("1 555 123 4567"),
            Some("+15551234567".to_string())
        );
    }

    #[test]
    fn normalize_too_short() {
        assert_eq!(PhoneTransform::normalize_e164("12345"), None);
    }

    #[test]
    fn infer_operator_mobile() {
        assert_eq!(PhoneTransform::infer_operator("+33612345678"), "Mobile");
        assert_eq!(PhoneTransform::infer_operator("+33712345678"), "Mobile");
    }

    #[test]
    fn infer_operator_fixe() {
        assert_eq!(PhoneTransform::infer_operator("+33123456789"), "Fixe");
        assert_eq!(PhoneTransform::infer_operator("+33923456789"), "Fixe");
    }

    #[test]
    fn infer_operator_unknown() {
        assert_eq!(PhoneTransform::infer_operator("+442071234567"), "Inconnu");
    }
}
