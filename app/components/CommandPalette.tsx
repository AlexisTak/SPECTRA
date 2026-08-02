'use client'

/**
 * Palette de commandes globale (Ctrl+K).
 *
 * Tout est accessible au clavier : flèches pour naviguer,
 * Entrée pour activer, Échap pour fermer.
 * Conforme WCAG 2.1 AA : focus trap, aria-modal, aria-activedescendant,
 * restauration du focus à la fermeture.
 */

import { useEffect, useRef, useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { useRouter } from 'next/navigation'
import { cn } from '@/lib/utils'

interface CommandItem {
  id: string
  label: string
  shortcut?: string
  category?: string
  action: () => void
}

export function CommandPalette() {
  const { t } = useTranslation()
  const router = useRouter()
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const [index, setIndex] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)
  const listRef = useRef<HTMLUListElement>(null)
  // Élément focalisé juste avant l'ouverture, pour restaurer le focus après.
  const triggerRef = useRef<HTMLElement | null>(null)

  const nav = t('commandPalette.navigation')
  const commands: CommandItem[] = [
    { id: 'go-dashboard', label: t('nav.dashboard'), category: nav, action: () => router.push('/') },
    { id: 'go-cases', label: t('nav.cases'), category: nav, action: () => router.push('/cases') },
    { id: 'go-osint', label: t('nav.osint'), category: nav, action: () => router.push('/osint') },
    { id: 'go-graph', label: t('nav.graph'), category: nav, action: () => router.push('/graph') },
    { id: 'go-timeline', label: t('nav.timeline'), category: nav, action: () => router.push('/timeline') },
    { id: 'go-table', label: t('nav.table'), category: nav, action: () => router.push('/table') },
    { id: 'go-map', label: t('nav.map'), category: nav, action: () => router.push('/map') },
    { id: 'go-notes', label: t('nav.notes'), category: nav, action: () => router.push('/notes') },
    { id: 'go-ach', label: t('nav.ach'), category: nav, action: () => router.push('/ach') },
    { id: 'go-ai', label: t('nav.ai'), category: nav, action: () => router.push('/ai') },
    { id: 'go-search', label: t('nav.search'), category: nav, action: () => router.push('/search') },
    { id: 'go-reports', label: t('nav.reports'), category: nav, action: () => router.push('/reports') },
    { id: 'go-audit', label: t('nav.integrity'), category: nav, action: () => router.push('/audit') },
    { id: 'go-settings', label: t('nav.settings'), category: nav, action: () => router.push('/settings') },
  ]

  const filtered = query.trim()
    ? commands.filter((c) => c.label.toLowerCase().includes(query.toLowerCase()))
    : commands

  useEffect(() => {
    setIndex(0)
  }, [query])

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault()
        setOpen((prev) => {
          const next = !prev
          if (next) {
            triggerRef.current = document.activeElement as HTMLElement | null
            setQuery('')
            setIndex(0)
          }
          return next
        })
      }
      if (e.key === 'Escape') {
        setOpen((prev) => {
          if (prev) setQuery('')
          return false
        })
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [])

  // Focus l'input à l'ouverture, restaure le focus déclencheur à la fermeture.
  // L'élément est capturé AVANT l'ouverture : à la fermeture la palette est
  // démontée, `document.activeElement` vaudrait alors <body>.
  useEffect(() => {
    if (!open) return
    const trigger = triggerRef.current
    inputRef.current?.focus()
    return () => {
      trigger?.focus?.()
    }
  }, [open])

  // Scroll l'élément sélectionné dans la vue
  useEffect(() => {
    if (open && listRef.current) {
      const el = listRef.current.children[index] as HTMLElement | undefined
      el?.scrollIntoView({ block: 'nearest' })
    }
  }, [open, index])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        // Garde : `% 0` produirait NaN et bloquerait la sélection.
        if (filtered.length === 0) return
        setIndex((i) => (i + 1) % filtered.length)
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        if (filtered.length === 0) return
        setIndex((i) => (i - 1 + filtered.length) % filtered.length)
      } else if (e.key === 'Enter') {
        e.preventDefault()
        const item = filtered[index]
        if (item) {
          item.action()
          setOpen(false)
          setQuery('')
        }
      } else if (e.key === 'Tab') {
        e.preventDefault() // piège à focus interne
      }
    },
    [filtered, index]
  )

  if (!open) return null

  const activeId = filtered[index]?.id ?? ''

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-[20vh]"
      onClick={() => {
        setOpen(false)
        setQuery('')
      }}
      role="dialog"
      aria-modal="true"
      aria-label={t('commandPalette.title')}
    >
      <div
        className="w-full max-w-xl overflow-hidden rounded-lg border border-[var(--color-edge)] bg-[var(--color-panel)] shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Input */}
        <div className="flex items-center gap-3 border-b border-[var(--color-edge)] px-4 py-3">
          <span className="text-sm text-[var(--color-muted)]" aria-hidden="true">⌘</span>
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t('commandPalette.placeholder')}
            className="flex-1 bg-transparent text-sm text-[var(--color-ink)] outline-none placeholder:text-[var(--color-muted)]"
            aria-label={t('commandPalette.placeholder')}
            aria-autocomplete="list"
            aria-controls="command-palette-list"
            aria-activedescendant={activeId}
          />
          <kbd className="rounded border border-[var(--color-edge)] bg-[var(--color-surface)] px-1.5 py-0.5 text-[10px] text-[var(--color-muted)]">
            ESC
          </kbd>
        </div>

        {/* Liste */}
        <ul
          ref={listRef}
          id="command-palette-list"
          role="listbox"
          className="max-h-[50vh] overflow-auto py-2"
        >
          {filtered.length === 0 && (
            <li className="px-4 py-3 text-sm text-[var(--color-muted)]">
              {t('commandPalette.noResults')}
            </li>
          )}
          {filtered.map((item, i) => (
            <li
              key={item.id}
              id={item.id}
              role="option"
              aria-selected={i === index}
              className={cn(
                'mx-2 flex cursor-pointer items-center justify-between rounded px-3 py-2 text-sm transition-colors',
                i === index
                  ? 'bg-sky-500/15 text-sky-200'
                  : 'text-[var(--color-muted)] hover:bg-white/5 hover:text-[var(--color-ink)]'
              )}
              onClick={() => {
                item.action()
                setOpen(false)
                setQuery('')
              }}
              onMouseEnter={() => setIndex(i)}
            >
              <div className="flex items-center gap-2">
                {item.category && (
                  <span className="text-[10px] uppercase tracking-wider text-[var(--color-muted)]">
                    {item.category}
                  </span>
                )}
                <span>{item.label}</span>
              </div>
              {item.shortcut && (
                <kbd className="rounded bg-white/10 px-1.5 py-0.5 text-[10px] font-mono text-[var(--color-muted)]">
                  {item.shortcut}
                </kbd>
              )}
            </li>
          ))}
        </ul>

        {/* Footer */}
        <div className="flex items-center gap-4 border-t border-[var(--color-edge)] px-4 py-2 text-[10px] text-[var(--color-muted)]">
          <span>{t('commandPalette.openHint', { shortcut: t('commandPalette.shortcut') })}</span>
          <span className="ml-auto">{t('commandPalette.footer')}</span>
        </div>
      </div>
    </div>
  )
}
