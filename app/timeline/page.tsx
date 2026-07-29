'use client'

/**
 * Timeline — vue chronologique synchronisée avec le graphe.
 *
 * Chaque événement porte un observed_at. Le curseur temporel permet de
 * rejouer l'état du dossier à n'importe quelle date.
 */

import { useState } from 'react'
import { useAsync } from '@/lib/hooks/useCases'
import { PageHeader } from '@/components/layout/shell'
import { ErrorBox, Panel, formatDateTime } from '@/components/ui/primitives'
import { listAudit } from '@/lib/api'

interface TimelineEvent {
  id: string
  timestamp: string
  action: string
  actor: string
  entityKind?: string
  entityId?: string
  details?: string
}

export default function TimelinePage() {
  const [filter, setFilter] = useState('')
  const auditQuery = useAsync<TimelineEvent[]>(async () => {
    const rows = await listAudit('default', { limit: 100 })
    return rows.map((r: any) => ({
      id: String(r.id ?? r.sequence),
      timestamp: r.timestamp,
      action: r.action,
      actor: r.actor,
      entityKind: r.entityKind,
      entityId: r.entityId,
      details: r.payload,
    }))
  })

  const events = (auditQuery.data ?? [])
    .filter((e) => {
      if (!filter) return true
      const q = filter.toLowerCase()
      return (
        e.action.toLowerCase().includes(q) ||
        e.actor.toLowerCase().includes(q) ||
        (e.entityKind?.toLowerCase().includes(q) ?? false)
      )
    })
    .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())

  return (
    <div>
      <PageHeader
        title="Timeline"
        subtitle="Chronologie des événements du dossier"
      />
      <div className="px-8 py-6">
        {auditQuery.error && <ErrorBox message={String(auditQuery.error)} />}
        <div className="mb-4 flex items-center gap-3">
          <input
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder="Filtrer par action, acteur ou type…"
            className="w-full max-w-md rounded border border-[var(--color-edge)] bg-transparent px-3 py-2 text-sm text-[var(--color-ink)] placeholder:text-[var(--color-muted)] outline-none focus:border-sky-500"
          />
          <span className="text-xs text-[var(--color-muted)]">{events.length} événements</span>
        </div>
        <Panel>
          <div className="relative">
            <div className="absolute left-4 top-0 bottom-0 w-px bg-[var(--color-edge)]" />
            <ul className="space-y-4">
              {events.map((e) => (
                <li key={e.id} className="relative flex items-start gap-4 pl-10">
                  <span className="absolute left-2.5 top-2 h-3 w-3 rounded-full border-2 border-[var(--color-panel)] bg-sky-500" />
                  <div className="flex-1 rounded border border-[var(--color-edge)] bg-[var(--color-panel)]/50 p-3">
                    <div className="flex items-center justify-between gap-3">
                      <span className="text-sm font-medium text-[var(--color-ink)]">
                        {e.action}
                      </span>
                      <span className="inline-flex items-center rounded border border-slate-500/40 bg-slate-500/10 px-2 py-0.5 text-xs font-medium text-slate-300">{formatDateTime(e.timestamp)}</span>
                    </div>
                    <p className="mt-1 text-xs text-[var(--color-muted)]">
                      Par <span className="text-[var(--color-ink)]">{e.actor}</span>
                      {e.entityKind && (
                        <>
                          {' · '}
                          <span className="text-sky-300">{e.entityKind}</span>
                          {e.entityId && (
                            <span className="font-mono text-[10px] opacity-60"> #{e.entityId.slice(0, 8)}</span>
                          )}
                        </>
                      )}
                    </p>
                    {e.details && (
                      <pre className="mt-2 max-h-32 overflow-auto rounded bg-black/30 p-2 text-[10px] text-[var(--color-muted)]">
                        {e.details}
                      </pre>
                    )}
                  </div>
                </li>
              ))}
              {events.length === 0 && (
                <li className="pl-10 text-sm text-[var(--color-muted)]">Aucun événement</li>
              )}
            </ul>
          </div>
        </Panel>
      </div>
    </div>
  )
}
