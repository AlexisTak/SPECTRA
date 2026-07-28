import type { Metadata } from 'next'
import { AppShell } from '@/components/layout/shell'
import './globals.css'

export const metadata: Metadata = {
  title: 'Cekarna — enquêtes',
  description: "Plateforme d'investigation locale",
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="fr">
      <body className="min-h-screen antialiased">
        <AppShell>{children}</AppShell>
      </body>
    </html>
  )
}
