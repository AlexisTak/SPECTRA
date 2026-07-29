//! Runtime Extism WASM pour SPECTRA.
//!
//! # Rôle
//!
//! Ce crate charge des transforms écrits en WebAssembly et les exécute dans une
//! sandbox via le runtime Extism. L'hôte contrôle tout accès réseau via des
//! *host functions*.
//!
//! # Invariants
//!
//! 1. **Pas d'accès filesystem direct.** Le plugin ne voit que son répertoire
//!    alloué et les données passées par l'hôte.
//! 2. **Pas de socket brut.** Tout HTTP passe par les host functions qui
//!    appliquent l'allowlist.
//! 3. **Signature vérifiée.** Seuls les plugins signés Ed25519 sont chargés.

pub mod host;
pub mod manifest;
pub mod registry;
pub mod sandbox;
pub mod transform;

pub use manifest::PluginManifest;
pub use registry::PluginRegistry;
pub use transform::PluginTransform;
