'use client'

/**
 * Qualification des elements d'un dossier.
 *
 * Permet de marquer une preuve, un sujet ou un evenement comme :
 * - Preuve : element retenu a charge ou a decharge
 * - Indice : element a corroborer
 * - Hypothese : piste de travail
 * - Non verifie : en attente d'analyse
 *
 * La qualification est elle-meme versee au journal d'audit.
 */

import { useState } from 'react'
import {
  listClaims,
  setClaim,
  deleteClaim,
  listEvidence,
  listSubjects,
  listEvents,
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
} from '@/components/ui/primitives'

const QUALIFICATIONS = [
  { value: 'preuve' as Qualification, label: 'Preuve', tone: 'border-emerald-500/40 bg-emerald-500/10 text-emerald-300' },
  { value: 'indice' as Qualification, label: 'Indice', tone: 'border-amber-500/40 bg-amber-500/10 text-amber-300' },
  { value: 'hypothese' as Qualification, label: 'Hypothese', tone: 'border-violet-500/40 bg-violet-500/10 text-violet-300' },
  { value: 'non_verifie' as Qualification, label: 'Non verifie', tone: 'border-slate-500/40 bg-slate-500/10 text-slate-300' },
]

type Element =
  | { kind: 'evidence'; id: string; nom: string }
  | { kind: 'subject'; id: string; nom: string }
  | { kind: 'event'; id: string; titre: string }

export function ClaimsPanel({ caseId }: { caseId: string }) {
  const claimsQuery = useAsync(() => listClaims(caseId), [caseId])
  const evidenceQuery = useAsync(() => listEvidence(caseId), [caseId])
  const subjectsQuery = useAsync(() => listSubjects(caseId), [caseId])
  const eventsQuery = useAsync(() => listEvents(caseId), [caseId])

  const [selected, setSelected] = useState<Element | null>(null)
  const [qualification, setQualification] = useState<Qualification>('indice')
  const [busy, setBusy] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)

  const claims = claimsQuery.data ?? []
  const evidence = evidenceQuery.data ?? []
  const subjects = subjectsQuery.data ?? []
  const events = eventsQuery.data ?? []

  // Elements deja qualifies
  const qualifiedIds = new Set(claims.map((c) => c.refKind + ':' + c.refId))

  // Options disponibles (non qualifiees)
  const options: Element[] = [
    ...evidence
      .filter((e) => !qualifiedIds.has('evidence:' + e.id))
      .map((e) => ({ kind: 'evidence' as const, id: e.id, nom: e.nom ?? 'Sans nom' })),
    ...subjects
      .filter((s) => !qualifiedIds.has('subject:' + s.id))
      .map((s) => ({
        kind: 'subject' as const,
        id: s.id,
        nom: [s.prenom, s.nom].filter(Boolean).join(' ') || 'Sans nom',
      })),
    ...events
      .filter((e) => !qualifiedIds.has('event:' + e.id))
      .map((e) => ({ kind: 'event' as const, id: e.id, titre: e.titre ?? e.type })),
  ]

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!selected) return
    setBusy(true)
    setActionError(null)
    try {
      await setClaim({
        caseId,
        refKind: selected.kind,
        refId: selected.id,
        qualification,
        fiabilite: 3,
        source: 'analyste',
      })
      setSelected(null)
      claimsQuery.reload()
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err))
    } finally {
      setBusy(false)
    }
  }

  const remove = async (id: string) => {
    setActionError(null)
    try {
      await deleteClaim(id, caseId)
      claimsQuery.reload()
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err))
    }
  }

  const getTone = (q: Qualification) => {
    const def = QUALIFICATIONS.find((x) => x.value === q)
    return def ? def.tone : QUALIFICATIONS[3].tone
  }

  const getLabel = (q: Qualification) => {
    const def = QUALIFICATIONS.find((x) => x.value === q)
    return def?.label ?? q
  }

  return (
    <Panel
      title={'Qualification (' + claims.length + ')'}
      action={
        <Select
          value={selected ? selected.kind + ':' + selected.id : ''}
          onChange={(e) => {
            const parts = e.target.value.split(':')
            const kind = parts[0]
            const id = parts[1]
            if (kind === 'evidence') {
              const el = evidence.find((x) => x.id === id)
              if (el) setSelected({ kind: 'evidence', id: el.id, nom: el.nom ?? 'Sans nom' })
            } else if (kind === 'subject') {
              const el = subjects.find((x) => x.id === id)
              if (el) setSelected({ kind: 'subject', id: el.id, nom: [el.prenom, el.nom].filter(Boolean).join(' ') || 'Sans nom' })
            } else if (kind === 'event') {
              const el = events.find((x) => x.id === id)
              if (el) setSelected({ kind: 'event', id: el.id, titre: el.titre ?? el.type })
            }
          }}
        >
          <option value="">Qualifier un element...</option>
          {options.map((o) => (
            <option key={o.kind + ':' + o.id} value={o.kind + ':' + o.id}>
              {o.kind === 'evidence' ? 'Preuve' : o.kind === 'subject' ? 'Sujet' : 'Evenement'} - {'nom' in o ? o.nom : o.titre}
            </option>
          ))}
        </Select>
      }
    >
      {claimsQuery.error && <ErrorBox message={claimsQuery.error || 'Erreur'} />}
      {actionError && <ErrorBox message={actionError} />}

      {selected && (
        <form onSubmit={submit} className="mb-4 flex items-center gap-2 rounded border border-[var(--color-edge)] p-3">
          <span className="text-sm">
            {selected.kind === 'evidence' ? 'Preuve' : selected.kind === 'subject' ? 'Sujet' : 'Evenement'} :{' '}
            <span className="font-medium">{'nom' in selected ? selected.nom : selected.titre}</span>
          </span>
          <Select value={qualification} onChange={(e) => setQualification(e.target.value as Qualification)}>
            {QUALIFICATIONS.map((q) => (
              <option key={q.value} value={q.value}>
                {q.label}
              </option>
            ))}
          </Select>
          <Button type="submit" variant="primary" disabled={busy}>
            {busy ? 'Enregistrement...' : 'Qualifier'}
          </Button>
          <Button type="button" onClick={() => setSelected(null)}>
            Annuler
          </Button>
        </form>
      )}

      {claims.length === 0 ? (
        <EmptyState
          title="Aucun element qualifie"
          hint="Selectionnez une preuve, un sujet ou un evenement pour le qualifier."
        />
      ) : (
        <ul className="flex flex-col divide-y divide-[var(--color-edge)]">
          {claims.map((c) => (
            <li key={c.id} className="flex items-center justify-between gap-3 py-2.5">
              <div className="flex min-w-0 items-center gap-3">
                <span className={'inline-flex items-center rounded border px-2 py-0.5 text-xs font-medium ' + getTone(c.qualification)}>
                  {getLabel(c.qualification)}
                </span>
                <span className="truncate text-sm">
                  {c.refKind === 'evidence' ? 'Preuve' : c.refKind === 'subject' ? 'Sujet' : 'Evenement'} - {c.refId.slice(0, 8)}...
                </span>
              </div>
              <div className="flex shrink-0 items-center gap-2">
                <span className="text-xs text-[var(--color-muted)]">
                  Fiabilite : {c.fiabilite}/5
                </span>
                <Button variant="danger" onClick={() => void remove(c.id)}>
                  Supprimer
                </Button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </Panel>
  )
}
