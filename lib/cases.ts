// Mock cases module for development
// In production, this would use Tauri cases commands

import type { Case, CaseStatus, CasePriority, CaseCategory } from '@/types'

export interface CreateCaseInput {
  titre: string
  description?: string
  categorie?: string
  priorite: CasePriority
  enqueteur?: string
  notes?: string
  tags?: string[]
}

export interface UpdateCaseInput {
  titre?: string
  description?: string
  categorie?: string
  priorite?: CasePriority
  enqueteur?: string
  notes?: string
  tags?: string[]
  statut?: CaseStatus
}

export async function getCases(
  filters?: { statut?: string; search?: string },
): Promise<Case[]> {
  // Mock implementation - would call Tauri get_cases in production
  return []
}

export async function getCase(id: string): Promise<Case | null> {
  // Mock implementation - would call Tauri get_case in production
  return null
}

export async function getCaseStats(): Promise<Record<string, number>> {
  // Mock implementation - would call Tauri get_case_stats in production
  return { total: 0, ouvert: 0, en_cours: 0, clos: 0 }
}

export async function createCase(data: CreateCaseInput): Promise<Case> {
  // Mock implementation - would call Tauri create_case in production
  return {
    id: crypto.randomUUID(),
    reference: `ENQ-${new Date().getFullYear()}-0001`,
    titre: data.titre,
    description: data.description,
    categorie: data.categorie as CaseCategory | undefined,
    statut: 'ouvert',
    priorite: data.priorite,
    enqueteur: data.enqueteur,
    dateCreation: new Date().toISOString(),
    tags: data.tags || [],
  }
}

export async function updateCase(
  id: string,
  data: UpdateCaseInput,
): Promise<Case> {
  // Mock implementation - would call Tauri update_case in production
  return {
    id,
    reference: '',
    titre: data.titre || '',
    description: data.description,
    statut: 'ouvert' as any,
    priorite: data.priorite || 'normale',
    dateCreation: new Date().toISOString(),
    tags: [],
  }
}

export async function deleteCase(id: string): Promise<void> {
  // Mock implementation - would call Tauri delete_case in production
}
