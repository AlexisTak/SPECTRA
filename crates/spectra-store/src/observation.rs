//! CRUD observations.

use crate::store::{Store, StoreError};
use spectra_core::{AdmiraltyCode, Observation, Provenance};

impl Store {
    /// Insère ou remplace une observation.
    pub fn insert_observation(
        &self, obs: &Observation) -> Result<(), StoreError> {
        let value_json = serde_json::to_string(&obs.value)?;
        let prov_json = serde_json::to_string(&obs.provenance)?;
        let prov_str = prov_json.trim_matches('"');
        let confidence_src = format!("{:?}", obs.confidence.source_reliability);
        let confidence_info = obs.confidence.information_credibility as i64;

        self.conn.execute(
            "INSERT OR REPLACE INTO observations
             (id, subject, predicate, value, source, method, observed_at,
              valid_from, valid_to, confidence_source, confidence_info,
              provenance, raw_hash, operator)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            rusqlite::params![
                &obs.id,
                &obs.subject,
                &obs.predicate,
                &value_json,
                &obs.source,
                &obs.method,
                &obs.observed_at,
                &obs.valid_from,
                &obs.valid_to,
                &confidence_src,
                confidence_info,
                prov_str,
                &obs.raw_hash,
                &obs.operator,
            ],
        )?;
        Ok(())
    }

    /// Récupère les observations d'une entité donnée.
    pub fn get_observations_by_subject(
        &self,
        subject: &str,
    ) -> Result<Vec<Observation>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, subject, predicate, value, source, method, observed_at,
                    valid_from, valid_to, confidence_source, confidence_info,
                    provenance, raw_hash, operator
             FROM observations WHERE subject = ?1 ORDER BY observed_at"
        )?;
        let rows = stmt.query_map([subject], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, i64>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, Option<String>>(12)?,
                row.get::<_, String>(13)?,
            ))
        })?;

        let mut out = Vec::new();
        for r in rows {
            let (
                id, subject_id, predicate, value_json, source, method, observed_at,
                valid_from, valid_to, conf_src, conf_info, prov_str, raw_hash, operator,
            ) = r?;

            let provenance: Provenance = serde_json::from_str(&format!("\"{}\"", prov_str))?;
            let source_reliability: spectra_core::SourceReliability =
                serde_json::from_str(&format!("\"{}\"", conf_src))?;
            let information_credibility = match conf_info {
                1 => spectra_core::InformationCredibility::V1,
                2 => spectra_core::InformationCredibility::V2,
                3 => spectra_core::InformationCredibility::V3,
                4 => spectra_core::InformationCredibility::V4,
                5 => spectra_core::InformationCredibility::V5,
                _ => spectra_core::InformationCredibility::V6,
            };

            out.push(Observation {
                id,
                subject: subject_id,
                predicate,
                value: serde_json::from_str(&value_json)?,
                source,
                method,
                observed_at,
                valid_from,
                valid_to,
                confidence: AdmiraltyCode {
                    source_reliability,
                    information_credibility,
                },
                provenance,
                raw_hash,
                operator,
            });
        }
        Ok(out)
    }

    /// Récupère une observation par son identifiant.
    pub fn get_observation_by_id(
        &self,
        id: &str,
    ) -> Result<Option<Observation>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, subject, predicate, value, source, method, observed_at,
                    valid_from, valid_to, confidence_source, confidence_info,
                    provenance, raw_hash, operator
             FROM observations WHERE id = ?1"
        )?;
        let raw = stmt
            .query_row([id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, i64>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, Option<String>>(12)?,
                    row.get::<_, String>(13)?,
                ))
            })
            .ok();

        let Some((
            id, subject_id, predicate, value_json, source, method, observed_at,
            valid_from, valid_to, conf_src, conf_info, prov_str, raw_hash, operator,
        )) = raw else {
            return Ok(None);
        };

        let provenance: Provenance = serde_json::from_str(&format!("\"{}\"", prov_str))?;
        let source_reliability: spectra_core::SourceReliability =
            serde_json::from_str(&format!("\"{}\"", conf_src))?;
        let information_credibility = match conf_info {
            1 => spectra_core::InformationCredibility::V1,
            2 => spectra_core::InformationCredibility::V2,
            3 => spectra_core::InformationCredibility::V3,
            4 => spectra_core::InformationCredibility::V4,
            5 => spectra_core::InformationCredibility::V5,
            _ => spectra_core::InformationCredibility::V6,
        };

        Ok(Some(Observation {
            id,
            subject: subject_id,
            predicate,
            value: serde_json::from_str(&value_json)?,
            source,
            method,
            observed_at,
            valid_from,
            valid_to,
            confidence: AdmiraltyCode {
                source_reliability,
                information_credibility,
            },
            provenance,
            raw_hash,
            operator,
        }))
    }
}
