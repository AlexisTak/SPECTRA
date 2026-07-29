/**
 * Générateur de graphe synthétique réaliste pour le benchmark.
 *
 * CLAUDE.md §3 impose une **loi de puissance**, pas un graphe aléatoire
 * uniforme : un graphe uniforme est trivialement plus facile à afficher
 * (degrés homogènes, pas de hub), et donnerait un benchmark trop optimiste.
 *
 * L'implémentation utilise l'attachement préférentiel de Barabási–Albert :
 * la probabilité qu'un nouveau nœud se connecte à un nœud existant est
 * proportionnelle au degré de celui-ci. On obtient naturellement quelques hubs
 * très connectés et une longue traîne de nœuds peu connectés — la topologie
 * réelle d'un dossier OSINT (un compte central, des dizaines de pivots).
 *
 * Complexité : O(E). Une sélection pondérée naïve (parcours de tous les nœuds
 * à chaque tirage) serait O(N·E) — soit ~7,5 milliards d'opérations pour
 * 50k nœuds / 150k arêtes, ce qui rend le benchmark inexécutable.
 * L'astuce est la « liste de répétition » : chaque nœud y apparaît autant de
 * fois que son degré, donc un tirage uniforme dans cette liste est déjà
 * proportionnel au degré, en O(1).
 */

import Graph from 'graphology'

export interface GeneratedGraphStats {
  nodes: number
  edges: number
  maxDegree: number
  avgDegree: number
  /** Nombre de nœuds concentrant 50 % des arêtes — petit = topologie en hubs. */
  hubCount: number
}

/**
 * Génère un graphe sans échelle par attachement préférentiel.
 *
 * @param nodeCount Nombre de nœuds souhaité.
 * @param edgeCount Nombre d'arêtes souhaité (doit être >= nodeCount - 1).
 * @param seed Graine pour la reproductibilité du benchmark.
 */
export function generateScaleFreeGraph(
  nodeCount: number,
  edgeCount: number,
  seed = 42,
): Graph {
  if (nodeCount < 2) {
    throw new Error('nodeCount doit être >= 2')
  }

  const rng = makeRng(seed)
  const graph = new Graph({ type: 'undirected' })

  // Liste de répétition : chaque nœud y figure `degré` fois. Un tirage
  // uniforme dedans est donc pondéré par le degré, en O(1).
  const repeated: string[] = []

  // Amorce : deux nœuds reliés, sinon le premier tirage n'a rien à viser.
  graph.addNode('n0', { x: 0, y: 0, size: 1, label: 'n0' })
  graph.addNode('n1', { x: 1, y: 0, size: 1, label: 'n1' })
  graph.addUndirectedEdge('n0', 'n1')
  repeated.push('n0', 'n1')

  // Chaque nouveau nœud apporte `m` arêtes vers des nœuds existants,
  // choisis proportionnellement à leur degré.
  const m = Math.max(1, Math.round(edgeCount / nodeCount))

  for (let i = 2; i < nodeCount; i++) {
    const node = `n${i}`
    graph.addNode(node, {
      x: rng() * 1000,
      y: rng() * 1000,
      size: 1,
      label: node,
    })

    const targets = new Set<string>()
    // Borne les tentatives : sur un petit graphe naissant, les collisions
    // sont fréquentes et une boucle non bornée ne terminerait pas.
    let attempts = 0
    while (targets.size < m && attempts < m * 10) {
      attempts++
      const target = repeated[Math.floor(rng() * repeated.length)]
      if (target !== undefined && target !== node && !graph.hasEdge(node, target)) {
        targets.add(target)
      }
    }

    for (const target of targets) {
      graph.addUndirectedEdge(node, target)
      repeated.push(node, target)
    }
  }

  // Complète jusqu'au nombre d'arêtes voulu, toujours par attachement
  // préférentiel, pour atteindre exactement la densité demandée.
  let guard = 0
  const maxGuard = edgeCount * 20
  while (graph.size < edgeCount && guard < maxGuard) {
    guard++
    const a = repeated[Math.floor(rng() * repeated.length)]
    const b = repeated[Math.floor(rng() * repeated.length)]
    if (a !== undefined && b !== undefined && a !== b && !graph.hasEdge(a, b)) {
      graph.addUndirectedEdge(a, b)
      repeated.push(a, b)
    }
  }

  return graph
}

/** Statistiques de topologie, pour vérifier que la loi de puissance est bien là. */
export function describeGraph(graph: Graph): GeneratedGraphStats {
  let maxDegree = 0
  const degrees: number[] = []

  graph.forEachNode((node) => {
    const d = graph.degree(node)
    degrees.push(d)
    if (d > maxDegree) maxDegree = d
  })

  degrees.sort((a, b) => b - a)
  const totalDegree = degrees.reduce((s, d) => s + d, 0)

  // Combien de nœuds (les plus connectés) concentrent la moitié des arêtes ?
  let running = 0
  let hubCount = 0
  for (const d of degrees) {
    running += d
    hubCount++
    if (running >= totalDegree / 2) break
  }

  return {
    nodes: graph.order,
    edges: graph.size,
    maxDegree,
    avgDegree: graph.order > 0 ? totalDegree / graph.order : 0,
    hubCount,
  }
}

/**
 * PRNG déterministe (mulberry32). `Math.random()` n'est pas reproductible,
 * or un benchmark doit pouvoir être rejoué à l'identique.
 */
function makeRng(seed: number): () => number {
  let a = seed >>> 0
  return () => {
    a += 0x6d2b79f5
    let t = a
    t = Math.imul(t ^ (t >>> 15), t | 1)
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}
