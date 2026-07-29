//! Journal d'audit persisté en SQLite.

use crate::store::{Store, StoreError};
use spectra_audit::chain::{compute_link, AuditEvent, ChainLink};

/// Événement d'audit stocké.
#[derive(Debug, Clone)]
pub struct StoredAuditEvent {
    /// Identifiant unique.
    pub id: String,
    /// Identifiant du dossier.
    pub case_id: String,
    /// Action.
    pub action: String,
    /// Type d'entité.
    pub entity_kind: String,
    /// Identifiant de l'entité (optionnel).
    pub entity_id: Option<String>,
    /// Acteur.
    pub actor: String,
    /// Numéro de séquence.
    pub sequence: u64,
    /// Payload JSON.
    pub payload: serde_json::Value,
    /// Hachage du maillon.
    pub hash: String,
    /// Hachage du maillon précédent.
    pub previous_hash: String,
    /// Horodatage ISO 8601.
    pub timestamp: String,
}

impl Store {
    /// Ajoute un événement d'audit à la chaîne du dossier.
    ///
    /// La méthode récupère automatiquement le `previous_hash` du dernier
    /// événement du dossier et calcule le nouveau maillon.
    pub fn append_audit_event(
        &self,
        event: &AuditEvent,
    ) -> Result<ChainLink, StoreError> {
        let previous_hash = self.last_audit_hash(&event.case_id)?;
        let link = compute_link(event, &previous_hash);

        self.conn.execute(
            "INSERT INTO audit_log
             (id, case_id, action, entity_kind, entity_id, actor, sequence, payload, hash, previous_hash, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (
                &event.id,
                &event.case_id,
                &event.action,
                &event.entity_kind,
                &event.entity_id,
                &event.actor,
                event.sequence as i64,
                serde_json::to_string(&event.payload)?,
                &link.hash,
                &link.previous_hash,
                chrono::Utc::now().to_rfc3339(),
            ),
        )?;

        Ok(link)
    }

    /// Récupère tous les événements d'audit d'un dossier, ordonnés par séquence.
    pub fn list_audit_events(
        &self,
        case_id: &str,
    ) -> Result<Vec<StoredAuditEvent>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, case_id, action, entity_kind, entity_id, actor, sequence, payload, hash, previous_hash, timestamp
             FROM audit_log
             WHERE case_id = ?1
             ORDER BY sequence ASC"
        )?;

        let rows = stmt.query_map([case_id], |row| {
            let payload_str: String = row.get(7)?;
            let payload = serde_json::from_str(&payload_str).unwrap_or(serde_json::Value::Null);
            Ok(StoredAuditEvent {
                id: row.get(0)?,
                case_id: row.get(1)?,
                action: row.get(2)?,
                entity_kind: row.get(3)?,
                entity_id: row.get(4)?,
                actor: row.get(5)?,
                sequence: {
                    let seq: i64 = row.get(6)?;
                    seq as u64
                },
                payload,
                hash: row.get(8)?,
                previous_hash: row.get(9)?,
                timestamp: row.get(10)?,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        Ok(events)
    }

    /// Retourne le hash du dernier événement d'audit pour un dossier, ou une
    /// chaîne vide si aucun événement n'existe.
    fn last_audit_hash(&self, case_id: &str) -> Result<String, StoreError> {
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT hash FROM audit_log WHERE case_id = ?1 ORDER BY sequence DESC LIMIT 1",
                [case_id],
                |row| row.get(0),
            )
            .ok();
        Ok(result.unwrap_or_default())
    }
}
