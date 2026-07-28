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

export interface IntegrityCheck {
  evidenceId: string
  nom: string | null
  /** `intact` | `altered` | `missing` | `no_hash` | `no_path` */
  status: string
  expectedHash: string | null
  actualHash: string | null
  message: string
  checkedAt: string
}

export interface CaseIntegrityReport {
  caseId: string
  total: number
  intact: number
  altered: number
  missing: number
  noHash: number
  checks: IntegrityCheck[]
  generatedAt: string
}

export const listEvidence = (caseId: string): Promise<EvidenceRecord[]> =>
  tauriInvoke<EvidenceRecord[]>('get_evidence', { caseId })

/**
 * Ingère une preuve à partir d'un chemin de fichier.
 *
 * Le backend lit le fichier, calcule son empreinte et le copie dans le magasin.
 * L'empreinte n'est **jamais** fournie par le frontend : une valeur que le
 * système n'a pas observée n'atteste rien (`audit.md`, P1-2).
 */
export const addEvidence = (
  caseId: string,
  chemin: string,
  data: {
    type: string
    nom?: string | null
    description?: string | null
    source?: string | null
    sourceUrl?: string | null
  },
): Promise<EvidenceRecord> =>
  tauriInvoke<EvidenceRecord>('add_evidence', {
    caseId,
    data: { caseId, chemin, ...data },
    options: null,
  })

export const deleteEvidence = (id: string, caseId: string): Promise<void> =>
  tauriInvoke<void>('delete_evidence', { id, caseId, options: null })

/** Relit le fichier et recompare son empreinte. */
export const verifyEvidence = (evidenceId: string): Promise<IntegrityCheck> =>
  tauriInvoke<IntegrityCheck>('verify_evidence', { evidenceId })

/** Vérifie toutes les preuves d'un dossier. */
export const verifyCaseEvidence = (
  caseId: string,
): Promise<CaseIntegrityReport> =>
  tauriInvoke<CaseIntegrityReport>('verify_case_evidence', { caseId })

// ---------------------------------------------------------------------------
// Claims (qualification des éléments)
// ---------------------------------------------------------------------------

export type Qualification = 'preuve' | 'indice' | 'hypothese' | 'non_verifie'

export interface ClaimRecord {
  id: string
  caseId: string
  refKind: string
  refId: string
  qualification: Qualification
  fiabilite: number
  source: string | null
  sourceUrl: string | null
  takenBy: string | null
  notes: string | null
  datePreuve: string | null
  metadata: unknown
}

export const listClaims = (caseId: string): Promise<ClaimRecord[]> =>
  tauriInvoke<ClaimRecord[]>('list_claims', { caseId })

export const setClaim = (input: {
  caseId: string
  refKind: string
  refId: string
  qualification: Qualification
  fiabilite: number
  source: string
  notes?: string | null
}): Promise<ClaimRecord> =>
  tauriInvoke<ClaimRecord>('set_claim', {
    input: {
      sourceUrl: null,
      takenBy: null,
      notes: null,
      datePreuve: null,
      metadata: null,
      ...input,
    },
  })

export const deleteClaim = (id: string, caseId: string): Promise<void> =>
  tauriInvoke<void>('delete_claim', { id, caseId })

// ---------------------------------------------------------------------------
// Recherche
// ---------------------------------------------------------------------------

export interface SearchHit {
  id: string
  kind: string
  caseId: string | null
  title: string
  snippet: string
  score: number
}

export const search = (
  query: string,
  kinds?: string[],
  caseId?: string,
): Promise<SearchHit[]> =>
  tauriInvoke<SearchHit[]>('search_action', {
    query,
    kinds: kinds ?? null,
    caseId: caseId ?? null,
  })

export const reindexAll = (): Promise<void> => tauriInvoke<void>('reindex_all')

// ---------------------------------------------------------------------------
// Instantanés
// ---------------------------------------------------------------------------

export interface SnapshotRecord {
  id: string
  caseId: string
  nom: string
  description: string | null
  hashSha256: string
  taille: number | null
  createdAt: string
}

export const listSnapshots = (caseId: string): Promise<SnapshotRecord[]> =>
  tauriInvoke<SnapshotRecord[]>('list_snapshots', { caseId })

export const takeSnapshot = (input: {
  caseId: string
  nom: string
  description?: string | null
}): Promise<SnapshotRecord> =>
  tauriInvoke<SnapshotRecord>('take_snapshot', {
    input: { description: null, metadata: null, ...input },
  })

export const deleteSnapshot = (id: string): Promise<void> =>
  tauriInvoke<void>('delete_snapshot', { id })

export const verifySnapshotIntegrity = (
  id: string,
): Promise<{
  status: string
  message: string
  expected_hash?: string
  actual_hash?: string
}> => tauriInvoke('verify_snapshot_integrity', { id })

// ---------------------------------------------------------------------------
// Rapports
// ---------------------------------------------------------------------------

export interface ReportRecord {
  id: string
  reference: string
  caseId: string
  titre: string
  description: string | null
  statut: string
  dateCreation: string
  dateEcheance: string | null
  dateCloture: string | null
  auteur: string | null
  metadata: unknown
}

export const listReports = (caseId?: string): Promise<ReportRecord[]> =>
  tauriInvoke<ReportRecord[]>('get_reports', { caseId: caseId ?? null })

export const createReport = (data: {
  caseId: string
  titre: string
  description?: string | null
  auteur?: string | null
}): Promise<ReportRecord> =>
  tauriInvoke<ReportRecord>('create_report', {
    data: {
      statut: null,
      dateEcheance: null,
      auteur: null,
      metadata: null,
      ...data,
    },
  })

export const deleteReport = (id: string): Promise<void> =>
  tauriInvoke<void>('delete_report', { id })

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
