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
| FPS (pan/zoom) | ≥ 45 | **53** (p5), 65 (moyen) | tenu |
| Mémoire | < 1,5 Go | 93 Mo | tenu |
| Layout FA2 | < 30 s | **5,8 s** | tenu |
| Layout en Web Worker | obligatoire | **non fait** | **à faire** |

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

## Reste à faire avant de clore la Phase 2

1. **Déporter FA2 dans un Web Worker** (exigence explicite du §3). Le layout gèle
   actuellement l'interface 5,8 s.
2. **Rejouer la mesure dans la coquille Tauri** (WebView2), pas seulement dans
   Chromium sous Playwright.
3. Mesurer à nouveau **avec les interactions** (sélection, survol, expansion),
   qui ajoutent un coût par frame non couvert ici.
