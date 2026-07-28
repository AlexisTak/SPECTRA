//! Extraction de métadonnées depuis les réponses HTTP.

use crate::schema::{Extraction, ExtractionKind};
use regex::Regex;

/// Extrait les métadonnées d'une réponse HTML/JSON
pub fn extract_metadata(
    extractions: &[Extraction],
    body: &str,
    _final_url: &str,
) -> std::collections::HashMap<String, String> {
    let mut result = std::collections::HashMap::new();

    for ext in extractions {
        let value = match &ext.kind {
            ExtractionKind::Text => {
                // Extraction simple de texte (pourrait être améliorée avec un vrai parser HTML)
                Some(body.to_string())
            }
            ExtractionKind::Attribute { attr } => {
                // Chercher attr="valeur" dans le HTML
                let pattern = format!(r#"{}=["']([^"']+)["']"#, attr);
                Regex::new(&pattern)
                    .ok()
                    .and_then(|re| re.captures(body))
                    .map(|cap| cap[1].to_string())
            }
            ExtractionKind::JsonPointer { pointer } => {
                serde_json::from_str::<serde_json::Value>(body)
                    .ok()
                    .and_then(|json| json.pointer(pointer.as_str()).cloned())
                    .map(|v| v.to_string())
            }
            ExtractionKind::Regex { pattern } => {
                Regex::new(pattern)
                    .ok()
                    .and_then(|re| re.captures(body))
                    .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            }
        };

        if let Some(v) = value {
            result.insert(ext.name.clone(), v);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_pointer_extraction() {
        let json = r#"{"user": {"name": "Alice", "bio": "Developer"}}"#;
        let extractions = vec![
            Extraction {
                name: "name".to_string(),
                selector: "/user/name".to_string(),
                kind: ExtractionKind::JsonPointer { pointer: "/user/name".to_string() },
            },
        ];

        let result = extract_metadata(&extractions, json, "");
        assert_eq!(result.get("name"), Some(&"Alice".to_string()));
    }

    #[test]
    fn test_regex_extraction() {
        let html = r#"<span class="username">@johndoe</span>"#;
        let extractions = vec![
            Extraction {
                name: "username".to_string(),
                selector: r#"class="username">([^<]+)"#.to_string(),
                kind: ExtractionKind::Regex { pattern: r#"class="username">([^<]+)"#.to_string() },
            },
        ];

        let result = extract_metadata(&extractions, html, "");
        assert_eq!(result.get("username"), Some(&"@johndoe".to_string()));
    }
}
