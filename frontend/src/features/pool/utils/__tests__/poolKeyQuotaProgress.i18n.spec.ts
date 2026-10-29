import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { PoolKeyDetail } from '@/api/endpoints/pool'
import { setI18nLocale, translate } from '@/i18n'
import {
  buildQuotaProgressItemsFromSnapshot,
  formatCompactQuotaCountdownText,
  getQuotaLabelOrder,
  getQuotaProgressCountdownText,
  getQuotaProgressLabel,
  getQuotaProgressResetDisplayText,
  normalizeQuotaLabel,
  parseQuotaProgressItems,
} from '../poolKeyQuotaProgress'

beforeEach(() => setI18nLocale('zh-CN'))
afterEach(() => {
  setI18nLocale('zh-CN')
  vi.useRealTimers()
})

describe('pool quota display localization', () => {
  it('retains canonical parser categories and sorting across locales', () => {
    setI18nLocale('en-US')
    expect(normalizeQuotaLabel('周剩余')).toBe('周')
    expect(getQuotaLabelOrder(normalizeQuotaLabel('周剩余'))).toBe(7)
    const key = { account_quota: '月剩余 25% | 5H剩余 50% | 周剩余 75%' } as PoolKeyDetail
    const items = parseQuotaProgressItems(key, 'codex', translate)
    expect(items.map(item => item.label)).toEqual(['5H', '周', '月'])
    expect(items.map(item => getQuotaProgressLabel(item.label))).toEqual(['5H', 'Weekly', 'Month'])
    expect(getQuotaProgressLabel('保存：自定义配额')).toBe('保存：自定义配额')
  })

  it('keeps countdowns working after localized label and duration changes', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-01-01T00:00:00Z'))
    const item = { label: '周', remainingPercent: 20, resetSeconds: 90061 }
    expect(getQuotaProgressResetDisplayText(item, 0)).toBe('1天 1:01:01')
    setI18nLocale('en-US')
    expect(getQuotaProgressCountdownText(item, 0)).toBe('Resets in 1d 1:01:01')
    expect(getQuotaProgressResetDisplayText(item, 0)).toBe('1d 1:01:01')
    expect(formatCompactQuotaCountdownText('1天 1:01:01 后重置')).toBe('1d 1:01:01')
  })

  it('translates snapshot detail counts at the display boundary', () => {
    setI18nLocale('en-US')
    const key = {
      status_snapshot: {
        oauth: { code: 'none' },
        account: { code: 'ok', blocked: false },
        quota: {
          provider_type: 'windsurf', code: 'ok', exhausted: false, allowed_models_count: 4,
          windows: [{ code: 'daily', remaining_value: 3, limit_value: 10 }],
        },
      },
    } as PoolKeyDetail
    const items = buildQuotaProgressItemsFromSnapshot(key, 'windsurf', translate)
    expect(items.map(item => item.label)).toEqual(['日', '模型'])
    expect(items.map(item => item.detail)).toEqual(['Remaining 3/10', '4 items'])
    expect(items.map(item => getQuotaProgressLabel(item.label))).toEqual(['Day', 'Model'])
  })
})
