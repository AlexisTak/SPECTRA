'use client'

/**
 * Palette de commandes globale (Ctrl+K).
 *
 * Inspirée de VS Code, Raycast, et les outils d'analyse modernes.
 * Tout est accessible au clavier : navigation flèches, Entrée pour activer,
 * Échap pour fermer, Ctrl+K pour ouvrir.
 */

import React, { useEffect, useState, useRef, useCallback } from 'react'
import { useRouter } from 'next/navigation'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'

interface CommandItem {
  id: string
  label: string
  shortcut?: string
  category?: string
  action: () => void
}

export function CommandPaletteProvider({ children }: { children: React.ReactNode }) {
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)
  const router = useRouter()
  const { t } = useTranslation()

  const commands: CommandItem[] = [
    { id: 'nav-dashboard', label: t('nav.dashboard') ?? 'Dashboard', category: t('nav.dashboard'), action: () => router.push('/') },
    { id: 'nav-cases', label: t('nav.cases') ?? 'Cases', category: t('nav.cases'), action: () => router.push('/cases') },
    { id: 'nav-osint', label: t('nav.osint') ?? 'OSINT', category: 'OSINT', action: () => router.push('/osint') },
    { id: 'nav-graph', label: t('nav.graph') ?? 'Graph', category: t('nav.graph'), action: () => router.push('/graph') },
    { id: 'nav-timeline', label: 'Timeline', category: 'Timeline', action: () => router.push('/timeline') },
    { id: 'nav-table', label: 'Table', category: 'Table', action: () => router.push('/table') },
    { id: 'nav-map', label: 'Carte', category: 'Carte', action: () => router.push('/map') },
    { id: 'nav-notes', label: 'Notes', category: 'Notes', action: () => router.push('/notes') },
    { id: 'nav-ach', label: 'ACH', category: 'ACH', action: () => router.push('/ach') },
    { id: 'nav-ai', label: 'IA', category: 'IA', action: () => router.push('/ai') },
    { id: 'nav-search', label: t('nav.search') ?? 'Search', category: t('nav.search'), action: () => router.push('/search') },
    { id: 'nav-reports', label: t('nav.reports') ?? 'Reports', category: t('nav.reports'), action: () => router.push('/reports') },
    { id: 'nav-audit', label: t('nav.integrity') ?? 'Integrity', category: t('nav.integrity'), action: () => router.push('/audit') },
    { id: 'nav-settings', label: 'Paramètres', category: 'Paramètres', action: () => router.push('/settings') },
  ]

  const filtered = query.trim()
    ? commands.filter((c) => c.label.toLowerCase().includes(query.toLowerCase()))
    : commands

  useEffect(() => {
    setSelectedIndex(0)
  }, [query])

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault()
        setOpen((prev) => !prev)
      }
      if (e.key === 'Escape') {
        setOpen(false)
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [])

  useEffect(() => {
    if (open && inputRef.current) {
      inputRef.current.focus()
    }
  }, [open])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        setSelectedIndex((i) => (i + 1) % filtered.length)
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        setSelectedIndex((i) => (i - 1 + filtered.length) % filtered.length)
      } else if (e.key === 'Enter') {
        e.preventDefault()
        const cmd = filtered[selectedIndex]
        if (cmd) {
          cmd.action()
          setOpen(false)
          setQuery('')
        }
      }
    },
    [filtered, selectedIndex]
  )

  if (!open) return <>{children}</>

  return (
    <>
      {children}
      {/* Overlay */}
      <div
        className="fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-[20vh]"
        role="dialog"
        aria-modal="true"
        aria-label={t('commandPalette.title') ?? 'Command palette'}
        onClick={(e) => {
          if (e.target === e.currentTarget) setOpen(false)
        }}
      >
        <div className="w-full max-w-xl overflow-hidden rounded-lg border border-[var(--color-edge)] bg-[var(--color-panel)] shadow-2xl">
          {/* Input */}
          <div className="flex items-center gap-3 border-b border-[var(--color-edge)] px-4 py-3">
            <span className="text-sm text-[var(--color-muted)]">⌘</span>
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={t('commandPalette.placeholder') ?? 'Type a command…'}
              className="flex-1 bg-transparent text-sm outline-none text-[var(--color-ink)] placeholder:text-[var(--color-muted)]"
              aria-autocomplete="list"
              aria-controls="command-list"
              aria-activedescendant={filtered[selectedIndex]?.id}
            />
            <kbd className="rounded border border-[var(--color-edge)] bg-[var(--color-surface)] px-1.5 py-0.5 text-[10px] text-[var(--color-muted)]">
              ESC
            </kbd>
          </div>

          {/* Liste */}
          <ul
            id="command-list"
            role="listbox"
            className="max-h-[50vh] overflow-y-auto py-2"
          >
            {filtered.length === 0 && (
              <li className="px-4 py-3 text-sm text-[var(--color-muted)]">
                Aucun résultat
              </li>
            )}
            {filtered.map((cmd, i) => (
              <li
                key={cmd.id}
                id={cmd.id}
                role="option"
                aria-selected={i === selectedIndex}
                onClick={() => {
                  cmd.action()
                  setOpen(false)
                  setQuery('')
                }}
                className={cn(
                  'flex cursor-pointer items-center justify-between px-4 py-2.5 text-sm transition-colors',
                  i === selectedIndex
                    ? 'bg-sky-500/15 text-sky-200'
                    : 'text-[var(--color-ink)] hover:bg-white/5'
                )}
              >
                <div className="flex items-center gap-3">
                  <span className="text-xs text-[var(--color-muted)]">
                    {cmd.category}
                  </span>
                  <span>{cmd.label}</span>
                </div>
                {cmd.shortcut && (
                  <kbd className="rounded border border-[var(--color-edge)] bg-[var(--color-surface)] px-1.5 py-0.5 text-[10px] text-[var(--color-muted)]">
                    {cmd.shortcut}
                  </kbd>
                )}
              </li>
            ))}
          </ul>

          {/* Footer */}
          <div className="flex items-center justify-between border-t border-[var(--color-edge)] px-4 py-2 text-[10px] text-[var(--color-muted)]">
            <span>↑↓ pour naviguer · ↵ pour activer</span>
            <span>{filtered.length} commande{filtered.length > 1 ? 's' : ''}</span>
          </div>
        </div>
      </div>
    </>
  )
}
