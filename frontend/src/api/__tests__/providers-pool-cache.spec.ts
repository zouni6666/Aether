import { beforeEach, describe, expect, it, vi } from 'vitest'

const { getMock, patchMock } = vi.hoisted(() => ({ getMock: vi.fn(), patchMock: vi.fn() }))

vi.mock('@/api/client', () => ({ default: { get: getMock, patch: patchMock } }))

import { getPoolOverview, listPoolKeys, listPoolScores } from '@/api/endpoints/pool'
import { getProvider, updateProvider } from '@/api/endpoints/providers'
import { cache } from '@/utils/cache'

const options = { cacheTtlMs: 30_000 }
const provider = { id: 'codex', pool_advanced: { reserve_minimum_quota: false } }

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((resolvePromise) => { resolve = resolvePromise })
  return { promise, resolve }
}

beforeEach(() => {
  cache.clear()
  getMock.mockReset()
  patchMock.mockReset()
  patchMock.mockResolvedValue({ data: provider })
})

describe('provider pool settings cache invalidation', () => {
  it('refreshes every cached key page, score page and overview after changing pool settings', async () => {
    getMock.mockResolvedValue({ data: { version: 'old' } })
    await getPoolOverview(options)
    await listPoolKeys('codex', {}, options)
    await listPoolKeys('codex', { page: 2, status: 'quota_exhausted' }, options)
    await listPoolScores('codex', {}, options)
    await listPoolKeys('codex-other', {}, options)
    await updateProvider('codex', { pool_advanced: { reserve_minimum_quota: false } })
    getMock.mockResolvedValue({ data: { version: 'new' } })

    await expect(getPoolOverview(options)).resolves.toEqual({ version: 'new' })
    await expect(listPoolKeys('codex', {}, options)).resolves.toEqual({ version: 'new' })
    await expect(listPoolKeys('codex', { page: 2, status: 'quota_exhausted' }, options))
      .resolves.toEqual({ version: 'new' })
    await expect(listPoolScores('codex', {}, options)).resolves.toEqual({ version: 'new' })
    await expect(listPoolKeys('codex-other', {}, options)).resolves.toEqual({ version: 'old' })
  })

  it('does not reuse or cache a key request started before the settings were saved', async () => {
    const oldResponse = deferred<{ data: { version: string } }>()
    const newResponse = deferred<{ data: { version: string } }>()
    getMock.mockReturnValueOnce(oldResponse.promise).mockReturnValueOnce(newResponse.promise)
    const oldRequest = listPoolKeys('codex', { page: 2 }, options)
    await updateProvider('codex', { pool_advanced: { reserve_minimum_quota: false } })
    const newRequest = listPoolKeys('codex', { page: 2 }, options)
    expect(getMock).toHaveBeenCalledTimes(2)

    oldResponse.resolve({ data: { version: 'old' } })
    await oldRequest
    const deduped = listPoolKeys('codex', { page: 2 }, options)
    expect(getMock).toHaveBeenCalledTimes(2)
    newResponse.resolve({ data: { version: 'new' } })
    await expect(newRequest).resolves.toEqual({ version: 'new' })
    await expect(deduped).resolves.toEqual({ version: 'new' })
    await expect(listPoolKeys('codex', { page: 2 }, options)).resolves.toEqual({ version: 'new' })
  })

  it('does not reuse a provider detail request started before a successful save', async () => {
    const oldResponse = deferred<{ data: typeof provider }>()
    getMock.mockReturnValueOnce(oldResponse.promise).mockResolvedValueOnce({ data: provider })
    const oldRequest = getProvider('codex')
    await updateProvider('codex', { pool_advanced: { reserve_minimum_quota: false } })
    const newRequest = getProvider('codex')
    expect(getMock).toHaveBeenCalledTimes(2)
    oldResponse.resolve({ data: { ...provider, pool_advanced: { reserve_minimum_quota: true } } })
    await oldRequest
    await expect(newRequest).resolves.toMatchObject(provider)
  })
})
