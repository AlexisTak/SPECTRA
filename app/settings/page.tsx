'use client'

/**
 * Paramètres applicatifs.
 *
 * - Langue (FR / EN)
 * - Raccourcis clavier référencés
 * - Informations de version
 */

import React, { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { AppShell, PageHeader } from '@/components/layout/shell'
import { Panel, Select } from '@/components/ui/primitives'
import { Globe, Keyboard, Info } from 'lucide-react'

export default function SettingsPage() {
  const { t, i18n } = useTranslation()
  const [version, setVersion] = useState('0.2.0')

  useEffect(() => {
    // Récupère la version depuis Tauri si disponible
    if (typeof window !== 'undefined' && (window as any).__TAURI__) {
      import('@tauri-apps/api/app').then((m) => {
        m.getVersion().then(setVersion).catch(() => {})
      })
    }
  }, [])

  const shortcuts = [
    { scope: t('shortcuts.global'), items: [
      { keys: 'Ctrl+K', desc: t('shortcuts.openPalette') },
      { keys: 'Esc', desc: t('shortcuts.closeModal') },
    ]},
    { scope: t('shortcuts.navigation'), items: [
      { keys: 'Ctrl+Shift+F', desc: t('shortcuts.focusSearch') },
    ]},
    { scope: t('shortcuts.graph'), items: [
      { keys: 'Ctrl++', desc: t('shortcuts.zoomIn') },
      { keys: 'Ctrl+-', desc: t('shortcuts.zoomOut') },
      { keys: 'Ctrl+0', desc: t('shortcuts.fitView') },
      { keys: 'L', desc: t('shortcuts.toggleLabels') },
    ]},
  ]

  return (
    <AppShell>
      <PageHeader title="Paramètres" />
      <div className="space-y-6 px-8 py-5">

        {/* Langue */}
        <Panel title="Langue">
          <div className="flex items-center gap-3">
            <Globe className="h-4 w-4 text-[var(--color-muted)]" />
            <Select
              value={i18n.language}
              onChange={(e) => i18n.changeLanguage(e.target.value)}
              className="w-40"
            >
              <option value="fr">Français</option>
              <option value="en">English</option>
            </Select>
            <span className="text-sm text-[var(--color-muted)]">
              {i18n.language === 'fr' ? 'Interface en français' : 'English interface'}
            </span>
          </div>
        </Panel>

        {/* Raccourcis */}
        <Panel title={t('shortcuts.title')}>
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-xs text-[var(--color-muted)]">
              <Keyboard className="h-3 w-3" />
              <span>Shift + ? pour ouvrir depuis n'importe où</span>
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
        <Panel title="À propos">
          <div className="flex items-start gap-3">
            <Info className="mt-0.5 h-4 w-4 text-[var(--color-muted)]" />
            <div className="space-y-2 text-sm">
              <p><strong>SPECTRA</strong> — {t('app.tagline')}</p>
              <p className="text-[var(--color-muted)]">{t('app.version')} : {version}</p>
              <p className="text-[var(--color-muted)]">Licence : AGPL-3.0-only</p>
              <p className="text-xs text-[var(--color-muted)]">
                © 2026 SPECTRA Contributors · Aucune télémétrie · 100 % offline
              </p>
            </div>
          </div>
        </Panel>
      </div>
    </AppShell>
  )
}
