//! Test d'intégration : génération d'un rapport PDF complet avec journal d'audit.

use spectra_core::{
    AdmiraltyCode, Entity, EntityKind, Observation, PropertyValue, Provenance,
};
use spectra_report::{ReportEngine, ReportFormat, ReportOptions};
use spectra_store::Store;
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

fn dummy_obs(subject: &str, predicate: &str, value: &str) -> Observation {
    Observation {
        id: format!("obs-{subject}-{predicate}"),
        subject: subject.to_string(),
        predicate: predicate.to_string(),
        value: PropertyValue::String(value.to_string()),
        source: "collect:whatsmyname".to_string(),
        method: "HTTP_GET".to_string(),
        observed_at: "2026-01-01T00:00:00Z".to_string(),
        valid_from: None,
        valid_to: None,
        confidence: AdmiraltyCode::from_str("C3").unwrap(),
        provenance: Provenance::Collected,
        raw_hash: None,
        operator: "Alice".to_string(),
    }
}

#[test]
fn pdf_report_with_audit_and_hashes() {
    let store = Store::open_in_memory().unwrap();

    // Entités et observations
    let e = dummy_entity("u1", EntityKind::Username);
    store.insert_entity(&e).unwrap();

    let mut obs1 = dummy_obs("u1", "profile_url", "https://example.com/u1");
    obs1.raw_hash = Some("blake3-abc123".to_string());
    store.insert_observation(&obs1).unwrap();

    let obs2 = dummy_obs("u1", "bio", "Rust developer");
    store.insert_observation(&obs2).unwrap();

    // Journal d'audit
    use spectra_audit::chain::AuditEvent;
    use serde_json::json;

    let ev1 = AuditEvent {
        id: "AE-001".to_string(),
        case_id: "CASE-42".to_string(),
        action: "create".to_string(),
        entity_kind: "case".to_string(),
        entity_id: None,
        actor: "system".to_string(),
        sequence: 1,
        payload: json!({}),
    };
    store.append_audit_event(&ev1).unwrap();

    let ev2 = AuditEvent {
        id: "AE-002".to_string(),
        case_id: "CASE-42".to_string(),
        action: "create".to_string(),
        entity_kind: "entity".to_string(),
        entity_id: Some("u1".to_string()),
        actor: "Alice".to_string(),
        sequence: 2,
        payload: json!({"source": "whatsmyname"}),
    };
    store.append_audit_event(&ev2).unwrap();

    // Génère le PDF
    let engine = ReportEngine::new();
    let opts = ReportOptions {
        title: "Rapport intégration".to_string(),
        case_id: "CASE-42".to_string(),
        operator: "Alice".to_string(),
        include_inferred: false,
        format: ReportFormat::Pdf,
    };
    let pdf = engine.generate_pdf(&store, &opts).unwrap();

    // Vérifie la structure PDF
    assert!(!pdf.is_empty());
    assert!(pdf.starts_with(b"%PDF"));
    // Un PDF de plusieurs pages avec audit et annexes fait > 1 Ko
    assert!(pdf.len() > 1_000, "PDF trop petit pour contenir audit + annexes");
}
