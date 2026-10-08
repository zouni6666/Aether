import { describe, expect, it } from 'vitest'
import { createEmptyRoutingGroupConfig, getModelPolicy, setModelProviderPriorityOverrides } from '@/features/routing/utils/routingPolicy'
import { moveGroupProvider, providerGroupPriority, sortGroupProviders } from '../groupPriority'

const providers = [
  { id: 'a', name: 'A', provider_priority: 10 },
  { id: 'b', name: 'B', provider_priority: 20 },
  { id: 'c', name: 'C', provider_priority: 20 },
  { id: 'd', name: 'D', provider_priority: 30 },
]

describe('provider directory group priorities', () => {
  it('uses model overrides, then group defaults, then resource priority', () => {
    let config = setModelProviderPriorityOverrides(createEmptyRoutingGroupConfig(), '*', { a: 40 })
    config = setModelProviderPriorityOverrides(config, 'model-x', { b: 5 })
    expect(sortGroupProviders(config, providers).map(p => p.id)).toEqual(['b', 'c', 'd', 'a'])
    expect(providers.map(p => providerGroupPriority(config, p, 'model-x'))).toEqual([40, 5, 20, 30])
    expect(sortGroupProviders(createEmptyRoutingGroupConfig(), providers).map(p => p.id)).toEqual(['a', 'b', 'c', 'd'])
  })

  it('moves a provider across hidden rows while preserving untouched ties and saved missing-provider overrides', () => {
    const config = setModelProviderPriorityOverrides(createEmptyRoutingGroupConfig(), '*', { missing: 100 })
    const next = moveGroupProvider(config, providers, 'd', 'a')
    expect(sortGroupProviders(next, providers).map(p => p.id)).toEqual(['d', 'a', 'b', 'c'])
    const priorities = getModelPolicy(next, '*').provider_priority_overrides
    expect(priorities.b).toBe(priorities.c)
    expect(priorities.missing).toBe(100)
    expect(getModelPolicy(config, '*').provider_priority_overrides).toEqual({ missing: 100 })
  })

  it('splits a moved provider out of a tied priority and isolates model-specific changes', () => {
    const config = createEmptyRoutingGroupConfig()
    const next = moveGroupProvider(config, providers, 'c', 'b', 'model-x')
    expect(sortGroupProviders(next, providers, 'model-x').map(p => p.id)).toEqual(['a', 'c', 'b', 'd'])
    expect(providerGroupPriority(next, providers[2], 'model-x')).toBeLessThan(providerGroupPriority(next, providers[1], 'model-x'))
    expect(sortGroupProviders(next, providers).map(p => p.id)).toEqual(['a', 'b', 'c', 'd'])
  })
})
