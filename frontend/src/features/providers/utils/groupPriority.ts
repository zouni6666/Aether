import {
  getModelPolicy,
  setModelProviderPriorityOverrides,
  type RoutingGroupConfig,
} from '@/features/routing/utils/routingPolicy'

interface RankedProvider { id: string; name: string; provider_priority: number }

export function providerGroupPriority(config: RoutingGroupConfig | null, provider: RankedProvider, model = '*'): number {
  if (!config) return provider.provider_priority
  return config.model_policies.find(policy => policy.model === model)?.provider_priority_overrides[provider.id]
    ?? config.model_policies.find(policy => policy.model === '*')?.provider_priority_overrides[provider.id]
    ?? provider.provider_priority
}

export function sortGroupProviders<T extends RankedProvider>(config: RoutingGroupConfig | null, providers: T[], model = '*'): T[] {
  const priorities = new Map(providers.map(provider => [provider.id, providerGroupPriority(config, provider, model)]))
  return [...providers].sort((a, b) => (priorities.get(a.id) ?? 0) - (priorities.get(b.id) ?? 0)
    || a.name.localeCompare(b.name) || a.id.localeCompare(b.id))
}

// Move within the complete priority order, keeping hidden providers and untouched ties.
export function moveGroupProvider(config: RoutingGroupConfig, providers: RankedProvider[], providerId: string, targetId: string, model = '*'): RoutingGroupConfig {
  const rows = sortGroupProviders(config, providers, model).map(provider => ({
    id: provider.id, priority: providerGroupPriority(config, provider, model),
  }))
  const from = rows.findIndex(row => row.id === providerId)
  const to = rows.findIndex(row => row.id === targetId)
  if (from < 0 || to < 0 || from === to) return config
  const [moving] = rows.splice(from, 1)
  rows.splice(to, 0, moving)
  let rank = 0
  const overrides = { ...getModelPolicy(config, model).provider_priority_overrides }
  rows.forEach((row, index) => {
    const previous = rows[index - 1]
    if (previous && (previous.priority !== row.priority || (previous.id === providerId) !== (row.id === providerId))) rank += 1
    // A moved row starts its own priority bucket; untouched equal-priority
    // rows remain tied so the backend's normal tie-break still applies.
    overrides[row.id] = rank
  })
  return setModelProviderPriorityOverrides(config, model, overrides)
}
