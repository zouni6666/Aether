import { describe, expect, it, vi } from 'vitest'
import {
  createEmptyModelPolicy,
  createEmptyRoutingGroupConfig,
  getDefaultModelPolicy,
  getModelPolicy,
  getModelScheduling,
  isRoutingProviderEnabled,
  modelSchedulingRuleId,
  setDefaultProviderPriorityOverrides,
  setModelKeyPriorityOverridesForFormat,
  upsertModelPolicy,
  upsertModelSchedulingRule,
  type RoutingRule,
} from '../utils/routingPolicy'
import {
  createSchedulingPolicy,
  normalizeProviderSchedulingConfig,
  readSchedulingPolicies,
  schedulingPolicyEditorConfig,
  validateSchedulingPolicies,
  writeSchedulingPolicies,
} from '../utils/schedulingPolicies'

describe('strategy-scoped scheduling policies', () => {
  it.each([false, true])('preserves user visibility %s and billing multipliers through scheduling changes and provider editor projections', userVisible => {
    for (const multiplier of [0, 1.25]) {
      const config = { ...createEmptyRoutingGroupConfig(), billing_multiplier: multiplier, user_visible: userVisible }
      const entry = createSchedulingPolicy(config, 'selected')
      entry.models = ['gpt-5']
      entry.schedulingMode = 'fixed_order'
      entry.policy.provider_priority_overrides = { provider: 2 }
      const updated = writeSchedulingPolicies(config, [entry])
      expect(updated.billing_multiplier).toBe(multiplier)
      expect(updated.user_visible).toBe(userVisible)
      expect(normalizeProviderSchedulingConfig(updated).billing_multiplier).toBe(multiplier)
      expect(normalizeProviderSchedulingConfig(updated).user_visible).toBe(userVisible)
      expect(schedulingPolicyEditorConfig(updated, readSchedulingPolicies(updated)[0]).billing_multiplier).toBe(multiplier)
      expect(schedulingPolicyEditorConfig(updated, readSchedulingPolicies(updated)[0]).user_visible).toBe(userVisible)
    }
  })

  it('normalizes legacy Key scheduling without mutating unrelated rules, scopes, or historical priorities', () => {
    const config = createEmptyRoutingGroupConfig()
    config.default_policy.priority_mode = 'global_key'
    config.disabled_providers = ['disabled']
    config.model_policies = [{ ...createEmptyModelPolicy('*'), key_priority_overrides: { key: 5 }, provider_priority_overrides: { provider: 3 } }]
    config.rules = [{
      id: 'custom-rule', priority: 1, enabled: true, phase: 'client_request', stop_processing: true,
      conditions: { field: 'model', op: 'prefix', value: 'example' },
      actions: [
        { type: 'set_scheduling', priority_mode: 'global_key', scheduling_mode: 'load_balance', sticky_key_attempts: 4 },
        { type: 'set_scheduling', scheduling_mode: 'fixed_order' },
        { type: 'set_header', name: 'x-test', value: 'kept' },
      ],
    }]
    const before = JSON.stringify(config)
    const normalized = normalizeProviderSchedulingConfig(config)
    expect(normalized.default_policy.priority_mode).toBe('provider')
    expect(normalized.disabled_providers).toEqual(config.disabled_providers)
    expect(normalized.model_policies).toEqual(config.model_policies)
    expect(normalized.rules[0]).toEqual({ ...config.rules[0], actions: [
      { type: 'set_scheduling', priority_mode: 'provider', scheduling_mode: 'load_balance', sticky_key_attempts: 4 },
      ...config.rules[0].actions.slice(1),
    ] })
    expect(readSchedulingPolicies(config).every(entry => entry.priorityMode === 'provider')).toBe(true)
    expect(createSchedulingPolicy(config).priorityMode).toBe('provider')
    expect(JSON.stringify(config)).toBe(before)
  })

  it('starts with one all-model strategy', () => {
    const entries = readSchedulingPolicies(createEmptyRoutingGroupConfig())
    expect(entries).toHaveLength(1)
    expect(entries[0]).toMatchObject({ scope: 'all', priorityMode: 'provider', schedulingMode: 'cache_affinity' })
    expect(validateSchedulingPolicies(entries)).toBeNull()
  })

  it('generates unique policy ids on HTTP pages without crypto.randomUUID', () => {
    vi.stubGlobal('crypto', {})
    try {
      const config = createEmptyRoutingGroupConfig()
      expect(createSchedulingPolicy(config).id).not.toBe(createSchedulingPolicy(config).id)
      expect(readSchedulingPolicies(config)).toHaveLength(1)
    } finally {
      vi.unstubAllGlobals()
    }
  })

  it('persists all models as a wildcard rather than enumerating the current catalog', () => {
    const config = createEmptyRoutingGroupConfig()
    const entry = createSchedulingPolicy(config, 'all')
    entry.models = ['model-a', 'model-b']
    entry.priorityMode = 'global_key'
    entry.schedulingMode = 'load_balance'
    entry.policy.provider_priority_overrides = { provider: 2 }
    const saved = JSON.parse(JSON.stringify(writeSchedulingPolicies(config, [entry])))
    expect(saved.rules).toEqual([])
    expect(saved.model_policies.map((policy: { model: string }) => policy.model)).toEqual(['*'])
    expect(readSchedulingPolicies(saved)[0]).toMatchObject({ scope: 'all', models: [] })
    expect(getDefaultModelPolicy(saved).provider_priority_overrides).toEqual({ provider: 2 })
    for (const model of ['model-a', 'model-b', 'future-model']) {
      expect(getModelScheduling(saved, model)).toMatchObject({ priority_mode: 'provider', scheduling_mode: 'load_balance' })
    }
  })

  it('persists one strategy for multiple models with shared rankings', () => {
    const config = createEmptyRoutingGroupConfig()
    const entry = createSchedulingPolicy(config)
    entry.models = ['model-a', 'model-b']
    entry.priorityMode = 'global_key'
    entry.schedulingMode = 'fixed_order'
    entry.policy = {
      ...entry.policy,
      allowed_providers: ['provider-a'],
      provider_enabled_overrides: { 'provider-a': false, 'provider-b': true },
      provider_priority_overrides: { 'provider-a': 2 },
      key_priority_overrides_by_format: { 'openai:chat': { 'key-a': 1 } },
      pool_priority_overrides: { 'pool-a': 3 },
      pool_policy_overrides: { 'pool-a': { scheduling_presets: [{ preset: 'cache_affinity', enabled: true }] } },
    }
    const saved = writeSchedulingPolicies(config, [entry])
    expect(saved.rules).toHaveLength(1)
    expect(saved.rules[0].conditions).toEqual({ any: [
      { field: 'model', op: 'eq', value: 'model-a' },
      { field: 'model', op: 'eq', value: 'model-b' },
    ] })
    for (const model of entry.models) {
      expect(getModelScheduling(saved, model)).toMatchObject({ priority_mode: 'provider', scheduling_mode: 'fixed_order' })
      expect(getModelPolicy(saved, model)).toEqual({ ...entry.policy, model })
    }
    expect(getModelScheduling(saved, 'other-model').scheduling_mode).toBe('cache_affinity')
    const reloaded = readSchedulingPolicies(JSON.parse(JSON.stringify(saved)))
    expect(reloaded).toHaveLength(1)
    expect(reloaded[0]).toMatchObject({ id: entry.id, models: ['model-a', 'model-b'], policy: entry.policy })
    expect(schedulingPolicyEditorConfig(saved, reloaded[0]).model_policies).toEqual([entry.policy])
  })

  it('retains group exclusions in the isolated ranking editor config', () => {
    const config = createEmptyRoutingGroupConfig()
    config.disabled_providers = ['disabled-provider']
    const entry = createSchedulingPolicy(config)
    const editorConfig = schedulingPolicyEditorConfig(config, entry)
    expect(editorConfig.disabled_providers).toEqual(['disabled-provider'])
    editorConfig.disabled_providers.push('local-only')
    expect(config.disabled_providers).toEqual(['disabled-provider'])
  })

  it('preserves independent model membership through serialization without changing group defaults', () => {
    const config = createEmptyRoutingGroupConfig()
    config.disabled_providers = ['legacy-disabled']
    const first = { ...createSchedulingPolicy(config), models: ['model-a', 'model-b'] }
    first.policy.provider_enabled_overrides = { provider: false, 'legacy-disabled': true }
    const second = { ...createSchedulingPolicy(config), models: ['model-c'] }
    const fallback = createSchedulingPolicy(config, 'all')
    fallback.policy.provider_enabled_overrides = { 'default-disabled': false }
    const saved = writeSchedulingPolicies(config, [first, second, fallback])
    const reloaded = readSchedulingPolicies(JSON.parse(JSON.stringify(saved)))
    expect(reloaded).toHaveLength(3)
    expect(reloaded[0].policy.provider_enabled_overrides).toEqual(first.policy.provider_enabled_overrides)
    expect(reloaded[1].policy.provider_enabled_overrides).toEqual({})
    expect(saved.disabled_providers).toEqual(['legacy-disabled'])
    expect(getDefaultModelPolicy(saved).provider_enabled_overrides).toEqual({ 'default-disabled': false })
    for (const model of ['model-a', 'model-b']) {
      expect(getModelPolicy(saved, model).provider_enabled_overrides).toEqual(first.policy.provider_enabled_overrides)
    }
    const firstEditor = schedulingPolicyEditorConfig(saved, reloaded[0])
    const secondEditor = schedulingPolicyEditorConfig(saved, reloaded[1])
    expect(isRoutingProviderEnabled(firstEditor, 'provider')).toBe(false)
    expect(isRoutingProviderEnabled(secondEditor, 'provider')).toBe(true)
    expect(isRoutingProviderEnabled(firstEditor, 'legacy-disabled')).toBe(true)
    expect(isRoutingProviderEnabled(secondEditor, 'legacy-disabled')).toBe(false)
    expect(isRoutingProviderEnabled(firstEditor, 'default-disabled')).toBe(false)
    expect(getDefaultModelPolicy(firstEditor).provider_enabled_overrides).toEqual(first.policy.provider_enabled_overrides)
    getDefaultModelPolicy(firstEditor).provider_enabled_overrides.provider = true
    expect(getModelPolicy(saved, 'model-a').provider_enabled_overrides.provider).toBe(false)
  })

  it('does not merge legacy models with different membership into one configuration', () => {
    const config = createEmptyRoutingGroupConfig()
    config.model_policies = [
      { ...createEmptyModelPolicy('model-a'), provider_enabled_overrides: { provider: false } },
      { ...createEmptyModelPolicy('model-b'), provider_enabled_overrides: { provider: true } },
    ]
    expect(readSchedulingPolicies(config).map(entry => entry.models)).toEqual([['model-a'], ['model-b']])
  })

  it('retains separate strategies even when their settings are identical', () => {
    const config = createEmptyRoutingGroupConfig()
    const first = { ...createSchedulingPolicy(config), models: ['model-a'] }
    const second = { ...createSchedulingPolicy(config), models: ['model-b'] }
    const entries = readSchedulingPolicies(writeSchedulingPolicies(config, [first, second]))
    expect(entries.map(entry => entry.id)).toEqual([first.id, second.id])
  })

  it('loads legacy per-model policies, including rules without a ranking policy', () => {
    let config = createEmptyRoutingGroupConfig()
    for (const model of ['model-a', 'model-b']) {
      config = upsertModelPolicy(config, { ...createEmptyModelPolicy(model), provider_priority_overrides: { provider: 2 } })
      config = upsertModelSchedulingRule(config, model, { priority_mode: 'provider', scheduling_mode: 'load_balance' })
    }
    config = upsertModelSchedulingRule(config, 'model-c', { priority_mode: 'global_key', scheduling_mode: 'fixed_order' })
    const entries = readSchedulingPolicies(config)
    expect(entries).toHaveLength(2)
    expect(entries[0].models).toEqual(['model-a', 'model-b'])
    expect(entries[1].models).toEqual(['model-c'])
    const saved = writeSchedulingPolicies(config, entries)
    for (const model of ['model-a', 'model-b', 'model-c', 'other']) {
      expect(getModelScheduling(saved, model)).toEqual({ ...getModelScheduling(config, model), priority_mode: 'provider' })
      expect(getModelPolicy(saved, model)).toEqual(getModelPolicy(config, model))
    }
    expect(saved.rules.every(rule => !rule.id.startsWith('ui_model_scheduling:'))).toBe(true)
  })

  it('preserves group execution options, failover rules, and custom routing rules', () => {
    const config = createEmptyRoutingGroupConfig()
    config.default_policy.cancel_on_client_disconnect = true
    config.default_policy.sticky_key_attempts = 5
    config.default_policy.max_transfer_count = 7
    config.default_policy.failover_rules.error_stop_patterns = [{ pattern: '', status_codes: [429] }]
    const rule: RoutingRule = {
      id: 'custom-header-rule', priority: 3, enabled: true, phase: 'provider_request',
      conditions: { field: 'model', op: 'prefix', value: 'model-' },
      actions: [{ type: 'set_header', name: 'x-test', value: 'kept' }], stop_processing: false,
    }
    config.rules.push(rule)
    const entry = { ...createSchedulingPolicy(config), models: ['model-a'] }
    const saved = writeSchedulingPolicies(config, [entry])
    expect(saved.default_policy).toEqual(config.default_policy)
    expect(saved.rules[0]).toEqual(rule)
    expect(config.model_policies).toEqual([])
    expect(config.rules).toEqual([rule])
  })

  it('keeps the wildcard fallback before specific rankings and preserves per-format keys', () => {
    let config = setDefaultProviderPriorityOverrides(createEmptyRoutingGroupConfig(), { provider: 4 })
    config = setModelKeyPriorityOverridesForFormat(config, 'model-a', 'openai:chat', { key: 2 })
    const entries = readSchedulingPolicies(config)
    expect(entries.map(entry => entry.scope)).toEqual(['selected', 'all'])
    const saved = writeSchedulingPolicies(config, entries)
    expect(saved.model_policies.map(policy => policy.model)).toEqual(['*', 'model-a'])
    expect(getModelPolicy(saved, 'model-a').key_priority_overrides_by_format).toEqual({ 'openai:chat': { key: 2 } })
    expect(getModelPolicy(saved, '*').provider_priority_overrides).toEqual({ provider: 4 })
  })

  it('removes obsolete model rules when a model leaves a strategy', () => {
    const config = createEmptyRoutingGroupConfig()
    const entry = { ...createSchedulingPolicy(config), models: ['model-a', 'model-b'], schedulingMode: 'load_balance' as const }
    const previous = writeSchedulingPolicies(config, [entry])
    const saved = writeSchedulingPolicies(previous, [{ ...entry, models: ['model-b', 'model-c'] }])
    expect(saved.model_policies.map(policy => policy.model)).toEqual(['model-b', 'model-c'])
    expect(saved.rules).toHaveLength(1)
    expect(getModelScheduling(saved, 'model-a').scheduling_mode).toBe('cache_affinity')
    expect(getModelScheduling(saved, 'model-c').scheduling_mode).toBe('load_balance')
  })

  it('preserves legacy model retry overrides and prefix matching', () => {
    const config = upsertModelSchedulingRule(createEmptyRoutingGroupConfig(), 'legacy-*', {
      priority_mode: 'provider', scheduling_mode: 'fixed_order',
    })
    const rule = config.rules.find(rule => rule.id === modelSchedulingRuleId('legacy-*'))!
    rule.actions = [{ type: 'set_scheduling', priority_mode: 'provider', scheduling_mode: 'fixed_order', sticky_key_attempts: 4 }]
    const saved = writeSchedulingPolicies(config, readSchedulingPolicies(config))
    expect(getModelScheduling(saved, 'legacy-model').sticky_key_attempts).toBe(4)
    expect(getModelScheduling(saved, 'other-model').scheduling_mode).toBe('cache_affinity')
  })

  it('rejects empty or overlapping scopes', () => {
    const config = createEmptyRoutingGroupConfig()
    const first = createSchedulingPolicy(config)
    expect(validateSchedulingPolicies([first])).toContain('选择至少一个')
    first.models = ['model-a']
    expect(validateSchedulingPolicies([first, { ...createSchedulingPolicy(config), models: ['model-a'] }])).toContain('不能重复')
    expect(validateSchedulingPolicies([createSchedulingPolicy(config, 'all'), createSchedulingPolicy(config, 'all')])).toContain('只能有一条')
    expect(validateSchedulingPolicies([])).not.toBeNull()
  })
})
