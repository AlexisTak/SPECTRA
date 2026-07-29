//! Transform WASM d'exemple — renvoie l'entité d'entrée avec une observation
//! fictive pour démontrer le runtime Extism, et expose un second export qui
//! effectue un appel HTTP contrôlé via host function.

use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TransformPayload {
    context: serde_json::Value,
    input: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct TransformResult {
    entities: Vec<serde_json::Value>,
    observations: Vec<serde_json::Value>,
    relations: Vec<serde_json::Value>,
}

#[host_fn("extism:host/user")]
extern "ExtismHost" {
    fn spectra_http_get(url: String) -> String;
}

/// Point d'entrée classique — sans appel réseau.
#[plugin_fn]
pub fn transform(input: String) -> FnResult<String> {
    let payload: TransformPayload = serde_json::from_str(&input)
        .map_err(|e| Error::msg(format!("parse: {e}")))?;

    let result = TransformResult {
        entities: vec![],
        observations: vec![serde_json::json!({
            "id": "obs-example-001",
            "subject": payload.input.get("entity").and_then(|e| e.get("id")).unwrap_or(&serde_json::Value::Null),
            "predicate": "example_transform_ran",
            "value": { "status": "ok" },
            "source": "example-transform",
            "method": "WASM_EXEC",
            "observed_at": "2026-07-29T00:00:00Z",
            "confidence": {
                "source_reliability": "C",
                "information_credibility": "3"
            },
            "provenance": "collected",
            "operator": "SPECTRA"
        })],
        relations: vec![],
    };

    Ok(serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()))
}

/// Point d'entrée avec appel HTTP — démontre le contrôle d'allowlist.
#[plugin_fn]
pub fn transform_with_http(input: String) -> FnResult<String> {
    let payload: TransformPayload = serde_json::from_str(&input)
        .map_err(|e| Error::msg(format!("parse: {e}")))?;

    let url = payload
        .input
        .get("params")
        .and_then(|p| p.get("url"))
        .and_then(|u| u.as_str())
        .unwrap_or("http://example.com/default");

    let response = unsafe { spectra_http_get(url.to_string())? };

    let result = TransformResult {
        entities: vec![],
        observations: vec![serde_json::json!({
            "id": "obs-example-http-001",
            "subject": payload.input.get("entity").and_then(|e| e.get("id")).unwrap_or(&serde_json::Value::Null),
            "predicate": "http_response",
            "value": response,
            "source": "example-transform",
            "method": "WASM_HTTP",
            "observed_at": "2026-07-29T00:00:00Z",
            "confidence": {
                "source_reliability": "C",
                "information_credibility": "3"
            },
            "provenance": "collected",
            "operator": "SPECTRA"
        })],
        relations: vec![],
    };

    Ok(serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()))
}
