//! Scoring d'identité et propositions de fusion.

use spectra_core::Entity;

/// Score de similarité entre deux entités.
#[derive(Debug, Clone)]
pub struct IdentityScore {
    /// Score global entre 0.0 et 1.0.
    pub score: f64,
    /// Explications lisibles pour l'analyste.
    pub reasons: Vec<String>,
}

/// Heuristiques de correspondance détectées entre deux entités.
#[derive(Debug, Clone)]
pub enum MatchHint {
    /// Pseudonyme identique (ou quasi-identique).
    SameUsername {
        /// Distance d'édition normalisée entre les deux pseudonymes.
        distance: f64,
    },
    /// Même avatar (pHash distance).
    SameAvatar {
        /// Distance de Hamming entre les deux pHash.
        phash_distance: u32,
    },
    /// Bio / description similaire.
    SimilarBio {
        /// Ratio de similarité de Levenshtein entre les deux biographies.
        levenshtein_ratio: f64,
    },
    /// Email partiellement masqué recoupé.
    EmailOverlap {
        /// Motif masqué recoupé (par exemple `j***@e***.com`).
        pattern: String,
    },
    /// Fuseau horaire d'activité chevauchant.
    OverlappingTimezone {
        /// Décalage horaire inféré, en heures par rapport à UTC.
        offset_hours: i8,
    },
}

/// Moteur de corrélation d'identité.
pub struct Correlator;

impl Correlator {
    /// Crée le moteur de corrélation. Sans état, sans I/O.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Compare deux entités et retourne un score + des raisons.
    ///
    /// Retourne `None` si aucune correspondance n'est détectée.
    pub fn compare(&self,
        a: &Entity,
        b: &Entity,
    ) -> Option<IdentityScore> {
        let mut score: f64 = 0.0;
        let mut reasons = Vec::new();

        // Pseudo / valeur canonique identique
        if a.canonical_value == b.canonical_value {
            score += 0.4;
            reasons.push(format!(
                "Valeur canonique identique : {}",
                a.canonical_value
            ));
        } else {
            let dist =
                strsim::normalized_levenshtein(&a.canonical_value, &b.canonical_value);
            if dist > 0.8 {
                score += 0.25;
                reasons.push(format!(
                    "Valeur canonique très similaire (Levenshtein {:.0}%)",
                    dist * 100.0
                ));
            }
        }

        // Propriétés en commun
        let common_props: Vec<_> = a
            .properties
            .keys()
            .filter(|k| b.properties.contains_key(*k))
            .collect();
        for k in common_props {
            if let (Some(pa), Some(pb)) = (a.properties.get(k), b.properties.get(k)) {
                if pa == pb {
                    score += 0.15;
                    reasons.push(format!("Propriété '{}' identique", k));
                } else {
                    // Si c'est une chaîne, on calcule une distance de Levenshtein
                    if let (Ok(sa), Ok(sb)) = (
                        serde_json::to_string(pa),
                        serde_json::to_string(pb),
                    ) {
                        let d = strsim::normalized_levenshtein(&sa, &sb);
                        if d > 0.7 {
                            score += 0.1;
                            reasons.push(format!(
                                "Propriété '{}' similaire ({:.0}%)",
                                k,
                                d * 100.0
                            ));
                        }
                    }
                }
            }
        }

        if score > 0.0 {
            Some(IdentityScore {
                score: score.min(1.0),
                reasons,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectra_core::{Entity, EntityKind, PropertyValue};
    use std::collections::BTreeMap;

    fn dummy_entity(id: &str, value: &str, props: BTreeMap<String, PropertyValue>) -> Entity {
        Entity {
            id: id.to_string(),
            kind: EntityKind::Username,
            canonical_value: value.to_string(),
            display_label: value.to_string(),
            properties: props,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            merged_from: vec![],
        }
    }

    #[test]
    fn exact_match() {
        let c = Correlator::new();
        let a = dummy_entity("e1", "alice", BTreeMap::new());
        let b = dummy_entity("e2", "alice", BTreeMap::new());
        let res = c.compare(&a, &b).expect("doit matcher");
        assert!(res.score >= 0.4);
        assert!(res.reasons.iter().any(|r| r.contains("identique")));
    }

    #[test]
    fn no_match() {
        let c = Correlator::new();
        let a = dummy_entity("e1", "alice", BTreeMap::new());
        let b = dummy_entity("e2", "bob", BTreeMap::new());
        assert!(c.compare(&a, &b).is_none());
    }

    #[test]
    fn similar_username() {
        let c = Correlator::new();
        let a = dummy_entity("e1", "alicce", BTreeMap::new());
        let b = dummy_entity("e2", "alice", BTreeMap::new());
        let res = c.compare(&a, &b).expect("doit matcher");
        assert!(res.score > 0.0);
        assert!(res.reasons.iter().any(|r| r.contains("similaire")));
    }

    #[test]
    fn common_property() {
        let c = Correlator::new();
        let mut props_a = BTreeMap::new();
        props_a.insert("bio".to_string(), PropertyValue::String("dev rust".to_string()));
        let mut props_b = BTreeMap::new();
        props_b.insert("bio".to_string(), PropertyValue::String("dev rust".to_string()));
        let a = dummy_entity("e1", "alice", props_a);
        let b = dummy_entity("e2", "alicce", props_b);
        let res = c.compare(&a, &b).expect("doit matcher");
        assert!(res.score > 0.3);
        assert!(res.reasons.iter().any(|r| r.contains("bio")));
    }
}
