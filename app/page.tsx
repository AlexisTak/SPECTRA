'use client'

/**
 * Tableau de bord : état d'ensemble des dossiers.
 */

import Link from 'next/link'
import { listCases, type CaseRecord } from '@/lib/api'
import { useAsync } from '@/lib/hooks/useCases'
import { PageHeader } from '@/components/layout/shell'
import {
  Badge,
  Button,
  EmptyState,
  ErrorBox,
  Panel,
  formatDate,
} from '@/components/ui/primitives'

export default function DashboardPage() {
  const { data, loading, error, reload } = useAsync<CaseRecord[]>(() =>
    listCases(),
  )

  const cases = data ?? []
  const byStatus = countBy(cases, (c) => c.statut)
  const urgent = cases.filter(
    (c) => c.priorite === 'urgente' || c.priorite === 'haute',
  )

  return (
    <>
      <PageHeader
        title="Tableau de bord"
        subtitle="Vue d'ensemble des dossiers en cours"
        action={
          <Link href="/cases/new">
            <Button variant="primary">Nouveau dossier</Button>
          </Link>
        }
      />

      <div className="flex flex-col gap-6 p-8">
        {error && <ErrorBox message={error} />}

        {loading && (
          <p className="text-sm text-[var(--color-muted)]">Chargement…</p>
        )}

        {!loading && !error && (
          <>
            <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
              <Stat label="Dossiers" value={cases.length} />
              <Stat label="Ouverts" value={byStatus.ouvert ?? 0} />
              <Stat label="En cours" value={byStatus.en_cours ?? 0} />
              <Stat label="Clos" value={byStatus.clos ?? 0} />
            </div>

            <Panel
              title="Dossiers récents"
              action={
                <Link
                  href="/cases"
                  className="text-xs text-[var(--color-accent)] hover:underline"
                >
                  Tout voir
                </Link>
              }
            >
              {cases.length === 0 ? (
                <EmptyState
                  title="Aucun dossier"
                  hint="Créez un premier dossier pour commencer une enquête."
                  action={
                    <Link href="/cases/new" className="mt-2">
                      <Button variant="primary">Créer un dossier</Button>
                    </Link>
                  }
                />
              ) : (
                <ul className="flex flex-col divide-y divide-[var(--color-edge)]">
                  {cases.slice(0, 8).map((c) => (
                    <li key={c.id}>
                      <Link
                        href={`/cases/detail?id=${c.id}`}
                        className="flex items-center justify-between gap-4 py-2.5 transition-colors hover:text-[var(--color-accent)]"
                      >
                        <span className="flex min-w-0 items-center gap-3">
                          <code className="shrink-0 text-xs text-[var(--color-muted)]">
                            {c.reference}
                          </code>
                          <span className="truncate text-sm">{c.titre}</span>
                        </span>
                        <span className="flex shrink-0 items-center gap-3">
                          <Badge value={c.statut} />
                          <span className="text-xs text-[var(--color-muted)]">
                            {formatDate(c.dateCreation)}
                          </span>
                        </span>
                      </Link>
                    </li>
                  ))}
                </ul>
              )}
            </Panel>

            {urgent.length > 0 && (
              <Panel title="Priorité haute ou urgente">
                <ul className="flex flex-col divide-y divide-[var(--color-edge)]">
                  {urgent.map((c) => (
                    <li key={c.id}>
                      <Link
                        href={`/cases/detail?id=${c.id}`}
                        className="flex items-center justify-between gap-4 py-2.5 transition-colors hover:text-[var(--color-accent)]"
                      >
                        <span className="truncate text-sm">{c.titre}</span>
                        <Badge value={c.priorite} />
                      </Link>
                    </li>
                  ))}
                </ul>
              </Panel>
            )}

            <div>
              <Button onClick={reload}>Actualiser</Button>
            </div>
          </>
        )}
      </div>
    </>
  )
}

function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-lg border border-[var(--color-edge)] bg-[var(--color-panel)] px-5 py-4">
      <p className="text-xs text-[var(--color-muted)]">{label}</p>
      <p className="mt-1 text-2xl font-semibold tabular-nums">{value}</p>
    </div>
  )
}

function countBy<T>(items: T[], key: (item: T) => string): Record<string, number> {
  const out: Record<string, number> = {}
  for (const item of items) {
    const k = key(item)
    out[k] = (out[k] ?? 0) + 1
  }
  return out
}
