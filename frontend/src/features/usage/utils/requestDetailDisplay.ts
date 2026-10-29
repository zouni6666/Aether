import { translate } from '@/i18n'
import type { RequestDetail } from '@/api/dashboard'
import type { RequestTrace } from '@/api/requestTrace'
import { formatByteSize, formatCompactNumber, formatTokens } from '@/utils/format'

export type RequestStateStatus = 'pending' | 'streaming' | 'completed' | 'failed' | 'cancelled'

export const REQUEST_STATE_STATUSES = new Set<RequestStateStatus>([
  'pending',
  'streaming',
  'completed',
  'failed',
  'cancelled',
])

export type CacheTTLPriceEntry = {
  ttl_minutes?: number | null
  cache_creation_price_per_1m?: number | null
  cache_read_price_per_1m?: number | null
}

export type PricingTierLike = {
  up_to?: number | null
  input_price_per_1m?: number | null
  output_price_per_1m?: number | null
  cache_creation_price_per_1m?: number | null
  cache_read_price_per_1m?: number | null
  cache_ttl_pricing?: CacheTTLPriceEntry[] | null
}

export type JsonRecord = Record<string, unknown>

export const METADATA_BYTE_FIELD_PATTERN = /(^bytes$|_bytes$|bytes$)/i

export function asRecord(value: unknown): JsonRecord | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  return value as JsonRecord
}

export function shouldFormatMetadataByteField(key: string, value: unknown): value is number {
  return typeof value === 'number'
    && Number.isFinite(value)
    && METADATA_BYTE_FIELD_PATTERN.test(key)
}

export function formatMetadataDisplayValue(value: unknown, key = ''): unknown {
  if (shouldFormatMetadataByteField(key, value)) {
    return formatByteSize(value)
  }
  if (Array.isArray(value)) {
    return value.map(item => formatMetadataDisplayValue(item))
  }
  if (value && typeof value === 'object') {
    return Object.entries(value as Record<string, unknown>)
      .reduce<Record<string, unknown>>((formatted, [childKey, childValue]) => {
        formatted[childKey] = formatMetadataDisplayValue(childValue, childKey)
        return formatted
      }, {})
  }
  return value
}

export function mapTraceFinalStatusToRequestStatus(
  status?: RequestTrace['final_status'] | null
): RequestStateStatus | undefined {
  switch (status) {
    case 'success':
      return 'completed'
    case 'failed':
      return 'failed'
    case 'cancelled':
      return 'cancelled'
    case 'streaming':
      return 'streaming'
    case 'pending':
      return 'pending'
    default:
      return undefined
  }
}

export function normalizeRequestStateStatus(status: unknown): RequestStateStatus | undefined {
  const normalized = typeof status === 'string' ? status.trim().toLowerCase() : ''
  return REQUEST_STATE_STATUSES.has(normalized as RequestStateStatus)
    ? normalized as RequestStateStatus
    : undefined
}

export function hasRequestFailureSignal(statusCode?: number | null, errorMessage?: string | null): boolean {
  return (typeof statusCode === 'number' && statusCode >= 400) ||
    (typeof errorMessage === 'string' && errorMessage.trim().length > 0)
}

export function resolveRequestStateStatus(
  status: unknown,
  statusCode?: number | null,
  errorMessage?: string | null
): RequestStateStatus | undefined {
  const normalized = normalizeRequestStateStatus(status)
  if ((normalized == null || normalized === 'pending' || normalized === 'streaming') &&
    hasRequestFailureSignal(statusCode, errorMessage)) {
    return 'failed'
  }
  return normalized
}

export function resolveRequestStateStatusFromDetail(nextDetail: Pick<RequestDetail, 'status' | 'status_code' | 'error_message'>): RequestStateStatus | undefined {
  return resolveRequestStateStatus(nextDetail.status, nextDetail.status_code, nextDetail.error_message)
}

export function detailTotalCost(nextDetail: RequestDetail): number | null {
  const structuredCost = typeof nextDetail.cost === 'object' ? nextDetail.cost?.total : null
  const totalCost = toNumber(nextDetail.total_cost)
  const legacyCost = toNumber(nextDetail.cost)
  return [structuredCost, totalCost, legacyCost]
    .map(value => toNumber(value))
    .find((value): value is number => value != null && value > 0)
    ?? toNumber(structuredCost)
    ?? totalCost
    ?? legacyCost
}

export function detailCacheCreation5mTokens(nextDetail: RequestDetail): number | null {
  return toNumber(nextDetail.cache_creation_input_tokens_5m)
    ?? toNumber(nextDetail.cache_creation_ephemeral_5m_input_tokens)
}

export function detailCacheCreation1hTokens(nextDetail: RequestDetail): number | null {
  return toNumber(nextDetail.cache_creation_input_tokens_1h)
    ?? toNumber(nextDetail.cache_creation_ephemeral_1h_input_tokens)
}

export function toNumber(value: unknown): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) return value
  if (typeof value === 'string') {
    const parsed = Number(value)
    return Number.isFinite(parsed) ? parsed : null
  }
  return null
}

export function getNestedValue(record: JsonRecord | null, ...path: string[]): unknown {
  let current: unknown = record
  for (const key of path) {
    const object = asRecord(current)
    if (!object) return null
    current = object[key]
  }
  return current
}

export function getNestedNumber(record: JsonRecord | null, ...path: string[]): number | null {
  return toNumber(getNestedValue(record, ...path))
}

export function getNestedString(record: JsonRecord | null, ...path: string[]): string | null {
  const value = getNestedValue(record, ...path)
  return typeof value === 'string' && value.trim() ? value.trim() : null
}

export function getCaseInsensitiveString(record: JsonRecord | null, key: string): string | null {
  if (!record) return null
  const normalizedKey = key.toLowerCase()
  for (const [name, value] of Object.entries(record)) {
    if (name.toLowerCase() === normalizedKey && typeof value === 'string' && value.trim()) {
      return value.trim()
    }
  }
  return null
}

export function normalizeCacheTtlPricing(value: unknown): CacheTTLPriceEntry[] {
  if (!Array.isArray(value)) return []
  return value
    .map((entry) => asRecord(entry))
    .filter((entry): entry is JsonRecord => entry !== null)
    .map((entry) => ({
      ttl_minutes: toNumber(entry.ttl_minutes),
      cache_creation_price_per_1m: toNumber(entry.cache_creation_price_per_1m),
      cache_read_price_per_1m: toNumber(entry.cache_read_price_per_1m),
    }))
}

// 判断数据是否有实际内容（非空对象/数组）
export function hasContent(data: unknown): boolean {
  if (data === null || data === undefined) return false
  if (typeof data === 'object') {
    return Object.keys(data as object).length > 0
  }
  return true
}

export function toFiniteNumber(value: unknown): number | null {
  const num = Number(value)
  return Number.isFinite(num) ? num : null
}

export function getTierCachePriceForTTL(
  tier: PricingTierLike | null | undefined,
  ttlMinutes: number,
  priceKey: 'cache_creation_price_per_1m' | 'cache_read_price_per_1m',
): number | null {
  const fallback = toFiniteNumber(tier?.[priceKey])
  const ttlPricing = Array.isArray(tier?.cache_ttl_pricing)
    ? tier.cache_ttl_pricing
        .filter((entry): entry is CacheTTLPriceEntry => !!entry && typeof entry === 'object')
        .sort((a, b) => Number(a.ttl_minutes || 0) - Number(b.ttl_minutes || 0))
    : []

  if (ttlPricing.length === 0) return fallback

  const matched = ttlPricing.find((entry) => Number(entry.ttl_minutes || 0) === ttlMinutes)
  const price = toFiniteNumber(matched?.[priceKey])
  return price ?? fallback
}

export function getTierMatchedCachePricingEntry(
  tier: PricingTierLike | null | undefined,
  ttlMinutes: number,
): CacheTTLPriceEntry | null {
  const ttlPricing = Array.isArray(tier?.cache_ttl_pricing)
    ? tier.cache_ttl_pricing
        .filter((entry): entry is CacheTTLPriceEntry => !!entry && typeof entry === 'object')
        .sort((a, b) => Number(a.ttl_minutes || 0) - Number(b.ttl_minutes || 0))
    : []

  if (ttlPricing.length === 0) return null
  return ttlPricing.find((entry) => Number(entry.ttl_minutes || 0) === ttlMinutes) ?? null
}

export function hasTierCacheCreationSplitPricing(tier: PricingTierLike | null | undefined): boolean {
  const ttlPricing = Array.isArray(tier?.cache_ttl_pricing) ? tier.cache_ttl_pricing : []
  return ttlPricing.some((entry) =>
    Number(entry?.ttl_minutes || 0) >= 60
    && toFiniteNumber(entry?.cache_creation_price_per_1m) !== null,
  )
}

export function formatCacheTtlLabel(ttlMinutes: number | null | undefined): string {
  if (!ttlMinutes || ttlMinutes <= 0) return translate('ui.cacheCreation')
  if (ttlMinutes >= 60) return translate('ui.cacheWrite1Hour')
  if (ttlMinutes <= 5) return translate('ui.cacheWrite5Minutes')
  return `缓存创建(${ttlMinutes}min)`
}

// 格式化视频/音频时长
export function formatMediaDuration(seconds: number): string {
  if (seconds < 60) {
    return `${seconds.toFixed(1)}s`
  }
  const mins = Math.floor(seconds / 60)
  const secs = seconds % 60
  if (mins < 60) {
    return `${mins}m ${secs.toFixed(0)}s`
  }
  const hours = Math.floor(mins / 60)
  const remainMins = mins % 60
  return `${hours}h ${remainMins}m`
}

// 获取任务类型标签
export function getTaskTypeLabel(taskType: string): string {
  switch (taskType) {
    case 'video':
      return translate('ui.videoGeneration')
    case 'image':
      return translate('ui.imageGeneration')
    case 'audio':
      return translate('ui.audioGeneration')
    default:
      return taskType
  }
}

export function parseImageSizePixels(size: string | null): number | null {
  if (!size) return null
  const normalized = size.trim().toLowerCase().replace(/\s+/g, '').replace(/×/g, 'x')
  const [widthText, heightText] = normalized.split('x')
  const width = Number(widthText)
  const height = Number(heightText)
  if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) return null
  return Math.trunc(width * height)
}

export function formatImagePriceBucket(bucket: string): string {
  if (bucket === 'default') return translate('ui.defaultPrice')
  if (bucket === 'unbounded') return translate('ui.unlimited')
  const match = bucket.match(/^<=([0-9]+)px$/)
  if (match) return `<= ${formatPixels(Number(match[1]))}`
  return bucket
}

export function formatPixels(value: number): string {
  return `${formatCompactNumber(value)} px`
}

// 格式化价格，修复浮点数精度问题
export function formatPrice(price: number): string {
  // 处理浮点数精度问题，最多保留4位小数，去掉尾部的0
  const fixed = price.toFixed(4)
  return parseFloat(fixed).toString()
}

// 获取阶梯范围文本
export function getTierRangeText(tier: { up_to?: number | null }, index: number, tiers: Array<{ up_to?: number | null }>): string {
  const prevTier = index > 0 ? tiers[index - 1] : null
  const start = prevTier?.up_to ? prevTier.up_to + 1 : 0

  if (tier.up_to) {
    if (start === 0) {
      return `0 ~ ${formatTokens(tier.up_to)} tokens`
    }
    return `${formatTokens(start)} ~ ${formatTokens(tier.up_to)} tokens`
  }
  // 无上限的情况
  return `> ${formatTokens(start)} tokens`
}
