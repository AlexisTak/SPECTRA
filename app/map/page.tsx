'use client'

/**
 * Vue carte — géolocalisation des entités porteuses de coordonnées.
 *
 * Affiche un plan SVG stylisé avec une grille, les entités dotées de
 * latitude/longitude sont projetées et listées dans un panneau latéral.
 * Les tuiles ne sont pas chargées depuis un service externe (respect de C1/C2).
 */

import { useMemo, useState } from 'react'
import { PageHeader } from '@/components/layout/shell'
import { Panel } from '@/components/ui/primitives'

interface MapEntity {
  id: string
  label: string
  kind: string
  lat: number
  lng: number
  source: string
  confidence: string
}

const MOCK_LOCATIONS: MapEntity[] = [
  { id: 'loc1', label: 'Résidence principale', kind: 'Location', lat: 48.8566, lng: 2.3522, source: 'Manuel', confidence: 'A1' },
  { id: 'loc2', label: 'Bureau co.', kind: 'Location', lat: 48.8606, lng: 2.3376, source: 'OSINT', confidence: 'B2' },
  { id: 'loc3', label: 'Point de rencontre', kind: 'Location', lat: 48.8589, lng: 2.3470, source: 'Témoin', confidence: 'C3' },
  { id: 'loc4', label: 'Aéroport CDG', kind: 'Location', lat: 49.0097, lng: 2.5479, source: 'Billets', confidence: 'A1' },
  { id: 'loc5', label: 'Gare Lyon', kind: 'Location', lat: 48.8448, lng: 2.3745, source: 'Transport', confidence: 'B2' },
]

// Bounding box autour de l'Île-de-France pour la démo
const BBOX = { minLat: 48.7, maxLat: 49.2, minLng: 2.1, maxLng: 2.7 }

export default function MapPage() {
  const [hoveredId, setHoveredId] = useState<string | null>(null)
  const [selectedId, setSelectedId] = useState<string | null>(null)

  const projected = useMemo(() => {
    const width = 800
    const height = 600
    const xScale = width / (BBOX.maxLng - BBOX.minLng)
    const yScale = height / (BBOX.maxLat - BBOX.minLat)
    return MOCK_LOCATIONS.map((e) => ({
      ...e,
      x: (e.lng - BBOX.minLng) * xScale,
      y: height - (e.lat - BBOX.minLat) * yScale,
    }))
  }, [])

  const selected = projected.find((p) => p.id === selectedId)

  return (
    <div>
      <PageHeader title="Carte" subtitle="Géolocalisation des entités" />
      <div className="flex h-[calc(100vh-6rem)]">
        {/* Canvas SVG */}
        <div className="relative flex-1 bg-[#0b1020]">
          <svg
            viewBox="0 0 800 600"
            className="h-full w-full"
            preserveAspectRatio="xMidYMid meet"
          >
            {/* Grille */}
            {Array.from({ length: 11 }).map((_, i) => (
              <g key={`v${i}`}>
                <line
                  x1={i * 80}
                  y1={0}
                  x2={i * 80}
                  y2={600}
                  stroke="#1e293b"
                  strokeWidth={0.5}
                />
                <text
                  x={i * 80 + 2}
                  y={596}
                  fill="#334155"
                  fontSize={8}
                >
                  {(BBOX.minLng + (i * (BBOX.maxLng - BBOX.minLng)) / 10).toFixed(2)}°E
                </text>
              </g>
            ))}
            {Array.from({ length: 9 }).map((_, i) => (
              <g key={`h${i}`}>
                <line
                  x1={0}
                  y1={i * 75}
                  x2={800}
                  y2={i * 75}
                  stroke="#1e293b"
                  strokeWidth={0.5}
                />
                <text
                  x={4}
                  y={i * 75 + 10}
                  fill="#334155"
                  fontSize={8}
                >
                  {(BBOX.maxLat - (i * (BBOX.maxLat - BBOX.minLat)) / 8).toFixed(2)}°N
                </text>
              </g>
            ))}

            {/* Marqueurs */}
            {projected.map((p) => {
              const isActive = p.id === hoveredId || p.id === selectedId
              return (
                <g
                  key={p.id}
                  transform={`translate(${p.x}, ${p.y})`}
                  onMouseEnter={() => setHoveredId(p.id)}
                  onMouseLeave={() => setHoveredId(null)}
                  onClick={() => setSelectedId(p.id === selectedId ? null : p.id)}
                  style={{ cursor: 'pointer' }}
                >
                  <circle
                    r={isActive ? 8 : 5}
                    fill={isActive ? '#38bdf8' : '#0ea5e9'}
                    stroke="#0f172a"
                    strokeWidth={2}
                    opacity={0.9}
                  />
                  {isActive && (
                    <text
                      y={-12}
                      textAnchor="middle"
                      fill="#e2e8f0"
                      fontSize={10}
                      fontWeight={500}
                    >
                      {p.label}
                    </text>
                  )}
                </g>
              )
            })}
          </svg>

          {/* Légende overlay */}
          <div className="pointer-events-none absolute bottom-4 left-4 rounded border border-slate-700/50 bg-slate-900/80 px-3 py-2 text-xs text-slate-300 backdrop-blur">
            <div className="mb-1 font-medium text-slate-200">Entités géolocalisées</div>
            <div className="flex items-center gap-2">
              <span className="inline-block h-2 w-2 rounded-full bg-sky-500" />
              Collectée
              <span className="ml-2 inline-block h-2 w-2 rounded-full bg-sky-300" />
              Sélectionnée
            </div>
          </div>
        </div>

        {/* Panneau latéral */}
        <aside className="w-80 shrink-0 border-l border-[var(--color-edge)] bg-[var(--color-panel)]">
          <div className="px-5 py-4 text-sm font-medium text-[var(--color-ink)]">
            Points ({projected.length})
          </div>
          <div className="flex flex-col gap-1 px-3 pb-4">
            {projected.map((p) => (
              <button
                key={p.id}
                onClick={() => setSelectedId(p.id === selectedId ? null : p.id)}
                onMouseEnter={() => setHoveredId(p.id)}
                onMouseLeave={() => setHoveredId(null)}
                className={`flex flex-col gap-0.5 rounded px-3 py-2 text-left transition-colors ${
                  selectedId === p.id
                    ? 'bg-sky-500/10 ring-1 ring-sky-500/30'
                    : 'hover:bg-white/5'
                }`}
              >
                <span className="text-sm font-medium text-[var(--color-ink)]">
                  {p.label}
                </span>
                <span className="text-xs text-[var(--color-muted)]">
                  {p.lat.toFixed(4)}°N, {p.lng.toFixed(4)}°E · {p.source}
                </span>
              </button>
            ))}
          </div>

          {selected && (
            <div className="border-t border-[var(--color-edge)] px-5 py-4">
              <h3 className="mb-2 text-sm font-semibold text-[var(--color-ink)]">
                {selected.label}
              </h3>
              <dl className="space-y-1 text-xs text-[var(--color-muted)]">
                <div className="flex justify-between">
                  <dt>Type</dt>
                  <dd className="text-[var(--color-ink)]">{selected.kind}</dd>
                </div>
                <div className="flex justify-between">
                  <dt>Latitude</dt>
                  <dd className="font-mono text-[var(--color-ink)]">{selected.lat.toFixed(6)}</dd>
                </div>
                <div className="flex justify-between">
                  <dt>Longitude</dt>
                  <dd className="font-mono text-[var(--color-ink)]">{selected.lng.toFixed(6)}</dd>
                </div>
                <div className="flex justify-between">
                  <dt>Source</dt>
                  <dd className="text-[var(--color-ink)]">{selected.source}</dd>
                </div>
                <div className="flex justify-between">
                  <dt>Confiance</dt>
                  <dd className="text-[var(--color-ink)]">{selected.confidence}</dd>
                </div>
              </dl>
            </div>
          )}
        </aside>
      </div>
    </div>
  )
}
