// Mock web archive module for development
// In production, this would use Tauri web archive commands

import type { WebArchive } from '@/types'

export async function getWebArchives(caseId: string): Promise<WebArchive[]> {
  // Mock implementation - would call Tauri commands for web archives in production
  return []
}

export async function addWebArchive(
  caseId: string,
  url: string,
): Promise<WebArchive> {
  // Mock implementation - would call Tauri commands for web archives in production
  return {
    id: crypto.randomUUID(),
    caseId,
    url,
    urlHash: crypto.randomUUID(),
    fetchedDate: new Date().toISOString(),
    assetCount: 0,
  }
}

export async function deleteWebArchive(id: string): Promise<void> {
  // Mock implementation - would call Tauri commands for web archives in production
}
