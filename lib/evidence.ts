// Mock evidence module for development
// In production, this would use Tauri evidence commands

import type { Evidence, IntegrityStatus } from '@/types'

export interface AddEvidenceInput {
  caseId: string
  filename: string
  chemin?: string
  tailleOctets?: number
  typeMime?: string
  sha256?: string
  md5?: string
  notes?: string
}

export interface IntegrityCheck {
  evidenceId: string
  filename: string
  status: IntegrityStatus
  expectedHash?: string
  actualHash?: string
  message?: string
}

export interface CaseIntegrityReport {
  caseId: string
  totalChecked: number
  intactCount: number
  alteredCount: number
  missingCount: number
  noHashCount: number
  noPathCount: number
  checks: IntegrityCheck[]
  generatedAt: string
}

export async function getEvidence(caseId: string): Promise<Evidence[]> {
  // Mock implementation - would call Tauri get_evidence in production
  return []
}

export async function addEvidence(
  data: AddEvidenceInput,
): Promise<Evidence> {
  // Mock implementation - would call Tauri add_evidence in production
  return {
    id: crypto.randomUUID(),
    caseId: data.caseId,
    filename: data.filename,
    dateImport: new Date().toISOString(),
  }
}

export async function deleteEvidence(
  id: string,
): Promise<void> {
  // Mock implementation - would call Tauri delete_evidence in production
}

export async function verifyCaseEvidence(
  caseId: string,
): Promise<CaseIntegrityReport> {
  // Mock implementation - would call Tauri verify_case_evidence in production
  return {
    caseId,
    totalChecked: 0,
    intactCount: 0,
    alteredCount: 0,
    missingCount: 0,
    noHashCount: 0,
    noPathCount: 0,
    checks: [],
    generatedAt: new Date().toISOString(),
  }
}
