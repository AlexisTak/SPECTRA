/**
 * Banc d'essai de rendu : FPS réels en pan/zoom avec Sigma v3.
 *
 * Chargé par `bench.html`. Le layout n'est volontairement **pas** recalculé
 * ici : on mesure le coût du rendu, pas celui du placement (déjà mesuré par
 * `bench-layout.ts`). Les positions viennent du générateur.
 *
 * Le résultat est exposé sur `window.__BENCH__` pour qu'un pilote externe
 * (Playwright) puisse le lire sans dépendre du DOM.
 */

import Sigma from 'sigma'
import forceAtlas2 from 'graphology-layout-forceatlas2'
import { generateScaleFreeGraph } from './generate-graph'

export interface RenderBenchResult {
  nodes: number
  edges: number
  generationMs: number
  layoutMs: number
  firstRenderMs: number
  fpsAvg: number
  fpsMin: number
  fpsP5: number
  heapMb: number
  passed: boolean
}

declare global {
  interface Window {
    __BENCH__?: RenderBenchResult
    __BENCH_ERROR__?: string
    runRenderBench?: (nodes: number, edges: number) => Promise<RenderBenchResult>
  }
}

const PALETTE = ['#38bdf8', '#a78bfa', '#f472b6', '#4ade80', '#fbbf24']

export async function runRenderBench(
  nodeCount = 50_000,
  edgeCount = 150_000,
): Promise<RenderBenchResult> {
  const container = document.getElementById('graph')
  if (!container) throw new Error('conteneur #graph introuvable')

  const t0 = performance.now()
  const graph = generateScaleFreeGraph(nodeCount, edgeCount)
  const generationMs = performance.now() - t0

  // Couleur et taille par degré : les hubs doivent rester lisibles au dézoom.
  graph.forEachNode((node) => {
    const degree = graph.degree(node)
    graph.setNodeAttribute(node, 'size', Math.min(12, 1 + Math.sqrt(degree)))
    graph.setNodeAttribute(
      node,
      'color',
      PALETTE[Math.min(PALETTE.length - 1, Math.floor(Math.log2(degree + 1)))],
    )
  })

  // Paramètres retenus après l'étude de sensibilité (`bench-tuning.ts`) :
  // theta 1.2 + 50 itérations tiennent le budget avec une marge confortable.
  const t1 = performance.now()
  forceAtlas2.assign(graph, {
    iterations: 50,
    settings: {
      barnesHutOptimize: true,
      barnesHutTheta: 1.2,
      gravity: 1,
      scalingRatio: 10,
      slowDown: 1,
      adjustSizes: false,
      linLogMode: false,
      strongGravityMode: false,
    },
  })
  const layoutMs = performance.now() - t1

  // LOD : sans ces seuils, Sigma tente de composer 50 000 étiquettes DOM à
  // chaque frame et le rendu s'effondre quelle que soit la carte graphique.
  const t2 = performance.now()
  const renderer = new Sigma(graph, container as HTMLElement, {
    renderLabels: true,
    labelRenderedSizeThreshold: 14,
    labelDensity: 0.07,
    labelGridCellSize: 60,
    zIndex: true,
    defaultNodeColor: '#38bdf8',
    defaultEdgeColor: 'rgba(148,163,184,0.15)',
  })
  await nextFrame()
  const firstRenderMs = performance.now() - t2

  const samples = await measureFpsDuringInteraction(renderer)

  const heapMb =
    'memory' in performance
      ? (performance as unknown as { memory: { usedJSHeapSize: number } }).memory
          .usedJSHeapSize /
        (1024 * 1024)
      : 0

  const sorted = [...samples].sort((a, b) => a - b)
  const fpsAvg = samples.reduce((s, v) => s + v, 0) / (samples.length || 1)
  const fpsMin = sorted[0] ?? 0
  // Le 5e centile plutôt que le strict minimum : une frame isolée à 12 fps
  // (GC, changement d'onglet) ne dit rien de la fluidité perçue.
  const fpsP5 = sorted[Math.floor(sorted.length * 0.05)] ?? 0

  const result: RenderBenchResult = {
    nodes: graph.order,
    edges: graph.size,
    generationMs: Math.round(generationMs),
    layoutMs: Math.round(layoutMs),
    firstRenderMs: Math.round(firstRenderMs),
    fpsAvg: Math.round(fpsAvg),
    fpsMin: Math.round(fpsMin),
    fpsP5: Math.round(fpsP5),
    heapMb: Math.round(heapMb),
    passed: fpsP5 >= 45 && heapMb < 1536,
  }

  window.__BENCH__ = result
  return result
}

/**
 * Anime un pan/zoom continu et échantillonne le temps entre frames.
 *
 * Mesurer les FPS sur un graphe immobile n'aurait aucun sens : Sigma ne
 * redessine que sur changement de caméra. Il faut donc solliciter la caméra
 * en continu pendant la mesure.
 */
async function measureFpsDuringInteraction(
  renderer: Sigma,
  durationMs = 6000,
): Promise<number[]> {
  const camera = renderer.getCamera()
  const samples: number[] = []
  const start = performance.now()
  let last = start

  return new Promise((resolve) => {
    function step(): void {
      const now = performance.now()
      const delta = now - last
      last = now

      // On ignore la toute première frame (coût d'amorçage non représentatif).
      if (delta > 0 && now - start > 100) {
        samples.push(1000 / delta)
      }

      const elapsed = now - start
      const phase = elapsed / durationMs

      // Trajectoire déterministe : zoom avant/arrière combiné à un panoramique
      // circulaire, pour solliciter à la fois le culling et le LOD.
      camera.setState({
        x: 0.5 + Math.cos(phase * Math.PI * 4) * 0.15,
        y: 0.5 + Math.sin(phase * Math.PI * 4) * 0.15,
        ratio: 1.2 + Math.sin(phase * Math.PI * 2) * 0.8,
        angle: 0,
      })

      if (elapsed < durationMs) {
        requestAnimationFrame(step)
      } else {
        resolve(samples)
      }
    }
    requestAnimationFrame(step)
  })
}

function nextFrame(): Promise<void> {
  return new Promise((resolve) => requestAnimationFrame(() => resolve()))
}

window.runRenderBench = runRenderBench
