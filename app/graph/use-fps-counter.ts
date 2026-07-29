/**
 * Compteur FPS embarqué — mesure réelle via `requestAnimationFrame`.
 *
 * Contrairement à Playwright qui throttle rAF à 1 Hz, ce compteur tourne dans
 * la vraie fenêtre Tauri/WebView2 et reflète le taux de rendu perçu par
 * l'analyste.
 */

import { useEffect, useRef, useState } from 'react'

export interface FpsSnapshot {
  /** FPS instantané (frame courante). */
  current: number
  /** Moyenne glissante sur les dernières N frames. */
  avg: number
  /** Minimum sur la période de mesure. */
  min: number
  /** Maximum sur la période de mesure. */
  max: number
  /** Mémoire heap JS en Mo (0 si indisponible). */
  heapMb: number
}

interface UseFpsCounterOptions {
  /** Taille de la fenêtre glissante pour la moyenne. */
  windowSize?: number
  /** Intervalle de rafraîchissement de l'état React, en ms. */
  updateIntervalMs?: number
}

export function useFpsCounter(options: UseFpsCounterOptions = {}) {
  const { windowSize = 60, updateIntervalMs = 200 } = options
  const [fps, setFps] = useState<FpsSnapshot>({
    current: 0,
    avg: 0,
    min: 0,
    max: 0,
    heapMb: 0,
  })
  const rafRef = useRef<number>(0)
  const lastRef = useRef<number>(0)
  const deltasRef = useRef<number[]>([])
  const lastUpdateRef = useRef<number>(0)

  useEffect(() => {
    let running = true

    const tick = (now: number) => {
      if (!running) return

      const delta = now - lastRef.current
      lastRef.current = now

      if (delta > 0 && lastRef.current > 0) {
        const instantFps = 1000 / delta
        deltasRef.current.push(instantFps)
        if (deltasRef.current.length > windowSize) {
          deltasRef.current.shift()
        }
      }

      // Throttle les mises à jour React pour ne pas saturer le thread principal
      if (now - lastUpdateRef.current >= updateIntervalMs && deltasRef.current.length > 1) {
        const samples = deltasRef.current
        const sorted = [...samples].sort((a, b) => a - b)
        const avg = samples.reduce((s, v) => s + v, 0) / samples.length

        const perfMem = (performance as unknown as { memory?: { usedJSHeapSize: number } }).memory
        const heapMb = perfMem ? Math.round(perfMem.usedJSHeapSize / (1024 * 1024)) : 0

        setFps({
          current: Math.round(1000 / delta),
          avg: Math.round(avg),
          min: Math.round(sorted[0] ?? 0),
          max: Math.round(sorted[sorted.length - 1] ?? 0),
          heapMb,
        })
        lastUpdateRef.current = now
      }

      rafRef.current = requestAnimationFrame(tick)
    }

    lastRef.current = performance.now()
    lastUpdateRef.current = performance.now()
    rafRef.current = requestAnimationFrame(tick)

    return () => {
      running = false
      cancelAnimationFrame(rafRef.current)
    }
  }, [windowSize, updateIntervalMs])

  return fps
}
