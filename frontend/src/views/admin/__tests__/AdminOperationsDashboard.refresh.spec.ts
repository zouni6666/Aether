import { createApp, nextTick, type App } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import AdminOperationsDashboard from '../AdminOperationsDashboard.vue'
import { setI18nLocale } from '@/i18n'

const api = vi.hoisted(() => ({ summary: vi.fn(), timeseries: vi.fn(), live: vi.fn(), performance: vi.fn(), resources: vi.fn() }))
vi.mock('@/api/overview', () => ({ overviewApi: api }))
vi.mock('@/components/common/TimeRangePicker.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return { default: defineComponent({
    props: {
      modelValue: { type: Object, required: true },
      presetOnly: Boolean,
      showGranularity: Boolean,
      presetOptions: { type: Array, default: () => [] },
    },
    emits: ['update:modelValue'],
    setup: (props, { emit }) => () => h('select', {
      'aria-label': '时间范围',
      'data-preset-only': String(props.presetOnly),
      'data-show-granularity': String(props.showGranularity),
      value: props.modelValue.preset || 'custom',
      onChange: (event: Event) => emit('update:modelValue', { preset: (event.target as HTMLSelectElement).value, granularity: 'day' }),
    }, [
      h('option', { value: 'custom', disabled: true }, '已选时段'),
      ...props.presetOptions.map(preset => h('option', { value: String(preset) }, String(preset))),
    ]),
  }) }
})
vi.mock('@/features/overview/components/OverviewMetrics.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return { default: defineComponent({ props: { metrics: { type: Object, required: true } }, setup: props => () => h('p', { 'data-request-count': '' }, String(props.metrics.request_count)) }) }
})
vi.mock('@/features/overview/components/OverviewTrend.vue', () => ({ default: { render: () => null } }))
vi.mock('@/features/overview/operations/LiveMetrics.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return { default: defineComponent({ props: { snapshot: { type: Object, required: true }, resources: Boolean }, setup: props => () => h('p', { 'data-live-node': props.resources ? 'resources' : 'runtime' }, props.snapshot.node_id) }) }
})
vi.mock('@/features/overview/components/OverviewStatus.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return { default: defineComponent({ props: { error: String }, setup: props => () => props.error ? h('p', { role: 'alert' }, props.error) : null }) }
})

const mounted: App[] = []
async function settle() { await vi.advanceTimersByTimeAsync(0); await nextTick() }
async function mount(url = '/admin/operations?timezone=UTC') {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/admin/operations', component: AdminOperationsDashboard }, { path: '/admin/usage', component: { render: () => null } }] })
  await router.push(url)
  const root = document.createElement('div')
  const app = createApp(AdminOperationsDashboard).use(router)
  app.mount(root)
  mounted.push(app)
  await settle()
  return { root, router, app }
}
function change(root: HTMLElement, selector: string, value: string) {
  const input = root.querySelector<HTMLInputElement | HTMLSelectElement>(selector)!
  input.value = value
  input.dispatchEvent(new Event(input.tagName === 'SELECT' ? 'change' : 'input', { bubbles: true }))
}
function liveNodes(root: HTMLElement) { return Array.from(root.querySelectorAll('[data-live-node]'), node => node.textContent) }
function enableAutoRefresh(root: HTMLElement) {
  const button = root.querySelector<HTMLButtonElement>('button[aria-label="开启自动刷新（每 10 秒）"]')!
  expect(button.getAttribute('aria-pressed')).toBe('false')
  button.click()
}
function disableAutoRefresh(root: HTMLElement) {
  const button = root.querySelector<HTMLButtonElement>('button[aria-label="关闭自动刷新（每 10 秒）"]')!
  expect(button.getAttribute('aria-pressed')).toBe('true')
  button.click()
}

beforeEach(() => {
  vi.useFakeTimers()
  vi.setSystemTime(new Date('2026-09-18T08:00:00Z'))
  vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('visible')
  setI18nLocale('zh-CN')
  vi.clearAllMocks()
  api.summary.mockResolvedValue({ data: { request_count: 42 } })
  api.timeseries.mockResolvedValue({ data: { items: [] } })
  api.live.mockResolvedValue({ data: { node_id: 'live-node' } })
  api.performance.mockResolvedValue({ data: { summary: { request_count: 42 }, timeseries: [], providers: null, errors: [] } })
  api.resources.mockResolvedValue({ data: { node_id: 'resource-node' } })
})
afterEach(() => { for (const app of mounted.splice(0)) app.unmount(); vi.restoreAllMocks(); vi.useRealTimers() })

describe('operations refresh behavior', () => {
  it('passes grouped live load into both analysis tables and updates it using the existing two refresh queries', async () => {
    const executionActivity = {
      observed_at: '2026-09-18T08:00:00Z', observed_from: '2026-09-18T07:59:00Z',
      window_seconds: 60, observed_window_seconds: 60, scope: { kind: 'node' }, coverage: 'complete',
      providers: [{ provider_id: 'provider-a', provider: 'Provider A', requests_per_minute: 12, current_concurrency: 2 }],
      models: [{ model: 'model-a', requests_per_minute: 10, current_concurrency: 1 }],
    }
    api.live.mockResolvedValue({ data: { node_id: 'live-node', execution_activity: executionActivity } })
    api.performance.mockResolvedValue({ data: {
      summary: { request_count: 42 }, timeseries: [], providers: { providers: [] }, models: [], errors: [],
    } })
    const { root } = await mount()
    const values = (dimension: string) => {
      const row = root.querySelector(`[data-performance-analysis="${dimension}"] [data-analysis-row]`)!
      return [row.querySelector('[data-analysis-concurrency]')?.textContent?.trim(), row.querySelector('[data-analysis-rpm]')?.textContent?.trim()]
    }
    expect(values('provider')).toEqual(['2', '12'])
    expect(values('model')).toEqual(['1', '10'])

    api.live.mockResolvedValue({ data: { node_id: 'live-node', execution_activity: {
      ...executionActivity,
      providers: [{ ...executionActivity.providers[0], requests_per_minute: 21, current_concurrency: 4 }],
      models: [{ ...executionActivity.models[0], requests_per_minute: 18, current_concurrency: 3 }],
    } } })
    enableAutoRefresh(root)
    await settle()
    expect(values('provider')).toEqual(['4', '21'])
    expect(values('model')).toEqual(['3', '18'])
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(2)
    for (const method of [api.resources, api.summary, api.timeseries]) expect(method).not.toHaveBeenCalled()
  })

  it('starts disabled, refreshes immediately when enabled and polls every ten seconds from that click without duplicate queries', async () => {
    const { root } = await mount('/admin/operations?view=resources&timezone=UTC')
    expect(root.querySelector('[role="tablist"]')).toBeNull()
    expect(root.querySelector('[role="tab"]')).toBeNull()
    expect(root.querySelector('input[type="checkbox"]')).toBeNull()
    expect(root.querySelector('[data-request-count]')?.textContent).toBe('42')
    expect(root.querySelector('[data-live-node="runtime"]')?.textContent).toBe('live-node')
    expect(root.querySelector('[data-live-node="resources"]')?.textContent).toBe('live-node')
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(1)

    await vi.advanceTimersByTimeAsync(12_345)
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(1)

    enableAutoRefresh(root)
    await settle()
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(2)
    expect(root.querySelector('button[aria-label="关闭自动刷新（每 10 秒）"]')?.getAttribute('aria-pressed')).toBe('true')

    await vi.advanceTimersByTimeAsync(9_999)
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(2)
    await vi.advanceTimersByTimeAsync(1)
    await settle()
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(3)
    expect(api.resources).not.toHaveBeenCalled()
    expect(api.summary).not.toHaveBeenCalled()
    expect(api.timeseries).not.toHaveBeenCalled()
  })

  it('stops polling without a request when disabled and does not accumulate timers when toggled repeatedly', async () => {
    const interval = vi.spyOn(globalThis, 'setInterval')
    const clear = vi.spyOn(globalThis, 'clearInterval')
    const { root } = await mount()
    enableAutoRefresh(root)
    await settle()
    const firstTimer = interval.mock.results[interval.mock.results.length - 1]?.value
    disableAutoRefresh(root)
    await settle()
    expect(clear).toHaveBeenCalledWith(firstTimer)
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(2)
    await vi.advanceTimersByTimeAsync(20_000)
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(2)

    for (let index = 0; index < 3; index += 1) {
      enableAutoRefresh(root)
      await settle()
      await vi.advanceTimersByTimeAsync(1_000)
      disableAutoRefresh(root)
      await settle()
    }
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(5)
    enableAutoRefresh(root)
    await settle()
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(6)
    await vi.advanceTimersByTimeAsync(9_999)
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(6)
    await vi.advanceTimersByTimeAsync(1)
    await settle()
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(7)
  })

  it('clears the active interval when leaving the page', async () => {
    const interval = vi.spyOn(globalThis, 'setInterval')
    const clear = vi.spyOn(globalThis, 'clearInterval')
    const { root, app } = await mount()
    enableAutoRefresh(root)
    await settle()
    const activeTimer = interval.mock.results[interval.mock.results.length - 1]?.value
    app.unmount()
    mounted.splice(mounted.indexOf(app), 1)
    expect(clear).toHaveBeenCalledWith(activeTimer)
    await vi.advanceTimersByTimeAsync(30_000)
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(2)
  })

  it('skips polling while the page is hidden and resumes at the next visible interval', async () => {
    const { root } = await mount()
    enableAutoRefresh(root)
    await settle()
    vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('hidden')
    await vi.advanceTimersByTimeAsync(20_000)
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(2)
    vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('visible')
    await vi.advanceTimersByTimeAsync(10_000)
    await settle()
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(3)
  })

  it('loads performance even when the initial live snapshot fails', async () => {
    api.live.mockRejectedValue(new Error('live unavailable'))
    const { root } = await mount()
    expect(root.textContent).toContain('live unavailable')
    expect(root.querySelector('[data-request-count]')?.textContent).toBe('42')
    expect(liveNodes(root)).toEqual([])
    for (const method of [api.live, api.performance]) expect(method).toHaveBeenCalledTimes(1)
    expect(api.resources).not.toHaveBeenCalled()
  })

  it('advances relative ranges when refreshing and changing the time preset while ignoring legacy analysis filters', async () => {
    const { root, router } = await mount()
    expect(api.performance.mock.lastCall?.[0]).toMatchObject({ from: '2026-09-18T00:00:00.000Z', to: '2026-09-18T08:00:00.000Z' })
    vi.setSystemTime(new Date('2026-09-18T08:30:00Z'))
    enableAutoRefresh(root)
    await settle()
    expect(api.performance.mock.lastCall?.[0].to).toBe('2026-09-18T08:30:00.000Z')
    await vi.advanceTimersByTimeAsync(10_000)
    await settle()
    expect(api.performance.mock.lastCall?.[0].to).toBe('2026-09-18T08:30:10.000Z')

    const callsBeforeLegacyFilters = api.performance.mock.calls.length
    await router.replace({ query: {
      ...router.currentRoute.value.query,
      provider_id: 'legacy-provider', model: 'legacy-model', is_stream: 'false',
      api_format: 'openai', slow_threshold_ms: '7500',
    } })
    await settle()
    expect(api.performance).toHaveBeenCalledTimes(callsBeforeLegacyFilters)
    expect(root.querySelector('form, input')).toBeNull()
    vi.setSystemTime(new Date('2026-09-18T08:59:50Z'))
    await vi.advanceTimersByTimeAsync(10_000)
    await settle()
    expect(api.performance.mock.lastCall?.[0].to).toBe('2026-09-18T09:00:00.000Z')
    for (const key of ['provider_id', 'model', 'is_stream', 'api_format', 'slow_threshold_ms']) {
      expect(api.performance.mock.lastCall?.[0][key]).toBeUndefined()
    }

    change(root, 'select[aria-label="时间范围"]', 'last1hour')
    await settle()
    vi.setSystemTime(new Date('2026-09-18T09:29:50Z'))
    await vi.advanceTimersByTimeAsync(10_000)
    await settle()
    expect(api.performance.mock.lastCall?.[0]).toMatchObject({ from: '2026-09-18T08:30:00.000Z', to: '2026-09-18T09:30:00.000Z' })
    expect(router.currentRoute.value.query.relative_preset).toBe('last1hour')
  })

  it('keeps explicit historical ranges fixed when enabling refresh and polling', async () => {
    const { root } = await mount('/admin/operations?from=2026-09-10T23:30:00Z&to=2026-09-11T00:30:00Z&timezone=UTC')
    vi.setSystemTime(new Date('2026-09-18T09:00:00Z'))
    enableAutoRefresh(root)
    await settle()
    await vi.advanceTimersByTimeAsync(10_000)
    await settle()
    expect(api.performance.mock.calls.length).toBeGreaterThan(1)
    for (const [query] of api.performance.mock.calls) expect(query).toMatchObject({ from: '2026-09-10T23:30:00.000Z', to: '2026-09-11T00:30:00.000Z' })
  })

  it('uses the shared preset picker without exact time, custom date or timezone controls', async () => {
    const { root } = await mount()
    const picker = root.querySelector<HTMLSelectElement>('select[aria-label="时间范围"]')!
    expect(picker.dataset.presetOnly).toBe('true')
    expect(picker.dataset.showGranularity).toBe('false')
    expect(Array.from(picker.options).filter(option => !option.disabled).map(option => option.value)).toEqual(['last1hour', 'today', 'last24hours', 'last7days', 'last30days'])
    expect(root.querySelector('button[aria-label="精确时间"]')).toBeNull()
    expect(root.querySelector('input[type="datetime-local"], input[type="date"]')).toBeNull()
    expect(root.querySelector('input[list="overview-timezones"]')).toBeNull()
  })

  it('restores rolling refresh when choosing a preset from an explicit historical range', async () => {
    const { root, router } = await mount('/admin/operations?from=2026-09-10T23:30:00Z&to=2026-09-11T00:30:00Z&timezone=UTC')
    expect(root.querySelector<HTMLSelectElement>('select[aria-label="时间范围"]')!.value).toBe('custom')
    expect(router.currentRoute.value.query.relative_preset).toBeUndefined()
    change(root, 'select[aria-label="时间范围"]', 'last24hours')
    await settle()
    expect(router.currentRoute.value.query.relative_preset).toBe('last24hours')
    expect(api.performance.mock.lastCall?.[0]).toMatchObject({ from: '2026-09-17T08:00:00.000Z', to: '2026-09-18T08:00:00.000Z' })
    vi.setSystemTime(new Date('2026-09-18T09:00:00Z'))
    enableAutoRefresh(root)
    await settle()
    expect(api.performance.mock.lastCall?.[0]).toMatchObject({ from: '2026-09-17T09:00:00.000Z', to: '2026-09-18T09:00:00.000Z' })
    await vi.advanceTimersByTimeAsync(10_000)
    await settle()
    expect(api.performance.mock.lastCall?.[0]).toMatchObject({ from: '2026-09-17T09:00:10.000Z', to: '2026-09-18T09:00:10.000Z' })
  })

  it('retains data on a failed refresh and ignores legacy filters but clears performance results for a different time preset', async () => {
    const { root, router } = await mount()
    expect(root.querySelector('[data-request-count]')?.textContent).toBe('42')
    expect(liveNodes(root)).toEqual(['live-node', 'live-node'])
    api.live.mockRejectedValue(new Error('live unavailable'))
    api.performance.mockRejectedValue(new Error('performance unavailable'))
    vi.setSystemTime(new Date('2026-09-18T08:30:00Z'))
    enableAutoRefresh(root)
    await settle()
    expect(root.querySelector('[data-request-count]')?.textContent).toBe('42')
    expect(liveNodes(root)).toEqual(['live-node', 'live-node'])
    for (const message of ['live unavailable', 'performance unavailable']) expect(root.textContent).toContain(message)
    await router.replace({ query: { ...router.currentRoute.value.query, model: 'legacy-model' } })
    await settle()
    expect(root.querySelector('[data-request-count]')?.textContent).toBe('42')
    expect(api.performance).toHaveBeenCalledTimes(2)
    change(root, 'select[aria-label="时间范围"]', 'last1hour')
    await settle()
    expect(root.querySelector('[data-request-count]')).toBeNull()
    expect(api.performance).toHaveBeenCalledTimes(3)
    expect(liveNodes(root)).toEqual(['live-node', 'live-node'])
    expect(root.textContent).toContain('performance unavailable')
    expect(api.live).toHaveBeenCalledTimes(2)
    expect(api.resources).not.toHaveBeenCalled()
  })
})
