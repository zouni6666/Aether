import { effectScope, nextTick, ref } from 'vue'
import { describe, expect, it, vi } from 'vitest'
import { useOverviewRequest } from '../useOverviewRequest'

describe('overview request lifecycle', () => {
  it('aborts replaced requests and never publishes a stale result', async () => {
    const source = ref('first')
    const requests: { signal: AbortSignal; resolve: (value: string) => void }[] = []
    const fetcher = vi.fn((signal: AbortSignal) => new Promise<string>(resolve => requests.push({ signal, resolve })))
    const scope = effectScope()
    const state = scope.run(() => useOverviewRequest(source, fetcher))!
    source.value = 'second'
    await nextTick()
    expect(requests[0]?.signal.aborted).toBe(true)
    requests[1]!.resolve('second result')
    await Promise.resolve()
    requests[0]!.resolve('first result')
    await Promise.resolve()
    expect(state.data.value).toBe('second result')
    scope.stop()
    expect(requests[1]?.signal.aborted).toBe(true)
  })
  it('retains the prior snapshot on failed manual refresh and exposes the error', async () => {
    const fetcher = vi.fn().mockResolvedValueOnce('snapshot').mockRejectedValueOnce(new Error('database timeout'))
    const scope = effectScope()
    const state = scope.run(() => useOverviewRequest(ref(1), fetcher))!
    await Promise.resolve()
    await state.refresh()
    expect(state.data.value).toBe('snapshot')
    expect(state.error.value).toBe('database timeout')
    scope.stop()
  })
  it('retains revision refresh snapshots but clears data when the filter scope changes', async () => {
    const revision = ref(0)
    const filter = ref('provider-a')
    const fetcher = vi.fn().mockResolvedValueOnce('provider-a snapshot').mockRejectedValue(new Error('database timeout'))
    const scope = effectScope()
    const state = scope.run(() => useOverviewRequest(() => [filter.value, revision.value], fetcher, { scopeKey: filter }))!
    await Promise.resolve()
    revision.value += 1
    await nextTick()
    await Promise.resolve()
    expect(state.data.value).toBe('provider-a snapshot')
    expect(state.error.value).toBe('database timeout')
    filter.value = 'provider-b'
    await nextTick()
    await Promise.resolve()
    expect(state.data.value).toBeNull()
    expect(state.error.value).toBe('database timeout')
    scope.stop()
  })
})
