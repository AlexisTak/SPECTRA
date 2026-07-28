// Mock cross links module for development
// In production, this would use Tauri cross links commands

import type { CrossLinkType } from '@/types'

export interface CreateCrossLinkInput {
  sourceCaseId: string
  targetCaseId: string
  linkType: CrossLinkType
  linkValue: string
  confidence?: number
  notes?: string
}

export async function createCrossLink(
  data: CreateCrossLinkInput,
): Promise<void> {
  // Mock implementation - would call Tauri commands for cross links in production
}

export async function deleteCrossLink(id: string): Promise<void> {
  // Mock implementation - would call Tauri commands for cross links in production
}

export async function getCrossLinks(
  caseId: string,
): Promise<Record<string, string>[]> {
  // Mock implementation - would call Tauri commands for cross links in production
  return []
}
