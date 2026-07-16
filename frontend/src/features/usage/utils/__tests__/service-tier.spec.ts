import { describe, expect, it } from 'vitest'

import {
  formatServiceTierFact,
  hasServiceTierFact,
  normalizeServiceTierFact,
  resolveServiceTierFacts,
} from '../service-tier'

describe('service tier facts', () => {
  it('keeps requested, actual and billing tiers independent', () => {
    const facts = resolveServiceTierFacts({
      service_tier: 'priority',
      actual_service_tier: 'default',
      settlement: {
        settlement_snapshot: {
          pricing_snapshot: {
            requested_processing_tier: 'ignored-requested-snapshot',
            actual_processing_tier: 'ignored-actual-snapshot',
            billing_processing_tier: 'standard',
          },
        },
      },
    })

    expect(facts).toEqual({ requested: 'priority', actual: 'default', billing: 'standard' })
    expect(hasServiceTierFact(facts)).toBe(true)
  })

  it('does not infer billing from requested or actual tiers', () => {
    expect(resolveServiceTierFacts({
      service_tier: 'priority',
      actual_service_tier: 'flex',
    })).toEqual({ requested: 'priority', actual: 'flex', billing: null })
  })

  it('normalizes only non-empty string facts', () => {
    expect(normalizeServiceTierFact('  Batch  ')).toBe('Batch')
    expect(normalizeServiceTierFact('  ')).toBeNull()
    expect(normalizeServiceTierFact(0)).toBeNull()
  })

  it.each(['priority', 'fast', ' Priority ', 'FAST'])(
    'displays the raw %s tier as Fast',
    (tier) => {
      expect(formatServiceTierFact(tier)).toBe('Fast')
    },
  )

  it('keeps non-fast tier labels unchanged', () => {
    expect(formatServiceTierFact('  Batch  ')).toBe('Batch')
    expect(formatServiceTierFact('  ')).toBeNull()
  })
})
