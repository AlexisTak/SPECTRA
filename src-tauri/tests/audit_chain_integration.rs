//! Test d'intégration du chaînage d'audit.
//!
//! Vérifie de bout en bout que :
//! 1. La chaîne est valide après une séquence normale de créations/modifications
//! 2. Toute altération d'un hash est détectée avec précision

use spectra_audit::{compute_link, verify_chain, AuditEvent, ChainLink};
use serde_json::json;

#[test]
fn test_compute_link_deterministic() {
    let event = AuditEvent {
        id: "ae-test".to_string(),
        case_id: "case-test".to_string(),
        action: "create".to_string(),
        entity_kind: "case".to_string(),
        entity_id: None,
        actor: "test".to_string(),
        sequence: 1,
        payload: json!({"titre": "Test"}),
    };

    // Le même événement avec le même previous_hash doit produire le même hash
    let link1 = compute_link(&event, "");
    let link2 = compute_link(&event, "");

    assert_eq!(link1.hash, link2.hash, "Le hash n'est pas déterministe");
    assert_eq!(link1.previous_hash, link2.previous_hash);

    // Un previous_hash différent doit produire un hash différent
    let link3 = compute_link(&event, "previous-different");
    assert_ne!(link1.hash, link3.hash, "Le hash doit changer avec un previous_hash différent");
}

#[test]
fn test_verify_chain_valid() {
    let events = vec![
        AuditEvent {
            id: "ae-1".to_string(),
            case_id: "case-1".to_string(),
            action: "create".to_string(),
            entity_kind: "case".to_string(),
            entity_id: None,
            actor: "system".to_string(),
            sequence: 1,
            payload: json!({}),
        },
        AuditEvent {
            id: "ae-2".to_string(),
            case_id: "case-1".to_string(),
            action: "create".to_string(),
            entity_kind: "evidence".to_string(),
            entity_id: Some("evd-1".to_string()),
            actor: "analyst".to_string(),
            sequence: 2,
            payload: json!({"type": "file"}),
        },
        AuditEvent {
            id: "ae-3".to_string(),
            case_id: "case-1".to_string(),
            action: "update".to_string(),
            entity_kind: "case".to_string(),
            entity_id: Some("case-1".to_string()),
            actor: "analyst".to_string(),
            sequence: 3,
            payload: json!({"statut": "en_cours"}),
        },
    ];

    // Construit une chaîne valide
    let mut previous_hash = String::new();
    let mut links: Vec<ChainLink> = Vec::new();
    for event in &events {
        let link = compute_link(event, &previous_hash);
        previous_hash = link.hash.clone();
        links.push(link);
    }

    // Vérifie la chaîne - doit passer
    let result = verify_chain(&events, &links);
    assert!(result.is_ok(), "La chaîne valide a échoué : {:?}", result);
    assert_eq!(result.unwrap(), 3, "Nombre d'événements vérifiés incorrect");
}

#[test]
fn test_verify_chain_detects_broken_link() {
    let events = vec![
        AuditEvent {
            id: "ae-1".to_string(),
            case_id: "case-2".to_string(),
            action: "create".to_string(),
            entity_kind: "case".to_string(),
            entity_id: None,
            actor: "system".to_string(),
            sequence: 1,
            payload: json!({}),
        },
        AuditEvent {
            id: "ae-2".to_string(),
            case_id: "case-2".to_string(),
            action: "create".to_string(),
            entity_kind: "evidence".to_string(),
            entity_id: Some("evd-2".to_string()),
            actor: "analyst".to_string(),
            sequence: 2,
            payload: json!({}),
        },
        AuditEvent {
            id: "ae-3".to_string(),
            case_id: "case-2".to_string(),
            action: "update".to_string(),
            entity_kind: "case".to_string(),
            entity_id: Some("case-2".to_string()),
            actor: "analyst".to_string(),
            sequence: 3,
            payload: json!({}),
        },
    ];

    // Construit une chaîne valide
    let mut previous_hash = String::new();
    let mut links: Vec<ChainLink> = Vec::new();
    for event in &events {
        let link = compute_link(event, &previous_hash);
        previous_hash = link.hash.clone();
        links.push(link);
    }

    // Corrompt le hash du 2ème événement
    links[1].hash = "TAMPERED".to_string();

    // Vérifie - doit échouer sur ae-2 (dont le hash a été corrompu)
    // ou ae-3 (qui référence le hash corrompu)
    let result = verify_chain(&events, &links);
    assert!(result.is_err(), "La corruption n'a pas été détectée");
    let (index, reason) = result.unwrap_err();
    // La corruption peut être détectée soit sur l'élément corrompu (index 1),
    // soit sur le suivant qui référence un previous_hash invalide (index 2)
    assert!(index == 1 || index == 2, "Index de corruption inattendu : {}", index);
    assert!(reason.contains("rupture") || reason.contains("invalide"), "Message d'erreur inattendu : {}", reason);
}

#[test]
fn test_verify_chain_detects_tampered_payload() {
    let event = AuditEvent {
        id: "ae-tamper".to_string(),
        case_id: "case-tamper".to_string(),
        action: "create".to_string(),
        entity_kind: "case".to_string(),
        entity_id: None,
        actor: "system".to_string(),
        sequence: 1,
        payload: json!({"titre": "Original"}),
    };

    let link = compute_link(&event, "");

    // Modifie le payload
    let mut tampered_event = event.clone();
    tampered_event.payload = json!({"titre": "Tampered"});

    // Vérifie - doit échouer
    let result = verify_chain(&[tampered_event], &[link]);
    assert!(result.is_err(), "La modification du payload n'a pas été détectée");
}
