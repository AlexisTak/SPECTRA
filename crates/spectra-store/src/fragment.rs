//! Export et import de fragments de dossier.
//!
//! Permet l'échange de sous-ensembles d'entités et d'observations entre
//! analystes sans serveur central — conformément à la contrainte C4 (données
//! locales, cloisonnées par dossier).

use crate::store::{Store, StoreError};
use crate::StoredRelation;
use serde::{Deserialize, Serialize};
use spectra_core::{Entity, Observation};
use std::collections::BTreeSet;

/// Fragment exportable d'un dossier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fragment {
    /// Version du format de fragment, pour la compatibilité ascendante.
    pub version: u32,
    /// Date d'export (RFC 3339).
    pub exported_at: String,
    /// Analyste ayant produit le fragment.
    pub exported_by: String,
    /// Entités incluses dans le fragment.
    pub entities: Vec<Entity>,
    /// Observations rattachées aux entités incluses.
    pub observations: Vec<Observation>,
    /// Relations dont la source ou la cible fait partie du fragment.
    pub relations: Vec<StoredRelation>,
}

impl Store {
    /// Exporte un fragment contenant les entités listées, leurs observations
    /// et les relations les concernant (source ou target).
    pub fn export_fragment(
        &self,
        entity_ids: &[String],
        exported_by: &str,
    ) -> Result<Fragment, StoreError> {
        let mut entities = Vec::new();
        let mut observations = Vec::new();
        let mut relations = Vec::new();
        let mut ids: BTreeSet<String> = entity_ids.iter().cloned().collect();

        for id in entity_ids {
            if let Some(e) = self.get_entity(id)? {
                entities.push(e);
                for o in self.get_observations_by_subject(id)? {
                    observations.push(o);
                }
                for r in self.get_relations_from(id)? {
                    ids.insert(r.target.clone());
                    relations.push(r);
                }
                for r in self.get_relations_to(id)? {
                    ids.insert(r.source.clone());
                    relations.push(r);
                }
            }
        }

        // Récupérer les entités manquantes (voisinage)
        for id in &ids {
            if entities.iter().any(|e| &e.id == id) {
                continue;
            }
            if let Some(e) = self.get_entity(id)? {
                entities.push(e);
            }
        }

        Ok(Fragment {
            version: 1,
            exported_at: chrono::Utc::now().to_rfc3339(),
            exported_by: exported_by.to_string(),
            entities,
            observations,
            relations,
        })
    }

    /// Importe un fragment en fusionnant avec le dossier courant via CRDT.
    ///
    /// Stratégie de fusion :
    /// - **Entités** : si l'ID existe, on fusionne les `merged_from` (G-Set) et
    ///   on garde les propriétés du fragment si `created_at` est plus récent.
    /// - **Observations** : LWW par `observed_at` sur `(subject, predicate, value)`.
    ///   Une observation avec un `observed_at` plus récent remplace l'ancienne
    ///   portant le même `id`.
    /// - **Relations** : si `(source, target, kind)` existe déjà, on ignore ;
    ///   sinon on insère.
    pub fn import_fragment(&self,
        fragment: &Fragment,
    ) -> Result<ImportSummary, StoreError> {
        let mut summary = ImportSummary {
            entities_inserted: 0,
            entities_skipped: 0,
            entities_merged: 0,
            observations_inserted: 0,
            observations_skipped: 0,
            observations_updated: 0,
            relations_inserted: 0,
            relations_skipped: 0,
        };

        for e in &fragment.entities {
            match self.get_entity(&e.id)? {
                Some(existing) => {
                    // Fusion CRDT : union des merged_from (G-Set) + propriétés LWW
                    let mut new_mf: BTreeSet<String> = existing.merged_from.iter().cloned().collect();
                    let mf_grew = e.merged_from.iter().any(|m| !new_mf.contains(m));
                    for m in &e.merged_from {
                        new_mf.insert(m.clone());
                    }
                    let newer = e.created_at > existing.created_at;

                    if !newer && !mf_grew {
                        // Aucun changement apporté par le fragment
                        summary.entities_skipped += 1;
                    } else {
                        let mut merged = e.clone();
                        merged.merged_from = new_mf.into_iter().collect();
                        if !newer {
                            // On garde les propriétés existantes car elles sont plus récentes
                            merged.properties = existing.properties.clone();
                        }
                        self.insert_entity(&merged)?; // INSERT OR REPLACE fait l'update
                        summary.entities_merged += 1;
                    }
                }
                None => {
                    self.insert_entity(e)?;
                    summary.entities_inserted += 1;
                }
            }
        }

        for o in &fragment.observations {
            let existing = self.get_observation_by_id(&o.id)?;
            match existing {
                Some(prev) => {
                    if o.observed_at > prev.observed_at {
                        self.insert_observation(o)?;
                        summary.observations_updated += 1;
                    } else {
                        summary.observations_skipped += 1;
                    }
                }
                None => {
                    self.insert_observation(o)?;
                    summary.observations_inserted += 1;
                }
            }
        }

        for r in &fragment.relations {
            let from = self.get_relations_from(&r.source)?;
            if from.iter().any(|rel| rel.target == r.target && rel.kind == r.kind) {
                summary.relations_skipped += 1;
            } else {
                self.insert_relation(r)?;
                summary.relations_inserted += 1;
            }
        }

        Ok(summary)
    }
}

/// Compte-rendu d'un import.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ImportSummary {
    /// Entités absentes du dossier et créées à l'import.
    pub entities_inserted: usize,
    /// Entités laissées inchangées, le fragment n'apportant rien de plus récent.
    pub entities_skipped: usize,
    /// Entités existantes mises à jour par fusion CRDT.
    pub entities_merged: usize,
    /// Observations créées.
    pub observations_inserted: usize,
    /// Observations ignorées car déjà connues et plus récentes.
    pub observations_skipped: usize,
    /// Observations remplacées par une version plus récente.
    pub observations_updated: usize,
    /// Relations créées.
    pub relations_inserted: usize,
    /// Relations déjà présentes, ignorées.
    pub relations_skipped: usize,
}
