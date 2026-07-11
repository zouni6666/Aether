import type { PricingTier, TieredPricingConfig } from './endpoints/types'
import { getAuthoritativeModelPricing } from './authoritative-model-pricing'

export interface ModelsDevTokenCost {
  input: number
  output: number
  reasoning?: number
  cache_read?: number
  cache_write?: number
  input_audio?: number
  output_audio?: number
}

export interface ModelsDevCostTier extends ModelsDevTokenCost {
  tier: {
    type: 'context'
    size: number
  }
}

export interface ModelsDevCost extends ModelsDevTokenCost {
  tiers?: ModelsDevCostTier[]
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function isPrice(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0
}

function parseTokenPrices(value: unknown): Omit<PricingTier, 'up_to'> | null {
  if (!isRecord(value) || !isPrice(value.input) || !isPrice(value.output)) return null
  if (value.cache_write !== undefined && !isPrice(value.cache_write)) return null
  if (value.cache_read !== undefined && !isPrice(value.cache_read)) return null

  return {
    input_price_per_1m: value.input,
    output_price_per_1m: value.output,
    ...(value.cache_write === undefined
      ? {}
      : { cache_creation_price_per_1m: value.cache_write }),
    ...(value.cache_read === undefined
      ? {}
      : { cache_read_price_per_1m: value.cache_read }),
  }
}

function parseContextTier(value: unknown): { size: number; prices: Omit<PricingTier, 'up_to'> } | null {
  if (!isRecord(value) || !isRecord(value.tier)) return null
  if (
    value.tier.type !== 'context'
    || typeof value.tier.size !== 'number'
    || !Number.isSafeInteger(value.tier.size)
    || value.tier.size < 0
  ) {
    return null
  }
  const prices = parseTokenPrices(value)
  return prices ? { size: value.tier.size, prices } : null
}

export function buildModelsDevTieredPricing(cost: unknown): TieredPricingConfig | null {
  const basePrices = parseTokenPrices(cost)
  if (!basePrices || !isRecord(cost)) return null

  const rawTiers = cost.tiers
  if (rawTiers !== undefined && !Array.isArray(rawTiers)) return null
  const contextTiers = (rawTiers ?? []).map(parseContextTier)
  if (contextTiers.some(tier => tier === null)) return null

  const sortedTiers = contextTiers
    .filter((tier): tier is NonNullable<typeof tier> => tier !== null)
    .sort((a, b) => a.size - b.size)
  if (sortedTiers.some((tier, index) => index > 0 && tier.size === sortedTiers[index - 1].size)) {
    return null
  }

  const tiers: PricingTier[] = []
  if (sortedTiers[0]?.size !== 0) {
    tiers.push({
      ...basePrices,
      up_to: sortedTiers[0] ? sortedTiers[0].size - 1 : null,
    })
  }
  tiers.push(...sortedTiers.map((tier, index) => ({
    ...tier.prices,
    up_to: sortedTiers[index + 1] ? sortedTiers[index + 1].size - 1 : null,
  })))

  return { tiers }
}

export function resolveModelsDevTieredPricing(
  providerId: string,
  modelId: string,
  cost: unknown,
): TieredPricingConfig | null {
  return getAuthoritativeModelPricing(providerId, modelId)
    ?? buildModelsDevTieredPricing(cost)
}
