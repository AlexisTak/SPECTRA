// Mock transcriptions module for development
// In production, this would use Tauri transcription commands

export interface TranscriptionInput {
  caseId: string
  evidenceId: string
  language: string
  text: string
  model?: string
}

export async function createTranscription(
  input: TranscriptionInput,
): Promise<void> {
  // Mock implementation - would call Tauri commands for transcriptions in production
}

export async function getTranscriptions(
  caseId: string,
): Promise<Record<string, unknown>[]> {
  // Mock implementation - would call Tauri commands for transcriptions in production
  return []
}
