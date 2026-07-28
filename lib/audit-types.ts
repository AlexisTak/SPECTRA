// Audit types partagés entre Rust et TypeScript

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

export interface LogAuditEventParams {
  caseId?: string
  actor: string
  action: string
  entityKind: string
  entityId?: string
  beforeJson?: string
  afterJson?: string
  metadata?: Record<string, unknown>
}
