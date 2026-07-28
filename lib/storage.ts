// Mock storage module for development
// In production, this would use Tauri FS API

export interface StorageConfig {
  rootDir: string
}

export const DEFAULT_STORAGE_CONFIG: StorageConfig = {
  rootDir: '.storage',
}

export function getStoragePath(kind: string, id?: string): string {
  const base = DEFAULT_STORAGE_CONFIG.rootDir
  if (id) {
    return `${base}/${kind}/${id.substring(0, 2)}/${id}`
  }
  return `${base}/${kind}`
}

export function ensureStorageDirs(): string[] {
  const dirs = [
    'evidence',
    'tiktok',
    'web-archives',
    'snapshots',
    'transcriptions',
    'temp',
  ]
  return dirs.map((d) => `${DEFAULT_STORAGE_CONFIG.rootDir}/${d}`)
}
