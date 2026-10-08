import type { IntervalTimelineResponse } from '@/api/cache'
import type { DailyStat, DailyStatsResponse, ModelBreakdown } from '@/api/dashboard'
import type { OverviewDashboardSummary } from '@/api/overview'
import type { DateRangeParams } from '@/features/usage/types'

const DAY_MS = 86_400_000
const HOUR_MS = 3_600_000
const modelNames = ['gpt-5.4', 'claude-sonnet-4', 'gemini-2.5-pro']
const providerNames = ['OpenAI', 'Anthropic', 'Google']

function dateKey(instant: number, timezone: string): string {
  const parts = new Intl.DateTimeFormat('en-CA', {
    timeZone: timezone, year: 'numeric', month: '2-digit', day: '2-digit',
  }).formatToParts(new Date(instant))
  const part = (type: string) => parts.find(item => item.type === type)?.value ?? ''
  return `${part('year')}-${part('month')}-${part('day')}`
}

function shiftDate(date: string, days: number): string {
  return new Date(Date.parse(`${date}T00:00:00Z`) + days * DAY_MS).toISOString().slice(0, 10)
}

function dayStart(date: string, timezone: string): number {
  // Locate the actual local date boundary, including midnight DST transitions.
  const nominal = Date.parse(`${date}T00:00:00Z`)
  let left = nominal - 36 * HOUR_MS
  let right = nominal + 36 * HOUR_MS
  while (left < right) {
    const middle = Math.floor((left + right) / 2)
    if (dateKey(middle, timezone) < date) left = middle + 1
    else right = middle
  }
  return left
}

function dailyModels(date: string, today: string): ModelBreakdown[] {
  const age = Math.round((Date.parse(`${today}T00:00:00Z`) - Date.parse(`${date}T00:00:00Z`)) / DAY_MS)
  const serial = Math.floor(Date.parse(`${date}T00:00:00Z`) / DAY_MS)
  const total = age < 0 || age >= 90 ? 0 : 1800 + ((serial % 13 + 13) % 13) * 120
  const requests = [Math.floor(total * 0.5), Math.floor(total * 0.3)]
  requests.push(total - requests[0] - requests[1])
  return modelNames.map((model, index) => ({
    model, requests: requests[index], tokens: requests[index] * [1100, 1600, 850][index],
    cost: requests[index] * [3, 5, 2][index] / 100,
  }))
}

function dailyRow(date: string, models: ModelBreakdown[]): DailyStat {
  const requests = models.reduce((sum, model) => sum + model.requests, 0)
  return {
    date, requests, tokens: models.reduce((sum, model) => sum + model.tokens, 0),
    cost: Number(models.reduce((sum, model) => sum + model.cost, 0).toFixed(2)),
    avg_response_time: requests ? 1.52 : 0, unique_models: requests ? models.length : 0,
    unique_providers: requests ? providerNames.length : 0, model_breakdown: models,
  }
}

/** Synthetic display data only; no account, usage, or billing writes. */
export function createDashboardDemo(timezone: string): OverviewDashboardSummary {
  const now = Date.now()
  const today = dateKey(now, timezone)
  const todayFrom = dayStart(today, timezone)
  const activityDays = Array.from({ length: 365 }, (_, index) => {
    const date = shiftDate(today, index - 364)
    return { date, requests: dailyModels(date, today).reduce((sum, model) => sum + model.requests, 0) }
  })
  const days = activityDays.slice(-90).map(day => dailyRow(day.date, dailyModels(day.date, today)))
  const current = days[days.length - 1]
  const input = Math.round(current.tokens * 0.75)
  const stream = Math.round(current.requests * 0.91)
  const amount = (value: number) => ({ value: value.toFixed(8), currency: 'USD', basis: 'billable', status: 'known' })
  const observedFrom = Math.max(todayFrom, now - 2 * HOUR_MS)
  return {
    stats_since: new Date(dayStart(shiftDate(today, -89), timezone)).toISOString(),
    generated_at: new Date(now).toISOString(), timezone,
    today_from: new Date(todayFrom).toISOString(), window_seconds: Math.floor((now - todayFrom) / 1000),
    today: {
      request_count: current.requests, input_tokens: input, output_tokens: current.tokens - input,
      total_tokens: current.tokens, billable_amount: amount(current.cost), active_users: 24,
      cache_read_tokens: Math.round(input * 0.63), cache_creation_tokens: Math.round(input * 0.12), cache_input_tokens: input,
      avg_first_byte_ms: 247, avg_response_ms: 1520, stream_requests: stream, standard_requests: current.requests - stream,
    },
    total: {
      request_count: days.reduce((sum, day) => sum + day.requests, 0),
      total_tokens: days.reduce((sum, day) => sum + day.tokens, 0),
      billable_amount: amount(days.reduce((sum, day) => sum + day.cost, 0)),
      cache_read_tokens: Math.round(days.reduce((sum, day) => sum + Math.round(day.tokens * 0.75), 0) * 0.58),
      cache_input_tokens: days.reduce((sum, day) => sum + Math.round(day.tokens * 0.75), 0),
    },
    users: { total: 63, created_today: 4, deleted_today: 1 }, consecutive_active_days: 90, active_days: 90, activity_days: activityDays,
    concurrency: {
      avg: 6.84, peak: 23, observed_from: new Date(observedFrom).toISOString(), observed_through: new Date(now).toISOString(),
      scope: 'node', coverage: observedFrom > todayFrom ? 'partial' : 'complete',
    },
  }
}

export function createDashboardTimelineDemo(): IntervalTimelineResponse {
  const now = Date.now()
  const userIds = ['demo-dev', 'demo-production', 'demo-sandbox']
  const points = Array.from({ length: 108 }, (_, index) => ({
    x: new Date(now - DAY_MS + (index + 1) * DAY_MS / 109).toISOString(),
    y: Number((1.5 + (index * 17 % 240) + (index % 3) * 0.4).toFixed(1)),
    user_id: userIds[index % userIds.length], model: modelNames[index % modelNames.length],
  }))
  return {
    analysis_period_hours: 24, total_points: points.length, points,
    users: { 'demo-dev': '开发团队', 'demo-production': '生产应用', 'demo-sandbox': '测试环境' }, models: [...modelNames],
  }
}

export function createDashboardDailyDemo(params: DateRangeParams): DailyStatsResponse {
  const timezone = params.timezone || Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC'
  const now = Date.now()
  const today = dateKey(now, timezone)
  const periodDays = params.preset === 'last90days' ? 90 : params.preset === 'last30days' ? 30
    : params.preset === 'today' || params.preset === 'yesterday' ? 1 : 7
  const preciseFrom = params.from ? Date.parse(params.from) : NaN
  const preciseTo = params.to ? Date.parse(params.to) : NaN
  const precise = Number.isFinite(preciseFrom) && Number.isFinite(preciseTo) && preciseFrom < preciseTo
  const validDate = (value: string | undefined) => value && /^\d{4}-\d{2}-\d{2}$/.test(value) && Number.isFinite(Date.parse(value)) ? value : undefined
  let endDate = precise ? dateKey(preciseTo - 1, timezone)
    : validDate(params.end_date) || shiftDate(today, params.preset === 'yesterday' ? -1 : 0)
  let startDate = precise ? dateKey(preciseFrom, timezone)
    : validDate(params.start_date) || shiftDate(endDate, 1 - periodDays)
  if (startDate > endDate) [startDate, endDate] = [endDate, startDate]
  startDate = startDate < shiftDate(endDate, -365) ? shiftDate(endDate, -365) : startDate
  const days = Math.round((Date.parse(endDate) - Date.parse(startDate)) / DAY_MS) + 1
  const rows: DailyStat[] = []
  for (let index = 0; index < days; index += 1) {
    const date = shiftDate(startDate, index)
    const models = dailyModels(date, today)
    if (params.granularity !== 'hour') {
      rows.push(dailyRow(date, models))
      continue
    }
    const from = dayStart(date, timezone)
    const to = Math.min(dayStart(shiftDate(date, 1), timezone), now + 1)
    const hours = Math.max(0, Math.ceil((to - from) / HOUR_MS))
    for (let hour = 0; hour < hours; hour += 1) {
      const bucket = from + hour * HOUR_MS
      if (precise && (bucket >= preciseTo || bucket + HOUR_MS <= preciseFrom)) continue
      const buckets = models.map(model => {
        const requests = Math.floor(model.requests / hours) + (hour < model.requests % hours ? 1 : 0)
        const share = model.requests ? requests / model.requests : 0
        return { ...model, requests, tokens: Math.round(model.tokens * share), cost: Number((model.cost * share).toFixed(2)) }
      })
      rows.push(dailyRow(new Date(bucket).toISOString(), buckets))
    }
  }
  const modelSummary = modelNames.map((model, index) => {
    const entries = rows.map(row => row.model_breakdown[index])
    const requests = entries.reduce((sum, entry) => sum + entry.requests, 0)
    const tokens = entries.reduce((sum, entry) => sum + entry.tokens, 0)
    const cost = Number(entries.reduce((sum, entry) => sum + entry.cost, 0).toFixed(2))
    return { model, requests, tokens, cost, avg_response_time: requests ? 1.52 : 0, cost_per_request: requests ? cost / requests : 0, tokens_per_request: requests ? tokens / requests : 0 }
  })
  return {
    daily_stats: rows, model_summary: modelSummary,
    provider_summary: modelSummary.map((model, index) => ({ provider: providerNames[index], requests: model.requests, tokens: model.tokens, cost: model.cost })),
    period: { start_date: startDate, end_date: endDate, days },
  }
}
