// Mock search module for development
// In production, this would use Tauri search commands

export interface SearchHit {
  kind: string
  refId: string
  caseId?: string
  title: string
  body: string
  score: number
}

export interface SearchOptions {
  kinds?: string[]
  caseId?: string
}

export async function search(
  query: string,
  options?: SearchOptions,
): Promise<SearchHit[]> {
  // Mock implementation - would call Tauri search_action in production
  return []
}

export async function reindexAll(): Promise<void> {
  // Mock implementation - would call Tauri reindex_all in production
}
