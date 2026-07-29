'use client'

/**
 * Aide des raccourcis clavier (Shift + ?).
 *
 * Affiche une modale avec tous les raccourcis organisés par scope.
 * Accessibilité : piège à focus, aria-modal, ESC pour fermer.
 */

import { useEffect, useRef, useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'

interface ShortcutGroup {
  scope: string
  items: { keys: string; desc: string }[]
}

export function ShortcutsHelp() {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const modalRef = useRef<HTMLDivElement>(null)

  const groups: ShortcutGroup[] = [
    {
      scope: t('shortcuts.global'),
      items: [
        { keys: 'Ctrl + K', desc: t('shortcuts.openPalette') },
        { keys: 'Shift + ?', desc: t('shortcuts.title') },
        { keys: 'Esc', desc: t('shortcuts.closeModal') },
      ],
    },
    {
      scope: t('shortcuts.navigation'),
      items: [
        { keys: 'Ctrl + Shift + F', desc: t('shortcuts.focusSearch') },
        { keys: 'Ctrl + S', desc: t('shortcuts.save') },
      ],
    },
    {
      scope: t('shortcuts.graph'),
      items: [
        { keys: 'Ctrl + +', desc: t('shortcuts.zoomIn') },
        { keys: 'Ctrl + -', desc: t('shortcuts.zoomOut') },
        { keys: 'Ctrl + 0', desc: t('shortcuts.fitView') },
        { keys: 'L', desc: t('shortcuts.toggleLabels') },
      ],
    },
  ]

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === '?' && e.shiftKey) {
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

  // Piège à focus : garde le focus dans la modale
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key !== 'Tab' || !modalRef.current) return
      const focusable = modalRef.current.querySelectorAll(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      )
      const first = focusable[0] as HTMLElement
      const last = focusable[focusable.length - 1] as HTMLElement
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault()
        last?.focus()
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault()
        first?.focus()
      }
    },
    []
  )

  if (!open) return null

  return (
    <div
      className="fixed inset-0 z-[60] flex items-center justify-center bg-black/60"
      onClick={() => setOpen(false)}
      role="dialog"
      aria-modal="true"
      aria-label={t('shortcuts.title')}
    >
      <div
        ref={modalRef}
        className="w-full max-w-lg overflow-hidden rounded-lg border border-[var(--color-edge)] bg-[var(--color-panel)] shadow-2xl"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        <div className="flex items-center justify-between border-b border-[var(--color-edge)] px-5 py-3">
          <h2 className="text-sm font-semibold">{t('shortcuts.title')}</h2>
          <button
            onClick={() => setOpen(false)}
            className="rounded px-2 py-1 text-xs text-[var(--color-muted)] hover:text-[var(--color-ink)]"
            aria-label={t('common.close')}
          >
            ESC
          </button>
        </div>

        <div className="max-h-[60vh] overflow-auto p-5">
          <div className="space-y-5">
            {groups.map((g) => (
              <div key={g.scope}>
                <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-[var(--color-muted)]">
                  {g.scope}
                </h3>
                <div className="space-y-1">
                  {g.items.map((item) => (
                    <div
                      key={item.keys}
                      className="flex items-center justify-between rounded border border-[var(--color-edge)] bg-[var(--color-surface)] px-3 py-2"
                    >
                      <span className="text-sm">{item.desc}</span>
                      <div className="flex items-center gap-1">
                        {item.keys.split(' + ').map((part, i) => (
                          <span key={i} className="flex items-center gap-1">
                            <kbd className="rounded border border-[var(--color-edge)] bg-[var(--color-panel)] px-1.5 py-0.5 text-xs font-mono text-[var(--color-muted)]">
                              {part.trim()}
                            </kbd>
                            {i < item.keys.split(' + ').length - 1 && (
                              <span className="text-[var(--color-muted)]">+</span>
                            )}
                          </span>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
