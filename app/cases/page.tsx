'use client'

/**
 * Liste des dossiers, avec filtre par statut et recherche.
 *
 * Les deux filtres sont indépendants côté backend depuis la correction du
 * défaut P2-6 : une recherche sans filtre de statut ne doit pas masquer
 * silencieusement les dossiers clos.
 */

import Link from 'next/link'
import { useState } from 'react'
import { listCases, type CaseRecord, type CaseStatus } from '@/lib/api'
import { useAsync } from '@/lib/hooks/useCases'
import { PageHeader } from '@/components/layout/shell'
import {
  Badge,
  Button,
  EmptyState,
  ErrorBox,
  Input,
  Panel,
  Select,
  formatDate,
} from '@/components/ui/primitives'

const STATUSES: Array<{ value: '' | CaseStatus; label: string }> = [
  { value: '', label: 'Tous les statuts' },
  { value: 'ouvert', label: 'Ouvert' },
  { value: 'en_cours', label: 'En cours' },
  { value: 'transmis', label: 'Transmis' },
  { value: 'clos', label: 'Clos' },
]

export default function CasesPage() {
  const [statut, setStatut] = useState<'' | CaseStatus>('')
  const [search, setSearch] = useState('')
  const [applied, setApplied] = useState({ statut: '' as '' | CaseStatus, search: '' })

  const { data, loading, error } = useAsync<CaseRecord[]>(
    () =>
      listCases({
        statut: applied.statut === '' ? undefined : applied.statut,
        search: applied.search === '' ? undefined : applied.search,
      }),
    [applied.statut, applied.search],
  )

  const cases = data ?? []

  return (
    <>
      <PageHeader
        title="Dossiers"
        subtitle={`${cases.length} dossier${cases.length > 1 ? 's' : ''}`}
        action={
          <Link href="/cases/new">
            <Button variant="primary">Nouveau dossier</Button>
          </Link>
        }
      />

      <div className="flex flex-col gap-6 p-8">
        <Panel>
          <form
            className="flex flex-wrap items-end gap-3"
            onSubmit={(e) => {
              e.preventDefault()
              setApplied({ statut, search })
            }}
          >
            <div className="flex flex-col gap-1.5">
              <span className="text-xs font-medium text-[var(--color-muted)]">
                Statut
              </span>
              <Select
                value={statut}
                onChange={(e) => setStatut(e.target.value as '' | CaseStatus)}
              >
                {STATUSES.map((s) => (
                  <option key={s.value} value={s.value}>
                    {s.label}
                  </option>
                ))}
              </Select>
            </div>

            <div className="flex flex-1 flex-col gap-1.5">
              <span className="text-xs font-medium text-[var(--color-muted)]">
                Recherche
              </span>
              <Input
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="Référence, titre ou description"
              />
            </div>

            <Button type="submit">Filtrer</Button>
          </form>
        </Panel>

        {error && <ErrorBox message={error} />}
        {loading && (
          <p className="text-sm text-[var(--color-muted)]">Chargement…</p>
        )}

        {!loading && !error && (
          <Panel>
            {cases.length === 0 ? (
              <EmptyState
                title="Aucun dossier ne correspond"
                hint={
                  applied.search || applied.statut
                    ? 'Essayez d’élargir les critères de recherche.'
                    : 'Créez un premier dossier pour commencer.'
                }
              />
            ) : (
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-[var(--color-edge)] text-left text-xs uppercase tracking-wider text-[var(--color-muted)]">
                    <th className="pb-2 font-medium">Référence</th>
                    <th className="pb-2 font-medium">Titre</th>
                    <th className="pb-2 font-medium">Statut</th>
                    <th className="pb-2 font-medium">Priorité</th>
                    <th className="pb-2 text-right font-medium">Créé le</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-[var(--color-edge)]">
                  {cases.map((c) => (
                    <tr
                      key={c.id}
                      className="transition-colors hover:bg-white/[0.03]"
                    >
                      <td className="py-2.5">
                        <Link
                          href={`/cases/detail?id=${c.id}`}
                          className="font-mono text-xs text-[var(--color-accent)] hover:underline"
                        >
                          {c.reference}
                        </Link>
                      </td>
                      <td className="max-w-xs truncate py-2.5">{c.titre}</td>
                      <td className="py-2.5">
                        <Badge value={c.statut} />
                      </td>
                      <td className="py-2.5">
                        <Badge value={c.priorite} />
                      </td>
                      <td className="py-2.5 text-right text-xs text-[var(--color-muted)]">
                        {formatDate(c.dateCreation)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </Panel>
        )}
      </div>
    </>
  )
}
