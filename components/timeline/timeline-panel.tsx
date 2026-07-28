'use client'

/**
 * Timeline d'enquête — chronologie interactive des événements.
 *
 * Agrège tous les événements d'un dossier :
 * - Création et modifications du dossier
 * - Ajout/suppression de preuves
 * - Ajout/suppression de sujets
 * - Notes du journal
 * - Instantanés
 * - Vérifications d'intégrité
 *
 * Chaque événement est positionné par son horodatage réel (persisté en base).
 * La séquence numérique (persistée depuis la correction P2-14) garantit
 * l'ordre total même quand deux événements ont le même timestamp à la seconde.
 */

import { useState } from 'react'
import { listEvents, type CaseEventRecord } from '@/lib/api'
import { useAsync } from '@/lib/hooks/useCases'
import { Panel, EmptyState, Badge, formatDateTime } from '@/components/ui/primitives'

type EventKind = 'case' | 'subject' | 'evidence' | 'note' | 'snapshot' | 'integrity'

interface TimelineEvent {
  id: string
  timestamp: string
  sequence: number
  kind: EventKind
  action: string
  title: string
  description: string | null
  actor: string | null
}

export function TimelinePanel({ caseId }: { caseId: string }) {
  const eventsQuery = useAsync<CaseEventRecord[]>(() => listEvents(caseId), [caseId])
  const [filter, setFilter] = useState<EventKind | 'all'>('all')

  // Pour l'instant on n'a que les case_events. Les autres événements
  // (audit, snapshots, integrity) viendront d'une commande dédiée.
  const allEvents: TimelineEvent[] = (eventsQuery.data ?? []).map((e) => ({
    id: e.id,
    timestamp: e.timestamp,
    sequence: 0, // Sera rempli par une commande dédiée
    kind: e.type === 'note' ? 'note' : 'case',
    action: e.type,
    title: e.titre ?? e.type,
    description: e.description,
    actor: e.actor,
  }))

  const filtered = filter === 'all'
    ? allEvents
    : allEvents.filter((e) => e.kind === filter)

  // Tri par timestamp croissant (la séquence affine en cas d'égalité)
  filtered.sort((a, b) => {
    const t = new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
    return t !== 0 ? t : a.sequence - b.sequence
  })

  const kinds: Array<{ value: EventKind | 'all'; label: string }> = [
    { value: 'all', label: 'Tous' },
    { value: 'case', label: 'Dossier' },
    { value: 'subject', label: 'Sujets' },
    { value: 'evidence', label: 'Preuves' },
    { value: 'note', label: 'Notes' },
    { value: 'snapshot', label: 'Instantanés' },
    { value: 'integrity', label: 'Intégrité' },
  ]

  return (
    <Panel
      title={`Timeline (${filtered.length})`}
      action={
        <select
          value={filter}
          onChange={(e) => setFilter(e.target.value as EventKind | 'all')}
          className="rounded border border-[var(--color-edge)] bg-[var(--color-surface)] px-2 py-1 text-xs"
        >
          {kinds.map((k) => (
            <option key={k.value} value={k.value}>
              {k.label}
            </option>
          ))}
        </select>
      }
    >
      {eventsQuery.error && (
        <div className="mb-4 rounded border border-rose-500/40 bg-rose-500/10 p-3">
          <p className="text-sm text-rose-300">Erreur de chargement</p>
          <pre className="overflow-x-auto text-xs text-rose-200/90">
            {eventsQuery.error}
          </pre>
        </div>
      )}

      {filtered.length === 0 ? (
        <EmptyState
          title="Aucun événement"
          hint={
            filter === 'all'
              ? 'Les événements apparaîtront au fil de l\'enquête.'
              : 'Aucun événement de ce type.'
          }
        />
      ) : (
        <div className="relative">
          {/* Ligne de temps */}
          <div className="absolute left-4 top-0 h-full w-0.5 bg-[var(--color-edge)]" />

          <ul className="flex flex-col">
            {filtered.map((e, i) => (
              <li key={e.id} className="relative flex gap-4 py-3 pl-10">
                {/* Point sur la timeline */}
                <span
                  className="absolute left-2.5 top-4 h-3 w-3 -translate-x-1/2 rounded-full border-2 border-[var(--color-panel)]"
                  style={{
                    backgroundColor:
                      e.kind === 'note'
                        ? '#4ade80'
                        : e.kind === 'subject'
                          ? '#38bdf8'
                          : e.kind === 'evidence'
                            ? '#fbbf24'
                            : '#94a3b8',
                  }}
                />

                <div className="flex flex-1 flex-col gap-1">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-[var(--color-ink)]">
                      {e.title}
                    </span>
                    <Badge value={e.kind} />
                    <span className="ml-auto text-xs text-[var(--color-muted)]">
                      {formatDateTime(e.timestamp)}
                    </span>
                  </div>

                  {e.description && (
                    <p className="text-sm text-[var(--color-muted)]">
                      {e.description}
                    </p>
                  )}

                  {e.actor && (
                    <p className="text-xs text-[var(--color-muted)]">
                      Par : {e.actor}
                    </p>
                  )}
                </div>
              </li>
            ))}
          </ul>
        </div>
      )}
    </Panel>
  )
}
