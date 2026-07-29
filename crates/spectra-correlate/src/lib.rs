//! Moteur de corrélation d'identité.
//!
//! Compare des observations et des entités pour proposer des fusions avec un
//! score et une explication lisible. Le moteur est entièrement déterministe :
//! aucune sortie n'est marquée `Inferred` sans validation humaine.
//!
//! # Heuristiques supportées
//!
//! - Pseudo identique / similaire (Levenshtein)
//! - Propriétés en commun (bio, localisation, etc.)
//! - Avatar (pHash — placeholder pour l'instant, nécessite `image` + `img_hash`)
//! - Chevauchement de fuseau horaire
//! - Email partiellement masqué

pub mod scoring;

pub use scoring::{Correlator, IdentityScore, MatchHint};
