import type { OverviewBreakdown, OverviewDashboardCharts, OverviewRange } from '@/api/overview'
import { getI18nLocale } from '@/i18n'
import { zonedInput, zonedInstant } from '../query'
import { amountValue } from './amount'

export const chartColors = ['#3b82f6', '#10b981', '#f59e0b', '#8b5cf6', '#ef4444', '#06b6d4', '#94a3b8']
type ModelRow = OverviewDashboardCharts['models'][number]

export function chartDate(value: string, timezone: string): string {
  return new Intl.DateTimeFormat(getI18nLocale(), { timeZone: timezone, month: 'short', day: 'numeric' }).format(new Date(value))
}

function knownSum(rows: { billable_amount: OverviewBreakdown['billable_amount'] }[]): number | null {
  if (!rows.length) return 0
  const values = rows.map(row => amountValue(row.billable_amount)).filter((value): value is number => value !== null)
  return values.length ? values.reduce((sum, value) => sum + value, 0) : null
}

export function modelDatasets(data: OverviewDashboardCharts, unknownLabel: string, otherLabel: string) {
  const groups = new Map<string | null, ModelRow[]>()
  for (const row of data.models) {
    const rows = groups.get(row.id)
    if (rows) rows.push(row)
    else groups.set(row.id, [row])
  }
  const sorted = [...groups].sort((a, b) => (knownSum(b[1]) ?? -1) - (knownSum(a[1]) ?? -1))
  const visible = sorted.slice(0, 6).map(([id, rows]) => ({ id, label: rows[0]?.label || unknownLabel, rows }))
  if (sorted.length > 6) visible.push({ id: '__other__', label: otherLabel, rows: sorted.slice(6).flatMap(([, rows]) => rows) })
  return visible.map((group, index) => ({
    label: group.label,
    data: data.series.map(day => knownSum(group.rows.filter(row => Date.parse(row.bucket_start) === Date.parse(day.bucket_start)))),
    backgroundColor: chartColors[index % chartColors.length],
    borderRadius: 2, stack: 'models', barPercentage: 0.6, categoryPercentage: 0.7,
  }))
}

export function providerSlices(rows: OverviewBreakdown[], unknownLabel: string, otherLabel: string) {
  const known = rows.map(row => ({ label: row.label || unknownLabel, value: amountValue(row.billable_amount) }))
    .filter((row): row is { label: string; value: number } => row.value !== null && row.value > 0)
    .sort((a, b) => b.value - a.value)
  const visible = known.slice(0, 6)
  if (known.length > 6) visible.push({ label: otherLabel, value: known.slice(6).reduce((sum, row) => sum + row.value, 0) })
  return visible
}

export function dayRange(bucket: string, range: OverviewRange): OverviewRange {
  const date = zonedInput(bucket, range.timezone).slice(0, 10)
  const nextDate = new Date(Date.parse(`${date}T00:00:00Z`) + 86_400_000).toISOString().slice(0, 10)
  const from = localDayStart(date, range.timezone)
  const to = localDayStart(nextDate, range.timezone)
  return {
    from: new Date(Math.max(Date.parse(from), Date.parse(range.from))).toISOString(),
    to: new Date(Math.min(Date.parse(to), Date.parse(range.to))).toISOString(),
    timezone: range.timezone,
  }
}

function localDayStart(date: string, timezone: string): string {
  const midnight = Date.parse(`${date}T00:00:00Z`)
  // Some timezone transitions skip midnight; start at the first existing local minute.
  for (let minute = 0; minute <= 1440; minute++) {
    const wall = new Date(midnight + minute * 60_000).toISOString().slice(0, 16)
    const instant = zonedInstant(wall, timezone)
    if (instant) return instant
  }
  throw new Error('Unable to resolve local day boundary')
}
