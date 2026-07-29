//! Test d'intégration WhatsMyName.
//!
//! Ce test touche le réseau. Il est marqué `#[ignore]` et doit être lancé
//! explicitement :
//!   cargo test -p spectra-collect --test whatsmyname_integration -- --ignored

use spectra_collect::WhatsMyNameTransform;
use spectra_core::{Entity, EntityKind, Provenance};
use spectra_store::Store;
use spectra_transform::{Transform, TransformContext, TransformInput};
use std::collections::BTreeMap;

#[tokio::test]
#[ignore]
async fn whatsmyname_populates_case_with_provenance() {
    let store = Store::open_in_memory().unwrap();

    let username = "testuser";
    let input_entity = Entity {
        id: "ent-username-1".to_string(),
        kind: EntityKind::Username,
        canonical_value: username.to_string(),
        display_label: username.to_string(),
        properties: BTreeMap::new(),
        created_at: chrono::Utc::now().to_rfc3339(),
        merged_from: vec![],
    };

    let transform = WhatsMyNameTransform::new().unwrap();
    let ctx = TransformContext {
        case_id: "case-test".to_string(),
        network_profile: "direct".to_string(),
        user_agent: "SPECTRA-TEST/0.1".to_string(),
        operator: "test".to_string(),
    };
    let input = TransformInput {
        entity: input_entity.clone(),
        params: BTreeMap::new(),
    };

    let t0 = std::time::Instant::now();
    let output = transform.execute(ctx, input).await.unwrap();
    let elapsed = t0.elapsed();

    // Persister dans le store
    store.insert_entity(&input_entity).unwrap();
    for e in &output.entities {
        store.insert_entity(e).unwrap();
    }
    for o in &output.observations {
        store.insert_observation(o).unwrap();
    }
    for r in &output.relations {
        let rel = spectra_store::StoredRelation {
            id: None,
            source: r.source.clone(),
            target: r.target.clone(),
            kind: r.kind.clone(),
            properties: r.properties.clone(),
        };
        store.insert_relation(&rel).unwrap();
    }

    // Vérifications
    let entities = store.list_entities().unwrap();
    assert!(
        entities.len() > 1,
        "au moins une entité découverte (trouvé {})",
        entities.len()
    );

    let mut total_obs = 0;
    for e in &entities {
        if e.id == input_entity.id {
            continue;
        }
        let obs = store.get_observations_by_subject(&e.id).unwrap();
        total_obs += obs.len();
        for o in &obs {
            assert_eq!(
                o.provenance,
                Provenance::Collected,
                "toute observation doit avoir provenance=Collected"
            );
            assert_eq!(
                o.source, "collect:whatsmyname",
                "source attendue : collect:whatsmyname"
            );
            assert!(
                o.confidence.source_reliability >= spectra_core::SourceReliability::C,
                "fiabilité source C minimum"
            );
        }
    }

    println!(
        "WhatsMyName pour '{}' : {} entités (dont input), {} observations, {} relations en {:?}",
        username,
        entities.len(),
        total_obs,
        output.relations.len(),
        elapsed
    );
    assert!(
        elapsed.as_secs() < 60,
        "doit s'exécuter en moins de 60 s (a pris {:?})",
        elapsed
    );
}
