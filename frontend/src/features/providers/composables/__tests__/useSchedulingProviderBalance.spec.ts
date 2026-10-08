import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, nextTick, ref, type App } from 'vue'
import type { ActionResultResponse } from '@/api/providerOps'
import { provideSchedulingProviderBalance, useSchedulingProviderBalance } from '../useSchedulingProviderBalance'

const api = vi.hoisted(() => ({
  batchQueryBalance: vi.fn<(providerIds: string[]) => Promise<Record<string, ActionResultResponse>>>(),
  getArchitectures: vi.fn().mockResolvedValue([]),
}))
vi.mock('@/api/providerOps', () => api)

let app: App | undefined
let root: HTMLDivElement
function mountBalance() {
  const revision = ref(0)
  let balance!: NonNullable<ReturnType<typeof useSchedulingProviderBalance>>
  const Child = defineComponent({ setup() { balance = useSchedulingProviderBalance()!; return () => null } })
  root = document.createElement('div')
  app = createApp({
    setup() { provideSchedulingProviderBalance(() => revision.value); return () => h(Child) },
  })
  app.mount(root)
  return { balance, revision }
}
function result(status: ActionResultResponse['status'], available = 0): ActionResultResponse {
  return { status, action_type: 'query_balance', data: { total_available: available, currency: 'USD', extra: {} }, message: null, executed_at: '2026-10-05T00:00:00Z', response_time_ms: 0, cache_ttl_seconds: 0 }
}
function provider(id: string, opsConfigured = true) { return { id, ops_configured: opsConfigured } }

beforeEach(() => { vi.useFakeTimers(); api.batchQueryBalance.mockReset() })
afterEach(() => { app?.unmount(); app = undefined; root?.remove(); vi.useRealTimers() })

describe('shared scheduling provider balances', () => {
  it('deduplicates providers repeated across model panels and skips unconfigured providers', async () => {
    const { balance } = mountBalance()
    api.batchQueryBalance.mockResolvedValue({ a: result('success', 0), b: result('success', 10) })
    balance.register(provider('a'))
    balance.register(provider('a'))
    balance.register(provider('b'))
    balance.register(provider('without-ops', false))
    await vi.advanceTimersByTimeAsync(80)
    expect(api.batchQueryBalance).toHaveBeenCalledExactlyOnceWith(['a', 'b'])
    expect(balance.getProviderBalance('a')).toEqual({ available: 0, currency: 'USD' })
    balance.register(provider('a'))
    await vi.advanceTimersByTimeAsync(80)
    expect(api.batchQueryBalance).toHaveBeenCalledOnce()
  })

  it('continues retrying pending balances when another model panel registers providers', async () => {
    const { balance } = mountBalance()
    api.batchQueryBalance
      .mockResolvedValueOnce({ a: result('pending') })
      .mockResolvedValueOnce({ a: result('pending'), b: result('success', 20) })
      .mockResolvedValueOnce({ a: result('success', 10) })
    balance.register(provider('a'))
    await vi.advanceTimersByTimeAsync(80)
    expect(balance.isBalanceLoading('a')).toBe(true)
    balance.register(provider('b'))
    await vi.advanceTimersByTimeAsync(80)
    await vi.advanceTimersByTimeAsync(12_000)
    expect(api.batchQueryBalance.mock.calls).toEqual([[['a']], [['a', 'b']], [['a']]])
    expect(balance.getProviderBalance('a')).toEqual({ available: 10, currency: 'USD' })
    expect(balance.getProviderBalance('b')).toEqual({ available: 20, currency: 'USD' })
  })

  it('serializes loads and coalesces registrations and revisions arriving during a request', async () => {
    const { balance, revision } = mountBalance()
    let finishFirst!: (results: Record<string, ActionResultResponse>) => void
    api.batchQueryBalance
      .mockImplementationOnce(() => new Promise(resolve => { finishFirst = resolve }))
      .mockResolvedValueOnce({ a: result('success', 12), b: result('success', 20), c: result('success', 30) })
    balance.register(provider('a'))
    await vi.advanceTimersByTimeAsync(80)
    balance.register(provider('b'))
    balance.register(provider('c'))
    revision.value += 1
    await nextTick()
    await vi.advanceTimersByTimeAsync(800)
    expect(api.batchQueryBalance).toHaveBeenCalledOnce()
    finishFirst({ a: result('success', 10) })
    await vi.advanceTimersByTimeAsync(80)
    expect(api.batchQueryBalance.mock.calls).toEqual([[['a']], [['a', 'b', 'c']]])
    expect(balance.getProviderBalance('a')?.available).toBe(12)
  })

  it('refreshes previously registered balances when provider details change', async () => {
    const { balance, revision } = mountBalance()
    api.batchQueryBalance.mockResolvedValueOnce({ a: result('success', 10) }).mockResolvedValueOnce({ a: result('success', 15) })
    balance.register(provider('a'))
    await vi.advanceTimersByTimeAsync(80)
    revision.value += 1
    await nextTick()
    await vi.advanceTimersByTimeAsync(80)
    expect(api.batchQueryBalance).toHaveBeenCalledTimes(2)
    expect(balance.getProviderBalance('a')?.available).toBe(15)
  })

  it('removes balances when ops becomes unconfigured during a request', async () => {
    const { balance } = mountBalance()
    let finish!: (results: Record<string, ActionResultResponse>) => void
    api.batchQueryBalance.mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    balance.register(provider('a'))
    await vi.advanceTimersByTimeAsync(80)
    balance.register(provider('a', false))
    finish({ a: result('pending') })
    await vi.advanceTimersByTimeAsync(80)
    await vi.advanceTimersByTimeAsync(12_000)
    expect(balance.balanceCache.value).toEqual({})
    expect(api.batchQueryBalance).toHaveBeenCalledOnce()
  })

  it('cancels queued and pending work when the workspace unmounts', async () => {
    const { balance } = mountBalance()
    let finish!: (results: Record<string, ActionResultResponse>) => void
    api.batchQueryBalance.mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    balance.register(provider('a'))
    await vi.advanceTimersByTimeAsync(80)
    balance.register(provider('b'))
    app?.unmount()
    app = undefined
    finish({ a: result('pending') })
    await vi.advanceTimersByTimeAsync(20_000)
    expect(api.batchQueryBalance).toHaveBeenCalledOnce()
    expect(balance.balanceCache.value).toEqual({})
    expect(vi.getTimerCount()).toBe(0)
    balance.register(provider('c'))
    expect(vi.getTimerCount()).toBe(0)
  })
})
