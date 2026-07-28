export type CaseStatus = 'ouvert' | 'en_cours' | 'transmis' | 'clos'
export type CasePriority = 'basse' | 'normale' | 'haute' | 'urgente'
export type CaseCategory =
  | 'cybercriminalite'
  | 'arnaque'
  | 'pedocriminalite'
  | 'harcèlement'
  | 'usurpation_identite'
  | 'trafic_donnees'
  | 'contenu_illicite'
  | 'autre'
export type SubjectStatus = 'suspect' | 'victime' | 'temoin' | 'inconnu'

// Report types
export type ReportCategory =
  | 'contenu_pedocriminel'
  | 'grooming'
  | 'sollicitation_sexuelle_mineurs'
  | 'diffusion_images_illicites'
  | 'trafic_images'

export type ReportStatus = 'brouillon' | 'en_analyse' | 'pret' | 'transmis'
export type ReportUrgency = 'normale' | 'elevee' | 'critique'
export type EventType =
  | 'note'
  | 'evidence_added'
  | 'status_change'
  | 'subject_added'
  | 'tiktok_video_added'
  | 'web_archive_added'
  | 'snapshot_taken'
  | 'ai_extraction'
  | 'transcription_done'
  | 'integrity_check'

export type SnapshotKind = 'manual' | 'before_delete' | 'before_close'
export type IntegrityStatus =
  | 'intact'
  | 'altered'
  | 'missing'
  | 'no_hash'
  | 'no_path'
export type AiExtractionKind =
  | 'summary'
  | 'entities'
  | 'transcription'
  | 'keywords'
  | 'translation'
export type CrossLinkType =
  | 'username'
  | 'ip'
  | 'email'
  | 'sha256'
  | 'phone'
  | 'hashtag'
  | 'mention'

export interface Case {
  id: string
  reference: string
  titre: string
  description?: string
  categorie?: CaseCategory
  statut: CaseStatus
  priorite: CasePriority
  enqueteur?: string
  dateCreation: string
  dateCloture?: string
  notes?: string
  tags: string[]
  evidenceCount?: number
  subjectCount?: number
  tiktokCount?: number
  archiveCount?: number
  snapshotCount?: number
}

export interface CaseEvent {
  id: string
  caseId: string
  typeEvent: EventType
  description: string
  dateEvent: string
}

export interface Evidence {
  id: string
  caseId: string
  filename: string
  chemin?: string
  tailleOctets?: number
  typeMime?: string
  sha256?: string
  md5?: string
  dateImport: string
  notes?: string
}

export interface SubjectCompte {
  plateforme: string
  identifiant: string
}

export interface Subject {
  id: string
  caseId: string
  alias?: string
  nomReel?: string
  dateNaissance?: string
  nationalite?: string
  ips: string[]
  comptes: SubjectCompte[]
  emails: string[]
  telephones: string[]
  description?: string
  statut: SubjectStatus
  dateCreation: string
}

export interface TiktokVideo {
  id: string
  caseId: string
  url: string
  videoId?: string
  authorUsername?: string
  authorUrl?: string
  title?: string
  description?: string
  durationSec?: number
  viewCount?: number
  likeCount?: number
  commentCount?: number
  shareCount?: number
  uploadDate?: string
  downloadDate: string
  filePath?: string
  sha256?: string
  tailleOctets?: number
  hashtags: string[]
  mentions: string[]
  transcript?: string
  transcriptDone: boolean
  deletedRemote: boolean
  rawMetadata?: string
}

export interface CreateTiktokInput {
  caseId: string
  url: string
  videoId?: string
  authorUsername?: string
  authorUrl?: string
  title?: string
  description?: string
  durationSec?: number
  viewCount?: number
  likeCount?: number
  commentCount?: number
  shareCount?: number
  uploadDate?: string
}

export interface WebArchive {
  id: string
  caseId: string
  url: string
  urlHash: string
  title?: string
  fetchedDate: string
  filePath?: string
  sha256?: string
  tailleOctets?: number
  httpStatus?: number
  contentType?: string
  assetCount: number
}

export interface Snapshot {
  id: string
  caseId: string
  label?: string
  takenAt: string
  frozenBy?: string
  filePath: string
  sha256: string
  tailleOctets: number
  content: string
  kind: SnapshotKind
}

export interface SnapshotMetadata {
  id: string
  caseId: string
  label?: string
  takenAt: string
  size: number
  kind: SnapshotKind
}

export interface AiExtraction {
  id: string
  caseId: string
  evidenceId?: string
  kind: AiExtractionKind
  model?: string
  prompt?: string
  result: string
  resultJson?: string
  createdAt: string
  durationMs?: number
}

export interface Transcription {
  id: string
  caseId: string
  evidenceId: string
  filePath?: string
  language: string
  durationSec?: number
  text: string
  model?: string
  createdAt: string
}

export interface CrossCaseLink {
  id: string
  sourceCaseId: string
  targetCaseId: string
  linkType: CrossLinkType
  linkValue: string
  confidence: number
  detectedAt: string
  notes?: string
}

export interface AuditEvent {
  id: string
  caseId: string
  actor: string
  action: string
  entityKind: string
  entityId?: string
  beforeJson?: string
  afterJson?: string
  metadata?: string
  createdAt: string
  prevHash?: string
  rowHash: string
}

export interface AuditVerifyResult {
  ok: boolean
  total: number
  brokenAtId?: string
  brokenAtIndex?: number
  reason?: string
}

export interface Claim {
  id: string
  caseId: string
  refKind: string
  refId: string
  qualification: string
  reliability: number
  source: string
  sourceUrl?: string
  takenBy?: string
  takenAt: string
  notes?: string
}

export interface ClaimStats {
  total: number
  byQualification: Record<string, number>
  averageReliability: number
  unverifiedCount: number
  totalEntities: number
}

export interface Report {
  id: string
  caseId?: string
  reference: string
  title: string
  description: string
  category: string
  urgency: string
  status: string
  platform?: string
  platformUrl?: string
  platformUserId?: string
  incidentStartDate?: string
  incidentEndDate?: string
  reportedAt: string
  suspectUsername?: string
  victimInfo?: string
  contentAnalysis?: string
  aiSummary?: string
  keywordsDetected?: string
  urlsDetected?: string
  checklistPreuves: number
  checklistAnalyse: number
  checklistFaits: number
  checklistHypotheses: number
  checklistNonSource: number
  createdBy: string
  createdAt: string
  updatedAt: string
  transmittedAt?: string
  transmittedTo?: string
  tags: string[]
  notes?: string
}

export interface ReportContent {
  id: string
  reportId: string
  type: string
  url?: string
  contentText?: string
  filePath?: string
  sha256?: string
  metadata?: string
  flaggedAt: string
  reviewedBy?: string
  reviewedAt?: string
  reviewNotes?: string
  requiresReview: number
}

export interface ReportTimelineEvent {
  id: string
  reportId: string
  eventType: string
  eventDate: string
  description: string
  source?: string
  metadata?: string
  createdAt: string
}

export interface ReportEvidence {
  id: string
  reportId: string
  type: string
  filePath: string
  sha256: string
  tailleOctets: number
  description: string
  collectedAt: string
  integrityVerified: number
  lastVerifiedAt?: string
}

export interface ReportCorrelation {
  id: string
  sourceReportId: string
  targetReportId: string
  correlationType: string
  confidence: number
  description: string
  detectedAt: string
}

export interface ReportStats {
  total: number
  criticalCount: number
  transmittedCount: number
  byStatus: Record<string, number>
  byCategory: Record<string, number>
}

export interface IntegrityCheck {
  evidenceId: string
  filename: string
  status: IntegrityStatus
  expectedHash?: string
  actualHash?: string
  file_path?: string
  message?: string
}

export interface CaseIntegrityReport {
  caseId: string
  totalChecked: number
  intactCount: number
  alteredCount: number
  missingCount: number
  noHashCount: number
  noPathCount: number
  checks: IntegrityCheck[]
  generatedAt: string
}

export interface SearchHit {
  kind: string
  refId: string
  caseId?: string
  title: string
  body: string
  score: number
}

export interface SearchOptions {
  limit?: number
  offset?: number
  highlight?: boolean
}

export interface AiSettings {
  enabled: boolean
  baseUrl: string
  defaultModel: string
  embeddingModel: string
  investigationType: string
  statusTimeoutMs: number
  generateTimeoutMs: number
  temperature: number
}

export interface OllamaStatus {
  available: boolean
  baseUrl?: string
  models: string[]
  error?: string
}

export interface SnapshotMetadata {
  id: string
  caseId: string
  label?: string
  takenAt: string
  size: number
  kind: SnapshotKind
}

// Types for Tauri bridge
export interface AddEvidenceInput {
  caseId: string
  type: string
  nom?: string
  description?: string
  chemin?: string
  hashSha256?: string
  hashMd5?: string
  taille?: number
  source?: string
  sourceUrl?: string
  metadata?: Record<string, unknown>
}

export interface CreateSubjectInput {
  caseId: string
  nom?: string
  prenom?: string
  statut?: string
  dateNaissance?: string
  lieuNaissance?: string
  nationalite?: string
  telephone?: string
  email?: string
  adresse?: string
  description?: string
  metadata?: Record<string, unknown>
}

export interface UpdateSubjectInput {
  nom?: string
  prenom?: string
  statut?: string
  dateNaissance?: string
  lieuNaissance?: string
  nationalite?: string
  telephone?: string
  email?: string
  adresse?: string
  description?: string
  metadata?: Record<string, unknown>
}

export interface CreateCaseInput {
  titre: string
  description?: string
  statut?: string
  priorite?: string
  categorie?: string
  tags?: string[]
  meta?: Record<string, unknown>
}

export interface UpdateCaseInput {
  titre?: string
  description?: string
  statut?: string
  priorite?: string
  categorie?: string
  tags?: string[]
  meta?: Record<string, unknown>
}

export interface CreateReportInput {
  caseId?: string
  titre: string
  description?: string
  statut?: string
  dateEcheance?: string
  auteur?: string
  metadata?: Record<string, unknown>
}

export interface UpdateReportInput {
  titre?: string
  description?: string
  statut?: string
  dateEcheance?: string
  metadata?: Record<string, unknown>
}

export interface CreateSnapshotInput {
  caseId: string
  nom: string
  description?: string
  metadata?: Record<string, unknown>
}
