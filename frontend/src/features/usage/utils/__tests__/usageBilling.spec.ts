import { describe, expect, it } from 'vitest'
import { mergeUsageBillingSnapshot, resolveUsageBilling } from '../usageBilling'

describe('request billing snapshots', () => {
  it.each([undefined, null, -1, NaN, Infinity])('uses the historical default for invalid or absent multiplier %s', multiplier => {
    expect(resolveUsageBilling({ cost: 2, actual_cost: 0.5, billing_multiplier: multiplier }))
      .toEqual({ cost: 0.5, multiplier: 1 })
  })

  it('does not invent a historical charge when actual_cost is unavailable', () => {
    expect(resolveUsageBilling({ cost: 2 })).toEqual({ cost: null, multiplier: 1 })
    expect(resolveUsageBilling({ cost: 2, actual_cost: 0 })).toEqual({ cost: 0, multiplier: 1 })
  })

  it.each([null, NaN, Infinity, -1])('does not recalculate an explicit unavailable billing cost %s', billingCost => {
    expect(resolveUsageBilling({ cost: 2, actual_cost: 0.5, billing_multiplier: 3, billing_cost: billingCost }))
      .toEqual({ cost: null, multiplier: 3 })
  })

  it('keeps an overflowed or unavailable base-cost product unknown', () => {
    expect(resolveUsageBilling({ cost: Number.MAX_VALUE, billing_multiplier: 2 }))
      .toEqual({ cost: null, multiplier: 2 })
    expect(resolveUsageBilling({ cost: NaN, billing_multiplier: 0 }))
      .toEqual({ cost: null, multiplier: 0 })
  })

  it('preserves a captured zero cost instead of recalculating it', () => {
    expect(resolveUsageBilling({ cost: 2, billing_multiplier: 3, billing_cost: 0 }))
      .toEqual({ cost: 0, multiplier: 3 })
  })

  it('keeps a snapshot across sparse or stale updates and accepts authoritative zeroes', () => {
    const existing = { billing_multiplier: 2, billing_cost: 4 }
    expect(mergeUsageBillingSnapshot(existing, {})).toEqual(existing)
    expect(mergeUsageBillingSnapshot(existing, { billing_multiplier: null })).toEqual(existing)
    const free = { billing_multiplier: 0, billing_cost: 0 }
    expect(mergeUsageBillingSnapshot(existing, free, false)).toEqual(existing)
    expect(mergeUsageBillingSnapshot(existing, free)).toEqual(free)
  })

  it.each([null, NaN, Infinity, -1])('retains an authoritative unavailable amount %s across sparse refreshes', billingCost => {
    const unavailable = mergeUsageBillingSnapshot(
      { cost: 2, billing_multiplier: 2, billing_cost: 4 },
      { billing_cost: billingCost },
    )
    expect(unavailable.billing_cost).toBeNull()
    const refreshed = mergeUsageBillingSnapshot({ ...unavailable, cost: 2 }, { cost: 3, billing_multiplier: 4 })
    expect(resolveUsageBilling({ ...refreshed, cost: 3 })).toEqual({ cost: null, multiplier: 4 })
    expect(mergeUsageBillingSnapshot(refreshed, { billing_cost: 12 }).billing_cost).toBe(12)
  })

  it('keeps historical names through sparse updates without attaching one group name to another ID', () => {
    const existing = { routing_group_id: 'g1', routing_group_name: '历史分组', billing_multiplier: 2, billing_cost: 4 }
    expect(mergeUsageBillingSnapshot(existing, {})).toEqual(existing)
    expect(mergeUsageBillingSnapshot(existing, { routing_group_id: 'g2', routing_group_name: '新分组' }, false)).toEqual(existing)
    const changedGroup = mergeUsageBillingSnapshot(existing, { routing_group_id: 'g2' })
    expect(changedGroup).toEqual({
      routing_group_id: 'g2', routing_group_name: undefined,
      billing_multiplier: undefined, billing_cost: null,
    })
    expect(resolveUsageBilling({ ...changedGroup, cost: 2, actual_cost: 0.5 })).toEqual({ cost: null, multiplier: 1 })
    const replacementSnapshot = mergeUsageBillingSnapshot(existing, {
      routing_group_id: 'g2', billing_multiplier: 3, billing_cost: 6,
    })
    expect(resolveUsageBilling({ ...replacementSnapshot, cost: 2 })).toEqual({ cost: 6, multiplier: 3 })
  })

  it('recalculates fallback cost when a new multiplier arrives without its paired amount', () => {
    const next = mergeUsageBillingSnapshot(
      { billing_multiplier: 2, billing_cost: 4 },
      { billing_multiplier: 0 },
    )
    expect(resolveUsageBilling({ ...next, cost: 2 })).toEqual({ cost: 0, multiplier: 0 })
    const previouslyMissingMultiplier = mergeUsageBillingSnapshot(
      { billing_cost: 4 },
      { billing_multiplier: 0 },
    )
    expect(resolveUsageBilling({ ...previouslyMissingMultiplier, cost: 2 })).toEqual({ cost: 0, multiplier: 0 })
  })

  it('does not reuse a paired amount when another billing factor changes within the same routing group', () => {
    const updated = mergeUsageBillingSnapshot(
      { cost: 2, routing_group_id: 'g1', billing_multiplier: 2, billing_cost: 4 },
      { routing_group_id: 'g1', billing_multiplier: 6 },
    )
    expect(resolveUsageBilling({ ...updated, cost: 2 })).toEqual({ cost: 12, multiplier: 6 })
  })

  it('invalidates a pending amount when a trusted base cost advances without a paired amount', () => {
    const pending = { cost: 0, routing_group_id: 'g1', billing_multiplier: 2, billing_cost: 0 }
    const completed = mergeUsageBillingSnapshot(pending, { cost: 3, routing_group_id: 'g1', billing_multiplier: 2 })
    expect(resolveUsageBilling({ ...completed, cost: 3 })).toEqual({ cost: 6, multiplier: 2 })
    const stale = mergeUsageBillingSnapshot(pending, { cost: 3 }, false)
    expect(stale.billing_cost).toBe(0)
    const paired = mergeUsageBillingSnapshot(pending, { cost: 3, billing_cost: 5.99999999 })
    expect(paired.billing_cost).toBe(5.99999999)
  })
})
