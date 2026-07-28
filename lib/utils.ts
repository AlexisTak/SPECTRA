// Utility functions

import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs))
}

export function formatDate(
  iso: string | undefined | null,
  withTime = false,
): string {
  if (!iso) return '—'
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '—'
  const date = d.toLocaleDateString('fr-FR', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
  })
  if (!withTime) return date
  const time = d.toLocaleTimeString('fr-FR', {
    hour: '2-digit',
    minute: '2-digit',
  })
  return `${date} ${time}`
}

export function formatBytes(n: number | undefined | null): string {
  if (!n && n !== 0) return '—'
  if (n < 1024) return `${n} o`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} Kio`
  if (n < 1024 * 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} Mio`
  return `${(n / 1024 / 1024 / 1024).toFixed(2)} Gio`
}

export function parseTags(raw: string | null | undefined): string[] {
  if (!raw) return []
  try {
    const parsed = JSON.parse(raw)
    return Array.isArray(parsed) ? parsed.filter((t) => typeof t === 'string') : []
  } catch {
    return []
  }
}

export function safeJsonParse<T>(
  raw: string | null | undefined,
  fallback: T,
): T {
  if (!raw) return fallback
  try {
    return JSON.parse(raw) as T
  } catch {
    return fallback
  }
}

export function truncate(text: string | null | undefined, maxLength: number): string {
  if (!text) return '—'
  if (text.length <= maxLength) return text
  return text.substring(0, maxLength - 3) + '...'
}
