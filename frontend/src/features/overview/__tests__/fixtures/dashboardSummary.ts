import type { OverviewDashboardSummary } from '@/api/overview'

export function dashboardSummary(): OverviewDashboardSummary {
  const amount = (value: string) => ({ value, currency: 'USD', basis: 'billable', status: 'known' })
  return {
    stats_since: '2026-09-18T00:00:00Z', generated_at: '2026-09-19T04:00:00Z', timezone: 'UTC',
    today_from: '2026-09-19T00:00:00Z', window_seconds: 14400,
    today: {
      request_count: 103, input_tokens: 12000, output_tokens: 3456, total_tokens: 15456,
      billable_amount: amount('9.87'), active_users: 8,
      cache_read_tokens: 6000, cache_creation_tokens: 1000, cache_input_tokens: 20000,
      avg_first_byte_ms: 80, avg_response_ms: 1520, stream_requests: 90, standard_requests: 13,
    },
    total: { request_count: 12345, total_tokens: 1234567, billable_amount: amount('98.76'), cache_read_tokens: 360000, cache_input_tokens: 800000 },
    users: { total: 14, created_today: 2, deleted_today: 1 },
    consecutive_active_days: 2, active_days: 413,
    activity_days: [{ date: '2026-09-18', requests: 250 }, { date: '2026-09-19', requests: 103 }],
    concurrency: { avg: 2.5, peak: 8, observed_from: '2026-09-19T02:00:00Z', observed_through: '2026-09-19T04:00:00Z', scope: 'node', coverage: 'partial' },
  }
}
