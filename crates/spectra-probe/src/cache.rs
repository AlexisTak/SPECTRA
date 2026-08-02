//! Cache des résultats de sondes.
//!
//! Clé : (probe_id, hash(selector))
//! TTL : 24h par défaut, 1h pour les sites volatils

use crate::schema::ProbeOutcome;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct CacheEntry {
    outcome: ProbeOutcome,
    expires_at: Instant,
}

#[derive(Clone)]
pub struct ProbeCache {
    data: Arc<DashMap<String, CacheEntry>>,
}

impl ProbeCache {
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<ProbeOutcome> {
        let entry = self.data.get(key)?;
        if entry.expires_at < Instant::now() {
            self.data.remove(key);
            None
        } else {
            Some(entry.outcome.clone())
        }
    }

    pub fn insert(&self, key: &str, outcome: ProbeOutcome, ttl: Duration) {
        let expires_at = Instant::now() + ttl;
        self.data.insert(key.to_string(), CacheEntry { outcome, expires_at });
    }

    pub fn clear(&self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl Default for ProbeCache {
    fn default() -> Self {
        Self::new()
    }
}
