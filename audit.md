# Audit technique — Cekarna (logiciel OSINT d'enquêtes)

**Date de l'audit** : 2026-07-28
**Version auditée** : 0.1.0 (état du répertoire de travail, hors contrôle de version)
**Auditeur** : revue de code statique + vérification de compilation
**Périmètre** : `src-tauri/` (backend Rust), `lib/` + `types/` (couche TypeScript), `app/` + `components/` (frontend), configuration de build, documentation projet

---

## 1. Résumé exécutif

Cekarna se présente comme une application de bureau (Tauri 2 + Next.js 16) destinée à gérer des
enquêtes OSINT avec **preuves horodatées, vérification d'intégrité par hachage, journal d'audit
immuable chaîné en SHA-256 et instantanés (snapshots) vérifiables**. C'est-à-dire un outil dont la
valeur repose entièrement sur la **fiabilité probatoire** de ce qu'il enregistre.

Le verdict de l'audit est net : **l'application n'est pas fonctionnelle, et les garanties
probatoires qu'elle annonce ne sont pas implémentées.** Il ne s'agit pas d'un logiciel
« à corriger » mais d'un squelette d'architecture (schéma de base de données + signatures de
commandes) accompagné d'une documentation qui décrit un produit fini qui n'existe pas.

Trois constats structurants :

1. **Le backend Rust ne compile pas.** `cargo check` remonte **127 erreurs de compilation**
   (vérifié, voir §7). Aucune des 43 commandes déclarées n'est donc exécutable aujourd'hui.
2. **Le frontend n'existe pas.** Les répertoires `app/` et `components/` sont **vides** (0 fichier),
   alors que la documentation décrit 12 routes et 12 familles de composants. Le build Tauri pointe
   sur `../out`, qui ne contient aucun `index.html`.
3. **Les fonctions probatoires sont factices.** La « vérification d'intégrité » ne relit jamais les
   fichiers, le « hash d'audit » est parfois une chaîne de caractères littérale, le chaînage n'est
   pas un chaînage, et les snapshots ne capturent aucun contenu. Un enquêteur obtiendrait
   « Hash verified successfully » sur une preuve modifiée ou supprimée du disque.

Sur ce dernier point l'exposition est plus sérieuse qu'un simple bug : un outil qui affiche une
intégrité **fausse** est plus dangereux qu'un outil qui n'affiche rien, parce qu'il produit une
confiance injustifiée sur des éléments susceptibles d'être versés à un dossier.

### Tableau de synthèse

| Sévérité | Nombre | Nature |
|---|---|---|
| **P0 — Bloquant** | 6 | Le logiciel ne compile pas / ne démarre pas / n'a pas d'interface |
| **P1 — Critique** | 11 | Garanties probatoires fausses, perte de données, données personnelles non protégées |
| **P2 — Majeur** | 14 | Bugs fonctionnels certains à l'exécution, contrats de données incohérents |
| **P3 — Mineur** | 9 | Outillage, dette technique, documentation trompeuse |

### Maturité par domaine

| Domaine | État | Note |
|---|---|---|
| Schéma de données (SQLite) | Structuré, cohérent dans l'ensemble | 6/10 |
| Backend Rust (commandes) | Ne compile pas ; logique souvent incomplète | 2/10 |
| Chaîne de custody / intégrité | Simulée, non implémentée | 1/10 |
| Journal d'audit | Non conforme à sa propre spécification | 1/10 |
| Couche TypeScript (`lib/`) | ~20 modules « mock » retournant des valeurs vides | 2/10 |
| Interface utilisateur | Inexistante | 0/10 |
| Sécurité / confidentialité | Aucun contrôle d'accès, aucun chiffrement, pas de CSP | 1/10 |
| Tests | Zéro test (répertoire vide, pas de script `test`) | 0/10 |
| Documentation | Abondante mais décrit un produit inexistant | 3/10 (fiabilité) |

---

## 2. Méthode et éléments vérifiés

* Lecture intégrale de `src-tauri/src/` (13 fichiers Rust), `lib/` (27 fichiers TS), `types/`,
  et des fichiers de configuration.
* Exécution réelle de `cargo check` sur le crate `cekarna` (résultat en §7).
* Exécution réelle de `npx tsc --noEmit` (résultat : **succès**, mais sur un périmètre réduit à
  quelques fichiers utilitaires — voir P3-3).
* Inventaire des fichiers présents/absents (`app/`, `components/`, `docs/`, tests, licence).
* Comparaison systématique documentation ↔ code (`PROJECT_SUMMARY.md`, `MEMORY.md`, `PROJET.md`).

Ce qui **n'a pas** pu être audité, faute d'exister : le comportement à l'exécution, l'ergonomie,
l'accessibilité, les performances, l'intégration Ollama réelle, les collecteurs TikTok / archives web.

---

## 3. P0 — Bloquants

### P0-1 — Le backend Rust ne compile pas (127 erreurs)

`cargo check` échoue. Les erreurs ne sont pas marginales, elles sont **systémiques** : elles
touchent les 43 commandes et le type d'état partagé.

Causes racines, par ordre d'impact :

| # | Cause | Occurrences | Correctif |
|---|---|---|---|
| a | Les commandes retournent `anyhow::Result<T>` ; Tauri exige un type d'erreur `Serialize` (`error[E0599]: the method async_kind exists ... but its trait bounds were not satisfied`) | ~43 | Définir un `AppError` avec `thiserror` + `impl Serialize`, et `Result<T, AppError>` |
| b | `serde_json::Value` utilisé comme paramètre/colonne SQL sans la feature `serde_json` de rusqlite (`the trait ToSql/FromSql is not implemented for serde_json::Value`) | ~20 | `rusqlite = { version = "0.31", features = ["bundled", "serde_json"] }` |
| c | `#[derive(Clone)]` sur `AppState` qui contient un `Mutex<Connection>` non clonable — `database.rs:17-20` | 1 | Retirer `Clone` ; partager via `State<'_, AppState>` |
| d | `serde_json::Map<String, i64>` : ce type n'existe pas (la `Map` de serde_json est `Map<String, Value>`) — `claims.rs:45-46`, `reports.rs:69-70` | 6+ | `HashMap<String, i64>` |
| e | `.run()` appelé sans contexte — `lib.rs:98` (`this method takes 1 argument but 0 arguments were supplied`) | 1 | `.run(tauri::generate_context!())` |
| f | `conn.query_map(...)` appelé sur la connexion au lieu d'un `Statement` — `cases.rs:142,149`, `reports.rs:310,321`, `claims.rs:251,263` | 6 | `conn.prepare(...)?.query_map(...)` |
| g | Emprunts de temporaires dans les `UPDATE` dynamiques (`params.push(&Utc::now().to_rfc3339())`) — `cases.rs:253,261`, `reports.rs:238` | 3 | Lier les valeurs dans des variables locales |
| h | `Vec<&dyn ToSql>` passé comme `Params` — `reports.rs:243` | 2 | `rusqlite::params_from_iter(...)` |
| i | Futures non `await` puis `.map()` — `cases.rs:213,276` | 2 | `.await` avant `.map()` |
| j | Valeurs déplacées puis réutilisées (`data.statut.unwrap_or_else` deux fois) — `subjects.rs` | 2 | Cloner ou calculer une fois |
| k | Tableaux de longueurs différentes dans les branches `if/else` puis `as _` — `cases.rs:72-78` | 1 | `params_from_iter` |

### P0-2 — `main()` défini dans la bibliothèque et privé

`src-tauri/src/main.rs` appelle `cekarna::main()`, mais `lib.rs:29` déclare `fn main()` (non `pub`).
Le point d'entrée Tauri 2 attendu est `pub fn run()` dans `lib.rs`, appelé par `main.rs`.

### P0-3 — Aucune interface utilisateur

`app/` et `components/` contiennent **0 fichier**. `PROJECT_SUMMARY.md` et `PROJET.md` décrivent
pourtant `app/cases`, `app/reports`, `app/search`, `app/settings`, `app/collect/tiktok`, etc., et
12 familles de composants (`components/evidence`, `components/integrity`, `components/audit`…).
Seule l'arborescence de répertoires vides subsiste.

Conséquence : `next build` avec `output: 'export'` ne peut produire aucune page ; `out/` ne contient
qu'un dossier `_next` (aucun `index.html`).

### P0-4 — La cible de build Tauri est vide

`tauri.conf.json` : `"frontendDist": "../out"`. Sans `index.html`, la fenêtre Tauri ouvrira une page
blanche. Aucun `beforeBuildCommand`/`beforeDevCommand` n'est configuré : le build frontend n'est
jamais déclenché par `npm run tauri build`.

### P0-5 — Les commandes de réglages IA n'existent pas côté Rust

`lib/tauri-bridge.ts:381-405` appelle `get_ai_settings`, `save_ai_settings`,
`get_available_ollama_models`, `test_ollama_connection`. Aucun fichier `commands/settings.rs`
n'existe et aucune de ces commandes n'est enregistrée dans `lib.rs`. Tout accès à l'écran Réglages
échouerait par `Command not found`.

`PROJECT_SUMMARY.md` annonce néanmoins un module « settings | 4 commandes ».

### P0-6 — Cache de build corrompu par un déplacement de projet

Le premier `cargo check` a échoué avant même la compilation :

```
failed to read plugin permissions: failed to read file
'...\htdoc\Biscuits IA\cekarna - enquetes\src-tauri\target\...\permissions\app\autogenerated\commands\app_hide.toml'
```

Le projet a été déplacé depuis `htdoc\Biscuits IA\` sans nettoyage ; `target/` et
`src-tauri/gen/schemas/` contiennent des chemins absolus obsolètes. Correctif : `cargo clean`
(effectué durant l'audit pour obtenir les erreurs réelles).

---

## 4. P1 — Critiques : les garanties probatoires ne tiennent pas

C'est le cœur du problème. Chaque promesse du produit est examinée ci-dessous.

### P1-1 — La vérification d'intégrité ne vérifie rien

`src-tauri/src/commands/integrity.rs:30`

```rust
let verified = hash_sha256.as_ref().map(|h| !h.is_empty()).unwrap_or(false);
```

`verify_evidence` **ne lit jamais le fichier** désigné par `evidence.chemin` et ne recalcule aucun
hachage. « Vérifié » signifie uniquement : *une chaîne non vide est présente dans la colonne
`hash_sha256`*. Le message retourné est pourtant `"Hash verified successfully"`.

Scénario d'échec : une preuve dont le fichier a été modifié, remplacé ou supprimé du disque est
rapportée comme intacte. `verify_case_evidence` (`integrity.rs:61-65`) reproduit le même comptage et
qualifie de `broken_count` les preuves *sans hash*, ce qui masque les vraies altérations dans un
indicateur qui ne mesure pas ce qu'il prétend mesurer.

À noter : la logique correcte existe côté TypeScript (`lib/hashes.ts:14-26`, `verifyFileHash` avec
états `intact | altered | missing`) mais n'est jamais appelée, et utilise `fs`/`crypto` Node
indisponibles dans un webview.

### P1-2 — Les hachages de preuve sont fournis par l'appelant, jamais calculés

`commands/evidence.rs:88-152` : `add_evidence` insère `data.hash_sha256` et `data.hash_md5` tels que
transmis par le client. Le backend ne lit pas le fichier, ne vérifie pas son existence, ne le copie
pas dans un magasin contrôlé, ne contrôle pas la taille (`max_evidence_size_mb: 100` dans
`APP_CONFIG`, `lib.rs:13-22`, est une constante morte).

Conséquence probatoire : le hash n'atteste rien puisqu'il n'a jamais été observé par le système au
moment de l'ingestion. La preuve reste en outre un chemin vers un fichier **externe et mutable**
(`lib/storage.ts` définit un magasin `.storage/` … qui n'est jamais créé ni utilisé).

### P1-3 — Le « hash d'audit » est une chaîne de caractères littérale

`commands/cases.rs:207`

```rust
&format!("hex(sha256('create:case:{}:{}'))", id, now),
```

La colonne `imma` (le hash d'audit) reçoit le **texte** `hex(sha256('create:case:<id>:<date>'))`.
Aucun SHA-256 n'est calculé. Tout événement d'audit de création de dossier est donc porteur d'un
faux hash, et `verify_audit_trail` le considère valide puisqu'il se contente de vérifier qu'il est
non vide (`audit.rs:83`).

### P1-4 — Le chaînage d'audit n'est pas un chaînage

Trois défauts cumulés :

* `commands/audit.rs:198` et `cases.rs:208` insèrent la **chaîne `"NULL"`** (et non `NULL` SQL) dans
  `imma_precedent`. Au premier événement, `verify_audit_trail` comparera `Some("NULL")` à `None` et
  déclarera la chaîne rompue : la vérification est donc à la fois laxiste (P1-3) et faussement
  alarmante.
* `database.rs:672-688` : le déclencheur `audit_update` calcule `imma_precedent` comme un hash des
  *anciennes valeurs de la ligne*, au lieu de reprendre l'`imma` de l'événement précédent. Un
  chaînage doit référencer le maillon antérieur ; ici chaque maillon est indépendant.
* `audit.rs:66-94` : la vérification **ne recalcule jamais** `imma` à partir du contenu de
  l'événement. Modifier `action`, `actor` ou `metadata` en base ne casse rien. Et le lien n'est
  contrôlé que si `imma_precedent` est renseigné (`if let Some(prev)`), donc mettre ce champ à NULL
  suffit à faire passer une suppression de maillons.

Le terme « journal immuable » utilisé dans la documentation est, en l'état, sans fondement : rien
n'empêche l'édition directe du fichier SQLite et rien ne la détecterait.

### P1-5 — Les déclencheurs d'audit SQL sont inopérants

`database.rs:653-691` :

* `sha256()` **n'est pas une fonction SQLite standard** (rusqlite `bundled` ne l'expose pas). Les
  déclencheurs `audit_insert` / `audit_update` échoueront à l'exécution avec `no such function:
  sha256`, faisant échouer **toute insertion ou mise à jour de dossier**.
* La génération d'identifiant `printf('%06d', (SELECT COALESCE(MAX(id),0)+1 FROM audit_events ...))`
  applique `MAX()` puis une arithmétique sur des identifiants **textuels** (`AE-2026-000001`) :
  le résultat vaut toujours `1`, donc le deuxième événement viole la clé primaire.

### P1-6 — Identifiants d'événements codés en dur → collision garantie

`evidence.rs:166` et `254`, `subjects.rs`, `snapshots.rs:118`, `claims.rs:219` :

```rust
&format!("CE-{}-000001", Utc::now().year()),
```

Le compteur est figé à `000001`. Le **deuxième** ajout de preuve, de sujet, de snapshot ou la
deuxième suppression de claim de l'année échouera sur `UNIQUE constraint failed: case_events.id`.
Comme aucune écriture n'est encapsulée dans une transaction (voir P2-1), l'insertion métier aura
déjà été validée : la base se retrouve avec une preuve **sans** son événement de traçabilité.

### P1-7 — Les snapshots ne capturent pas le dossier

`commands/snapshots.rs:88-97` : le JSON haché ne contient que `id`, `case_id`, `nom`, `description`,
`timestamp`, `metadata` — c'est-à-dire les métadonnées du snapshot lui-même. Aucune preuve, aucun
sujet, aucune claim, aucun événement n'est exporté. La table `snapshot_contents` n'est jamais
alimentée, `chemin` reçoit la chaîne `"NULL"` (`snapshots.rs:109`), et le hash — calculé sur un UUID
aléatoire et un horodatage — est unique par construction et n'atteste de rien.

`verify_snapshot_integrity` (`snapshots.rs:250-256`) ne recalcule rien non plus : `hash_valid` vaut
`!hash.is_empty()`.

Un « instantané vérifiable » qui ne contient pas l'état qu'il prétend figer ne remplit aucune de ses
fonctions (comparaison, restauration, preuve de non-modification).

### P1-8 — Suppressions définitives et non tracées, y compris du dossier entier

* `delete_case` (`cases.rs:283-298`) : `DELETE FROM cases`. Avec les `ON DELETE CASCADE` du schéma,
  cela détruit **preuves, sujets, claims, rapports, snapshots, journaux d'intégrité et
  `audit_events` du dossier**. Aucune trace, aucune confirmation, aucune restauration possible.
* `delete_evidence`, `delete_snapshot`, `delete_report`, `delete_claim` : suppressions physiques,
  sans écriture dans `audit_events` (seulement, parfois, dans `case_events`).
* Le schéma prévoit pourtant `statut = 'supprime'` pour les preuves (`database.rs:208`) : la
  suppression logique était prévue puis abandonnée.

Pour un outil d'enquête, la suppression physique d'éléments et de leur journal d'audit est
inacceptable : elle rend indémontrable l'intégrité du dossier restant.

### P1-9 — Données personnelles en clair, sans contrôle d'accès

* La table `subjects` (`database.rs:218-237`) stocke nom, prénom, date et lieu de naissance,
  nationalité, téléphone, e-mail, adresse — des données personnelles sensibles portant sur des
  personnes qualifiées de « suspect », « victime », « témoin ».
* **Aucun chiffrement au repos** : SQLite brut, pas de SQLCipher, pas de chiffrement applicatif.
* **Aucune authentification** : les tables `users` (avec rôles `admin/user/viewer`) et
  `session_logs` existent, mais aucune commande ne les lit ou ne les écrit. Toute personne ayant
  accès à la session du poste a un accès total.
* **Aucune politique de conservation** : `audit_settings.retentionDays = 365` n'est jamais appliqué.
* `fts_subjects` duplique e-mails et téléphones dans un index de recherche. Comme **aucun
  déclencheur FTS n'existe pour `subjects`** (les déclencheurs ne couvrent que `cases`, `evidence`,
  `tiktok_videos` — `database.rs:576-647`), la suppression d'un sujet **laisse ses données
  personnelles dans l'index FTS indéfiniment**. C'est un défaut d'effacement au sens du RGPD, en
  plus d'une incohérence de données.

### P1-10 — La base est écrite dans le répertoire courant

`lib.rs:31` : `Path::new("casetrack.db")`. Chemin **relatif** au répertoire de lancement, alors que
la documentation annonce « casetrack.db in app data ». Selon la manière de lancer l'application, les
dossiers d'enquête finissent dans des emplacements différents, potentiellement dans un répertoire
synchronisé ou temporaire, et deux lancements depuis des répertoires distincts créent deux bases
disjointes silencieusement.

De plus `AppState::new()` (`database.rs:23-29`) ouvre une base **en mémoire** avec `.expect()` ; en
cas d'échec d'initialisation, `lib.rs:32-34` panique au démarrage.

### P1-11 — Aucune CSP dans la configuration Tauri

`tauri.conf.json` → `app.security` ne contient que `capabilities`. Sans `csp`, aucune politique de
sécurité de contenu n'est appliquée au webview. Pour une application qui a précisément vocation à
**ingérer et afficher du contenu web collecté** (archives web, descriptions TikTok, contenu
transcrit), c'est le vecteur XSS le plus direct : du HTML/JS provenant d'une page archivée s'exécute
dans un contexte qui a accès à `invoke()`, donc à toute la base d'enquête.

À combiner avec `"shell:allow-open"` (`capabilities/default.json`) : l'ouverture d'URL issues de
données collectées doit être filtrée (schémas `file:`, `javascript:`, etc.).

---

## 5. P2 — Majeurs : bugs certains à l'exécution et contrats incohérents

### P2-1 — Aucune transaction sur les écritures multi-tables

Toutes les commandes d'écriture enchaînent 2 à 4 `INSERT`/`UPDATE`/`DELETE` sans
`BEGIN`/`COMMIT` (`evidence.rs`, `subjects.rs`, `snapshots.rs`, `cases.rs`, `reports.rs`,
`claims.rs`). Le moindre échec intermédiaire (et P1-6 en garantit un) laisse la base dans un état
partiel : preuve sans événement, dossier sans audit, rapport sans index FTS.

### P2-2 — Les tables FTS5 ne peuvent pas fonctionner comme configurées

`database.rs:501-574` : chaque table virtuelle déclare `content = '<table>'` et
`content_rowid = 'id'`, alors que les colonnes `id` sont des **UUID textuels**. FTS5 exige un
`rowid` **entier**. Toute insertion (`INSERT INTO fts_evidence (rowid, ...) VALUES ('<uuid>', ...)`)
produira une erreur de type de données. Le mode « external content » impose par ailleurs de ne pas
insérer directement le contenu, ce que fait pourtant le code.

Corollaires : `search_fts_table` (`search.rs:98-99`) lit `rowid` en `i64` et l'utilise comme
identifiant d'entité (`id: rowid.to_string()`), ce qui ne correspond à aucun UUID existant.

### P2-3 — La recherche interroge des colonnes qui n'existent pas

`search.rs:93-96` exécute pour **toutes** les tables :

```sql
SELECT rowid, nom, description FROM <fts_table> WHERE <fts_table> MATCH ?
```

Or `fts_cases` et `fts_reports` ont les colonnes `reference, titre, description` (pas `nom`),
`fts_notes` a `titre, contenu`, `fts_web_archives` a `url, title, content`. Cinq des sept branches
échoueront sur `no such column: nom`.

### P2-4 — Le filtre par dossier de la recherche est inopérant

`search.rs:112-116` :

```rust
if let Some(ref cid) = case_id {
    if &cid != cid.as_ref().map_or("", |c| c) { continue; }
}
```

La variable `cid` du `if let` masque celle de la boucle : la condition compare une valeur à
elle-même. Le paramètre `case_id` n'a donc aucun effet — une recherche « dans ce dossier » renvoie
des résultats de tous les dossiers. C'est en outre l'une des erreurs de compilation relevées
(`E0282` à `search.rs:113`).

Par ailleurs `score` est toujours `0.0` (`search.rs:124`), donc le tri par pertinence
(`search.rs:79`) est arbitraire, et la troncature à 50 résultats survient **après** ce tri
arbitraire.

### P2-5 — `update_report` écrit dans une colonne inexistante

`reports.rs:237` ajoute `dateMiseJour = ?` alors que la table `reports` (`database.rs:258-275`) n'a
pas cette colonne (elle a `updated_at`). **Toute** mise à jour de rapport échouera sur
`no such column: dateMiseJour`.

### P2-6 — `get_cases` masque silencieusement des dossiers

`cases.rs:61-78` : dès que `search` est renseigné, la requête impose `statut = ?`, et `statut_val`
retombe sur `"ouvert"` si le filtre de statut n'est pas fourni (`cases.rs:69`). Une recherche sans
filtre de statut ne renvoie donc **que** les dossiers ouverts, sans que rien ne l'indique à
l'utilisateur — un dossier clos devient introuvable.

### P2-7 — Contrats de données incompatibles entre les trois couches

Aucune structure Rust ne porte `#[serde(rename_all = "camelCase")]` (vérifié : zéro occurrence dans
`src-tauri/src`). Conséquences :

* **Entrées** : `lib/tauri-bridge.ts:116-124` envoie `data: { caseId, sourceUrl, … }` ; le
  `AddEvidenceInput` Rust attend `case_id`, `source_url`. La conversion automatique camelCase →
  snake_case de Tauri s'applique aux **arguments de commande**, pas aux champs des structures
  imbriquées → échec de désérialisation.
* **Sorties** : Rust renvoie `case_id`, `date_ajout`, `hash_sha256` ; le type TS attend `caseId`,
  `dateImport`, `sha256`.
* Trois formes incompatibles coexistent pour la même entité :

| | `types/index.ts:86-97` | Rust `evidence.rs:11-27` | Colonnes SQL |
|---|---|---|---|
| nom du fichier | `filename` | `nom` | `nom` |
| taille | `tailleOctets` | `taille` | `taille` |
| type MIME | `typeMime` | `type` (métier) | `type` (métier) |
| hash | `sha256` / `md5` | `hash_sha256` / `hash_md5` | idem Rust |
| date | `dateImport` | `date_ajout` | `dateAjout` |

Rien ne garantit aujourd'hui la cohérence de ces trois définitions : c'est une source de bugs
permanente.

### P2-8 — Interblocages et double verrouillage du mutex

* `cases.rs:172` : `generate_case_reference_locked(&state.conn)` verrouille le mutex **alors que
  `conn` le détient déjà** (`cases.rs:168`) → interblocage garanti (un `tokio::sync::Mutex` n'est
  pas réentrant). Accessoirement, `AppState.conn` est privé : l'accès depuis un autre module est
  aussi une erreur de compilation.
* `snapshots.rs:313` : `load_snapshot_bundle` appelle `get_snapshot(state, id)` en tenant déjà le
  verrou → même problème.
* Plus généralement, le verrou est pris pour toute la durée de la commande, y compris pendant les
  parcours de résultats : une seule opération à la fois, quelle que soit sa durée.

### P2-9 — Doublons d'indexation FTS

Les commandes insèrent manuellement dans les tables FTS (`evidence.rs:154-160`, `cases.rs:193-196`,
`reports.rs:191-194`, `subjects.rs`) **alors que** des déclencheurs `AFTER INSERT` font déjà le
travail pour `cases` et `evidence` (`database.rs:578-621`). Chaque preuve serait indexée deux fois.
Et l'indexation manuelle est conditionnée à `if let Some(nom)` : une preuve sans nom n'est pas
indexée du tout.

### P2-10 — Déclencheurs FTS absents pour 4 entités sur 7

Déclencheurs présents : `cases`, `evidence`, `tiktok_videos`. Absents : `subjects`, `reports`,
`web_archives`, `notes`. Ces index se désynchronisent dès la première modification ou suppression
(voir aussi P1-9 pour l'impact « données personnelles »).

### P2-11 — Génération de références sujette aux courses et aux réemplois

`database.rs:750-794` : `MAX(CAST(SUBSTR(reference, -4) AS INTEGER)) + 1`. Après suppression du
dernier dossier de l'année, la référence est **réattribuée** — deux dossiers différents partagent
alors la même référence dans le temps, ce qui est rédhibitoire pour un identifiant de dossier cité
dans des documents. Passé `ENQ-YYYY-9999`, `SUBSTR(-4)` tronque et le compteur repart faux.

### P2-12 — `set_claim` ignore le dossier

`claims.rs:73-96` : l'existence puis la mise à jour se font sur `(refKind, refId)` **sans**
`caseId`. Deux dossiers partageant un identifiant de référence se contaminent mutuellement. Un
`UPDATE` peut donc modifier la qualification d'un élément d'un **autre** dossier.

De plus, changer la qualification (`preuve` → `hypothese`) ou la fiabilité **écrase** la valeur
précédente sans aucun historique ni événement d'audit : l'évolution du raisonnement de l'enquêteur,
qui est une information probatoire en soi, est perdue.

### P2-13 — Journal d'audit incomplet par construction

Seuls la création/modification de dossier (par déclencheur, inopérant — P1-5) et `log_access`
écrivent dans `audit_events`. **Aucune** commande n'y écrit pour : ajout/suppression de preuve,
création/modification/suppression de sujet, claims, rapports, snapshots. Le journal d'audit
n'observe donc pas les opérations qui ont le plus besoin d'être auditées.

### P2-14 — Ordonnancement du journal d'audit non fiable

`audit.rs:46` trie par `timestamp ASC`, mais les horodatages sont écrits dans **deux formats
différents** : `datetime('now')` (`"2026-07-28 11:30:00"`, défaut SQL) et
`Utc::now().to_rfc3339()` (`"2026-07-28T11:30:00+00:00"`, code Rust). Le tri lexicographique mélange
les deux (l'espace précède `T`), et la granularité à la seconde rend l'ordre indéterminé pour des
événements rapprochés. Un chaînage dont l'ordre est indéterminé n'est pas vérifiable.

---

## 6. P3 — Mineurs : outillage, dette, documentation

| # | Constat | Détail |
|---|---|---|
| P3-1 | **Aucun test** | `lib/__tests__/` est vide ; `vitest` est configuré mais `package.json` n'a pas de script `test` ; aucun `#[cfg(test)]` dans le code Rust. `PROJET.md:419-436` documente pourtant `cargo test --test test_audit_hash`, `npx vitest`, etc. `lib/canonical.ts:6` référence un test Rust `audit::tests::test_canonical_hash_matches_ts` **qui n'existe pas** — or c'est précisément le test qui garantirait que les hachages TS et Rust concordent. |
| P3-2 | **Pas de contrôle de version** | Le répertoire n'est pas un dépôt Git. Pour un outil à visée probatoire, l'absence de traçabilité du code lui-même est un problème de gouvernance autant que de développement. Pas de `.gitignore` non plus, alors que `target/` (251 Mio), `.next/`, `out/` et `node_modules/` sont présents. |
| P3-3 | **Le typecheck TS est trompeusement vert** | `npx tsc --noEmit` passe — mais il n'y a ni composant ni page à vérifier, et `skipLibCheck: true`. Ce succès ne dit rien de la santé du projet. |
| P3-4 | **Lint non configuré** | Aucun fichier de configuration ESLint ; `npm run lint` (`eslint` sans cible) échouera. |
| P3-5 | **Dépendance PostgreSQL inutile** | `pg` et `@types/pg` dans `package.json` pour une application locale SQLite : client réseau embarqué sans usage, à retirer (surface d'attaque et poids inutiles). |
| P3-6 | **Plugins Tauri déclarés mais non initialisés** | `tauri-plugin-shell` et `tauri-plugin-dialog` sont dans `Cargo.toml` et leurs permissions dans `capabilities/default.json`, mais `lib.rs` n'appelle jamais `.plugin(tauri_plugin_shell::init())`. `MEMORY.md` mentionne en outre des plugins `store` et `updater` totalement absents. |
| P3-7 | **`createUpdaterArtifacts: true` sans updater** | `tauri.conf.json` demande la génération d'artefacts de mise à jour alors qu'aucun plugin updater ni clé publique n'est configuré → échec de bundling. |
| P3-8 | **Licence annoncée, fichier absent** | GPL-3.0 déclarée dans `Cargo.toml` et `PROJECT_SUMMARY.md`, aucun fichier `LICENSE` à la racine. `repository` pointe vers `github.com/cekarna/enquetes`, non vérifiable. |
| P3-9 | **Répertoires et documents fantômes** | `docs/`, `docs/superpowers/plans/`, `scripts-archive/`, `public/` sont vides. Le code mort inclut `APP_CONFIG` (`lib.rs:13`), `AppState::execute`/`get_mutex` (jamais appelés), `create_tags_tables` qui **ignore le résultat** de `execute_batch` (`database.rs:725-733`, erreur silencieuse). |

### P3-10 — La couche `lib/` est constituée de simulacres

Ce point est classé ici parce qu'il ne s'agit pas d'un bug mais d'un état d'avancement — il est
toutefois **le plus trompeur** du projet. Une vingtaine de modules `lib/*.ts` annoncent en
commentaire « Mock … module for development » et renvoient des valeurs vides ou fabriquées :

| Module | Comportement réel |
|---|---|
| `lib/audit.ts:39` | `verifyAuditTrail()` retourne **toujours** `{ ok: true, total: 0 }` |
| `lib/audit.ts:29` | `rowHash: crypto.randomUUID()` — un UUID aléatoire en guise de hash |
| `lib/evidence.ts:68-83` | `verifyCaseEvidence()` retourne un rapport « 0 altéré, 0 manquant » |
| `lib/cases.ts`, `subjects.ts`, `claims.ts`, `search.ts`, `tiktok.ts`, `web-archives.ts`, `cross-links.ts`, `transcriptions.ts`, `ai-extractions.ts`, `settings.ts` | listes vides / objets fabriqués localement |

Deux implémentations concurrentes coexistent pour les mêmes fonctions : `lib/<domaine>.ts` (mock) et
`lib/tauri-bridge.ts` (invoke réel). **Le résultat dépend de l'import choisi par l'appelant.** Dans
un outil probatoire, un module de secours qui répond « audit intègre, aucune preuve altérée » est la
catégorie de défaut la plus dangereuse : il produit exactement le message rassurant attendu, sans
rien vérifier. Ces modules doivent être supprimés, pas conservés « pour le développement ».

---

## 7. Preuve : sortie de `cargo check`

Après `cargo clean -p tauri -p cekarna` (nécessaire, cf. P0-6) :

```
error: could not compile `cekarna` (lib) due to 127 previous errors; 11 warnings emitted
```

Répartition des erreurs par fichier (les 127 se ramènent aux 11 causes racines du P0-1) :

| Fichier | Erreurs |
|---|---|
| `commands/reports.rs` | ~25 |
| `commands/claims.rs` | ~13 |
| `commands/cases.rs` | ~9 |
| `commands/snapshots.rs` | ~8 |
| `commands/evidence.rs` | ~5 |
| `commands/subjects.rs` | ~6 |
| `commands/audit.rs` | 3 |
| `commands/search.rs` | 3 |
| `commands/integrity.rs`, `events.rs` | 2 chacun |
| `database.rs`, `lib.rs` | 1 chacun |
| *(le reste : `async_kind` sur chacune des 43 commandes)* | ~43 |

---

## 8. Plan de remise en état recommandé

L'ordre importe : il est inutile de corriger la logique probatoire d'un code qui ne compile pas, et
inutile de bâtir une interface sur un backend dont les garanties sont fausses.

### Lot 0 — Retrouver un projet compilable (prérequis absolu)

1. `cargo clean` ; supprimer `src-tauri/gen/` et `.next/`, `out/` obsolètes.
2. Initialiser Git + `.gitignore` (`target/`, `node_modules/`, `.next/`, `out/`, `*.db`, `.storage/`).
3. Créer un type d'erreur sérialisable (`thiserror` + `Serialize`) et remplacer les 43
   `anyhow::Result<T>` par `Result<T, AppError>`.
4. `rusqlite` : ajouter la feature `serde_json`.
5. Corriger `AppState` (retirer `Clone`, exposer un accès contrôlé), `lib.rs` (`pub fn run()` +
   `generate_context!()`), les `query_map` sur `Statement`, les `HashMap<String, i64>`, les
   emprunts de temporaires, les `.await` manquants.
6. Critère de sortie : `cargo check` sans erreur, `npm run tauri dev` ouvre une fenêtre.

### Lot 1 — Rendre la chaîne de custody réelle (le cœur du produit)

7. **Ingestion des preuves** : au moment de `add_evidence`, le backend lit le fichier, calcule
   SHA-256 (et MD5 si l'on veut la compatibilité outillage), le copie dans un magasin en écriture
   contrôlée (`.storage/evidence/<aa>/<uuid>`), enregistre taille et empreinte **observées**, et
   refuse un hash fourni par le client.
8. **Vérification réelle** : `verify_evidence` relit le fichier stocké, recalcule, et renvoie
   `intact | altered | missing | no_hash` — la sémantique déjà présente dans `lib/hashes.ts`.
9. **Journal d'audit véritable** : calcul en Rust de `imma = SHA256(canonical_json(événement) ||
   imma_precedent)`, `imma_precedent` lu depuis le dernier événement **du dossier**, insertion
   dans la même transaction que l'opération métier, et vérification qui **recalcule** toute la
   chaîne. Supprimer les déclencheurs SQL `audit_*` (P1-5) : la logique de hachage doit vivre dans
   un seul endroit, en Rust.
10. Écrire l'événement d'audit pour **toutes** les mutations (preuves, sujets, claims, rapports,
    snapshots), horodatage unique en UTC RFC 3339 avec précision milliseconde + numéro de séquence
    monotone par dossier (pour un ordre déterministe).
11. **Suppression logique uniquement** : `statut = 'supprime'`, jamais de `DELETE`. Retirer les
    `ON DELETE CASCADE` qui peuvent détruire `audit_events`. Toute suppression est un événement
    d'audit, avec acteur et motif.
12. **Snapshots réels** : sérialiser l'état complet du dossier en JSON canonique (le
    `canonicalJson` de `lib/canonical.ts` est un bon point de départ, à porter en Rust), hacher ce
    contenu, chaîner sur le snapshot précédent, alimenter `snapshot_contents`, et faire de
    `verify_snapshot_integrity` un vrai recalcul.
13. Ajouter les tests qui manquent, en priorité : concordance des hachages TS/Rust (le test que
    `lib/canonical.ts` promet déjà), détection d'une preuve altérée, détection d'un maillon d'audit
    supprimé ou modifié, refus de réattribution de référence.

### Lot 2 — Corriger les fondations techniques

14. Transactions autour de chaque commande d'écriture (P2-1).
15. Refondre le schéma FTS5 : `rowid` entier (colonne `INTEGER PRIMARY KEY` dédiée) ou tables FTS
    autonomes ; déclencheurs pour **les sept** entités ; supprimer les insertions FTS manuelles
    (P2-2, P2-9, P2-10).
16. Réécrire `search_fts_table` par entité (colonnes réelles, `bm25()` pour le score, filtre
    `case_id` fonctionnel) (P2-3, P2-4).
17. `#[serde(rename_all = "camelCase")]` sur toutes les structures exposées, et **une** source de
    vérité pour les types — de préférence générée depuis Rust (`ts-rs` ou `specta`) pour supprimer
    définitivement la dérive décrite en P2-7.
18. Références de dossier via une table de compteurs transactionnelle, jamais réattribuées (P2-11).
19. Corriger `set_claim` (clé `(caseId, refKind, refId)`) et historiser les changements de
    qualification/fiabilité (P2-12).
20. Base de données dans le répertoire de données applicatives via l'API Tauri (`app_data_dir`),
    plus de chemin relatif, plus de `expect()` au démarrage (P1-10).

### Lot 3 — Sécurité et confidentialité

21. Définir une CSP stricte dans `tauri.conf.json` ; assainir systématiquement tout contenu web
    collecté avant affichage ; filtrer les URL avant `shell:open` (schémas autorisés) (P1-11).
22. Chiffrement au repos (SQLCipher, ou chiffrement du magasin de preuves) et verrouillage de
    l'application (mot de passe / clé) : sans cela, la présence de données personnelles de suspects
    et de victimes en clair est difficilement défendable (P1-9).
23. Implémenter réellement `users` / `session_logs` (acteur authentifié) — sans acteur fiable,
    l'audit n'attribue rien : `actor` est aujourd'hui `"system"` ou une chaîne libre fournie par
    l'appelant.
24. Appliquer `retentionDays` ; purger `fts_*` en même temps que les données ; documenter la base
    légale de traitement et la durée de conservation.

### Lot 4 — Interface et documentation

25. Reconstruire `app/` et `components/`, en s'appuyant sur `lib/tauri-bridge.ts` et
    `lib/hooks/useTauriQuery.ts` (les deux seules pièces frontend saines et réutilisables).
26. **Supprimer** les ~20 modules mock de `lib/` dès que le backend répond (P3-10). Aucun repli
    silencieux : en cas d'échec, l'interface doit afficher l'erreur, jamais « intègre ».
27. Aligner la documentation sur la réalité : `PROJECT_SUMMARY.md`, `MEMORY.md` et `PROJET.md`
    décrivent aujourd'hui un produit terminé et testé. Le décalage est tel qu'il constitue un
    risque en soi — quiconque reprend le projet (ou décide de s'en servir sur un dossier réel) sera
    induit en erreur. Un `STATUS.md` honnête, daté, listant ce qui fonctionne et ce qui ne
    fonctionne pas, vaut mieux que 1 500 lignes de spécification présentée comme un état des lieux.

---

## 9. Conclusion

Le travail conceptuel est réel : le modèle de données (24 tables), la distinction
`preuve / indice / hypothèse / non vérifié` avec échelle de fiabilité, l'idée du chaînage d'audit et
des snapshots vérifiables, le choix d'une IA locale (Ollama, pas de fuite de données vers un tiers)
constituent une base de conception pertinente pour un outil d'enquête. `lib/canonical.ts`,
`lib/hashes.ts`, `lib/tauri-bridge.ts` et `lib/hooks/useTauriQuery.ts` sont des pièces correctes.

Mais l'exécution est à l'état d'ébauche, et l'écart entre ce que le logiciel affirme et ce qu'il
fait est le vrai danger. Un outil dont le rôle est d'attester l'intégrité de preuves ne peut pas
répondre « vérifié » sans avoir vérifié. En l'état :

> **Cekarna ne doit pas être utilisé sur un dossier réel, et aucune sortie de ce logiciel ne doit
> être présentée comme un élément dont l'intégrité est établie.**

Le chemin de remise en état est clair et hiérarchisé (§8). L'effort principal n'est pas de corriger
127 erreurs de compilation — c'est mécanique — mais d'implémenter réellement les quatre garanties
qui définissent le produit : hachage à l'ingestion, vérification par recalcul, journal d'audit
chaîné et recalculable, snapshots qui capturent l'état. Tant que ces quatre points ne sont pas
couverts par des tests automatisés, le produit n'a pas de valeur probatoire.

---

## 10. Conformité au cahier des charges `CLAUDE.md`

> Ajouté le 2026-07-28 à la demande de vérification des exigences.

### 10.1 Constat préalable : le `CLAUDE.md` ne décrit pas ce projet

`CLAUDE.md` (racine du dépôt) spécifie **SPECTRA** : une plateforme OSINT d'*identity resolution*
avec canvas graphe Sigma.js, moteur WhatsMyName natif, plugins WebAssembly Extism, monorepo Cargo de
10 crates, licence AGPL-3.0. Le code présent est **Cekarna** : gestion d'enquêtes Next.js + Tauri,
SQLite, licence GPL-3.0, crate unique.

Vérification : la chaîne « spectra » n'apparaît **nulle part** dans le code source du projet —
uniquement dans `CLAUDE.md` (et dans des artefacts de build de dépendances tierces). Aucun des dix
crates spécifiés (`spectra-core`, `spectra-store`, `spectra-transform`, `spectra-plugins`,
`spectra-collect`, `spectra-correlate`, `spectra-ai`, `spectra-report`, `spectra-audit`,
`spectra-cli`) n'existe.

Chronologie : les fichiers Cekarna datent du 2026-06-22 ; `CLAUDE.md` se déclare daté de juillet 2026
(§13.2). Le cahier des charges est donc **postérieur** au code, ce qui suggère un changement de
projet plutôt qu'une dérive d'implémentation.

**Conséquence : le taux de conformité global est de l'ordre de 5 à 10 %.** Les tableaux ci-dessous
détaillent l'écart, exigence par exigence.

### 10.2 Contraintes non négociables (§1)

| # | Exigence | État | Détail |
|---|---|---|---|
| C1 | Aucun appel réseau hors sources OSINT explicites, zéro télémétrie | ✅ | Seul appel sortant : `localhost:11434` (Ollama). Aucune télémétrie détectée. Conformité par absence de fonctionnalité. |
| C2 | Pleinement fonctionnel hors ligne | ⚠️ | Rien ne dépend du réseau, mais rien ne fonctionne (§3). |
| C3 | Aucune clé API pour le socle | ✅ | Aucune clé requise. |
| C4 | Données dans un fichier de dossier local, portable, **chiffrable** | ❌ | SQLite en clair, aucun chiffrement (P1-9), chemin relatif au répertoire courant (P1-10). Pas de format de dossier portable. |
| C5 | Licence **AGPL-3.0** cœur / Apache-2.0 SDK plugins | ❌ | `Cargo.toml` déclare GPL-3.0, aucun fichier `LICENSE`, aucun SDK de plugins. Aucune vérification de compatibilité des licences de dépendances. |
| C6 | **Pas de code mort, pas de fonctionnalité « mock »** | ❌❌ | Violation la plus flagrante : ~20 modules `lib/*.ts` explicitement « Mock … for development » (P3-10), code mort (`APP_CONFIG`, `AppState::execute`, `get_mutex`), répertoires UI vides. |

### 10.3 Périmètre fonctionnel (§2)

**Pilier A — Recherche de personne : 0 % implémenté.** Aucune des six capacités n'existe :
énumération de pseudos (WhatsMyName), vérification d'email, parsing téléphone, extraction de
métadonnées de profil, corrélation d'identité (pHash/dHash), fuites de données.

**Pilier B — Système d'enquêtes : ~15 % au niveau conceptuel, 0 % opérationnel.**

| Exigence | État | Détail |
|---|---|---|
| Dossiers `.spectra` autonomes, chiffrables | ❌ | Base SQLite globale unique, pas un fichier par enquête. |
| **Canvas graphe (« le cœur visuel »)** | ❌ | Absent. Ni Sigma.js, ni graphology, ni aucune visualisation. |
| Timeline synchronisée | ⚠️ | Tables `report_timeline` / `case_events` en base, aucune vue. |
| Carte | ❌ | Absent (pas de coordonnées dans le schéma). |
| Vue table | ❌ | Absent. |
| Notes & hypothèses ACH | ⚠️ | Table `notes`, pas de Markdown, pas d'ACH. |
| Chaîne de possession (source, méthode, horodatage, hash, opérateur, **Admiralty A1–F6**) | ❌ | Champs partiels (`source`, `sourceUrl`, `takenBy`), échelle `fiabilite 0–5` au lieu d'Admiralty, hash non calculé (P1-2), opérateur non authentifié. |
| Rapports PDF/DOCX/HTML/MD + annexes hashées | ❌ | Tables de rapports uniquement, aucun générateur. |
| Journal d'audit append-only hash-chaîné inaltérable | ❌ | Implémenté sur le papier, faux en pratique (P1-3, P1-4, P1-5). |
| Collaboration hors-ligne, fusion CRDT | ❌ | Absent. |

### 10.4 Stack technique arrêtée (§3)

| Couche | Exigé | Présent | État |
|---|---|---|---|
| Shell | Tauri 2 (`2.11.x`) | Tauri `2.0` | ⚠️ version non vérifiée/épinglée comme demandé |
| Frontend | React 19 | React 19.2.4 (aucun composant) | ⚠️ |
| Build | **Vite** | **Next.js 16** (`output: 'export'`) | ❌ divergence structurante |
| Styles | Tailwind CSS 4 | dépendance présente, non utilisée | ⚠️ |
| Composants | shadcn/ui (Radix) | absent | ❌ |
| État serveur | TanStack Query | `useTauriQuery` maison | ❌ |
| État client | Zustand | absent | ❌ |
| Routing | TanStack Router / React Router 7 | routeur Next.js | ❌ |
| Tests | Vitest + Testing Library + Playwright | Vitest configuré, **0 test** | ❌ |
| **Rendu graphe** | **Sigma v3 + graphology + FA2 en Web Worker, 50 k nœuds ≥ 45 fps** | **absent** | ❌ exigence bloquante non abordée |
| Backend | `reqwest` | déclaré `optional`, jamais utilisé | ❌ |
| | `scraper`, `chromiumoxide` | absents | ❌ |
| | `rusqlite` | présent (feature `serde_json` manquante, P0-1b) | ⚠️ |
| | `tantivy` | absent — FTS5 utilisé à la place | ❌ |
| | `petgraph`, `governor` | absents | ❌ |
| | `thiserror` | **déclaré dans `Cargo.toml`, zéro utilisation** | ❌ |
| | `tracing` | absent (aucun log structuré) | ❌ |
| | `rustls` | absent | ❌ |
| | `argon2` + `chacha20poly1305` | absents → pas de chiffrement de dossier | ❌ |
| | `image` + `img_hash` | absents | ❌ |
| Stockage | `sqlite-vec` (index vectoriel) | absent | ❌ |
| Plugins | **Extism / wasmtime**, manifeste TOML, allowlist, signature Ed25519 | absent | ❌ |
| IA locale | `ollama-rs` en Rust, trait `LlmBackend`, `mistral.rs`, `fastembed-rs` | appel `fetch()` depuis TypeScript, aucun trait, aucun embedding | ❌ |
| Garde-fous IA | sortie marquée `INFERRED`, exclue des rapports | absent (aucune IA fonctionnelle) | ❌ |

### 10.5 Architecture du dépôt (§4)

| Exigence | État |
|---|---|
| Monorepo workspace Cargo + workspace pnpm | ❌ crate unique `cekarna`, npm sans workspace |
| 10 crates `spectra-*` | ❌ aucun |
| `apps/desktop/src/features/{graph,timeline,map,table,entity,transforms,report,case}` | ❌ `app/` et `components/` vides |
| `plugins/`, `ontology/` | ❌ absents |
| `docs/adr/`, `docs/ontology.md`, `docs/plugin-sdk.md`, `docs/legal/` | ❌ `docs/` vide |
| `.github/workflows/` | ❌ aucune CI |
| Règle : `spectra-core` sans I/O | ❌ sans objet |
| **CLI de premier ordre** (`spectra-cli`) | ❌ absente |

### 10.6 Modèle de données (§5) — l'écart le plus profond

`CLAUDE.md` §5 pose que « le graphe affiché est une **projection** des observations », avec un
curseur temporel rejouant l'état du dossier à toute date, et présente ce point comme « ce qui sépare
un jouet d'un outil d'enquête ».

| Exigence | État |
|---|---|
| `Entity` (UUIDv7, `kind` d'ontologie, `canonical_value`, `merged_from`) | ❌ tables plates par type métier ; UUIDv4 |
| **`Observation`** (`predicate`, `source`, `method`, `observed_at`, `valid_from`/`valid_to`, `confidence` Admiralty, `provenance` COLLECTED/ASSERTED/INFERRED, `raw_hash`, `operator`) | ❌ **inexistant** — les propriétés sont des faits nus dans les colonnes |
| Projection temporelle / curseur de date | ❌ inexistant |
| Ontologie extensible en TOML (18 types minimum) | ❌ types codés en dur dans le schéma SQL |
| Résolution d'entités, fusion proposée jamais automatique, réversible | ❌ inexistant |

Ce point n'est pas un manque de fonctionnalité : c'est une **incompatibilité de modèle**. Passer du
schéma Cekarna au modèle Entity/Observation implique une réécriture du stockage, pas une migration.

### 10.7 UX (§6)

Aucune interface n'existe (P0-3) : canvas graphe, palette `Ctrl+K`, expansion progressive par
clic droit, panneau de collecte en direct avec kill switch, distinction visuelle de provenance
(plein / pointillé / gris), mode sombre, i18n FR+EN — **8 exigences sur 8 non satisfaites**.

### 10.8 Sécurité, OPSEC et conformité RGPD (§7)

| Exigence | État | Renvoi |
|---|---|---|
| Profils réseau direct / proxy / SOCKS5 / **Tor** | ❌ | — |
| Rotation User-Agent, cookie jar isolé par dossier | ❌ | — |
| Détection de fuite (refus de collecte si circuit Tor non établi) | ❌ | — |
| **Chiffrement au repos** XChaCha20-Poly1305 + Argon2id, verrouillage auto | ❌ | P1-9 |
| Purge sécurisée d'un dossier et de ses caches | ❌ | données résiduelles en FTS, P1-9 |
| **CSP Tauri stricte** | ❌ | P1-11 — aucune CSP définie |
| Permissions Tauri minimales, capabilities explicites | ⚠️ | capabilities présentes mais plugins non initialisés (P3-6) |
| Aucune commande n'accepte de chemin arbitraire sans validation | ❌ | `add_evidence` accepte `chemin` sans validation (P1-2) |
| `cargo deny` + `cargo audit` + `npm audit` en CI | ❌ | aucune CI |
| Aucun `unsafe` non justifié | ✅ | aucun `unsafe` |
| Fuzzing des parseurs | ❌ | — |
| **RGPD — base légale + finalité obligatoires à la création du dossier** | ❌ | champs absents du schéma |
| Durée de conservation paramétrable + alerte + purge | ❌ | `audit_settings.retentionDays` jamais lu |
| Avertissement UI sur les données de l'art. 9 | ❌ | — |
| Journal d'audit exportable (demande d'accès / contrôle CNIL) | ❌ | aucun export |
| Écran de première utilisation rappelant le cadre légal | ❌ | — |
| `docs/legal/RESPONSIBLE_USE.md` | ❌ | absent |
| Respect de `robots.txt` par défaut | ❌ | aucun collecteur |

Point d'attention supplémentaire : `lib/cross-links.ts` (mock) prévoit des liens **entre dossiers**,
alors que `CLAUDE.md` §9 interdit explicitement d'agréger quoi que ce soit entre enquêtes
(« SPECTRA n'agrège rien entre dossiers ; chaque enquête est cloisonnée »).

### 10.9 Contrat qualité (§8)

| Exigence | État |
|---|---|
| Couverture ≥ 90 % sur `core` et `correlate` | ❌ 0 test |
| Aucun test ne touche le réseau (fixtures `wiremock`) | — sans objet |
| `thiserror` en bibliothèque / `anyhow` en binaire | ❌ **inversé** : `anyhow` dans la bibliothèque (cause de 43 erreurs de compilation, P0-1a), `thiserror` inutilisé |
| **Jamais de `unwrap()` / `expect()`** hors tests | ❌ 5 occurrences : `lib.rs:34`, `lib.rs:99`, `database.rs:26`, `database.rs:801`, `reports.rs:91` |
| `#![deny(warnings)]`, `clippy::pedantic` | ❌ absents (11 warnings au dernier check) |
| TS `strict` ✓, `noUncheckedIndexedAccess`, **zéro `any`** | ⚠️ `strict: true` ✓ ; `noUncheckedIndexedAccess` absent ; 3 `any` (`lib/cases.ts:70`, `lib/subjects.ts:67`, `lib/tauri-bridge.ts:66`) |
| Types partagés générés depuis Rust (`ts-rs` / `specta`) | ❌ trois définitions divergentes maintenues à la main (P2-7) |
| Benchmarks `criterion`, régression > 10 % = build cassé | ❌ absents |
| Conventional Commits, un commit = une unité compilable | ❌ **aucun dépôt Git** (P3-2) |
| Un ADR par décision structurante | ❌ `docs/adr/` inexistant |
| `//!` de module par crate | ✅ présent sur les modules Rust |

### 10.10 Interdits (§9) et protocole de travail (§12–§14)

| Interdit | Respecté |
|---|---|
| Envelopper des outils Python | ✅ (aucune collecte) |
| Embarquer des dumps illégaux | ✅ |
| Construire une base de personnes inter-dossiers | ⚠️ `cross-links.ts` va à l'encontre du principe |
| Dépendances non maintenues | ⚠️ non vérifié ; `pg` inutile (P3-5) |
| **Empiler des fonctionnalités avant que le canvas et le modèle d'observation soient irréprochables** | ❌ 24 tables, 43 commandes, 12 routes documentées — sans canvas ni modèle d'observation |
| Ajouter du LLM là où une heuristique suffit | ✅ |

Concernant le plan par phases (§12) : la **Phase 0** (vérification des versions, scaffold du
workspace, CI, ADR stockage/graphe/plugins/IA) n'a jamais été exécutée, et son critère d'acceptation
(`cargo build --workspace` vert) est aujourd'hui en échec. La **Phase 2** (canvas graphe, désignée
comme « le plus risqué, à faire tôt ») n'a pas été abordée. Le code existant correspond à une
tentative partielle de Phase 1 et de Phase 5, hors séquence.

Le premier message attendu au §14 (versions vérifiées, état de LadybugDB, 5 questions, 3 risques,
plan de Phase 0) n'a pas de trace dans le dépôt.

### 10.11 Synthèse de conformité

| Chapitre `CLAUDE.md` | Conformité |
|---|---|
| §1 Contraintes non négociables | 2 / 6 |
| §2 Pilier A (recherche de personne) | 0 / 6 |
| §2 Pilier B (système d'enquêtes) | ~1 / 10 |
| §3 Stack technique | ~3 / 25 |
| §4 Architecture du dépôt | 0 / 8 |
| §5 Modèle de données | 0 / 5 |
| §6 UX | 0 / 8 |
| §7 Sécurité / OPSEC / RGPD | 1 / 18 |
| §8 Contrat qualité | 1 / 11 |
| §12 Plan par phases | Phase 0 non faite, séquence non respectée |

**Il n'existe pas de « patch » qui rende ce code conforme à `CLAUDE.md`.** Les deux documents
décrivent deux produits différents : le cahier des charges impose une architecture (workspace de
crates, modèle Entity/Observation, canvas graphe, plugins WASM, Vite) incompatible avec
l'implémentation existante. Une décision d'orientation est nécessaire avant tout travail de
correction — voir la question posée à l'issue de cet audit.
