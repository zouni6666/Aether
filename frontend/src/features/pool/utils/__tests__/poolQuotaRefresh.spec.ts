import { describe, expect, it } from 'vitest'

import type { PoolKeyDetail } from '@/api/endpoints/pool'
import type { RefreshQuotaResult } from '@/api/endpoints/keys'
import { mergePoolKeyQuotaSnapshots } from '@/features/pool/utils/poolQuotaRefresh'

function createKey(keyId: string): PoolKeyDetail {
  return {
    key_id: keyId,
    key_name: keyId,
    is_active: true,
    auth_type: 'oauth',
    api_formats: ['openai:responses'],
    internal_priority: 0,
    account_quota: null,
    cooldown_reason: null,
    cooldown_ttl_seconds: null,
    cost_window_usage: 0,
    cost_limit: null,
    request_count: 0,
    total_tokens: 0,
    total_cost_usd: '0',
    sticky_sessions: 0,
    lru_score: null,
    created_at: null,
    last_used_at: null,
  }
}

describe('mergePoolKeyQuotaSnapshots', () => {
  it('merges snapshots from non-success quota results', () => {
    const keys = [createKey('exhausted'), createKey('invalid'), createKey('unchanged')]
    const results: RefreshQuotaResult['results'] = [
      {
        key_id: 'exhausted',
        key_name: 'exhausted',
        status: 'quota_exhausted',
        quota_snapshot: {
          code: 'exhausted',
          exhausted: true,
          updated_at: 123,
        },
      },
      {
        key_id: 'invalid',
        key_name: 'invalid',
        status: 'auth_invalid',
        quota_snapshot: {
          code: 'unknown',
          exhausted: false,
          observed_at: 456,
        },
      },
      {
        key_id: 'unchanged',
        key_name: 'unchanged',
        status: 'error',
      },
    ]

    const merged = mergePoolKeyQuotaSnapshots(keys, results)

    expect(merged[0].status_snapshot?.quota.code).toBe('exhausted')
    expect(merged[0].quota_updated_at).toBe(123)
    expect(merged[1].status_snapshot?.quota.code).toBe('unknown')
    expect(merged[1].quota_updated_at).toBe(456)
    expect(merged[2]).toBe(keys[2])
  })
})
