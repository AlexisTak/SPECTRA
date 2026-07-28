/**
 * Contrat de données entre le frontend et le backend Rust.
 *
 * Ces types reflètent **exactement** ce que les commandes Tauri renvoient. Les
 * structures Rust portent `#[serde(rename_all = "camelCase")]` : `date_creation`
 * arrive donc en `dateCreation`.
 *
 * L'audit (`audit.md`, P2-7) relevait trois définitions divergentes de la même
 * entité — TypeScript, Rust et SQL — maintenues à la main. Ce fichier est la
 * seule source côté frontend. À terme, ces types devraient être **générés**
 * depuis Rust (`ts-rs` ou `specta`) pour que la dérive devienne impossible ;
 * en attendant, toute modification d'une structure Rust exposée doit être
 * répercutée ici.
 */

import { tauriInvoke } from '@/lib/tauri-bridge'

export type CaseStatus = 'ouvert' | 'en_cours' | 'transmis' | 'clos'
export type CasePriority = 'basse' | 'normale' | 'haute' | 'urgente'
export type SubjectStatus = 'suspect' | 'victime' | 'temoin' | 'inconnu'

export interface CaseRecord {
  id: string
  reference: string
  titre: string
  description: string | null
  statut: CaseStatus
  priorite: CasePriority
  categorie: string | null
  dateCreation: string
  dateMiseJour: string
  tags: string[] | null
  meta: unknown
}

export interface SubjectRecord {
  id: string
  caseId: string
  nom: string | null
  prenom: string | null
  statut: SubjectStatus
  dateNaissance: string | null
  lieuNaissance: string | null
  nationalite: string | null
  telephone: string | null
  email: string | null
  adresse: string | null
  description: string | null
  metadata: unknown
}

export interface EvidenceRecord {
  id: string
  caseId: string
  type: string
  nom: string | null
  description: string | null
  chemin: string | null
  hashSha256: string | null
  hashMd5: string | null
  taille: number | null
  dateAjout: string
  statut: string
  source: string | null
  sourceUrl: string | null
  metadata: unknown
}

export interface CaseEventRecord {
  id: string
  caseId: string
  type: string
  titre: string | null
  description: string | null
  timestamp: string
  actor: string | null
  metadata: unknown
}

export interface AuditVerification {
  ok: boolean
  total: number
  brokenAtId: string | null
  brokenAtIndex: number | null
  reason: string | null
}

export interface CreateCaseInput {
  titre: string
  description?: string | null
  statut?: CaseStatus
  priorite?: CasePriority
  categorie?: string | null
  tags?: string[] | null
}

export interface CreateSubjectInput {
  caseId: string
  nom?: string | null
  prenom?: string | null
  statut?: SubjectStatus
  dateNaissance?: string | null
  lieuNaissance?: string | null
  nationalite?: string | null
  telephone?: string | null
  email?: string | null
  adresse?: string | null
  description?: string | null
}

// ---------------------------------------------------------------------------
// Dossiers
// ---------------------------------------------------------------------------

export const listCases = (filters?: {
  statut?: CaseStatus
  search?: string
}): Promise<CaseRecord[]> =>
  tauriInvoke<CaseRecord[]>('get_cases', {
    statut: filters?.statut ?? null,
    search: filters?.search ?? null,
  })

export const getCase = (id: string): Promise<CaseRecord | null> =>
  tauriInvoke<CaseRecord | null>('get_case', { id })

export const createCase = (data: CreateCaseInput): Promise<CaseRecord> =>
  tauriInvoke<CaseRecord>('create_case', { data })

export const updateCase = (
  id: string,
  data: Partial<CreateCaseInput>,
): Promise<CaseRecord> => tauriInvoke<CaseRecord>('update_case', { id, data })

export const deleteCase = (id: string): Promise<void> =>
  tauriInvoke<void>('delete_case', { id })

// ---------------------------------------------------------------------------
// Sujets
// ---------------------------------------------------------------------------

export const listSubjects = (caseId: string): Promise<SubjectRecord[]> =>
  tauriInvoke<SubjectRecord[]>('get_subjects', { caseId })

export const createSubject = (
  data: CreateSubjectInput,
): Promise<SubjectRecord> =>
  tauriInvoke<SubjectRecord>('create_subject', { data, options: null })

export const deleteSubject = (id: string, caseId: string): Promise<void> =>
  tauriInvoke<void>('delete_subject', { id, caseId, options: null })

// ---------------------------------------------------------------------------
// Preuves
// ---------------------------------------------------------------------------

export const listEvidence = (caseId: string): Promise<EvidenceRecord[]> =>
  tauriInvoke<EvidenceRecord[]>('get_evidence', { caseId })

// ---------------------------------------------------------------------------
// Journal du dossier
// ---------------------------------------------------------------------------

export const listEvents = (caseId: string): Promise<CaseEventRecord[]> =>
  tauriInvoke<CaseEventRecord[]>('get_events', { caseId })

export const addNote = (
  caseId: string,
  description: string,
): Promise<CaseEventRecord> =>
  tauriInvoke<CaseEventRecord>('add_note', { caseId, description })

// ---------------------------------------------------------------------------
// Audit
// ---------------------------------------------------------------------------

/**
 * Vérifie la chaîne d'audit du dossier.
 *
 * Le backend **recalcule** chaque maillon (`spectra_audit::verify_chain`) au
 * lieu de se contenter de constater que le champ est non vide — c'était le
 * défaut P1-4 de l'audit.
 */
export const verifyAuditTrail = (caseId: string): Promise<AuditVerification> =>
  tauriInvoke<AuditVerification>('verify_audit_trail', { caseId })

export const listAudit = (
  caseId: string,
  options?: { limit?: number },
): Promise<unknown[]> =>
  tauriInvoke<unknown[]>('list_audit', {
    caseId,
    options: options ?? null,
  })
