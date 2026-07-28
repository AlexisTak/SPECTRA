'use client'

/**
 * Coquille applicative : barre latérale de navigation et zone de contenu.
 */

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { cn } from '@/lib/utils'

const NAV = [
  { href: '/', label: 'Tableau de bord' },
  { href: '/cases', label: 'Dossiers' },
  { href: '/graph', label: 'Graphe' },
  { href: '/search', label: 'Recherche' },
  { href: '/reports', label: 'Rapports' },
  { href: '/audit', label: 'Intégrité' },
] as const

export function AppShell({ children }: { children: React.ReactNode }) {
  const pathname = usePathname()

  return (
    <div className="flex min-h-screen">
      <aside className="flex w-56 shrink-0 flex-col border-r border-[var(--color-edge)] bg-[var(--color-panel)]">
        <div className="border-b border-[var(--color-edge)] px-5 py-4">
          <Link href="/" className="block">
            <p className="text-sm font-semibold tracking-tight">Cekarna</p>
            <p className="text-xs text-[var(--color-muted)]">
              Investigation locale
            </p>
          </Link>
        </div>

        <nav className="flex flex-col gap-0.5 p-3">
          {NAV.map((item) => {
            // `/` ne doit être actif que sur l'accueil exact, sinon il le
            // resterait sur toutes les routes.
            const active =
              item.href === '/'
                ? pathname === '/'
                : pathname.startsWith(item.href)
            return (
              <Link
                key={item.href}
                href={item.href}
                className={cn(
                  'rounded px-3 py-2 text-sm transition-colors',
                  active
                    ? 'bg-sky-500/15 text-sky-200'
                    : 'text-[var(--color-muted)] hover:bg-white/5 hover:text-[var(--color-ink)]',
                )}
              >
                {item.label}
              </Link>
            )
          })}
        </nav>

        <div className="mt-auto border-t border-[var(--color-edge)] px-5 py-3">
          <p className="text-xs text-[var(--color-muted)]">
            Données locales uniquement
          </p>
        </div>
      </aside>

      <main className="flex-1 overflow-x-hidden">{children}</main>
    </div>
  )
}

export function PageHeader({
  title,
  subtitle,
  action,
}: {
  title: string
  subtitle?: string
  action?: React.ReactNode
}) {
  return (
    <header className="flex items-start justify-between gap-4 border-b border-[var(--color-edge)] px-8 py-5">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">{title}</h1>
        {subtitle && (
          <p className="mt-0.5 text-sm text-[var(--color-muted)]">{subtitle}</p>
        )}
      </div>
      {action}
    </header>
  )
}
