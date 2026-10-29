import { translate } from '@/i18n'
import type { DateRangeParams } from '../types'

// 请求间隔时间线按所选时间范围折算的小时窗口

export function clampIntervalTimelineHours(hours: number): number {
  return Math.min(720, Math.max(1, Math.ceil(hours)))
}

export function getIntervalTimelineHours(dateRange: DateRangeParams): number {
  switch (dateRange.preset) {
    case 'yesterday':
      return 48
    case 'last7days':
      return 24 * 7
    case 'last30days':
      return 24 * 30
    case 'last90days':
      return 24 * 30
    case 'today':
      return 24
    default:
      break
  }

  if (dateRange.start_date && dateRange.end_date) {
    const start = new Date(`${dateRange.start_date}T00:00:00`)
    const end = new Date(`${dateRange.end_date}T23:59:59`)
    const diffMs = end.getTime() - start.getTime()
    if (!Number.isNaN(diffMs) && diffMs >= 0) {
      return clampIntervalTimelineHours(diffMs / (1000 * 60 * 60))
    }
  }

  return 24
}

export function formatIntervalTimelineWindow(hours: number): string {
  if (hours === 24) return translate('ui.last24Hours')
  if (hours % 24 === 0) return `最近${hours / 24}天`
  return `最近${hours}小时`
}
