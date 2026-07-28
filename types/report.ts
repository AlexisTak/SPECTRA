export type ReportCategory =
  | 'contenu_pedocriminel'
  | 'grooming'
  | 'sollicitation_sexuelle_mineurs'
  | 'diffusion_images_illicites'
  | 'trafic_images'

export type ReportUrgency = 'normale' | 'elevee' | 'critique'
export type ReportStatus = 'brouillon' | 'en_analyse' | 'pret' | 'transmis'

export interface ReportForm {
  caseId?: string
  title: string
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
  tags: string[]
}

export const CATEGORY_LABELS: Record<ReportCategory, string> = {
  contenu_pedocriminel: 'Contenu pédocriminel',
  grooming: 'Grooming',
  sollicitation_sexuelle_mineurs: 'Sollicitation sexuelle',
  diffusion_images_illicites: 'Diffusion images illicites',
  trafic_images: "Trafic d'images",
}

export const URGENCY_LABELS: Record<ReportUrgency, string> = {
  normale: 'Normale',
  elevee: 'Élevée',
  critique: 'Critique',
}

export const STATUS_LABELS: Record<ReportStatus, string> = {
  brouillon: 'Brouillon',
  en_analyse: 'En analyse',
  pret: 'Prêt à transmettre',
  transmis: 'Transmis',
}

export const CATEGORY_COLORS: Record<ReportCategory, string> = {
  contenu_pedocriminel: 'bg-red-500/10 text-red-400 border-red-500/30',
  grooming: 'bg-orange-500/10 text-orange-400 border-orange-500/30',
  sollicitation_sexuelle_mineurs: 'bg-amber-500/10 text-amber-400 border-amber-500/30',
  diffusion_images_illicites: 'bg-yellow-500/10 text-yellow-400 border-yellow-500/30',
  trafic_images: 'bg-purple-500/10 text-purple-400 border-purple-500/30',
}

export const URGENCY_COLORS: Record<ReportUrgency, string> = {
  normale: 'bg-zinc-500/10 text-zinc-400 border-zinc-500/30',
  elevee: 'bg-orange-500/10 text-orange-400 border-orange-500/30',
  critique: 'bg-red-500/10 text-red-400 border-red-500/30',
}

export const STATUS_COLORS: Record<ReportStatus, string> = {
  brouillon: 'bg-zinc-500/10 text-zinc-400 border-zinc-500/30',
  en_analyse: 'bg-blue-500/10 text-blue-400 border-blue-500/30',
  pret: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30',
  transmis: 'bg-violet-500/10 text-violet-400 border-violet-500/30',
}
