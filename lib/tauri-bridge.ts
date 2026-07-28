// lib/tauri-bridge.ts
// =============================================================================
// Couche d'abstraction unique entre le frontend React et le backend Tauri 2.
//
// But : remplacer TOUTES les Server Actions de `actions/*.ts` par des appels
// `invoke()`. Mêmes noms, mêmes shapes d'arguments → zéro (ou presque) diff
// dans les composants client (il suffira de changer l'import path :
// `from '@/actions/X'` → `from '@/lib/tauri-bridge'`).
//
// Sécurité : on refuse poliment de tourner hors contexte Tauri (browser dev,
// tests unitaires, SSR) avec une erreur explicite. Les vrais runtimes sont
// `npx tauri dev` / `npx tauri build`.
//
// Pas de `'use server'` ici : ce fichier est consommé côté client uniquement.
// =============================================================================

import type {
  AddEvidenceInput,
  Case,
  CaseEvent,
  CaseIntegrityReport,
  CaseIntegrityReport as CaseIntegrityReportType,
  CaseStatus,
  CasePriority,
  CreateCaseInput,
  CreateReportInput,
  CreateSnapshotInput,
  CreateSubjectInput,
  Evidence,
  IntegrityCheck,
  Report,
  ReportContent,
  ReportStats,
  ReportTimelineEvent,
  SearchHit,
  SearchOptions,
  Snapshot,
  SnapshotMetadata,
  Subject,
  UpdateCaseInput,
  UpdateReportInput,
  UpdateSubjectInput,
} from '@/types'
import type { AiSettings } from '@/lib/ai-settings'
import type { Qualification } from '@/lib/claims-types'

import { invoke } from '@tauri-apps/api/core'

/** Vrai uniquement dans une fenêtre Tauri (pas en SSR, pas en navigateur nu). */
export const isTauri = (): boolean =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

/**
 * Point de passage unique vers le backend Rust.
 *
 * Utilise l'API publique `@tauri-apps/api/core` plutôt que
 * `window.__TAURI_INTERNALS__` : cette dernière est un détail d'implémentation
 * interne, non typé, susceptible de changer entre versions mineures.
 *
 * Aucun repli silencieux (`audit.md`, P3-10) : hors contexte Tauri, on lève.
 * Renvoyer une valeur par défaut ferait croire à l'appelant que le backend a
 * répondu, ce qui est précisément le défaut le plus dangereux pour un outil
 * probatoire.
 */
export async function tauriInvoke<T = unknown>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isTauri()) {
    throw new Error(
      `invoke('${command}') appelé hors contexte Tauri. ` +
        'Lancer `npm run tauri:dev` — les commandes backend ne sont pas ' +
        'disponibles dans un navigateur ordinaire.',
    )
  }
  return invoke<T>(command, args)
}

// =============================================================================
// CASES — remplace actions/cases.ts
// =============================================================================

export const getCases = (filters?: {
  statut?: CaseStatus
  search?: string
}): Promise<Case[]> =>
  tauriInvoke<Case[]>('get_cases', {
    statut: filters?.statut ?? null,
    search: filters?.search ?? null,
  })

export const getCase = (id: string): Promise<Case | null> =>
  tauriInvoke<Case | null>('get_case', { id })

export const getCaseStats = (): Promise<Record<string, number>> =>
  tauriInvoke<Record<string, number>>('get_case_stats')

export const createCase = (data: CreateCaseInput): Promise<Case> =>
  tauriInvoke<Case>('create_case', { data })

export const updateCase = (id: string, data: UpdateCaseInput): Promise<Case> =>
  tauriInvoke<Case>('update_case', { id, data })

export const deleteCase = (id: string): Promise<void> =>
  tauriInvoke<void>('delete_case', { id })

// =============================================================================
// EVIDENCE — remplace actions/evidence.ts
// =============================================================================

export interface AddEvidenceOptions {
  actor?: string
  source?: string
  reliability?: number
  qualification?: 'indice' | 'preuve' | 'hypothèse' | 'non_vérifié'
  sourceUrl?: string | null
}

export interface DeleteEvidenceOptions {
  actor?: string
}

export const getEvidence = (caseId: string): Promise<Evidence[]> =>
  tauriInvoke<Evidence[]>('get_evidence', { caseId })

export const addEvidence = (
  data: AddEvidenceInput,
  options?: AddEvidenceOptions,
): Promise<Evidence> =>
  tauriInvoke<Evidence>('add_evidence', {
    caseId: data.caseId,
    data,
    options: options ?? null,
  })

export const deleteEvidence = (
  id: string,
  caseId: string,
  options?: DeleteEvidenceOptions,
): Promise<void> =>
  tauriInvoke<void>('delete_evidence', {
    id,
    caseId,
    options: options ?? null,
  })

// =============================================================================
// SUBJECTS — remplace actions/subjects.ts
// =============================================================================

export interface CreateSubjectOptions {
  actor?: string
}

export interface UpdateSubjectOptions {
  actor?: string
}

export interface DeleteSubjectOptions {
  actor?: string
}

export const getSubjects = (caseId: string): Promise<Subject[]> =>
  tauriInvoke<Subject[]>('get_subjects', { caseId })

export const createSubject = (
  data: CreateSubjectInput,
  options?: CreateSubjectOptions,
): Promise<Subject> =>
  tauriInvoke<Subject>('create_subject', {
    data,
    options: options ?? null,
  })

export const updateSubject = (
  id: string,
  data: UpdateSubjectInput,
  options?: UpdateSubjectOptions,
): Promise<Subject> =>
  tauriInvoke<Subject>('update_subject', {
    id,
    data,
    options: options ?? null,
  })

export const deleteSubject = (
  id: string,
  caseId: string,
  options?: DeleteSubjectOptions,
): Promise<void> =>
  tauriInvoke<void>('delete_subject', {
    id,
    caseId,
    options: options ?? null,
  })

// =============================================================================
// EVENTS — remplace actions/events.ts
// =============================================================================

export const getEvents = (caseId: string): Promise<CaseEvent[]> =>
  tauriInvoke<CaseEvent[]>('get_events', { caseId })

export const addNote = (
  caseId: string,
  description: string,
): Promise<CaseEvent> =>
  tauriInvoke<CaseEvent>('add_note', { caseId, description })

// =============================================================================
// SNAPSHOTS — remplace actions/snapshots.ts
// =============================================================================

export const takeSnapshot = (input: CreateSnapshotInput): Promise<Snapshot> =>
  tauriInvoke<Snapshot>('take_snapshot', { input })

export const listSnapshots = (caseId: string): Promise<SnapshotMetadata[]> =>
  tauriInvoke<SnapshotMetadata[]>('list_snapshots', { caseId })

export const getSnapshot = (id: string): Promise<Snapshot | null> =>
  tauriInvoke<Snapshot | null>('get_snapshot', { id })

export const verifySnapshotIntegrity = (id: string): Promise<unknown> =>
  tauriInvoke<unknown>('verify_snapshot_integrity', { id })

export const deleteSnapshot = (id: string): Promise<void> =>
  tauriInvoke<void>('delete_snapshot', { id })

export const loadSnapshotBundle = (
  id: string,
): Promise<{ bundle: unknown; fileSize: number | undefined } | null> =>
  tauriInvoke<{ bundle: unknown; fileSize: number | undefined } | null>(
    'load_snapshot_bundle',
    { id },
  )

// =============================================================================
// INTEGRITY — remplace actions/integrity.ts
// =============================================================================

export const verifyEvidence = (
  evidenceId: string,
): Promise<IntegrityCheck> =>
  tauriInvoke<IntegrityCheck>('verify_evidence', { evidenceId })

export const verifyCaseEvidence = (
  caseId: string,
): Promise<CaseIntegrityReportType> =>
  tauriInvoke<CaseIntegrityReportType>('verify_case_evidence', { caseId })

// =============================================================================
// AUDIT — remplace actions/audit.ts
// =============================================================================

export const verifyAuditTrail = (
  caseId: string,
): Promise<{
  ok: boolean
  total: number
  brokenAtId?: string
  brokenAtIndex?: number
  reason?: string
}> =>
  tauriInvoke<{
    ok: boolean
    total: number
    brokenAtId?: string
    brokenAtIndex?: number
    reason?: string
  }>('verify_audit_trail', { caseId })

export const listAuditRead = (
  caseId: string,
  options?: { limit?: number; action?: string; entityKind?: string },
): Promise<unknown[]> =>
  tauriInvoke<unknown[]>('list_audit', {
    caseId,
    options: options ?? null,
  })

export const logAccessAction = (input: {
  caseId: string
  actor: string
  metadata?: Record<string, unknown>
}): Promise<void> => tauriInvoke<void>('log_access', input)

// =============================================================================
// CLAIMS — remplace actions/claims.ts
// =============================================================================

export interface SetClaimActionInput {
  caseId: string
  refKind: 'evidence' | 'subject' | 'tiktok_video' | 'web_archive' | 'note'
  refId: string
  qualification: Qualification
  reliability: number
  source: string
  sourceUrl?: string | null
  takenBy?: string | null
  notes?: string | null
}

export const setClaimAction = (input: SetClaimActionInput): Promise<unknown> =>
  tauriInvoke<unknown>('set_claim', { input })

export const listClaims = (caseId: string): Promise<unknown[]> =>
  tauriInvoke<unknown[]>('list_claims', { caseId })

export const getClaim = (
  refKind: string,
  refId: string,
): Promise<unknown | null> =>
  tauriInvoke<unknown | null>('get_claim', { refKind, refId })

export const deleteClaim = (
  id: string,
  caseId: string,
): Promise<void> => tauriInvoke<void>('delete_claim', { id, caseId })

export const getClaimStats = (caseId: string): Promise<unknown> =>
  tauriInvoke<unknown>('get_claim_stats', { caseId })

// =============================================================================
// SEARCH — remplace actions/search.ts
// =============================================================================

export const searchAction = (
  query: string,
  kinds?: string[],
  caseId?: string,
): Promise<SearchHit[]> => tauriInvoke<SearchHit[]>('search_action', { query, kinds, caseId })

export const reindexAll = (): Promise<void> => tauriInvoke<void>('reindex_all')

// =============================================================================
// REPORTS — remplace actions/reports.ts
// =============================================================================

export const getReports = (
  caseId?: string,
): Promise<Report[]> => tauriInvoke<Report[]>('get_reports', { caseId })

export const getReport = (id: string): Promise<Report | null> =>
  tauriInvoke<Report | null>('get_report', { id })

export const createReport = (data: CreateReportInput): Promise<Report> =>
  tauriInvoke<Report>('create_report', { data })

export const updateReport = (
  id: string,
  data: UpdateReportInput,
): Promise<Report> => tauriInvoke<Report>('update_report', { id, data })

export const deleteReport = (id: string): Promise<void> =>
  tauriInvoke<void>('delete_report', { id })

export const getReportStats = (): Promise<ReportStats> =>
  tauriInvoke<ReportStats>('get_report_stats')

export const addReportContent = (
  reportId: string,
  section: string,
  contenu: string,
  ordre?: number,
): Promise<unknown> =>
  tauriInvoke<unknown>('add_report_content', { reportId, section, contenu, ordre })

export const getReportContents = (reportId: string): Promise<ReportContent[]> =>
  tauriInvoke<ReportContent[]>('get_report_contents', { reportId })

export const addReportTimelineEvent = (
  reportId: string,
  dateEvent: string,
  description: string,
): Promise<ReportTimelineEvent> =>
  tauriInvoke<ReportTimelineEvent>('add_report_timeline_event', {
    reportId,
    dateEvent,
    description,
  })

export const getReportTimeline = (
  reportId: string,
): Promise<ReportTimelineEvent[]> =>
  tauriInvoke<ReportTimelineEvent[]>('get_report_timeline', { reportId })

// =============================================================================
// SETTINGS — remplace actions/settings.ts
// =============================================================================

export const getAiSettings = (): Promise<AiSettings> =>
  tauriInvoke<AiSettings>('get_ai_settings')

export const saveAiSettings = (
  settings: Partial<AiSettings>,
): Promise<AiSettings> => tauriInvoke<AiSettings>('save_ai_settings', { settings })

export const getAvailableOllamaModels = (): Promise<{
  models: string[]
  error?: string
}> => tauriInvoke<{ models: string[]; error?: string }>('get_available_ollama_models')

export const getAvailableOllamaModelsForSettings = (
  settings: AiSettings,
): Promise<{ models: string[]; error?: string }> =>
  tauriInvoke<{ models: string[]; error?: string }>('get_available_ollama_models', {
    settings,
  })

export const testOllamaConnection = (
  settings: AiSettings,
): Promise<{ ok: boolean; message: string; model?: string }> =>
  tauriInvoke<{ ok: boolean; message: string; model?: string }>('test_ollama_connection', {
    settings,
  })

// =============================================================================
// OLLAMA — fetch direct (pas de Tauri command)
// =============================================================================

export async function fetchOllamaStatus(): Promise<{
  ok: boolean
  baseUrl?: string
  models: string[]
  error?: string
}> {
  try {
    const s = await getAiSettings()
    const res = await fetch(`${s.baseUrl}/api/tags`, {
      signal: AbortSignal.timeout(s.statusTimeoutMs),
    })
    const data = await res.json() as { models: Array<{ name: string }> }
    return { ok: true, models: data.models.map((m) => m.name), baseUrl: s.baseUrl }
  } catch (e) {
    return {
      ok: false,
      models: [],
      error: String(e instanceof Error ? e.message : 'Erreur Ollama'),
      baseUrl: 'http://localhost:11434',
    }
  }
}
