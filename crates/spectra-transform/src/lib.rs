//! Moteur de transforms SPECTRA.
//!
//! # Rôle
//!
//! Ce crate définit le contrat [`Transform`] et l'[`Orchestrator`] qui l'exécute.
//! Il ne fait aucune I/O réseau directe : il orchestre des transforms qui
//! peuvent être natifs (Rust) ou sandboxés (WASM via `spectra-plugins`).
//!
//! # Invariants
//!
//! 1. **Aucune I/O réseau directe.** L'orchestrateur gère le rate limiting et
//!    le cache, mais ne parle pas HTTP.
//! 2. **Annulation propre.** Toute exécution est annulable via
//!    `tokio_util::sync::CancellationToken`.
//! 3. **Pas de blocage du thread principal.** Tout est async.

pub mod cache;
pub mod manifest;
pub mod orchestrator;
pub mod transform;

pub use cache::ResponseCache;
pub use manifest::{
    NetworkPermission, TransformManifest, TransformPermission,
};
pub use orchestrator::{ExecutionPlan, Orchestrator};
pub use transform::{
    Relation, Transform, TransformContext, TransformError, TransformInput, TransformOutput,
};
