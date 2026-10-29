import type { EndpointHealthDetail } from '@/api/endpoints'
import { compareApiFormats, formatApiFormat } from '@/api/endpoints/types/api-format'
import { defaultLocale, translateMessage, type Locale } from '@/i18n/messages'

// 端点状态枚举
export type EndpointStatus = 'disabled' | 'no_keys' | 'keys_disabled' | 'available'

/**
 * 端点排序
 */
export function sortEndpoints<T extends { api_format: string }>(endpoints: T[]): T[] {
  return [...endpoints].sort((a, b) => compareApiFormats(a.api_format, b.api_format))
}

/**
 * 获取端点状态
 */
export function getEndpointStatus(endpoint: EndpointHealthDetail): EndpointStatus {
  if (endpoint.is_active === false) {
    return 'disabled'
  }
  if ((endpoint.active_keys ?? 0) === 0) {
    return (endpoint.total_keys ?? 0) > 0 ? 'keys_disabled' : 'no_keys'
  }
  return 'available'
}

/**
 * 判断端点是否可用
 */
export function isEndpointAvailable(endpoint: EndpointHealthDetail): boolean {
  return getEndpointStatus(endpoint) === 'available'
}

function getEndpointHealthScore(endpoint: EndpointHealthDetail): number | null {
  const score = endpoint.health_score
  if (!isEndpointAvailable(endpoint) || typeof score !== 'number' || !Number.isFinite(score)) {
    return null
  }
  return Math.min(1, Math.max(0, score))
}

export function getEndpointHealthLabel(endpoint: EndpointHealthDetail): string {
  const score = getEndpointHealthScore(endpoint)
  return score === null ? '-' : `${(score * 100).toFixed(0)}%`
}

export function getEndpointHealthBarWidth(endpoint: EndpointHealthDetail): string {
  const score = getEndpointHealthScore(endpoint)
  return score === null ? '100%' : `${Math.max(score * 100, 5)}%`
}

/**
 * 根据健康分数获取颜色
 */
export function getHealthScoreColor(score: number | undefined | null): string {
  if (score === undefined || score === null || !Number.isFinite(score)) {
    return 'bg-muted-foreground/40'
  }
  if (score >= 0.8) return 'bg-green-500'
  if (score >= 0.5) return 'bg-amber-500'
  return 'bg-red-500'
}

/**
 * 端点不可用时进度条颜色
 */
export function getEndpointDotColor(endpoint: EndpointHealthDetail): string {
  return getHealthScoreColor(getEndpointHealthScore(endpoint))
}

/**
 * 端点提示文本
 */
export function getEndpointTooltip(endpoint: EndpointHealthDetail, locale: Locale = defaultLocale): string {
  const format = formatApiFormat(endpoint.api_format)
  const status = getEndpointStatus(endpoint)

  switch (status) {
    case 'disabled':
      return `${format}: ${translateMessage('feedback.endpoint.disabled', locale)}`
    case 'no_keys':
      return `${format}: ${translateMessage('feedback.endpoint.noKeys', locale)}`
    case 'keys_disabled':
      return `${format}: ${translateMessage('feedback.endpoint.keysDisabled', locale)}`
    case 'available': {
      const score = getEndpointHealthScore(endpoint)
      if (score === null) {
        return `${format}: ${translateMessage('feedback.endpoint.unknownHealth', locale)}`
      }
      return `${format}: ${translateMessage('feedback.endpoint.health', locale, { value: (score * 100).toFixed(0) })}`
    }
  }
}
