/**
 * Primitives d'interface.
 *
 * Densité d'information élevée mais respirable, mode sombre par défaut
 * (CLAUDE.md §6). Les couleurs de statut ne sont pas décoratives : elles
 * portent du sens métier et doivent rester cohérentes d'un écran à l'autre.
 */

import { cn } from '@/lib/utils'

export function Panel({
  title,
  action,
  children,
  className,
}: {
  title?: string
  action?: React.ReactNode
  children: React.ReactNode
  className?: string
}) {
  return (
    <section
      className={cn(
        'rounded-lg border border-[var(--color-edge)] bg-[var(--color-panel)]',
        className,
      )}
    >
      {(title || action) && (
        <div className="flex items-center justify-between border-b border-[var(--color-edge)] px-5 py-3">
          {title && (
            <h2 className="text-xs font-medium uppercase tracking-wider text-[var(--color-muted)]">
              {title}
            </h2>
          )}
          {action}
        </div>
      )}
      <div className="p-5">{children}</div>
    </section>
  )
}

const STATUS_TONE: Record<string, string> = {
  ouvert: 'border-sky-500/40 bg-sky-500/10 text-sky-300',
  en_cours: 'border-amber-500/40 bg-amber-500/10 text-amber-300',
  transmis: 'border-violet-500/40 bg-violet-500/10 text-violet-300',
  clos: 'border-slate-500/40 bg-slate-500/10 text-slate-300',
  suspect: 'border-rose-500/40 bg-rose-500/10 text-rose-300',
  victime: 'border-emerald-500/40 bg-emerald-500/10 text-emerald-300',
  temoin: 'border-sky-500/40 bg-sky-500/10 text-sky-300',
  inconnu: 'border-slate-500/40 bg-slate-500/10 text-slate-300',
  basse: 'border-slate-500/40 bg-slate-500/10 text-slate-300',
  normale: 'border-sky-500/40 bg-sky-500/10 text-sky-300',
  haute: 'border-amber-500/40 bg-amber-500/10 text-amber-300',
  urgente: 'border-rose-500/40 bg-rose-500/10 text-rose-300',
}

const STATUS_LABEL: Record<string, string> = {
  ouvert: 'Ouvert',
  en_cours: 'En cours',
  transmis: 'Transmis',
  clos: 'Clos',
  suspect: 'Suspect',
  victime: 'Victime',
  temoin: 'Témoin',
  inconnu: 'Inconnu',
  basse: 'Basse',
  normale: 'Normale',
  haute: 'Haute',
  urgente: 'Urgente',
}

export function Badge({ value }: { value: string }) {
  return (
    <span
      className={cn(
        'inline-flex items-center rounded border px-2 py-0.5 text-xs font-medium',
        STATUS_TONE[value] ?? 'border-slate-500/40 bg-slate-500/10 text-slate-300',
      )}
    >
      {STATUS_LABEL[value] ?? value}
    </span>
  )
}

export function Button({
  variant = 'default',
  className,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: 'default' | 'primary' | 'danger'
}) {
  return (
    <button
      {...props}
      className={cn(
        'inline-flex items-center gap-2 rounded border px-3 py-1.5 text-sm transition-colors',
        'disabled:cursor-not-allowed disabled:opacity-50',
        variant === 'primary' &&
          'border-sky-500/50 bg-sky-500/15 text-sky-200 hover:bg-sky-500/25',
        variant === 'danger' &&
          'border-rose-500/50 bg-rose-500/10 text-rose-300 hover:bg-rose-500/20',
        variant === 'default' &&
          'border-[var(--color-edge)] text-[var(--color-ink)] hover:border-[var(--color-accent)] hover:text-[var(--color-accent)]',
        className,
      )}
    />
  )
}

export function Field({
  label,
  hint,
  children,
}: {
  label: string
  hint?: string
  children: React.ReactNode
}) {
  return (
    <label className="flex flex-col gap-1.5">
      <span className="text-xs font-medium text-[var(--color-muted)]">
        {label}
      </span>
      {children}
      {hint && <span className="text-xs text-[var(--color-muted)]">{hint}</span>}
    </label>
  )
}

const CONTROL_CLASS =
  'rounded border border-[var(--color-edge)] bg-[var(--color-surface)] px-3 py-1.5 text-sm ' +
  'outline-none transition-colors focus:border-[var(--color-accent)]'

export function Input(props: React.InputHTMLAttributes<HTMLInputElement>) {
  return <input {...props} className={cn(CONTROL_CLASS, props.className)} />
}

export function Textarea(
  props: React.TextareaHTMLAttributes<HTMLTextAreaElement>,
) {
  return <textarea {...props} className={cn(CONTROL_CLASS, props.className)} />
}

export function Select(props: React.SelectHTMLAttributes<HTMLSelectElement>) {
  return <select {...props} className={cn(CONTROL_CLASS, props.className)} />
}

/**
 * Affichage d'erreur.
 *
 * L'erreur brute est toujours montrée. Un outil probatoire ne doit jamais
 * masquer un échec derrière un message générique rassurant (`audit.md`, P3-10).
 */
export function ErrorBox({ message }: { message: string }) {
  return (
    <div className="rounded border border-rose-500/40 bg-rose-500/10 p-3">
      <p className="mb-1 text-sm font-medium text-rose-300">Échec</p>
      <pre className="overflow-x-auto text-xs text-rose-200/90">{message}</pre>
    </div>
  )
}

export function EmptyState({
  title,
  hint,
  action,
}: {
  title: string
  hint?: string
  action?: React.ReactNode
}) {
  return (
    <div className="flex flex-col items-center gap-2 py-10 text-center">
      <p className="text-sm text-[var(--color-muted)]">{title}</p>
      {hint && <p className="max-w-md text-xs text-[var(--color-muted)]">{hint}</p>}
      {action}
    </div>
  )
}

export function formatDate(iso: string | null | undefined): string {
  if (!iso) return '—'
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '—'
  return d.toLocaleDateString('fr-FR', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
  })
}

export function formatDateTime(iso: string | null | undefined): string {
  if (!iso) return '—'
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '—'
  return d.toLocaleString('fr-FR', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}
