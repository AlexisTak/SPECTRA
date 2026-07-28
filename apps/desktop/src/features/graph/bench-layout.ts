/**
 * Banc d'essai headless : génération du graphe + layout ForceAtlas2.
 *
 * Mesure ce qui est mesurable **sans navigateur** :
 * - temps de génération d'un graphe sans échelle,
 * - temps du layout FA2 (le poste de coût dominant, O(n log n) avec Barnes-Hut),
 * - mémoire du processus.
 *
 * Les FPS de pan/zoom exigent un vrai contexte WebGL : ils sont mesurés
 * séparément dans la page `bench.html` (voir `bench-render.ts`). Ce fichier ne
 * prétend donc **pas** valider à lui seul le critère « ≥ 45 fps » de la Phase 2.
 *
 * Exécution : `npx tsx apps/desktop/src/features/graph/bench-layout.ts`
 */

import forceAtlas2 from 'graphology-layout-forceatlas2'
import { describeGraph, generateScaleFreeGraph } from './generate-graph'

interface LayoutBenchResult {
  nodes: number
  edges: number
  generationMs: number
  layoutMs: number
  iterations: number
  heapMb: number
  maxDegree: number
  hubCount: number
}

function runLayoutBench(
  nodeCount: number,
  edgeCount: number,
  iterations: number,
): LayoutBenchResult {
  const t0 = performance.now()
  const graph = generateScaleFreeGraph(nodeCount, edgeCount)
  const generationMs = performance.now() - t0

  const stats = describeGraph(graph)

  // Barnes-Hut est indispensable au-delà de ~10k nœuds : sans lui, FA2 est en
  // O(n²) par itération et ne termine pas en temps raisonnable sur 50k.
  const t1 = performance.now()
  forceAtlas2.assign(graph, {
    iterations,
    settings: {
      barnesHutOptimize: true,
      barnesHutTheta: 0.5,
      gravity: 1,
      scalingRatio: 10,
      slowDown: 1,
      adjustSizes: false,
      linLogMode: false,
      strongGravityMode: false,
    },
  })
  const layoutMs = performance.now() - t1

  const heapMb = process.memoryUsage().heapUsed / (1024 * 1024)

  return {
    nodes: stats.nodes,
    edges: stats.edges,
    generationMs,
    layoutMs,
    iterations,
    heapMb,
    maxDegree: stats.maxDegree,
    hubCount: stats.hubCount,
  }
}

function fmt(ms: number): string {
  return ms >= 1000 ? `${(ms / 1000).toFixed(2)} s` : `${Math.round(ms)} ms`
}

function main(): void {
  // Paliers croissants : si le budget explose avant 50k, on veut savoir où.
  const scenarios: Array<[number, number, number]> = [
    [1_000, 3_000, 100],
    [10_000, 30_000, 100],
    [50_000, 150_000, 100],
  ]

  console.log('Banc d\'essai layout — graphe sans échelle (Barabási–Albert)')
  console.log('Cible Phase 2 : 50 000 nœuds / 150 000 arêtes, layout < 30 s\n')

  for (const [nodes, edges, iterations] of scenarios) {
    const r = runLayoutBench(nodes, edges, iterations)
    const verdict = r.layoutMs < 30_000 ? 'OK' : 'HORS BUDGET'
    console.log(
      `${r.nodes.toLocaleString('fr-FR')} nœuds / ${r.edges.toLocaleString('fr-FR')} arêtes`,
    )
    console.log(`  génération   ${fmt(r.generationMs)}`)
    console.log(`  layout FA2   ${fmt(r.layoutMs)}  (${r.iterations} itérations) — ${verdict}`)
    console.log(`  tas Node     ${r.heapMb.toFixed(0)} Mo`)
    console.log(`  topologie    degré max ${r.maxDegree}, ${r.hubCount} nœuds portent 50 % des arêtes`)
    console.log()
  }
}

main()
