//! Journal d'audit hash-chaîné.
//!
//! # Rôle
//!
//! Calcule et vérifie la chaîne de hachages qui rend le journal d'enquête
//! inaltérable sans détection. C'est la brique qui donne — ou non — sa valeur
//! probatoire au dossier.
//!
//! # Invariants
//!
//! 1. **Le hachage est calculé, jamais reçu.** Aucune valeur de hachage fournie
//!    par un appelant n'est stockée telle quelle. L'implémentation héritée
//!    stockait la chaîne littérale `hex(sha256('…'))` (`audit.md`, P1-3).
//! 2. **`hash(n) = SHA-256(canonical_json(événement n) || hash(n-1))`**, où
//!    `hash(n-1)` est l'empreinte *effective* du maillon précédent du même
//!    dossier — pas un hachage recalculé à partir d'anciennes valeurs
//!    (`audit.md`, P1-4).
//! 3. **La vérification recalcule.** Constater qu'un champ est non vide n'est
//!    pas une vérification. Toute altération de `action`, `actor` ou `metadata`
//!    doit être détectée.
//! 4. **Sérialisation canonique déterministe** : clés triées, valeurs
//!    `undefined`/`null` traitées identiquement côté Rust et TypeScript. Un test
//!    de concordance croisée est obligatoire (`lib/canonical.ts` le promet déjà
//!    sans l'honorer — `audit.md`, P3-1).
//! 5. **Ordre total explicite** : les événements portent un numéro de séquence
//!    monotone par dossier. Trier sur un horodatage à la seconde ne définit pas
//!    un ordre (`audit.md`, P2-14).

pub mod canonical;
pub mod chain;

pub use canonical::canonical_json;
pub use chain::{compute_link, verify_chain, verify_link, AuditEvent, ChainLink};
