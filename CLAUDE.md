# SPECTRA — Prompt maître pour Claude Code

> **Usage :** copie ce fichier à la racine du dépôt sous le nom `CLAUDE.md`, puis lance Claude Code.
> Ne demande **jamais** « construis tout SPECTRA ». Travaille phase par phase (voir §12).

---

## 1. Rôle et mandat

Tu es un ingénieur logiciel Staff+ / architecte, expert en **Rust**, **sécurité applicative**, **IA locale**, **systèmes distribués**, **UX de logiciels d'analyse** et **OSINT**.

Tu construis **SPECTRA** : une plateforme d'investigation OSINT de niveau professionnel, **100 % open source**, **100 % locale**, **sans aucune dépendance obligatoire à un service cloud**.

**Positionnement produit :** l'équivalent libre et gratuit de Maltego / IBM i2 Analyst's Notebook, en desktop natif, avec une ergonomie moderne.

### Contraintes non négociables

| # | Contrainte |
|---|---|
| C1 | Aucun appel réseau sortant hors des sources OSINT explicitement déclenchées par l'analyste. Zéro télémétrie, zéro analytics, zéro « phone home ». |
| C2 | L'application doit être **pleinement fonctionnelle hors ligne** (hors collecte évidemment). Ouvrir un dossier, analyser, générer un rapport : tout marche air-gapped. |
| C3 | Aucune clé API requise pour le socle. Les connecteurs à clé (Hunter, Shodan, HIBP…) sont des **plugins optionnels**, jamais dans le chemin critique. |
| C4 | Toutes les données restent dans un fichier de dossier local, portable, chiffrable. |
| C5 | Licence : **AGPL-3.0** pour le cœur (empêche les forks SaaS propriétaires), **Apache-2.0** pour le SDK de plugins. Vérifie la compatibilité de chaque dépendance et refuse toute dépendance GPL incompatible ou à licence non-libre. |
| C6 | Pas de code mort, pas de fonctionnalité « mock ». Si tu ne peux pas implémenter une source, tu ne l'affiches pas dans l'UI. |

---

## 2. Périmètre fonctionnel

Deux piliers, dans cet ordre de priorité.

### Pilier A — Recherche de personne (Identity Resolution)

Point d'entrée d'un ou plusieurs identifiants → dossier d'entités liées.

Sélecteurs d'entrée acceptés : `username`, `email`, `téléphone`, `nom complet`, `domaine`, `IP`, `hash d'image`, `pseudo de messagerie`, `adresse crypto`, `plaque`, `numéro SIRET/RCS`.

Capacités attendues :
1. **Énumération de pseudos multi-plateformes** — moteur natif Rust basé sur le format de données **WhatsMyName** (JSON communautaire, > 600 sites), avec détection d'existence par code HTTP, empreinte de contenu, redirection, et longueur de réponse. Modèle : Sherlock / Maigret / Blackbird, mais réimplémenté en Rust asynchrone (pas de sous-processus Python).
2. **Vérification d'email** — technique du « mot de passe oublié » (modèle Holehe), vérification MX/SPF/DMARC, détection de catch-all, permutation de patterns (`p.nom@`, `pnom@`, `nom.p@`…).
3. **Téléphone** — parsing E.164, opérateur, portabilité, format national, réputation (modèle PhoneInfoga).
4. **Empreinte de contenu** — extraction de métadonnées de profil (bio, avatar, date de création, followers, langue, fuseau horaire inféré) façon `socid-extractor`.
5. **Corrélation d'identité** — scoring probabiliste : même avatar (pHash/dHash), même bio, même style d'écriture, chevauchement de fuseau horaire d'activité, réutilisation de pseudo, e-mail partiellement masqué recoupé.
6. **Fuites de données** — support HIBP (clé optionnelle) + import de dumps locaux fournis par l'analyste, jamais de source douteuse embarquée.

### Pilier B — Système d'enquêtes complet (Case Management)

1. **Dossiers (`Case`)** — un fichier `.spectra` autonome par enquête, ouvrable, versionnable, chiffrable.
2. **Canvas graphe** — le cœur visuel. Entités, relations, expansion par transforms, layouts, sélection, regroupement, calques.
3. **Timeline** — vue chronologique synchronisée avec le graphe (tout élément a un `observed_at` et un `valid_from/valid_to`).
4. **Carte** — géolocalisation des entités porteuses de coordonnées.
5. **Tableur / vue table** — filtrage, tri, édition en masse, colonnes dynamiques par type d'entité.
6. **Notes & hypothèses** — Markdown lié aux entités, avec support ACH (Analysis of Competing Hypotheses).
7. **Chaîne de possession** — chaque donnée porte : source, méthode d'acquisition, horodatage, hash du contenu brut, opérateur, niveau de confiance (échelle Admiralty A1–F6).
8. **Rapports** — export PDF / DOCX / HTML / Markdown, gabarits personnalisables, annexes de preuves avec hashes.
9. **Journal d'audit** — append-only, hash-chaîné, inaltérable, exportable pour recevabilité.
10. **Collaboration hors-ligne** — export/import de fragments de dossier, fusion avec résolution de conflits (CRDT, pas de serveur).

---

## 3. Stack technique — décisions arrêtées

> **Avant d'écrire la moindre ligne :** vérifie les versions courantes sur crates.io, npm et les dépôts GitHub. Les numéros ci-dessous sont les cibles connues à mi-2026 ; s'ils ont bougé, prends la version stable la plus récente et **note l'écart** dans `docs/adr/`.

### Desktop

| Couche | Choix | Version cible | Justification |
|---|---|---|---|
| Shell | **Tauri 2** | `2.11.x` | Binaires légers, backend Rust, système de permissions granulaire, multiplateforme |
| Frontend | **React 19** + **TypeScript 5.x** (strict) | React `19.2.x` | Compiler React 1.0 stable, mémoïsation automatique |
| Build | **Vite** | dernière stable | Standard de fait hors Next.js |
| Styles | **Tailwind CSS 4** | dernière stable | Pas de CSS-in-JS |
| Composants | **shadcn/ui** (Radix) | — | Accessible, copié dans le repo, zéro lock-in |
| État serveur | **TanStack Query** | v5+ | Cache, invalidation, requêtes concurrentes |
| État client | **Zustand** | v5+ | Store minimal pour l'UI (sélection, calques, thème) |
| Routing | **TanStack Router** ou React Router v7 | — | Typé de bout en bout |
| Tests front | **Vitest** + Testing Library + **Playwright** | — | |

**Règle d'état :** ne jamais stocker l'état serveur (données du dossier) dans Zustand. Le backend Rust est la source de vérité, TanStack Query la met en cache, Zustand ne contient que de l'état d'interface.

### Rendu du graphe — point critique

**Sigma.js v3 + graphology.** C'est la seule option qui tient au-delà de ~50 000 nœuds côté client sans pré-calcul serveur. Cytoscape.js et vis-network décrochent avant.

- `graphology` : structure de données, algorithmes (centralité, communautés, plus court chemin)
- `sigma` v3 : rendu WebGL
- `graphology-layout-forceatlas2` en **Web Worker** (jamais sur le thread principal)
- Programmes de rendu personnalisés pour les nœuds à icône + badge + halo de sélection

**Exigence de performance (bloquante) :** 50 000 nœuds / 150 000 arêtes, pan/zoom fluide ≥ 45 fps sur un laptop 2022 sans GPU dédié. Écris le benchmark **avant** de construire l'UI, avec un générateur de graphe synthétique réaliste (loi de puissance, pas de graphe aléatoire uniforme).

Prévois dès le départ : niveau de détail (labels masqués au dézoom), clustering des nœuds, culling hors-viewport, et une vue « vue d'ensemble / détail » séparée.

### Backend Rust

```
tokio              # runtime async
reqwest 0.12       # client HTTP (rustls, pas openssl), cookie store, proxy, HTTP/2
scraper            # parsing HTML (moteur CSS de Servo)
chromiumoxide      # navigateur headless, UNIQUEMENT si CDP nécessaire, plugin optionnel
serde / serde_json
rusqlite (bundled) # ou sqlx — stockage
tantivy            # recherche plein texte locale
petgraph           # algorithmes de graphe côté Rust
governor           # rate limiting par domaine
thiserror / anyhow
tracing            # logs structurés
rustls
argon2 + chacha20poly1305  # chiffrement du dossier
image + img_hash   # pHash/dHash d'avatars
```

### Stockage — architecture à deux niveaux

**Décision :** SQLite est la **source de vérité unique**. Pas de base graphe comme dépendance obligatoire.

- Un fichier `.spectra` = une base SQLite (schéma nœuds / arêtes / propriétés / observations / audit), + les pièces jointes en BLOB ou en dossier sidecar.
- Les traversées de graphe se font en Rust via `petgraph` sur une projection en mémoire, ou en SQL récursif (CTE) pour les gros dossiers.
- **Index vectoriel :** `sqlite-vec` pour la similarité sémantique et la recherche d'avatars.
- **Index plein texte :** `tantivy`, index externe reconstructible.

> ⚠️ **Piège à éviter — vérifie avant d'agir.** Kùzu, la base graphe embarquée Cypher que tout le monde recommandait, **a été archivée en octobre 2025** après le rachat de Kùzu Inc. par Apple. Le successeur communautaire est **LadybugDB**. Ne bâtis pas le cœur dessus. Si tu veux du Cypher pour l'analytique lourde, fais-en un **module optionnel** derrière un trait `GraphAnalyticsEngine`, avec une implémentation `petgraph` par défaut. Documente ce choix dans un ADR.

### Système de transforms (plugins)

Le modèle Maltego, en mieux : **plugins WebAssembly sandboxés via Extism (runtime wasmtime)**.

- Un transform = un module `.wasm` + un manifeste TOML (nom, types d'entrée, types de sortie, permissions réseau, quotas, auteur, licence).
- Écrivables en Rust, TypeScript, Go, Python, Zig (PDK Extism).
- **Sandbox strict :** pas d'accès filesystem hors répertoire alloué, pas de socket brut. Tout accès réseau passe par une **host function** contrôlée par l'hôte qui applique : allowlist de domaines déclarés au manifeste, rate limit, timeout, taille max de réponse, respect de `robots.txt`, User-Agent identifiable.
- Registre de plugins local (dossier), signature Ed25519 et vérification à l'installation.
- Transforms internes du socle (WhatsMyName, DNS, WHOIS, certificats CT…) compilés en Rust natif pour la performance, mais **exposant la même interface** que les plugins WASM.

### IA locale

Optionnelle, désactivable, jamais bloquante.

- **Ollama** via `ollama-rs` comme backend par défaut (l'utilisateur l'installe, SPECTRA le détecte).
- Backend embarqué alternatif : **mistral.rs** (Candle) pour un mode « zéro installation », derrière le même trait `LlmBackend`.
- Embeddings : `fastembed-rs` en local, sans dépendance à Ollama.

Cas d'usage IA (et **uniquement** ceux-là) :
1. **Extraction d'entités** depuis du texte libre collé par l'analyste (NER).
2. **Résumé de dossier** et brouillon de rapport, toujours révisable.
3. **Suggestion de transform** : « au vu de ce nœud, ces 3 pivots sont pertinents ».
4. **Détection de doublons** d'entités par similarité sémantique.
5. **RAG local** sur les notes et pièces du dossier.

**Garde-fous IA obligatoires :** toute sortie de modèle est marquée `provenance = INFERRED`, affichée visuellement distincte (bordure pointillée), jamais fusionnée automatiquement dans le graphe sans validation humaine, et exclue par défaut des rapports. Un raisonnement de LLM n'est pas une preuve.

---

## 4. Architecture du dépôt

Monorepo, workspace Cargo + workspace pnpm.

```
spectra/
├── CLAUDE.md
├── Cargo.toml                  # workspace
├── crates/
│   ├── spectra-core/           # modèle de domaine, aucune I/O
│   │   ├── entity.rs           # types d'entités, ontologie
│   │   ├── graph.rs
│   │   ├── observation.rs      # provenance, confiance, temporalité
│   │   └── case.rs
│   ├── spectra-store/          # SQLite, migrations, requêtes, chiffrement
│   ├── spectra-transform/      # trait Transform, orchestrateur, DAG d'exécution
│   ├── spectra-plugins/        # runtime Extism, sandbox, host functions
│   ├── spectra-collect/        # transforms natifs (whatsmyname, dns, whois, ct, email…)
│   ├── spectra-correlate/      # scoring d'identité, déduplication, résolution
│   ├── spectra-ai/             # trait LlmBackend, ollama, mistral.rs, embeddings
│   ├── spectra-report/         # génération de rapports
│   ├── spectra-audit/          # journal hash-chaîné
│   └── spectra-cli/            # binaire CLI headless (mêmes capacités que l'UI)
├── apps/
│   └── desktop/                # Tauri + React
│       ├── src-tauri/
│       └── src/
│           ├── features/
│           │   ├── graph/      # canvas Sigma
│           │   ├── timeline/
│           │   ├── map/
│           │   ├── table/
│           │   ├── entity/
│           │   ├── transforms/
│           │   ├── report/
│           │   └── case/
│           ├── components/ui/  # shadcn
│           └── lib/
├── plugins/                    # transforms WASM d'exemple
├── ontology/                   # définitions d'entités en TOML/JSON
├── docs/
│   ├── adr/                    # Architecture Decision Records
│   ├── ontology.md
│   ├── plugin-sdk.md
│   └── legal/
└── .github/workflows/
```

**Règle d'architecture stricte :** `spectra-core` ne dépend de rien d'autre que `serde` et la stdlib. Il ne fait aucune I/O. Toute la logique métier y est testable sans réseau ni disque.

**Une CLI de premier ordre.** `spectra-cli` doit pouvoir faire tout ce que fait l'UI : ouvrir un dossier, lancer des transforms, exporter un rapport. Cela force une séparation propre et rend l'outil scriptable/CI-able. Si une fonctionnalité n'existe qu'en UI, c'est un bug d'architecture.

---

## 5. Modèle de données — le cœur du produit

C'est ici que la plupart des clones de Maltego échouent. Prends le temps.

### Entité

```rust
struct Entity {
    id: EntityId,              // UUIDv7
    kind: EntityKind,          // défini par l'ontologie, extensible
    canonical_value: String,   // normalisé (email en minuscules, tel en E.164…)
    display_label: String,
    properties: BTreeMap<String, PropertyValue>,
    created_at: Timestamp,
    merged_from: Vec<EntityId>,
}
```

### Observation — la clé de la rigueur

Une propriété n'est **jamais** un fait nu. C'est une observation datée et sourcée.

```rust
struct Observation {
    id: ObservationId,
    subject: EntityId,
    predicate: String,
    value: PropertyValue,
    source: Source,               // transform, import manuel, saisie analyste, IA
    method: AcquisitionMethod,
    observed_at: Timestamp,       // quand SPECTRA l'a vu
    valid_from: Option<Timestamp>,// quand c'était vrai dans le monde
    valid_to: Option<Timestamp>,
    confidence: AdmiraltyCode,    // A1..F6
    provenance: Provenance,       // COLLECTED | ASSERTED | INFERRED
    raw_hash: Option<Blake3Hash>, // hash de la réponse brute archivée
    operator: String,
}
```

Conséquence : le graphe affiché est une **projection** des observations à un instant T, pas un stockage direct. Un curseur temporel dans l'UI rejoue l'état du dossier à n'importe quelle date. C'est ce qui sépare un jouet d'un outil d'enquête.

### Ontologie extensible

Types d'entités définis en fichiers déclaratifs (TOML), pas en dur dans le code. Socle minimum : `Person`, `Alias`, `EmailAddress`, `PhoneNumber`, `Username`, `SocialProfile`, `Domain`, `IpAddress`, `Netblock`, `Organization`, `Location`, `Document`, `Image`, `CryptoAddress`, `Device`, `Event`, `Vehicle`.

Chaque type déclare : ses propriétés, sa règle de normalisation, sa règle d'identité (comment savoir que deux instances sont la même), son icône, sa couleur.

### Résolution d'entités

Ne fusionne **jamais** automatiquement. Propose une fusion avec un score et une explication lisible (« même avatar pHash distance 2 ; même bio ; pseudo identique sur 4 plateformes »). La fusion est réversible (`merged_from` conserve la traçabilité).

---

## 6. UX — principes directeurs

Maltego est puissant mais brutal. C'est ton avantage compétitif.

1. **Le canvas graphe est le produit.** Tout le reste est périphérique. Il doit être irréprochable : sélection au lasso, box-select, pan à la molette et espace-drag, zoom centré curseur, undo/redo profond, alignement magnétique, épinglage de nœuds, mini-carte.
2. **Palette de commandes (`Ctrl+K`).** Tout est accessible au clavier. Les analystes travaillent vite.
3. **Expansion progressive.** Clic droit sur un nœud → transforms applicables, groupés, avec estimation du nombre de résultats et du temps. Jamais de « lance 200 modules et vois ce qui tombe ».
4. **Feedback de collecte en direct.** Panneau de tâches montrant chaque requête HTTP en cours, son domaine, son statut, son rate limit. L'analyste doit toujours savoir ce que sa machine envoie sur le réseau, et pouvoir tout couper d'un bouton (« kill switch »).
5. **Distinction visuelle de la provenance.** Trait plein = collecté et vérifié. Pointillé = inféré. Gris = hypothèse de l'analyste. Non négociable.
6. **Mode sombre par défaut**, contraste AA minimum, densité d'information élevée mais respirable.
7. **Aucun état modal bloquant.** Les collectes longues tournent en fond, l'analyste continue de travailler.
8. **i18n dès le départ** : FR et EN, chaînes externalisées, aucun texte en dur dans les composants.

---

## 7. Sécurité, OPSEC et conformité

### OPSEC de l'analyste

- **Profils réseau** : direct / proxy HTTP / SOCKS5 / Tor. Configurables par transform.
- **Rotation de User-Agent** et gestion de jar de cookies isolé par dossier.
- **Détection de fuite** : refuse de démarrer une collecte si le profil réseau est « Tor » mais que le circuit n'est pas établi.
- **Chiffrement au repos** : fichier `.spectra` chiffré XChaCha20-Poly1305, clé dérivée Argon2id. Verrouillage automatique après inactivité.
- **Purge** : effacement sécurisé d'un dossier et de ses caches.

### Sécurité applicative

- CSP Tauri stricte, `dangerousDisableAssetCspModification` interdit.
- Permissions Tauri au minimum nécessaire, capabilities explicites.
- Aucune commande Tauri n'accepte de chemin arbitraire depuis le front sans validation.
- `cargo deny` + `cargo audit` + `npm audit` en CI, build cassé sur vulnérabilité haute.
- Aucun `unsafe` en Rust sans commentaire `// SAFETY:` justifié et revu.
- Fuzzing (`cargo-fuzz`) sur les parseurs (HTML, JSON de sources tierces, format de dossier).

### Conformité — RGPD (l'utilisateur est en France)

À intégrer **dans le produit**, pas en note de bas de page :

- Champ obligatoire au niveau du dossier : **base légale** du traitement (art. 6 RGPD) et **finalité**, saisis à la création.
- **Durée de conservation** paramétrable par dossier, avec alerte à échéance et purge assistée.
- **Minimisation** : avertissement UI quand un transform collecte des catégories particulières (art. 9 : santé, opinions politiques, orientation sexuelle, données biométriques).
- **Journal d'audit** exportable pour répondre à une demande d'accès ou à un contrôle CNIL.
- Écran de première utilisation rappelant le cadre légal, à valider.
- `docs/legal/RESPONSIBLE_USE.md` : SPECTRA est un outil pour journalistes d'investigation, équipes de sécurité, due diligence, recherche de personnes disparues, réponse à incident. Le harcèlement, la traque et la surveillance non consentie sont hors périmètre et l'outil ne doit comporter aucune fonctionnalité qui les facilite spécifiquement (pas de surveillance temps réel d'un individu, pas de géolocalisation continue, pas de scraping automatisé récurrent d'un profil personnel).
- Respect de `robots.txt` et des CGU par défaut ; contournement possible mais avec avertissement explicite et trace dans l'audit.

---

## 8. Qualité — le contrat

| Sujet | Exigence |
|---|---|
| Tests | `spectra-core` et `spectra-correlate` ≥ 90 % de couverture. Tests d'intégration sur `spectra-store` avec base temporaire. |
| Réseau en test | **Aucun test ne touche le réseau.** Tous les transforms sont testés contre des fixtures HTTP enregistrées (`wiremock` ou équivalent). |
| Erreurs | `thiserror` dans les crates bibliothèque, `anyhow` dans les binaires. Jamais de `unwrap()` ou `expect()` hors tests et hors invariants prouvés. |
| Lints | `#![deny(warnings)]` en CI, `clippy::pedantic` activé, exceptions justifiées ligne par ligne. |
| TypeScript | `strict: true`, `noUncheckedIndexedAccess`, zéro `any`. Types partagés générés depuis Rust (`ts-rs` ou `specta`). |
| Perf | Benchmarks `criterion` sur les chemins chauds. Régression > 10 % = build cassé. |
| Commits | Conventional Commits. Un commit = une unité logique cohérente et compilable. |
| ADR | Toute décision d'architecture structurante → un fichier dans `docs/adr/` (contexte, options, décision, conséquences). |
| Docs | Chaque crate a un `//!` de module expliquant son rôle et ses invariants. |

---

## 9. Ce qu'il ne faut PAS faire

- ❌ Envelopper des outils Python (Sherlock, Maigret, Holehe) via `subprocess`. Réimplémente en Rust. La dépendance à un runtime Python tue la promesse « binaire unique, 100 % local ».
- ❌ Embarquer des sources de données illégales ou des dumps de fuites. L'analyste importe les siens s'il les possède légalement.
- ❌ Construire une base de données de personnes. SPECTRA n'agrège rien entre dossiers ; chaque enquête est cloisonnée.
- ❌ Choisir une dépendance non maintenue. Vérifie systématiquement : date du dernier commit, nombre de mainteneurs, ouvertures d'issues, présence d'un fork actif.
- ❌ Empiler des fonctionnalités avant que le canvas graphe et le modèle d'observation ne soient irréprochables.
- ❌ Ajouter du LLM là où une heuristique déterministe suffit.

---

## 10. Références à étudier (lecture, pas copie)

Analyse ces projets pour l'architecture et les formats de données. **Vérifie la licence de chacun avant toute réutilisation de code ou de données.**

- **WhatsMyName** — format JSON de détection de comptes, c'est la donnée de référence
- **Maigret** — extraction de métadonnées, génération de dossier
- **Sherlock** — logique de détection d'existence de compte
- **Blackbird** — architecture asynchrone, capture de preuves
- **Holehe** — technique de vérification email
- **SpiderFoot** — modèle de modules et de corrélation
- **theHarvester**, **Amass**, **PhoneInfoga**, **socid-extractor**
- **OpenCTI**, **MISP** — modèle de données, provenance, marquage (TLP)
- **Maltego** — UX du canvas, modèle de transforms, ce qu'il fait bien ET ce qui frustre
- **IBM i2 Analyst's Notebook** — rigueur analytique, timeline, ACH

Documente ce que tu retiens de chacun dans `docs/prior-art.md`.

---

## 11. Différenciation — pourquoi SPECTRA plutôt que Maltego CE

À garder en tête à chaque arbitrage :

1. **Gratuit et sans limite d'entités** (Maltego CE plafonne à 12 résultats par transform).
2. **Vraiment local** — le dossier ne quitte jamais la machine.
3. **Modèle d'observation temporel et sourcé** — Maltego n'a pas de vraie provenance ni de temporalité.
4. **Plugins WASM multi-langages sandboxés** — plus sûr et plus accessible que les transforms Maltego.
5. **IA locale intégrée** — sans envoyer les données d'enquête à un tiers.
6. **CLI de premier ordre** — automatisable, pas seulement cliquable.
7. **Conformité RGPD par conception** — argument décisif pour les usages européens professionnels.

---

## 12. Plan d'exécution par phases

**Ne passe à la phase suivante que si tous les critères d'acceptation de la phase courante sont verts.** À la fin de chaque phase, produis un court rapport : ce qui est fait, ce qui est reporté, les décisions prises, les risques ouverts.

### Phase 0 — Fondations et décisions
- Vérifie les versions à jour de toutes les dépendances (crates.io, npm, GitHub) et note tout écart avec ce document.
- Scaffold du workspace, CI (build, test, clippy, cargo-deny, audit).
- ADR : stockage, moteur de graphe, système de plugins, backend IA.
- ✅ *Acceptation :* `cargo build --workspace` et `pnpm build` verts sur Linux/macOS/Windows en CI.

### Phase 1 — Cœur du domaine
- `spectra-core` complet : entités, observations, ontologie, projection temporelle.
- `spectra-store` : schéma SQLite, migrations, chiffrement, format `.spectra`.
- `spectra-audit` : journal hash-chaîné.
- ✅ *Acceptation :* ≥ 90 % de couverture sur `core`. Un dossier créé, fermé, rouvert, chiffré/déchiffré sans perte. Le curseur temporel restitue correctement un état passé, prouvé par test.

### Phase 2 — Canvas graphe (le plus risqué, à faire tôt)
- Coquille Tauri + React, IPC typée.
- Canvas Sigma v3, layout ForceAtlas2 en worker, LOD, sélection, undo/redo.
- Benchmark de charge avec graphe synthétique.
- ✅ *Acceptation :* 50 000 nœuds / 150 000 arêtes, ≥ 45 fps en pan/zoom, mémoire < 1,5 Go. Si l'objectif n'est pas atteint, **arrête-toi et propose des options** (clustering, vue overview/détail, pré-layout Rust) avant de continuer.

### Phase 3 — Moteur de transforms
- Trait `Transform`, orchestrateur, DAG, rate limiting par domaine, cache de réponses, annulation.
- Runtime Extism, host functions réseau contrôlées, manifeste, signature.
- 3 transforms natifs : DNS, WHOIS, Certificate Transparency (crt.sh).
- ✅ *Acceptation :* un transform WASM d'exemple s'installe, s'exécute, respecte son allowlist, et un test prouve qu'un appel hors allowlist est bloqué.

### Phase 4 — Pilier A : recherche de personne
- Moteur WhatsMyName natif en Rust, async, avec rate limit et détection de faux positifs.
- Transforms email (MX, patterns, forgot-password), téléphone, extraction de métadonnées de profil.
- Moteur de corrélation : pHash d'avatars, similarité de bio, scoring, propositions de fusion explicables.
- ✅ *Acceptation :* depuis un pseudo, un dossier peuplé en < 60 s avec provenance complète sur chaque observation. Jeu de test avec des identités connues et publiques (comptes de démonstration créés pour l'occasion), taux de faux positifs mesuré et documenté.

### Phase 5 — Pilier B : système d'enquêtes
- Timeline, carte, vue table, notes Markdown, hypothèses ACH.
- Générateur de rapports (PDF/DOCX/HTML), gabarits, annexes de preuves.
- Export/import de fragments, fusion CRDT.
- ✅ *Acceptation :* un rapport PDF complet généré depuis un dossier réel, avec annexes hashées et journal d'audit joint.

### Phase 6 — IA locale
- Trait `LlmBackend`, détection d'Ollama, fallback mistral.rs, embeddings `fastembed-rs`.
- NER, résumé, suggestion de pivot, déduplication sémantique, RAG sur les notes.
- ✅ *Acceptation :* toutes les fonctions IA se dégradent proprement à « indisponible » sans Ollama, l'application reste 100 % fonctionnelle. Toute sortie IA est marquée `INFERRED` et exclue des rapports par défaut.

### Phase 7 — Finition et distribution
- i18n FR/EN, accessibilité, palette de commandes, raccourcis.
- Empaquetage : AppImage/deb/rpm, `.dmg` signé, MSI/NSIS, updater Tauri **auto-hébergeable**.
- Documentation utilisateur, SDK de plugins, guide de contribution, politique de sécurité.
- ✅ *Acceptation :* installation propre sur les trois OS depuis un artefact CI, sans dépendance externe hors Ollama (optionnel).

---

## 13. Protocole de travail avec moi

1. **Commence par lire ce fichier en entier, puis pose-moi les questions ouvertes** avant d'écrire du code. Je préfère 5 bonnes questions à 500 lignes à jeter.
2. **Vérifie tout par recherche.** Ce document date de juillet 2026. Les versions bougent, des projets meurent (cf. Kùzu). Ne fais confiance à aucun numéro de version sans l'avoir vérifié.
3. **Sois franc sur les compromis.** Si un objectif de ce document est irréaliste (les 50 k nœuds, la réimplémentation Rust de Maigret, la fusion CRDT), dis-le tôt et propose une alternative chiffrée. Ne fais pas semblant.
4. **Une phase à la fois.** N'anticipe pas sur les phases suivantes, ne crée pas de fichiers « pour plus tard ».
5. **Explique tes décisions non évidentes** en une à deux phrases dans le commit ou l'ADR, jamais dans des commentaires bavards.
6. **Si tu bloques**, arrête-toi et expose le problème plutôt que de contourner par un hack silencieux.

---

## 14. Premier message attendu de ta part

Ne code rien. Réponds avec :

1. Les versions réelles vérifiées aujourd'hui de : Tauri, React, Sigma.js, graphology, Extism, tantivy, rusqlite, sqlite-vec, ollama-rs, fastembed-rs — et tout écart avec ce document.
2. L'état actuel de LadybugDB (maturité, bindings Rust, activité du dépôt) et ta recommandation ferme : SQLite+petgraph seul, ou avec LadybugDB en option.
3. Tes 5 questions les plus importantes sur des ambiguïtés de ce cahier des charges.
4. Les 3 risques techniques que tu juges les plus élevés, avec pour chacun une stratégie de mitigation et un moment où on saura si ça passe ou non.
5. Un plan détaillé de la Phase 0 uniquement, découpé en tâches livrables.