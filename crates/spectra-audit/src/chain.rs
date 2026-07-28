//! Chaîne de hachages d'audit.
//!
//! # Rôle
//!
//! Calcule et vérifie la chaîne de hachages qui rend le journal d'enquête
//! inaltérable sans détection. C'est la brique qui donne — ou non — sa valeur
//! probatoire au dossier.
//!
//! # Invariants
//!
//! 1. **Le hachage est calculé, jamais reçu.** Aucune valeur de hachage fournie
//!    par un appelant n'est stockée telle quelle.
//! 2. **`hash(n) = SHA-256(canonical_json(événement n) || hash(n-1))`**, où
//!    `hash(n-1)` est l'empreinte *effective* du maillon précédent du même
//!    dossier — pas un hachage recalculé à partir d'anciennes valeurs.
//! 3. **La vérification recalcule.** Constater qu'un champ est non vide n'est
//!    pas une vérification. Toute altération de `action`, `actor` ou `metadata`
//!    doit être détectée.
//! 4. **Sérialisation canonique déterministe** : clés triées, valeurs
//!    `undefined`/`null` traitées identiquement côté Rust et TypeScript.
//! 5. **Ordre total explicite** : les événements portent un numéro de séquence
//!    monotone par dossier. Trier sur un horodatage à la seconde ne définit pas
//!    un ordre.

use crate::canonical::canonical_json;
use sha2::{Digest, Sha256};

/// Un événement d'audit minimal pour le chaînage.
///
/// Les champs sont ceux strictement nécessaires au calcul du hachage. Le reste
/// (métadonnées, horodatages secondaires) est inclus dans `payload` qui sera
/// sérialisé de manière canonique.
#[derive(Debug, Clone)]
pub struct AuditEvent {
    /// Identifiant unique de l'événement.
    pub id: String,
    /// Identifiant du dossier.
    pub case_id: String,
    /// Action : `"create"`, `"update"`, `"delete"`, `"access"`, `"export"`.
    pub action: String,
    /// Type d'entité : `"case"`, `"evidence"`, `"subject"`, etc.
    pub entity_kind: String,
    /// Identifiant de l'entité (optionnel).
    pub entity_id: Option<String>,
    /// Acteur : analyste ou processus.
    pub actor: String,
    /// Numéro de séquence monotone par dossier.
    pub sequence: u64,
    /// Contenu sérialisable de l'événement (hors champs ci-dessus).
    pub payload: serde_json::Value,
}

/// Résultat du calcul d'un maillon.
#[derive(Debug, Clone)]
pub struct ChainLink {
    /// Hachage hexadécimal de l'événement.
    pub hash: String,
    /// Hachage du maillon précédent (vide pour le premier).
    pub previous_hash: String,
}

/// Calcule le hachage d'un événement d'audit.
///
/// ```
/// use spectra_audit::chain::{compute_link, AuditEvent};
/// use serde_json::json;
///
/// let event = AuditEvent {
///     id: "AE-001".to_string(),
///     case_id: "CASE-001".to_string(),
///     action: "create".to_string(),
///     entity_kind: "evidence".to_string(),
///     entity_id: Some("EVD-001".to_string()),
///     actor: "analyst@example.com".to_string(),
///     sequence: 1,
///     payload: json!({"source": "manual"}),
/// };
///
/// let link = compute_link(&event, "");
/// assert!(!link.hash.is_empty());
/// assert_eq!(link.previous_hash, "");
/// ```
pub fn compute_link(event: &AuditEvent, previous_hash: &str) -> ChainLink {
    // Construit l'objet à hacher dans l'ordre canonique (clés triées).
    // Le `previous_hash` effectif est inclus, pas un recalcul.
    let mut obj = serde_json::Map::new();
    obj.insert(
        "action".to_string(),
        serde_json::Value::String(event.action.clone()),
    );
    obj.insert(
        "actor".to_string(),
        serde_json::Value::String(event.actor.clone()),
    );
    obj.insert(
        "case_id".to_string(),
        serde_json::Value::String(event.case_id.clone()),
    );
    obj.insert(
        "entity_id".to_string(),
        match &event.entity_id {
            Some(id) => serde_json::Value::String(id.clone()),
            None => serde_json::Value::Null,
        },
    );
    obj.insert(
        "entity_kind".to_string(),
        serde_json::Value::String(event.entity_kind.clone()),
    );
    obj.insert(
        "id".to_string(),
        serde_json::Value::String(event.id.clone()),
    );
    obj.insert("payload".to_string(), event.payload.clone());
    obj.insert(
        "previous_hash".to_string(),
        serde_json::Value::String(previous_hash.to_string()),
    );
    obj.insert(
        "sequence".to_string(),
        serde_json::Value::Number(event.sequence.into()),
    );

    let canonical = canonical_json(&serde_json::Value::Object(obj));
    let hash = Sha256::digest(canonical.as_bytes());
    let hash_hex = hex::encode(hash);

    ChainLink {
        hash: hash_hex,
        previous_hash: previous_hash.to_string(),
    }
}

/// Vérifie un maillon de la chaîne.
///
/// Retourne `Ok(())` si le hachage correspond, une erreur descriptive sinon.
///
/// ```
/// use spectra_audit::chain::{compute_link, verify_link, AuditEvent};
/// use serde_json::json;
///
/// let event = AuditEvent {
///     id: "AE-001".to_string(),
///     case_id: "CASE-001".to_string(),
///     action: "create".to_string(),
///     entity_kind: "evidence".to_string(),
///     entity_id: Some("EVD-001".to_string()),
///     actor: "analyst@example.com".to_string(),
///     sequence: 1,
///     payload: json!({"source": "manual"}),
/// };
///
/// let link = compute_link(&event, "");
/// assert!(verify_link(&event, &link.hash, "").is_ok());
///
/// // Modifier l'action doit casser le hachage
/// let mut tampered = event.clone();
/// tampered.action = "delete".to_string();
/// assert!(verify_link(&tampered, &link.hash, "").is_err());
/// ```
pub fn verify_link(
    event: &AuditEvent,
    expected_hash: &str,
    previous_hash: &str,
) -> Result<(), String> {
    let computed = compute_link(event, previous_hash);
    if computed.hash == expected_hash {
        Ok(())
    } else {
        Err(format!(
            "hachage invalide pour l'événement {} : attendu {}, obtenu {}",
            event.id, expected_hash, computed.hash
        ))
    }
}

/// Vérifie une chaîne complète d'événements.
///
/// Retourne le nombre d'événements vérifiés, ou une erreur au premier maillon
/// rompu avec son index et la raison.
///
/// ```
/// use spectra_audit::chain::{compute_link, verify_chain, AuditEvent};
/// use serde_json::json;
///
/// let events = vec![
///     AuditEvent {
///         id: "AE-001".to_string(),
///         case_id: "CASE-001".to_string(),
///         action: "create".to_string(),
///         entity_kind: "case".to_string(),
///         entity_id: None,
///         actor: "system".to_string(),
///         sequence: 1,
///         payload: json!({}),
///     },
///     AuditEvent {
///         id: "AE-002".to_string(),
///         case_id: "CASE-001".to_string(),
///         action: "create".to_string(),
///         entity_kind: "evidence".to_string(),
///         entity_id: Some("EVD-001".to_string()),
///         actor: "analyst@example.com".to_string(),
///         sequence: 2,
///         payload: json!({"source": "manual"}),
///     },
/// ];
///
/// // Construit une chaîne valide
/// let mut previous_hash = String::new();
/// let mut links = Vec::new();
/// for event in &events {
///     let link = compute_link(event, &previous_hash);
///     previous_hash = link.hash.clone();
///     links.push(link);
/// }
///
/// // Vérifie la chaîne
/// assert!(verify_chain(&events, &links).is_ok());
/// ```
pub fn verify_chain(events: &[AuditEvent], links: &[ChainLink]) -> Result<usize, (usize, String)> {
    if events.len() != links.len() {
        return Err((
            0,
            format!(
                "nombre d'événements ({}) != nombre de maillons ({})",
                events.len(),
                links.len()
            ),
        ));
    }

    let mut previous_hash = String::new();
    for (i, (event, link)) in events.iter().zip(links.iter()).enumerate() {
        // Vérifie que le maillon référence bien le précédent
        if link.previous_hash != previous_hash {
            return Err((
                i,
                format!(
                    "rupture de chaînage à l'index {} : previous_hash attendu {}, obtenu {}",
                    i, previous_hash, link.previous_hash
                ),
            ));
        }

        // Vérifie le hachage de l'événement
        if let Err(reason) = verify_link(event, &link.hash, &previous_hash) {
            return Err((i, reason));
        }

        previous_hash = link.hash.clone();
    }

    Ok(events.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_first_link_empty_previous() {
        let event = AuditEvent {
            id: "AE-001".to_string(),
            case_id: "CASE-001".to_string(),
            action: "create".to_string(),
            entity_kind: "case".to_string(),
            entity_id: None,
            actor: "system".to_string(),
            sequence: 1,
            payload: json!({}),
        };

        let link = compute_link(&event, "");
        assert!(!link.hash.is_empty());
        assert_eq!(link.previous_hash, "");
    }

    #[test]
    fn test_chain_two_events() {
        let events = vec![
            AuditEvent {
                id: "AE-001".to_string(),
                case_id: "CASE-001".to_string(),
                action: "create".to_string(),
                entity_kind: "case".to_string(),
                entity_id: None,
                actor: "system".to_string(),
                sequence: 1,
                payload: json!({}),
            },
            AuditEvent {
                id: "AE-002".to_string(),
                case_id: "CASE-001".to_string(),
                action: "create".to_string(),
                entity_kind: "evidence".to_string(),
                entity_id: Some("EVD-001".to_string()),
                actor: "analyst@example.com".to_string(),
                sequence: 2,
                payload: json!({"source": "manual"}),
            },
        ];

        let mut previous_hash = String::new();
        let mut links = Vec::new();
        for event in &events {
            let link = compute_link(event, &previous_hash);
            previous_hash = link.hash.clone();
            links.push(link);
        }

        assert_eq!(verify_chain(&events, &links).unwrap(), 2);
    }

    #[test]
    fn test_tampered_action_detected() {
        let event = AuditEvent {
            id: "AE-001".to_string(),
            case_id: "CASE-001".to_string(),
            action: "create".to_string(),
            entity_kind: "case".to_string(),
            entity_id: None,
            actor: "system".to_string(),
            sequence: 1,
            payload: json!({}),
        };

        let link = compute_link(&event, "");

        // Modifie l'action
        let mut tampered = event.clone();
        tampered.action = "delete".to_string();

        assert!(verify_link(&tampered, &link.hash, "").is_err());
    }

    #[test]
    fn test_tampered_metadata_detected() {
        let event = AuditEvent {
            id: "AE-001".to_string(),
            case_id: "CASE-001".to_string(),
            action: "create".to_string(),
            entity_kind: "evidence".to_string(),
            entity_id: Some("EVD-001".to_string()),
            actor: "analyst@example.com".to_string(),
            sequence: 1,
            payload: json!({"source": "manual", "confidence": "high"}),
        };

        let link = compute_link(&event, "");

        // Modifie les métadonnées
        let mut tampered = event.clone();
        tampered.payload = json!({"source": "manual", "confidence": "low"});

        assert!(verify_link(&tampered, &link.hash, "").is_err());
    }

    #[test]
    fn test_broken_chain_detected() {
        let events = vec![
            AuditEvent {
                id: "AE-001".to_string(),
                case_id: "CASE-001".to_string(),
                action: "create".to_string(),
                entity_kind: "case".to_string(),
                entity_id: None,
                actor: "system".to_string(),
                sequence: 1,
                payload: json!({}),
            },
            AuditEvent {
                id: "AE-002".to_string(),
                case_id: "CASE-001".to_string(),
                action: "create".to_string(),
                entity_kind: "evidence".to_string(),
                entity_id: Some("EVD-001".to_string()),
                actor: "analyst@example.com".to_string(),
                sequence: 2,
                payload: json!({"source": "manual"}),
            },
        ];

        let mut previous_hash = String::new();
        let mut links = Vec::new();
        for event in &events {
            let link = compute_link(event, &previous_hash);
            previous_hash = link.hash.clone();
            links.push(link);
        }

        // Casse le chaînage du deuxième maillon
        links[1].previous_hash = "INVALID".to_string();

        let err = verify_chain(&events, &links).unwrap_err();
        assert_eq!(err.0, 1);
        assert!(err.1.contains("rupture de chaînage"));
    }

    #[test]
    fn test_canonical_json_deterministic() {
        use crate::canonical::canonical_json;
        use serde_json::json;

        // Deux objets sémantiquement égaux mais avec ordre de clés différent
        let obj1 = json!({"b": 2, "a": 1});
        let obj2 = json!({"a": 1, "b": 2});

        assert_eq!(canonical_json(&obj1), canonical_json(&obj2));
    }
}
