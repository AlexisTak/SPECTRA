# Banc d'essai du canvas graphe — Phase 2

**Date :** 2026-07-28
**Machine :** Windows 11, Node 24.16, RTX 4060, 12 cœurs
**Environnement de mesure :** Chromium piloté par Playwright
**Critères CLAUDE.md §12 :** 50 000 nœuds / 150 000 arêtes, ≥ 45 fps en pan/zoom, < 1,5 Go

## Verdict : critères partiellement statués

| Critère | Cible | Mesuré | Verdict |
|---|---|---|---|
| Nœuds / arêtes | 50 000 / 150 000 | 50 000 / 150 000 | conforme |
| Layout ForceAtlas2 | < 30 s | **5,8 s** | **tenu** |
| Mémoire (tas JS) | < 1,5 Go | 93 Mo | **tenu** |
| **FPS pan/zoom** | ≥ 45 | — | **non mesurable ici** |

![Rendu 50k nœuds](bench-50k.png)

## Avertissement : les FPS ne sont pas mesurables dans cet environnement

Une première rédaction de ce document annonçait « 53 fps (p5), critères tenus ».
**C'était faux**, et voici pourquoi je le signale plutôt que de le corriger en
silence : le même banc, sans aucune modification de code, a donné successivement
65, puis 36, puis 1 fps.

Diagnostic, obtenu en exécutant une boucle `requestAnimationFrame` **sur une
page vide, sans Sigma, sans graphe, sans worker** :

```
3 frames en 2 000 ms — delta médian 1 000 ms — soit 1 fps
```

Chromium sous Playwright bride `requestAnimationFrame` à **1 Hz** quand la
fenêtre n'est pas réellement composée à l'écran, alors même que
`document.visibilityState` vaut `"visible"` et que `document.hasFocus()` est
vrai. Les deux premières exécutions à 65 fps ont vraisemblablement bénéficié
d'une fenêtre effectivement affichée au premier plan ; les suivantes non.

**Conséquence :** toute mesure de FPS produite ici mesure le throttling du
navigateur, pas la performance de rendu. Le critère « ≥ 45 fps » de la Phase 2
**reste à statuer** dans un environnement non bridé — coquille Tauri au premier
plan, ou compteur intégré à l'application.

Ce qui reste valide de ces mesures : tout ce qui ne dépend pas de la cadence
d'affichage — durée du layout, empreinte mémoire, et **durée des blocages du
thread principal**.

## Résultats fiables

### Layout ForceAtlas2 : 5,8 s (budget 30 s)

Mesuré en headless (`bench-layout.ts`), sans navigateur, donc sans throttling.

| Nœuds / arêtes | Génération | Layout FA2 (100 it.) |
|---|---|---|
| 1 000 / 3 000 | 9 ms | 0,38 s |
| 10 000 / 30 000 | 108 ms | 5,50 s |
| 50 000 / 150 000 | 572 ms | 39,88 s *(défaut theta 0,5 — hors budget)* |
| 50 000 / 150 000 | 572 ms | **6,8 s** *(theta 1,2 / 50 it. — retenu)* |

### Le réglage qui fait la différence

Le défaut `barnesHutTheta = 0.5` est **hors budget**. Étude de sensibilité
(`bench-tuning.ts`, 50k/150k) :

| theta | itérations | temps |
|---|---|---|
| 0,5 | 100 | 41,0 s |
| 0,8 | 100 | 28,6 s |
| 1,2 | 100 | 16,8 s |
| 1,5 | 100 | 11,2 s |
| 0,8 | 50 | 17,8 s |
| **1,2** | **50** | **6,8 s** ← retenu |
| 1,5 | 50 | 5,2 s |

`theta` est le seuil d'approximation de Barnes-Hut : au-delà, un amas éloigné
est traité comme une masse unique. Facteur 6 sur le temps, pour une perte de
fidélité de placement acceptable à cette échelle.

### Blocage du thread principal : 1 017 ms → 33 ms

Mesure de **durées de blocage**, non affectée par le throttling
(`bench-diagnose.ts`, 50k nœuds, layout en worker dans les deux cas) :

| Configuration | Frame la plus longue |
|---|---|
| Sigma attaché au graphe que le worker met à jour | **1 017 ms** |
| Graphe de travail détaché, sans observateur | **33 ms** |

C'est le résultat le plus utile de la séance. **Déporter FA2 dans un worker ne
suffit pas** : `assignLayoutChanges` réécrit les 50 000 attributs sur le thread
principal à chaque message du worker, et chaque écriture déclenche une
réindexation de Sigma. Le calcul est parallélisé, l'application des résultats
ne l'est pas.

La parade retenue (`runLayoutDetached`) : le superviseur travaille sur un graphe
jumeau que personne n'observe, et les positions sont recopiées vers le graphe
affiché à cadence choisie.

**Piège corollaire, mesuré :** recopier ces positions avec `setNodeAttribute`
(50 000 nœuds × 2 coordonnées = 100 000 événements graphology) est **pire** que
la version synchrone. Il faut `updateEachNodeAttributes`, qui fait un seul
passage et n'émet qu'un événement.

## Méthode : ce qui a été fait correctement

**Le graphe n'est pas aléatoire uniforme.** CLAUDE.md §3 impose une loi de
puissance. Le générateur (`generate-graph.ts`) utilise l'attachement
préférentiel de Barabási–Albert. Topologie obtenue sur 50k : degré max **681**,
**10 705 nœuds** concentrent 50 % des arêtes. Un graphe uniforme, sans hub,
aurait donné un benchmark nettement plus flatteur et sans valeur.

**Le générateur est déterministe** (PRNG mulberry32, graine 42) : la topologie
est reproductible à l'identique d'une exécution à l'autre. C'est précisément ce
qui a permis d'écarter le générateur comme cause de la variabilité observée.

**Complexité maîtrisée.** Une sélection pondérée naïve serait en O(N·E), soit
~7,5 milliards d'opérations sur 50k/150k. La « liste de répétition » ramène le
tirage proportionnel au degré en O(1), et la génération complète en O(E) —
572 ms mesurées.

## Reproduire

```bash
# Layout seul — headless, fiable, sans navigateur
npx tsx apps/desktop/src/features/graph/bench-layout.ts

# Sensibilité des paramètres FA2 — headless, fiable
npx tsx apps/desktop/src/features/graph/bench-tuning.ts

# Pages navigateur (durées de blocage exploitables, FPS NON exploitables ici)
cd apps/desktop/src/features/graph && npx vite --port 5199
#   /bench.html              rendu après layout synchrone
#   /bench-worker.html       réactivité pendant layout en worker
#   /bench-diagnose.html     isole le coût de la réindexation Sigma
#   /bench-render-only.html  contrôle : rendu seul, sans layout
#   /bench-repeat.html       répétabilité (révèle le throttling)
```

Avant toute mesure de FPS, vérifier que l'environnement n'est pas bridé :

```js
// Sur une page vide. Si le delta médian vaut ~1000 ms, les FPS sont inexploitables.
let last = performance.now(), deltas = []
;(function f(){ const n = performance.now(); deltas.push(n - last); last = n
  if (deltas.length < 60) requestAnimationFrame(f); else console.log(deltas) })()
```
