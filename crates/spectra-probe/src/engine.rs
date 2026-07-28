//! Moteur d'exécution de sondes OSINT.
//!
//! Exécution concurrente, rate limiting simple, cache.

use crate::schema::*;
use crate::decide::evaluate_decision;
use crate::cache::ProbeCache;

use dashmap::DashMap;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::{Instant, sleep};
use tracing::debug;

/// Moteur de sondes
pub struct ProbeEngine {
    http: Client,
    global_limit: Arc<Semaphore>,
    per_host_delay: Arc<DashMap<String, Instant>>,
    cache: ProbeCache,
    user_agent: String,
}

impl ProbeEngine {
    /// Crée un nouveau moteur
    pub fn new(user_agent: String, concurrency: usize) -> Self {
        let http = Client::builder()
            .user_agent(&user_agent)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .expect("client HTTP invalide");

        Self {
            http,
            global_limit: Arc::new(Semaphore::new(concurrency)),
            per_host_delay: Arc::new(DashMap::new()),
            cache: ProbeCache::new(),
            user_agent,
        }
    }

    /// Exécute une campagne de sondes
    pub async fn run_campaign(
        &self,
        selector: &str,
        probes: &[Probe],
        _cancel: tokio_util::sync::CancellationToken,
    ) -> Vec<ProbeResult> {
        let mut handles = Vec::new();

        for probe in probes {
            let selector = selector.to_string();
            let probe = probe.clone();
            let permit = self.global_limit.clone();
            let http = self.http.clone();
            let per_host_delay = self.per_host_delay.clone();
            let cache = self.cache.clone();
            let user_agent = self.user_agent.clone();

            let handle = tokio::spawn(async move {
                // Acquérir permit global
                let Ok(_permit) = permit.acquire().await else {
                    return None;
                };

                // Rate limit par hôte : attendre 500ms entre chaque requête au même hôte
                let host = extract_host(&probe.request.url).unwrap_or_else(|| "unknown".to_string());
                let now = Instant::now();

                if let Some(last) = per_host_delay.get(&host) {
                    let elapsed = now.duration_since(*last);
                    if elapsed < Duration::from_millis(500) {
                        sleep(Duration::from_millis(500) - elapsed).await;
                    }
                }
                per_host_delay.insert(host, now);

                // Vérifier cache
                let cache_key = format!("{}:{}", probe.id.0, selector);
                if let Some(cached) = cache.get(&cache_key) {
                    debug!("cache hit pour {}", cache_key);
                    return Some(ProbeResult {
                        probe_id: probe.id.clone(),
                        selector: selector.clone(),
                        outcome: cached,
                        elapsed_ms: 0,
                    });
                }

                // Exécuter la sonde
                let start = Instant::now();
                let outcome = execute_probe(&http, &probe, &selector, &user_agent).await;
                let elapsed = start.elapsed().as_millis() as u64;

                // Mettre en cache
                if let ProbeOutcome::Exists { .. } | ProbeOutcome::Missing = &outcome {
                    cache.insert(&cache_key, outcome.clone(), Duration::from_secs(86400));
                }

                Some(ProbeResult {
                    probe_id: probe.id,
                    selector,
                    outcome,
                    elapsed_ms: elapsed,
                })
            });

            handles.push(handle);
        }

        // Attendre tous les résultats
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Some(result)) => results.push(result),
                Ok(None) => {}
                Err(_) => {}
            }
        }
        results
    }
}

/// Exécute une sonde individuelle
async fn execute_probe(
    http: &Client,
    probe: &Probe,
    selector: &str,
    user_agent: &str,
) -> ProbeOutcome {
    // Construire la requête
    let url = probe.request.url.replace("{selector}", selector);

    let mut req = match probe.request.method {
        Method::Get => http.get(&url),
        Method::Post => http.post(&url),
        Method::Head => http.head(&url),
        Method::Options => http.head(&url),
    };

    // Headers
    for (k, v) in &probe.request.headers {
        req = req.header(k, v);
    }
    req = req.header("User-Agent", user_agent);

    // Corps
    if let Some(body) = &probe.request.body {
        match body {
            BodyTemplate::Json(json) => {
                let json = json.replace("{selector}", selector);
                req = req.header("Content-Type", "application/json").body(json);
            }
            BodyTemplate::Form(form) => {
                let form_data: Vec<(String, String)> = form
                    .iter()
                    .map(|(k, v)| (k.clone(), v.replace("{selector}", selector)))
                    .collect();
                req = req.form(&form_data);
            }
        }
    }

    // Exécuter
    let response = match req.send().await {
        Ok(r) => r,
        Err(e) if e.is_timeout() => return ProbeOutcome::Error(ProbeError::Timeout),
        Err(e) => return ProbeOutcome::Error(ProbeError::Network(e.to_string())),
    };

    let status = response.status().as_u16();
    let final_url = response.url().to_string();
    let headers = response.headers().clone();

    let body = match response.text().await {
        Ok(b) => b,
        Err(e) => return ProbeOutcome::Error(ProbeError::ParseError(e.to_string())),
    };

    // Décider
    evaluate_decision(&probe.decision, status, &body, &final_url, &headers)
}

fn extract_host(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_string()))
}
