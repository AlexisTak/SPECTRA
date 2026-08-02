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

/// Métadonnées d'identification du plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    /// Identifiant unique du plugin (utilisé comme nom de dossier).
    pub id: String,
    /// Nom lisible affiché dans l'UI.
    pub name: String,
    /// Version sémantique du plugin.
    pub version: String,
    /// Auteur ou organisation responsable.
    pub author: String,
    /// Licence du plugin, vérifiée à l'installation.
    pub license: String,
}

/// Spécification d'un transform exposé par le plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformSpec {
    /// Identifiant du transform, unique au sein du plugin.
    pub id: String,
    /// Nom lisible affiché dans le menu d'expansion.
    pub name: String,
    /// Description de ce que le transform collecte.
    pub description: String,
    /// Types d'entités acceptés en entrée.
    pub input_kinds: Vec<EntityKind>,
    /// Types d'entités susceptibles d'être produits.
    pub output_kinds: Vec<EntityKind>,
}

/// Permissions demandées par le plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    /// Permissions réseau, seul canal de sortie offert au plugin.
    pub network: NetworkPermissions,
}

/// Permissions réseau déclarées au manifeste et imposées par l'hôte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPermissions {
    /// Domaines joignables. Tout autre domaine est refusé par l'hôte.
    pub allowlist: HashSet<String>,
    /// Nombre maximal de requêtes autorisées par exécution.
    pub max_requests_per_run: usize,
    /// Taille maximale d'une réponse acceptée, en octets.
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
