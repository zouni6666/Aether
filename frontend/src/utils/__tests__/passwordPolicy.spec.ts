import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { setI18nLocale } from '@/i18n'
import {
  PASSWORD_POLICY_OPTIONS,
  getPasswordPolicyErrors,
  getPasswordPolicyHint,
  getPasswordPolicyPlaceholder,
  validatePasswordByPolicy,
} from '../passwordPolicy'

beforeEach(() => setI18nLocale('zh-CN'))
afterEach(() => setI18nLocale('zh-CN'))

describe('passwordPolicy utils', () => {
  it('rejects passwords longer than 72 bytes', () => {
    expect(getPasswordPolicyErrors('a'.repeat(80), 'weak')).toContain('长度不能超过72字节')
  })

  it('rejects multibyte passwords longer than 72 bytes', () => {
    expect(getPasswordPolicyErrors('中'.repeat(25), 'weak')).toContain('长度不能超过72字节')
  })

  it('formats validation errors into a single message', () => {
    expect(validatePasswordByPolicy('abc', 'strong')).toBe(
      '密码需要：至少 8 个字符、包含大写字母、包含数字、包含特殊字符',
    )
  })

  it('uses keyed English messages for validation and byte limits', () => {
    setI18nLocale('en-US')
    expect(validatePasswordByPolicy('abc', 'strong')).toBe(
      'Password requires: at least 8 characters, include uppercase letters, include numbers, include special characters',
    )
    expect(validatePasswordByPolicy('a'.repeat(80), 'weak')).toBe('Password length must not exceed 72 bytes')
    expect(getPasswordPolicyErrors('中'.repeat(25), 'weak')).toEqual(['length must not exceed 72 bytes'])
    expect(getPasswordPolicyErrors('Abc123!@', 'strong')).toEqual([])
  })

  it('renders exported policy options and hints in the current language', () => {
    expect(PASSWORD_POLICY_OPTIONS[0].label).toBe('弱密码')
    setI18nLocale('en-US')
    expect(PASSWORD_POLICY_OPTIONS[0].label).toBe('Weak password')
    expect(PASSWORD_POLICY_OPTIONS[1].description).toBe('At least 8 characters, including letters and numbers')
    expect(getPasswordPolicyHint('strong')).not.toMatch(/[\u4e00-\u9fff]/u)
    expect(getPasswordPolicyPlaceholder('medium')).toBe('At least 8 characters, with letters and numbers')
    setI18nLocale('zh-CN')
    expect(PASSWORD_POLICY_OPTIONS[0].description).toBe('至少 6 个字符')
  })
})
