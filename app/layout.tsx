import type { Metadata } from 'next'
import { AppShell } from '@/components/layout/shell'
import { I18nProvider } from './components/I18nProvider'
import { CommandPalette } from './components/CommandPalette'
import { ShortcutsHelp } from './components/ShortcutsHelp'
import './globals.css'

export const metadata: Metadata = {
  title: 'SPECTRA — Investigation OSINT',
  description: "Plateforme d'investigation OSINT 100 % locale",
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="fr">
      <body className="min-h-screen antialiased">
        <I18nProvider>
          <AppShell>{children}</AppShell>
          <CommandPalette />
          <ShortcutsHelp />
        </I18nProvider>
      </body>
    </html>
  )
}
