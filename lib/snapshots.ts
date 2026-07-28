// Mock snapshots module for development
// In production, this would use Tauri snapshots commands

import type { Snapshot, SnapshotKind } from '@/types'

export interface CreateSnapshotInput {
  caseId: string
  label?: string
}

export async function takeSnapshot(
  input: CreateSnapshotInput,
): Promise<Snapshot> {
  // Mock implementation - would call Tauri take_snapshot in production
  return {
    id: crypto.randomUUID(),
    caseId: input.caseId,
    label: input.label,
    takenAt: new Date().toISOString(),
    sha256: crypto.randomUUID(),
    tailleOctets: 0,
    content: '{}',
    kind: 'manual' as SnapshotKind,
    filePath: '',
  }
}

export async function listSnapshots(
  caseId: string,
): Promise<Snapshot[]> {
  // Mock implementation - would call Tauri list_snapshots in production
  return []
}

export async function getSnapshot(
  id: string,
): Promise<Snapshot | null> {
  // Mock implementation - would call Tauri get_snapshot in production
  return null
}

export async function verifySnapshotIntegrity(
  id: string,
): Promise<string> {
  // Mock implementation - would call Tauri verify_snapshot_integrity in production
  return 'intact'
}

export async function deleteSnapshot(
  id: string,
): Promise<void> {
  // Mock implementation - would call Tauri delete_snapshot in production
}
