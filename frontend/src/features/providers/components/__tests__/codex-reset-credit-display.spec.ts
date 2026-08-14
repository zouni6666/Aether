import { describe, expect, it } from 'vitest'

import {
  clearPendingCodexResetCreditIdempotencyKey,
  clearPendingCodexResetCreditIdempotencyKeyForOutcome,
  createCodexResetCreditIdempotencyKey,
  formatCodexResetCreditCount,
  formatCodexResetCreditExpiresAt,
  getCodexResetCreditAvailableCount,
  getCodexResetCreditReservationIdempotencyKey,
  getVisibleCodexResetCreditItems,
  isCodexResetCreditTerminalOutcome,
  mergeCodexQuotaDisplays,
  readPendingCodexResetCreditIdempotencyKey,
  rememberPendingCodexResetCreditIdempotencyKey,
} from '@/features/providers/components/codex-reset-credit-display'
import type { QuotaResetCreditsSnapshot } from '@/api/endpoints/types'

describe('codex reset credit display helpers', () => {
  it('keeps reset credits and usage windows when snapshot sources are partially populated', () => {
    const merged = mergeCodexQuotaDisplays(
      {
        updated_at: 1_700_000_100,
        primary_used_percent: 25,
        reset_credits: {
          available_count: 2,
        },
      },
      {
        updated_at: 1_700_000_000,
        secondary_used_percent: 40,
        reset_credits: {
          credits: [{ id: 'credit-1', expires_at: 1_700_086_400 }],
        },
      },
    )

    expect(merged).toMatchObject({
      updated_at: 1_700_000_100,
      primary_used_percent: 25,
      secondary_used_percent: 40,
      reset_credits: {
        available_count: 2,
        credits: [{ id: 'credit-1', expires_at: 1_700_086_400 }],
      },
    })
  })

  it('keeps zero available credits displayable but non-positive detail items hidden', () => {
    const snapshot: QuotaResetCreditsSnapshot = {
      available_count: 0,
      updated_at: 1_700_000_000,
      credits: [
        {
          id: 'expired-1111',
          display_key: 'expired',
          status: 'available',
          expires_at: 1_699_999_999,
        },
      ],
    }

    expect(getCodexResetCreditAvailableCount(snapshot)).toBe(0)
    expect(formatCodexResetCreditCount(0)).toBe('共 0 次机会')
    expect(getVisibleCodexResetCreditItems(snapshot, 1_700_000_000)).toEqual([])
  })

  it('recovers the active idempotency key from a persisted server reservation', () => {
    expect(getCodexResetCreditReservationIdempotencyKey({
      reset_credits: { available_count: 0 },
      account_quota_reset_reservation: {
        idempotency_key: ' server-active-attempt ',
        generation: 3,
      },
    })).toBe('server-active-attempt')
    expect(getCodexResetCreditReservationIdempotencyKey({
      account_quota_reset_reservation: { idempotency_key: ' ' },
    })).toBeNull()
  })

  it('sorts available detail items by remaining time and labels visible items with short ordinal keys', () => {
    const snapshot: QuotaResetCreditsSnapshot = {
      available_count: 7,
      updated_at: 1_700_000_000,
      credits: [
        { id: 'sixth-0000', status: 'available', expires_at: 1_700_060_000 },
        { id: 'spent-0000', status: 'redeemed', expires_at: 1_700_001_000 },
        { id: 'fifth-0000', status: 'active', expires_at: 1_700_050_000 },
        { id: 'third-0000', status: 'available', remaining_seconds: 30_000 },
        { id: 'missing-expiry-0000', status: 'available' },
        { id: 'first-0000', status: 'available', expires_at: 1_700_010_000 },
        { id: 'second-0000', status: 'available', expires_at: 1_700_020_000 },
        {
          id: 'fourth-0000',
          display_key: 'RateLimitResetCredit_05cbb6eeeb9c81918e011d8300f9ebfb',
          status: 'available',
          expires_at: 1_700_040_000,
        },
      ],
    }

    const items = getVisibleCodexResetCreditItems(snapshot, 1_700_000_000)

    expect(items.map(item => item.displayKey)).toEqual([
      'Key-1',
      'Key-2',
      'Key-3',
      'Key-4',
      'Key-5',
    ])
    expect(items.map(item => item.title)).toEqual([
      'Codex 重置机会 Key-1',
      'Codex 重置机会 Key-2',
      'Codex 重置机会 Key-3',
      'Codex 重置机会 Key-4',
      'Codex 重置机会 Key-5',
    ])
    expect(items.map(item => item.remainingSeconds)).toEqual([
      10_000,
      20_000,
      30_000,
      40_000,
      50_000,
    ])
  })

  it('formats reset credit expiry as a precise local timestamp', () => {
    const expiresAt = new Date(2026, 6, 12, 22, 4, 41).getTime() / 1000
    expect(formatCodexResetCreditExpiresAt(expiresAt)).toBe('07-12 22:04:41')
    expect(formatCodexResetCreditExpiresAt(null)).toBe('-')
  })

  it('derives a stable expiry timestamp from remaining seconds', () => {
    const snapshot: QuotaResetCreditsSnapshot = {
      available_count: 1,
      updated_at: 1_700_000_000,
      credits: [{ status: 'available', remaining_seconds: 600 }],
    }

    expect(getVisibleCodexResetCreditItems(snapshot, 1_700_000_300)[0]?.expiresAt)
      .toBe(1_700_000_600)
  })

  it('generates a UUID v4 with secure random bytes when randomUUID is unavailable', () => {
    const idempotencyKey = createCodexResetCreditIdempotencyKey({
      getRandomValues(array) {
        array.set(Array.from({ length: 16 }, (_, index) => index))
        return array
      },
    })

    expect(idempotencyKey).toBe('00010203-0405-4607-8809-0a0b0c0d0e0f')
  })

  it('prefers the browser randomUUID implementation when available', () => {
    expect(createCodexResetCreditIdempotencyKey({
      randomUUID: () => 'existing-random-uuid',
      getRandomValues: array => array,
    })).toBe('existing-random-uuid')
  })

  it('keeps an unresolved reset idempotency key until a terminal response clears it', () => {
    const values = new Map<string, string>()
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
      removeItem: (key: string) => values.delete(key),
    }

    rememberPendingCodexResetCreditIdempotencyKey(
      'key-1',
      'reset-attempt-1',
      'credential-v1',
      storage,
    )
    expect(readPendingCodexResetCreditIdempotencyKey('key-1', 'credential-v1', storage))
      .toBe('reset-attempt-1')
    expect(readPendingCodexResetCreditIdempotencyKey('key-2', 'credential-v1', storage)).toBeNull()

    clearPendingCodexResetCreditIdempotencyKey('key-1', storage)
    expect(readPendingCodexResetCreditIdempotencyKey('key-1', 'credential-v1', storage)).toBeNull()
  })

  it('clears pending idempotency keys for compatibility history replays', () => {
    const values = new Map<string, string>()
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
      removeItem: (key: string) => values.delete(key),
    }

    rememberPendingCodexResetCreditIdempotencyKey(
      'key-1',
      'reset-attempt-1',
      'credential-v1',
      storage,
    )
    expect(isCodexResetCreditTerminalOutcome('historical_replay')).toBe(true)
    expect(clearPendingCodexResetCreditIdempotencyKeyForOutcome(
      'key-1',
      'historical_replay',
      storage,
    )).toBe(true)
    expect(readPendingCodexResetCreditIdempotencyKey('key-1', 'credential-v1', storage)).toBeNull()

    rememberPendingCodexResetCreditIdempotencyKey(
      'key-1',
      'reset-attempt-2',
      'credential-v1',
      storage,
    )
    expect(clearPendingCodexResetCreditIdempotencyKeyForOutcome('key-1', 'unknown', storage))
      .toBe(false)
    expect(readPendingCodexResetCreditIdempotencyKey('key-1', 'credential-v1', storage))
      .toBe('reset-attempt-2')
    expect(isCodexResetCreditTerminalOutcome('unknown')).toBe(false)
    expect(isCodexResetCreditTerminalOutcome('error')).toBe(false)
  })

  it('keeps the active idempotency key in memory when session storage is unavailable', () => {
    const unavailableStorage = {
      getItem: () => { throw new Error('storage unavailable') },
      setItem: () => { throw new Error('storage unavailable') },
      removeItem: () => { throw new Error('storage unavailable') },
    }

    rememberPendingCodexResetCreditIdempotencyKey(
      'key-without-storage',
      'reset-attempt-original',
      'credential-v1',
      unavailableStorage,
    )
    expect(readPendingCodexResetCreditIdempotencyKey(
      'key-without-storage',
      'credential-v1',
      unavailableStorage,
    ))
      .toBe('reset-attempt-original')

    rememberPendingCodexResetCreditIdempotencyKey(
      'key-without-storage',
      'reset-attempt-from-conflict',
      'credential-v1',
      unavailableStorage,
    )
    expect(readPendingCodexResetCreditIdempotencyKey(
      'key-without-storage',
      'credential-v1',
      unavailableStorage,
    ))
      .toBe('reset-attempt-from-conflict')

    clearPendingCodexResetCreditIdempotencyKey('key-without-storage', unavailableStorage)
    expect(readPendingCodexResetCreditIdempotencyKey(
      'key-without-storage',
      'credential-v1',
      unavailableStorage,
    ))
      .toBeNull()
  })

  it('does not resurrect a cleared key when session storage removal fails', () => {
    const values = new Map<string, string>()
    const partiallyUnavailableStorage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
      removeItem: () => { throw new Error('storage removal unavailable') },
    }

    rememberPendingCodexResetCreditIdempotencyKey(
      'key-with-stale-storage',
      'terminal-attempt',
      'credential-v1',
      partiallyUnavailableStorage,
    )
    clearPendingCodexResetCreditIdempotencyKey(
      'key-with-stale-storage',
      partiallyUnavailableStorage,
    )

    expect(readPendingCodexResetCreditIdempotencyKey(
      'key-with-stale-storage',
      'credential-v1',
      partiallyUnavailableStorage,
    )).toBeNull()
  })

  it('drops a pending attempt when the credential generation changes', () => {
    const values = new Map<string, string>()
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
      removeItem: (key: string) => values.delete(key),
    }

    rememberPendingCodexResetCreditIdempotencyKey(
      'key-rotated',
      'attempt-for-account-a',
      'credential-a',
      storage,
    )
    expect(readPendingCodexResetCreditIdempotencyKey('key-rotated', 'credential-b', storage))
      .toBeNull()
    expect([...values.values()]).toEqual([])
  })

  it('never replays a legacy v1 pending value without a credential generation', () => {
    const values = new Map<string, string>([
      ['aether:codex-reset-credit-pending:v1:key-legacy', 'legacy-attempt'],
    ])
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
      removeItem: (key: string) => values.delete(key),
    }

    expect(readPendingCodexResetCreditIdempotencyKey('key-legacy', 'credential-current', storage))
      .toBeNull()
    expect([...values.values()]).toEqual([])
  })
})
