//! Manifeste de plugin — métadonnées et permissions au format TOML.

use serde::{Deserialize, Serialize};
use spectra_core::EntityKind;
use std::collections::HashSet;

/// Manifeste d'un plugin WASM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Informations générales.
    pub meta: Meta,
    /// Entrées/sorties supportées.
    pub transforms: Vec<TransformSpec>,
    /// Permissions réseau.
    pub permissions: Permissions,
    /// Clé publique Ed25519 attendue (hex, 64 caractères).
    pub expected_public_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub license: String,
}

/// Spécification d'un transform exposé par le plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    pub input_kinds: Vec<EntityKind>,
    pub output_kinds: Vec<EntityKind>,
}

/// Permissions demandées par le plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    pub network: NetworkPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPermissions {
    pub allowlist: HashSet<String>,
    pub max_requests_per_run: usize,
    pub max_response_bytes: usize,
}

impl PluginManifest {
    /// Charge un manifeste depuis une chaîne TOML.
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Vérifie qu'un domaine est dans la liste blanche.
    pub fn allows_domain(&self, domain: &str) -> bool {
        self.permissions.network.allowlist.contains(domain)
    }
}
