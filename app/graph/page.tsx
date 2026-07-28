'use client'

/**
 * Visualisation graphe d'un dossier.
 *
 * Composant purement client — Sigma nécessite WebGL2, indisponible en SSR.
 */

import dynamic from 'next/dynamic'

const GraphViewDynamic = dynamic(() => import('./graph-view'), {
  ssr: false,
  loading: () => <p className="p-8 text-sm text-[var(--color-muted)]">Chargement…</p>,
})

export default function GraphPage() {
  return <GraphViewDynamic />
}
