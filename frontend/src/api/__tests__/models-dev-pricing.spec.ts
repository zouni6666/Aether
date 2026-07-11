import { describe, expect, it } from 'vitest'

import {
  buildModelsDevTieredPricing,
  resolveModelsDevTieredPricing,
} from '@/api/models-dev-pricing'

describe('buildModelsDevTieredPricing', () => {
  it('maps context bands and cache prices without flattening them', () => {
    expect(buildModelsDevTieredPricing({
      input: 5,
      output: 30,
      cache_read: 0.5,
      cache_write: 6.25,
      tiers: [{
        input: 10,
        output: 45,
        cache_read: 1,
        cache_write: 12.5,
        tier: { type: 'context', size: 272_000 },
      }],
    })).toEqual({
      tiers: [
        {
          up_to: 271_999,
          input_price_per_1m: 5,
          output_price_per_1m: 30,
          cache_creation_price_per_1m: 6.25,
          cache_read_price_per_1m: 0.5,
        },
        {
          up_to: null,
          input_price_per_1m: 10,
          output_price_per_1m: 45,
          cache_creation_price_per_1m: 12.5,
          cache_read_price_per_1m: 1,
        },
      ],
    })
  })

  it('sorts multiple context boundaries into contiguous Aether bands', () => {
    const cost = {
      input: 1,
      output: 2,
      tiers: [
        { input: 5, output: 6, tier: { type: 'context' as const, size: 200_000 } },
        { input: 3, output: 4, tier: { type: 'context' as const, size: 100_000 } },
      ],
    }

    expect(buildModelsDevTieredPricing(cost)?.tiers).toEqual([
      { up_to: 99_999, input_price_per_1m: 1, output_price_per_1m: 2 },
      { up_to: 199_999, input_price_per_1m: 3, output_price_per_1m: 4 },
      { up_to: null, input_price_per_1m: 5, output_price_per_1m: 6 },
    ])
    expect(cost.tiers.map(tier => tier.tier.size)).toEqual([200_000, 100_000])
  })

  it('keeps flat token pricing as one unbounded band', () => {
    expect(buildModelsDevTieredPricing({ input: 0, output: 0.1 })).toEqual({
      tiers: [{ up_to: null, input_price_per_1m: 0, output_price_per_1m: 0.1 }],
    })
  })

  it('omits an empty base band when context pricing starts at zero', () => {
    expect(buildModelsDevTieredPricing({
      input: 1,
      output: 2,
      tiers: [
        { input: 3, output: 4, tier: { type: 'context', size: 0 } },
        { input: 5, output: 6, tier: { type: 'context', size: 100_000 } },
      ],
    })?.tiers).toEqual([
      { up_to: 99_999, input_price_per_1m: 3, output_price_per_1m: 4 },
      { up_to: null, input_price_per_1m: 5, output_price_per_1m: 6 },
    ])
  })

  it.each([
    { input: -1, output: 2 },
    { input: 1, output: Number.POSITIVE_INFINITY },
    {
      input: 1,
      output: 2,
      tiers: [{ input: 3, output: 4, tier: { type: 'context', size: Number.MAX_SAFE_INTEGER + 1 } }],
    },
    {
      input: 1,
      output: 2,
      tiers: [{ input: 3, output: 4, tier: { type: 'context', size: -1 } }],
    },
    {
      input: 1,
      output: 2,
      tiers: [
        { input: 3, output: 4, tier: { type: 'context', size: 100 } },
        { input: 5, output: 6, tier: { type: 'context', size: 100 } },
      ],
    },
  ])('fails closed for malformed structured pricing', (cost) => {
    expect(buildModelsDevTieredPricing(cost)).toBeNull()
  })
})

describe('resolveModelsDevTieredPricing', () => {
  it.each([
    {
      modelId: 'gpt-5.6-sol',
      standard: [5, 30, 6.25, 0.5],
      longContext: [10, 45, 12.5, 1],
    },
    {
      modelId: 'gpt-5.6-terra',
      standard: [2.5, 15, 3.125, 0.25],
      longContext: [5, 22.5, 6.25, 0.5],
    },
    {
      modelId: 'gpt-5.6-luna',
      standard: [1, 6, 1.25, 0.1],
      longContext: [2, 9, 2.5, 0.2],
    },
  ])('uses the complete OpenAI catalog for $modelId', ({ modelId, standard, longContext }) => {
    const tier = (
      upTo: number | null,
      prices: number[],
      multiplier: number,
    ) => ({
      up_to: upTo,
      input_price_per_1m: prices[0] * multiplier,
      output_price_per_1m: prices[1] * multiplier,
      cache_creation_price_per_1m: prices[2] * multiplier,
      cache_read_price_per_1m: prices[3] * multiplier,
    })

    expect(resolveModelsDevTieredPricing('openai', modelId, { input: 999, output: 999 }))
      .toEqual({
        tiers: [
          tier(272_000, standard, 1),
          tier(null, longContext, 1),
        ],
        processing_tiers: {
          flex: {
            tiers: [
              tier(272_000, standard, 0.5),
              tier(null, longContext, 0.5),
            ],
          },
          priority: {
            tiers: [tier(272_000, standard, 2)],
          },
        },
      })
  })

  it('keeps the models.dev lower-bound conversion for models outside the catalog', () => {
    expect(resolveModelsDevTieredPricing('openai', 'other-model', {
      input: 1,
      output: 2,
      tiers: [{ input: 3, output: 4, tier: { type: 'context', size: 272_000 } }],
    })?.tiers.map(tier => tier.up_to)).toEqual([271_999, null])
  })

  it.each([
    ['other-provider', 'gpt-5.6-sol'],
    ['openai', 'GPT-5.6-SOL'],
    ['openai', 'gpt-5.6-sol-latest'],
    ['openai', '__proto__'],
    ['openai', 'constructor'],
  ])('matches provider and model identities exactly for %s/%s', (providerId, modelId) => {
    expect(resolveModelsDevTieredPricing(providerId, modelId, { input: 1, output: 2 }))
      .toEqual({
        tiers: [{ up_to: null, input_price_per_1m: 1, output_price_per_1m: 2 }],
      })
  })
})
