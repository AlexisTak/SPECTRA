//! Persistance SPECTRA.
//!
//! # Rôle
//!
//! Traduit le modèle de [`spectra_core`] vers un fichier de dossier `.spectra`
//! (une base `SQLite` unique par enquête) et retour. Seul crate autorisé à
//! connaître SQL.
//!
//! # Invariants
//!
//! 1. **`SQLite` est la source de vérité unique.** Aucune base graphe n'est une
//!    dépendance obligatoire (voir `docs/adr/0002`).
//! 2. **Toute écriture multi-tables est transactionnelle.** Un échec partiel ne
//!    doit jamais laisser une preuve sans sa trace d'audit — défaut identifié
//!    dans l'implémentation héritée (`audit.md`, P2-1).
//! 3. **Aucune suppression physique** d'un élément porteur de valeur probatoire :
//!    les suppressions sont logiques et auditées (`audit.md`, P1-8).
//! 4. Un dossier est portable et chiffrable : le chemin du fichier est fourni
//!    par l'appelant, jamais déduit du répertoire courant (`audit.md`, P1-10).
//!
//! # État
//!
//! Phase 0 : crate déclaré et intégré au workspace. Le schéma et les migrations
//! sont l'objet de la Phase 1.
