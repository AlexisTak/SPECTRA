'use client'

/**
 * Paramètres applicatifs.
 *
 * - Langue (FR / EN), persistée localement
 * - Raccourcis clavier réellement implémentés
 * - Informations de version
 *
 * La coquille applicative est fournie par `app/layout.tsx` : cette page ne
 * doit PAS remonter un second `AppShell` (double barre latérale).
 */

import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { PageHeader } from '@/components/layout/shell'
import { Panel, Select } from '@/components/ui/primitives'
import { setStoredLanguage } from '../i18n'
import { Globe, Keyboard, Info } from 'lucide-react'

/** Version de repli : doit rester alignée sur `src-tauri/tauri.conf.json`. */
const FALLBACK_VERSION = '0.2.0'

export default function SettingsPage() {
  const { t, i18n } = useTranslation()
  const [version, setVersion] = useState(FALLBACK_VERSION)

  useEffect(() => {
    // Hors Tauri (navigateur), l'import échoue ou l'IPC est absente :
    // on garde la version de repli sans casser la page.
    let cancelled = false
    import('@tauri-apps/api/app')
      .then((m) => m.getVersion())
      .then((v) => {
        if (!cancelled && v) setVersion(v)
      })
      .catch(() => {})
    return () => {
      cancelled = true
    }
  }, [])

  const shortcuts = [
    {
      scope: t('shortcuts.global'),
      items: [
        { keys: 'Ctrl + K', desc: t('shortcuts.openPalette') },
        { keys: 'Shift + ?', desc: t('shortcuts.title') },
        { keys: 'Esc', desc: t('shortcuts.closeModal') },
      ],
    },
  ]

  const changeLanguage = (lng: string) => {
    void i18n.changeLanguage(lng)
    setStoredLanguage(lng)
  }

  return (
    <>
      <PageHeader title={t('nav.settings')} />
      <div className="space-y-6 px-8 py-5">
        {/* Langue */}
        <Panel title={t('settings.language')}>
          <div className="flex items-center gap-3">
            <Globe className="h-4 w-4 text-[var(--color-muted)]" />
            <Select
              value={i18n.language}
              onChange={(e) => changeLanguage(e.target.value)}
              className="w-40"
              aria-label={t('settings.language')}
            >
              <option value="fr">Français</option>
              <option value="en">English</option>
            </Select>
            <span className="text-sm text-[var(--color-muted)]">
              {t('settings.languageHint')}
            </span>
          </div>
        </Panel>

        {/* Raccourcis */}
        <Panel title={t('shortcuts.title')}>
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-xs text-[var(--color-muted)]">
              <Keyboard className="h-3 w-3" />
              <span>{t('settings.shortcutsHint')}</span>
            </div>
            {shortcuts.map((group) => (
              <div key={group.scope}>
                <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-[var(--color-muted)]">
                  {group.scope}
                </h3>
                <div className="grid grid-cols-1 gap-1 md:grid-cols-2">
                  {group.items.map((s) => (
                    <div
                      key={s.keys}
                      className="flex items-center justify-between rounded border border-[var(--color-edge)] bg-[var(--color-surface)] px-3 py-2"
                    >
                      <span className="text-sm">{s.desc}</span>
                      <kbd className="rounded border border-[var(--color-edge)] bg-[var(--color-panel)] px-1.5 py-0.5 text-xs font-mono text-[var(--color-muted)]">
                        {s.keys}
                      </kbd>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </Panel>

        {/* À propos */}
        <Panel title={t('settings.about')}>
          <div className="flex items-start gap-3">
            <Info className="mt-0.5 h-4 w-4 text-[var(--color-muted)]" />
            <div className="space-y-2 text-sm">
              <p>
                <strong>SPECTRA</strong> — {t('app.tagline')}
              </p>
              <p className="text-[var(--color-muted)]">
                {t('app.version')} : {version}
              </p>
              <p className="text-[var(--color-muted)]">
                {t('settings.license')} : AGPL-3.0-only
              </p>
              <p className="text-xs text-[var(--color-muted)]">
                © 2026 SPECTRA Contributors · {t('settings.noTelemetry')}
              </p>
            </div>
          </div>
        </Panel>
      </div>
    </>
  )
}
