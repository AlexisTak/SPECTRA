//! Moteur de décision — évaluation des assertions.
//!
//! Ordre d'évaluation strict : Exists puis Missing puis fallback Indeterminate.
//! Aucun accès réseau ici : on évalue sur une réponse déjà reçue.

use crate::schema::{Assertion, DecisionRules, ProbeOutcome, BlockReason, Evidence};
use regex::Regex;

/// Résultat de l'évaluation d'une assertion
#[derive(Debug, Clone)]
pub struct AssertionResult {
    pub matched: bool,
    pub detail: Option<String>,
}

/// Évalue une assertion sur une réponse HTTP
pub fn evaluate_assertion(
    assertion: &Assertion,
    status: u16,
    body: &str,
    final_url: &str,
    headers: &http::HeaderMap,
) -> AssertionResult {
    match assertion {
        Assertion::StatusCode(code) => {
            AssertionResult {
                matched: status == *code,
                detail: Some(format!("status={status}, attendu={code}")),
            }
        }
        Assertion::StatusIn { min, max } => {
            AssertionResult {
                matched: status >= *min && status <= *max,
                detail: Some(format!("status={status}, intervalle=[{min},{max}]")),
            }
        }
        Assertion::BodyContains(text) => {
            AssertionResult {
                matched: body.contains(text),
                detail: Some(format!("body contains '{}': {}", text, body.contains(text))),
            }
        }
        Assertion::BodyNotContains(text) => {
            AssertionResult {
                matched: !body.contains(text),
                detail: Some(format!("body not contains '{}': {}", text, !body.contains(text))),
            }
        }
        Assertion::BodyMatches(pattern) => {
            match Regex::new(pattern) {
                Ok(re) => {
                    let matched = re.is_match(body);
                    AssertionResult {
                        matched,
                        detail: Some(format!("regex '{}' matched: {}", pattern, matched)),
                    }
                }
                Err(e) => AssertionResult {
                    matched: false,
                    detail: Some(format!("regex invalide: {}", e)),
                },
            }
        }
        Assertion::FinalUrlContains(text) => {
            AssertionResult {
                matched: final_url.contains(text),
                detail: Some(format!("url '{}' contains '{}': {}", final_url, text, final_url.contains(text))),
            }
        }
        Assertion::FinalUrlEquals(url) => {
            AssertionResult {
                matched: final_url == url,
                detail: Some(format!("url '{}' == '{}': {}", final_url, url, final_url == url)),
            }
        }
        Assertion::HeaderPresent(name) => {
            let present = headers.contains_key(name.as_str());
            AssertionResult {
                matched: present,
                detail: Some(format!("header '{}' present: {}", name, present)),
            }
        }
        Assertion::JsonPointerEquals { pointer, value } => {
            match serde_json::from_str::<serde_json::Value>(body) {
                Ok(json) => {
                    let ptr_value = json.pointer(pointer.as_str());
                    let matched = ptr_value == Some(value);
                    AssertionResult {
                        matched,
                        detail: Some(format!("pointer '{}' matched: {}", pointer, matched)),
                    }
                }
                Err(_) => AssertionResult {
                    matched: false,
                    detail: Some("body n'est pas du JSON valide".to_string()),
                },
            }
        }
        Assertion::ContentLengthGreaterThan(len) => {
            AssertionResult {
                matched: body.len() > *len,
                detail: Some(format!("body.len()={} > {}: {}", body.len(), len, body.len() > *len)),
            }
        }
    }
}

/// Évalue les règles de décision
pub fn evaluate_decision(
    rules: &DecisionRules,
    status: u16,
    body: &str,
    final_url: &str,
    headers: &http::HeaderMap,
) -> ProbeOutcome {
    // 1. Vérifier si bloqué
    for assertion in &rules.blocked_if {
        if evaluate_assertion(assertion, status, body, final_url, headers).matched {
            return ProbeOutcome::Blocked {
                reason: detect_block_reason(status, body, headers),
            };
        }
    }

    // 2. Vérifier si existe
    for assertion in &rules.exists_if {
        let result = evaluate_assertion(assertion, status, body, final_url, headers);
        if result.matched {
            return ProbeOutcome::Exists {
                evidence: Evidence {
                    triggered_assertion: result.detail,
                    status_code: status,
                    final_url: final_url.to_string(),
                    body_hash: blake3::hash(body.as_bytes()).to_string(),
                    body_preview: Some(body.chars().take(256).collect()),
                    timestamp: chrono::Utc::now(),
                },
                extracted: Default::default(),
            };
        }
    }

    // 3. Vérifier si manquant
    for assertion in &rules.missing_if {
        let result = evaluate_assertion(assertion, status, body, final_url, headers);
        if result.matched {
            return ProbeOutcome::Missing;
        }
    }

    // 4. Indéterminé
    ProbeOutcome::Indeterminate {
        reason: format!("aucune règle déclenchée (status={}, body_len={})", status, body.len()),
    }
}

fn detect_block_reason(
    status: u16,
    body: &str,
    headers: &http::HeaderMap,
) -> BlockReason {
    // Cloudflare
    if status == 403 || headers.get("cf-ray").is_some() {
        return BlockReason::Cloudflare;
    }

    // Captcha
    if body.contains("captcha") || body.contains("CAPTCHA") || body.contains("robot") {
        return BlockReason::Captcha;
    }

    // Rate limit
    if status == 429 || headers.get("retry-after").is_some() {
        return BlockReason::RateLimit;
    }

    // WAF générique
    if status == 403 || status == 406 {
        return BlockReason::Waf;
    }

    BlockReason::Other(format!("status={}", status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_code_assertion() {
        let assertion = Assertion::StatusCode(200);
        let result = evaluate_assertion(&assertion, 200, "", "", &http::HeaderMap::new());
        assert!(result.matched);

        let result = evaluate_assertion(&assertion, 404, "", "", &http::HeaderMap::new());
        assert!(!result.matched);
    }

    #[test]
    fn test_body_contains_assertion() {
        let assertion = Assertion::BodyContains("compte introuvable".to_string());
        let result = evaluate_assertion(&assertion, 200, "compte introuvable", "", &http::HeaderMap::new());
        assert!(result.matched);

        let result = evaluate_assertion(&assertion, 200, "compte trouvé", "", &http::HeaderMap::new());
        assert!(!result.matched);
    }

    #[test]
    fn test_decision_blocked_cloudflare() {
        let rules = DecisionRules {
            exists_if: vec![Assertion::StatusCode(200)],
            missing_if: vec![Assertion::StatusCode(404)],
            blocked_if: vec![Assertion::StatusCode(403)],
        };

        let mut headers = http::HeaderMap::new();
        headers.insert("cf-ray", "test".parse().unwrap());

        let outcome = evaluate_decision(&rules, 403, "", "", &headers);
        assert!(matches!(outcome, ProbeOutcome::Blocked { reason: BlockReason::Cloudflare }));
    }

    #[test]
    fn test_decision_indeterminate() {
        let rules = DecisionRules {
            exists_if: vec![Assertion::StatusCode(200)],
            missing_if: vec![Assertion::StatusCode(404)],
            blocked_if: vec![],
        };

        let outcome = evaluate_decision(&rules, 500, "erreur", "", &http::HeaderMap::new());
        assert!(matches!(outcome, ProbeOutcome::Indeterminate { .. }));
    }
}
