// Mock reports module for development
// In production, this would use Tauri reports commands

import type { Report, ReportCategory, ReportStatus, ReportUrgency } from '@/types'

export interface CreateReportInput {
  caseId?: string
  titre: string
  description: string
  category: ReportCategory
  urgency: ReportUrgency
  platform?: string
  platformUrl?: string
  platformUserId?: string
  incidentStartDate?: string
  incidentEndDate?: string
  suspectUsername?: string
  victimInfo?: string
  contentAnalysis?: string
  tags?: string[]
}

export interface UpdateReportInput {
  titre?: string
  description?: string
  category?: ReportCategory
  urgency?: ReportUrgency
  platform?: string
  platformUrl?: string
  platformUserId?: string
  incidentStartDate?: string
  incidentEndDate?: string
  suspectUsername?: string
  victimInfo?: string
  contentAnalysis?: string
  tags?: string[]
  status?: ReportStatus
  transmittedTo?: string
}

export async function getReports(
  caseId?: string,
): Promise<Report[]> {
  // Mock implementation - would call Tauri get_reports in production
  return []
}

export async function getReport(id: string): Promise<Report | null> {
  // Mock implementation - would call Tauri get_report in production
  return null
}

export async function createReport(
  data: CreateReportInput,
): Promise<Report> {
  // Mock implementation - would call Tauri create_report in production
  return {
    id: crypto.randomUUID(),
    reference: `RPT-${new Date().getFullYear()}-0001`,
    caseId: data.caseId,
    title: data.titre,
    description: data.description,
    category: data.category,
    urgency: data.urgency,
    status: 'brouillon',
    platform: data.platform,
    reportedAt: new Date().toISOString(),
    checklistPreuves: 0,
    checklistAnalyse: 0,
    checklistFaits: 0,
    checklistHypotheses: 0,
    checklistNonSource: 0,
    createdBy: 'analyste',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    tags: data.tags || [],
  }
}

export async function updateReport(
  id: string,
  data: UpdateReportInput,
): Promise<Report> {
  // Mock implementation - would call Tauri update_report in production
  return {
    id,
    reference: '',
    caseId: undefined,
    title: data.titre || '',
    description: data.description || '',
    category: data.category || 'contenu_pedocriminel',
    urgency: data.urgency || 'normale',
    status: data.status || 'brouillon',
    reportedAt: new Date().toISOString(),
    checklistPreuves: 0,
    checklistAnalyse: 0,
    checklistFaits: 0,
    checklistHypotheses: 0,
    checklistNonSource: 0,
    createdBy: 'analyste',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    tags: data.tags || [],
  }
}

export async function deleteReport(id: string): Promise<void> {
  // Mock implementation - would call Tauri delete_report in production
}

export async function getReportStats(): Promise<{
  total: number
  criticalCount: number
  transmittedCount: number
  byStatus: Record<string, number>
  byCategory: Record<string, number>
}> {
  // Mock implementation - would call Tauri get_report_stats in production
  return {
    total: 0,
    criticalCount: 0,
    transmittedCount: 0,
    byStatus: { brouillon: 0, en_analyse: 0, pret: 0, transmis: 0 },
    byCategory: {
      contenu_pedocriminel: 0,
      grooming: 0,
      sollicitation_sexuelle_mineurs: 0,
      diffusion_images_illicites: 0,
      trafic_images: 0,
    },
  }
}
