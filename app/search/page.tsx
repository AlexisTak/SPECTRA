'use client'

/**
 * Recherche plein texte transversale.
 *
 * Le classement vient de `bm25()` (FTS5) : plus la valeur est **négative**,
 * plus le résultat est pertinent. L'implémentation d'origine renvoyait un score
 * constant de 0, ce qui rendait le tri et la troncature arbitraires
 * (`audit.md`, P2-3).
 */

import Link from 'next/link'
import { useState } from 'react'
import { reindexAll, search, type SearchHit } from '@/lib/api'
import { PageHeader } from '@/components/layout/shell'
import {
  Button,
  EmptyState,
  ErrorBox,
  Input,
  Panel,
  Select,
} from '@/components/ui/primitives'

const KINDS = [
  { value: '', label: 'Tout' },
  { value: 'case', label: 'Dossiers' },
  { value: 'evidence', label: 'Preuves' },
  { value: 'subject', label: 'Sujets' },
  { value: 'report', label: 'Rapports' },
  { value: 'note', label: 'Notes' },
]

const KIND_LABEL: Record<string, string> = {
  case: 'Dossier',
  evidence: 'Preuve',
  subject: 'Sujet',
  report: 'Rapport',
  note: 'Note',
  web_archive: 'Archive web',
  tiktok_video: 'Vidéo',
}

export default function SearchPage() {
  const [query, setQuery] = useState('')
  const [kind, setKind] = useState('')
  const [hits, setHits] = useState<SearchHit[] | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const run = async (e: React.FormEvent) => {
    e.preventDefault()
    if (query.trim() === '') return
    setBusy(true)
    setError(null)
    try {
      setHits(await search(query.trim(), kind ? [kind] : undefined))
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  const reindex = async () => {
    setBusy(true)
    setError(null)
    try {
      await reindexAll()
      setHits(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  return (
    <>
      <PageHeader
        title="Recherche"
        subtitle="Index plein texte sur l’ensemble des dossiers"
        action={
          <Button onClick={() => void reindex()} disabled={busy}>
            Reconstruire l’index
          </Button>
        }
      />

      <div className="flex flex-col gap-6 p-8">
        <Panel>
          <form onSubmit={run} className="flex flex-wrap items-end gap-3">
            <div className="flex flex-1 flex-col gap-1.5">
              <span className="text-xs font-medium text-[var(--color-muted)]">
                Termes
              </span>
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Mots-clés"
                autoFocus
              />
            </div>
            <div className="flex flex-col gap-1.5">
              <span className="text-xs font-medium text-[var(--color-muted)]">
                Type
              </span>
              <Select value={kind} onChange={(e) => setKind(e.target.value)}>
                {KINDS.map((k) => (
                  <option key={k.value} value={k.value}>
                    {k.label}
                  </option>
                ))}
              </Select>
            </div>
            <Button type="submit" variant="primary" disabled={busy}>
              {busy ? 'Recherche…' : 'Rechercher'}
            </Button>
          </form>
        </Panel>

        {error && <ErrorBox message={error} />}

        {hits && (
          <Panel title={`${hits.length} résultat${hits.length > 1 ? 's' : ''}`}>
            {hits.length === 0 ? (
              <EmptyState
                title="Aucun résultat"
                hint="Vérifiez l’orthographe, ou reconstruisez l’index si des données ont été importées hors de l’application."
              />
            ) : (
              <ul className="flex flex-col divide-y divide-[var(--color-edge)]">
                {hits.map((h) => (
                  <li key={`${h.kind}-${h.id}`} className="flex flex-col gap-1 py-3">
                    <div className="flex items-center justify-between gap-3">
                      {h.caseId ? (
                        <Link
                          href={`/cases/detail?id=${h.caseId}`}
                          className="truncate text-sm hover:text-[var(--color-accent)]"
                        >
                          {h.title}
                        </Link>
                      ) : (
                        <span className="truncate text-sm">{h.title}</span>
                      )}
                      <span className="shrink-0 rounded border border-[var(--color-edge)] px-2 py-0.5 text-xs text-[var(--color-muted)]">
                        {KIND_LABEL[h.kind] ?? h.kind}
                      </span>
                    </div>
                    {h.snippet && (
                      <p className="line-clamp-2 text-xs text-[var(--color-muted)]">
                        {h.snippet}
                      </p>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </Panel>
        )}
      </div>
    </>
  )
}
