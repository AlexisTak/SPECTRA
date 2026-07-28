# ADR 0001 — Versions vérifiées et écarts avec `CLAUDE.md`

* **Statut** : accepté
* **Date** : 2026-07-28
* **Phase** : 0
* **Sources** : API crates.io, registry npm, API GitHub — relevés le 2026-07-28

## Contexte

`CLAUDE.md` §3 fixe des versions cibles « connues à mi-2026 » et impose (§13.2) de tout vérifier
avant d'écrire du code, en notant tout écart dans `docs/adr/`. Ce document consigne le relevé.

## Relevé

| Composant | Cible `CLAUDE.md` | Version stable réelle | Date de publication | Écart |
|---|---|---|---|---|
| **Tauri** | `2.11.x` | **2.11.5** | 2026-07-01 | ✅ conforme |
| `@tauri-apps/api` (npm) | — | **2.11.1** | — | — |
| **React** | `19.2.x` | **19.2.8** | — | ✅ conforme |
| **Vite** | « dernière stable » | **8.1.5** | — | ✅ |
| **Tailwind CSS** | v4 | **4.3.3** | — | ✅ |
| **Sigma.js** | v3 | **3.0.3** | — | ✅ conforme |
| **graphology** | — | **0.26.0** | — | — |
| `graphology-layout-forceatlas2` | — | **0.10.1** | — | — |
| **Extism** (crate hôte) | — | **1.30.0** | 2026-06-04 | ✅ actif (203 k DL récents) |
| **wasmtime** | — | **47.0.2** | 2026-07-21 | ⚠️ Extism embarque sa propre version, ne pas épingler séparément |
| **tantivy** | — | **0.26.1** | 2026-04-21 | ✅ |
| **rusqlite** | — | **0.40.1** | 2026-06-06 | ⚠️ le code Cekarna utilisait `0.31` (9 versions de retard) |
| **sqlite-vec** (crate FFI) | — | **0.1.9** stable / `0.1.10-alpha.4` | 2026-05-18 | ⚠️ **pré-1.0** — voir risque R4 |
| **ollama-rs** | — | **0.3.6** | 2026-07-24 | ✅ très actif |
| **fastembed** (`fastembed-rs`) | — | **5.17.3** | 2026-07-15 | ✅ ; nom du crate = `fastembed` |
| **petgraph** | — | **0.8.3** | 2025-09-30 | ⚠️ 10 mois sans release — bibliothèque mature et stable, pas abandonnée, mais à surveiller |
| **governor** | — | **0.10.4** | 2025-12-16 | ✅ |

## Décisions

1. Les cibles de `CLAUDE.md` §3 sont **exactes** pour Tauri, React, Sigma v3 et Tailwind 4 : aucun
   ajustement du cahier des charges nécessaire.
2. `rusqlite` sera épinglé en **0.40** (et non 0.31), avec les features `bundled` et `serde_json`
   dès le départ — l'absence de cette seconde feature était la cause de ~20 erreurs de compilation
   dans le code précédent.
3. `wasmtime` ne sera **pas** déclaré en dépendance directe : il est fourni transitivement par
   `extism`. Le déclarer séparément expose à un conflit de versions.
4. Le crate d'embeddings s'appelle `fastembed` (et non `fastembed-rs`, qui est le nom du dépôt).
5. `sqlite-vec` étant pré-1.0, son intégration est repoussée à la phase qui en a besoin
   (déduplication sémantique / similarité d'avatars, Phase 4-6) et placée derrière un trait, jamais
   dans le chemin critique du stockage.

## Conséquences

* Les versions ci-dessus sont à réévaluer au démarrage de chaque phase, pas une fois pour toutes.
* Aucune dépendance relevée n'est sous licence incompatible avec AGPL-3.0 (contrainte C5) :
  Tauri (MIT/Apache-2.0), rusqlite (MIT), tantivy (MIT), Extism (BSD-3-Clause),
  wasmtime (Apache-2.0 WITH LLVM-exception), petgraph (MIT/Apache-2.0), governor (MIT).
  Une vérification automatisée (`cargo deny`) sera mise en place en Phase 0 plutôt que de reposer
  sur ce relevé manuel.
