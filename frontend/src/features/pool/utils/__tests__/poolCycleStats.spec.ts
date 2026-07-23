import { describe, expect, it } from 'vitest'

import type { QuotaWindowSnapshot } from '@/api/endpoints/types'
import { resetCodexCycleUsageWindows } from '@/features/pool/utils/poolCycleStats'

describe('resetCodexCycleUsageWindows', () => {
  it('mirrors the backend reset scope and creates zero usage for resettable windows', () => {
    const windows: QuotaWindowSnapshot[] = [
      {
        code: '5h',
        scope: 'account',
        window_minutes: 300,
        usage: null,
      },
      {
        code: 'spark_weekly',
        scope: 'account',
        window_minutes: 10_080,
        usage: { request_count: 3, total_tokens: 30, total_cost_usd: '0.03' },
      },
      {
        code: 'model_daily',
        scope: 'model',
        window_minutes: 1_440,
        usage: { request_count: 4, total_tokens: 40, total_cost_usd: '0.04' },
      },
      {
        code: 'lifetime',
        scope: 'account',
        window_minutes: 0,
        usage: { request_count: 5, total_tokens: 50, total_cost_usd: '0.05' },
      },
    ]

    const result = resetCodexCycleUsageWindows(windows, 123)

    expect(result[0]).toMatchObject({
      usage_reset_at: 123,
      usage: { request_count: 0, total_tokens: 0, total_cost_usd: '0.00000000' },
    })
    expect(result[1]).toBe(windows[1])
    expect(result[2]).toBe(windows[2])
    expect(result[3]).toBe(windows[3])
  })

  it('treats a missing scope as account and ignores empty codes', () => {
    const windows: QuotaWindowSnapshot[] = [
      { code: 'weekly', usage: { request_count: 7 } },
      { code: '   ', scope: 'account', usage: { request_count: 8 } },
    ]

    const result = resetCodexCycleUsageWindows(windows, 456)

    expect(result[0].usage?.request_count).toBe(0)
    expect(result[0].usage_reset_at).toBe(456)
    expect(result[1]).toBe(windows[1])
  })
})
