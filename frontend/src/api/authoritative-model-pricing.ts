import type { PricingTier, TieredPricingConfig } from './endpoints/types'

interface TokenPrices {
  input: number
  output: number
  cacheCreation: number
  cacheRead: number
}

interface ContextPricing {
  standard: TokenPrices
  longContext: TokenPrices
}

const OPENAI_GPT_56_PRICING = new Map<string, ContextPricing>([
  ['gpt-5.6-sol', {
    standard: { input: 5, output: 30, cacheCreation: 6.25, cacheRead: 0.5 },
    longContext: { input: 10, output: 45, cacheCreation: 12.5, cacheRead: 1 },
  }],
  ['gpt-5.6-terra', {
    standard: { input: 2.5, output: 15, cacheCreation: 3.125, cacheRead: 0.25 },
    longContext: { input: 5, output: 22.5, cacheCreation: 6.25, cacheRead: 0.5 },
  }],
  ['gpt-5.6-luna', {
    standard: { input: 1, output: 6, cacheCreation: 1.25, cacheRead: 0.1 },
    longContext: { input: 2, output: 9, cacheCreation: 2.5, cacheRead: 0.2 },
  }],
])

const STANDARD_CONTEXT_LIMIT = 272_000

function pricingTier(
  upTo: number | null,
  prices: TokenPrices,
  multiplier = 1,
): PricingTier {
  return {
    up_to: upTo,
    input_price_per_1m: prices.input * multiplier,
    output_price_per_1m: prices.output * multiplier,
    cache_creation_price_per_1m: prices.cacheCreation * multiplier,
    cache_read_price_per_1m: prices.cacheRead * multiplier,
  }
}

function contextPricingTiers(pricing: ContextPricing, multiplier: number): PricingTier[] {
  return [
    pricingTier(STANDARD_CONTEXT_LIMIT, pricing.standard, multiplier),
    pricingTier(null, pricing.longContext, multiplier),
  ]
}

export function getAuthoritativeModelPricing(
  providerId: string,
  modelId: string,
): TieredPricingConfig | null {
  if (providerId !== 'openai') return null

  const pricing = OPENAI_GPT_56_PRICING.get(modelId)
  if (!pricing) return null

  return {
    tiers: contextPricingTiers(pricing, 1),
    processing_tiers: {
      flex: { tiers: contextPricingTiers(pricing, 0.5) },
      priority: {
        tiers: [pricingTier(STANDARD_CONTEXT_LIMIT, pricing.standard, 2)],
      },
    },
  }
}
