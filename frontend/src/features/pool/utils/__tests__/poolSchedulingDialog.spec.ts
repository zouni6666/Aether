import { describe, expect, it } from 'vitest'

import {
  mergePoolAdvancedPatch,
  moveStrategyItem,
  normalizeMutexSelection,
} from '@/features/pool/utils/poolSchedulingDialog'

interface TestPresetItem {
  preset: string
  mutexGroup: string | null
  enabled: boolean
  applicable: boolean
}

function buildItems(): TestPresetItem[] {
  return [
    { preset: 'cache_affinity', mutexGroup: 'distribution_mode', enabled: false, applicable: true },
    { preset: 'lru', mutexGroup: 'distribution_mode', enabled: true, applicable: true },
    { preset: 'single_account', mutexGroup: 'distribution_mode', enabled: false, applicable: true },
    { preset: 'load_balance', mutexGroup: 'distribution_mode', enabled: false, applicable: true },
    { preset: 'recent_refresh', mutexGroup: null, enabled: true, applicable: true },
    { preset: 'quota_balanced', mutexGroup: null, enabled: false, applicable: true },
    { preset: 'priority_first', mutexGroup: null, enabled: true, applicable: true },
  ]
}

describe('poolSchedulingDialog', () => {
  it('moves only strategy items upward without disturbing distribution presets', () => {
    const moved = moveStrategyItem(buildItems(), 6, -1)

    expect(moved.map(item => item.preset)).toEqual([
      'cache_affinity',
      'lru',
      'single_account',
      'load_balance',
      'recent_refresh',
      'priority_first',
      'quota_balanced',
    ])
  })

  it('keeps the original order when a strategy item is already at the top boundary', () => {
    const original = buildItems()
    const moved = moveStrategyItem(original, 4, -1)

    expect(moved.map(item => item.preset)).toEqual(original.map(item => item.preset))
  })

  it('moves a strategy item downward within the strategy group', () => {
    const moved = moveStrategyItem(buildItems(), 4, 1)

    expect(moved.map(item => item.preset)).toEqual([
      'cache_affinity',
      'lru',
      'single_account',
      'load_balance',
      'quota_balanced',
      'recent_refresh',
      'priority_first',
    ])
  })

  it('keeps the original order when a strategy item is already at the bottom boundary', () => {
    const original = buildItems()
    const moved = moveStrategyItem(original, 6, 1)

    expect(moved.map(item => item.preset)).toEqual(original.map(item => item.preset))
  })

  it('keeps the original order when the target item is not a strategy preset', () => {
    const original = buildItems()
    const moved = moveStrategyItem(original, 1, 1)

    expect(moved.map(item => item.preset)).toEqual(original.map(item => item.preset))
  })

  it('keeps the first enabled distribution from the saved order', () => {
    const items = buildItems()
    items[0].enabled = true
    items[1].enabled = false
    items[3].enabled = true
    const savedOrder = [items[3], items[0], ...items.slice(1, 3), ...items.slice(4)]

    const normalized = normalizeMutexSelection(savedOrder)

    expect(normalized.find(item => item.preset === 'load_balance')?.enabled).toBe(true)
    expect(normalized.find(item => item.preset === 'cache_affinity')?.enabled).toBe(false)
  })

  it('does not invent a distribution mode when all are disabled', () => {
    const items = buildItems().map(item => ({
      ...item,
      enabled: item.mutexGroup ? false : item.enabled,
    }))

    const normalized = normalizeMutexSelection(items)

    expect(normalized.filter(item => item.mutexGroup).every(item => !item.enabled)).toBe(true)
  })

  it('preserves pool fields that the current dialog does not edit', () => {
    const merged = mergePoolAdvancedPatch({
      sticky_session_ttl_seconds: 900,
      cost_window_seconds: 7200,
      cost_limit_per_key_tokens: 100_000,
      probing_target_percent: 25,
      global_priority: 7,
    }, {
      score_top_n: 256,
    })

    expect(merged).toEqual({
      sticky_session_ttl_seconds: 900,
      cost_window_seconds: 7200,
      cost_limit_per_key_tokens: 100_000,
      probing_target_percent: 25,
      global_priority: 7,
      score_top_n: 256,
    })
  })
})
