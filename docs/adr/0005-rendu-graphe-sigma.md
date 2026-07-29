# ADR 0005 — Rendu du graphe : Sigma.js v3 + graphology

* **Statut** : accepté
* **Date** : 2026-07-28
* **Phase** : 2
* **Question** : Comment rendre 50k+ nœuds de manière fluide ?

## Contexte

CLAUDE.md §3 tranche : **Sigma.js v3 + graphology**. Cytoscape.js et vis-network "décrochent avant" 50k nœuds.

Contrainte de performance (§12 Phase 2) :
- **50 000 nœuds / 150 000 arêtes**
- **≥ 45 fps** en pan/zoom sur laptop 2022 sans GPU dédié
- Layout ForceAtlas2 en **Web Worker** (jamais thread principal)

## Décision

1. **graphology** comme structure de données unique (mémoire + algorithmes)
2. **Sigma v3** comme moteur de rendu WebGL
3. **graphology-layout-forceatlas2** exécuté en Web Worker
4. **Niveaux de détail (LOD)** : labels masqués au dézoom, clustering des nœuds serrés
5. **Culling** hors-viewport géré par Sigma nativement
6. **Vue "overview/detail"** : mini-carte pour navigation globale

## Justification

- **Sigma v3** : seul moteur qui tient 50k+ nœuds en WebGL pur
- **graphology** : API cohérente, algorithmes intégrés (centralité, communautés, plus court chemin)
- **Web Worker** : le layout FA2 est O(n²) — bloquerait le thread UI sans worker
- **LOD + culling** : réduisent le nombre de primitives WebGL à dessiner

## Conséquences

- Le benchmark **précède** l'UI (§12) : si < 45 fps, on s'arrête et on propose des options
- Options de repli si échec :
  1. **Pré-layout Rust** : calculer positions initiales côté backend (petgraph)
  2. **Clustering hiérarchique** : regrouper les nœuds serrés en "super-nœuds"
  3. **Échantillonnage** : n'afficher qu'un sous-ensemble + recherche à la demande

## Critères d'acceptation Phase 2

| Critère | Cible | Mesuré (2026-07-28) | Verdict |
|---|---|---|---|
| Nœuds | 50 000 | 50 000 | conforme |
| Arêtes | 150 000 | 150 000 | conforme |
| Layout FA2 | < 30 s | **5,8 s** | tenu |
| Mémoire | < 1,5 Go | 93 Mo | tenu |
| **FPS (pan/zoom)** | ≥ 45 | **non mesurable** dans cet environnement | **non statué** |
| Layout en Web Worker | obligatoire | fait (`layout-worker.ts`) | à re-valider |

> **Correction du 2026-07-28.** Une version antérieure de cet ADR annonçait
> « 53 fps (p5), tenu ». **Ce chiffre n'est pas fiable et le critère FPS n'est
> pas statué.** Le banc rejoué une heure plus tard, sans modification de code,
> a donné 36 puis 1 fps. Diagnostic : dans un Chromium piloté par Playwright,
> `requestAnimationFrame` est bridé à **1 Hz** — vérifié sur une page vide, sans
> Sigma ni graphe (3 frames en 2 s, delta médian 1 000 ms). Les mesures de FPS
> obtenues ainsi ne mesurent pas le rendu, mais le throttling du navigateur.

Détail, méthode et réserves : `docs/bench/README.md`.

## Réglages retenus, mesurés et non devinés

Le défaut `barnesHutTheta = 0.5` donne **39,9 s** de layout sur 50k — hors
budget. L'étude de sensibilité (`bench-tuning.ts`) montre que **theta 1,2 avec
50 itérations** ramène à **6,8 s**, soit un facteur 6, pour une perte de
fidélité de placement acceptable à cette échelle.

Les seuils de LOD sur les étiquettes (`labelRenderedSizeThreshold: 14`,
`labelDensity: 0.07`) sont tout aussi structurants : sans eux, Sigma compose
50 000 étiquettes par frame et le rendu s'effondre quelle que soit la machine.

Ces deux réglages ne sont pas cosmétiques — sans eux le critère n'est pas tenu.

## Ce que le worker a réellement apporté

Le layout est désormais déporté (`layout-worker.ts`, `runLayoutDetached`). Le
diagnostic mené au passage a une valeur qui dépasse la mesure de FPS, parce
qu'il porte sur des temps de blocage — non affectés par le throttling :

| Configuration | Frame la plus longue |
|---|---|
| superviseur FA2 avec Sigma attaché au graphe calculé | 1 017 ms |
| superviseur FA2 sur graphe détaché, sans observateur | **33 ms** |

Déporter le *calcul* ne suffisait pas : `assignLayoutChanges` réécrit les 50 000
attributs sur le thread principal à chaque message du worker, et chaque écriture
déclenche une réindexation de Sigma. D'où le graphe de travail détaché, avec
recopie des positions à cadence choisie.

Corollaire mesuré : recopier ces positions avec `setNodeAttribute` (100 000
événements) est **pire** que la version synchrone. Il faut
`updateEachNodeAttributes`, qui n'émet qu'un seul événement.

## Mise à jour 2026-07-29 — Compteur FPS embarqué et mode benchmark intégré

Les trois points restants ont été adressés :

1. **Compteur FPS embarqué** (`app/graph/use-fps-counter.ts`) : mesure via
   `requestAnimationFrame` dans la vraie fenêtre Tauri/WebView2, avec moyenne
   glissante sur 60 frames, min/max, et heap JS. Contrairement à Playwright,
   ce compteur reflète le taux de rendu perçu par l'analyste.

2. **Mode benchmark intégré** (`app/graph/graph-view.tsx`) : bouton
   "Benchmark 50k nœuds" qui charge le graphe synthétique (loi de puissance)
   et utilise le layout détaché (`runLayoutDetached`). Le FPS est mesuré en
   temps réel pendant le layout.

3. **Stress test avec interactions** : bouton "Stress test (6 s)" qui anime
   la caméra (pan circulaire + zoom in/out déterministe) pendant 6 secondes
   et capture les FPS min/max/moyenne. Cela sollicite le culling, le LOD,
   et le rendu WebGL sous charge.

### Reste à faire avant de clore la Phase 2

Les mécanismes sont en place ; il reste à **exécuter le stress test sur une
machine réelle avec WebView2 au premier plan** pour obtenir les chiffres
 définitifs et statuer le critère FPS ≥ 45. Cette mesure ne peut pas être
faite dans l'environnement de développement actuel (pas d'écran garanti).
