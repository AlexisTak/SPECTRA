'use client'

/**
 * Visualisation graphe d'un dossier — avec compteur FPS intégré et mode benchmark.
 *
 * Composant client uniquement : Sigma nécessite WebGL2, indisponible en SSR.
 *
 * # Compteur FPS
 *
 * Mesure via `requestAnimationFrame` dans la vraie fenêtre Tauri/WebView2,
 * contrairement à Playwright qui throttle à 1 Hz. Affiche : instantané,
 * moyenne glissante, min/max, et mémoire heap.
 *
 * # Mode benchmark
 *
 * Bouton "Benchmark 50k nœuds" qui charge le graphe synthétique (loi de
 * puissance) et le layout détaché. Le FPS est mesuré en temps réel pendant
 * le layout et les interactions.
 */

import { useSearchParams } from 'next/navigation'
import { useEffect, useRef, useState, useCallback } from 'react'
import Graph from 'graphology'
import Sigma from 'sigma'
import FA2LayoutSupervisor from 'graphology-layout-forceatlas2/worker'
import { useFpsCounter } from './use-fps-counter'
import { generateScaleFreeGraph } from '@/lib/graph/generate-graph'
import {
  runLayoutDetached,
  FA2_SETTINGS,
  type LayoutProgress,
} from '@/lib/graph/layout-worker'

export default function GraphView() {
  const params = useSearchParams()
  const caseId = params.get('id')
  const containerRef = useRef<HTMLDivElement>(null)
  const sigmaRef = useRef<Sigma | null>(null)
  const [status, setStatus] = useState<'idle' | 'loading' | 'layout' | 'ready'>('idle')
  const [elapsedMs, setElapsedMs] = useState(0)
  const [nodeCount, setNodeCount] = useState(0)
  const [edgeCount, setEdgeCount] = useState(0)
  const [layoutProgress, setLayoutProgress] = useState<LayoutProgress | null>(null)
  const [stressResult, setStressResult] = useState<{ avg: number; min: number; max: number; durationMs: number } | null>(null)
  const stressRafRef = useRef<number>(0)
  const fps = useFpsCounter({ windowSize: 60, updateIntervalMs: 250 })

  const clearGraph = useCallback(() => {
    if (stressRafRef.current) {
      cancelAnimationFrame(stressRafRef.current)
      stressRafRef.current = 0
    }
    if (sigmaRef.current) {
      sigmaRef.current.kill()
      sigmaRef.current = null
    }
    setStatus('idle')
    setElapsedMs(0)
    setLayoutProgress(null)
    setStressResult(null)
  }, [])

  const loadDemoGraph = useCallback(() => {
    clearGraph()
    if (!containerRef.current) return

    const g = buildDemoGraph(caseId || 'demo')
    styleGraph(g)
    setNodeCount(g.order)
    setEdgeCount(g.size)

    sigmaRef.current = new Sigma(g, containerRef.current, {
      renderLabels: true,
      minCameraRatio: 0.001,
      maxCameraRatio: 10,
      labelRenderedSizeThreshold: 10,
      labelDensity: 0.1,
      labelGridCellSize: 60,
      zIndex: true,
      defaultNodeColor: '#38bdf8',
      defaultEdgeColor: 'rgba(148,163,184,0.2)',
    })

    setStatus('layout')
    const t0 = performance.now()

    const supervisor = new FA2LayoutSupervisor(g, { settings: FA2_SETTINGS })
    supervisor.start()

    const timer = setTimeout(() => {
      supervisor.stop()
      supervisor.kill()
      setStatus('ready')
      setElapsedMs(performance.now() - t0)
    }, 5_000)

    return () => {
      clearTimeout(timer)
      supervisor.stop()
      supervisor.kill()
    }
  }, [caseId, clearGraph])

  const runStressTest = useCallback(() => {
    if (!sigmaRef.current) return
    const renderer = sigmaRef.current
    const camera = renderer.getCamera()
    const samples: number[] = []
    const durationMs = 6_000
    const start = performance.now()
    let last = start

    const step = (): void => {
      const now = performance.now()
      const delta = now - last
      last = now
      const elapsed = now - start

      // Ignorer la première frame (coût d'amorçage)
      if (delta > 0 && elapsed > 100) {
        samples.push(1000 / delta)
      }

      const phase = elapsed / durationMs
      // Trajectoire déterministe : pan circulaire + zoom in/out
      camera.setState({
        x: 0.5 + Math.cos(phase * Math.PI * 4) * 0.15,
        y: 0.5 + Math.sin(phase * Math.PI * 4) * 0.15,
        ratio: 1.2 + Math.sin(phase * Math.PI * 2) * 0.8,
        angle: 0,
      })

      if (elapsed < durationMs) {
        stressRafRef.current = requestAnimationFrame(step)
      } else {
        const sorted = [...samples].sort((a, b) => a - b)
        const avg = samples.reduce((s, v) => s + v, 0) / (samples.length || 1)
        setStressResult({
          avg: Math.round(avg),
          min: Math.round(sorted[0] ?? 0),
          max: Math.round(sorted[sorted.length - 1] ?? 0),
          durationMs: Math.round(elapsed),
        })
      }
    }

    setStressResult(null)
    stressRafRef.current = requestAnimationFrame(step)
  }, [])

  const loadBenchmarkGraph = useCallback(() => {
    clearGraph()
    if (!containerRef.current) return

    setStatus('loading')
    const t0 = performance.now()

    // La génération peut bloquer brièvement : utiliser setTimeout pour laisser
    // React mettre à jour l'UI (spinner "Chargement…") avant le gros travail.
    setTimeout(() => {
      const g = generateScaleFreeGraph(50_000, 150_000)
      styleGraph(g)
      setNodeCount(g.order)
      setEdgeCount(g.size)

      sigmaRef.current = new Sigma(g, containerRef.current!, {
        renderLabels: true,
        minCameraRatio: 0.0001,
        maxCameraRatio: 10,
        labelRenderedSizeThreshold: 14,
        labelDensity: 0.07,
        labelGridCellSize: 60,
        zIndex: true,
        defaultNodeColor: '#38bdf8',
        defaultEdgeColor: 'rgba(148,163,184,0.15)',
      })

      setStatus('layout')
      const layoutT0 = performance.now()

      const handle = runLayoutDetached(g, {
        maxDurationMs: 15_000,
        stabilityThreshold: 0.35,
        sampleIntervalMs: 400,
        syncIntervalMs: 500,
        minDurationMs: 2_000,
        chunkSize: 5_000,
        onProgress: (p) => {
          setLayoutProgress(p)
          if (p.done) {
            setStatus('ready')
            setElapsedMs(performance.now() - layoutT0)
          }
        },
      })

      // Nettoyage si le composant est démonté avant la fin du layout
      return () => {
        handle.cancel()
      }
    }, 50)
  }, [clearGraph])

  // Chargement initial : graphe de démo
  useEffect(() => {
    const cleanup = loadDemoGraph()
    return () => {
      cleanup?.()
      clearGraph()
    }
  }, [loadDemoGraph, clearGraph])

  const statusLabel =
    status === 'idle'
      ? 'Inactif'
      : status === 'loading'
      ? 'Génération…'
      : status === 'layout'
      ? `Layout ${layoutProgress ? `— ${(layoutProgress.elapsedMs / 1000).toFixed(1)} s` : 'en cours…'}`
      : 'Prêt'

  const statusColor =
    status === 'ready'
      ? 'border-emerald-500/40 bg-emerald-500/10 text-emerald-200'
      : status === 'layout'
      ? 'border-amber-500/40 bg-amber-500/10 text-amber-200'
      : 'border-slate-500/40 bg-slate-500/10 text-slate-300'

  const fpsColor =
    fps.avg >= 45 ? 'text-emerald-300' : fps.avg >= 30 ? 'text-amber-300' : 'text-rose-300'

  return (
    <div className="relative h-screen w-full bg-[#0b1120]">
      <div ref={containerRef} className="absolute inset-0" />

      {/* Overlay haut-gauche : titre + contrôles */}
      <div className="pointer-events-auto absolute left-4 top-4 flex flex-col gap-2">
        <div>
          <h1 className="text-lg font-semibold text-slate-100">
            Graphe {caseId ? `du dossier ${caseId.slice(0, 8)}…` : '(démonstration)'}
          </h1>
          <p className="text-xs text-slate-400">
            {nodeCount > 0 && `${nodeCount.toLocaleString('fr')} nœuds · ${edgeCount.toLocaleString('fr')} arêtes`}
          </p>
        </div>

        <div className="flex flex-wrap gap-2">
          <button
            onClick={loadDemoGraph}
            className="rounded bg-slate-700/80 px-3 py-1.5 text-xs font-medium text-slate-200 transition hover:bg-slate-600"
          >
            Démonstration (230 nœuds)
          </button>
          <button
            onClick={loadBenchmarkGraph}
            disabled={status === 'loading' || status === 'layout'}
            className="rounded bg-indigo-600/80 px-3 py-1.5 text-xs font-medium text-white transition hover:bg-indigo-500 disabled:cursor-not-allowed disabled:opacity-50"
          >
            Benchmark 50k nœuds
          </button>
          {status === 'ready' && (
            <button
              onClick={runStressTest}
              className="rounded bg-rose-600/80 px-3 py-1.5 text-xs font-medium text-white transition hover:bg-rose-500"
            >
              Stress test (6 s)
            </button>
          )}
        </div>
      </div>

      {/* Overlay haut-droite : compteur FPS + mémoire + statut */}
      <div className="pointer-events-none absolute right-4 top-4 flex flex-col items-end gap-2">
        {/* Statut layout */}
        <div
          className={`rounded border px-3 py-2 text-xs font-medium ${statusColor}`}
        >
          {statusLabel}
          {status === 'ready' && ` — ${(elapsedMs / 1000).toFixed(1)} s`}
        </div>

        {/* Compteur FPS */}
        {status !== 'idle' && (
          <div className="rounded border border-slate-500/40 bg-slate-900/80 px-3 py-2 text-xs">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wider text-slate-500">
              Performance
            </div>
            <div className="flex gap-4">
              <div className="text-right">
                <div className={`text-lg font-bold tabular-nums leading-none ${fpsColor}`}>
                  {fps.avg}
                </div>
                <div className="mt-0.5 text-[10px] text-slate-500">FPS moy.</div>
              </div>
              <div className="text-right">
                <div className="text-lg font-bold tabular-nums leading-none text-slate-200">
                  {fps.min}
                </div>
                <div className="mt-0.5 text-[10px] text-slate-500">min</div>
              </div>
              <div className="text-right">
                <div className="text-lg font-bold tabular-nums leading-none text-slate-200">
                  {fps.max}
                </div>
                <div className="mt-0.5 text-[10px] text-slate-500">max</div>
              </div>
            </div>
            {fps.heapMb > 0 && (
              <div className="mt-1.5 border-t border-slate-700 pt-1 text-[10px] text-slate-500">
                Heap {fps.heapMb.toLocaleString('fr')} Mo
              </div>
            )}
            {stressResult && (
              <div className="mt-1.5 border-t border-slate-700 pt-1">
                <div className="mb-0.5 text-[10px] font-semibold uppercase tracking-wider text-slate-500">
                  Stress test ({(stressResult.durationMs / 1000).toFixed(1)} s)
                </div>
                <div className="flex gap-3">
                  <div className="text-right">
                    <div className="text-sm font-bold tabular-nums leading-none text-slate-200">{stressResult.avg}</div>
                    <div className="text-[9px] text-slate-500">avg</div>
                  </div>
                  <div className="text-right">
                    <div className="text-sm font-bold tabular-nums leading-none text-rose-300">{stressResult.min}</div>
                    <div className="text-[9px] text-slate-500">min</div>
                  </div>
                  <div className="text-right">
                    <div className="text-sm font-bold tabular-nums leading-none text-emerald-300">{stressResult.max}</div>
                    <div className="text-[9px] text-slate-500">max</div>
                  </div>
                </div>
                <div className="mt-1 text-[10px] text-slate-400">
                  {stressResult.avg >= 45
                    ? 'Conforme (>= 45 fps)'
                    : stressResult.avg >= 30
                    ? 'Dégradé (30-45 fps)'
                    : 'Non conforme (< 30 fps)'}
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Overlay bas-droite : légende */}
      <div className="pointer-events-none absolute bottom-4 right-4">
        <div className="rounded border border-slate-500/20 bg-slate-900/60 px-3 py-2 text-[10px] text-slate-400">
          <div className="flex items-center gap-2">
            <span className="inline-block h-2 w-2 rounded-full bg-sky-400" />
            Sujet
          </div>
          <div className="flex items-center gap-2">
            <span className="inline-block h-2 w-2 rounded-full bg-emerald-400" />
            Preuve
          </div>
          <div className="flex items-center gap-2">
            <span className="inline-block h-2 w-2 rounded-full bg-amber-400" />
            Événement
          </div>
        </div>
      </div>
    </div>
  )
}

function buildDemoGraph(caseId: string): Graph {
  const g = new Graph({ type: 'undirected' })

  for (let i = 0; i < 50; i++) {
    g.addNode(`s${i}`, { kind: 'subject', label: `Sujet ${i}` })
  }
  for (let i = 0; i < 150; i++) {
    g.addNode(`e${i}`, { kind: 'evidence', label: `Preuve ${i}` })
  }
  for (let i = 0; i < 30; i++) {
    g.addNode(`ev${i}`, { kind: 'event', label: `Événement ${i}` })
  }

  for (let i = 0; i < 200; i++) {
    const s = `s${Math.floor(Math.random() * 50)}`
    const e = `e${Math.floor(Math.random() * 150)}`
    if (!g.hasEdge(s, e)) g.addEdge(s, e)
  }
  for (let i = 0; i < 100; i++) {
    const e = `e${Math.floor(Math.random() * 150)}`
    const ev = `ev${Math.floor(Math.random() * 30)}`
    if (!g.hasEdge(e, ev)) g.addEdge(e, ev)
  }

  return g
}

const PALETTE = {
  subject: '#38bdf8',
  evidence: '#4ade80',
  event: '#fbbf24',
  default: '#94a3b8',
}

function styleGraph(g: Graph) {
  g.forEachNode((node) => {
    if (!g.hasNodeAttribute(node, 'x')) g.setNodeAttribute(node, 'x', Math.random() * 1000)
    if (!g.hasNodeAttribute(node, 'y')) g.setNodeAttribute(node, 'y', Math.random() * 1000)
    if (!g.hasNodeAttribute(node, 'size')) {
      const degree = g.degree(node)
      g.setNodeAttribute(node, 'size', Math.min(12, 1 + Math.sqrt(degree)))
    }
    if (!g.hasNodeAttribute(node, 'color')) {
      const kind = (g.getNodeAttribute(node, 'kind') as string) ?? 'default'
      g.setNodeAttribute(
        node,
        'color',
        // @ts-expect-error — palette lookup
        PALETTE[kind] ?? PALETTE.default,
      )
    }
    if (!g.hasNodeAttribute(node, 'label')) {
      g.setNodeAttribute(node, 'label', node)
    }
  })

  g.forEachEdge((edge) => {
    if (!g.hasEdgeAttribute(edge, 'color')) {
      g.setEdgeAttribute(edge, 'color', 'rgba(148,163,184,0.15)')
    }
  })
}
