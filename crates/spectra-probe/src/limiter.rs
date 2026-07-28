//! Rate limiting par hôte — version simplifiée sans governor.

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Instant, Duration};

/// Limiteur par hôte — attend 500ms entre chaque requête
pub struct HostLimiter {
    last_request: Arc<Mutex<Option<Instant>>>,
    min_interval: Duration,
}

impl HostLimiter {
    pub fn new(min_interval_ms: u64) -> Self {
        Self {
            last_request: Arc::new(Mutex::new(None)),
            min_interval: Duration::from_millis(min_interval_ms),
        }
    }

    pub async fn wait(&self) {
        let mut last = self.last_request.lock().await;
        if let Some(instant) = *last {
            let elapsed = instant.elapsed();
            if elapsed < self.min_interval {
                tokio::time::sleep(self.min_interval - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }
}
