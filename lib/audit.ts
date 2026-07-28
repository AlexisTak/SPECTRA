// Mock audit module for development
// In production, this would use Tauri audit commands

import { canonicalJson } from './canonical'
import type { AuditEvent, LogAuditEventParams } from './audit-types'

export async function logAuditEvent(
  params: LogAuditEventParams,
): Promise<AuditEvent> {
  // Mock implementation - would call Tauri log_audit_event in production
  const event: AuditEvent = {
    id: crypto.randomUUID(),
    caseId: params.caseId || '',
    actor: params.actor,
    action: params.action,
    entityKind: params.entityKind,
    entityId: params.entityId,
    beforeJson: params.beforeJson ? canonicalJson(params.beforeJson) : undefined,
    afterJson: params.afterJson ? canonicalJson(params.afterJson) : undefined,
    metadata: params.metadata ? canonicalJson(params.metadata) : undefined,
    createdAt: new Date().toISOString(),
    rowHash: crypto.randomUUID(),
  }
  return event
}

export async function verifyAuditTrail(caseId: string): Promise<{
  ok: boolean
  total: number
  brokenAtId?: string
  brokenAtIndex?: number
  reason?: string
}> {
  // Mock implementation - would call Tauri verify_audit_trail in production
  return { ok: true, total: 0 }
}

export async function listAudit(
  caseId: string,
  options?: { limit?: number; action?: string; entityKind?: string },
): Promise<AuditEvent[]> {
  // Mock implementation - would call Tauri list_audit in production
  return []
}
