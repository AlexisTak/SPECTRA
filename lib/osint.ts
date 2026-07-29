/**
 * Client OSINT — appel aux commandes Tauri.
 */

import { invoke } from '@tauri-apps/api/core'

export type SelectorKind = 'username' | 'email' | 'phone'

export type ProbeOutcome =
  | { exists: { url: string | null } }
  | { missing: null }
  | { blocked: { reason: string } }
  | { indeterminate: { reason: string } }
  | { error: { message: string } }

export interface ProbeResult {
  site: string
  outcome: ProbeOutcome
  elapsed_ms: number
}

/**
 * Lance une campagne OSINT sur un sélecteur donné.
 *
 * @param selector Le pseudo, email ou téléphone à rechercher
 * @param kind Le type de sélecteur
 * @returns La liste des résultats
 */
export async function runOsintCampaign(
  selector: string,
  kind: SelectorKind,
): Promise<ProbeResult[]> {
  const raw = await invoke<string>('run_osint_campaign', { selector, kind })
  return JSON.parse(raw) as ProbeResult[]
}

/**
 * Charge la liste des sondes disponibles.
 */
export async function loadOsintProbes(): Promise<unknown[]> {
  return invoke('load_osint_probes')
}

/**
 * Met à jour les datasets OSINT (WhatsMyName, Sherlock, Maigret).
 */
export async function updateOsintDatasets(): Promise<{
  added: number
  removed: number
  modified: number
  errors: string[]
}> {
  return invoke('update_osint_datasets')
}
