//! Moteur d'exécution de sondes OSINT.
//!
//! # Principe
//!
//! Un moteur d'exécution de sondes en Rust async, un schéma de sonde interne,
//! N convertisseurs de datasets amont → schéma interne, exécutés au build.
//!
//! Ajouter un nouvel outil OSINT de cette famille doit coûter un convertisseur
//! de ~150 lignes, jamais un nouveau moteur.

pub mod schema;
pub mod engine;
pub mod decide;
pub mod control;
pub mod health;
pub mod cache;
pub mod limiter;
pub mod extract;

pub use schema::*;
pub use engine::ProbeEngine;
