//! Cache de réponses de transforms.
//!
//! Évite de re-interroger une source quand le résultat est déjà connu et n'a
//! pas expiré. La clé de cache est un hash du : transform id + entité
//! d'entrée (canonical_value) + paramètres.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

/// Clé de cache : identifie de manière unique un appel à transform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub transform_id: String,
    pub entity_kind: String,
    pub canonical_value: String,
    pub params_hash: String,
}

/// Entrée de cache avec TTL.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub output_json: String,
    pub cached_at: Instant,
    pub ttl: Duration,
}

/// Cache thread-safe en mémoire.
pub struct ResponseCache {
    inner: DashMap<CacheKey, CacheEntry>,
    default_ttl: Duration,
}

impl ResponseCache {
    /// Crée un nouveau cache avec une durée de vie par défaut.
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            inner: DashMap::new(),
            default_ttl,
        }
    }

    /// Insère une entrée dans le cache.
    pub fn insert(&self,
        key: CacheKey,
        output_json: String,
        ttl: Option<Duration>,
    ) {
        self.inner.insert(key, CacheEntry {
            output_json,
            cached_at: Instant::now(),
            ttl: ttl.unwrap_or(self.default_ttl),
        });
    }

    /// Récupère une entrée si elle existe et n'a pas expiré.
    pub fn get(&self, key: &CacheKey) -> Option<String> {
        let entry = self.inner.get(key)?;
        if entry.cached_at.elapsed() < entry.ttl {
            Some(entry.output_json.clone())
        } else {
            drop(entry);
            self.inner.remove(key);
            None
        }
    }

    /// Invalide toutes les entrées d'un transform donné.
    pub fn invalidate_transform(&self, transform_id: &str,
    ) {
        self.inner.retain(|k, _| k.transform_id != transform_id);
    }

    /// Vide le cache.
    pub fn clear(&self) {
        self.inner.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_hit_and_miss() {
        let cache = ResponseCache::new(Duration::from_secs(60));
        let key = CacheKey {
            transform_id: "dns:a".to_string(),
            entity_kind: "Domain".to_string(),
            canonical_value: "example.com".to_string(),
            params_hash: "default".to_string(),
        };

        cache.insert(key.clone(), "{\"result\":\"ok\"}".to_string(), None);
        assert_eq!(cache.get(&key), Some("{\"result\":\"ok\"}".to_string()));

        cache.clear();
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn cache_expiration() {
        let cache = ResponseCache::new(Duration::from_secs(60));
        let key = CacheKey {
            transform_id: "dns:a".to_string(),
            entity_kind: "Domain".to_string(),
            canonical_value: "example.com".to_string(),
            params_hash: "default".to_string(),
        };

        cache.insert(key.clone(), "old".to_string(), Some(Duration::from_nanos(1)));
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(cache.get(&key), None);
    }
}
