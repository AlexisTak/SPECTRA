// Mock settings module for development
// In production, this would use Tauri settings commands

import type { AiSettings } from '@/lib/ai-settings'

export async function getAiSettings(): Promise<AiSettings> {
  // Mock implementation - would call Tauri get_ai_settings in production
  return {
    enabled: false,
    baseUrl: 'http://localhost:11434',
    defaultModel: 'qwen2.5-coder:7b',
    embeddingModel: 'nomic-embed-text',
    investigationType: 'general',
    statusTimeoutMs: 3000,
    generateTimeoutMs: 60000,
    temperature: 0.7,
  }
}

export async function saveAiSettings(
  settings: Partial<AiSettings>,
): Promise<AiSettings> {
  // Mock implementation - would call Tauri save_ai_settings in production
  return {
    enabled: settings.enabled ?? false,
    baseUrl: settings.baseUrl ?? 'http://localhost:11434',
    defaultModel: settings.defaultModel ?? 'qwen2.5-coder:7b',
    embeddingModel: settings.embeddingModel ?? 'nomic-embed-text',
    investigationType: settings.investigationType ?? 'general',
    statusTimeoutMs: settings.statusTimeoutMs ?? 3000,
    generateTimeoutMs: settings.generateTimeoutMs ?? 60000,
    temperature: settings.temperature ?? 0.7,
  }
}

export async function getAvailableOllamaModels(): Promise<{ models: string[]; error?: string }> {
  // Mock implementation - would call Tauri get_ai_settings + fetch Ollama in production
  return { models: ['qwen2.5-coder:7b', 'llama3:8b', 'mistral:7b'] }
}

export async function testOllamaConnection(
  settings: AiSettings,
): Promise<{ ok: boolean; message: string }> {
  // Mock implementation - would call Tauri get_ai_settings + fetch Ollama in production
  return { ok: true, message: 'Test Ollama réussi' }
}
