/**
 * Banc d'essai du layout en Web Worker.
 *
 * Ce que le banc précédent ne prouvait pas : que le thread principal reste
 * utilisable **pendant** le calcul du layout. Avec `assign()` en synchrone,
 * l'interface gelait 5,8 s — le §3 de CLAUDE.md l'interdit explicitement.
 *
 * La mesure décisive est donc ici le **FPS pendant le layout**, pas après.
 * Un layout en worker qui laisserait quand même l'UI à 5 fps n'aurait rien
 * résolu.
 *
 * Chargé par `bench-worker.html`.
 */

import Sigma from 'sigma'
import { generateScaleFreeGraph } from './generate-graph'
import { runLayoutDetached } from './layout-worker'

export interface WorkerBenchResult {
  nodes: number
  edges: number
  generationMs: number
  layoutMs: number
  layoutReason: string
  /** FPS du thread principal PENDANT le calcul du layout. C'est le critère. */
  fpsDuringLayoutAvg: number
  fpsDuringLayoutP5: number
  /** Plus longue frame observée pendant le layout : mesure le gel résiduel. */
  longestFrameMs: number
  heapMb: number
  passed: boolean
}

declare global {
  interface Window {
    __WORKER_BENCH__?: WorkerBenchResult
    __WORKER_BENCH_ERROR__?: string
  }
}

export async function runWorkerBench(
  nodeCount = 50_000,
  edgeCount = 150_000,
): Promise<WorkerBenchResult> {
  const container = document.getElementById('graph')
  if (!container) throw new Error('conteneur #graph introuvable')

  const t0 = performance.now()
  const graph = generateScaleFreeGraph(nodeCount, edgeCount)
  const generationMs = performance.now() - t0

  graph.forEachNode((node) => {
    const degree = graph.degree(node)
    graph.setNodeAttribute(node, 'size', Math.min(12, 1 + Math.sqrt(degree)))
    graph.setNodeAttribute(node, 'color', '#38bdf8')
  })

  // Sigma est monté AVANT le layout : il observe le graphe et affiche la
  // relaxation en direct. C'est tout l'intérêt du mode continu.
  const renderer = new Sigma(graph, container as HTMLElement, {
    renderLabels: true,
    labelRenderedSizeThreshold: 14,
    labelDensity: 0.07,
    labelGridCellSize: 60,
    defaultNodeColor: '#38bdf8',
    defaultEdgeColor: 'rgba(148,163,184,0.15)',
  })

  // Sonde de réactivité : elle tourne sur le thread principal en parallèle du
  // layout. Si le worker ne tenait pas sa promesse, les intervalles entre
  // frames exploseraient ici.
  const frameDeltas: number[] = []
  let probing = true
  let longestFrameMs = 0
  let lastFrame = performance.now()

  const camera = renderer.getCamera()
  const probeStart = performance.now()

  function probe(): void {
    const now = performance.now()
    const delta = now - lastFrame
    lastFrame = now

    if (delta > 0 && now - probeStart > 100) {
      frameDeltas.push(delta)
      if (delta > longestFrameMs) longestFrameMs = delta
    }

    // On sollicite la caméra en continu : mesurer les FPS sans mouvement ne
    // dirait rien, Sigma ne redessinant que sur changement.
    const phase = (now - probeStart) / 4000
    camera.setState({
      x: 0.5 + Math.cos(phase * Math.PI * 2) * 0.1,
      y: 0.5 + Math.sin(phase * Math.PI * 2) * 0.1,
      ratio: 1.4 + Math.sin(phase * Math.PI) * 0.5,
      angle: 0,
    })

    if (probing) requestAnimationFrame(probe)
  }
  requestAnimationFrame(probe)

  // `live=1` : synchronisation en direct (relaxation visible pendant le calcul).
  // Sinon : une seule synchronisation à la fin. Voir docs/bench/README.md.
  const live = new URLSearchParams(location.search).get('live') === '1'

  const handle = runLayoutDetached(graph, {
    maxDurationMs: 30_000,
    stabilityThreshold: 0.35,
    // Un intervalle supérieur au budget revient à ne synchroniser qu'à la fin.
    syncIntervalMs: live ? 500 : 60_000,
  })
  const progress = await handle.finished
  probing = false

  // Laisse la dernière frame se terminer avant de lire les compteurs.
  await new Promise((r) => requestAnimationFrame(() => r(null)))

  const fps = frameDeltas.map((d) => 1000 / d).sort((a, b) => a - b)
  const fpsAvg = fps.reduce((s, v) => s + v, 0) / (fps.length || 1)
  const fpsP5 = fps[Math.floor(fps.length * 0.05)] ?? 0

  const heapMb =
    'memory' in performance
      ? (performance as unknown as { memory: { usedJSHeapSize: number } })
          .memory.usedJSHeapSize /
        (1024 * 1024)
      : 0

  const result: WorkerBenchResult = {
    nodes: graph.order,
    edges: graph.size,
    generationMs: Math.round(generationMs),
    layoutMs: Math.round(progress.elapsedMs),
    layoutReason: progress.reason ?? 'inconnu',
    fpsDuringLayoutAvg: Math.round(fpsAvg),
    fpsDuringLayoutP5: Math.round(fpsP5),
    longestFrameMs: Math.round(longestFrameMs),
    heapMb: Math.round(heapMb),
    // Le critère est la réactivité PENDANT le layout, et l'absence de gel long.
    passed: fpsP5 >= 45 && longestFrameMs < 500,
  }

  window.__WORKER_BENCH__ = result
  return result
}
