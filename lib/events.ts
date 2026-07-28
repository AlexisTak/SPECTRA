// Mock events module for development
// In production, this would use Tauri events commands

import type { CaseEvent, EventType } from '@/types'

export async function getEvents(caseId: string): Promise<CaseEvent[]> {
  // Mock implementation - would call Tauri get_events in production
  return []
}

export async function addNote(
  caseId: string,
  description: string,
): Promise<CaseEvent> {
  // Mock implementation - would call Tauri add_note in production
  return {
    id: crypto.randomUUID(),
    caseId,
    typeEvent: 'note',
    description,
    dateEvent: new Date().toISOString(),
  }
}

export async function addEvidenceEvent(
  caseId: string,
  filename: string,
): Promise<CaseEvent> {
  return {
    id: crypto.randomUUID(),
    caseId,
    typeEvent: 'evidence_added',
    description: `Preuve ajoutée : ${filename}`,
    dateEvent: new Date().toISOString(),
  }
}

export async function addStatusChangeEvent(
  caseId: string,
  fromStatus: string,
  toStatus: string,
): Promise<CaseEvent> {
  return {
    id: crypto.randomUUID(),
    caseId,
    typeEvent: 'status_change',
    description: `Statut : ${fromStatus} → ${toStatus}`,
    dateEvent: new Date().toISOString(),
  }
}
