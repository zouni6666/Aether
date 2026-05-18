import { describe, expect, it } from 'vitest'

import {
  getProviderMaskedSecretLabel,
  shouldShowOAuthRefreshControl,
} from '@/utils/providerKeyAuth'

describe('providerKeyAuth', () => {
  it('renders Grok OAuth-managed cookies as sessions without OAuth refresh controls', () => {
    const key = {
      auth_type: 'oauth',
      oauth_managed: true,
      can_refresh_oauth: false,
    }

    expect(getProviderMaskedSecretLabel(key, 'grok')).toBe('[Session Cookie]')
    expect(shouldShowOAuthRefreshControl(key, 'grok')).toBe(false)
  })

  it('keeps standard OAuth providers on OAuth token semantics', () => {
    const key = {
      auth_type: 'oauth',
      oauth_managed: true,
      can_refresh_oauth: false,
    }

    expect(getProviderMaskedSecretLabel(key, 'codex')).toBe('[OAuth Token]')
    expect(shouldShowOAuthRefreshControl(key, 'codex')).toBe(true)
  })
})
