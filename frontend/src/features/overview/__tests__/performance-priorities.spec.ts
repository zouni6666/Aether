import { createApp, h, nextTick, ref, type App } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import PerformanceView from '../operations/PerformanceView.vue'
import { setI18nLocale } from '@/i18n'
import type { OverviewExecutionActivity } from '@/api/overview'

const api = vi.hoisted(() => ({ performance: vi.fn() }))
vi.mock('@/api/overview', () => ({ overviewApi: api }))
vi.mock('@/components/charts/BarChart.vue', () => ({ default: { render: () => null } }))
vi.mock('@/components/charts/LineChart.vue', () => ({ default: { render: () => null } }))
vi.mock('../components/OverviewTrend.vue', () => ({ default: { render: () => null } }))
vi.mock('../components/OverviewStatus.vue', () => ({ default: { render: () => null } }))

let app: App | undefined
beforeEach(() => { vi.clearAllMocks(); setI18nLocale('zh-CN') })
afterEach(() => { app?.unmount(); app = undefined })

async function settle() {
  await new Promise(resolve => setTimeout(resolve, 0))
  await nextTick()
}
async function mount(url = '/admin/operations?timezone=UTC', initialActivity?: OverviewExecutionActivity | null) {
  const router = createRouter({ history: createMemoryHistory(), routes: [
    { path: '/admin/operations', component: PerformanceView },
    { path: '/admin/usage', component: { render: () => null } },
  ] })
  await router.push(url)
  const root = document.createElement('div')
  const activity = ref(initialActivity)
  app = createApp({ render: () => h(PerformanceView, { revision: 0, activity: activity.value }) }).use(router)
  app.mount(root)
  await settle()
  return { root, router, activity }
}
function performance(rate: number | null, errors: number | null = 3, requests = 100) {
  return {
    request_count: requests, success_count: errors == null ? null : requests - errors,
    error_count: errors, success_rate: rate,
    avg_first_byte_time_ms: null as number | null, avg_output_tps: null as number | null,
    avg_response_time_ms: null, slow_request_count: 3,
  }
}
function provider(id: string, rate: number | null, errors: number | null = 3, requests = 100) {
  return { ...performance(rate, errors, requests), provider_id: id, provider: id }
}
function model(name: string | null, rate: number | null, errors: number | null = 3, requests = 100) {
  return { ...performance(rate, errors, requests), model: name }
}
function response(providers = [provider('example', 97)], models = [model('model-a', 95)]) {
  return { data: {
    summary: { request_count: 300, latency_ms: { p95: null } },
    timeseries: [], errors: [], models,
    providers: {
      summary: { avg_first_byte_time_ms: 0, avg_output_tps: null, slow_request_count: 3 },
      providers, timeline: [],
    },
  } }
}
function analysis(root: HTMLElement, dimension: 'provider' | 'model') {
  return root.querySelector<HTMLElement>(`[data-performance-analysis="${dimension}"]`)!
}
function rows(root: HTMLElement, dimension: 'provider' | 'model') {
  return Array.from(analysis(root, dimension).querySelectorAll<HTMLTableRowElement>('[data-analysis-row]'))
}
function liveActivity(): OverviewExecutionActivity {
  return {
    observed_at: '2026-09-20T08:00:00Z', observed_from: '2026-09-20T07:59:00Z',
    window_seconds: 60, observed_window_seconds: 60, scope: { kind: 'node' }, coverage: 'complete',
    providers: [{ provider_id: 'example', provider: 'Example', requests_per_minute: 83, current_concurrency: 7 }],
    models: [{ model: 'model-a', requests_per_minute: 61, current_concurrency: 4 }],
  }
}
function loadValues(root: HTMLElement, dimension: 'provider' | 'model') {
  return rows(root, dimension).map(row => ({
    entity: row.dataset.analysisRow,
    concurrency: row.querySelector('[data-analysis-concurrency]')?.textContent?.trim(),
    rpm: row.querySelector('[data-analysis-rpm]')?.textContent?.trim(),
  }))
}
function linkQuery(link: HTMLAnchorElement) {
  const url = new URL(link.getAttribute('href')!, 'http://localhost')
  expect(url.pathname).toBe('/admin/usage')
  return Object.fromEntries(url.searchParams)
}

const fixedRange = {
  from: '2026-09-18T00:00:00.000Z', to: '2026-09-19T00:00:00.000Z', timezone: 'UTC',
}
const legacyUrl = '/admin/operations?from=2026-09-18T00:00:00Z&to=2026-09-19T00:00:00Z&timezone=UTC&model=old-model&api_format=openai&is_stream=false&has_format_conversion=true&provider_id=old-provider&slow_threshold_ms=7500&status=failed'

describe('operations automatic analysis', () => {
  it('shows provider and model reports together without filters and uses the backend model aggregates', async () => {
    api.performance.mockResolvedValue(response(
      [provider('provider-a', 97, 3, 100), provider('provider-b', 95, 10, 200)],
      [model('shared-model', 96.25, 6, 160)],
    ))
    const { root } = await mount()
    expect(root.querySelectorAll('[data-performance-analysis]').length).toBe(2)
    expect(root.querySelector('form, input, select, [role="tab"], [role="tablist"]')).toBeNull()
    expect(rows(root, 'provider')).toHaveLength(2)
    const modelRows = rows(root, 'model')
    expect(modelRows).toHaveLength(1)
    expect(modelRows[0].querySelector('[data-analysis-entity-link]')?.textContent).toContain('shared-model')
    expect(modelRows[0].querySelectorAll('td')[1].textContent).toContain('160')
    expect(modelRows[0].querySelectorAll('td')[2].textContent).toContain('96.25%')
    expect(root.querySelector<HTMLDetailsElement>('[data-operations-diagnostics]')?.open).toBe(false)
    expect(api.performance).toHaveBeenCalledTimes(1)
  })

  it('orders measured failures first, uses percentage points and distinguishes unavailable values from zero', async () => {
    const measured = provider('measured', 97.25, 11)
    measured.avg_first_byte_time_ms = 0
    measured.avg_output_tps = 0
    api.performance.mockResolvedValue(response([
      provider('unavailable', null, null),
      provider('no-failures', 100, 0, 1_000),
      measured,
      provider('zero-success', 0, 25),
    ], [model('model-zero', 0, 100), model('model-unknown', null, null)]))
    const { root } = await mount()
    const providerRows = rows(root, 'provider')
    expect(providerRows.map(row => row.dataset.analysisRow)).toEqual(['zero-success', 'measured', 'no-failures', 'unavailable'])
    expect(providerRows[0].querySelectorAll('td')[2].textContent).toContain('0%')
    expect(providerRows[1].querySelectorAll('td')[2].textContent).toContain('97.25%')
    expect(providerRows[3].querySelectorAll('td')[2].textContent?.trim()).toMatch(/^-/)
    expect(providerRows[1].querySelectorAll('td')[5].textContent?.trim()).toBe('0 ms')
    expect(providerRows[1].querySelectorAll('td')[6].textContent?.trim()).toBe('0')
    expect(providerRows[3].querySelectorAll('td')[5].textContent?.trim()).toBe('-')
    expect(providerRows[3].querySelectorAll('td')[6].textContent?.trim()).toBe('-')
    const modelRows = rows(root, 'model')
    expect(modelRows[0].querySelectorAll('td')[2].textContent).toContain('0%')
    expect(modelRows[1].querySelectorAll('td')[2].textContent?.trim()).toMatch(/^-/)
  })

  it('shows current provider and model load independently of the selected historical range', async () => {
    api.performance.mockResolvedValue(response())
    const { root, router } = await mount(legacyUrl, liveActivity())
    expect(loadValues(root, 'provider')).toEqual([{ entity: 'example', concurrency: '7', rpm: '83' }])
    expect(loadValues(root, 'model')).toEqual([{ entity: 'model-a', concurrency: '4', rpm: '61' }])
    expect(analysis(root, 'provider').querySelector('thead')?.textContent).toContain('当前并发')
    expect(analysis(root, 'provider').querySelector('thead')?.textContent).toContain('近 60 秒')

    api.performance.mockResolvedValue(response([provider('example', 99, 10, 1_000)], [model('model-a', 99, 10, 1_000)]))
    await router.replace({ query: { from: '2026-09-01T00:00:00Z', to: '2026-09-19T00:00:00Z', timezone: 'UTC' } })
    await settle()
    expect(api.performance).toHaveBeenCalledTimes(2)
    expect(rows(root, 'provider')[0].querySelectorAll('td')[1].textContent).toContain('1,000')
    expect(loadValues(root, 'provider')).toEqual([{ entity: 'example', concurrency: '7', rpm: '83' }])
    expect(loadValues(root, 'model')).toEqual([{ entity: 'model-a', concurrency: '4', rpm: '61' }])
  })

  it('distinguishes measured zero load from missing or incomplete live coverage', async () => {
    api.performance.mockResolvedValue(response())
    const { root, activity } = await mount()
    for (const dimension of ['provider', 'model'] as const) {
      expect(loadValues(root, dimension)[0]).toMatchObject({ concurrency: '-', rpm: '-' })
    }
    activity.value = { ...liveActivity(), providers: [], models: [] }
    await settle()
    for (const dimension of ['provider', 'model'] as const) {
      expect(loadValues(root, dimension)[0]).toMatchObject({ concurrency: '0', rpm: '0' })
    }
    activity.value = { ...liveActivity(), coverage: 'partial' }
    await settle()
    for (const dimension of ['provider', 'model'] as const) {
      expect(loadValues(root, dimension)[0]).toMatchObject({ concurrency: '-', rpm: '-' })
    }
    activity.value = null
    await settle()
    expect(loadValues(root, 'provider')[0]).toMatchObject({ concurrency: '-', rpm: '-' })
    expect(api.performance).toHaveBeenCalledTimes(1)
  })

  it('keeps new active providers and models visible before historical usage has been recorded', async () => {
    api.performance.mockResolvedValue(response([], []))
    const { root } = await mount(legacyUrl, liveActivity())
    expect(loadValues(root, 'provider')).toEqual([{ entity: 'example', concurrency: '7', rpm: '83' }])
    expect(loadValues(root, 'model')).toEqual([{ entity: 'model-a', concurrency: '4', rpm: '61' }])
    for (const dimension of ['provider', 'model'] as const) {
      const row = rows(root, dimension)[0]
      expect(row.querySelectorAll('td')[1].textContent?.trim()).toMatch(/^0/)
      expect(row.querySelector('[data-analysis-success-rate]')?.textContent).toBe('-')
      expect(row.querySelectorAll('td')[5].textContent?.trim()).toBe('-')
      expect(row.querySelectorAll('td')[6].textContent?.trim()).toBe('-')
    }
  })

  it('ignores legacy filters in requests and limits drilldowns to the selected range and analysis entity', async () => {
    api.performance.mockResolvedValue(response())
    const { root, router } = await mount(legacyUrl)
    expect(api.performance.mock.lastCall?.[0]).toMatchObject(fixedRange)
    for (const key of ['model', 'api_format', 'is_stream', 'has_format_conversion', 'provider_id', 'slow_threshold_ms', 'status']) {
      expect(api.performance.mock.lastCall?.[0][key]).toBeUndefined()
    }
    for (const [dimension, entity, key] of [['provider', 'example', 'provider_id'], ['model', 'model-a', 'model']] as const) {
      const card = analysis(root, dimension)
      const entityLink = card.querySelector<HTMLAnchorElement>(`[data-analysis-entity-link="${entity}"]`)!
      const failureLink = card.querySelector<HTMLAnchorElement>(`[data-analysis-failure-link="${entity}"]`)!
      expect(linkQuery(entityLink)).toEqual({ ...fixedRange, [key]: entity })
      expect(linkQuery(failureLink)).toEqual({ ...fixedRange, [key]: entity, status: 'failed' })
    }
    await router.replace({ query: { ...router.currentRoute.value.query, model: 'another-old-model', provider_id: 'another-old-provider', slow_threshold_ms: '15000' } })
    await settle()
    expect(api.performance).toHaveBeenCalledTimes(1)
    expect(rows(root, 'model')).toHaveLength(1)
  })

  it.each([null, ''])('does not turn an unattributed model (%s) into an unrestricted request link', async missingModel => {
    api.performance.mockResolvedValue(response([], [model(missingModel, 100, 0)]))
    const { root } = await mount(legacyUrl)
    expect(rows(root, 'provider')).toHaveLength(0)
    const modelRows = rows(root, 'model')
    expect(modelRows).toHaveLength(1)
    expect(modelRows[0].querySelector('a')).toBeNull()
    expect(modelRows[0].querySelectorAll('td')[1].textContent).toContain('100')
  })
})
