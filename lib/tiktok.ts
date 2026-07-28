// Mock tiktok module for development
// In production, this would use Tauri tiktok commands

import type { TiktokVideo } from '@/types'

export async function getTiktokVideos(caseId: string): Promise<TiktokVideo[]> {
  // Mock implementation - would call Tauri commands for tiktok in production
  return []
}

export async function addTiktokVideo(
  caseId: string,
  url: string,
): Promise<TiktokVideo> {
  // Mock implementation - would call Tauri commands for tiktok in production
  return {
    id: crypto.randomUUID(),
    caseId,
    url,
    downloadDate: new Date().toISOString(),
    hashtags: [],
    mentions: [],
    transcriptDone: false,
    deletedRemote: false,
  }
}

export async function deleteTiktokVideo(id: string): Promise<void> {
  // Mock implementation - would call Tauri commands for tiktok in production
}
