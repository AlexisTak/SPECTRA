# ADR 0002 — Moteur de stockage et moteur de graphe

* **Statut** : accepté
* **Date** : 2026-07-28
* **Phase** : 0
* **Question posée par `CLAUDE.md` §14.2** : SQLite + petgraph seul, ou avec LadybugDB en option ?

## Contexte

`CLAUDE.md` §3 tranche déjà que SQLite est la source de vérité unique, et signale que Kùzu — la base
graphe embarquée Cypher — a été archivée en octobre 2025 après le rachat de Kùzu Inc. par Apple, au
profit du fork communautaire LadybugDB. Le cahier des charges demande une recommandation ferme.

## État réel de LadybugDB au 2026-07-28 (vérifié)

| Indicateur | Valeur |
|---|---|
| Dépôt principal `LadybugDB/ladybug` | C++, **MIT**, créé le **2025-10-07** (le jour de l'archivage de Kùzu) |
| Étoiles / watchers | **1 503 ★** mais seulement **9 watchers** |
| Issues ouvertes | 77 |
| Dernier push | **2026-07-28** (le jour du relevé) |
| Binding Rust — dépôt `ladybug-rust` | actif, dernier push 2026-07-26, mais **4 ★**, 1 issue |
| Binding Rust — crate publié | **`lbug` 0.18.3**, publié le 2026-07-21, **~140 000 téléchargements récents** |
| Écosystème | bindings Python / Node / .NET / Swift, extension Postgres, packaging deb/rpm, visualiseur `bugscope` |

Lecture : le projet est **vivant et soutenu** (activité quotidienne, releases régulières, écosystème
qui s'étoffe), mais le décalage entre 1 503 étoiles et 9 watchers indique que la notoriété est
**héritée du fork Kùzu**, pas encore acquise. Le binding Rust, en particulier, est très peu observé
(4 ★) alors même que le crate est téléchargé — signe d'une adoption automatisée plus que d'une
communauté de mainteneurs. Numérotation encore en `0.x`.

## Décision

**SQLite + `petgraph` seul pour le socle.** LadybugDB n'entre pas dans le chemin critique.

1. La source de vérité reste un fichier SQLite unique par dossier (`.spectra`), conformément à §3.
2. Les traversées et algorithmes de graphe se font en Rust via `petgraph` sur une projection en
   mémoire, ou en SQL récursif (CTE) au-delà d'un seuil à déterminer par benchmark.
3. Un trait `GraphAnalyticsEngine` est défini **dès la conception** de `spectra-core`, avec une
   implémentation `petgraph` par défaut. Aucune implémentation LadybugDB n'est écrite avant d'avoir
   un besoin analytique mesuré que `petgraph` ne couvre pas.
4. Si ce besoin apparaît (analytique Cypher lourde), l'implémentation passera par le crate `lbug`,
   sous licence MIT — compatible avec un cœur AGPL-3.0.

## Justification

* **Le risque d'un fork jeune est asymétrique.** Le socle d'un outil probatoire ne peut pas dépendre
  d'un projet dont la gouvernance a moins d'un an, quelle que soit son activité actuelle. Kùzu était
  lui aussi actif jusqu'au jour de son archivage : c'est précisément le scénario dont `CLAUDE.md`
  demande de se prémunir.
* **Le besoin n'est pas démontré.** Les cas d'usage OSINT du produit (voisinage à N sauts, plus
  court chemin, centralité, communautés) sont couverts par `petgraph` et, côté client, par
  `graphology`. Cypher n'apporte rien tant qu'on n'écrit pas de requêtes analytiques ad hoc.
* **Une dépendance native C++ a un coût de portabilité** (compilation croisée Windows/macOS/Linux,
  taille du binaire, contrainte C2 « binaire unique »).
* `petgraph` n'a pas publié depuis septembre 2025, mais c'est une bibliothèque mature de 443 M de
  téléchargements dont l'API est stable ; l'absence de release n'est pas un signal d'abandon. À
  réévaluer si aucune activité n'est constatée d'ici la Phase 3.

## Conséquences

* `spectra-core` doit exposer `GraphAnalyticsEngine` comme abstraction dès la Phase 1, sinon
  l'option LadybugDB devient impossible à ajouter proprement plus tard.
* Un benchmark est nécessaire en Phase 2 pour déterminer le seuil de bascule projection mémoire ↔
  SQL récursif.
* Cette décision sera réexaminée si, et seulement si : (a) un besoin analytique concret n'est pas
  couvert par `petgraph`, ou (b) LadybugDB atteint une version 1.0 avec plusieurs mainteneurs
  indépendants identifiés.
