// Mock Ollama client for development
// In production, this would use Tauri invoke

import type { AiSettings } from '@/lib/ai-settings'

const DEFAULT_TIMEOUT = 3000

export async function fetchOllamaStatus(): Promise<{ available: boolean; models: string[]; baseUrl: string; error?: string }> {
  try {
    const response = await fetch('http://localhost:11434/api/tags', {
      signal: AbortSignal.timeout(DEFAULT_TIMEOUT),
    })
    const data = await response.json()
    const models = (data.models as Array<{ name: string }>).map((m) => m.name)
    return { available: true, models, baseUrl: 'http://localhost:11434' }
  } catch {
    return {
      available: false,
      models: [],
      error: 'Impossible de se connecter à Ollama',
      baseUrl: 'http://localhost:11434',
    }
  }
}

export async function getAvailableOllamaModels(
  settings?: Partial<AiSettings>,
): Promise<{ models: string[]; error?: string }> {
  try {
    const baseUrl = settings?.baseUrl ?? 'http://localhost:11434'
    const response = await fetch(`${baseUrl}/api/tags`, {
      signal: AbortSignal.timeout(settings?.statusTimeoutMs ?? DEFAULT_TIMEOUT),
    })
    const data = await response.json()
    const models = (data.models as Array<{ name: string }>).map((m) => m.name)
    return { models }
  } catch {
    return { models: [], error: 'Impossible de récupérer les modèles' }
  }
}

export async function testOllamaConnection(
  settings: AiSettings,
): Promise<{ ok: boolean; message: string; model?: string }> {
  if (!settings.enabled) {
    return { ok: false, message: 'IA désactivée' }
  }
  try {
    const { models } = await getAvailableOllamaModels(settings)
    if (models.length === 0) {
      return { ok: false, message: 'Aucun modèle disponible' }
    }
    // Check if default model exists
    if (!models.includes(settings.defaultModel)) {
      return {
        ok: false,
        message: `Modèle "${settings.defaultModel}" introuvable`,
        model: models[0],
      }
    }
    return { ok: true, message: 'Connexion Ollama réussie', model: settings.defaultModel }
  } catch {
    return { ok: false, message: 'Erreur de connexion à Ollama' }
  }
}
