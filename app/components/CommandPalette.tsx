'use client'

import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useRouter } from 'next/navigation'

interface CommandItem {
  id: string
  label: string
  shortcut?: string
  action: () => void
}

export function CommandPalette() {
  const { t } = useTranslation()
  const router = useRouter()
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const [index, setIndex] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)

  const commands: CommandItem[] = [
    { id: 'nav-dashboard', label: t('nav.dashboard'), action: () => router.push('/') },
    { id: 'nav-cases', label: t('nav.cases'), action: () => router.push('/cases') },
    { id: 'nav-osint', label: t('nav.osint'), action: () => router.push('/osint') },
    { id: 'nav-graph', label: t('nav.graph'), action: () => router.push('/graph') },
    { id: 'nav-search', label: t('nav.search'), action: () => router.push('/search') },
    { id: 'nav-reports', label: t('nav.reports'), action: () => router.push('/reports') },
    { id: 'nav-integrity', label: t('nav.integrity'), action: () => router.push('/audit') },
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
          if (next) setQuery('')
          return next
        })
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

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      setIndex((i) => Math.min(i + 1, filtered.length - 1))
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      setIndex((i) => Math.max(i - 1, 0))
    } else if (e.key === 'Enter') {
      e.preventDefault()
      const item = filtered[index]
      if (item) {
        item.action()
        setOpen(false)
      }
    }
  }

  if (!open) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-[20vh]"
      onClick={() => setOpen(false)}
      role="dialog"
      aria-modal="true"
      aria-label={t('commandPalette.title')}
    >
      <div
        className="w-full max-w-lg overflow-hidden rounded-lg border border-[var(--color-edge)] bg-[var(--color-panel)] shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="border-b border-[var(--color-edge)] px-4 py-3">
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t('commandPalette.placeholder')}
            className="w-full bg-transparent text-sm text-[var(--color-ink)] outline-none placeholder:text-[var(--color-muted)]"
            aria-label={t('commandPalette.placeholder')}
          />
        </div>
        <ul className="max-h-80 overflow-auto py-2" role="listbox">
          {filtered.length === 0 && (
            <li className="px-4 py-3 text-sm text-[var(--color-muted)]">Aucun résultat</li>
          )}
          {filtered.map((item, i) => (
            <li
              key={item.id}
              role="option"
              aria-selected={i === index}
              className={`mx-2 flex cursor-pointer items-center justify-between rounded px-3 py-2 text-sm transition-colors ${
                i === index
                  ? 'bg-sky-500/15 text-sky-200'
                  : 'text-[var(--color-muted)] hover:bg-white/5 hover:text-[var(--color-ink)]'
              }`}
              onClick={() => {
                item.action()
                setOpen(false)
              }}
              onMouseEnter={() => setIndex(i)}
            >
              <span>{item.label}</span>
              {item.shortcut && (
                <kbd className="rounded bg-white/10 px-1.5 py-0.5 text-[10px] font-mono text-[var(--color-muted)]">
                  {item.shortcut}
                </kbd>
              )}
            </li>
          ))}
        </ul>
        <div className="flex items-center gap-4 border-t border-[var(--color-edge)] px-4 py-2 text-[10px] text-[var(--color-muted)]">
          <span>{t('commandPalette.shortcut')} pour ouvrir</span>
          <span className="ml-auto">↑↓ pour naviguer · ↵ pour valider · Esc pour fermer</span>
        </div>
      </div>
    </div>
  )
}
