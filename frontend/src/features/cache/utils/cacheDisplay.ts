import { translate } from '@/i18n'
import type { TTLAnalysisUser, UserAffinity } from '@/api/cache'
import { formatNumber } from '@/utils/format'

// 缓存监控页的展示文案：亲和性条目、TTL 与未映射状态

export function truncateMiddle(value: string, head = 8, tail = 6): string {
  const normalized = value.trim()
  if (!normalized) return '---'
  if (normalized.length <= head + tail + 3) return normalized
  return `${normalized.slice(0, head)}...${normalized.slice(-tail)}`
}

export function affinityUserLabel(item: UserAffinity): string {
  return item.username || item.email || item.user_id || '未知'
}

export function affinityUserTitle(item: UserAffinity): string | undefined {
  return item.email || item.username || item.user_id || undefined
}

export function affinityUserApiKeyLabel(item: UserAffinity): string {
  return item.user_api_key_name || item.user_api_key_prefix || truncateMiddle(item.affinity_key)
}

export function affinityModelLabel(item: UserAffinity): string {
  return item.model_display_name || item.model_name || item.global_model_id || '---'
}

export function affinityModelSubtitle(item: UserAffinity): string {
  if (!item.model_display_name || !item.model_name || item.model_display_name === item.model_name) {
    return ''
  }
  return item.model_name
}

export function providerKeyLabel(item: UserAffinity): string {
  if (item.key_name) return item.key_name
  if (item.key_prefix === '[OAuth Token]') return translate('ui.oauthAuthentication')
  if (item.key_prefix === '[OAuth Header]') return 'OAuth Header'
  return item.key_prefix || '---'
}

export function providerKeyTitle(item: UserAffinity): string | undefined {
  if (item.key_name && item.key_prefix) return `${item.key_name} · ${item.key_prefix}`
  return item.key_name || item.key_prefix || undefined
}

export function formatAffinityRequestCount(item: UserAffinity): string {
  if (item.request_count_known === false) return '—'
  return formatNumber(item.request_count || 0)
}

export function formatAffinityRequestCountUnit(item: UserAffinity): string {
  const count = formatAffinityRequestCount(item)
  return count === '—' ? count : `${count}次`
}

export function formatIntervalDescription(user: TTLAnalysisUser): string {
  const p90 = user.percentiles.p90
  if (p90 === null || p90 === undefined) return '-'
  if (p90 < 1) {
    const seconds = Math.round(p90 * 60)
    return `90% 请求间隔 < ${seconds} 秒`
  }
  return `90% 请求间隔 < ${p90.toFixed(1)} 分钟`
}

export function formatTTL(ttl: number | null): string {
  if (ttl === null || ttl < 0) return '-'
  if (ttl < 60) return `${ttl}s`
  const minutes = Math.floor(ttl / 60)
  const seconds = ttl % 60
  if (seconds === 0) return `${minutes}m`
  return `${minutes}m${seconds}s`
}

export function getUnmappedStatusBadge(status: string): { variant: 'default' | 'secondary' | 'destructive' | 'outline', text: string } {
  switch (status) {
    case 'not_found':
      return { variant: 'secondary', get text() { return translate('ui.notFound') } }
    case 'invalid':
      return { variant: 'destructive', get text() { return translate('ui.isInvalid') } }
    case 'error':
      return { variant: 'destructive', get text() { return translate('ui.error') } }
    default:
      return { variant: 'outline', text: status }
  }
}
