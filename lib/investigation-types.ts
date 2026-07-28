// Investigation types for AI settings

export type InvestigationType = 'general' | 'cyber' | 'grooming' | 'fraud' | 'harassment'

export interface InvestigationPreset {
  value: InvestigationType
  label: string
  description: string
  recommendedTemperature: number
  recommendedStatusTimeoutMs: number
  recommendedGenerateTimeoutMs: number
}

export const INVESTIGATION_TYPE_OPTIONS: InvestigationPreset[] = [
  {
    value: 'general',
    label: 'Général',
    description: 'Configuration équilibrée pour la plupart des enquêtes',
    recommendedTemperature: 0.5,
    recommendedStatusTimeoutMs: 5000,
    recommendedGenerateTimeoutMs: 60000,
  },
  {
    value: 'cyber',
    label: 'Cybercriminalité',
    description: 'Spécialisé pour les IOC, analyise technique détaillée',
    recommendedTemperature: 0.3,
    recommendedStatusTimeoutMs: 3000,
    recommendedGenerateTimeoutMs: 90000,
  },
  {
    value: 'grooming',
    label: 'Grooming / Pédocriminalité',
    description: 'Sensible, detection de patterns précis, temperature basse',
    recommendedTemperature: 0.1,
    recommendedStatusTimeoutMs: 2000,
    recommendedGenerateTimeoutMs: 120000,
  },
  {
    value: 'fraud',
    label: 'Arnaque / Fraude',
    description: 'Detection de motifs d\'arnaque, analyse de patterns',
    recommendedTemperature: 0.4,
    recommendedStatusTimeoutMs: 4000,
    recommendedGenerateTimeoutMs: 75000,
  },
  {
    value: 'harassment',
    label: 'Harcèlement',
    description: 'Analyse de comportements récurrents, tracking pattern',
    recommendedTemperature: 0.2,
    recommendedStatusTimeoutMs: 3000,
    recommendedGenerateTimeoutMs: 100000,
  },
]

export function getInvestigationPreset(type: InvestigationType): InvestigationPreset {
  const preset = INVESTIGATION_TYPE_OPTIONS.find((p) => p.value === type)
  return preset || INVESTIGATION_TYPE_OPTIONS[0]
}

export function getTemperatureProfile(temperature: number): {
  label: string
  description: string
} {
  if (temperature <= 0.2) {
    return { label: 'Précision', description: 'Factuel et strict, peu de variabilité' }
  }
  if (temperature <= 0.4) {
    return { label: 'Équilibre', description: 'Bon compromis entre précision et créativité' }
  }
  return { label: 'Exploration', description: 'Pistes et variantes, plus de créativité' }
}
