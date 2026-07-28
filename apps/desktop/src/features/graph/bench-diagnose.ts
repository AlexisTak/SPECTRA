/**
 * Diagnostic : d'où vient le blocage du thread principal malgré le worker ?
 *
 * Le banc `bench-worker.ts` montre 21 fps moyen et une frame à 1 014 ms alors
 * que le calcul FA2 tourne bien dans un worker. Deux suspects :
 *
 * A. **La réécriture des positions.** `assignLayoutChanges` parcourt les 50 000
 *    nœuds sur le thread principal à chaque message du worker.
 * B. **La réindexation de Sigma.** Chaque écriture d'attribut émet un événement
 *    graphology que Sigma écoute pour rafraîchir ses index.
 *
 * On isole en mesurant la réactivité du thread principal dans deux
 * configurations : superviseur **avec** Sigma attaché, puis **sans**. La
 * différence attribue le coût à l'un ou l'autre.
 *
 * Chargé par `bench-diagnose.html`.
 */

import Sigma from 'sigma'
import { generateScaleFreeGraph } from './generate-graph'
import { runLayoutInWorker } from './layout-worker'

export interface DiagnoseResult {
  withSigma: ProbeStats
  withoutSigma: ProbeStats
  verdict: string
}

interface ProbeStats {
  label: string
  fpsAvg: number
  fpsP5: number
  longestFrameMs: number
  layoutMs: number
}

declare global {
  interface Window {
    __DIAGNOSE__?: DiagnoseResult
    __DIAGNOSE_ERROR__?: string
  }
}

export async function runDiagnose(
  nodeCount = 50_000,
  edgeCount = 150_000,
): Promise<DiagnoseResult> {
  const withoutSigma = await probe('sans Sigma', nodeCount, edgeCount, false)
  const withSigma = await probe('avec Sigma', nodeCount, edgeCount, true)

  let verdict: string
  if (withoutSigma.fpsAvg >= 45 && withSigma.fpsAvg < 45) {
    verdict =
      'Le coût vient de la réindexation Sigma déclenchée par la réécriture continue des positions.'
  } else if (withoutSigma.fpsAvg < 45) {
    verdict =
      'Le coût vient de la réécriture des 50 000 positions sur le thread principal, indépendamment de Sigma.'
  } else {
    verdict = 'Les deux configurations tiennent le budget.'
  }

  const result: DiagnoseResult = { withSigma, withoutSigma, verdict }
  window.__DIAGNOSE__ = result
  return result
}

async function probe(
  label: string,
  nodeCount: number,
  edgeCount: number,
  attachSigma: boolean,
): Promise<ProbeStats> {
  const graph = generateScaleFreeGraph(nodeCount, edgeCount)
  graph.forEachNode((node) => {
    graph.setNodeAttribute(node, 'size', 2)
    graph.setNodeAttribute(node, 'color', '#38bdf8')
  })

  let renderer: Sigma | undefined
  if (attachSigma) {
    const container = document.getElementById('graph')
    if (!container) throw new Error('conteneur #graph introuvable')
    container.innerHTML = ''
    renderer = new Sigma(graph, container as HTMLElement, {
      renderLabels: false,
      defaultNodeColor: '#38bdf8',
      defaultEdgeColor: 'rgba(148,163,184,0.15)',
    })
  }

  const deltas: number[] = []
  let longest = 0
  let probing = true
  let last = performance.now()
  const start = last

  function tick(): void {
    const now = performance.now()
    const d = now - last
    last = now
    if (d > 0 && now - start > 100) {
      deltas.push(d)
      if (d > longest) longest = d
    }
    if (probing) requestAnimationFrame(tick)
  }
  requestAnimationFrame(tick)

  // Budget court : on cherche à caractériser la réactivité, pas à converger.
  const handle = runLayoutInWorker(graph, { maxDurationMs: 8000 })
  const progress = await handle.finished
  probing = false
  await new Promise((r) => requestAnimationFrame(() => r(null)))

  renderer?.kill()

  const fps = deltas.map((d) => 1000 / d).sort((a, b) => a - b)
  return {
    label,
    fpsAvg: Math.round(fps.reduce((s, v) => s + v, 0) / (fps.length || 1)),
    fpsP5: Math.round(fps[Math.floor(fps.length * 0.05)] ?? 0),
    longestFrameMs: Math.round(longest),
    layoutMs: Math.round(progress.elapsedMs),
  }
}
