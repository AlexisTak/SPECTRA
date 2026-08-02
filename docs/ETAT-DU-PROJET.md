# SPECTRA — État global du projet

**Date :** 2026-07-29
**Révision auditée :** `21b6b44` (branche `master`, arbre propre hors `tsconfig.tsbuildinfo`)
**Méthode :** lecture du dépôt + exécution réelle de `cargo test --workspace`, `cargo test --no-run`, `npx tsc --noEmit`, inventaire des imports et des dépendances.
**Portée :** l'ensemble du dépôt — workspace Cargo (12 membres), application Next.js/Tauri, CI, documentation.

Ce document remplace `audit.md` (2026-07-28) comme référence d'état. `audit.md` reste
utile comme **base de comparaison historique** : il décrit le point de départ (backend
qui ne compilait pas, frontend inexistant, garanties probatoires factices).

---

## 1. Résumé exécutif

En un mois de travail, le projet est passé d'un squelette non compilable à une
**application réellement fonctionnelle sur son chemin principal** : dossiers, sujets,
preuves, journal d'audit chaîné, snapshots vérifiables, recherche plein texte, rapports,
recherche OSINT par sondes réelles, assistance IA locale via Ollama.

Le verdict global est nuancé et tient en trois phrases :

1. **Ce qui est branché fonctionne et est testé.** 85 tests Rust passent, le typage
   TypeScript est propre, l'application se construit et se lance, les probes OSINT ont
   été vérifiées contre le réseau réel avec un mécanisme anti-faux-positifs qui a déjà
   attrapé un vrai défaut (PyPI).
2. **Une part significative du produit affiché n'est pas branchée.** Six des onze crates
   `spectra-*` ne sont référencés par aucun code applicatif, quatre écrans de l'interface
   affichent des données codées en dur, et dix-sept modules TypeScript « mock » subsistent
   sans aucun importateur. C'est une violation directe de la contrainte **C6** de
   `CLAUDE.md` (« pas de code mort, pas de fonctionnalité mock »).
3. **Le projet a avancé jusqu'à la Phase 7 alors que le critère d'acceptation bloquant de
   la Phase 2 n'a jamais été statué** (les FPS n'ont pas pu être mesurés). La règle
   « ne passe à la phase suivante que si tous les critères sont verts » n'a pas été tenue.

Il n'y a pas de fraude technique dans l'état actuel : les documents de banc d'essai
(`docs/bench/`) signalent eux-mêmes ce qui n'a pas pu être mesuré, et un commit antérieur
corrige explicitement une mesure FPS erronée. La dette est une dette de **périmètre
affiché** : l'interface promet plus que le backend ne délivre.

### Maturité par domaine

| Domaine | État | Note |
|---|---|---|
| Compilation / CI | Workspace vert sur 3 OS, `tsc` propre | 8/10 |
| Journal d'audit (chaînage SHA-256) | Implémenté, vérifié par 15 tests + 4 tests d'intégration | 8/10 |
| Intégrité des preuves / snapshots | Recalcul réel des hachages, vérification testée | 7/10 |
| Moteur de sondes OSINT (`spectra-probe`) | Réel, avec contrôle anti-faux-positifs | 7/10 |
| Bac à sable de plugins WASM (Extism) | Allowlist réseau prouvée par test | 7/10 |
| Modèle de domaine `spectra-core` | Écrit mais **0 test** et non utilisé pour stocker | 3/10 |
| Stockage cible `.spectra` (`spectra-store`) | Écrit, **jamais branché** à l'application | 2/10 |
| Chiffrement au repos | **Absent** (aucune dépendance Argon2 / XChaCha20) | 0/10 |
| Canvas graphe | Rendu Sigma v3 opérationnel, mais alimenté par un générateur synthétique | 4/10 |
| Écrans carte / table / notes / ACH | Données codées en dur | 1/10 |
| CLI `spectra-cli` | **Inexistante** | 0/10 |
| Conformité RGPD dans le produit | Un champ `retentionDays` en base, rien d'autre | 1/10 |
| OPSEC (proxy, Tor, kill switch) | Non implémenté | 1/10 |
| Documentation | Honnête, ADR présents, écarts assumés | 7/10 |

---

## 2. Inventaire chiffré

| Élément | Valeur |
|---|---|
| Commits | 31 (branche `master`) |
| Membres du workspace Cargo | 12 (`src-tauri` + 11 crates `spectra-*`) |
| Lignes Rust — crates `spectra-*` | ~7 650 |
| Lignes Rust — `src-tauri` (hérité `cekarna`) | ~5 246 |
| Fichiers TS/TSX (`app`, `components`, `lib`, `types`) | 64 |
| Lignes TS/TSX | ~9 138 |
| Commandes Tauri exposées | 52 |
| Tests Rust | **85 réussis, 0 échec, 1 ignoré** (réseau, volontaire) |
| Routes de l'interface | 13 |
| Sondes OSINT embarquées | **8** |
| ADR rédigés | 5 |

### Répartition des tests par crate

| Crate | Fichiers | Lignes | `#[test]` |
|---|---|---|---|
| `spectra-audit` | 3 | 599 | 15 |
| `spectra-collect` | 7 | 1 242 | 12 |
| `spectra-ai` | 5 | 765 | 7 |
| `spectra-probe` | 9 | 1 179 | 6 |
| `spectra-plugins` | 6 | 372 | 5 |
| `spectra-report` | 1 | 754 | 5 |
| `spectra-correlate` | 2 | 184 | 4 |
| `spectra-transform` | 5 | 679 | 4 |
| `spectra-store` | 7 | 891 | 3 |
| **`spectra-core`** | 2 | 239 | **0** |
| `spectra-probe-gen` | 3 | 305 | 0 |

`spectra-core` est le crate pour lequel `CLAUDE.md` §8 exige **≥ 90 % de couverture**.
Il en a zéro. C'est l'écart de qualité le plus net du dépôt.

---

## 3. Ce qui fonctionne réellement (vérifié)

Ces éléments ont été confirmés par exécution, pas par lecture de documentation.

- **`cargo build --workspace` et `cargo test --workspace` réussissent.** 85 tests passent.
  Le seul test ignoré (`whatsmyname_integration`) l'est délibérément parce qu'il touche
  le réseau — conforme à la règle « aucun test ne touche le réseau » de §8.
- **`npx tsc --noEmit` sort en code 0**, en `strict: true`, sur l'intégralité de `app/`,
  `components/`, `lib/` et `types/`.
- **Journal d'audit hash-chaîné réel.** `spectra-audit` implémente le chaînage avec JSON
  canonique ; 8 commits successifs ont branché le chaînage sur chaque commande mutante
  (`create_case`, `add_evidence`, `set_claim`, `delete_report`…). `verify_audit_trail`
  appelle bien `spectra_audit::verify_chain()`. 4 tests d'intégration
  (`src-tauri/tests/audit_chain_integration.rs`) couvrent le comportement bout en bout.
- **Vérification d'intégrité par recalcul.** Contrairement à l'état décrit dans `audit.md`
  (P1), la vérification relit les fichiers et recalcule les hachages. Test d'intégration
  dédié : `src-tauri/tests/evidence_ingestion.rs`.
- **Sondes OSINT réelles, avec garde-fou.** `spectra-probe` exécute des campagnes HTTP
  asynchrones. Le module `control.rs` génère un **sélecteur de contrôle aléatoire** et
  invalide toute sonde qui répondrait « trouvé » pour un pseudo inexistant. Ce mécanisme
  a détecté un vrai faux positif sur PyPI (documenté dans `docs/bench/osint-probes.md`).
  Faux positifs mesurés : **0 sur 8**.
- **Bac à sable de plugins WASM opérationnel.** `crates/spectra-plugins/tests/plugin_sandbox.rs`
  prouve les deux propriétés qui comptent : un appel HTTP **dans** l'allowlist du manifeste
  réussit, un appel vers `evil.com` **échoue**. C'est exactement le critère d'acceptation
  de la Phase 3.
- **CSP Tauri stricte et explicite.** `object-src 'none'`, `frame-ancestors 'none'`,
  `form-action 'none'`, `connect-src` limité à l'IPC et à `localhost:11434` (Ollama).
  Aucune désactivation de la CSP.
- **CI sérieuse.** Matrice 3 OS, `cargo fmt --check` et `clippy -D warnings` sur les crates
  `spectra-*`, `cargo-deny` (licences, conforme à C5), `cargo audit`, job frontend.
- **Empaquetage configuré.** Cibles AppImage / deb / rpm / dmg / msi / nsis, updater Tauri
  auto-hébergeable avec clé publique minisign et endpoint GitHub Releases.

---

## 4. Écarts par rapport à `CLAUDE.md`

### 4.1 Stack frontend — écart structurant, non documenté par un ADR

`CLAUDE.md` §3 arrête : **Vite + React 19 + TanStack Query + Zustand + shadcn/ui +
TanStack Router**.

La réalité du dépôt :

| Prescrit | Réel | Statut |
|---|---|---|
| Vite | **Next.js 16.2.9** en export statique vers `out/` | Écart non documenté |
| TanStack Query | Aucun — hooks maison (`lib/hooks/useTauriQuery.ts`) | Écart |
| Zustand | Aucun — `useState` local | Écart |
| shadcn/ui (Radix) | `components/ui/primitives` écrit à la main | Écart |
| TanStack Router / React Router | Routage de fichiers Next.js | Écart |
| React 19 | React 19.2.4 | ✅ |
| Sigma v3 + graphology | 3.0.3 + 0.26.0 | ✅ |
| Tailwind 4 | ✅ | ✅ |
| Vitest / Playwright | `vitest` déclaré, **`lib/__tests__/` est vide** | Écart |

Ces choix ne sont pas mauvais en soi — Next.js en export statique produit bien un bundle
local sans serveur, ce qui respecte C1/C2 — mais ils contredisent une décision écrite du
cahier des charges **sans ADR**. `CLAUDE.md` §8 exige qu'une décision structurante donne
lieu à un fichier dans `docs/adr/`. Il en manque un.

**Anomalie à traiter en priorité :** `package.json` déclare `pg` (client PostgreSQL) et
`@types/pg` en dépendances. Un client de base de données réseau n'a rien à faire dans une
application dont la contrainte C1 est « aucun appel réseau sortant hors sources OSINT ».
C'est vraisemblablement un vestige de l'application Cekarna d'origine, mais tant qu'il est
déclaré, il est dans le graphe de dépendances de l'artefact distribué.

**Anomalie secondaire :** le dépôt contient à la fois `package-lock.json` (345 Ko) et
`pnpm-lock.yaml` (202 Ko) plus un `pnpm-workspace.yaml` dont deux entrées valent encore
`"set this to true or false"`. Deux gestionnaires de paquets en concurrence garantissent
des builds divergents entre développeurs et CI.

### 4.2 Dépendances prescrites et absentes

| Prescrit §3 | Présent ? | Conséquence |
|---|---|---|
| `argon2` + `chacha20poly1305` | **Non** | **Aucun chiffrement au repos.** Contrainte C4 et §7 non tenues. |
| `tantivy` | Non | Recherche via **FTS5 SQLite**. Fonctionne, mais écart à documenter. |
| `sqlite-vec` | Non | Pas d'index vectoriel : pas de similarité sémantique ni de recherche d'avatar. |
| `fastembed-rs` | Non | Embeddings dépendants d'Ollama seul ; le mode « zéro installation » n'existe pas. |
| `image` + `img_hash` | Non | Le pHash d'avatar de `spectra-correlate` est explicitement un *placeholder* (commentaire en tête de `lib.rs`). |
| `petgraph` | Non | Le trait `GraphAnalyticsEngine` de l'ADR 0002 n'a pas d'implémentation. |
| `mistral.rs` | Non | Pas de repli embarqué au backend Ollama. |
| `scraper`, `chromiumoxide` | Non | Extraction de métadonnées de profil limitée. |
| `extism` | **Oui** (1.30) | Conforme. |
| `governor` | Partiel | Utilisé dans `spectra-transform` ; `spectra-probe` a son propre limiteur « simplifié sans governor » (`limiter.rs:1`). |

### 4.3 Éléments d'architecture prescrits et absents

- **`spectra-cli` n'existe pas.** `CLAUDE.md` §4 est catégorique : « Si une fonctionnalité
  n'existe qu'en UI, c'est un bug d'architecture. » Aujourd'hui, *toutes* les
  fonctionnalités n'existent qu'en UI.
- **Le répertoire `ontology/` n'existe pas.** Les types d'entités sont un `enum` Rust dans
  `spectra-core/src/entity.rs`, pas des fichiers TOML déclaratifs. L'ontologie n'est donc
  pas extensible sans recompilation, contrairement à §5.
- **`docs/prior-art.md` n'existe pas** (demandé en §10).
- **Conformité RGPD produit :** §7 demande base légale, finalité, durée de conservation,
  avertissement sur les données de l'article 9, écran de première utilisation. Le seul
  élément présent est une colonne `retentionDays INTEGER DEFAULT 365` dans la table
  `audit_settings`. Aucun champ de base légale ni de finalité à la création d'un dossier.
- **OPSEC :** aucun profil réseau (direct / proxy / SOCKS5 / Tor), aucune détection de
  fuite de circuit, aucun kill switch, aucune rotation de User-Agent (l'agent est une
  chaîne fixe passée à `ProbeEngine::new`).

---

## 5. Le problème central : deux modèles de données parallèles

C'est le point structurant à comprendre avant toute décision de suite.

Le dépôt contient **deux schémas de données complets qui ne se parlent pas** :

```
                    ┌──────────────────────────────┐
   INTERFACE ──────►│  src-tauri (crate cekarna)   │  ← utilisé, testé, vivant
   52 commandes     │  23 tables SQLite            │
                    │  cases / subjects / evidence │
                    │  reports / claims / snapshots│
                    └──────────────────────────────┘
                                  ▲
                                  │ dépend de : core, audit, ai, probe
                                  │
                    ┌──────────────────────────────┐
                    │  spectra-store               │  ← écrit, testé (3 tests),
                    │  entities / observations /   │     jamais appelé
                    │  relations / audit_log       │
                    └──────────────────────────────┘
                                  ▲
                    ┌─────────────┴────────────────┐
                    │ spectra-report, -transform,  │  ← aucun consommateur
                    │ -plugins, -collect, -correlate│
                    └──────────────────────────────┘
```

Vérification : `src-tauri/Cargo.toml` ne déclare que quatre crates internes —
`spectra-core`, `spectra-audit`, `spectra-ai`, `spectra-probe`. Les six autres
(`store`, `transform`, `plugins`, `collect`, `correlate`, `report`) ne sont référencés
par **aucun** code applicatif.

Conséquence directe : le **modèle d'observation temporelle**, qui est présenté au §11 de
`CLAUDE.md` comme la différenciation numéro 3 face à Maltego, existe en code mais **ne
stocke rien**. L'application enregistre des `evidence` et des `subjects` à l'ancienne, sans
`observed_at` / `valid_from` / `valid_to`, sans `Provenance`, sans code Admiralty par
observation. Le curseur temporel de §5 n'a aucune donnée à rejouer.

De même :
- `spectra-report` sait générer PDF/HTML/Markdown depuis un `Store`… que personne
  n'alimente. Les rapports réellement produits par l'interface passent par
  `src-tauri/src/commands/reports.rs`, qui est une implémentation distincte.
- `spectra-collect` implémente DNS, WHOIS, Certificate Transparency, email, téléphone et
  WhatsMyName — **1 242 lignes, 12 tests** — sans qu'aucun bouton de l'interface ne puisse
  les déclencher. L'écran OSINT appelle `run_osint_campaign`, qui passe par
  `spectra-probe` et ses 8 sondes.

---

## 6. Code mort et écrans fictifs (violation C6)

### 6.1 Modules TypeScript sans aucun importateur

Dix-sept fichiers de `lib/` sont des « mocks » retournant des valeurs vides, dont l'en-tête
dit lui-même « Mock … module for development ». Aucun n'est importé nulle part :

```
ai-extractions, audit, audit-types, canonical, cases, claims, cross-links,
events, evidence, hashes, investigation-types, osint, reports, search,
settings, snapshots, storage, subjects, tiktok, transcriptions, web-archives
```

Exemple type — `lib/cases.ts:26` :

```ts
export async function getCases(filters?: {…}): Promise<Case[]> {
  // Mock implementation - would call Tauri get_cases in production
  return []
}
```

La couche réellement utilisée est `lib/api.ts` (12 importateurs) qui délègue à
`lib/tauri-bridge.ts`. Ces 17 modules sont du bruit pur : ils dupliquent des types, ils
peuvent être importés par erreur, et ils font croire à un audit rapide que l'application
est factice alors qu'elle ne l'est plus.

### 6.2 Écrans alimentés par des données codées en dur

| Route | Appels backend | Données |
|---|---|---|
| `/` (dashboard) | 2 | réelles |
| `/cases` | 2 | réelles |
| `/audit` | 2 | réelles |
| `/reports` | 2 | réelles |
| `/timeline` | 2 | réelles |
| `/search` | 1 | réelles |
| `/ai` | 6 | réelles (Ollama) |
| `/osint` | `invoke('run_osint_campaign')` | réelles |
| **`/map`** | 0 | `MOCK_LOCATIONS` — 5 lieux parisiens en dur |
| **`/ach`** | 0 | `EVIDENCES` + hypothèses en dur |
| **`/notes`** | 0 | tableau en dur |
| **`/table`** | 0 | tableau en dur |
| **`/graph`** | 0 | `generateScaleFreeGraph()` — graphe synthétique |
| `/settings` | 0 (lecture version app) | pas de persistance |

Cinq écrans sur treize n'ont aucun lien avec un dossier réel. Le cas de `/graph` est le
plus gênant : `CLAUDE.md` §6 pose que « le canvas graphe **est** le produit ». Le canvas
existe, il tient la charge, mais il n'affiche jamais les entités d'une enquête — seulement
un graphe de benchmark.

---

## 7. État par phase, avec verdict d'acceptation

| Phase | Critère d'acceptation `CLAUDE.md` §12 | Verdict |
|---|---|---|
| **0 — Fondations** | `cargo build --workspace` + `pnpm build` verts en CI 3 OS | ✅ **Tenu** — CI en place, 5 ADR rédigés |
| **1 — Cœur du domaine** | ≥ 90 % couverture `core` ; dossier créé/rouvert/chiffré sans perte ; curseur temporel prouvé par test | ❌ **Non tenu** — `spectra-core` a 0 test, aucun chiffrement, aucun test de projection temporelle |
| **2 — Canvas graphe** | 50 k nœuds / 150 k arêtes, **≥ 45 fps**, < 1,5 Go | ⚠️ **Non statué** — layout 6,8 s et mémoire 93 Mo tenus ; les FPS sont **non mesurables** sous Playwright (throttling `requestAnimationFrame` à 1 Hz, diagnostic dans `docs/bench/README.md`). Le cahier des charges impose de **s'arrêter là** tant que ce n'est pas statué. |
| **3 — Moteur de transforms** | Un plugin WASM s'installe, s'exécute, un appel hors allowlist est bloqué par un test | ✅ **Tenu au niveau du crate** — `plugin_sandbox.rs` le prouve. ❌ **Non tenu au niveau produit** : rien de tout cela n'est atteignable depuis l'application. |
| **4 — Recherche de personne** | Dossier peuplé en < 60 s depuis un pseudo, provenance complète, taux de faux positifs mesuré | ⚠️ **Partiel** — moteur réel, 0 faux positif sur 8 sondes, mais **8 sondes au lieu de 600+**, `data/whatsmyname/` est vide, les faux négatifs ne sont pas mesurés, la corrélation d'avatars est un placeholder |
| **5 — Système d'enquêtes** | Rapport PDF complet depuis un dossier réel, annexes hashées, audit joint | ⚠️ **Partiel** — chemin rapports de `src-tauri` fonctionnel et testé (`pdf_with_audit.rs`) ; timeline réelle ; **carte, table, notes et ACH sont fictifs** ; fusion CRDT annoncée en commit mais sans test de conflit |
| **6 — IA locale** | Dégradation propre sans Ollama, sorties marquées `INFERRED` et exclues des rapports | ✅ **Probablement tenu** — trait `AiBackend` avec `is_available()`, `useOllamaStatus`, 7 tests. Le marquage `INFERRED` existe dans `spectra-core` ; **à re-vérifier explicitement** sur le chemin rapport |
| **7 — Finition** | Installation propre sur 3 OS depuis un artefact CI | ⚠️ **Configuré, non prouvé** — bundle 6 cibles, updater signé, i18n FR/EN présent. Aucune trace d'une installation réellement testée depuis un artefact |

**Lecture d'ensemble :** les phases ont été exécutées en séquence rapide, mais les critères
d'acceptation des phases 1, 2 et 4 ne sont pas verts. Le produit a la **surface** de sept
phases et la **profondeur** de trois.

---

## 8. Risques

| # | Risque | Gravité | Signal de déclenchement |
|---|---|---|---|
| R1 | **Absence de chiffrement au repos.** Un dossier d'enquête contenant des données personnelles est en clair sur le disque. Contrainte C4 et §7 non tenues, exposition RGPD réelle pour un utilisateur français. | **Critique** | Immédiat — aucun `argon2` / `chacha20poly1305` dans l'arbre de dépendances |
| R2 | **Le modèle d'observation n'est pas le modèle de stockage.** Plus l'interface accumule de fonctionnalités sur le schéma hérité, plus la migration vers `spectra-store` coûte cher. La différenciation produit numéro 3 est aujourd'hui du code non exécuté. | **Élevé** | Chaque nouvelle commande écrite sur `src-tauri/src/database.rs` aggrave la dette |
| R3 | **Critère FPS de la Phase 2 jamais statué.** Si le canvas ne tient pas 45 fps une fois branché sur des données réelles (labels, icônes, badges de provenance, halos), c'est un blocage architectural découvert très tard. | **Élevé** | Se saura à la première mesure dans la coquille Tauri au premier plan |
| R4 | **Écrans fictifs en production.** Un analyste qui ouvre `/map` voit cinq lieux parisiens qui ne viennent d'aucune enquête. Dans un outil probatoire, une donnée d'illustration indiscernable d'une donnée réelle est un défaut de sécurité, pas un défaut cosmétique. | **Élevé** | Immédiat |
| R5 | **Deux gestionnaires de paquets, dépendance `pg` résiduelle.** Builds non reproductibles et client réseau non justifié dans l'artefact. | Moyen | Immédiat |
| R6 | **Couverture `spectra-core` nulle.** Le crate qui porte la sémantique probatoire est le moins testé du dépôt. | Moyen | Immédiat |
| R7 | **8 sondes OSINT.** L'écart au format WhatsMyName (600+ sites) rend le Pilier A peu utile en pratique aujourd'hui. | Moyen | À l'usage réel |

---

## 9. Plan de remise à niveau proposé

Ordonné par rapport valeur / risque, pas par facilité.

### Lot A — Assainissement (rapide, sans arbitrage)

1. Supprimer les 17 modules mock de `lib/`. Aucun importateur, aucun risque.
2. Retirer `pg` et `@types/pg` de `package.json`.
3. Choisir **un seul** gestionnaire de paquets ; supprimer l'autre lockfile ; compléter
   ou supprimer `pnpm-workspace.yaml`.
4. Rédiger l'ADR manquant sur le choix Next.js plutôt que Vite, et l'ADR sur FTS5 plutôt
   que tantivy. Une décision non écrite se re-débat indéfiniment.
5. Écrire `docs/prior-art.md`.

### Lot B — Statuer la Phase 2 (bloquant selon `CLAUDE.md`)

6. Mesurer les FPS **dans la fenêtre Tauri au premier plan**, avec le compteur déjà présent
   (`app/graph/use-fps-counter.ts`), sur 50 k / 150 k nœuds avec le rendu final (labels,
   icônes, halos). Publier le résultat, quel qu'il soit.
7. Si < 45 fps : ne pas contourner. Présenter les options chiffrées (clustering, LOD plus
   agressif, vue overview/détail, pré-layout Rust) avant d'écrire du code.

### Lot C — Réunifier le modèle de données (le vrai chantier)

8. Décider explicitement, par ADR : soit `spectra-store` devient la source de vérité et le
   schéma hérité est migré, soit `spectra-store` est supprimé et le modèle d'observation
   est porté dans le schéma existant. **La situation actuelle — les deux — est la seule
   option à exclure.**
9. Quelle que soit l'option : faire porter à toute donnée un `observed_at`, une `Provenance`
   et un code Admiralty. C'est la condition d'existence du curseur temporel et de la
   distinction visuelle collecté / inféré / hypothèse (§6.5).
10. Brancher le canvas graphe sur les entités du dossier courant. Retirer le générateur
    synthétique du chemin applicatif (le conserver dans `apps/desktop/src/features/graph/`
    pour le banc d'essai).

### Lot D — Combler les contraintes non tenues

11. Chiffrement au repos : Argon2id + XChaCha20-Poly1305, verrouillage après inactivité.
12. Champs RGPD à la création d'un dossier : base légale, finalité, durée de conservation.
13. Profils réseau (direct / proxy / SOCKS5 / Tor) et kill switch, avec refus de démarrage
    si le profil Tor est sélectionné sans circuit établi.
14. `spectra-cli` : ouvrir un dossier, lancer un transform, exporter un rapport. C'est ce
    qui prouvera que la séparation UI / logique est réelle.

### Lot E — Densifier le Pilier A

15. Importer le jeu WhatsMyName complet via `spectra-probe-gen` (le convertisseur existe
    déjà, `data/whatsmyname/` est vide).
16. Brancher `spectra-collect` (DNS, WHOIS, CT, email, téléphone) sur l'interface : 1 242
    lignes testées attendent un point d'entrée.
17. Implémenter le pHash d'avatar (`image` + `img_hash`) pour sortir `spectra-correlate` de
    son état de placeholder.

### Ce qu'il ne faut pas faire maintenant

- Ajouter des écrans. Il y en a déjà cinq qui n'ont rien à afficher.
- Ajouter des fonctions IA. Le socle non-IA n'est pas complet, et §9 est explicite :
  « pas de LLM là où une heuristique déterministe suffit ».
- Empaqueter et distribuer avant le Lot D. Distribuer une application d'enquête sans
  chiffrement au repos, à des utilisateurs français, engage réellement.

---

## 10. Historique des corrections

### 2026-07-29 — Nettoyage Lot A

**Supprimés :**
- 17 modules mock de `lib/` : `ai-extractions`, `cases`, `cross-links`, `events`, `evidence`, `hashes`, `investigation-types`, `osint`, `reports`, `search`, `settings`, `snapshots`, `storage`, `subjects`, `tiktok`, `transcriptions`, `web-archives`
- `pg` et `@types/pg` de `package.json`
- `pnpm-workspace.yaml`, `pnpm-lock.yaml`, `package-lock.json`

**Ajoutés :**
- `graphology-types@0.24.8` (déclarations TypeScript manquantes)

**Résultat :**
- `cargo test --workspace` : 85 tests passés, 0 échec
- `npx tsc --noEmit` : 0 erreur
- Build propre, aucune dépendance réseau inutile dans l'artefact

---

## 11. Annexes — commandes de vérification

```bash
# Compilation et tests (résultat au 2026-07-29 : 85 passés, 0 échec, 1 ignoré)
cargo test --workspace

# Typage frontend (résultat : code 0)
npx tsc --noEmit

# Crates internes réellement consommés par l'application
grep -n "spectra-" src-tauri/Cargo.toml

# Modules TypeScript sans importateur
for f in lib/*.ts; do n=$(basename "$f" .ts); \
  echo "$n -> $(grep -rl "@/lib/$n'" app components lib types | grep -vc "lib/$n.ts")"; done

# Écrans sans appel backend
grep -Lc "tauriInvoke\|invoke(\|useCases\|@/lib/api" app/*/page.tsx
```

---

## 12. Conclusion

Le projet n'est plus le squelette décrit par `audit.md`. Le chemin critique —
créer un dossier, y verser des preuves, en vérifier l'intégrité, tracer les actions dans
un journal chaîné, lancer des sondes OSINT contrôlées, produire un rapport — **fonctionne
et est testé**. C'est un progrès considérable en 31 commits.

Ce qui manque n'est pas de la finition, c'est de la **cohérence** : six crates écrits et
non branchés, deux schémas de données concurrents, cinq écrans sans données, et trois
contraintes non négociables du cahier des charges (chiffrement, RGPD produit, CLI) encore
non satisfaites. Le remède n'est pas d'écrire plus, mais de **relier ce qui existe déjà et
de supprimer ce qui ne sert pas**.

La décision la plus urgente est celle du Lot C : quel modèle de données porte le produit.
Tant qu'elle n'est pas tranchée, tout ce qui s'ajoute s'ajoute deux fois.
