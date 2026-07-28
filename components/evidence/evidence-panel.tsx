'use client'

/**
 * Preuves d'un dossier : ingestion, vérification, qualification.
 *
 * L'ingestion passe un **chemin de fichier** au backend, qui lit le contenu et
 * calcule l'empreinte lui-même. Le frontend ne calcule ni ne transmet aucun
 * hachage : une empreinte que le système n'a pas observée n'atteste rien
 * (`audit.md`, P1-2).
 */

import { open } from '@tauri-apps/plugin-dialog'
import { useState } from 'react'
import {
  addEvidence,
  deleteEvidence,
  listEvidence,
  setClaim,
  verifyCaseEvidence,
  type CaseIntegrityReport,
  type EvidenceRecord,
  type Qualification,
} from '@/lib/api'
import { useAsync } from '@/lib/hooks/useCases'
import {
  Badge,
  Button,
  EmptyState,
  ErrorBox,
  Panel,
  Select,
  formatDateTime,
} from '@/components/ui/primitives'

const QUALIFICATIONS: Array<{ value: Qualification; label: string }> = [
  { value: 'preuve', label: 'Preuve' },
  { value: 'indice', label: 'Indice' },
  { value: 'hypothese', label: 'Hypothèse' },
  { value: 'non_verifie', label: 'Non vérifié' },
]

export function EvidencePanel({ caseId }: { caseId: string }) {
  const { data, loading, error, reload } = useAsync<EvidenceRecord[]>(
    () => listEvidence(caseId),
    [caseId],
  )
  const [busy, setBusy] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)
  const [report, setReport] = useState<CaseIntegrityReport | null>(null)

  const evidence = data ?? []

  const ingest = async () => {
    setActionError(null)
    try {
      const selected = await open({
        multiple: false,
        title: 'Sélectionner un fichier à verser au dossier',
      })
      if (typeof selected !== 'string') return

      setBusy(true)
      await addEvidence(caseId, selected, { type: 'file' })
      setReport(null)
      reload()
    } catch (e) {
      setActionError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  const verifyAll = async () => {
    setActionError(null)
    setBusy(true)
    try {
      setReport(await verifyCaseEvidence(caseId))
      reload()
    } catch (e) {
      setActionError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  const remove = async (id: string) => {
    setActionError(null)
    try {
      await deleteEvidence(id, caseId)
      setReport(null)
      reload()
    } catch (e) {
      setActionError(e instanceof Error ? e.message : String(e))
    }
  }

  const qualify = async (evidenceId: string, qualification: Qualification) => {
    setActionError(null)
    try {
      await setClaim({
        caseId,
        refKind: 'evidence',
        refId: evidenceId,
        qualification,
        // Fiabilité neutre par défaut : elle doit être posée par l'analyste,
        // pas devinée par l'outil.
        fiabilite: 3,
        source: 'analyste',
      })
      reload()
    } catch (e) {
      setActionError(e instanceof Error ? e.message : String(e))
    }
  }

  const statusFor = (id: string) =>
    report?.checks.find((c) => c.evidenceId === id)

  return (
    <Panel
      title={`Preuves (${evidence.length})`}
      action={
        <div className="flex gap-2">
          {evidence.length > 0 && (
            <Button onClick={() => void verifyAll()} disabled={busy}>
              {busy ? 'Vérification…' : 'Vérifier l’intégrité'}
            </Button>
          )}
          <Button variant="primary" onClick={() => void ingest()} disabled={busy}>
            Verser un fichier
          </Button>
        </div>
      }
    >
      {error && <ErrorBox message={error} />}
      {actionError && <ErrorBox message={actionError} />}

      {report && (
        <div
          className={
            report.altered + report.missing === 0
              ? 'mb-4 rounded border border-emerald-500/40 bg-emerald-500/10 p-3'
              : 'mb-4 rounded border border-rose-500/40 bg-rose-500/10 p-3'
          }
        >
          <p
            className={
              report.altered + report.missing === 0
                ? 'text-sm font-medium text-emerald-300'
                : 'text-sm font-medium text-rose-300'
            }
          >
            {report.altered + report.missing === 0
              ? 'Toutes les preuves sont intactes'
              : `${report.altered} altérée(s), ${report.missing} manquante(s)`}
          </p>
          <p className="mt-1 text-xs text-[var(--color-muted)]">
            {report.total} preuve(s) contrôlée(s) par recalcul d’empreinte —{' '}
            {formatDateTime(report.generatedAt)}
          </p>
        </div>
      )}

      {loading && <p className="text-sm text-[var(--color-muted)]">Chargement…</p>}

      {!loading && evidence.length === 0 && (
        <EmptyState
          title="Aucune preuve"
          hint="Le fichier est copié dans un magasin local et son empreinte SHA-256 calculée à l’ingestion."
        />
      )}

      {evidence.length > 0 && (
        <ul className="flex flex-col divide-y divide-[var(--color-edge)]">
          {evidence.map((e) => {
            const check = statusFor(e.id)
            return (
              <li key={e.id} className="flex flex-col gap-1.5 py-3">
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <p className="truncate text-sm">{e.nom ?? 'Sans nom'}</p>
                    <p className="text-xs text-[var(--color-muted)]">
                      {formatDateTime(e.dateAjout)}
                      {e.taille !== null && ` — ${formatBytes(e.taille)}`}
                    </p>
                  </div>
                  <div className="flex shrink-0 items-center gap-2">
                    {check && <IntegrityBadge status={check.status} />}
                    <Select
                      defaultValue=""
                      onChange={(ev) => {
                        const v = ev.target.value as Qualification | ''
                        if (v) void qualify(e.id, v)
                      }}
                      className="text-xs"
                    >
                      <option value="">Qualifier…</option>
                      {QUALIFICATIONS.map((q) => (
                        <option key={q.value} value={q.value}>
                          {q.label}
                        </option>
                      ))}
                    </Select>
                    <Button variant="danger" onClick={() => void remove(e.id)}>
                      Supprimer
                    </Button>
                  </div>
                </div>

                {e.hashSha256 && (
                  <code className="truncate text-xs text-[var(--color-muted)]">
                    SHA-256 {e.hashSha256}
                  </code>
                )}

                {check && check.status !== 'intact' && (
                  <p className="text-xs text-rose-300">{check.message}</p>
                )}
              </li>
            )
          })}
        </ul>
      )}
    </Panel>
  )
}

function IntegrityBadge({ status }: { status: string }) {
  const tone =
    status === 'intact'
      ? 'border-emerald-500/40 bg-emerald-500/10 text-emerald-300'
      : status === 'altered' || status === 'missing'
        ? 'border-rose-500/40 bg-rose-500/10 text-rose-300'
        : 'border-amber-500/40 bg-amber-500/10 text-amber-300'

  const label =
    status === 'intact'
      ? 'Intacte'
      : status === 'altered'
        ? 'Altérée'
        : status === 'missing'
          ? 'Manquante'
          : 'Non vérifiable'

  return (
    <span
      className={`inline-flex items-center rounded border px-2 py-0.5 text-xs font-medium ${tone}`}
    >
      {label}
    </span>
  )
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} o`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} Kio`
  if (n < 1024 * 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} Mio`
  return `${(n / 1024 / 1024 / 1024).toFixed(2)} Gio`
}

export { Badge }
