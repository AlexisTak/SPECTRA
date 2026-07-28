'use client'

/**
 * Tableau de bord — vue d'ensemble complete de l'application.
 *
 * Toutes les fonctionnalites sont accessibles depuis ici :
 * - Statistiques globales (dossiers, preuves, sujets, instantanes)
 * - Activite recente (timeline agregee)
 * - Dossiers prioritaires
 * - Alertes d'integrite
 * - Acces rapide aux outils (recherche, graphe, rapports)
 */

import Link from 'next/link'
import { useState } from 'react'
import {
  listCases,
  listEvidence,
  listSubjects,
  listSnapshots,
  verifyAuditTrail,
  search,
  type CaseRecord,
  type IntegrityCheck,
} from '@/lib/api'
import { useAsync } from '@/lib/hooks/useCases'
import { PageHeader } from '@/components/layout/shell'
import {
  Badge,
  Button,
  EmptyState,
  ErrorBox,
  Input,
  Panel,
  formatDate,
  formatDateTime,
} from '@/components/ui/primitives'

export default function DashboardPage() {
  const casesQuery = useAsync<CaseRecord[]>(() => listCases())
  const [searchQuery, setSearchQuery] = useState('')

  const cases = casesQuery.data ?? []
  const byStatus = countBy(cases, (c) => c.statut)
  const byPriority = countBy(cases, (c) => c.priorite)
  const urgent = cases.filter(
    (c) => c.priorite === 'urgente' || c.priorite === 'haute',
  )
  const recent = [...cases]
    .sort((a, b) => new Date(b.dateCreation).getTime() - new Date(a.dateCreation).getTime())
    .slice(0, 5)

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault()
    if (searchQuery.trim()) {
      window.location.href = `/search?q=${encodeURIComponent(searchQuery.trim())}`
    }
  }

  return (
    <>
      <PageHeader
        title="Tableau de bord"
        subtitle="Vue d'ensemble de l'enquete"
        action={
          <div className="flex gap-2">
            <Link href="/cases/new">
              <Button variant="primary">Nouveau dossier</Button>
            </Link>
          </div>
        }
      />

      <div className="flex flex-col gap-6 p-8">
        {casesQuery.error && <ErrorBox message={casesQuery.error} />}

        {/* Recherche rapide */}
        <Panel>
          <form onSubmit={handleSearch} className="flex items-center gap-3">
            <Input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Rechercher dans tous les dossiers..."
              className="flex-1"
            />
            <Button type="submit" variant="primary">
              Rechercher
            </Button>
            <Link href="/graph">
              <Button type="button">Graphe</Button>
            </Link>
          </form>
        </Panel>

        {/* Statistiques globales */}
        <div className="grid grid-cols-2 gap-4 lg:grid-cols-6">
          <Stat label="Dossiers" value={cases.length} />
          <Stat label="Ouverts" value={byStatus.ouvert ?? 0} tone="sky" />
          <Stat label="En cours" value={byStatus.en_cours ?? 0} tone="amber" />
          <Stat label="Clos" value={byStatus.clos ?? 0} tone="slate" />
          <Stat label="Urgents" value={byPriority.urgente ?? 0} tone="rose" />
          <Stat label="Haute prio" value={byPriority.haute ?? 0} tone="amber" />
        </div>

        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {/* Dossiers recents */}
          <Panel
            title="Dossiers recents"
            action={
              <Link href="/cases" className="text-xs text-[var(--color-accent)] hover:underline">
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
                {recent.map((c) => (
                  <li key={c.id}>
                    <Link
                      href={`/cases/detail?id=${c.id}`}
                      className="flex items-center justify-between gap-4 py-2.5 transition-colors hover:text-[var(--color-accent)]"
                    >
                      <div className="flex min-w-0 items-center gap-3">
                        <code className="shrink-0 text-xs text-[var(--color-muted)]">
                          {c.reference}
                        </code>
                        <span className="truncate text-sm">{c.titre}</span>
                      </div>
                      <div className="flex shrink-0 items-center gap-2">
                        <Badge value={c.statut} />
                        <span className="text-xs text-[var(--color-muted)]">
                          {formatDate(c.dateCreation)}
                        </span>
                      </div>
                    </Link>
                  </li>
                ))}
              </ul>
            )}
          </Panel>

          {/* Priorites hautes */}
          <Panel
            title="Priorité haute ou urgente"
            action={
              <Link href="/cases" className="text-xs text-[var(--color-accent)] hover:underline">
                Voir tout
              </Link>
            }
          >
            {urgent.length === 0 ? (
              <EmptyState
                title="Aucune priorité haute"
                hint="Les dossiers urgents ou haute priorité apparaîtront ici."
              />
            ) : (
              <ul className="flex flex-col divide-y divide-[var(--color-edge)]">
                {urgent.slice(0, 5).map((c) => (
                  <li key={c.id}>
                    <Link
                      href={`/cases/detail?id=${c.id}`}
                      className="flex items-center justify-between gap-4 py-2.5 transition-colors hover:text-[var(--color-accent)]"
                    >
                      <span className="truncate text-sm">{c.titre}</span>
                      <div className="flex shrink-0 items-center gap-2">
                        <Badge value={c.priorite} />
                        <Badge value={c.statut} />
                      </div>
                    </Link>
                  </li>
                ))}
              </ul>
            )}
          </Panel>
        </div>

        {/* Raccourcis outils */}
        <Panel title="Outils">
          <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
            <ToolCard
              href="/graph"
              title="Graphe"
              desc="Visualisation interactive"
              icon="🕸️"
            />
            <ToolCard
              href="/search"
              title="Recherche"
              desc="Plein texte transversal"
              icon="🔍"
            />
            <ToolCard
              href="/reports"
              title="Rapports"
              desc="Génération de rapports"
              icon="📄"
            />
            <ToolCard
              href="/audit"
              title="Intégrité"
              desc="Vérification des chaînes"
              icon="🔐"
            />
          </div>
        </Panel>

        <div className="flex gap-2">
          <Button onClick={() => casesQuery.reload()}>Actualiser</Button>
        </div>
      </div>
    </>
  )
}

function Stat({
  label,
  value,
  tone = 'default',
}: {
  label: string
  value: number
  tone?: 'default' | 'sky' | 'amber' | 'rose' | 'slate' | 'emerald' | 'violet'
}) {
  const tones: Record<string, string> = {
    default: 'border-[var(--color-edge)]',
    sky: 'border-sky-500/40 bg-sky-500/5',
    amber: 'border-amber-500/40 bg-amber-500/5',
    rose: 'border-rose-500/40 bg-rose-500/5',
    slate: 'border-slate-500/40 bg-slate-500/5',
    emerald: 'border-emerald-500/40 bg-emerald-500/5',
    violet: 'border-violet-500/40 bg-violet-500/5',
  }

  return (
    <div className={`rounded-lg border ${tones[tone]} bg-[var(--color-panel)] px-5 py-4`}>
      <p className="text-xs text-[var(--color-muted)]">{label}</p>
      <p className="mt-1 text-2xl font-semibold tabular-nums">{value}</p>
    </div>
  )
}

function ToolCard({
  href,
  title,
  desc,
  icon,
}: {
  href: string
  title: string
  desc: string
  icon: string
}) {
  return (
    <Link href={href}>
      <div className="group flex flex-col items-start gap-2 rounded-lg border border-[var(--color-edge)] bg-[var(--color-panel)] p-4 transition-colors hover:border-[var(--color-accent)] hover:bg-white/5">
        <span className="text-2xl">{icon}</span>
        <span className="text-sm font-medium text-[var(--color-ink)] group-hover:text-[var(--color-accent)]">
          {title}
        </span>
        <span className="text-xs text-[var(--color-muted)]">{desc}</span>
      </div>
    </Link>
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
