'use client'

/**
 * Écran d'accueil — vérification de bout en bout.
 *
 * Cette page n'est pas décorative : elle sert de preuve que la chaîne
 * frontend → IPC Tauri → Rust → SQLite fonctionne réellement. Tant qu'aucune
 * interface d'enquête n'existe, c'est le seul moyen de constater que le
 * backend répond — et surtout de voir *quelle* erreur remonte quand il ne
 * répond pas.
 *
 * Principe hérité de l'audit (`audit.md`, P3-10) : **aucun repli silencieux**.
 * En cas d'échec, on affiche l'erreur telle quelle. Jamais de valeur par défaut
 * rassurante qui masquerait un backend muet.
 */

import { useCallback, useEffect, useState } from 'react'
import { tauriInvoke } from '@/lib/tauri-bridge'

type Probe =
  | { state: 'pending' }
  | { state: 'ok'; caseCount: number; stats: unknown }
  | { state: 'error'; message: string }

export default function HomePage() {
  const [probe, setProbe] = useState<Probe>({ state: 'pending' })

  const runProbe = useCallback(async () => {
    setProbe({ state: 'pending' })
    try {
      // `get_cases` et `get_case_stats` traversent toute la pile : commande
      // Tauri, verrou sur la connexion, requête SQLite, sérialisation.
      const cases = await tauriInvoke<unknown[]>('get_cases', {
        statut: null,
        search: null,
      })
      const stats = await tauriInvoke<unknown>('get_case_stats')
      setProbe({ state: 'ok', caseCount: cases.length, stats })
    } catch (error) {
      setProbe({
        state: 'error',
        message: error instanceof Error ? error.message : String(error),
      })
    }
  }, [])

  useEffect(() => {
    void runProbe()
  }, [runProbe])

  return (
    <main className="mx-auto flex min-h-screen max-w-3xl flex-col gap-8 p-10">
      <header className="flex flex-col gap-1">
        <h1 className="text-2xl font-semibold tracking-tight">Cekarna</h1>
        <p className="text-sm text-[var(--color-muted)]">
          Plateforme d&apos;investigation locale — vérification de démarrage
        </p>
      </header>

      <section className="rounded-lg border border-[var(--color-edge)] bg-[var(--color-panel)] p-6">
        <h2 className="mb-4 text-xs font-medium uppercase tracking-wider text-[var(--color-muted)]">
          Liaison frontend ↔ backend
        </h2>

        {probe.state === 'pending' && (
          <p className="text-sm text-[var(--color-muted)]">Interrogation du backend…</p>
        )}

        {probe.state === 'ok' && (
          <div className="flex flex-col gap-3">
            <p className="text-sm text-[var(--color-ok)]">
              Backend joignable — base SQLite ouverte et interrogée.
            </p>
            <dl className="grid grid-cols-[auto_1fr] gap-x-6 gap-y-1 text-sm">
              <dt className="text-[var(--color-muted)]">Dossiers en base</dt>
              <dd className="tabular-nums">{probe.caseCount}</dd>
              <dt className="text-[var(--color-muted)]">Statistiques</dt>
              <dd className="font-mono text-xs break-all">
                {JSON.stringify(probe.stats)}
              </dd>
            </dl>
          </div>
        )}

        {probe.state === 'error' && (
          <div className="flex flex-col gap-3">
            <p className="text-sm text-[var(--color-danger)]">
              Backend injoignable.
            </p>
            {/* L'erreur brute est affichée volontairement : c'est elle qui
                permet de distinguer « lancé hors Tauri » d'une vraie panne. */}
            <pre className="overflow-x-auto rounded border border-[var(--color-edge)] bg-[var(--color-surface)] p-3 text-xs text-[var(--color-warn)]">
              {probe.message}
            </pre>
            <p className="text-xs text-[var(--color-muted)]">
              Dans un navigateur ordinaire, cet échec est attendu : les commandes
              n&apos;existent que dans la fenêtre Tauri. Lancer{' '}
              <code className="text-[var(--color-accent)]">npm run tauri:dev</code>.
            </p>
          </div>
        )}

        <button
          type="button"
          onClick={() => void runProbe()}
          className="mt-5 rounded border border-[var(--color-edge)] px-3 py-1.5 text-sm transition-colors hover:border-[var(--color-accent)] hover:text-[var(--color-accent)]"
        >
          Relancer la vérification
        </button>
      </section>

      <section className="rounded-lg border border-[var(--color-edge)] bg-[var(--color-panel)] p-6">
        <h2 className="mb-3 text-xs font-medium uppercase tracking-wider text-[var(--color-muted)]">
          État du chantier
        </h2>
        <ul className="flex flex-col gap-1.5 text-sm text-[var(--color-muted)]">
          <li>
            <span className="text-[var(--color-ok)]">✓</span> Journal d&apos;audit
            hash-chaîné, vérifié par recalcul (23 tests)
          </li>
          <li>
            <span className="text-[var(--color-ok)]">✓</span> Layout de graphe en
            Web Worker — 50 000 nœuds, 5,8 s
          </li>
          <li>
            <span className="text-[var(--color-warn)]">•</span> Canvas graphe,
            timeline, rapports : à construire
          </li>
        </ul>
      </section>
    </main>
  )
}
