// Claims types partagés entre Rust et TypeScript

export type Qualification = 'indice' | 'preuve' | 'hypothèse' | 'non_vérifié'

export interface Claim {
  id: string
  caseId: string
  refKind: string
  refId: string
  qualification: Qualification
  reliability: number
  source: string
  sourceUrl?: string
  takenBy?: string
  takenAt: string
  notes?: string
}

export interface ClaimStats {
  total: number
  byQualification: Record<string, number>
  averageReliability: number
  unverifiedCount: number
  totalEntities: number
}

export interface SetClaimInput {
  caseId: string
  refKind: string
  refId: string
  qualification: Qualification
  reliability: number
  source: string
  sourceUrl?: string
  takenBy?: string
  notes?: string
}

export const QUALIFICATIONS: Qualification[] = [
  'indice',
  'preuve',
  'hypothèse',
  'non_vérifié',
]

export function isValidQualification(value: string): value is Qualification {
  return QUALIFICATIONS.includes(value as Qualification)
}

export function isEligibleForEvidence(qualification: Qualification): boolean {
  return ['preuve', 'indice'].includes(qualification)
}
