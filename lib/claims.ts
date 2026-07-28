// Mock claims module for development
// In production, this would use Tauri claims commands

import type { Qualification } from '@/lib/claims-types'
import type { Claim } from '@/types'

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

export async function setClaim(input: SetClaimInput): Promise<Claim> {
  // Mock implementation - would call Tauri set_claim in production
  return {
    id: crypto.randomUUID(),
    caseId: input.caseId,
    refKind: input.refKind,
    refId: input.refId,
    qualification: input.qualification,
    reliability: input.reliability,
    source: input.source,
    takenAt: new Date().toISOString(),
  }
}

export async function getClaim(
  refKind: string,
  refId: string,
): Promise<Claim | null> {
  // Mock implementation - would call Tauri get_claim in production
  return null
}

export async function listClaims(caseId: string): Promise<Claim[]> {
  // Mock implementation - would call Tauri list_claims in production
  return []
}

export async function deleteClaim(id: string): Promise<void> {
  // Mock implementation - would call Tauri delete_claim in production
}

export async function getClaimStats(
  caseId: string,
): Promise<Record<string, number>> {
  // Mock implementation - would call Tauri get_claim_stats in production
  return { total: 0 }
}
