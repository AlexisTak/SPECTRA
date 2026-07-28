'use client'

/**
 * Rapports d'enquête.
 *
 * Réserve : l'export (PDF, DOCX, HTML) n'est pas implémenté. Un rapport est
 * pour l'instant un enregistrement structuré, pas un document exportable avec
 * annexes de preuves hachées — ce que le cahier des charges attend à terme.
 */

import Link from 'next/link'
import { useState } from 'react'
import {
  createReport,
  deleteReport,
  listCases,
  listReports,
  type CaseRecord,
  type ReportRecord,
} from '@/lib/api'
import { useAsync } from '@/lib/hooks/useCases'
import { PageHeader } from '@/components/layout/shell'
import {
  Badge,
  Button,
  EmptyState,
  ErrorBox,
  Field,
  Input,
  Panel,
  Select,
  formatDate,
} from '@/components/ui/primitives'

export default function ReportsPage() {
  const casesQuery = useAsync<CaseRecord[]>(() => listCases())
  const reportsQuery = useAsync<ReportRecord[]>(() => listReports())

  const [creating, setCreating] = useState(false)
  const [caseId, setCaseId] = useState('')
  const [titre, setTitre] = useState('')
  const [actionError, setActionError] = useState<string | null>(null)

  const cases = casesQuery.data ?? []
  const reports = reportsQuery.data ?? []
  const caseById = new Map(cases.map((c) => [c.id, c]))

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (caseId === '' || titre.trim() === '') return
    setActionError(null)
    try {
      await createReport({ caseId, titre: titre.trim() })
      setTitre('')
      setCreating(false)
      reportsQuery.reload()
    } catch (e) {
      setActionError(e instanceof Error ? e.message : String(e))
    }
  }

  const remove = async (id: string) => {
    setActionError(null)
    try {
      await deleteReport(id)
      reportsQuery.reload()
    } catch (e) {
      setActionError(e instanceof Error ? e.message : String(e))
    }
  }

  return (
    <>
      <PageHeader
        title="Rapports"
        subtitle={`${reports.length} rapport${reports.length > 1 ? 's' : ''}`}
        action={
          <Button
            variant="primary"
            onClick={() => setCreating((v) => !v)}
            disabled={cases.length === 0}
          >
            {creating ? 'Fermer' : 'Nouveau rapport'}
          </Button>
        }
      />

      <div className="flex flex-col gap-6 p-8">
        {(casesQuery.error || reportsQuery.error) && (
          <ErrorBox message={casesQuery.error ?? reportsQuery.error ?? ''} />
        )}
        {actionError && <ErrorBox message={actionError} />}

        <div className="rounded border border-amber-500/30 bg-amber-500/5 p-3">
          <p className="text-xs leading-relaxed text-amber-200/90">
            <strong>Limite connue.</strong> L’export en PDF, DOCX ou HTML avec
            annexes de preuves hachées n’est pas implémenté. Un rapport est pour
            l’instant un enregistrement structuré, pas un document livrable.
          </p>
        </div>

        {creating && (
          <Panel title="Nouveau rapport">
            <form onSubmit={submit} className="flex flex-wrap items-end gap-3">
              <Field label="Dossier">
                <Select value={caseId} onChange={(e) => setCaseId(e.target.value)}>
                  <option value="">Choisir…</option>
                  {cases.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.reference} — {c.titre}
                    </option>
                  ))}
                </Select>
              </Field>
              <div className="flex flex-1 flex-col gap-1.5">
                <span className="text-xs font-medium text-[var(--color-muted)]">
                  Titre
                </span>
                <Input value={titre} onChange={(e) => setTitre(e.target.value)} />
              </div>
              <Button
                type="submit"
                variant="primary"
                disabled={caseId === '' || titre.trim() === ''}
              >
                Créer
              </Button>
            </form>
          </Panel>
        )}

        <Panel>
          {reportsQuery.loading && (
            <p className="text-sm text-[var(--color-muted)]">Chargement…</p>
          )}

          {!reportsQuery.loading && reports.length === 0 && (
            <EmptyState
              title="Aucun rapport"
              hint={
                cases.length === 0
                  ? 'Créez d’abord un dossier.'
                  : 'Un rapport est rattaché à un dossier.'
              }
            />
          )}

          {reports.length > 0 && (
            <ul className="flex flex-col divide-y divide-[var(--color-edge)]">
              {reports.map((r) => {
                const parent = caseById.get(r.caseId)
                return (
                  <li key={r.id} className="flex items-center justify-between gap-4 py-3">
                    <div className="min-w-0">
                      <div className="flex items-center gap-3">
                        <code className="shrink-0 text-xs text-[var(--color-muted)]">
                          {r.reference}
                        </code>
                        <span className="truncate text-sm">{r.titre}</span>
                      </div>
                      {parent && (
                        <Link
                          href={`/cases/detail?id=${parent.id}`}
                          className="text-xs text-[var(--color-muted)] hover:text-[var(--color-accent)]"
                        >
                          {parent.reference} — {parent.titre}
                        </Link>
                      )}
                    </div>
                    <div className="flex shrink-0 items-center gap-3">
                      <Badge value={r.statut} />
                      <span className="text-xs text-[var(--color-muted)]">
                        {formatDate(r.dateCreation)}
                      </span>
                      <Button variant="danger" onClick={() => void remove(r.id)}>
                        Supprimer
                      </Button>
                    </div>
                  </li>
                )
              })}
            </ul>
          )}
        </Panel>
      </div>
    </>
  )
}
