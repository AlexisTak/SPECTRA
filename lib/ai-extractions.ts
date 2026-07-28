// Mock AI extractions module for development
// In production, this would use Tauri AI commands

import type { AiExtractionKind } from '@/types'

export interface ExtractionInput {
  caseId: string
  kind: AiExtractionKind
  prompt: string
  evidenceId?: string
}

export async function createExtraction(
  input: ExtractionInput,
): Promise<void> {
  // Mock implementation - would call Tauri commands for AI extractions in production
}

export async function getExtractions(
  caseId: string,
): Promise<Record<string, unknown>[]> {
  // Mock implementation - would call Tauri commands for AI extractions in production
  return []
}
