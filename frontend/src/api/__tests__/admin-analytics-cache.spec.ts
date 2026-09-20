import { beforeEach, describe, expect, it, vi } from 'vitest'

const { getMock, cachedRequestMock, buildCacheKeyMock } = vi.hoisted(() => ({
  getMock: vi.fn(),
  cachedRequestMock: vi.fn(async (_key: string, fetcher: () => Promise<unknown>) => fetcher()),
  buildCacheKeyMock: vi.fn((prefix: string) => prefix),
}))

vi.mock('@/api/client', () => ({
  default: {
    get: getMock,
  },
}))

vi.mock('@/utils/cache', () => ({
  cache: {
    clear: vi.fn(),
    delete: vi.fn(),
  },
  cachedRequest: cachedRequestMock,
  buildCacheKey: buildCacheKeyMock,
}))

import { adminApi } from '@/api/admin'

describe('adminApi analytics cache options', () => {
  const params = {
    start_date: '2026-07-01',
    end_date: '2026-07-15',
    preset: 'custom',
    timezone: 'Asia/Shanghai',
    tz_offset_minutes: 480,
  }

  beforeEach(() => {
    getMock.mockReset()
    getMock.mockResolvedValue({ data: {} })
    cachedRequestMock.mockClear()
    buildCacheKeyMock.mockClear()
  })

  it('keeps the existing 20-second cache TTL by default', async () => {
    await adminApi.getTimeSeries(params)
    await adminApi.getPercentiles(params)
    await adminApi.getProviderPerformance(params)
    await adminApi.getErrorDistribution(params)

    for (let call = 1; call <= 4; call += 1) {
      expect(cachedRequestMock).toHaveBeenNthCalledWith(
        call,
        expect.any(String),
        expect.any(Function),
        20 * 1000
      )
    }
  })

  it('uses a zero TTL when an analytics request skips the cache', async () => {
    const options = { skipCache: true }
    const providerParams = { ...params, include_timeline: false }

    await adminApi.getTimeSeries(params, options)
    await adminApi.getPercentiles(params, options)
    await adminApi.getProviderPerformance(providerParams, options)
    await adminApi.getErrorDistribution(params, options)

    for (let call = 1; call <= 4; call += 1) {
      expect(cachedRequestMock).toHaveBeenNthCalledWith(
        call,
        expect.any(String),
        expect.any(Function),
        0
      )
    }

    expect(getMock).toHaveBeenNthCalledWith(1, '/api/admin/stats/time-series', { params })
    expect(getMock).toHaveBeenNthCalledWith(2, '/api/admin/stats/performance/percentiles', {
      params,
    })
    expect(getMock).toHaveBeenNthCalledWith(3, '/api/admin/stats/performance/providers', {
      params: providerParams,
    })
    expect(getMock).toHaveBeenNthCalledWith(4, '/api/admin/stats/errors/distribution', { params })
  })

  it('requests the user group leaderboard with scoped cache parameters', async () => {
    const groupParams = {
      ...params,
      metric: 'cost' as const,
      offset: 10,
      limit: 10,
      include_inactive: true,
    }
    getMock.mockResolvedValueOnce({
      data: { items: [], total: 0, metric: 'cost', attribution: 'current_membership' },
    })

    await expect(adminApi.getLeaderboardUserGroups(groupParams)).resolves.toMatchObject({
      attribution: 'current_membership',
    })

    expect(buildCacheKeyMock).toHaveBeenCalledWith(
      'admin:stats:leaderboard:user-groups',
      groupParams
    )
    expect(cachedRequestMock).toHaveBeenCalledWith(
      'admin:stats:leaderboard:user-groups',
      expect.any(Function),
      20 * 1000
    )
    expect(getMock).toHaveBeenCalledWith('/api/admin/stats/leaderboard/user-groups', {
      params: groupParams,
    })
  })

})
