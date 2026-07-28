'use client'

/**
 * Instantanés d'un dossier.
 *
 * Réserve importante, affichée à l'analyste : dans l'implémentation actuelle,
 * l'instantané ne capture que ses propres métadonnées, **pas l'état du
 * dossier** (`audit.md`, P1-7). Il ne permet donc ni comparaison ni
 * restauration. Le panneau le dit plutôt que de laisser croire à une garantie
 * qui n'existe pas.
 */

import { useState } from 'react'
import {
  deleteSnapshot,
  listSnapshots,
  takeSnapshot,
  type SnapshotRecord,
} from '@/lib/api'
import { useAsync } from '@/lib/hooks/useCases'
import {
  Button,
  EmptyState,
  ErrorBox,
  Input,
  Panel,
  formatDateTime,
} from '@/components/ui/primitives'

export function SnapshotsPanel({ caseId }: { caseId: string }) {
  const { data, loading, error, reload } = useAsync<SnapshotRecord[]>(
    () => listSnapshots(caseId),
    [caseId],
  )
  const [nom, setNom] = useState('')
  const [busy, setBusy] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)

  const snapshots = data ?? []

  const create = async (e: React.FormEvent) => {
    e.preventDefault()
    if (nom.trim() === '') return
    setBusy(true)
    setActionError(null)
    try {
      await takeSnapshot({ caseId, nom: nom.trim() })
      setNom('')
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
      await deleteSnapshot(id)
      reload()
    } catch (e) {
      setActionError(e instanceof Error ? e.message : String(e))
    }
  }

  return (
    <Panel title={`Instantanés (${snapshots.length})`}>
      <div className="mb-4 rounded border border-amber-500/30 bg-amber-500/5 p-3">
        <p className="text-xs leading-relaxed text-amber-200/90">
          <strong>Limite connue.</strong> L’instantané enregistre pour l’instant
          ses seules métadonnées, pas le contenu du dossier. Il ne permet donc ni
          comparaison ni restauration. À compléter avant tout usage probatoire.
        </p>
      </div>

      {error && <ErrorBox message={error} />}
      {actionError && <ErrorBox message={actionError} />}

      <form onSubmit={create} className="mb-4 flex items-end gap-2">
        <Input
          value={nom}
          onChange={(e) => setNom(e.target.value)}
          placeholder="Nom de l’instantané"
          className="flex-1"
        />
        <Button type="submit" disabled={busy || nom.trim() === ''}>
          {busy ? 'Création…' : 'Créer'}
        </Button>
      </form>

      {loading && <p className="text-sm text-[var(--color-muted)]">Chargement…</p>}

      {!loading && snapshots.length === 0 && (
        <EmptyState title="Aucun instantané" />
      )}

      {snapshots.length > 0 && (
        <ul className="flex flex-col divide-y divide-[var(--color-edge)]">
          {snapshots.map((s) => (
            <li key={s.id} className="flex flex-col gap-1 py-2.5">
              <div className="flex items-center justify-between gap-3">
                <span className="truncate text-sm">{s.nom}</span>
                <div className="flex shrink-0 items-center gap-3">
                  <span className="text-xs text-[var(--color-muted)]">
                    {formatDateTime(s.createdAt)}
                  </span>
                  <Button variant="danger" onClick={() => void remove(s.id)}>
                    Supprimer
                  </Button>
                </div>
              </div>
              <code className="truncate text-xs text-[var(--color-muted)]">
                {s.hashSha256}
              </code>
            </li>
          ))}
        </ul>
      )}
    </Panel>
  )
}
