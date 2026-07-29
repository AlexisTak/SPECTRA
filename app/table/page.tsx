'use client'

/**
 * Vue table — exploration tabulaire des entités du dossier.
 *
 * Colonnes dynamiques, tri, filtrage, sélection en masse.
 * Porte la même donnée que le graphe, sous forme de lignes.
 */

import { useMemo, useState } from 'react'
import { PageHeader } from '@/components/layout/shell'
import { ErrorBox, Panel, Button } from '@/components/ui/primitives'

interface TableEntity {
  id: string
  kind: string
  canonicalValue: string
  displayLabel: string
  createdAt: string
  source: string
  confidence: string
}

const MOCK: TableEntity[] = [
  { id: 'e1', kind: 'Person', canonicalValue: 'alice.dupont@mail.fr', displayLabel: 'Alice Dupont', createdAt: '2026-07-20T10:00:00Z', source: ' Manuel', confidence: 'A1' },
  { id: 'e2', kind: 'Username', canonicalValue: 'alice_d', displayLabel: 'alice_d', createdAt: '2026-07-20T11:30:00Z', source: 'WhatsMyName', confidence: 'B2' },
  { id: 'e3', kind: 'EmailAddress', canonicalValue: 'alice.dupont@proton.me', displayLabel: 'alice.dupont@proton.me', createdAt: '2026-07-21T09:15:00Z', source: 'Email transform', confidence: 'C3' },
  { id: 'e4', kind: 'PhoneNumber', canonicalValue: '+33612345678', displayLabel: '+33 6 12 34 56 78', createdAt: '2026-07-21T14:00:00Z', source: 'PhoneInfoga', confidence: 'B2' },
  { id: 'e5', kind: 'Domain', canonicalValue: 'alice-dupont.blog', displayLabel: 'alice-dupont.blog', createdAt: '2026-07-22T08:00:00Z', source: 'DNS', confidence: 'A1' },
]

type SortKey = keyof TableEntity

export default function TablePage() {
  const [query, setQuery] = useState('')
  const [sortKey, setSortKey] = useState<SortKey>('createdAt')
  const [sortDesc, setSortDesc] = useState(true)
  const [selected, setSelected] = useState<Set<string>>(new Set())

  const filtered = useMemo(() => {
    let rows = MOCK.filter((r) => {
      if (!query) return true
      const q = query.toLowerCase()
      return (
        r.displayLabel.toLowerCase().includes(q) ||
        r.kind.toLowerCase().includes(q) ||
        r.canonicalValue.toLowerCase().includes(q)
      )
    })
    rows.sort((a, b) => {
      const av = a[sortKey]
      const bv = b[sortKey]
      const cmp = String(av).localeCompare(String(bv))
      return sortDesc ? -cmp : cmp
    })
    return rows
  }, [query, sortKey, sortDesc])

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDesc((d) => !d)
    } else {
      setSortKey(key)
      setSortDesc(true)
    }
  }

  const toggleSelect = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  const toggleAll = () => {
    if (selected.size === filtered.length) {
      setSelected(new Set())
    } else {
      setSelected(new Set(filtered.map((r) => r.id)))
    }
  }

  const headers: { key: SortKey; label: string }[] = [
    { key: 'displayLabel', label: 'Label' },
    { key: 'kind', label: 'Type' },
    { key: 'canonicalValue', label: 'Valeur canonique' },
    { key: 'source', label: 'Source' },
    { key: 'confidence', label: 'Confiance' },
    { key: 'createdAt', label: 'Date' },
  ]

  return (
    <div>
      <PageHeader title="Vue table" subtitle="Exploration tabulaire des entités" />
      <div className="px-8 py-6">
        <div className="mb-4 flex items-center gap-3">
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Filtrer…"
            className="w-full max-w-md rounded border border-[var(--color-edge)] bg-transparent px-3 py-2 text-sm text-[var(--color-ink)] placeholder:text-[var(--color-muted)] outline-none focus:border-sky-500"
          />
          {selected.size > 0 && (
            <span className="inline-flex items-center rounded border border-sky-500/40 bg-sky-500/10 px-2 py-0.5 text-xs font-medium text-sky-300">{selected.size} sélectionné(s)</span>
          )}
        </div>
        <Panel className="overflow-auto">
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="border-b border-[var(--color-edge)]">
                <th className="px-3 py-2">
                  <input
                    type="checkbox"
                    checked={filtered.length > 0 && selected.size === filtered.length}
                    onChange={toggleAll}
                    className="accent-sky-500"
                  />
                </th>
                {headers.map((h) => (
                  <th
                    key={h.key}
                    className="cursor-pointer select-none px-3 py-2 text-xs font-medium uppercase tracking-wider text-[var(--color-muted)] hover:text-[var(--color-ink)]"
                    onClick={() => toggleSort(h.key)}
                  >
                    <span className="flex items-center gap-1">
                      {h.label}
                      {sortKey === h.key && (
                        <span className="text-sky-400">{sortDesc ? '▼' : '▲'}</span>
                      )}
                    </span>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody className="divide-y divide-[var(--color-edge)]">
              {filtered.map((row) => (
                <tr
                  key={row.id}
                  className={selected.has(row.id) ? 'bg-sky-500/5' : 'hover:bg-white/5'}
                >
                  <td className="px-3 py-2">
                    <input
                      type="checkbox"
                      checked={selected.has(row.id)}
                      onChange={() => toggleSelect(row.id)}
                      className="accent-sky-500"
                    />
                  </td>
                  <td className="px-3 py-2 font-medium text-[var(--color-ink)]">{row.displayLabel}</td>
                  <td className="px-3 py-2">
                    <span className="inline-flex items-center rounded border border-slate-500/40 bg-slate-500/10 px-2 py-0.5 text-xs font-medium text-slate-300">{row.kind}</span>
                  </td>
                  <td className="px-3 py-2 font-mono text-xs text-[var(--color-muted)]">{row.canonicalValue}</td>
                  <td className="px-3 py-2 text-xs text-[var(--color-muted)]">{row.source}</td>
                  <td className="px-3 py-2">
                    <ConfidenceBadge code={row.confidence} />
                  </td>
                  <td className="px-3 py-2 text-xs text-[var(--color-muted)]">
                    {new Date(row.createdAt).toLocaleDateString('fr-FR')}
                  </td>
                </tr>
              ))}
              {filtered.length === 0 && (
                <tr>
                  <td colSpan={7} className="px-3 py-8 text-center text-sm text-[var(--color-muted)]">
                    Aucune entité ne correspond au filtre.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </Panel>
      </div>
    </div>
  )
}

function ConfidenceBadge({ code }: { code: string }) {
  const color =
    code.startsWith('A') ? 'bg-emerald-500/15 text-emerald-300' :
    code.startsWith('B') ? 'bg-sky-500/15 text-sky-300' :
    code.startsWith('C') ? 'bg-amber-500/15 text-amber-300' :
    'bg-rose-500/15 text-rose-300'
  return (
    <span className={`inline-flex rounded px-1.5 py-0.5 text-[10px] font-mono font-medium ${color}`}>
      {code}
    </span>
  )
}
