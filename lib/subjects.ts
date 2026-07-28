// Mock subjects module for development
// In production, this would use Tauri subjects commands

import type { Subject, SubjectStatus } from '@/types'

export interface CreateSubjectInput {
  caseId: string
  alias?: string
  nomReel?: string
  dateNaissance?: string
  nationalite?: string
  ips?: string[]
  comptes?: { plateforme: string; identifiant: string }[]
  emails?: string[]
  telephones?: string[]
  description?: string
  statut?: SubjectStatus
}

export interface UpdateSubjectInput {
  alias?: string
  nomReel?: string
  dateNaissance?: string
  nationalite?: string
  ips?: string[]
  comptes?: { plateforme: string; identifiant: string }[]
  emails?: string[]
  telephones?: string[]
  description?: string
  statut?: SubjectStatus
}

export async function getSubjects(caseId: string): Promise<Subject[]> {
  // Mock implementation - would call Tauri get_subjects in production
  return []
}

export async function createSubject(
  data: CreateSubjectInput,
): Promise<Subject> {
  // Mock implementation - would call Tauri create_subject in production
  return {
    id: crypto.randomUUID(),
    caseId: data.caseId,
    alias: data.alias,
    nomReel: data.nomReel,
    dateNaissance: data.dateNaissance,
    nationalite: data.nationalite,
    ips: data.ips || [],
    comptes: data.comptes || [],
    emails: data.emails || [],
    telephones: data.telephones || [],
    description: data.description,
    statut: data.statut || 'suspect',
    dateCreation: new Date().toISOString(),
  }
}

export async function updateSubject(
  id: string,
  data: UpdateSubjectInput,
): Promise<Subject> {
  // Mock implementation - would call Tauri update_subject in production
  return {
    id,
    caseId: '',
    statut: 'suspect' as any,
    dateCreation: new Date().toISOString(),
    ips: [],
    comptes: [],
    emails: [],
    telephones: [],
  }
}

export async function deleteSubject(id: string): Promise<void> {
  // Mock implementation - would call Tauri delete_subject in production
}
