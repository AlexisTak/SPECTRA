//! Manifeste de transform — métadonnées et permissions.

use serde::{Deserialize, Serialize};
use spectra_core::EntityKind;
use std::collections::HashSet;

/// Manifeste décrivant un transform (natif ou WASM).
///
/// Les transforms natifs peuvent être déclarés en code ; les transforms WASM
/// doivent fournir un fichier TOML conforme à ce schéma.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformManifest {
    /// Nom technique unique (ex: `com.example.dns-resolver`).
    pub id: String,
    /// Nom lisible pour l'UI.
    pub name: String,
    /// Description courte.
    pub description: String,
    /// Version semver.
    pub version: String,
    /// Auteur et contact.
    pub author: String,
    /// Licence SPDX (ex: "MIT", "Apache-2.0", "AGPL-3.0-only").
    pub license: String,
    /// Types d'entités d'entrée acceptés.
    pub input_kinds: Vec<EntityKind>,
    /// Types d'entités potentiellement produits.
    pub output_kinds: Vec<EntityKind>,
    /// Permissions requises pour s'exécuter.
    pub permissions: TransformPermission,
    /// Quotas d'exécution.
    pub quotas: TransformQuotas,
}

/// Permissions réseau et système demandées par le transform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformPermission {
    /// Accès réseau : domaines explicitement autorisés.
    pub network: NetworkPermission,
}

/// Permission réseau déclarative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPermission {
    /// Liste blanche de domaines (sans protocole ni chemin).
    /// Ex: `["crt.sh", "dns.google"]`.
    pub allowlist: HashSet<String>,
    /// Nombre max de requêtes HTTP autorisées par exécution.
    pub max_requests_per_run: usize,
    /// Taille max de réponse en octets.
    pub max_response_bytes: usize,
}

/// Quotas d'exécution pour protéger l'hôte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformQuotas {
    /// Temps max d'exécution en secondes.
    pub timeout_seconds: u64,
    /// Mémoire max en Mo.
    pub max_memory_mb: usize,
}

impl Default for TransformQuotas {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_memory_mb: 128,
        }
    }
}

impl TransformManifest {
    /// Vérifie qu'un domaine est dans la liste blanche.
    pub fn allows_domain(&self, domain: &str) -> bool {
        self.permissions.network.allowlist.contains(domain)
    }
}
