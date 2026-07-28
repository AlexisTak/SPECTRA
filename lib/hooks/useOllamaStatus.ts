// lib/hooks/useOllamaStatus.ts
// =============================================================================
// Hook qui poll le statut d'Ollama toutes les ~30s et le rend dispo
// aux composants enfants. On reste en polling plutôt qu'en SSE/WebSocket
// parce que l'API Ollama ne pousse rien, et qu'un ping toutes les 30s
// suffit largement pour un bandeau de statut.
//
// Au montage on interroge immédiatement, puis on cadence.
//
// Tauri 2 — au lieu de fetch('/api/ollama/status') (qui n'existe plus
// avec output: 'export'), on passe par le bridge qui appelle
// get_ai_settings puis fetch directement Ollama (autorisé par le CSP).

import { useEffect, useState } from 'react'
import type { OllamaStatus } from '@/types'
import { fetchOllamaStatus } from '@/lib/ollama'

const POLL_MS = 30_000
const INITIAL_DELAY_MS = 500 // ne pas bloquer le 1er render

const FALLBACK_STATUS: OllamaStatus = {
  available: false,
  baseUrl: '',
  models: [],
  error: 'IA indisponible',
}

export function useOllamaStatus(): OllamaStatus | null {
  const [status, setStatus] = useState<OllamaStatus | null>(null)

  useEffect(() => {
    let cancelled = false

    async function poll() {
      try {
        const r = await fetchOllamaStatus()
        if (cancelled) return
        setStatus({
          available: r.available,
          baseUrl: r.baseUrl || '',
          models: r.models,
          error: r.error,
        })
      } catch {
        if (!cancelled) setStatus(FALLBACK_STATUS)
      }
    }

    const t = setTimeout(poll, INITIAL_DELAY_MS)
    const id = setInterval(poll, POLL_MS)

    return () => {
      cancelled = true
      clearTimeout(t)
      clearInterval(id)
    }
  }, [])

  return status
}
