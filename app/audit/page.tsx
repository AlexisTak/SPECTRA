'use client'

/**
 * Contrôle d'intégrité transversal : vérifie la chaîne d'audit de tous les
 * dossiers en une passe.
 */

import Link from 'next/link'
import { useState } from 'react'
import { listCases, verifyAuditTrail, type CaseRecord } from '@/lib/api'
import { useAsync } from '@/lib/hooks/useCases'
import { PageHeader } from '@/components/layout/shell'
import {
  Button,
  EmptyState,
  ErrorBox,
  Panel,
} from '@/components/ui/primitives'

interface Verdict {
  caseId: string
  reference: string
  titre: string
  ok: boolean
  total: number
  reason: string | null
  error: string | null
}

export default function AuditPage() {
  const { data, loading, error } = useAsync<CaseRecord[]>(() => listCases())
  const [verdicts, setVerdicts] = useState<Verdict[] | null>(null)
  const [running, setRunning] = useState(false)

  const cases = data ?? []

  const runAll = async () => {
    setRunning(true)
    const out: Verdict[] = []
    for (const c of cases) {
      try {
        const r = await verifyAuditTrail(c.id)
        out.push({
          caseId: c.id,
          reference: c.reference,
          titre: c.titre,
          ok: r.ok,
          total: r.total,
          reason: r.reason,
          error: null,
        })
      } catch (e) {
        out.push({
          caseId: c.id,
          reference: c.reference,
          titre: c.titre,
          ok: false,
          total: 0,
          reason: null,
          error: e instanceof Error ? e.message : String(e),
        })
      }
    }
    setVerdicts(out)
    setRunning(false)
  }

  const broken = verdicts?.filter((v) => !v.ok) ?? []

  return (
    <>
      <PageHeader
        title="Intégrité"
        subtitle="Vérification des chaînes d'audit"
        action={
          <Button
            variant="primary"
            onClick={() => void runAll()}
            disabled={running || cases.length === 0}
          >
            {running ? 'Vérification…' : 'Tout vérifier'}
          </Button>
        }
      />

      <div className="flex flex-col gap-6 p-8">
        {error && <ErrorBox message={error} />}
        {loading && (
          <p className="text-sm text-[var(--color-muted)]">Chargement…</p>
        )}

        <Panel title="Principe">
          <p className="text-sm leading-relaxed text-[var(--color-muted)]">
            Chaque opération sur un dossier — création, modification,
            suppression — inscrit un événement dont le hachage inclut celui de
            l&apos;événement précédent. La vérification{' '}
            <strong className="text-[var(--color-ink)]">recalcule</strong> la
            chaîne complète depuis le contenu des événements. Constater
            qu&apos;un champ est renseigné ne serait pas une vérification.
          </p>
        </Panel>

        {!loading && cases.length === 0 && (
          <Panel>
            <EmptyState
              title="Aucun dossier à vérifier"
              action={
                <Link href="/cases/new" className="mt-2">
                  <Button variant="primary">Créer un dossier</Button>
                </Link>
              }
            />
          </Panel>
        )}

        {verdicts && (
          <Panel
            title={
              broken.length === 0
                ? `${verdicts.length} dossier${verdicts.length > 1 ? 's' : ''} vérifié${verdicts.length > 1 ? 's' : ''}`
                : `${broken.length} anomalie${broken.length > 1 ? 's' : ''}`
            }
          >
            <ul className="flex flex-col divide-y divide-[var(--color-edge)]">
              {verdicts.map((v) => (
                <li key={v.caseId} className="flex flex-col gap-1 py-3">
                  <div className="flex items-center justify-between gap-4">
                    <Link
                      href={`/cases/detail?id=${v.caseId}`}
                      className="flex min-w-0 items-center gap-3 hover:text-[var(--color-accent)]"
                    >
                      <code className="shrink-0 text-xs text-[var(--color-muted)]">
                        {v.reference}
                      </code>
                      <span className="truncate text-sm">{v.titre}</span>
                    </Link>
                    <span
                      className={
                        v.ok
                          ? 'shrink-0 text-xs text-emerald-400'
                          : 'shrink-0 text-xs text-rose-400'
                      }
                    >
                      {v.ok
                        ? `intacte — ${v.total} événement${v.total > 1 ? 's' : ''}`
                        : 'rompue'}
                    </span>
                  </div>
                  {(v.reason || v.error) && (
                    <pre className="overflow-x-auto text-xs text-rose-200/90">
                      {v.error ?? v.reason}
                    </pre>
                  )}
                </li>
              ))}
            </ul>
          </Panel>
        )}
      </div>
    </>
  )
}
