//! Host functions exposées aux plugins WASM.
//!
//! Tout accès réseau passe par ici. L'hôte applique l'allowlist du manifeste
//! et les quotas avant d'exécuter la moindre requête.

use extism::{host_fn, Error};
use std::collections::HashSet;
use std::sync::Arc;

/// Données utilisateur pour `spectra_http_get` : l'allowlist de domaines.
pub type AllowlistUserData = Arc<HashSet<String>>;

host_fn!(pub spectra_http_get(user_data: AllowlistUserData; url: String) -> String {
    let allowed = user_data.get()?;
    let allowed = allowed.lock().map_err(|e| Error::msg(format!("mutex poison: {e}")))?;
    let domain = extract_domain(&url);
    if !allowed.contains(&domain) {
        return Err(Error::msg(format!(
            "sandbox violation: domain '{domain}' not in allowlist"
        )));
    }
    // Dans un vrai scénario on ferait la requête HTTP ici.
    // Pour le moment on retourne un stub qui prouve que l'appel a été autorisé.
    Ok(format!("HTTP 200 OK {url}"))
});

fn extract_domain(url: &str) -> String {
    let mut rest = url.trim_start_matches("http://").trim_start_matches("https://");
    if let Some(idx) = rest.find('/') {
        rest = &rest[..idx];
    }
    if let Some(idx) = rest.find(':') {
        rest = &rest[..idx];
    }
    rest.to_lowercase()
}
