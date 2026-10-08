import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, nextTick, type App } from 'vue'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import HealthMonitorView from '../HealthMonitorView.vue'
import type { HealthMeta, PublicHealthObject } from '@/api/endpoints/health-v2'

const api = vi.hoisted(() => ({
  getAdminHealthSummary: vi.fn(), getAdminHealthObjects: vi.fn(), getAdminHealthObject: vi.fn(),
  getPublicHealthSummaryV2: vi.fn(), getPublicHealthObjects: vi.fn(), getPublicHealthObject: vi.fn(),
  getUserHealthSummary: vi.fn(), getUserHealthObjects: vi.fn(), getUserHealthObject: vi.fn(),
  getHealthPublication: vi.fn(), saveHealthPublication: vi.fn(),
}))
vi.mock('@/api/endpoints/health-v2', () => api)
vi.mock('@/components/ui/dialog/Dialog.vue', () => ({
  default: defineComponent({ props: { open: Boolean, title: String }, setup: (props, { slots }) => () => props.open ? h('section', [h('h2', props.title), slots.default?.()]) : null }),
}))

const ratio = { numerator: 100, denominator: 100, value: 1 }
const meta: HealthMeta = {
  schema_version: 2, metric_version: 'service-health-v1', scope: { kind: 'published', object_kind: 'api_format' },
  range: { from: '2026-09-11T00:00:00Z', to: '2026-09-11T06:00:00Z', timezone: 'UTC', time_basis: 'request_started_at' },
  generated_at: '2026-09-11T06:00:00Z', data_through: null, freshness: 'unknown',
  policy: { version: 'service-health-v1', minimum_samples: 20, healthy_threshold: 0.99, degraded_threshold: 0.95 },
}
function object(name = 'Chat API'): PublicHealthObject {
  return { id: 'chat', kind: 'api_format', name, status: 'healthy', request_count: 100,
    request_success: ratio, service_availability: ratio,
    coverage: { status: 'complete', sample_status: 'sufficient', classified_count: 100, excluded_count: 0, unknown_failure_count: 0, exclusion_policy: 'policy' },
    average_latency_ms: 450, latency_sample_count: 100, last_request_at: null, timeline: [],
  }
}
function summary() { return { meta, data: { status: 'healthy', object_count: 40, healthy_count: 40, degraded_count: 0, unavailable_count: 0, unknown_count: 0, requests: object() } } }
function page(name = 'Chat API') { return { meta, data: { items: [object(name)], total: 40, limit: 25, offset: 0 } } }
const mounted: Array<{ app: App; root: HTMLElement }> = []
async function flush() { for (let i = 0; i < 8; i++) { await Promise.resolve(); await nextTick() } }
async function mountView(admin = false, publicPage = !admin): Promise<{ root: HTMLElement; router: Router }> {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:pathMatch(.*)*', component: defineComponent({ setup: () => () => h('div') }) }] })
  await router.push(admin ? '/admin/health' : '/status')
  await router.isReady()
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp(HealthMonitorView, { isAdmin: admin, publicPage })
  app.use(router).mount(root)
  mounted.push({ app, root })
  await flush()
  return { root, router }
}
function button(root: HTMLElement, text: string) { return Array.from(root.querySelectorAll('button')).find(button => button.textContent?.includes(text))! }

beforeEach(() => {
  vi.clearAllMocks()
  api.getAdminHealthSummary.mockResolvedValue(summary())
  api.getPublicHealthSummaryV2.mockResolvedValue(summary())
  api.getAdminHealthObjects.mockResolvedValue(page())
  api.getPublicHealthObjects.mockResolvedValue(page())
  api.getUserHealthSummary.mockResolvedValue(summary())
  api.getUserHealthObjects.mockResolvedValue(page())
  api.getAdminHealthObject.mockResolvedValue({ meta, data: object() })
  api.getPublicHealthObject.mockResolvedValue({ meta, data: object() })
  api.getHealthPublication.mockResolvedValue({ enabled: false, objects: [] })
})
afterEach(() => { for (const { app, root } of mounted.splice(0)) { app.unmount(); root.remove() } })

describe('independent health monitor', () => {
  it('keeps authenticated health independent of the public publication switch', async () => {
    const { root } = await mountView(false, false)
    expect(api.getUserHealthSummary).toHaveBeenCalledOnce()
    expect(api.getUserHealthObjects).toHaveBeenCalledOnce()
    expect(api.getPublicHealthSummaryV2).not.toHaveBeenCalled()
    expect(api.getAdminHealthSummary).not.toHaveBeenCalled()
    expect(root.textContent).toContain('Chat API')
  })
  it('uses only public queries and renders server totals independently of page rows', async () => {
    const { root, router } = await mountView()
    expect(api.getAdminHealthSummary).not.toHaveBeenCalled()
    expect(api.getAdminHealthObjects).not.toHaveBeenCalled()
    expect(root.textContent).toContain('40 个对象')
    expect(root.textContent).toContain('Chat API')
    expect(root.textContent).not.toContain('状态页发布')
    expect(root.textContent).not.toContain('提供商')
    const navigated = new Promise<void>(resolve => { const stop = router.afterEach(() => { stop(); resolve() }) })
    button(root, '模型').click()
    await navigated
    await flush()
    expect(router.currentRoute.value.query.kind).toBe('model')
    expect(api.getPublicHealthSummaryV2).toHaveBeenLastCalledWith(expect.objectContaining({ kind: 'model' }), expect.any(AbortSignal))
  })

  it('retains previous data when a manual refresh fails', async () => {
    const { root } = await mountView()
    api.getPublicHealthSummaryV2.mockRejectedValueOnce(new Error('network unavailable'))
    root.querySelector<HTMLButtonElement>('[aria-label="刷新健康数据"]')!.click()
    await flush()
    expect(root.textContent).toContain('健康数据暂时不可用')
    expect(root.textContent).toContain('Chat API')
  })

  it('keeps changed filters when an older request resolves late', async () => {
    let resolveOld!: (value: ReturnType<typeof page>) => void
    api.getPublicHealthObjects.mockReturnValueOnce(new Promise(resolve => { resolveOld = resolve }))
    const { root, router } = await mountView()
    await router.push('/status?kind=model')
    await flush()
    resolveOld(page('Obsolete data'))
    await flush()
    expect(root.textContent).toContain('Chat API')
    expect(root.textContent).not.toContain('Obsolete data')
  })

  it('shows publication save failures without dismissing the editor', async () => {
    const { root } = await mountView(true)
    button(root, '状态页发布').click()
    await flush()
    api.saveHealthPublication.mockRejectedValueOnce({ response: { data: { detail: 'Duplicate public ID' } } })
    button(root, '保存').click()
    await flush()
    expect(api.saveHealthPublication).toHaveBeenCalledWith({ enabled: false, objects: [] })
    expect(root.textContent).toContain('Duplicate public ID')
    expect(root.textContent).toContain('状态页发布（/status）')
  })
})
