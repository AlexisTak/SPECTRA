/**
 * Layout ForceAtlas2 déporté hors du thread principal.
 *
 * CLAUDE.md §3 : « `graphology-layout-forceatlas2` en **Web Worker**
 * (jamais sur le thread principal) ». Le banc d'essai initial exécutait le
 * layout en synchrone : 5,8 s de gel complet de l'interface sur 50k nœuds.
 *
 * # Pourquoi un superviseur et pas un simple `assign()` dans un worker
 *
 * `FA2LayoutSupervisor` (fourni par la bibliothèque) tourne en **continu** :
 * il enchaîne les itérations et réécrit les positions dans le graphe à chaque
 * message du worker, jusqu'à `stop()`. Il n'a pas de notion de « fais N
 * itérations puis arrête-toi ».
 *
 * C'est en réalité le bon modèle pour une UI : l'analyste voit le graphe se
 * déplier progressivement au lieu de fixer un écran gelé pendant 6 secondes.
 * Mais cela impose de décider **nous-mêmes** quand arrêter. Deux critères ici :
 *
 * 1. un budget de temps (`maxDurationMs`), garde-fou dur ;
 * 2. la **stabilisation** : quand les nœuds ne bougent presque plus, continuer
 *    ne fait que consommer du CPU sans améliorer le placement.
 *
 * # Contrainte de sécurité
 *
 * La bibliothèque crée son worker depuis une URL `blob:`
 * (`URL.createObjectURL`). Une CSP `worker-src 'self'` le bloque
 * silencieusement. `tauri.conf.json` doit donc autoriser `worker-src blob:`
 * — c'est une ouverture volontaire et documentée, pas un oubli.
 */

import Graph from 'graphology'
import FA2LayoutSupervisor from 'graphology-layout-forceatlas2/worker'

/** Réglages retenus après l'étude de sensibilité (`docs/bench/README.md`). */
export const FA2_SETTINGS = {
  barnesHutOptimize: true,
  // Le défaut (0,5) coûte 39,9 s sur 50k, hors budget. 1,2 ramène à ~6 s
  // pour une perte de fidélité de placement acceptable à cette échelle.
  barnesHutTheta: 1.2,
  gravity: 1,
  scalingRatio: 10,
  slowDown: 1,
  adjustSizes: false,
  linLogMode: false,
  strongGravityMode: false,
} as const

export interface LayoutProgress {
  /** Millisecondes écoulées depuis le démarrage. */
  elapsedMs: number
  /** Déplacement moyen des nœuds échantillonnés depuis la dernière mesure. */
  movement: number
  /** Vrai quand le layout s'est arrêté (stabilisé ou budget épuisé). */
  done: boolean
  /** Raison de l'arrêt, renseignée uniquement quand `done` est vrai. */
  reason?: 'stabilized' | 'timeout' | 'cancelled'
}

export interface RunLayoutOptions {
  /** Budget maximal. Au-delà, on arrête quel que soit l'état. */
  maxDurationMs?: number
  /** Seuil de déplacement moyen en dessous duquel on considère stabilisé. */
  stabilityThreshold?: number
  /** Intervalle entre deux évaluations de la stabilité. */
  sampleIntervalMs?: number
  /** Appelé à chaque évaluation, pour alimenter une barre de progression. */
  onProgress?: (progress: LayoutProgress) => void
}

export interface LayoutHandle {
  /** Résolue quand le layout s'arrête. */
  readonly finished: Promise<LayoutProgress>
  /** Arrête le layout immédiatement. */
  cancel: () => void
}

/**
 * Lance FA2 sur une **copie détachée** du graphe, et ne recopie les positions
 * vers le graphe affiché qu'à intervalle contrôlé.
 *
 * # Pourquoi cette indirection
 *
 * Mesure du 2026-07-28 sur 50k nœuds, layout déjà en worker :
 *
 * | Configuration          | FPS moyen | Frame la + longue |
 * |------------------------|-----------|-------------------|
 * | sans Sigma attaché     | 94        | 33 ms             |
 * | avec Sigma attaché     | 24        | 1 017 ms          |
 *
 * Déporter le **calcul** ne suffit donc pas. `assignLayoutChanges` réécrit les
 * 50 000 attributs sur le thread principal à chaque message du worker, et
 * chaque écriture émet un événement graphology que Sigma écoute pour réindexer.
 * Le goulot n'est pas FA2 : c'est la réindexation, plusieurs fois par seconde.
 *
 * La parade : le superviseur travaille sur un graphe jumeau que personne
 * n'observe, et on recopie les positions vers le graphe rendu à une cadence
 * qu'on choisit (par défaut ~2 Hz). Sigma ne réindexe alors que 2 fois par
 * seconde au lieu de 10+.
 */
export function runLayoutDetached(
  displayGraph: Graph,
  options: RunLayoutOptions & {
    syncIntervalMs?: number
    /** Durée minimale avant de tester la stabilisation. */
    minDurationMs?: number
    /** Nœuds traités par lot lors des copies découpées. */
    chunkSize?: number
  } = {},
): LayoutHandle {
  const {
    maxDurationMs = 30_000,
    stabilityThreshold = 0.35,
    sampleIntervalMs = 400,
    syncIntervalMs = 500,
    // Le superviseur met un temps non nul à démarrer son worker. Sans plancher,
    // la première mesure voit un mouvement quasi nul et conclut à tort que le
    // layout a convergé (constaté : arrêt après 1,7 s sur un graphe non relaxé).
    minDurationMs = 2_000,
    chunkSize = 5_000,
    onProgress,
  } = options

  const started = performance.now()
  let supervisor: FA2LayoutSupervisor | undefined
  let stabilityTimer: ReturnType<typeof setInterval> | undefined
  let syncTimer: ReturnType<typeof setInterval> | undefined
  let settled = false
  let cancelledEarly = false

  let resolveFinished: (p: LayoutProgress) => void
  const finished = new Promise<LayoutProgress>((resolve) => {
    resolveFinished = resolve
  })

  // Graphe de travail : même topologie, mais aucun observateur — c'est ce qui
  // évite la réindexation de Sigma à chaque message du worker.
  const workGraph = new Graph({ type: 'undirected' })

  const nodes = displayGraph.nodes()
  const edges = displayGraph.edges()

  const finish = (reason: LayoutProgress['reason'], movement: number): void => {
    if (settled) return
    settled = true
    if (stabilityTimer !== undefined) clearInterval(stabilityTimer)
    if (syncTimer !== undefined) clearInterval(syncTimer)
    if (supervisor) {
      supervisor.stop()
      supervisor.kill()
      // Synchronisation finale : l'affichage doit refléter l'état convergé.
      displayGraph.updateEachNodeAttributes(
        (node, attr) => {
          if (!workGraph.hasNode(node)) return attr
          attr.x = workGraph.getNodeAttribute(node, 'x') as number
          attr.y = workGraph.getNodeAttribute(node, 'y') as number
          return attr
        },
        { attributes: ['x', 'y'] },
      )
    }
    const progress: LayoutProgress = {
      elapsedMs: performance.now() - started,
      movement,
      done: true,
      reason,
    }
    onProgress?.(progress)
    resolveFinished(progress)
  }

  /**
   * Copie la topologie par lots, en rendant la main au navigateur entre chaque.
   *
   * Une copie synchrone de 50 000 nœuds et 150 000 arêtes produit une frame
   * bloquante de ~1,2 s — soit exactement le gel que le worker était censé
   * supprimer. Le découpage la rend invisible.
   */
  const buildWorkGraph = async (): Promise<void> => {
    for (let i = 0; i < nodes.length; i += chunkSize) {
      if (cancelledEarly) return
      const end = Math.min(i + chunkSize, nodes.length)
      for (let j = i; j < end; j++) {
        const node = nodes[j]
        if (node === undefined) continue
        workGraph.addNode(node, {
          x: displayGraph.getNodeAttribute(node, 'x') as number,
          y: displayGraph.getNodeAttribute(node, 'y') as number,
        })
      }
      await yieldToBrowser()
    }

    for (let i = 0; i < edges.length; i += chunkSize) {
      if (cancelledEarly) return
      const end = Math.min(i + chunkSize, edges.length)
      for (let j = i; j < end; j++) {
        const edge = edges[j]
        if (edge === undefined) continue
        const source = displayGraph.source(edge)
        const target = displayGraph.target(edge)
        if (!workGraph.hasEdge(source, target)) {
          workGraph.addUndirectedEdge(source, target)
        }
      }
      await yieldToBrowser()
    }
  }

  /**
   * Recopie les positions vers l'affichage en **une seule** opération groupée.
   *
   * `setNodeAttribute` émet un événement graphology par appel : 50 000 nœuds ×
   * 2 coordonnées = 100 000 événements que Sigma traite un par un. Mesuré :
   * cela effondre le thread principal à 1 fps, pire que la version synchrone.
   *
   * `updateEachNodeAttributes` fait un seul passage et n'émet qu'un événement
   * `eachNodeAttributesUpdated`, que Sigma traite par une réindexation unique.
   */
  const syncPositions = (): void => {
    if (settled && supervisor === undefined) return
    displayGraph.updateEachNodeAttributes(
      (node, attr) => {
        if (!workGraph.hasNode(node)) return attr
        attr.x = workGraph.getNodeAttribute(node, 'x') as number
        attr.y = workGraph.getNodeAttribute(node, 'y') as number
        return attr
      },
      { attributes: ['x', 'y'] },
    )
  }

  void (async () => {
    await buildWorkGraph()
    if (cancelledEarly || settled) return

    supervisor = new FA2LayoutSupervisor(workGraph, { settings: FA2_SETTINGS })
    supervisor.start()

    const sample = pickSample(workGraph, 200)
    let previous = readPositions(workGraph, sample)

    syncTimer = setInterval(syncPositions, syncIntervalMs)

    stabilityTimer = setInterval(() => {
      const current = readPositions(workGraph, sample)
      const movement = averageDisplacement(previous, current)
      previous = current

      const elapsedMs = performance.now() - started

      if (elapsedMs >= maxDurationMs) {
        finish('timeout', movement)
        return
      }
      if (elapsedMs >= minDurationMs && movement < stabilityThreshold) {
        finish('stabilized', movement)
        return
      }

      onProgress?.({ elapsedMs, movement, done: false })
    }, sampleIntervalMs)
  })()

  return {
    finished,
    cancel: () => {
      cancelledEarly = true
      finish('cancelled', 0)
    },
  }
}

/** Rend la main au navigateur pour qu'il puisse dessiner une frame. */
function yieldToBrowser(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0)
  })
}

/**
 * Lance FA2 dans un worker et rend la main immédiatement.
 *
 * Le graphe est muté en continu pendant l'exécution : un moteur de rendu qui
 * l'observe (Sigma) affiche la relaxation en direct.
 */
export function runLayoutInWorker(
  graph: Graph,
  options: RunLayoutOptions = {},
): LayoutHandle {
  const {
    maxDurationMs = 30_000,
    stabilityThreshold = 0.35,
    sampleIntervalMs = 400,
    onProgress,
  } = options

  const supervisor = new FA2LayoutSupervisor(graph, { settings: FA2_SETTINGS })

  // Échantillon de nœuds pour estimer le mouvement : parcourir les 50 000 à
  // chaque tick coûterait plus cher que le gain, et un échantillon aléatoire
  // suffit à détecter une stabilisation globale.
  const sample = pickSample(graph, 200)
  let previous = readPositions(graph, sample)

  const started = performance.now()
  let timer: ReturnType<typeof setInterval> | undefined
  let settled = false

  let resolveFinished: (p: LayoutProgress) => void
  const finished = new Promise<LayoutProgress>((resolve) => {
    resolveFinished = resolve
  })

  const finish = (reason: LayoutProgress['reason'], movement: number): void => {
    if (settled) return
    settled = true
    if (timer !== undefined) clearInterval(timer)
    supervisor.stop()
    supervisor.kill()
    const progress: LayoutProgress = {
      elapsedMs: performance.now() - started,
      movement,
      done: true,
      reason,
    }
    onProgress?.(progress)
    resolveFinished(progress)
  }

  supervisor.start()

  timer = setInterval(() => {
    const current = readPositions(graph, sample)
    const movement = averageDisplacement(previous, current)
    previous = current

    const elapsedMs = performance.now() - started

    if (movement < stabilityThreshold) {
      finish('stabilized', movement)
      return
    }
    if (elapsedMs >= maxDurationMs) {
      finish('timeout', movement)
      return
    }

    onProgress?.({ elapsedMs, movement, done: false })
  }, sampleIntervalMs)

  return {
    finished,
    cancel: () => finish('cancelled', 0),
  }
}

function pickSample(graph: Graph, size: number): string[] {
  const nodes = graph.nodes()
  if (nodes.length <= size) return nodes
  const step = Math.floor(nodes.length / size)
  const sample: string[] = []
  for (let i = 0; i < nodes.length && sample.length < size; i += step) {
    const node = nodes[i]
    if (node !== undefined) sample.push(node)
  }
  return sample
}

function readPositions(
  graph: Graph,
  nodes: string[],
): Array<{ x: number; y: number }> {
  return nodes.map((node) => ({
    x: (graph.getNodeAttribute(node, 'x') as number) ?? 0,
    y: (graph.getNodeAttribute(node, 'y') as number) ?? 0,
  }))
}

function averageDisplacement(
  before: Array<{ x: number; y: number }>,
  after: Array<{ x: number; y: number }>,
): number {
  if (before.length === 0) return 0
  let total = 0
  for (let i = 0; i < before.length; i++) {
    const a = before[i]
    const b = after[i]
    if (a === undefined || b === undefined) continue
    total += Math.hypot(b.x - a.x, b.y - a.y)
  }
  return total / before.length
}
