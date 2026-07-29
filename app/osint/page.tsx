'use client'

/**
 * Interface OSINT — exécution de sondes sur pseudos/emails/téléphones.
 *
 * Permet de lancer une campagne de vérification sur :
 * - Un nom d'utilisateur (WhatsMyName, Sherlock, Maigret)
 * - Un email (Holehe — à implémenter)
 * - Un téléphone (PhoneInfoga — à implémenter)
 *
 * Les résultats apparaissent en temps réel avec :
 * - ✅ Compte trouvé (vert)
 * - ❌ Compte inexistant (gris)
 * - ⚠️ Indéterminé (orange)
 * - 🚫 Bloqué (rouge)
 */

import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { PageHeader } from '@/components/layout/shell'
import {
  Badge,
  Button,
  EmptyState,
  ErrorBox,
  Input,
  Panel,
  Select,
} from '@/components/ui/primitives'

type SelectorKind = 'username' | 'email' | 'phone'
type ProbeOutcome = 'exists' | 'missing' | 'blocked' | 'indeterminate' | 'error'

interface ProbeResult {
  site: string
  outcome: ProbeOutcome
  url?: string
  reason?: string
}

interface RawProbeResult {
  site: string
  outcome: {
    exists?: { url: string | null }
    missing?: null
    blocked?: { reason: string }
    indeterminate?: { reason: string }
    error?: { message: string }
  }
  elapsed_ms: number
}

export default function OsintPage() {
  const [selectorType, setSelectorType] = useState<SelectorKind>('username')
  const [selector, setSelector] = useState('')
  const [running, setRunning] = useState(false)
  const [progress, setProgress] = useState({ current: 0, total: 0 })
  const [results, setResults] = useState<ProbeResult[]>([])
  const [error, setError] = useState<string | null>(null)

  const runCampaign = async () => {
    if (!selector.trim()) return
    setRunning(true)
    setError(null)
    setResults([])
    setProgress({ current: 0, total: 0 })

    try {
      const raw = await invoke<string>('run_osint_campaign', {
        selector,
        kind: selectorType,
      })
      const parsed: RawProbeResult[] = JSON.parse(raw)

      setProgress({ current: parsed.length, total: parsed.length })
      setResults(
        parsed.map((r) => {
          let outcome: ProbeOutcome = 'missing'
          let url: string | undefined
          let reason: string | undefined

          if (r.outcome.exists) {
            outcome = 'exists'
            url = r.outcome.exists.url || undefined
          } else if (r.outcome.blocked) {
            outcome = 'blocked'
            reason = r.outcome.blocked.reason
          } else if (r.outcome.indeterminate) {
            outcome = 'indeterminate'
            reason = r.outcome.indeterminate.reason
          } else if (r.outcome.error) {
            outcome = 'error'
            reason = r.outcome.error.message
          }

          return { site: r.site, outcome, url, reason }
        }),
      )
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setRunning(false)
    }
  }

  const stats = {
    found: results.filter((r) => r.outcome === 'exists').length,
    missing: results.filter((r) => r.outcome === 'missing').length,
    blocked: results.filter((r) => r.outcome === 'blocked').length,
    indeterminate: results.filter((r) => r.outcome === 'indeterminate').length,
  }

  return (
    <>
      <PageHeader
        title="Recherche OSINT"
        subtitle="Vérification de pseudos, emails et téléphones sur +700 sites"
      />

      <div className="flex flex-col gap-6 p-8">
        {/* Formulaire */}
        <Panel>
          <div className="flex flex-wrap items-end gap-4">
            <div className="flex flex-col gap-1.5">
              <span className="text-xs font-medium text-[var(--color-muted)]">
                Type
              </span>
              <Select
                value={selectorType}
                onChange={(e) => setSelectorType(e.target.value as SelectorKind)}
              >
                <option value="username">Nom d'utilisateur</option>
                <option value="email">Email</option>
                <option value="phone">Téléphone</option>
              </Select>
            </div>

            <div className="flex flex-1 flex-col gap-1.5">
              <span className="text-xs font-medium text-[var(--color-muted)]">
                {selectorType === 'username'
                  ? 'Pseudo à rechercher'
                  : selectorType === 'email'
                    ? 'Email à rechercher'
                    : 'Téléphone à rechercher'}
              </span>
              <Input
                value={selector}
                onChange={(e) => setSelector(e.target.value)}
                placeholder={
                  selectorType === 'username'
                    ? 'ex: johndoe'
                    : selectorType === 'email'
                      ? 'ex: john@example.com'
                      : 'ex: +33612345678'
                }
                onKeyDown={(e) => e.key === 'Enter' && !running && runCampaign()}
              />
            </div>

            <Button
              variant="primary"
              onClick={runCampaign}
              disabled={running || !selector.trim()}
            >
              {running ? 'Recherche en cours...' : 'Lancer la recherche'}
            </Button>
          </div>
        </Panel>

        {error && <ErrorBox message={error} />}

        {/* Progression */}
        {running && (
          <Panel>
            <div className="flex items-center gap-4">
              <div className="h-2 flex-1 rounded-full bg-[var(--color-edge)]">
                <div
                  className="h-2 rounded-full bg-[var(--color-accent)] transition-all"
                  style={{
                    width:
                      progress.total > 0
                        ? (progress.current / progress.total) * 100
                        : 0,
                  }}
                />
              </div>
              <span className="text-sm text-[var(--color-muted)]">
                {progress.current} / {progress.total}
              </span>
            </div>
          </Panel>
        )}

        {/* Statistiques */}
        {results.length > 0 && (
          <div className="grid grid-cols-4 gap-4">
            <StatBox label="Comptes trouvés" value={stats.found} tone="emerald" />
            <StatBox label="Inexistants" value={stats.missing} tone="slate" />
            <StatBox label="Bloqués" value={stats.blocked} tone="rose" />
            <StatBox label="Indéterminés" value={stats.indeterminate} tone="amber" />
          </div>
        )}

        {/* Résultats */}
        {results.length > 0 && (
          <Panel title={`Résultats (${results.length})`}>
            <div className="max-h-96 overflow-y-auto">
              <table className="w-full text-sm">
                <thead className="sticky top-0 bg-[var(--color-panel)]">
                  <tr className="border-b border-[var(--color-edge)] text-left text-xs uppercase tracking-wider text-[var(--color-muted)]">
                    <th className="pb-2">Site</th>
                    <th className="pb-2">Statut</th>
                    <th className="pb-2">URL</th>
                    <th className="pb-2 text-right">Action</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-[var(--color-edge)]">
                  {results.map((r, i) => (
                    <tr key={i} className="transition-colors hover:bg-white/5">
                      <td className="py-2">{r.site}</td>
                      <td className="py-2">
                        <OutcomeBadge outcome={r.outcome} />
                      </td>
                      <td className="py-2">
                        {r.url ? (
                          <a
                            href={r.url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-[var(--color-accent)] hover:underline"
                          >
                            {truncate(r.url, 40)}
                          </a>
                        ) : (
                          <span className="text-[var(--color-muted)]">—</span>
                        )}
                      </td>
                      <td className="py-2 text-right">
                        {r.outcome === 'exists' && (
                          <Button variant="primary" className="text-xs">
                            Voir le profil
                          </Button>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </Panel>
        )}

        {results.length === 0 && !running && (
          <EmptyState
            title="Aucun résultat"
            hint="Entrez un pseudo, email ou téléphone pour lancer la recherche."
          />
        )}
      </div>
    </>
  )
}

function OutcomeBadge({ outcome }: { outcome: ProbeOutcome }) {
  const config = {
    exists: { label: 'Trouvé', className: 'border-emerald-500/40 bg-emerald-500/10 text-emerald-300' },
    missing: { label: 'Inexistant', className: 'border-slate-500/40 bg-slate-500/10 text-slate-300' },
    blocked: { label: 'Bloqué', className: 'border-rose-500/40 bg-rose-500/10 text-rose-300' },
    indeterminate: { label: 'Indéterminé', className: 'border-amber-500/40 bg-amber-500/10 text-amber-300' },
    error: { label: 'Erreur', className: 'border-red-500/40 bg-red-500/10 text-red-300' },
  }

  const c = config[outcome]
  return (
    <span className={`inline-flex items-center rounded border px-2 py-0.5 text-xs font-medium ${c.className}`}>
      {c.label}
    </span>
  )
}

function StatBox({ label, value, tone }: { label: string; value: number; tone: string }) {
  const tones: Record<string, string> = {
    emerald: 'border-emerald-500/40 bg-emerald-500/5 text-emerald-300',
    slate: 'border-slate-500/40 bg-slate-500/5 text-slate-300',
    rose: 'border-rose-500/40 bg-rose-500/5 text-rose-300',
    amber: 'border-amber-500/40 bg-amber-500/5 text-amber-300',
  }

  return (
    <div className={`rounded-lg border ${tones[tone]} px-5 py-4`}>
      <p className="text-xs text-[var(--color-muted)]">{label}</p>
      <p className="mt-1 text-2xl font-semibold tabular-nums">{value}</p>
    </div>
  )
}

function truncate(str: string, len: number) {
  return str.length > len ? str.slice(0, len) + '...' : str
}
