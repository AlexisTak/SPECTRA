//! Test d'intégration du runtime Extism — vérification de la sandbox.
//!
//! # Objectif
//!
//! Prouver qu'un plugin WASM respecte son allowlist et qu'un appel vers un
//! domaine non autorisé est bloqué par l'hôte.

use spectra_plugins::{PluginManifest, PluginTransform};
use spectra_transform::Transform;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Chemin vers le plugin d'exemple compilé.
fn wasm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../plugins/example-transform/target/wasm32-unknown-unknown/release/example_transform.wasm")
}

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../plugins/example-transform/manifest.toml")
}

fn build_entity() -> spectra_core::Entity {
    spectra_core::Entity {
        id: "e1".to_string(),
        kind: spectra_core::EntityKind::Domain,
        canonical_value: "test.com".to_string(),
        display_label: "test.com".to_string(),
        properties: Default::default(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        merged_from: vec![],
    }
}

fn build_ctx() -> spectra_transform::TransformContext {
    spectra_transform::TransformContext {
        case_id: "c1".to_string(),
        network_profile: "direct".to_string(),
        user_agent: "SPECTRA/0.1".to_string(),
        operator: "test".to_string(),
    }
}

#[test]
fn plugin_manifest_parsing() {
    let manifest_toml = std::fs::read_to_string(manifest_path())
        .expect("manifest.toml doit exister");
    let manifest = PluginManifest::from_toml(&manifest_toml)
        .expect("le manifeste doit être valide");

    assert_eq!(manifest.meta.id, "example-transform");
    assert!(manifest.allows_domain("example.com"));
    assert!(!manifest.allows_domain("evil.com"));
}

#[test]
fn plugin_transform_allows_domain() {
    let manifest_toml = std::fs::read_to_string(manifest_path()).unwrap();
    let manifest = PluginManifest::from_toml(&manifest_toml).unwrap();
    let wasm = std::fs::read(wasm_path()).expect("le fichier WASM doit exister");

    let pt = PluginTransform::new(manifest, wasm);

    assert!(pt.allows_domain("example.com"));
    assert!(!pt.allows_domain("evil.com"));
    assert!(!pt.allows_domain("attacker.net"));
}

#[tokio::test]
async fn plugin_transform_executes() {
    let manifest_toml = std::fs::read_to_string(manifest_path()).unwrap();
    let manifest = PluginManifest::from_toml(&manifest_toml).unwrap();
    let wasm = std::fs::read(wasm_path()).expect("le fichier WASM doit exister");

    let pt = PluginTransform::new(manifest, wasm);

    let input = spectra_transform::TransformInput {
        entity: build_entity(),
        params: Default::default(),
    };

    let output = pt.execute(build_ctx(), input)
        .await
        .expect("le plugin doit s'exécuter");
    assert_eq!(output.observations.len(), 1);
}

#[test]
fn plugin_http_allowed_inside_allowlist() {
    let manifest_toml = std::fs::read_to_string(manifest_path()).unwrap();
    let manifest = PluginManifest::from_toml(&manifest_toml).unwrap();
    let wasm = std::fs::read(wasm_path()).expect("le fichier WASM doit exister");

    let pt = PluginTransform::new(manifest, wasm);

    let mut params = BTreeMap::new();
    params.insert("url".to_string(), serde_json::json!("http://example.com/page"));

    let input = spectra_transform::TransformInput {
        entity: build_entity(),
        params,
    };

    let output = pt.execute_export("transform_with_http", build_ctx(), input)
        .expect("l'appel vers example.com doit être autorisé");
    assert_eq!(output.observations.len(), 1);
    let obs = output.observations.first().unwrap();
    let value_json = serde_json::to_string(&obs.value).expect("serialize");
    assert!(value_json.contains("HTTP 200 OK"));
}

#[test]
fn plugin_http_blocked_outside_allowlist() {
    let manifest_toml = std::fs::read_to_string(manifest_path()).unwrap();
    let manifest = PluginManifest::from_toml(&manifest_toml).unwrap();
    let wasm = std::fs::read(wasm_path()).expect("le fichier WASM doit exister");

    let pt = PluginTransform::new(manifest, wasm);

    let mut params = BTreeMap::new();
    params.insert("url".to_string(), serde_json::json!("http://evil.com/secret"));

    let input = spectra_transform::TransformInput {
        entity: build_entity(),
        params,
    };

    let result = pt.execute_export("transform_with_http", build_ctx(), input);
    assert!(result.is_err(), "un appel vers evil.com doit être bloqué");
    // La host function lève une erreur qui se traduit par une trap WASM ;
    // l'important est que le résultat soit une erreur alors que example.com passe.
}
