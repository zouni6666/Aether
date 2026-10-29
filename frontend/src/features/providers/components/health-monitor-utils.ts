
import { translate as t } from '@/i18n'
import { getI18nLocale } from '@/i18n'

export type HealthBadgeVariant =
  | 'default'
  | 'secondary'
  | 'destructive'
  | 'outline'
  | 'success'
  | 'warning'
  | 'dark'

export type HealthMonitorSourceKind = 'endpoint' | 'model' | 'provider'

export interface HealthMonitorDetailSource {
  kind: HealthMonitorSourceKind
  value: string
  title: string
  metaText?: string | null
  totalAttempts: number
  successCount: number
  failedCount: number
  successRate: number
  avgLatencyMs?: number | null
  avgFirstByteMs?: number | null
  avgTps?: number | null
  timeline?: string[] | null
  timelineDetails?: HealthTimelineTooltipMetrics[] | null
  timeRangeStart?: string | null
  timeRangeEnd?: string | null
}

export interface HealthMonitorDetailTarget {
  source: HealthMonitorDetailSource
  lookbackHours: number
}

export interface HealthMonitorAvailability {
  total_attempts: number
  success_rate: number
}

export interface HealthTimelineTooltipMetrics {
  time_range_start?: string | null
  time_range_end?: string | null
  total_attempts?: number | null
  success_count?: number | null
  failed_count?: number | null
  success_rate?: number | null
  avg_latency_ms?: number | null
  avg_first_byte_ms?: number | null
  avg_tps?: number | null
}

/** 健康卡片共用的指标与时间线字段，端点/模型/提供商/关联监控对象都满足 */
export interface HealthMonitorItemMetrics extends HealthMonitorAvailability {
  avg_latency_ms?: number | null
  avg_first_byte_ms?: number | null
  avg_tps?: number | null
  timeline?: string[] | null
  timeline_details?: HealthTimelineTooltipMetrics[] | null
  time_range_start?: string | null
  time_range_end?: string | null
}

/** 健康监控分区中的一项：已经整理好标题、说明与详情来源 */
export interface HealthMonitorSectionItem<T = unknown> {
  key: string
  title: string
  metaText: string
  entityName: string
  monitor: HealthMonitorItemMetrics
  detail: HealthMonitorDetailSource
  raw: T
}

export interface HealthMonitorSectionSummary {
  total: number
  healthy: number
  warning: number
  unhealthy: number
  empty: number
  attempts: number
}

export function summarizeHealthMonitorItems(
  items: HealthMonitorAvailability[]
): HealthMonitorSectionSummary {
  return items.reduce<HealthMonitorSectionSummary>((summary, item) => {
    summary.total += 1
    summary.attempts += item.total_attempts
    if (item.total_attempts <= 0) {
      summary.empty += 1
    } else if (item.success_rate >= 0.95) {
      summary.healthy += 1
    } else if (item.success_rate >= 0.8) {
      summary.warning += 1
    } else {
      summary.unhealthy += 1
    }
    return summary
  }, createEmptyHealthMonitorSectionSummary())
}

export function createEmptyHealthMonitorSectionSummary(): HealthMonitorSectionSummary {
  return {
    total: 0,
    healthy: 0,
    warning: 0,
    unhealthy: 0,
    empty: 0,
    attempts: 0
  }
}

export function getHealthLabel(
  item: HealthMonitorAvailability,
  emptyLabel = t('ui.noRequestsYet')
) {
  if (item.total_attempts <= 0) return emptyLabel
  if (item.success_rate >= 0.95) return t('ui.normal')
  if (item.success_rate >= 0.8) return t('ui.warning1e3c8be')
  return t('ui.unhealthy')
}

export function getHealthBadgeVariant(
  item: HealthMonitorAvailability
): HealthBadgeVariant {
  if (item.total_attempts <= 0) return 'outline'
  if (item.success_rate >= 0.95) return 'success'
  if (item.success_rate >= 0.8) return 'warning'
  return 'destructive'
}

export function getSuccessRateClass(rate: number) {
  if (rate >= 0.95) return 'text-green-600 dark:text-green-400'
  if (rate >= 0.8) return 'text-amber-600 dark:text-amber-400'
  return 'text-red-600 dark:text-red-400'
}

export function getAvailabilityClass(item: HealthMonitorAvailability) {
  if (item.total_attempts <= 0) return ''
  return getSuccessRateClass(item.success_rate)
}

export function formatMs(value?: number | null) {
  if (typeof value !== 'number' || Number.isNaN(value)) return '-'
  const absValue = Math.abs(value)
  if (absValue < 1000) return `${Math.round(value)} ms`
  if (absValue < 60_000) return `${formatDurationNumber(value / 1000)} s`
  return `${formatDurationNumber(value / 60_000)} min`
}

function formatDurationNumber(value: number) {
  return new Intl.NumberFormat(getI18nLocale(), {
    maximumFractionDigits: Math.abs(value) < 10 ? 2 : 1
  }).format(value)
}

export function formatPercent(value: number) {
  if (typeof value !== 'number' || Number.isNaN(value)) return '-'
  return `${(value * 100).toFixed(2)}%`
}

export function formatAvailability(item: HealthMonitorAvailability) {
  if (item.total_attempts <= 0) return '-'
  return formatPercent(item.success_rate)
}

export function formatTps(value?: number | null) {
  if (typeof value !== 'number' || Number.isNaN(value)) return '-'
  return `${new Intl.NumberFormat(getI18nLocale(), {
    maximumFractionDigits: value < 10 ? 2 : value < 100 ? 1 : 0
  }).format(value)} tps`
}

export function formatFullTimestamp(timestamp?: string | null) {
  if (!timestamp) return t('ui.unknownTime')
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return t('ui.unknownTime')
  const pad = (value: number) => value.toString().padStart(2, '0')
  return [
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`,
    `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
  ].join(' ')
}

export function formatTimelineTooltip(input: {
  status: string
  timeRangeStart: string
  timeRangeEnd: string
  metrics?: HealthTimelineTooltipMetrics | null
  entityLabel?: string
  entityName?: string | null
}) {
  const metrics = input.metrics
  const lines = [
    t('providers.migration.totalSucceededFailedAvailabilityStatus942438', { value: String(formatTimelineRequestBreakdown(metrics, input.status)) }),
    t('providers.migration.averageDurationTtfbSpeede7bbce', { value: String(formatTimelineAverageMetrics(metrics)) }),
    t('providers.migration.timeRange521cad', { value: String(formatFullTimestamp(input.timeRangeStart)), value_2: String(formatFullTimestamp(input.timeRangeEnd)) })
  ]
  if (input.entityLabel && input.entityName) {
    lines.push(`${input.entityLabel}：${input.entityName}`)
  }
  return lines.join('\n')
}

function formatTimelineAverageMetrics(metrics?: HealthTimelineTooltipMetrics | null) {
  return [
    formatMs(metrics?.avg_latency_ms),
    formatMs(metrics?.avg_first_byte_ms),
    formatTps(metrics?.avg_tps)
  ].join('/')
}

function formatTimelineRequestBreakdown(
  metrics: HealthTimelineTooltipMetrics | null | undefined,
  status: string
) {
  if (!metrics) return '-'
  const total = formatTimelineCount(metrics.total_attempts)
  const success = formatTimelineCount(metrics.success_count)
  const failed = formatTimelineCount(metrics.failed_count)
  const availability = formatTimelineMetricAvailability(metrics)
  return `${total}/${success}/${failed}/${availability}/${getTimelineLabel(status)}`
}

function formatTimelineCount(value?: number | null) {
  if (typeof value !== 'number' || Number.isNaN(value)) return '-'
  return t('providers.migration.requestsb57efe', { value: String(new Intl.NumberFormat(getI18nLocale()).format(value)) })
}

function formatTimelineMetricAvailability(metrics?: HealthTimelineTooltipMetrics | null) {
  if (!metrics) return '-'
  if (typeof metrics.total_attempts === 'number' && metrics.total_attempts <= 0) return '-'
  if (typeof metrics.success_rate !== 'number' || Number.isNaN(metrics.success_rate)) return '-'
  return formatPercent(metrics.success_rate)
}

export function formatCompactNumber(value: number) {
  return new Intl.NumberFormat(getI18nLocale(), {
    notation: 'compact',
    maximumFractionDigits: 1
  }).format(value)
}

export function formatTimestamp(timestamp?: string | null) {
  if (!timestamp) return t('ui.unknownTime')
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return t('ui.unknownTime')
  return date.toLocaleString(getI18nLocale(), {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

export function getTimelineColor(status: string) {
  switch (status) {
    case 'healthy':
      return 'bg-green-500/80 dark:bg-green-400/90'
    case 'warning':
      return 'bg-amber-400/80 dark:bg-amber-300/80'
    case 'unhealthy':
      return 'bg-red-500/80 dark:bg-red-400/90'
    default:
      return 'bg-gray-300 dark:bg-gray-600'
  }
}

export function getTimelineLabel(status: string) {
  switch (status) {
    case 'healthy':
      return t('ui.health')
    case 'warning':
      return t('ui.warning1e3c8be')
    case 'unhealthy':
      return t('ui.unhealthy')
    default:
      return t('ui.noRequests')
  }
}

export interface HealthMonitorSectionData<T> {
  items: HealthMonitorSectionItem<T>[]
  generatedAt?: string | null
}
