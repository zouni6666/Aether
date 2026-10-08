import { computed, inject, provide, ref, type ComputedRef, type InjectionKey } from 'vue'
import { useRoute, useRouter, type LocationQuery, type LocationQueryRaw } from 'vue-router'
import type { OverviewQuery, OverviewRange } from '@/api/overview'

interface OverviewRangeContext {
  rolling: boolean
  range: ComputedRef<OverviewRange>
  relativePreset: ComputedRef<string | null>
  refreshRange: () => void
}
const overviewRangeKey: InjectionKey<OverviewRangeContext> = Symbol('overview-range')
const relativePresets = ['today', 'last1hour', 'last24hours', 'last7days', 'last30days', 'last90days']

export function queryString(query: LocationQuery, key: string): string {
  const value = query[key]
  return (Array.isArray(value) ? value[0] : value) ?? ''
}

export function browserTimezone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC'
}

export function isTimezone(value: string): boolean {
  try { new Intl.DateTimeFormat('en-US', { timeZone: value }).format(); return true } catch { return false }
}

export function zonedInput(instant: string | Date, timezone: string): string {
  const parts = new Intl.DateTimeFormat('en-CA', {
    timeZone: timezone, year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit', hourCycle: 'h23',
  }).formatToParts(new Date(instant))
  const part = (key: string) => parts.find(p => p.type === key)?.value ?? ''
  return `${part('year')}-${part('month')}-${part('day')}T${part('hour')}:${part('minute')}`
}

export function zonedInstant(value: string, timezone: string): string | null {
  if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(value) || !isTimezone(timezone)) return null
  const wall = Date.parse(`${value}:00Z`)
  if (!Number.isFinite(wall)) return null
  // Probe both sides of a DST transition. Gaps are invalid; repeated times choose the earlier instant.
  const offsets = new Set<number>()
  for (const hours of [-36, -12, 0, 12, 36]) {
    const probe = wall + hours * 3_600_000
    const formatted = Date.parse(`${zonedInput(new Date(probe), timezone)}:00Z`)
    offsets.add(formatted - probe)
  }
  const candidates = [...offsets].map(offset => wall - offset)
    .filter(candidate => zonedInput(new Date(candidate), timezone) === value).sort((a, b) => a - b)
  const candidate = candidates[0]
  return candidate === undefined ? null : new Date(candidate).toISOString()
}

export function presetRange(preset: string, timezone = browserTimezone(), now = new Date()): OverviewRange {
  now = new Date(Math.floor(now.getTime() / 1000) * 1000)
  let start = now.getTime() - 7 * 86_400_000
  if (preset === 'last1hour') start = now.getTime() - 3_600_000
  if (preset === 'last24hours') start = now.getTime() - 86_400_000
  if (preset === 'last30days') start = now.getTime() - 30 * 86_400_000
  if (preset === 'last90days') start = now.getTime() - 90 * 86_400_000
  if (preset === 'today') {
    const midnight = zonedInstant(`${zonedInput(now, timezone).slice(0, 10)}T00:00`, timezone)
    if (midnight) start = Date.parse(midnight)
  }
  return { from: new Date(start).toISOString(), to: now.toISOString(), timezone }
}

export function rangeFromQuery(query: LocationQuery, fallback: OverviewRange): OverviewRange {
  const from = queryString(query, 'from')
  const to = queryString(query, 'to')
  const timezone = queryString(query, 'timezone') || fallback.timezone
  if (from && to && Number.isFinite(Date.parse(from)) && Date.parse(from) < Date.parse(to) && isTimezone(timezone)) {
    return { from: new Date(from).toISOString(), to: new Date(to).toISOString(), timezone }
  }
  if (!from && !to && isTimezone(timezone)) {
    const startDate = queryString(query, 'start_date')
    const endDate = queryString(query, 'end_date')
    const shiftDate = (date: string, days: number) => new Date(Date.parse(`${date}T00:00:00Z`) + days * 86_400_000).toISOString().slice(0, 10)
    if (/^\d{4}-\d{2}-\d{2}$/.test(startDate) && /^\d{4}-\d{2}-\d{2}$/.test(endDate) && Number.isFinite(Date.parse(`${endDate}T00:00:00Z`))) {
      const start = zonedInstant(`${startDate}T00:00`, timezone)
      const end = zonedInstant(`${shiftDate(endDate, 1)}T00:00`, timezone)
      if (start && end && start < end) return { from: start, to: end, timezone }
    }
    const preset = queryString(query, 'preset')
    const legacyDays: Record<string, number> = { today: 1, yesterday: 1, last7days: 7, last30days: 30, last90days: 90 }
    if (legacyDays[preset]) {
      const today = zonedInput(new Date(fallback.to), timezone).slice(0, 10)
      const endDate = preset === 'yesterday' ? today : shiftDate(today, 1)
      const start = zonedInstant(`${shiftDate(endDate, -legacyDays[preset])}T00:00`, timezone)
      const end = zonedInstant(`${endDate}T00:00`, timezone)
      if (start && end) return { from: start, to: end, timezone }
    }
    if (['last1hour', 'last24hours'].includes(preset)) return presetRange(preset, timezone, new Date(fallback.to))
  }
  return fallback
}

export function rangeQuery(range: OverviewRange): LocationQueryRaw {
  return { from: range.from, to: range.to, timezone: range.timezone }
}

export function usageQuery(query: OverviewQuery): OverviewQuery {
  const { search: _search, account_status: _account, usage_status: _usage, sort: _sort, order: _order, offset: _offset, limit: _limit, group_by: _group, ...filters } = query
  return filters
}

export function useOverviewQuery(defaultPreset = 'last7days', options: { rolling?: boolean } = {}) {
  const route = useRoute()
  const router = useRouter()
  const inheritedRange = inject(overviewRangeKey, null)
  const rolling = inheritedRange?.rolling ?? options.rolling ?? false
  const initialRange = presetRange(defaultPreset)
  const refreshedAt = ref(Date.parse(initialRange.to))
  const relativePreset = inheritedRange?.relativePreset ?? computed(() => {
    if (!rolling) return null
    const selected = queryString(route.query, 'relative_preset')
    if (relativePresets.includes(selected)) return selected
    if (['from', 'to', 'start_date', 'end_date'].some(key => queryString(route.query, key))) return null
    const legacyPreset = queryString(route.query, 'preset')
    return legacyPreset ? relativePresets.includes(legacyPreset) ? legacyPreset : null : defaultPreset
  })
  const range = inheritedRange?.range ?? computed(() => {
    if (!relativePreset.value) return rangeFromQuery(route.query, initialRange)
    const timezone = queryString(route.query, 'timezone') || initialRange.timezone
    const selectedAt = Date.parse(queryString(route.query, 'to'))
    return presetRange(relativePreset.value, isTimezone(timezone) ? timezone : initialRange.timezone,
      new Date(Math.max(refreshedAt.value, Number.isFinite(selectedAt) ? selectedAt : 0)))
  })
  const refreshRange = inheritedRange?.refreshRange ?? (() => {
    if (relativePreset.value) refreshedAt.value = Date.now()
  })
  provide(overviewRangeKey, { rolling, range, relativePreset, refreshRange })
  const query = computed<OverviewQuery>(() => {
    const value: OverviewQuery = { ...range.value }
    const textKeys = ['user_id', 'api_key_id', 'request_type', 'credential_owner_id', 'attribution_kind', 'model', 'provider_id', 'api_format', 'endpoint_kind', 'status', 'search', 'account_status', 'usage_status', 'sort', 'group_by', 'amount_basis'] as const
    for (const key of textKeys) {
      const text = queryString(route.query, key)
      if (text) value[key] = text
    }
    for (const key of ['is_stream', 'has_format_conversion'] as const) {
      const text = queryString(route.query, key)
      if (text === 'true' || text === 'false') value[key] = text === 'true'
    }
    for (const key of ['limit', 'offset', 'slow_threshold_ms'] as const) {
      const text = queryString(route.query, key)
      if (text && Number.isFinite(Number(text))) value[key] = Math.max(0, Math.floor(Number(text)))
    }
    value.order = queryString(route.query, 'order') === 'asc' ? 'asc' : 'desc'
    const granularity = queryString(route.query, 'granularity')
    value.granularity = granularity === 'hour' ? 'hour' : granularity === 'day' ? 'day'
      : Date.parse(range.value.to) - Date.parse(range.value.from) <= 48 * 3_600_000 ? 'hour' : 'day'
    return value
  })
  // Advancing a relative window is a refresh of the same selection, not a new filter scope.
  const scopeKey = computed(() => JSON.stringify(relativePreset.value
    ? { ...query.value, from: undefined, to: undefined, relative_preset: relativePreset.value }
    : query.value))
  function patch(values: LocationQueryRaw, push = false) {
    const next = { ...route.query, start_date: undefined, end_date: undefined, preset: undefined, tz_offset_minutes: undefined, ...rangeQuery(range.value), relative_preset: relativePreset.value || undefined, ...values }
    return router[push ? 'push' : 'replace']({ query: next })
  }
  function setRange(value: OverviewRange, preset?: string) {
    return patch({ ...rangeQuery(value), relative_preset: rolling && preset && relativePresets.includes(preset) ? preset : undefined, offset: undefined })
  }
  function link(path: string, filters: LocationQueryRaw = {}) {
    return { path, query: { ...rangeQuery(range.value), ...filters } }
  }
  return { route, router, range, query, relativePreset, scopeKey, refreshRange, patch, setRange, link }
}
