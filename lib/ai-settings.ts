// AI settings types
export interface AiSettings {
  enabled: boolean
  baseUrl: string
  defaultModel: string
  embeddingModel: string
  investigationType: string
  statusTimeoutMs: number
  generateTimeoutMs: number
  temperature: number
}

export const DEFAULT_AI_SETTINGS: AiSettings = {
  enabled: false,
  baseUrl: 'http://localhost:11434',
  defaultModel: 'qwen2.5-coder:7b',
  embeddingModel: 'nomic-embed-text',
  investigationType: 'general',
  statusTimeoutMs: 3000,
  generateTimeoutMs: 60000,
  temperature: 0.7,
}
