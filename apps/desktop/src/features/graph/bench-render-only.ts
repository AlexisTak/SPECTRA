/**
 * Contrôle : Sigma seul, sans aucun layout ni worker.
 *
 * Les bancs précédents montrent 1 à 4 fps pendant le layout, y compris sans
 * aucune synchronisation de positions. Avant d'incriminer le worker, il faut
 * établir la ligne de base : **que vaut le rendu seul** sur ce graphe ?
 *
 * Le tout premier banc (`bench-render.ts`) mesurait 65 fps — mais après un
 * layout FA2 convergé, donc sur des positions *étalées*. Ici les positions sont
 * celles du générateur : aléatoires dans un carré de 1000×1000, donc
 * extrêmement denses. Si les FPS s'effondrent, le coupable est la densité de
 * pixels, pas le worker.
 *
 * Chargé par `bench-render-only.html`.
 */

import Sigma from 'sigma'
import { generateScaleFreeGraph } from './generate-graph'

export interface RenderOnlyResult {
  nodes: number
  edges: number
  spread: string
  fpsAvg: number
  fpsP5: number
  longestFrameMs: number
}

declare global {
  interface Window {
    __RENDER_ONLY__?: RenderOnlyResult[]
  }
}

export async function runRenderOnly(
  nodeCount = 50_000,
  edgeCount = 150_000,
): Promise<RenderOnlyResult[]> {
  const results: RenderOnlyResult[] = []

  // Deux étalements pour isoler l'effet de la densité :
  // - « dense » : positions du générateur (carré 1000×1000)
  // - « étalé » : positions dispersées façon post-layout
  for (const spread of ['dense', 'étalé'] as const) {
    const graph = generateScaleFreeGraph(nodeCount, edgeCount)

    if (spread === 'étalé') {
      // Disposition en anneau bruité : approxime la géométrie d'un FA2
      // convergé sans avoir à le calculer.
      let i = 0
      const n = graph.order
      graph.updateEachNodeAttributes((_node, attr) => {
        const angle = (i / n) * Math.PI * 2
        const radius = 3000 + (i % 997) * 3
        attr.x = Math.cos(angle) * radius
        attr.y = Math.sin(angle) * radius
        i++
        return attr
      })
    }

    graph.updateEachNodeAttributes((_node, attr) => {
      attr.size = 2
      attr.color = '#38bdf8'
      return attr
    })

    const container = document.getElementById('graph')
    if (!container) throw new Error('conteneur #graph introuvable')
    container.innerHTML = ''

    const renderer = new Sigma(graph, container as HTMLElement, {
      renderLabels: false,
      defaultNodeColor: '#38bdf8',
      defaultEdgeColor: 'rgba(148,163,184,0.15)',
    })

    const stats = await probeFps(renderer, 5000)
    renderer.kill()

    results.push({
      nodes: graph.order,
      edges: graph.size,
      spread,
      ...stats,
    })
  }

  window.__RENDER_ONLY__ = results
  return results
}

async function probeFps(
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
        ratio: 1.2 + Math.sin(phase * Math.PI * 2) * 0.8,
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
