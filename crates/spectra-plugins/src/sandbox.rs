//! Sandbox et host functions réseau contrôlées.
//!
//! Ce module définit les fonctions hôte exposées aux plugins WASM. Tout accès
//! réseau passe par l'hôte qui applique :
//! - allowlist de domaines déclarés au manifeste,
//! - rate limiting,
//! - timeout,
//! - taille max de réponse.

use spectra_transform::TransformError;
use std::collections::HashSet;

/// Politique réseau appliquée par les host functions.
#[derive(Debug, Clone)]
pub struct NetworkPolicy {
    /// Domaines autorisés, issus du manifeste du plugin.
    pub allowlist: HashSet<String>,
    /// Nombre maximal de requêtes par exécution.
    pub max_requests_per_run: usize,
    /// Taille maximale d'une réponse acceptée, en octets.
    pub max_response_bytes: usize,
    /// Délai d'expiration appliqué à chaque requête, en secondes.
    pub timeout_seconds: u64,
}

impl NetworkPolicy {
    /// Vérifie qu'un domaine est autorisé.
    pub fn check_domain(&self, domain: &str) -> Result<(), TransformError> {
        if self.allowlist.contains(domain) {
            Ok(())
        } else {
            Err(TransformError::SandboxViolation(format!(
                "domaine '{domain}' hors allowlist"
            )))
        }
    }
}

/// Host function `http_get` contrôlée.
///
/// À enregistrer dans le runtime Extism via `extism::host_fn!`.
/// Pour l'instant, la définition reste ici en attendant le PDK côté plugin.
pub fn http_get_allowed(
    _url: &str,
    _policy: &NetworkPolicy,
) -> Result<Vec<u8>, TransformError> {
    // Implémentation réelle : parser l'URL, vérifier le domaine contre la
    // policy, effectuer la requête HTTP via reqwest avec timeout et rate
    // limiting, et renvoyer les bytes.
    todo!("host function http_get à implémenter avec reqwest + governor")
}
