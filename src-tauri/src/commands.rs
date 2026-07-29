//! Commands module for Cekarna Tauri application
//!
//! This module re-exports all command submodules.

// Import command submodules
pub mod cases;
pub mod evidence;
pub mod subjects;
pub mod events;
pub mod snapshots;
pub mod integrity;
pub mod audit;
pub mod claims;
pub mod search;
pub mod reports;
pub mod osint;

// Re-export all command functions for Tauri 2
// Use pub use to make Tauri's internal macros accessible to generate_handler!
pub use self::cases::*;
pub use self::evidence::*;
pub use self::subjects::*;
pub use self::events::*;
pub use self::snapshots::*;
pub use self::integrity::*;
pub use self::audit::*;
pub use self::claims::*;
pub use self::search::*;
pub use self::reports::*;
pub use self::osint::*;
