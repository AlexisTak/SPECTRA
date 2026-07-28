'use client'

/**
 * Détail d'un dossier : sujets, preuves, journal, intégrité.
 *
 * L'identifiant passe par la chaîne de requête plutôt que par une route
 * dynamique `[id]` : avec `output: 'export'`, Next.js exige que toutes les
 * routes dynamiques soient énumérables au build via `generateStaticParams`.
 * Or les identifiants de dossier n'existent qu'à l'exécution.
 */

import Link from 'next/link'
import { useSearchParams } from 'next/navigation'
import { Suspense, useState } from 'react'
import {
  addNote,
  createSubject,
  deleteSubject,
  getCase,
  listEvents,
  listSubjects,
  verifyAuditTrail,
  type CaseRecord,
  type CaseEventRecord,
  type SubjectRecord,
  type SubjectStatus,
} from '@/lib/api'
import { useAsync } from '@/lib/hooks/useCases'
import { EvidencePanel } from '@/components/evidence/evidence-panel'
import { SnapshotsPanel } from '@/components/snapshots/snapshots-panel'
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
  Textarea,
  formatDate,
  formatDateTime,
} from '@/components/ui/primitives'

export default function CaseDetailPage() {
  return (
    <Suspense
      fallback={<p className="p-8 text-sm text-[var(--color-muted)]">Chargement…</p>}
    >
      <CaseDetail />
    </Suspense>
  )
}

function CaseDetail() {
  const params = useSearchParams()
  const id = params.get('id') ?? ''

  const caseQuery = useAsync<CaseRecord | null>(
    () => (id ? getCase(id) : Promise.resolve(null)),
    [id],
  )

  if (!id) {
    return (
      <div className="p-8">
        <ErrorBox message="Aucun identifiant de dossier fourni." />
      </div>
    )
  }

  if (caseQuery.error) {
    return (
      <div className="p-8">
        <ErrorBox message={caseQuery.error} />
      </div>
    )
  }

  if (caseQuery.loading) {
    return <p className="p-8 text-sm text-[var(--color-muted)]">Chargement…</p>
  }

  const record = caseQuery.data
  if (!record) {
    return (
      <div className="p-8">
        <EmptyState
          title="Dossier introuvable"
          hint="Il a peut-être été supprimé."
          action={
            <Link href="/cases" className="mt-2">
              <Button>Retour aux dossiers</Button>
            </Link>
          }
        />
      </div>
    )
  }

  return (
    <>
      <PageHeader
        title={record.titre}
        subtitle={`${record.reference} — créé le ${formatDate(record.dateCreation)}`}
        action={
          <div className="flex items-center gap-2">
            <Badge value={record.statut} />
            <Badge value={record.priorite} />
          </div>
        }
      />

      <div className="grid grid-cols-1 gap-6 p-8 xl:grid-cols-3">
        <div className="flex flex-col gap-6 xl:col-span-2">
          {record.description && (
            <Panel title="Description">
              <p className="whitespace-pre-wrap text-sm leading-relaxed">
                {record.description}
              </p>
            </Panel>
          )}

          <SubjectsPanel caseId={id} />
          <EvidencePanel caseId={id} />
          <JournalPanel caseId={id} />
        </div>

        <div className="flex flex-col gap-6">
          <IntegrityPanel caseId={id} />
          <SnapshotsPanel caseId={id} />

          <Panel title="Métadonnées">
            <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
              <dt className="text-[var(--color-muted)]">Référence</dt>
              <dd className="font-mono text-xs">{record.reference}</dd>
              <dt className="text-[var(--color-muted)]">Catégorie</dt>
              <dd>{record.categorie ?? '—'}</dd>
              <dt className="text-[var(--color-muted)]">Créé le</dt>
              <dd>{formatDate(record.dateCreation)}</dd>
              <dt className="text-[var(--color-muted)]">Modifié le</dt>
              <dd>{formatDate(record.dateMiseJour)}</dd>
            </dl>
          </Panel>
        </div>
      </div>
    </>
  )
}

// ---------------------------------------------------------------------------

function SubjectsPanel({ caseId }: { caseId: string }) {
  const { data, loading, error, reload } = useAsync<SubjectRecord[]>(
    () => listSubjects(caseId),
    [caseId],
  )
  const [adding, setAdding] = useState(false)
  const [nom, setNom] = useState('')
  const [prenom, setPrenom] = useState('')
  const [statut, setStatut] = useState<SubjectStatus>('suspect')
  const [submitError, setSubmitError] = useState<string | null>(null)

  const subjects = data ?? []

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    setSubmitError(null)
    try {
      await createSubject({
        caseId,
        nom: nom.trim() || null,
        prenom: prenom.trim() || null,
        statut,
      })
      setNom('')
      setPrenom('')
      setAdding(false)
      reload()
    } catch (e) {
      setSubmitError(e instanceof Error ? e.message : String(e))
    }
  }

  const remove = async (subjectId: string) => {
    try {
      await deleteSubject(subjectId, caseId)
      reload()
    } catch (e) {
      setSubmitError(e instanceof Error ? e.message : String(e))
    }
  }

  return (
    <Panel
      title={`Sujets (${subjects.length})`}
      action={
        <Button onClick={() => setAdding((v) => !v)}>
          {adding ? 'Fermer' : 'Ajouter'}
        </Button>
      }
    >
      {error && <ErrorBox message={error} />}
      {submitError && <ErrorBox message={submitError} />}

      {adding && (
        <form
          onSubmit={submit}
          className="mb-4 flex flex-wrap items-end gap-3 rounded border border-[var(--color-edge)] p-4"
        >
          <Field label="Nom">
            <Input value={nom} onChange={(e) => setNom(e.target.value)} />
          </Field>
          <Field label="Prénom">
            <Input value={prenom} onChange={(e) => setPrenom(e.target.value)} />
          </Field>
          <Field label="Statut">
            <Select
              value={statut}
              onChange={(e) => setStatut(e.target.value as SubjectStatus)}
            >
              <option value="suspect">Suspect</option>
              <option value="victime">Victime</option>
              <option value="temoin">Témoin</option>
              <option value="inconnu">Inconnu</option>
            </Select>
          </Field>
          <Button type="submit" variant="primary">
            Enregistrer
          </Button>
        </form>
      )}

      {loading && <p className="text-sm text-[var(--color-muted)]">Chargement…</p>}

      {!loading && subjects.length === 0 && (
        <EmptyState title="Aucun sujet enregistré" />
      )}

      {subjects.length > 0 && (
        <ul className="flex flex-col divide-y divide-[var(--color-edge)]">
          {subjects.map((s) => (
            <li
              key={s.id}
              className="flex items-center justify-between gap-4 py-2.5"
            >
              <span className="flex items-center gap-3">
                <Badge value={s.statut} />
                <span className="text-sm">
                  {[s.prenom, s.nom].filter(Boolean).join(' ') || 'Sans nom'}
                </span>
              </span>
              <Button variant="danger" onClick={() => void remove(s.id)}>
                Supprimer
              </Button>
            </li>
          ))}
        </ul>
      )}
    </Panel>
  )
}

// ---------------------------------------------------------------------------

function JournalPanel({ caseId }: { caseId: string }) {
  const { data, loading, error, reload } = useAsync<CaseEventRecord[]>(
    () => listEvents(caseId),
    [caseId],
  )
  const [note, setNote] = useState('')
  const [submitError, setSubmitError] = useState<string | null>(null)
  const events = data ?? []

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (note.trim() === '') return
    setSubmitError(null)
    try {
      await addNote(caseId, note.trim())
      setNote('')
      reload()
    } catch (e) {
      setSubmitError(e instanceof Error ? e.message : String(e))
    }
  }

  return (
    <Panel title={`Journal (${events.length})`}>
      {error && <ErrorBox message={error} />}
      {submitError && <ErrorBox message={submitError} />}

      <form onSubmit={submit} className="mb-4 flex flex-col gap-2">
        <Textarea
          value={note}
          onChange={(e) => setNote(e.target.value)}
          rows={2}
          placeholder="Ajouter une note au dossier…"
        />
        <div>
          <Button type="submit" disabled={note.trim() === ''}>
            Ajouter la note
          </Button>
        </div>
      </form>

      {loading && <p className="text-sm text-[var(--color-muted)]">Chargement…</p>}

      {!loading && events.length === 0 && (
        <EmptyState title="Aucun événement" />
      )}

      {events.length > 0 && (
        <ul className="flex flex-col divide-y divide-[var(--color-edge)]">
          {events.map((e) => (
            <li key={e.id} className="flex flex-col gap-0.5 py-2.5">
              <span className="flex items-baseline justify-between gap-3">
                <span className="text-sm">{e.titre ?? e.type}</span>
                <span className="shrink-0 text-xs text-[var(--color-muted)]">
                  {formatDateTime(e.timestamp)}
                </span>
              </span>
              {e.description && (
                <p className="whitespace-pre-wrap text-sm text-[var(--color-muted)]">
                  {e.description}
                </p>
              )}
            </li>
          ))}
        </ul>
      )}
    </Panel>
  )
}

// ---------------------------------------------------------------------------

/**
 * Vérification de la chaîne d'audit.
 *
 * Le backend recalcule chaque maillon plutôt que de constater qu'un champ est
 * non vide. Une chaîne rompue est signalée avec l'index et la raison — jamais
 * masquée derrière un statut « OK » par défaut.
 */
function IntegrityPanel({ caseId }: { caseId: string }) {
  const [state, setState] = useState<
    | { kind: 'idle' }
    | { kind: 'running' }
    | { kind: 'done'; ok: boolean; total: number; reason: string | null }
    | { kind: 'failed'; message: string }
  >({ kind: 'idle' })

  const run = async () => {
    setState({ kind: 'running' })
    try {
      const r = await verifyAuditTrail(caseId)
      setState({
        kind: 'done',
        ok: r.ok,
        total: r.total,
        reason: r.reason,
      })
    } catch (e) {
      setState({
        kind: 'failed',
        message: e instanceof Error ? e.message : String(e),
      })
    }
  }

  return (
    <Panel title="Intégrité du journal">
      <p className="mb-4 text-xs leading-relaxed text-[var(--color-muted)]">
        Chaque événement est chaîné au précédent par un hachage SHA-256. La
        vérification recalcule la chaîne entière : toute modification d&apos;un
        événement, ou la suppression d&apos;un maillon, est détectée.
      </p>

      {state.kind === 'failed' && <ErrorBox message={state.message} />}

      {state.kind === 'done' && (
        <div
          className={
            state.ok
              ? 'mb-4 rounded border border-emerald-500/40 bg-emerald-500/10 p-3'
              : 'mb-4 rounded border border-rose-500/40 bg-rose-500/10 p-3'
          }
        >
          <p
            className={
              state.ok
                ? 'text-sm font-medium text-emerald-300'
                : 'text-sm font-medium text-rose-300'
            }
          >
            {state.ok ? 'Chaîne intacte' : 'Chaîne rompue'}
          </p>
          <p className="mt-1 text-xs text-[var(--color-muted)]">
            {state.total} événement{state.total > 1 ? 's' : ''} vérifié
            {state.total > 1 ? 's' : ''}
          </p>
          {state.reason && (
            <pre className="mt-2 overflow-x-auto text-xs text-rose-200/90">
              {state.reason}
            </pre>
          )}
        </div>
      )}

      <Button onClick={() => void run()} disabled={state.kind === 'running'}>
        {state.kind === 'running' ? 'Vérification…' : 'Vérifier la chaîne'}
      </Button>
    </Panel>
  )
}
