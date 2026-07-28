//! Sonde de contrôle — anti-faux-positifs.
//!
//! Avant chaque campagne, exécute chaque sonde avec un sélecteur de contrôle
//! (chaîne aléatoire qui n'existe statistiquement nulle part).
//!
//! Contrôle → Missing : la règle fonctionne. ✅
//! Contrôle → Exists : la règle est cassée, sonde marquée Degraded.
//! Contrôle → Blocked : le site nous bloque.

use crate::schema::{Probe, ProbeOutcome, ProbeQuality, ConfidenceLevel, ProbeResult};
use rand::Rng;
use std::time::Duration;

/// Génère un sélecteur de contrôle aléatoire de même forme
pub fn generate_control_selector(kind: &crate::schema::SelectorKind) -> String {
    let mut rng = rand::rng();
    match kind {
        crate::schema::SelectorKind::Username => {
            // Chaîne aléatoire de 12 caractères alphanumériques
            (0..12)
                .map(|_| {
                    let idx = rng.random_range(0..62);
                    match idx {
                        0..=25 => (b'a' + idx) as char,
                        26..=51 => (b'A' + idx - 26) as char,
                        _ => (b'0' + idx - 52) as char,
                    }
                })
                .collect()
        }
        crate::schema::SelectorKind::Email => {
            format!(
                "{}@example.com",
                (0..10)
                    .map(|_| {
                        let idx = rng.random_range(0..36);
                        match idx {
                            0..=25 => (b'a' + idx) as char,
                            _ => (b'0' + idx - 26) as char,
                        }
                    })
                    .collect::<String>()
            )
        }
        crate::schema::SelectorKind::Phone => {
            format!(
                "+336{:08}",
                rng.random_range(0..100_000_000)
            )
        }
    }
}

/// Exécute la sonde de contrôle
pub async fn run_control_probe(
    probe: &Probe,
    engine: &crate::engine::ProbeEngine,
) -> ControlResult {
    let control_selector = generate_control_selector(&probe.selector_kind);
    let cancel = tokio_util::sync::CancellationToken::new();

    let start = std::time::Instant::now();

    // Exécuter la campagne (une seule sonde)
    let results: Vec<ProbeResult> = engine.run_campaign(&control_selector, &[probe.clone()], cancel).await;

    let elapsed = start.elapsed();

    if results.is_empty() {
        return ControlResult {
            probe_id: probe.id.clone(),
            status: ControlStatus::Error("aucun résultat".to_string()),
            elapsed,
        };
    }

    let outcome = &results[0].outcome;

    let status = match outcome {
        ProbeOutcome::Missing => ControlStatus::Healthy,
        ProbeOutcome::Exists { .. } => ControlStatus::Degraded {
            reason: "la sonde retourne 'existe' pour un sélecteur aléatoire".to_string(),
        },
        ProbeOutcome::Blocked { reason } => ControlStatus::Blocked {
            reason: reason.clone(),
        },
        ProbeOutcome::Indeterminate { reason } => ControlStatus::Indeterminate {
            reason: reason.clone(),
        },
        ProbeOutcome::Error(e) => ControlStatus::Error(format!("{:?}", e)),
    };

    ControlResult {
        probe_id: probe.id.clone(),
        status,
        elapsed,
    }
}

#[derive(Debug, Clone)]
pub struct ControlResult {
    pub probe_id: crate::schema::ProbeId,
    pub status: ControlStatus,
    pub elapsed: Duration,
}

#[derive(Debug, Clone)]
pub enum ControlStatus {
    Healthy,
    Degraded { reason: String },
    Blocked { reason: crate::schema::BlockReason },
    Indeterminate { reason: String },
    Error(String),
}

/// Met à jour la qualité d'une sonde en fonction du résultat de contrôle
pub fn update_probe_quality(quality: &mut ProbeQuality, result: &ControlResult) {
    match result.status {
        ControlStatus::Healthy => {
            quality.control_success_rate = (quality.control_success_rate * 10.0 + 1.0) / 11.0;
            quality.confidence = ConfidenceLevel::High;
        }
        ControlStatus::Degraded { .. } => {
            quality.control_success_rate = (quality.control_success_rate * 10.0) / 11.0;
            quality.confidence = ConfidenceLevel::Degraded;
        }
        ControlStatus::Blocked { .. } => {
            quality.block_rate = (quality.block_rate * 10.0 + 1.0) / 11.0;
        }
        _ => {}
    }
    quality.last_success = Some(chrono::Utc::now());
}
