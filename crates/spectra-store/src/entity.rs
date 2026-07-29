//! CRUD entités.

use crate::store::{Store, StoreError};
use spectra_core::Entity;

impl Store {
    /// Insère ou remplace une entité.
    pub fn insert_entity(&self, entity: &Entity) -> Result<(), StoreError> {
        let props = serde_json::to_string(&entity.properties)?;
        let merged = serde_json::to_string(&entity.merged_from)?;
        let kind_json = serde_json::to_string(&entity.kind)?;
        let kind_str = kind_json.trim_matches('"');
        self.conn.execute(
            "INSERT OR REPLACE INTO entities
             (id, kind, canonical_value, display_label, properties, created_at, merged_from)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [
                &entity.id,
                kind_str,
                &entity.canonical_value,
                &entity.display_label,
                &props,
                &entity.created_at,
                &merged,
            ],
        )?;
        Ok(())
    }

    /// Récupère une entité par son identifiant.
    pub fn get_entity(&self, id: &str) -> Result<Option<Entity>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, canonical_value, display_label, properties, created_at, merged_from
             FROM entities WHERE id = ?1"
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
                ))
            })
            .ok();

        let Some((id, kind_str, canonical, label, props, created, merged)) = raw else {
            return Ok(None);
        };

        let kind = serde_json::from_str(&format!("\"{}\"", kind_str))?;
        let properties = serde_json::from_str(&props)?;
        let merged_from = serde_json::from_str(&merged)?;

        Ok(Some(Entity {
            id,
            kind,
            canonical_value: canonical,
            display_label: label,
            properties,
            created_at: created,
            merged_from,
        }))
    }

    /// Liste toutes les entités du dossier.
    pub fn list_entities(&self) -> Result<Vec<Entity>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, canonical_value, display_label, properties, created_at, merged_from
             FROM entities ORDER BY created_at"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;

        let mut out = Vec::new();
        for r in rows {
            let (id, kind_str, canonical, label, props, created, merged) = r?;
            let kind = serde_json::from_str(&format!("\"{}\"", kind_str))?;
            let properties = serde_json::from_str(&props)?;
            let merged_from = serde_json::from_str(&merged)?;
            out.push(Entity {
                id,
                kind,
                canonical_value: canonical,
                display_label: label,
                properties,
                created_at: created,
                merged_from,
            });
        }
        Ok(out)
    }
}
