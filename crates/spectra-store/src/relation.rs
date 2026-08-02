//! CRUD relations.

use crate::store::{Store, StoreError};
use std::collections::BTreeMap;

/// Une relation orientée entre deux entités.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredRelation {
    /// Identifiant en base, absent tant que la relation n'est pas insérée.
    pub id: Option<i64>,
    /// Identifiant de l'entité d'origine.
    pub source: String,
    /// Identifiant de l'entité cible.
    pub target: String,
    /// Nature du lien (`owns`, `mentions`, `resolves_to`…).
    pub kind: String,
    /// Propriétés libres attachées au lien.
    pub properties: BTreeMap<String, serde_json::Value>,
}

impl Store {
    /// Insère une relation.
    pub fn insert_relation(
        &self,
        rel: &StoredRelation,
    ) -> Result<(), StoreError> {
        let props = serde_json::to_string(&rel.properties)?;
        self.conn.execute(
            "INSERT INTO relations (source, target, kind, properties)
             VALUES (?1, ?2, ?3, ?4)",
            [
                &rel.source,
                &rel.target,
                &rel.kind,
                &props,
            ],
        )?;
        Ok(())
    }

    /// Récupère les relations sortantes d'une entité.
    pub fn get_relations_from(
        &self,
        source: &str,
    ) -> Result<Vec<StoredRelation>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source, target, kind, properties
             FROM relations WHERE source = ?1"
        )?;
        let rows = stmt.query_map([source], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        let mut out = Vec::new();
        for r in rows {
            let (id, src, tgt, kind, props) = r?;
            out.push(StoredRelation {
                id: Some(id),
                source: src,
                target: tgt,
                kind,
                properties: serde_json::from_str(&props)?,
            });
        }
        Ok(out)
    }

    /// Récupère les relations entrantes d'une entité.
    pub fn get_relations_to(
        &self,
        target: &str,
    ) -> Result<Vec<StoredRelation>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source, target, kind, properties
             FROM relations WHERE target = ?1"
        )?;
        let rows = stmt.query_map([target], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        let mut out = Vec::new();
        for r in rows {
            let (id, src, tgt, kind, props) = r?;
            out.push(StoredRelation {
                id: Some(id),
                source: src,
                target: tgt,
                kind,
                properties: serde_json::from_str(&props)?,
            });
        }
        Ok(out)
    }
}
