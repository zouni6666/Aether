import type { QuotaWindowSnapshot } from '@/api/endpoints/types'

export function resetCodexCycleUsageWindows(
  windows: QuotaWindowSnapshot[],
  resetAt: number,
): QuotaWindowSnapshot[] {
  return windows.map((window) => {
    const code = String(window.code || '').trim()
    const scope = String(window.scope || 'account').trim().toLowerCase()
    const shouldReset = Boolean(code)
      && scope === 'account'
      && !code.toLowerCase().startsWith('spark_')
      && window.window_minutes !== 0

    if (!shouldReset) return window

    return {
      ...window,
      usage_reset_at: Number.isFinite(resetAt) ? resetAt : window.usage_reset_at,
      usage: {
        request_count: 0,
        total_tokens: 0,
        total_cost_usd: '0.00000000',
      },
    }
  })
}
