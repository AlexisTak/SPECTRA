//! Score de qualité persistant des sondes.
//!
//! Chaque sonde accumule un historique local (data/probe-health.db) :
//! taux de succès du contrôle, taux de blocage, latence médiane,
//! date de dernière vérification réussie.

use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::schema::{ProbeId, ProbeQuality, ConfidenceLevel};

/// Base de données de santé des sondes
pub struct ProbeHealthDb {
    conn: Arc<Mutex<Connection>>,
}

impl ProbeHealthDb {
    pub fn open(path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS probe_health (
                probe_id TEXT PRIMARY KEY,
                control_success_rate REAL NOT NULL DEFAULT 0.0,
                block_rate REAL NOT NULL DEFAULT 0.0,
                median_latency_ms INTEGER NOT NULL DEFAULT 0,
                last_success TEXT,
                confidence TEXT NOT NULL DEFAULT 'unknown',
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn get(&self, probe_id: &ProbeId) -> Result<ProbeQuality, rusqlite::Error> {
        let conn = self.conn.lock().await;

        let quality = conn.query_row(
            "SELECT control_success_rate, block_rate, median_latency_ms, last_success, confidence
             FROM probe_health WHERE probe_id = ?",
            params![&probe_id.0],
            |row| {
                let rate: f64 = row.get(0)?;
                let block: f64 = row.get(1)?;
                let latency: i64 = row.get(2)?;
                let last_success: Option<String> = row.get(3)?;
                let confidence: String = row.get(4)?;

                let confidence = match confidence.as_str() {
                    "high" => ConfidenceLevel::High,
                    "medium" => ConfidenceLevel::Medium,
                    "low" => ConfidenceLevel::Low,
                    "degraded" => ConfidenceLevel::Degraded,
                    _ => ConfidenceLevel::Unknown,
                };

                Ok(ProbeQuality {
                    control_success_rate: rate,
                    block_rate: block,
                    median_latency_ms: latency as u64,
                    last_success: last_success.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&chrono::Utc))),
                    confidence,
                })
            },
        );

        match quality {
            Ok(q) => Ok(q),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(ProbeQuality::default()),
            Err(e) => Err(e),
        }
    }

    pub async fn update(&self, probe_id: &ProbeId, quality: &ProbeQuality) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().await;

        let confidence_str = match quality.confidence {
            ConfidenceLevel::High => "high",
            ConfidenceLevel::Medium => "medium",
            ConfidenceLevel::Low => "low",
            ConfidenceLevel::Degraded => "degraded",
            ConfidenceLevel::Unknown => "unknown",
        };

        let last_success = quality.last_success.map(|d| d.to_rfc3339());

        conn.execute(
            "INSERT OR REPLACE INTO probe_health
             (probe_id, control_success_rate, block_rate, median_latency_ms, last_success, confidence, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now'))",
            params![
                &probe_id.0,
                quality.control_success_rate,
                quality.block_rate,
                &(quality.median_latency_ms as i64),
                last_success,
                confidence_str,
            ],
        )?;

        Ok(())
    }

    pub async fn get_all(&self) -> Result<HashMap<ProbeId, ProbeQuality>, rusqlite::Error> {
        let conn = self.conn.lock().await;

        let mut stmt = conn.prepare(
            "SELECT probe_id, control_success_rate, block_rate, median_latency_ms, last_success, confidence
             FROM probe_health"
        )?;

        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let rate: f64 = row.get(1)?;
            let block: f64 = row.get(2)?;
            let latency: i64 = row.get(3)?;
            let last_success: Option<String> = row.get(4)?;
            let confidence: String = row.get(5)?;

            let confidence = match confidence.as_str() {
                "high" => ConfidenceLevel::High,
                "medium" => ConfidenceLevel::Medium,
                "low" => ConfidenceLevel::Low,
                "degraded" => ConfidenceLevel::Degraded,
                _ => ConfidenceLevel::Unknown,
            };

            Ok((
                ProbeId(id),
                ProbeQuality {
                    control_success_rate: rate,
                    block_rate: block,
                    median_latency_ms: latency as u64,
                    last_success: last_success.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&chrono::Utc))),
                    confidence,
                },
            ))
        })?;

        let mut result = HashMap::new();
        for row in rows {
            if let Ok((id, quality)) = row {
                result.insert(id, quality);
            }
        }

        Ok(result)
    }
}
