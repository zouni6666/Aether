import { describe, expect, it } from 'vitest'
import { messages, translateMessage, type MessageKey } from '@/i18n/messages'

const placeholders = (message: string) => [...message.matchAll(/\{(\w+)\}/g)]
  .map(match => match[1]).sort()

describe('translation coverage', () => {
  it('keeps locale keys and interpolation parameters aligned', () => {
    expect(Object.keys(messages['en-US']).sort()).toEqual(Object.keys(messages['zh-CN']).sort())
    for (const key of Object.keys(messages['zh-CN']) as MessageKey[]) {
      expect(placeholders(messages['en-US'][key]), key).toEqual(placeholders(messages['zh-CN'][key]))
      expect(messages['en-US'][key], key).not.toMatch(/[\u4e00-\u9fff]/u)
    }
  })

  it('interpolates parameters once without interpreting their contents as messages', () => {
    expect(translateMessage('auth.login.title', 'en-US', { siteName: '保存 {other}' }))
      .toBe('Sign in to 保存 {other}')
    expect(translateMessage('site.privacy.currentVersion', 'en-US'))
      .toBe('Current version: {version}')
  })
})
