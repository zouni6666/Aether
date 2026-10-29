import { afterEach, describe, expect, it } from 'vitest'
import { setI18nLocale } from '@/i18n'
import { getCodexResetCountdown, getOAuthExpiresCountdown } from '@/composables/useCountdownTimer'
import { CONDITION_OP_OPTIONS } from '@/features/providers/components/endpoint-rule-condition'
import { formatModelMappingRequestScope } from '../modelMappingScope'
import { parseProviderKeyBatchImport } from '../providerKeyBatchImport'
import { getOAuthRefreshFeedback } from '../oauthRefreshFeedback'
import { getOAuthStatusDisplayWithFallback } from '../providerKeyStatus'

afterEach(() => setI18nLocale('zh-CN'))

describe('provider locale switching', () => {
  it('updates shared option labels and generated validation messages', () => {
    expect(CONDITION_OP_OPTIONS[0].label).toBe('等于')
    expect(formatModelMappingRequestScope(undefined)).toBe('所有请求')
    setI18nLocale('en-US')
    expect(CONDITION_OP_OPTIONS[0].label).toBe('Equals')
    expect(formatModelMappingRequestScope(undefined)).toBe('All requests')
    expect(parseProviderKeyBatchImport('invalid entry').errors[0].message).toBe('Expected format: Name----Key')
  })

  it('keeps server error details verbatim while translating refresh feedback', () => {
    setI18nLocale('en-US')
    expect(getOAuthRefreshFeedback({
      accountStateRecheckAttempted: true,
      accountStateRecheckError: '上游返回原文',
    })).toEqual({
      tone: 'warning',
      message: 'Token refreshed, but quota/account status recheck failed: 上游返回原文',
    })
    expect(getOAuthStatusDisplayWithFallback({ auth_type: 'oauth', oauth_temporary: true }, 0)).toMatchObject({
      kind: 'missing_refresh_token',
      text: 'Not added',
    })
  })

  it('formats multi-day quota and token countdowns in English', () => {
    setI18nLocale('en-US')
    const expiresAt = Math.floor(Date.now() / 1000) + 3 * 86400 + 2 * 3600
    expect(getOAuthExpiresCountdown(expiresAt, 0, null, null)?.text).toBe('3d 2h')
    expect(getCodexResetCountdown(expiresAt, null, null, 0, null)?.text).toMatch(/^3d 2:00:00$/)
  })
})
