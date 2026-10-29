import { afterEach, beforeEach, describe, expect, it } from 'vitest'

import type { BillingEntitlement } from '@/api/billing'
import { getI18nLocale, setI18nLocale, type Locale } from '@/i18n'
import {
  billingEntitlementLabels,
  entitlementReplacementGroups,
  entitlementsWillReplaceExisting,
  isPlanEntitlementReplacementCandidate,
  usagePolicyEntitlementLabels,
} from '../billingEntitlements'

let previousLocale: Locale

beforeEach(() => {
  previousLocale = getI18nLocale()
  setI18nLocale('zh-CN')
})

afterEach(() => setI18nLocale(previousLocale))

function policy(replacementGroup?: string): BillingEntitlement {
  return {
    type: 'usage_policy',
    replacement_group: replacementGroup,
    rules: [{
      metric: 'request_count',
      window: { kind: 'calendar_week' },
      limit: 1_000,
    }],
  }
}

describe('billing entitlement replacement', () => {
  it('matches the backend replacement query for entitlement record state and expiry', () => {
    const now = Date.parse('2026-08-14T12:00:00Z')

    expect(isPlanEntitlementReplacementCandidate({
      status: 'active',
      expires_at: '2026-08-14T12:00:01Z',
    }, now)).toBe(true)
    expect(isPlanEntitlementReplacementCandidate({
      status: 'active',
      expires_at: '2026-08-14T12:00:00Z',
    }, now)).toBe(false)
    expect(isPlanEntitlementReplacementCandidate({
      status: 'replaced',
      expires_at: '2026-08-15T12:00:00Z',
    }, now)).toBe(false)
    expect(isPlanEntitlementReplacementCandidate({
      status: 'active',
      expires_at: null,
    }, now)).toBe(false)
    expect(isPlanEntitlementReplacementCandidate({
      status: 'active',
      expires_at: 'invalid',
    }, now)).toBe(false)
  })

  it('keeps usage policies stackable when neither package declares a group', () => {
    expect(entitlementsWillReplaceExisting([policy()], [policy()])).toBe(false)
    expect(entitlementsWillReplaceExisting(undefined, [policy()])).toBe(false)
    expect(entitlementsWillReplaceExisting([policy()], undefined)).toBe(false)
  })

  it('preserves legacy whole-package replacement for matching entitlement types', () => {
    const incoming: BillingEntitlement[] = [
      { type: 'daily_quota', daily_quota_usd: 20 },
      policy('new-policy-group'),
    ]
    const existing: BillingEntitlement[] = [
      { type: 'daily_quota', daily_quota_usd: 10 },
      policy('old-policy-group'),
    ]

    expect(entitlementsWillReplaceExisting(incoming, existing)).toBe(true)
    expect(entitlementsWillReplaceExisting(
      [{ type: 'membership_group', grant_user_groups: ['pro'] }],
      [{ type: 'membership_group', grant_user_groups: ['basic'] }],
    )).toBe(true)
    expect(entitlementsWillReplaceExisting(
      [{ type: 'daily_quota', daily_quota_usd: 20 }],
      [{ type: 'membership_group', grant_user_groups: ['basic'] }],
    )).toBe(false)
  })

  it('matches explicit groups across entitlement types after trimming whitespace', () => {
    expect(entitlementsWillReplaceExisting(
      [policy(' pro-tier ')],
      [{ type: 'wallet_credit', replacement_group: 'pro-tier', amount_usd: 10 }],
    )).toBe(true)
  })

  it('uses exact case-sensitive group identity like the backend', () => {
    expect(entitlementsWillReplaceExisting([policy('Pro-Tier')], [policy('pro-tier')])).toBe(false)
  })

  it('normalizes, removes blanks, and deduplicates groups in display order', () => {
    expect(entitlementReplacementGroups([
      policy(' pro-tier '),
      policy(''),
      policy('pro-tier'),
      policy('team-tier'),
    ])).toEqual(['pro-tier', 'team-tier'])
    expect(entitlementReplacementGroups(undefined)).toEqual([])
  })
})

describe('usage policy entitlement labels', () => {
  it('updates all entitlement labels when the locale changes and preserves policy names', () => {
    const items: BillingEntitlement[] = [
      { type: 'wallet_credit', amount_usd: 5 },
      { type: 'daily_quota', daily_quota_usd: 10 },
      { type: 'membership_group', grant_user_groups: ['pro'] },
      {
        type: 'usage_policy',
        name: '  ui.active / 自定义策略  ',
        rules: [{ metric: 'concurrency', window: { kind: 'concurrent' }, limit: 4 }],
      },
    ]

    expect(billingEntitlementLabels(items)).toEqual([
      '附赠余额 $5.00',
      '每日额度 $10.00',
      '会员权益',
      '使用策略 ui.active / 自定义策略',
      '并发 4',
    ])

    setI18nLocale('en-US')
    expect(billingEntitlementLabels(items)).toEqual([
      'Gift balance $5.00',
      'Daily quota $10.00',
      'Membership benefits',
      'Usage policy ui.active / 自定义策略',
      'Concurrency 4',
    ])
    expect(usagePolicyEntitlementLabels({ type: 'usage_policy', rules: [] })).toEqual(['Usage limits'])
  })

  it('localizes calendar, rolling, and subscription windows without Chinese fragments', () => {
    setI18nLocale('en-US')
    expect(usagePolicyEntitlementLabels({
      type: 'usage_policy',
      rules: [
        { metric: 'actual_cost_usd', window: { kind: 'rolling', seconds: 86_400 }, limit: 2.5 },
        { metric: 'request_count', window: { kind: 'rolling', seconds: 7_200 }, limit: 20 },
        { metric: 'actual_cost_usd', window: { kind: 'rolling', seconds: 60 }, limit: 1 },
        { metric: 'actual_cost_usd', window: { kind: 'rolling', seconds: 2 }, limit: 0.5 },
        { metric: 'request_count', window: { kind: 'calendar_day', timezone: ' UTC ' }, limit: 500 },
        { metric: 'actual_cost_usd', window: { kind: 'calendar_week', week_start: 7 }, limit: 10 },
        { metric: 'request_count', window: { kind: 'calendar_week', week_start: 0 }, limit: 2_000 },
        { metric: 'actual_cost_usd', window: { kind: 'calendar_month' }, limit: 20 },
        { metric: 'actual_cost_usd', window: { kind: 'subscription_period' }, limit: 30 },
        { metric: 'request_count', window: { kind: 'subscription_period' }, limit: 1_000 },
        { metric: 'request_count', window: { kind: 'concurrent' }, limit: 5 },
      ],
    })).toEqual([
      'Rolling 1 day $2.5',
      'Rolling 2 hours 20 requests',
      'Rolling 1 minute $1',
      'Rolling 2 seconds $0.5',
      'Daily 500 requests (UTC)',
      'Weekly $10 (system timezone, starting Sunday)',
      'Weekly 2,000 requests (system timezone, starting Monday)',
      'Monthly $20 (system timezone)',
      'Subscription period $30',
      'Subscription period 1,000 requests',
      'Concurrent window 5 requests',
    ])
  })

  it('shows calendar reset timezone and week start', () => {
    expect(usagePolicyEntitlementLabels({
      type: 'usage_policy',
      rules: [
        {
          metric: 'request_count',
          window: { kind: 'calendar_day', timezone: 'Asia/Shanghai' },
          limit: 500,
        },
        {
          metric: 'request_count',
          window: { kind: 'calendar_week', timezone: 'Asia/Shanghai', week_start: 7 },
          limit: 1_000,
        },
        {
          metric: 'actual_cost_usd',
          window: { kind: 'calendar_month' },
          limit: 20,
        },
      ],
    })).toEqual([
      '每日 500 次（Asia/Shanghai）',
      '每周 1,000 次（Asia/Shanghai，周日开始）',
      '每月 $20（系统时区）',
    ])
  })

  it('defaults calendar weeks to Monday and keeps rate and concurrency labels concise', () => {
    expect(usagePolicyEntitlementLabels({
      type: 'usage_policy',
      rules: [
        {
          metric: 'request_count',
          window: { kind: 'calendar_week' },
          limit: 10_000,
        },
        {
          metric: 'request_count',
          window: { kind: 'rolling', seconds: 1 },
          limit: 5,
        },
        {
          metric: 'request_count',
          window: { kind: 'rolling', seconds: 60 },
          limit: 60,
        },
        {
          metric: 'concurrency',
          window: { kind: 'concurrent' },
          limit: 4,
        },
      ],
    })).toEqual([
      '每周 10,000 次（系统时区，周一开始）',
      'QPS 5',
      'RPM 60',
      '并发 4',
    ])
  })
})
