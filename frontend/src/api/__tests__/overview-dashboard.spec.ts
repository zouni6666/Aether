import { beforeEach, describe, expect, it, vi } from 'vitest'

const { getMock } = vi.hoisted(() => ({ getMock: vi.fn() }))

vi.mock('@/api/client', () => ({
  default: { get: getMock },
}))

import { overviewApi, type OverviewQuery } from '@/api/overview'

describe('overview dashboard API contract', () => {
  beforeEach(() => {
    getMock.mockReset()
  })

  it('loads the compact flat dashboard snapshot with timezone and cancellation', async () => {
    const signal = new AbortController().signal
    const response = { stats_since: '2026-09-19T00:00:00Z', today: { request_count: 3 }, total: { request_count: 42 } }
    getMock.mockResolvedValueOnce({ data: response })
    expect(await overviewApi.dashboardSummary('Asia/Shanghai', signal)).toBe(response)
    expect(getMock).toHaveBeenCalledWith('/api/admin/overview/dashboard/summary', { params: { timezone: 'Asia/Shanghai' }, signal })
  })

  it.each([true, false, null])('loads today and lifetime totals with history_complete=%s using only the timezone', async (historyComplete) => {
    const signal = new AbortController().signal
    const response = {
      today: { data: { request_count: 2 } },
      total: { data: { request_count: 42 } },
      history_complete: historyComplete,
    }
    getMock.mockResolvedValueOnce({ data: response })

    const result = await overviewApi.dashboard('America/New_York', signal)

    expect(getMock).toHaveBeenCalledTimes(1)
    expect(getMock).toHaveBeenCalledWith('/api/admin/overview/dashboard', {
      params: { timezone: 'America/New_York' },
      signal,
    })
    expect(result).toBe(response)
  })

  it.each([{ status: 'pending' }, { status: 'ready', total: { data: { request_count: 42 } }, history_complete: null, stale: true }])(
    'loads independently computed lifetime totals: $status', async (response) => {
      const signal = new AbortController().signal
      getMock.mockResolvedValueOnce({ data: response })

      expect(await overviewApi.dashboardTotal('Asia/Shanghai', signal)).toBe(response)
      expect(getMock).toHaveBeenCalledWith('/api/admin/overview/dashboard/total', {
        params: { timezone: 'Asia/Shanghai' }, signal,
      })
    },
  )

  it('loads full-site daily charts without leaking filters, pagination, or hourly granularity', async () => {
    const signal = new AbortController().signal
    const query: OverviewQuery = {
      from: '2026-03-08T05:12:34.000Z',
      to: '2026-03-09T04:00:00.000Z',
      timezone: 'America/New_York',
      user_id: 'employee-1',
      api_key_id: 'key-1',
      credential_owner_id: 'owner-1',
      attribution_kind: 'employee',
      provider_id: 'provider-1',
      model: 'model-1',
      request_type: 'chat',
      api_format: 'openai',
      endpoint_kind: 'chat',
      is_stream: false,
      has_format_conversion: false,
      slow_threshold_ms: 1000,
      status: 'success',
      search: 'employee',
      account_status: 'active',
      usage_status: 'active',
      amount_basis: 'rated',
      group_by: 'provider',
      sort: 'request_count',
      order: 'asc',
      granularity: 'hour',
      limit: 1,
      offset: 10,
    }
    const response = {
      meta: { read_revision: 'dashboard-chart-revision' },
      data: {
        summary: { request_count: 5 },
        series: [{ bucket_start: '2026-03-08T05:00:00.000Z' }],
        models: [{ id: 'model-1', bucket_start: '2026-03-08T05:00:00.000Z' }],
        providers: [{ id: 'provider-1' }, { id: 'provider-2' }, { id: null }],
      },
    }
    getMock.mockResolvedValueOnce({ data: response })

    const result = await overviewApi.dashboardCharts(query, signal)

    expect(getMock).toHaveBeenCalledTimes(1)
    expect(getMock).toHaveBeenCalledWith('/api/admin/overview/dashboard/charts', {
      params: {
        from: query.from,
        to: query.to,
        timezone: query.timezone,
        granularity: 'day',
      },
      signal,
    })
    expect(result).toBe(response)
  })
})
