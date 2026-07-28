//! Modèle de domaine SPECTRA.
//!
//! # Rôle
//!
//! Ce crate définit ce qu'est une entité, une observation et une projection du
//! graphe à un instant donné. Il est le seul endroit où vit la sémantique
//! probatoire du produit.
//!
//! # Invariants
//!
//! 1. **Aucune I/O.** Pas de réseau, pas de disque, pas d'horloge système lue
//!    implicitement : les horodatages sont fournis par l'appelant. Ce crate doit
//!    être testable intégralement sans environnement.
//! 2. **Aucune dépendance hors `serde` et la stdlib** (CLAUDE.md §4).
//! 3. **Une propriété n'est jamais un fait nu.** Toute valeur portée par une
//!    entité est une [`Observation`] datée, sourcée et qualifiée. Le graphe
//!    affiché est une *projection* des observations à un instant T, jamais un
//!    stockage direct.
//! 4. **Aucune fusion d'entités n'est automatique.** La résolution d'identité
//!    propose, un humain décide, et la fusion reste réversible.

pub mod entity;

pub use entity::{
    AdmiraltyCode, Entity, EntityId, EntityKind, InformationCredibility, Observation,
    ObservationId, PropertyValue, Provenance, SourceReliability,
};
