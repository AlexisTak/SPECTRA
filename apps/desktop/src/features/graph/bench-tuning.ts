/**
 * Étude de sensibilité du layout FA2 sur 50k nœuds / 150k arêtes.
 *
 * Le banc initial dépasse le budget (39,9 s pour 100 itérations). Avant de
 * proposer une alternative d'architecture, on quantifie les deux leviers
 * disponibles :
 *
 * - `barnesHutTheta` : seuil d'approximation. Plus il est élevé, plus des
 *   groupes de nœuds éloignés sont traités comme une masse unique — donc plus
 *   c'est rapide, au prix de la fidélité du placement.
 * - `iterations` : nombre de passes. La qualité du layout sature bien avant
 *   la centième passe sur un graphe en hubs.
 *
 * Exécution : `npx tsx apps/desktop/src/features/graph/bench-tuning.ts`
 */

import forceAtlas2 from 'graphology-layout-forceatlas2'
import { generateScaleFreeGraph } from './generate-graph'

const NODES = 50_000
const EDGES = 150_000

console.log(`Sensibilité FA2 — ${NODES.toLocaleString('fr-FR')} nœuds / ${EDGES.toLocaleString('fr-FR')} arêtes\n`)
console.log('theta  iters   temps      verdict (budget 30 s)')
console.log('-----  -----   --------   ----------------------')

const combos: Array<[number, number]> = [
  [0.5, 100],
  [0.8, 100],
  [1.2, 100],
  [1.5, 100],
  [0.8, 50],
  [1.2, 50],
  [1.5, 50],
  [1.2, 30],
]

for (const [theta, iterations] of combos) {
  // Graphe régénéré à chaque essai : `assign` mute les positions, réutiliser
  // le même graphe ferait partir l'essai suivant d'un état déjà relaxé et
  // fausserait la mesure.
  const graph = generateScaleFreeGraph(NODES, EDGES)

  const t0 = performance.now()
  forceAtlas2.assign(graph, {
    iterations,
    settings: {
      barnesHutOptimize: true,
      barnesHutTheta: theta,
      gravity: 1,
      scalingRatio: 10,
      slowDown: 1,
      adjustSizes: false,
      linLogMode: false,
      strongGravityMode: false,
    },
  })
  const ms = performance.now() - t0

  const verdict = ms < 30_000 ? 'OK' : 'hors budget'
  console.log(
    `${theta.toFixed(1).padStart(5)}  ${String(iterations).padStart(5)}   ` +
      `${(ms / 1000).toFixed(1).padStart(6)} s   ${verdict}`,
  )
}
