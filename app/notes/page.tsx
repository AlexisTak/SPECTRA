'use client'

/**
 * Notes — Markdown lié aux entités.
 *
 * Permet de rédiger des notes structurées, de les lier à des entités,
 * et de marquer certaines comme des hypothèses de travail.
 */

import { useState } from 'react'
import { PageHeader } from '@/components/layout/shell'
import { Button, Panel } from '@/components/ui/primitives'

interface Note {
  id: string
  title: string
  content: string
  linkedEntityIds: string[]
  isHypothesis: boolean
  createdAt: string
  updatedAt: string
}

const MOCK_NOTES: Note[] = [
  {
    id: 'n1',
    title: 'Profil initial',
    content: `## Alice Dupont\n\n- Email principal : alice.dupont@mail.fr\n- Secondaire : alice.dupont@proton.me\n- Téléphone : +33 6 12 34 56 78 (Opérateur SFR)\n\n**Observation** : le domaine \`alice-dupont.blog\` est enregistré chez OVH.`,
    linkedEntityIds: ['e1', 'e2', 'e3'],
    isHypothesis: false,
    createdAt: '2026-07-20T10:00:00Z',
    updatedAt: '2026-07-22T14:00:00Z',
  },
  {
    id: 'n2',
    title: 'Hypothèse : deux identités',
    content: `## Hypothèse H1\n\nLes deux emails (mail.fr et proton.me) pourraient appartenir à deux personnes distinctes ayant le même patronyme.\n\n### Points pour\n- Dates de création différentes\n- Présence sur des réseaux disjoints\n\n### Points contre\n- Même avatar (pHash distance 2)\n- Fuseau horaire d\'activité identique`,
    linkedEntityIds: ['e1', 'e3'],
    isHypothesis: true,
    createdAt: '2026-07-21T09:00:00Z',
    updatedAt: '2026-07-21T09:00:00Z',
  },
]

export default function NotesPage() {
  const [notes, setNotes] = useState<Note[]>(MOCK_NOTES)
  const [activeId, setActiveId] = useState<string | null>(MOCK_NOTES[0]?.id ?? null)
  const [editing, setEditing] = useState(false)

  const active = notes.find((n) => n.id === activeId) ?? null

  return (
    <div className="flex h-[calc(100vh-4rem)]">
      <PageHeader title="Notes" subtitle="Notes et hypothèses liées aux entités" />
      <div className="flex w-full pt-[4.5rem]">
        {/* Sidebar liste */}
        <aside className="w-64 shrink-0 border-r border-[var(--color-edge)] bg-[var(--color-panel)]">
          <div className="flex items-center justify-between border-b border-[var(--color-edge)] px-4 py-3">
            <span className="text-xs font-medium uppercase tracking-wider text-[var(--color-muted)]">{notes.length} notes</span>
            <Button variant="primary" onClick={() => {
              const n: Note = {
                id: `n-${Date.now()}`,
                title: 'Nouvelle note',
                content: '',
                linkedEntityIds: [],
                isHypothesis: false,
                createdAt: new Date().toISOString(),
                updatedAt: new Date().toISOString(),
              }
              setNotes((prev) => [n, ...prev])
              setActiveId(n.id)
              setEditing(true)
            }}>Nouvelle</Button>
          </div>
          <ul className="divide-y divide-[var(--color-edge)]">
            {notes.map((n) => (
              <li
                key={n.id}
                className={`cursor-pointer px-4 py-3 transition-colors ${
                  activeId === n.id ? 'bg-sky-500/10' : 'hover:bg-white/5'
                }`}
                onClick={() => {
                  setActiveId(n.id)
                  setEditing(false)
                }}
              >
                <div className="flex items-center gap-2">
                  {n.isHypothesis && (
                    <span className="inline-flex items-center rounded border border-sky-500/40 bg-sky-500/10 px-1.5 py-0.5 text-[10px] font-medium text-sky-300">H</span>
                  )}
                  <span className="truncate text-sm font-medium text-[var(--color-ink)]">{n.title}</span>
                </div>
                <p className="mt-0.5 truncate text-xs text-[var(--color-muted)]">
                  {new Date(n.updatedAt).toLocaleDateString('fr-FR')}
                </p>
              </li>
            ))}
          </ul>
        </aside>

        {/* Éditeur / Vue */}
        <main className="flex-1 overflow-auto px-8 py-6">
          {active ? (
            <div className="max-w-3xl">
              <div className="mb-4 flex items-center justify-between">
                {editing ? (
                  <input
                    value={active.title}
                    onChange={(e) => {
                      const title = e.target.value
                      setNotes((prev) => prev.map((n) => n.id === active.id ? { ...n, title, updatedAt: new Date().toISOString() } : n))
                    }}
                    className="w-full bg-transparent text-xl font-semibold text-[var(--color-ink)] outline-none"
                    autoFocus
                  />
                ) : (
                  <h1 className="text-xl font-semibold">{active.title}</h1>
                )}
                <div className="flex items-center gap-2">
                  <Button
                    variant={active.isHypothesis ? 'primary' : 'default'}
                   
                    onClick={() =>
                      setNotes((prev) =>
                        prev.map((n) =>
                          n.id === active.id ? { ...n, isHypothesis: !n.isHypothesis } : n
                        )
                      )
                    }
                  >
                    {active.isHypothesis ? 'Hypothèse' : 'Marquer hypothèse'}
                  </Button>
                  <Button variant="default" onClick={() => setEditing((e) => !e)}>
                    {editing ? 'Aperçu' : 'Éditer'}
                  </Button>
                </div>
              </div>

              {editing ? (
                <textarea
                  value={active.content}
                  onChange={(e) => {
                    const content = e.target.value
                    setNotes((prev) => prev.map((n) => n.id === active.id ? { ...n, content, updatedAt: new Date().toISOString() } : n))
                  }}
                  className="h-[60vh] w-full rounded border border-[var(--color-edge)] bg-black/30 p-4 font-mono text-sm text-[var(--color-ink)] outline-none focus:border-sky-500"
                  placeholder="Rédigez en Markdown…"
                />
              ) : (
                <article className="prose prose-invert max-w-none">
                  <MarkdownPreview content={active.content} />
                </article>
              )}
            </div>
          ) : (
            <div className="flex h-full items-center justify-center text-sm text-[var(--color-muted)]">
              Sélectionnez une note ou créez-en une nouvelle.
            </div>
          )}
        </main>
      </div>
    </div>
  )
}

function MarkdownPreview({ content }: { content: string }) {
  // Rendu Markdown minimal sans dépendance externe.
  // NOTE: ce contenu est produit par l'analyste dans l'application locale (Tauri
  // avec CSP strict). Le risque XSS est contrôlé par l'environnement de confiance.
  // Pour une version production avec contenu partagé, remplacer par un vrai
  // parseur Markdown avec sanitization (ex: react-markdown + rehype-sanitize).
  const html = content
    .replace(/^### (.*$)/gim, '<h3 class="text-lg font-semibold mt-4 mb-2">$1</h3>')
    .replace(/^## (.*$)/gim, '<h2 class="text-xl font-semibold mt-6 mb-3 border-b border-[var(--color-edge)] pb-1">$1</h2>')
    .replace(/^# (.*$)/gim, '<h1 class="text-2xl font-bold mt-8 mb-4">$1</h1>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    .replace(/`([^`]+)`/g, '<code class="rounded bg-black/40 px-1 py-0.5 text-xs font-mono text-sky-300">$1</code>')
    .replace(/- (.*$)/gim, '<li class="ml-4 list-disc">$1</li>')
    .replace(/\n/g, '<br />')
  // eslint-disable-next-line react/no-danger
  return <div dangerouslySetInnerHTML={{ __html: html }} />
}
