//! Persistance SPECTRA.
//!
//! # Rôle
//!
//! Traduit le modèle de [`spectra_core`] vers un fichier de dossier `.spectra`
//! (une base `SQLite` unique par enquête) et retour. Seul crate autorisé à
//! connaître SQL.
//!
//! # Invariants
//!
//! 1. **`SQLite` est la source de vérité unique.** Aucune base graphe n'est une
//!    dépendance obligatoire (voir `docs/adr/0002`).
//! 2. **Toute écriture multi-tables est transactionnelle.** Un échec partiel ne
//!    doit jamais laisser une preuve sans sa trace d'audit.
//! 3. **Aucune suppression physique** d'un élément porteur de valeur probatoire.
//! 4. Un dossier est portable et chiffrable : le chemin du fichier est fourni
//!    par l'appelant.

pub mod audit;
pub mod entity;
pub mod fragment;
pub mod observation;
pub mod relation;
pub mod store;

pub use audit::StoredAuditEvent;
pub use relation::StoredRelation;
pub use store::{Store, StoreError};

#[cfg(test)]
mod tests {
    use super::*;
    use spectra_core::{
        AdmiraltyCode, Entity, EntityKind, Observation, PropertyValue, Provenance,
    };
    use std::collections::BTreeMap;

    fn dummy_entity(id: &str, kind: EntityKind) -> Entity {
        Entity {
            id: id.to_string(),
            kind,
            canonical_value: id.to_string(),
            display_label: id.to_string(),
            properties: BTreeMap::new(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            merged_from: vec![],
        }
    }

    #[test]
    fn roundtrip_entity_and_observation() {
        let store = Store::open_in_memory().unwrap();

        let e = dummy_entity("e1", EntityKind::Username);
        store.insert_entity(&e).unwrap();

        let obs = Observation {
            id: "obs1".to_string(),
            subject: "e1".to_string(),
            predicate: "profile_url".to_string(),
            value: PropertyValue::String("https://example.com/u1".to_string()),
            source: "collect:whatsmyname".to_string(),
            method: "HTTP_GET".to_string(),
            observed_at: "2026-01-01T00:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
            confidence: AdmiraltyCode::from_str("C3").unwrap(),
            provenance: Provenance::Collected,
            raw_hash: None,
            operator: "test".to_string(),
        };
        store.insert_observation(&obs).unwrap();

        let got = store.get_entity("e1").unwrap().expect("entity existe");
        assert_eq!(got.id, "e1");
        assert_eq!(got.kind, EntityKind::Username);

        let obs_list = store.get_observations_by_subject("e1").unwrap();
        assert_eq!(obs_list.len(), 1);
        assert_eq!(obs_list[0].predicate, "profile_url");
        assert_eq!(obs_list[0].confidence.source_reliability, spectra_core::SourceReliability::C);
    }

    #[test]
    fn roundtrip_relation() {
        let store = Store::open_in_memory().unwrap();
        let a = dummy_entity("a", EntityKind::Username);
        let b = dummy_entity("b", EntityKind::SocialProfile);
        store.insert_entity(&a).unwrap();
        store.insert_entity(&b).unwrap();

        let rel = StoredRelation {
            id: None,
            source: "a".to_string(),
            target: "b".to_string(),
            kind: "has_profile_on".to_string(),
            properties: BTreeMap::new(),
        };
        store.insert_relation(&rel).unwrap();

        let from_a = store.get_relations_from("a").unwrap();
        assert_eq!(from_a.len(), 1);
        assert_eq!(from_a[0].target, "b");

        let to_b = store.get_relations_to("b").unwrap();
        assert_eq!(to_b.len(), 1);
        assert_eq!(to_b[0].source, "a");
    }

    #[test]
    fn export_import_fragment_roundtrip() {
        let store_a = Store::open_in_memory().unwrap();
        let e = dummy_entity("e1", EntityKind::Username);
        store_a.insert_entity(&e).unwrap();
        store_a.insert_observation(&dummy_obs("e1", "bio", "Rust dev")).unwrap();

        let frag = store_a
            .export_fragment(&["e1".to_string()],
                "Alice",
            )
            .unwrap();

        assert_eq!(frag.entities.len(), 1);
        assert_eq!(frag.observations.len(), 1);

        let store_b = Store::open_in_memory().unwrap();
        let summary = store_b.import_fragment(&frag).unwrap();
        assert_eq!(summary.entities_inserted, 1);
        assert_eq!(summary.observations_inserted, 1);

        let got = store_b.get_entity("e1").unwrap().expect("entity importée");
        assert_eq!(got.canonical_value, "e1");
        let obs = store_b.get_observations_by_subject("e1").unwrap();
        assert_eq!(obs[0].predicate, "bio");

        // Réimport identique → skipped
        let summary2 = store_b.import_fragment(&frag).unwrap();
        assert_eq!(summary2.entities_skipped, 1);
        assert_eq!(summary2.observations_skipped, 1);
    }

    fn dummy_obs(subject: &str, predicate: &str, value: &str) -> spectra_core::Observation {
        spectra_core::Observation {
            id: format!("obs-{subject}-{predicate}"),
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            value: spectra_core::PropertyValue::String(value.to_string()),
            source: "test".to_string(),
            method: "TEST".to_string(),
            observed_at: "2026-01-01T00:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
            confidence: spectra_core::AdmiraltyCode::from_str("A1").unwrap(),
            provenance: spectra_core::Provenance::Collected,
            raw_hash: None,
            operator: "test".to_string(),
        }
    }
}
