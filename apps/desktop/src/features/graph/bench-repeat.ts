/**
 * Répétabilité : le même scénario, plusieurs fois de suite.
 *
 * Motif : le banc `bench-render.ts` a donné 65 fps (p5 53) lors de deux
 * exécutions, puis 36 fps (p5 1) une heure plus tard sans changement de code.
 * Une mesure non reproductible ne prouve rien — ni dans un sens ni dans
 * l'autre. Avant de conclure quoi que ce soit sur la Phase 2, il faut savoir
 * quelle est la dispersion réelle.
 *
 * On répète donc N fois le même protocole dans la même page, et on rapporte la
 * distribution plutôt qu'un chiffre unique.
 *
 * Chargé par `bench-repeat.html`.
 */

import Sigma from 'sigma'
import { generateScaleFreeGraph } from './generate-graph'

export interface RepeatRun {
  run: number
  fpsAvg: number
  fpsP5: number
  longestFrameMs: number
}

export interface RepeatSummary {
  nodes: number
  edges: number
  runs: RepeatRun[]
  fpsAvgMin: number
  fpsAvgMax: number
  fpsAvgMedian: number
  verdict: string
}

declare global {
  interface Window {
    __REPEAT__?: RepeatSummary
  }
}

export async function runRepeat(
  nodeCount = 20_000,
  edgeCount = 60_000,
  times = 5,
): Promise<RepeatSummary> {
  const container = document.getElementById('graph')
  if (!container) throw new Error('conteneur #graph introuvable')

  const runs: RepeatRun[] = []

  for (let run = 1; run <= times; run++) {
    const graph = generateScaleFreeGraph(nodeCount, edgeCount)

    // Positions étalées d'emblée : on isole le coût du rendu, pas du layout.
    let i = 0
    const n = graph.order
    graph.updateEachNodeAttributes((_node, attr) => {
      const angle = (i / n) * Math.PI * 2
      const radius = 2000 + (i % 613) * 4
      attr.x = Math.cos(angle) * radius
      attr.y = Math.sin(angle) * radius
      attr.size = 2
      attr.color = '#38bdf8'
      i++
      return attr
    })

    container.innerHTML = ''
    const renderer = new Sigma(graph, container as HTMLElement, {
      renderLabels: false,
      defaultNodeColor: '#38bdf8',
      defaultEdgeColor: 'rgba(148,163,184,0.15)',
    })

    const stats = await probe(renderer, 4000)
    renderer.kill()
    // Laisse le ramasse-miettes récupérer avant la mesure suivante, sinon la
    // pression mémoire de l'essai précédent contamine le suivant.
    await new Promise((r) => setTimeout(r, 1200))

    runs.push({ run, ...stats })
  }

  const avgs = runs.map((r) => r.fpsAvg).sort((a, b) => a - b)
  const median = avgs[Math.floor(avgs.length / 2)] ?? 0
  const min = avgs[0] ?? 0
  const max = avgs[avgs.length - 1] ?? 0

  const spread = max > 0 ? (max - min) / max : 0
  const verdict =
    spread > 0.4
      ? `Dispersion ${Math.round(spread * 100)} % : mesure non fiable, un chiffre unique ne veut rien dire.`
      : `Dispersion ${Math.round(spread * 100)} % : mesure reproductible.`

  const summary: RepeatSummary = {
    nodes: nodeCount,
    edges: edgeCount,
    runs,
    fpsAvgMin: min,
    fpsAvgMax: max,
    fpsAvgMedian: median,
    verdict,
  }
  window.__REPEAT__ = summary
  return summary
}

async function probe(
  renderer: Sigma,
  durationMs: number,
): Promise<{ fpsAvg: number; fpsP5: number; longestFrameMs: number }> {
  const camera = renderer.getCamera()
  const deltas: number[] = []
  let longest = 0
  const start = performance.now()
  let last = start

  return new Promise((resolve) => {
    function tick(): void {
      const now = performance.now()
      const d = now - last
      last = now
      if (d > 0 && now - start > 200) {
        deltas.push(d)
        if (d > longest) longest = d
      }

      const phase = (now - start) / durationMs
      camera.setState({
        x: 0.5 + Math.cos(phase * Math.PI * 4) * 0.15,
        y: 0.5 + Math.sin(phase * Math.PI * 4) * 0.15,
        ratio: 1.2 + Math.sin(phase * Math.PI * 2) * 0.6,
        angle: 0,
      })

      if (now - start < durationMs) {
        requestAnimationFrame(tick)
      } else {
        const fps = deltas.map((x) => 1000 / x).sort((a, b) => a - b)
        resolve({
          fpsAvg: Math.round(fps.reduce((s, v) => s + v, 0) / (fps.length || 1)),
          fpsP5: Math.round(fps[Math.floor(fps.length * 0.05)] ?? 0),
          longestFrameMs: Math.round(longest),
        })
      }
    }
    requestAnimationFrame(tick)
  })
}
