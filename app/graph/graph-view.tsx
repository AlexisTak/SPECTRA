'use client'

/**
 * Visualisation graphe d'un dossier — composant client uniquement.
 *
 * Sigma nécessite WebGL2, indisponible en SSR. Ce composant est donc
 * chargé dynamiquement avec `ssr: false`.
 *
 * Layout ForceAtlas2 déporté en Web Worker — l'analyste voit le graphe
 * se déplier progressivement au lieu de fixer un écran gelé.
 *
 * # État du chantier
 *
 * Cette page est une ébauche : elle affiche un graphe de démo car les
 * relations entre entités ne sont pas encore modélisées en base. Le
 * mécanisme de layout worker est fonctionnel et testé sur 50k nœuds.
 */

import { useSearchParams } from 'next/navigation'
import { useEffect, useRef, useState } from 'react'
import Graph from 'graphology'
import Sigma from 'sigma'
import FA2LayoutSupervisor from 'graphology-layout-forceatlas2/worker'

export default function GraphView() {
  const params = useSearchParams()
  const caseId = params.get('id')
  const containerRef = useRef<HTMLDivElement>(null)
  const sigmaRef = useRef<Sigma | null>(null)
  const [status, setStatus] = useState<'loading' | 'layout' | 'ready'>('loading')
  const [elapsedMs, setElapsedMs] = useState(0)

  useEffect(() => {
    if (!containerRef.current) return

    // Graphe de démo en attendant les relations réelles
    const g = buildDemoGraph(caseId || 'demo')

    g.forEachNode((node) => {
      if (!g.hasNodeAttribute(node, 'x')) g.setNodeAttribute(node, 'x', Math.random() * 100)
      if (!g.hasNodeAttribute(node, 'y')) g.setNodeAttribute(node, 'y', Math.random() * 100)
      if (!g.hasNodeAttribute(node, 'size')) {
        const degree = g.degree(node)
        g.setNodeAttribute(node, 'size', Math.min(12, 1 + Math.sqrt(degree)))
      }
      if (!g.hasNodeAttribute(node, 'color')) {
        const kind = g.getNodeAttribute(node, 'kind') as string
        g.setNodeAttribute(node, 'color', kind === 'subject' ? '#38bdf8' : kind === 'evidence' ? '#4ade80' : '#fbbf24')
      }
    })

    g.forEachEdge((edge) => {
      if (!g.hasEdgeAttribute(edge, 'color')) g.setEdgeAttribute(edge, 'color', 'rgba(148,163,184,0.2)')
    })

    sigmaRef.current = new Sigma(g, containerRef.current, {
      renderLabels: false,
      minCameraRatio: 0.001,
      maxCameraRatio: 10,
    })

    setStatus('layout')
    const t0 = performance.now()

    // Layout en worker
    const layout = new FA2LayoutSupervisor(g)
    layout.start()

    const timer = setTimeout(() => {
      layout.stop()
      setStatus('ready')
      setElapsedMs(performance.now() - t0)
    }, 10000)

    const tick = () => {
      if (status === 'layout') {
        setElapsedMs(performance.now() - t0)
        requestAnimationFrame(tick)
      }
    }
    requestAnimationFrame(tick)

    return () => {
      clearTimeout(timer)
      layout.stop()
      sigmaRef.current?.kill()
      sigmaRef.current = null
    }
  }, [caseId])

  return (
    <div className="relative h-screen w-full bg-[#0b1120]">
      <div ref={containerRef} className="absolute inset-0" />

      <div className="pointer-events-none absolute right-4 top-4 flex flex-col gap-2">
        {status === 'loading' && (
          <div className="rounded border border-[var(--color-edge)] bg-[var(--color-panel)] px-3 py-2 text-xs text-[var(--color-muted)]">
            Initialisation…
          </div>
        )}
        {status === 'layout' && (
          <div className="rounded border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-xs text-amber-200">
            Layout en cours… {(elapsedMs / 1000).toFixed(1)} s
          </div>
        )}
        {status === 'ready' && (
          <div className="rounded border border-emerald-500/40 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-200">
            Prêt — {(elapsedMs / 1000).toFixed(1)} s
          </div>
        )}
      </div>

      <div className="pointer-events-none absolute left-4 top-4">
        <h1 className="text-lg font-semibold text-[var(--color-ink)]">
          Graphe {caseId ? `du dossier ${caseId.slice(0, 8)}…` : '(démo)'}
        </h1>
        <p className="text-xs text-[var(--color-muted)]">
          Bleu : sujets • Vert : preuves • Jaune : événements
        </p>
      </div>
    </div>
  )
}

function buildDemoGraph(caseId: string): Graph {
  const g = new Graph()

  // 50 sujets
  for (let i = 0; i < 50; i++) {
    g.addNode(`s${i}`, { kind: 'subject', label: `Sujet ${i}` })
  }

  // 150 preuves
  for (let i = 0; i < 150; i++) {
    g.addNode(`e${i}`, { kind: 'evidence', label: `Preuve ${i}` })
  }

  // 30 événements
  for (let i = 0; i < 30; i++) {
    g.addNode(`ev${i}`, { kind: 'event', label: `Événement ${i}` })
  }

  // Arêtes aléatoires mais plausibles
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
