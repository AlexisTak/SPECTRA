// lib/hooks/useTauriQuery.ts
// =============================================================================
// Hook React qui wrappe `tauriInvoke` avec gestion loading/error/refetch.
//
// Pourquoi : dans les pages migrées en Client Components, on ne peut plus
// faire `await getCase(id)` côté serveur (plus de Server Action possible
// avec output:'export'). On délègue donc à Tauri via invoke(), et on gère
// loading/error côté React.
//
// API :
//   const { data, loading, error, refetch } = useTauriQuery<T>(
//     'get_case',
//     { id },                  // args (objet nommé passé à invoke)
//     [id]                    // deps custom (en plus de command + JSON.stringify(args))
//   )
//
// `args` peut être `null` pour skip l'appel (utile pour les requêtes
// paramétrées par un `id` encore non disponible).

import { useEffect, useState } from 'react'
import { tauriInvoke } from '@/lib/tauri-bridge'

export interface TauriQueryResult<T> {
  data: T | null
  loading: boolean
  error: string | null
  refetch: () => void
}

export function useTauriQuery<T>(
  command: string,
  args: Record<string, unknown> | null,
  deps: unknown[] = [],
): TauriQueryResult<T> {
  const [data, setData] = useState<T | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [tick, setTick] = useState(0)

  // Sérialiser args pour la dépendance (les objets ne sont jamais égaux
  // par référence, JSON.stringify donne une clé stable).
  const argsKey = args === null ? 'null' : JSON.stringify(args)

  useEffect(() => {
    let cancelled = false
    setLoading(true)
    setError(null)

    // Si args est null, on ne déclenche pas la requête (le caller signal
    // explicitement qu'il n'est pas prêt — ex: id pas encore connu).
    if (args === null) {
      setData(null)
      setLoading(false)
      return () => {
        cancelled = true
      }
    }

    tauriInvoke<T>(command, args)
      .then((d) => {
        if (!cancelled) {
          setData(d)
          setLoading(false)
        }
      })
      .catch((e: unknown) => {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : String(e))
          setLoading(false)
        }
      })

    return () => {
      cancelled = true
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [command, argsKey, tick, ...deps])

  return {
    data,
    loading,
    error,
    refetch: () => setTick((t) => t + 1),
  }
}
