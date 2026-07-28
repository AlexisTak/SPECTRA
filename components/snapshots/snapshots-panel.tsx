'use client'

/**
 * Instantanés d'un dossier.
 *
 * Chaque instantané capture l'état complet : métadonnées, sujets, preuves,
 * événements. Le fichier est stocké dans le magasin de preuves, et son
 * empreinte SHA-256 est calculée sur le contenu sérialisé.
 */

import { useState } from 'react'
import {
  deleteSnapshot,
  listSnapshots,
  takeSnapshot,
  verifySnapshotIntegrity,
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

interface IntegrityStatus {
  status: 'intact' | 'altered' | 'missing'
  message: string
  expectedHash?: string
  actualHash?: string
}

export function SnapshotsPanel({ caseId }: { caseId: string }) {
  const { data, loading, error, reload } = useAsync<SnapshotRecord[]>(
    () => listSnapshots(caseId),
    [caseId],
  )
  const [nom, setNom] = useState('')
  const [busy, setBusy] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)
  const [integrity, setIntegrity] = useState<Record<string, IntegrityStatus>>({})

  const snapshots = data ?? []

  const create = async (e: React.FormEvent) => {
    e.preventDefault()
    if (nom.trim() === '') return
    setBusy(true)
    setActionError(null)
    try {
      await takeSnapshot({ caseId, nom: nom.trim() })
      setNom('')
      setIntegrity({})
      reload()
    } catch (e) {
      setActionError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  const verify = async (id: string) => {
    setBusy(true)
    setActionError(null)
    try {
      const r = await verifySnapshotIntegrity(id)
      setIntegrity((prev) => ({
        ...prev,
        [id]: {
          status: r.status as IntegrityStatus['status'],
          message: r.message,
          expectedHash: r.expected_hash,
          actualHash: r.actual_hash,
        },
      }))
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
      setIntegrity((prev) => {
        const copy = { ...prev }
        delete copy[id]
        return copy
      })
      reload()
    } catch (e) {
      setActionError(e instanceof Error ? e.message : String(e))
    }
  }

  return (
    <Panel title={`Instantanés (${snapshots.length})`}>
      {error && <ErrorBox message={error} />}
      {actionError && <ErrorBox message={actionError} />}

      <form onSubmit={create} className="mb-4 flex items-end gap-2">
        <Input
          value={nom}
          onChange={(e) => setNom(e.target.value)}
          placeholder="Nom de l'instantané"
          className="flex-1"
        />
        <Button type="submit" disabled={busy || nom.trim() === ''}>
          {busy ? 'Création…' : 'Capturer'}
        </Button>
      </form>

      {loading && <p className="text-sm text-[var(--color-muted)]">Chargement…</p>}

      {!loading && snapshots.length === 0 && (
        <EmptyState
          title="Aucun instantané"
          hint="Un instantané capture l'état complet du dossier : métadonnées, sujets, preuves, événements."
        />
      )}

      {snapshots.length > 0 && (
        <ul className="flex flex-col divide-y divide-[var(--color-edge)]">
          {snapshots.map((s) => {
            const check = integrity[s.id]
            return (
              <li key={s.id} className="flex flex-col gap-1.5 py-3">
                <div className="flex items-center justify-between gap-3">
                  <span className="truncate text-sm">{s.nom}</span>
                  <div className="flex shrink-0 items-center gap-2">
                    <Button
                      onClick={() => void verify(s.id)}
                      disabled={busy}
                      className="text-xs"
                    >
                      Vérifier
                    </Button>
                    <Button variant="danger" onClick={() => void remove(s.id)}>
                      Supprimer
                    </Button>
                  </div>
                </div>
                <code className="truncate text-xs text-[var(--color-muted)]">
                  {s.hashSha256}
                </code>
                {check && (
                  <div
                    className={
                      check.status === 'intact'
                        ? 'rounded border border-emerald-500/40 bg-emerald-500/10 p-2'
                        : 'rounded border border-rose-500/40 bg-rose-500/10 p-2'
                    }
                  >
                    <p
                      className={
                        check.status === 'intact'
                          ? 'text-xs font-medium text-emerald-300'
                          : 'text-xs font-medium text-rose-300'
                      }
                    >
                      {check.status === 'intact'
                        ? 'Instantané intact'
                        : check.status === 'altered'
                          ? 'Contenu modifié'
                          : 'Fichier manquant'}
                    </p>
                    <p className="mt-0.5 text-xs text-[var(--color-muted)]">
                      {check.message}
                    </p>
                    {check.expectedHash && check.actualHash && (
                      <div className="mt-1 flex flex-col gap-0.5 text-[10px] text-[var(--color-muted)]">
                        <span>Attendu : {check.expectedHash}</span>
                        <span>Obtenu : {check.actualHash}</span>
                      </div>
                    )}
                  </div>
                )}
              </li>
            )
          })}
        </ul>
      )}
    </Panel>
  )
}
