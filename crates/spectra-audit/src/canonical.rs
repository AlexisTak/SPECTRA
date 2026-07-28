//! Sérialisation JSON canonique.
//!
//! # Rôle
//!
//! Produit une chaîne JSON déterministe à partir d'une valeur : clés triées
//! alphabétiquement à tous les niveaux, valeurs `null` préservées. Cette fonction
//! est le pendant Rust de `lib/canonical.ts` — leurs sorties doivent être
//! strictement identiques pour les mêmes données, sinon le chaînage d'audit ne
//! peut pas être vérifié de manière croisée.
//!
//! # Contrat de concordance TS/Rust
//!
//! Un test d'intégration (à écrire dans `spectra-audit/tests/canonical_crosscheck.rs`)
//! doit vérifier que pour tout objet JSON :
//!
//! ```text
//! canonical_json_ts(obj) == canonical_json_rs(obj)
//! ```
//!
//! En pratique :
//! - Les clés sont triées dans l'ordre lexicographique Unicode (code point par
//!   code point, pas par locale).
//! - Les `null` sont sérialisés tels quels.
//! - Les nombres sont sérialisés sans formatage spécial (la représentation JSON
//!   par défaut de serde_json est suffisante).
//! - Les chaînes sont échappées selon la norme JSON (géré par serde_json).

use serde_json::Value;

/// Sérialise une valeur JSON de manière canonique : clés triées, `null` préservés.
///
/// ```
/// use spectra_audit::canonical::canonical_json;
/// use serde_json::json;
///
/// let obj = json!({"b": 2, "a": 1, "nested": {"z": 26, "a": 1}});
/// let canonical = canonical_json(&obj);
/// assert_eq!(canonical, r#"{"a":1,"b":2,"nested":{"a":1,"z":26}}"#);
/// ```
pub fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => serde_json::to_string(s).expect("échec sérialisation chaîne"),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(canonical_json).collect();
            format!("[{}]", items.join(","))
        }
        Value::Object(obj) => {
            let mut keys: Vec<&String> = obj.keys().collect();
            keys.sort();
            let pairs: Vec<String> = keys
                .iter()
                .map(|k| {
                    let v = obj.get(*k).unwrap();
                    format!(
                        "{}:{}",
                        serde_json::to_string(*k).expect("échec sérialisation clé"),
                        canonical_json(v)
                    )
                })
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_canonical_sorts_keys() {
        let obj = json!({"b": 2, "a": 1});
        assert_eq!(canonical_json(&obj), r#"{"a":1,"b":2}"#);
    }

    #[test]
    fn test_canonical_nested() {
        let obj = json!({"outer": {"z": 26, "a": 1}, "a": 1});
        assert_eq!(canonical_json(&obj), r#"{"a":1,"outer":{"a":1,"z":26}}"#);
    }

    #[test]
    fn test_canonical_array() {
        let arr = json!([3, 1, 2]);
        assert_eq!(canonical_json(&arr), "[3,1,2]");
    }

    #[test]
    fn test_canonical_null() {
        let val = json!(null);
        assert_eq!(canonical_json(&val), "null");
    }

    #[test]
    fn test_canonical_string_escaping() {
        let val = json!("hello\nworld");
        // serde_json échappe correctement les caractères spéciaux
        assert_eq!(canonical_json(&val), r#""hello\nworld""#);
    }

    #[test]
    fn test_canonical_empty_object() {
        let val = json!({});
        assert_eq!(canonical_json(&val), "{}");
    }

    #[test]
    fn test_canonical_empty_array() {
        let val = json!([]);
        assert_eq!(canonical_json(&val), "[]");
    }

    #[test]
    fn test_canonical_deep_nesting() {
        let obj = json!({"a": {"b": {"c": {"d": 4}}}});
        assert_eq!(canonical_json(&obj), r#"{"a":{"b":{"c":{"d":4}}}}"#);
    }

    #[test]
    fn test_canonical_mixed_types() {
        let obj = json!({
            "string": "hello",
            "number": 42,
            "float": 3.14,
            "bool": true,
            "null": null,
            "array": [1, 2, 3],
            "object": {"nested": "value"}
        });
        let result = canonical_json(&obj);
        // Vérifie que toutes les clés sont présentes et triées
        assert!(result.starts_with("{"));
        assert!(result.ends_with("}"));
        assert!(result.contains(r#""array":"#));
        assert!(result.contains(r#""bool":"#));
        assert!(result.contains(r#""null":"#));
        assert!(result.contains(r#""number":"#));
        assert!(result.contains(r#""float":"#));
        assert!(result.contains(r#""object":"#));
        assert!(result.contains(r#""string":"#));
        // Vérifie l'ordre alphabétique
        let pos_array = result.find(r#""array":"#).unwrap();
        let pos_bool = result.find(r#""bool":"#).unwrap();
        let pos_null = result.find(r#""null":"#).unwrap();
        let pos_number = result.find(r#""number":"#).unwrap();
        assert!(pos_array < pos_bool);
        assert!(pos_bool < pos_null);
        assert!(pos_null < pos_number);
    }
}
