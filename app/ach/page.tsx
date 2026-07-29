'use client'

/**
 * ACH — Analysis of Competing Hypotheses.
 *
 * Structure inspirée de l'outil méthodologique de Richards Heuer (CIA).
 * Chaque hypothèse est évaluée contre un ensemble d'indices ;
 * le scoring aide à identifier laquelle explique le mieux l'ensemble
 * des données sans les contradictions.
 */

import { useState } from 'react'
import { PageHeader } from '@/components/layout/shell'
import { Button, Panel } from '@/components/ui/primitives'

type Consistency = 'coherent' | 'contradictoire' | 'neutre'

interface EvidenceRow {
  id: string
  label: string
}

interface Hypothesis {
  id: string
  title: string
  scores: Record<string, Consistency>
}

const EVIDENCES: EvidenceRow[] = [
  { id: 'ev1', label: 'Deux emails avec patronyme identique' },
  { id: 'ev2', label: 'Même avatar (pHash distance 2)' },
  { id: 'ev3', label: 'Fuseaux horaires d\'activité disjoints' },
  { id: 'ev4', label: 'Langue d\'interface différente (FR vs EN)' },
  { id: 'ev5', label: 'Style d\'écriture similaire (stylométrie)' },
]

const INIT_HYPOTHESES: Hypothesis[] = [
  {
    id: 'h1',
    title: 'Une seule personne (Alice Dupont)',
    scores: { ev1: 'coherent', ev2: 'coherent', ev3: 'contradictoire', ev4: 'contradictoire', ev5: 'coherent' },
  },
  {
    id: 'h2',
    title: 'Deux personnes distinctes (homonymes)',
    scores: { ev1: 'coherent', ev2: 'contradictoire', ev3: 'coherent', ev4: 'coherent', ev5: 'contradictoire' },
  },
  {
    id: 'h3',
    title: 'Alice Dupont avec une identité secondaire délibérée',
    scores: { ev1: 'coherent', ev2: 'coherent', ev3: 'neutre', ev4: 'neutre', ev5: 'coherent' },
  },
]

export default function AchPage() {
  const [hypotheses, setHypotheses] = useState<Hypothesis[]>(INIT_HYPOTHESES)
  const [evidences, setEvidences] = useState<EvidenceRow[]>(EVIDENCES)

  const setScore = (hId: string, evId: string, score: Consistency) => {
    setHypotheses((prev) =>
      prev.map((h) =>
        h.id === hId ? { ...h, scores: { ...h.scores, [evId]: score } } : h
      )
    )
  }

  const hypothesisScores = (h: Hypothesis) => {
    const vals = Object.values(h.scores)
    const coherent = vals.filter((v) => v === 'coherent').length
    const contradictoire = vals.filter((v) => v === 'contradictoire').length
    const neutre = vals.filter((v) => v === 'neutre').length
    return { coherent, contradictoire, neutre, total: vals.length }
  }

  return (
    <div>
      <PageHeader
        title="ACH"
        subtitle="Analysis of Competing Hypotheses"
      />
      <div className="px-8 py-6">
        <Panel className="overflow-auto">
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="border-b border-[var(--color-edge)]">
                <th className="sticky left-0 bg-[var(--color-panel)] px-3 py-3 text-xs font-medium uppercase tracking-wider text-[var(--color-muted)]">Indice / Hypothèse</th>
                {hypotheses.map((h) => {
                  const s = hypothesisScores(h)
                  return (
                    <th key={h.id} className="min-w-[12rem] px-3 py-3">
                      <div className="text-sm font-medium text-[var(--color-ink)]">{h.title}</div>
                      <div className="mt-1 flex gap-1">
                        <span className="inline-flex items-center rounded border border-emerald-500/40 bg-emerald-500/10 px-2 py-0.5 text-[10px] font-medium text-emerald-300">C{s.coherent}</span>
                        <span className="inline-flex items-center rounded border border-rose-500/40 bg-rose-500/10 px-2 py-0.5 text-[10px] font-medium text-rose-300">⊘{s.contradictoire}</span>
                        <span className="inline-flex items-center rounded border border-slate-500/40 bg-slate-500/10 px-2 py-0.5 text-[10px] font-medium text-slate-300">—{s.neutre}</span>
                      </div>
                    </th>
                  )
                })}
              </tr>
            </thead>
            <tbody className="divide-y divide-[var(--color-edge)]">
              {evidences.map((ev) => (
                <tr key={ev.id}>
                  <td className="sticky left-0 bg-[var(--color-panel)] px-3 py-3 text-[var(--color-ink)]">{ev.label}</td>
                  {hypotheses.map((h) => (
                    <td key={h.id} className="px-3 py-2">
                      <ScoreSelector
                        value={h.scores[ev.id] ?? 'neutre'}
                        onChange={(v) => setScore(h.id, ev.id, v)}
                      />
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </Panel>

        <div className="mt-4 flex gap-3">
          <Button
            variant="default"
            onClick={() => {
              const id = `h-${Date.now()}`
              const scores: Record<string, Consistency> = {}
              evidences.forEach((e) => (scores[e.id] = 'neutre'))
              setHypotheses((prev) => [
                ...prev,
                { id, title: 'Nouvelle hypothèse', scores },
              ])
            }}
          >
            + Hypothèse
          </Button>
          <Button
            variant="default"
            onClick={() => {
              const id = `ev-${Date.now()}`
              setEvidences((prev) => [...prev, { id, label: 'Nouvel indice' }])
              setHypotheses((prev) =>
                prev.map((h) => ({
                  ...h,
                  scores: { ...h.scores, [id]: 'neutre' },
                }))
              )
            }}
          >
            + Indice
          </Button>
        </div>
      </div>
    </div>
  )
}

function ScoreSelector({
  value,
  onChange,
}: {
  value: Consistency
  onChange: (v: Consistency) => void
}) {
  const options: { value: Consistency; label: string; tone: string }[] = [
    { value: 'coherent', label: 'C', tone: 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40' },
    { value: 'neutre', label: '—', tone: 'bg-slate-500/10 text-slate-400 border-slate-500/20' },
    { value: 'contradictoire', label: '⊘', tone: 'bg-rose-500/20 text-rose-300 border-rose-500/40' },
  ]
  return (
    <div className="flex gap-1">
      {options.map((opt) => (
        <button
          key={opt.value}
          onClick={() => onChange(opt.value)}
          className={`rounded border px-2 py-1 text-xs font-bold transition-all ${
            value === opt.value
              ? opt.tone + ' ring-1 ring-current'
              : 'border-transparent text-[var(--color-muted)] hover:bg-white/5'
          }`}
          title={opt.value}
        >
          {opt.label}
        </button>
      ))}
    </div>
  )
}
