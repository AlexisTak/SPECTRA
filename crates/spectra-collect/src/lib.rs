//! Transforms natifs de collecte SPECTRA.
//!
//! # Transforms implémentés
//!
//! | Transform | Entrée | Sortie |
//! |-----------|--------|--------|
//! | [`DnsTransform`] | `Domain`, `IpAddress` | Enregistrements DNS (A, AAAA, MX, TXT) |
//! | [`WhoisTransform`] | `Domain`, `IpAddress` | Informations WHOIS (registrar, dates, contact) |
//! | [`CtTransform`] | `Domain` | Entrées Certificate Transparency (`crt.sh`) |
//!
//! Tous les transforms implémentent le trait [`Transform`] de `spectra-transform`.
//! Les tests utilisent des fixtures statiques — aucun test ne touche le réseau.

pub mod ct;
pub mod dns;
pub mod email;
pub mod phone;
pub mod whatsmyname;
pub mod whois;

pub use ct::CtTransform;
pub use dns::DnsTransform;
pub use email::EmailTransform;
pub use phone::PhoneTransform;
pub use whatsmyname::WhatsMyNameTransform;
pub use whois::WhoisTransform;
