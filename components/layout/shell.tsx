'use client'

/**
 * Coquille applicative : barre latérale de navigation et zone de contenu.
 */

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'

const useNav = () => {
  const { t } = useTranslation()
  return [
    { href: '/', label: t('nav.dashboard') },
    { href: '/cases', label: t('nav.cases') },
    { href: '/osint', label: t('nav.osint'), highlight: true },
    { href: '/graph', label: t('nav.graph') },
    { href: '/timeline', label: 'Timeline' },
    { href: '/table', label: 'Table' },
    { href: '/map', label: 'Carte' },
    { href: '/notes', label: 'Notes' },
    { href: '/ach', label: 'ACH' },
    { href: '/search', label: t('nav.search') },
    { href: '/reports', label: t('nav.reports') },
    { href: '/audit', label: t('nav.integrity') },
  ] as const
}

export function AppShell({ children }: { children: React.ReactNode }) {
  const { t } = useTranslation()
  const pathname = usePathname()
  const NAV = useNav()

  return (
    <div className="flex min-h-screen">
      <aside className="flex w-56 shrink-0 flex-col border-r border-[var(--color-edge)] bg-[var(--color-panel)]">
        <div className="border-b border-[var(--color-edge)] px-5 py-4">
          <Link href="/" className="block">
            <p className="text-sm font-semibold tracking-tight">{t('app.name')}</p>
            <p className="text-xs text-[var(--color-muted)]">
              {t('app.tagline')}
            </p>
          </Link>
        </div>

        <nav className="flex flex-col gap-0.5 p-3">
          {NAV.map((item) => {
            const active =
              item.href === '/'
                ? pathname === '/'
                : pathname.startsWith(item.href)
            return (
              <Link
                key={item.href}
                href={item.href}
                className={cn(
                  'relative flex items-center justify-between rounded px-3 py-2 text-sm transition-colors',
                  active
                    ? 'bg-sky-500/15 text-sky-200'
                    : 'text-[var(--color-muted)] hover:bg-white/5 hover:text-[var(--color-ink)]',
                  (item as any).highlight && 'bg-gradient-to-r from-amber-500/10 to-rose-500/10 border border-amber-500/20',
                )}
              >
                {item.label}
                {(item as any).highlight && (
                  <span className="ml-2 rounded bg-gradient-to-r from-amber-500 to-rose-500 px-1.5 py-0.5 text-[10px] font-bold text-white">
                    {t('nav.new')}
                  </span>
                )}
              </Link>
            )
          })}
        </nav>

        <div className="mt-auto border-t border-[var(--color-edge)] px-5 py-3">
          <p className="text-xs text-[var(--color-muted)]">
            {t('app.footer')}
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
