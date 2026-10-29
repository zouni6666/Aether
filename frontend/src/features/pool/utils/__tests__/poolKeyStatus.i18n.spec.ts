import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import type { PoolKeyDetail } from '@/api/endpoints/pool'
import { setI18nLocale } from '@/i18n'
import {
  compactPoolStatusLabel,
  formatCooldownReason,
  getAccountAlertLabel,
  getAccountAlertTitle,
  getOAuthStatusBadgeLabel,
  getSchedulingBadgeLabel,
  getSchedulingStatus,
  getVisibleOAuthState,
} from '../poolKeyStatus'

function poolKey(overrides: Partial<PoolKeyDetail> = {}): PoolKeyDetail {
  return {
    key_id: 'key-1', key_name: '保存', is_active: true, auth_type: 'api_key',
    account_quota: null, cooldown_reason: null, cooldown_ttl_seconds: null,
    cost_window_usage: 0, cost_limit: null, request_count: 0, total_tokens: 0,
    total_cost_usd: '0', sticky_sessions: 0, lru_score: null,
    created_at: null, last_used_at: null,
    ...overrides,
  }
}

beforeEach(() => setI18nLocale('zh-CN'))
afterEach(() => setI18nLocale('zh-CN'))

describe('pool status localization', () => {
  it('reads local status labels in the current locale and preserves unknown labels', () => {
    expect(formatCooldownReason('rate_limited_429')).toBe('429 限流')
    setI18nLocale('en-US')
    expect(formatCooldownReason('rate_limited_429')).toBe('429 Rate limited')
    expect(formatCooldownReason('上游自定义错误：保存')).toBe('上游自定义错误：保存')
    expect(compactPoolStatusLabel('账号已封禁')).toBe('Account banned')
    expect(compactPoolStatusLabel('Workspace unavailable during maintenance')).toBe('Workspace unavailable during maintenance')
    expect(compactPoolStatusLabel(' 保存账户原文及空格 ')).toBe(' 保存账户原文及空格 ')
  })

  it('uses OAuth flags and kinds independently of the displayed text', () => {
    setI18nLocale('en-US')
    const base = { text: '保存', isExpired: false, isExpiringSoon: false, isInvalid: false }
    expect(getOAuthStatusBadgeLabel({ ...base, kind: 'missing_refresh_token' })).toBe('Not added')
    expect(getOAuthStatusBadgeLabel({ ...base, kind: 'unknown_expiration' })).toBe('Unknown')
    expect(getOAuthStatusBadgeLabel({ ...base, isInvalid: true })).toBe('Invalid')

    const temporary = poolKey({ auth_type: 'oauth', oauth_temporary: true })
    const unknownExpiry = poolKey({ auth_type: 'oauth' })
    expect(getOAuthStatusBadgeLabel(getVisibleOAuthState(temporary, 0))).toBe('Not added')
    expect(getOAuthStatusBadgeLabel(getVisibleOAuthState(unknownExpiry, 0))).toBe('Unknown')
    expect(getSchedulingStatus(temporary, 0)).toBe('available')
    expect(getSchedulingStatus(unknownExpiry, 0)).toBe('available')
  })

  it('refreshes cached account labels after a locale switch', () => {
    const key = poolKey({ oauth_invalid_reason: '[ACCOUNT_BLOCK] account suspended' })
    expect(getAccountAlertLabel(key)).toBe('账号封禁')
    setI18nLocale('en-US')
    expect(getAccountAlertLabel(key)).toBe('Account banned')
    expect(getAccountAlertTitle(key)).toBe('Account banned: account suspended')
  })

  it('translates known snapshot codes while retaining the server reason', () => {
    setI18nLocale('en-US')
    const key = poolKey({
      status_snapshot: {
        oauth: { code: 'none' },
        account: { code: 'workspace_deactivated', label: '工作区已停用', reason: '用户原文：保存', blocked: true },
        quota: { code: 'unknown', exhausted: false },
      },
    })
    expect(getAccountAlertLabel(key)).toBe('Workspace disabled')
    expect(getAccountAlertTitle(key)).toBe('Workspace disabled: 用户原文：保存')
    expect(getSchedulingBadgeLabel(key, 0)).toBe('Workspace disabled')

    key.status_snapshot!.account = { code: 'ok', blocked: false }
    const quotaKey = poolKey({
      status_snapshot: {
        ...key.status_snapshot!,
        quota: { code: 'banned', label: '账号已封禁', reason: '上游原文：删除', exhausted: false },
      },
    })
    expect(getAccountAlertLabel(quotaKey)).toBe('Account banned')
    expect(getAccountAlertTitle(quotaKey)).toBe('Account banned: 上游原文：删除')
  })

  it('uses known scheduling codes and leaves custom backend status labels complete', () => {
    setI18nLocale('en-US')
    expect(getSchedulingBadgeLabel(poolKey({ scheduling_reason: 'inactive', scheduling_label: '已禁用' }), 0)).toBe('Disabled')
    expect(getSchedulingBadgeLabel(poolKey({ scheduling_reason: 'provider_custom', scheduling_label: ' 保存：用户自定义状态 ' }), 0)).toBe(' 保存：用户自定义状态 ')
  })
})
