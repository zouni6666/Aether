import { translate } from '@/i18n'
import type {
  ImageOutputPriceRange,
  ImageOutputQualityPricing,
  PricingTier,
  ProcessingTierPricingConfig,
  TieredPricingConfig,
} from '@/api/endpoints/types'
import { comparePricingUpperBounds } from './tiered-pricing'

// 阶梯定价编辑器的类型、常量与不依赖编辑器状态的纯函数

export type ImageOutputQuality = 'low' | 'medium' | 'high'

export type ImageOutputPriceRow = {
  id: string
  size: string
  prices: Partial<Record<ImageOutputQuality, number>>
  rawPrices: ImageOutputQualityPricing
}

export type ImageOutputPriceRangeRow = {
  id: string
  upToPixels: string
  prices: Partial<Record<ImageOutputQuality, number>>
  rawRange: Record<string, unknown>
}

export type ImagePricingConfig = Pick<
  TieredPricingConfig,
  'image_output_prices' | 'image_output_price_default' | 'image_output_price_ranges'
>

export type ImagePricingState = {
  rows: ImageOutputPriceRow[]
  rangeRows: ImageOutputPriceRangeRow[]
  defaultPrice: string
}

export type CacheManualState = { creation: boolean; read: boolean; cache1h: boolean }

export type CachePriceMode = 'multiplier' | 'price'

export type CacheMultiplierDraft = { creation: string; read: string }

export type ProcessingTierOption = {
  scope: string
  key: string
  label: string
  configured: boolean
}

export type ProcessingTierMultiplierDraft = {
  enabled: boolean
  mode: 'multiplier' | 'custom'
  value: string
}

export type CompactProcessingTierOption = ProcessingTierMultiplierDraft & {
  key: string
  label: string
  detail?: string
  group?: string
  accessibleLabel: string
}

export type CompactProcessingTierGroup = {
  key: string
  label: string | null
  options: CompactProcessingTierOption[]
}

export type PricingScopePolicy = {
  allowEmptyTiers: boolean
  terminalUpperBound: 'require-unbounded' | 'finite-or-unbounded'
}

export const DEFAULT_IMAGE_OUTPUT_SIZES = ['1024x1024', '1536x1024', '1024x1536']

export const DEFAULT_IMAGE_OUTPUT_PIXEL_LIMITS = [1_048_576, 1_572_864, 2_097_152]

export const IMAGE_OUTPUT_QUALITIES: ImageOutputQuality[] = ['low', 'medium', 'high']

export const STANDARD_PRICING_SCOPE = 'standard'

export const PROCESSING_PRICING_SCOPE_PREFIX = 'processing:'

export const UNBOUNDED_THRESHOLD_VALUE = -2

export const KNOWN_PROCESSING_TIERS = [
  { key: 'priority', label: 'Fast（OpenAI）' },
  { key: 'fast', label: 'Fast（Claude）' },
  { key: 'flex', label: 'Flex' },
  { key: 'batch', label: 'Batch' },
] as const

export const COMPACT_PROCESSING_TIERS = [
  { key: 'priority', label: 'OpenAI', detail: 'Chat / Responses', group: 'Fast' },
  { key: 'fast', label: 'Claude', detail: 'Messages', group: 'Fast' },
  { key: 'flex', label: 'Flex' },
  { key: 'batch', label: 'Batch' },
] as const

export function processingTierScope(key: string): string {
  return `${PROCESSING_PRICING_SCOPE_PREFIX}${key}`
}

export function processingTierKeyFromScope(scope: string): string | null {
  return scope.startsWith(PROCESSING_PRICING_SCOPE_PREFIX)
    ? scope.slice(PROCESSING_PRICING_SCOPE_PREFIX.length)
    : null
}

export function pricingScopePolicy(scope: string): PricingScopePolicy {
  return processingTierKeyFromScope(scope) === null
    ? { allowEmptyTiers: false, terminalUpperBound: 'require-unbounded' }
    : { allowEmptyTiers: true, terminalUpperBound: 'finite-or-unbounded' }
}

export function hasOwn(object: object, key: PropertyKey): boolean {
  return Object.prototype.hasOwnProperty.call(object, key)
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

export function cloneJson<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T
}

export function processingTierHasExplicitPricingData(config: ProcessingTierPricingConfig): boolean {
  return (Array.isArray(config.tiers) && config.tiers.length > 0)
    || (typeof config.image_output_price_default === 'number'
      && Number.isFinite(config.image_output_price_default))
    || [
      'image_output_prices',
      'image_output_price_ranges',
      'image_output_price_per_image',
      'image_output_price_matrix',
      'image_prices',
    ].some(key => valueHasEntries(config[key]))
}

export function valueHasEntries(value: unknown): boolean {
  return (Array.isArray(value) && value.length > 0)
    || (isRecord(value) && Object.keys(value).length > 0)
}

export function parseProcessingTierMultiplier(value: string | number): number | null {
  const raw = String(value ?? '').trim()
  if (!raw) return null
  const multiplier = Number(raw)
  return Number.isFinite(multiplier) && multiplier >= 0 ? multiplier : null
}

export function automaticCachePrice(tier: PricingTier | undefined, multiplier: number): number {
  const inputPrice = tier?.input_price_per_1m
  if (typeof inputPrice !== 'number' || !Number.isFinite(inputPrice)) return 0
  return parseFloat((inputPrice * multiplier).toFixed(4))
}

export function replaceCacheTtlPrice(
  prices: PricingTier['cache_ttl_pricing'],
  ttlMinutes: number,
  price: number | null,
): NonNullable<PricingTier['cache_ttl_pricing']> {
  const next = prices ? cloneJson(prices) : []
  const index = next.findIndex(item => item.ttl_minutes === ttlMinutes)
  if (price === null) {
    return index === -1 ? next : next.filter((_, itemIndex) => itemIndex !== index)
  }
  const value = {
    ...(index >= 0 ? next[index] : {}),
    ttl_minutes: ttlMinutes,
    cache_creation_price_per_1m: price,
  }
  if (index >= 0) next[index] = value
  else next.push(value)
  return next
}

export function processingTierDisplayLabel(key: string): string {
  const compactTier = COMPACT_PROCESSING_TIERS.find(tier => tier.key === key)
  if (compactTier) {
    return [
      'group' in compactTier ? compactTier.group : null,
      compactTier.label,
      'detail' in compactTier ? compactTier.detail : null,
    ].filter((part) => Boolean(part)).join(' · ')
  }
  return KNOWN_PROCESSING_TIERS.find(tier => tier.key === key)?.label ?? key
}

export function validatePricingTiers(tiers: PricingTier[], policy: PricingScopePolicy): string | null {
  if (tiers.length === 0) {
    return policy.allowEmptyTiers ? null : '至少需要一个价格阶梯'
  }
  if (
    policy.terminalUpperBound === 'require-unbounded'
    && tiers[tiers.length - 1].up_to !== null
  ) {
    return translate('ui.theFinalPricingTierMustHaveNoUpperLimit')
  }

  let previousUpperBound = 0
  for (let index = 0; index < tiers.length; index += 1) {
    const upperBound = tiers[index].up_to
    if (upperBound === null) {
      if (index < tiers.length - 1) return `阶梯 ${index + 1} 的上限必须是有限值`
      continue
    }
    if (
      typeof upperBound !== 'number'
      || !Number.isFinite(upperBound)
      || !Number.isInteger(upperBound)
      || upperBound <= previousUpperBound
    ) {
      return `阶梯 ${index + 1} 的上限必须大于前一个阶梯`
    }
    previousUpperBound = upperBound
  }

  const priceFields: Array<[keyof PricingTier, string]> = [
    ['input_price_per_1m', '输入价格'],
    ['output_price_per_1m', '输出价格'],
    ['cache_creation_price_per_1m', '缓存创建价格'],
    ['cache_read_price_per_1m', '缓存读取价格'],
  ]
  for (const [index, tier] of tiers.entries()) {
    for (const [field, label] of priceFields) {
      const price = tier[field]
      if (price == null) continue
      if (typeof price !== 'number' || !Number.isFinite(price) || price < 0) {
        return `阶梯 ${index + 1} 的${label}必须是非负有限数值`
      }
    }
    for (const ttlPrice of tier.cache_ttl_pricing ?? []) {
      const price = ttlPrice.cache_creation_price_per_1m
      if (!Number.isFinite(price) || price < 0) {
        return `阶梯 ${index + 1} 的缓存 TTL 价格必须是非负有限数值`
      }
    }
  }
  return null
}

export function normalizedImageOutputPrices(state: ImagePricingState): Record<string, ImageOutputQualityPricing> {
  const out: Record<string, ImageOutputQualityPricing> = {}
  for (const row of state.rows) {
    const size = normalizeImageOutputSize(row.size)
    if (!size) continue
    const prices = cloneJson(row.rawPrices)
    for (const quality of IMAGE_OUTPUT_QUALITIES) {
      const price = row.prices[quality]
      if (price != null && Number.isFinite(price)) {
        prices[quality] = price
      } else {
        delete prices[quality]
      }
    }
    if (Object.keys(prices).length > 0) {
      out[size] = { ...out[size], ...prices }
    }
  }
  return out
}

export function normalizedImageOutputPriceRanges(state: ImagePricingState): ImageOutputPriceRange[] {
  const ranges: ImageOutputPriceRange[] = []
  for (const row of state.rangeRows) {
    const range = cloneJson(row.rawRange)
    const prices = isRecord(range.prices)
      ? cloneJson(range.prices) as ImageOutputQualityPricing
      : {}
    for (const quality of IMAGE_OUTPUT_QUALITIES) {
      const price = row.prices[quality]
      if (price != null && Number.isFinite(price)) {
        prices[quality] = price
      } else {
        delete prices[quality]
      }
    }
    if (Object.keys(prices).length === 0) continue
    range.up_to_pixels = parseOptionalInteger(row.upToPixels)
    range.prices = prices
    ranges.push(range as ImageOutputPriceRange)
  }
  return ranges.sort((a, b) => comparePricingUpperBounds(a.up_to_pixels, b.up_to_pixels))
}

export function parseOptionalFloat(value: string | number): number | null {
  if (value === '' || value === null || value === undefined) return null
  const number = typeof value === 'string' ? parseFloat(value) : value
  return Number.isFinite(number) ? number : null
}

export function parseOptionalInteger(value: string | number): number | null {
  if (value === '' || value === null || value === undefined) return null
  const number = typeof value === 'string' ? Number(value) : value
  return Number.isInteger(number) && number > 0 ? number : null
}

export function normalizeImageOutputSize(size: string): string {
  return String(size || '').trim().replace(/\s*[xX×]\s*/g, 'x')
}

export function getImageOutputPrice(row: ImageOutputPriceRow, quality: ImageOutputQuality): string | number {
  return row.prices[quality] ?? ''
}

export function getImageOutputRangePrice(row: ImageOutputPriceRangeRow, quality: ImageOutputQuality): string | number {
  return row.prices[quality] ?? ''
}

export function parseFloatInput(value: string | number): number {
  const num = typeof value === 'string' ? parseFloat(value) : value
  return isNaN(num) ? 0 : num
}
