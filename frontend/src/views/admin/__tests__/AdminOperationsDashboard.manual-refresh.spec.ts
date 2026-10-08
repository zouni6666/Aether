import { createApp, defineComponent, h, nextTick, onMounted, onUnmounted } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, describe, expect, it, vi } from 'vitest'
import AdminOperationsDashboard from '../AdminOperationsDashboard.vue'

vi.mock('@/features/overview/components/OverviewToolbar.vue', () => ({ default: { render: () => null } }))
const lifecycle = vi.hoisted(() => ({ mounted: [] as string[], unmounted: [] as string[] }))
function view(name: string) {
  return defineComponent({
    setup(_, { slots }) {
      onMounted(() => lifecycle.mounted.push(name))
      onUnmounted(() => lifecycle.unmounted.push(name))
      return () => h('div', { 'data-view': name }, [name, slots.default?.({ snapshot: { node_id: 'live-node' } }), slots.details?.(), slots.diagnostics?.()])
    },
  })
}
vi.mock('@/features/overview/operations/RuntimeView.vue', () => ({ default: view('runtime') }))
vi.mock('@/features/overview/operations/PerformanceView.vue', () => ({ default: view('performance') }))
vi.mock('@/features/overview/operations/LiveMetrics.vue', () => ({ default: view('details') }))
vi.mock('@/features/overview/operations/RuntimeFocus.vue', () => ({ default: { render: () => null } }))
async function settle() { await Promise.resolve(); await Promise.resolve(); await nextTick(); await new Promise(resolve => setTimeout(resolve, 0)) }

describe('operations view lifecycle', () => {
  afterEach(() => vi.useRealTimers())

  it('keeps every section mounted regardless of legacy view query values or browser history', async () => {
    lifecycle.mounted.length = 0
    lifecycle.unmounted.length = 0
    const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/admin/operations', component: AdminOperationsDashboard }] })
    await router.push('/admin/operations?view=performance&from=2026-09-10T23:30:00Z&to=2026-09-11T00:30:00Z&timezone=UTC')
    const root = document.createElement('div')
    const app = createApp(AdminOperationsDashboard).use(router)
    app.mount(root)
    expect([...lifecycle.mounted].sort()).toEqual(['details', 'details', 'performance', 'runtime'])
    expect(root.querySelector('[role="tablist"]')).toBeNull()
    expect(root.querySelector('[role="tab"]')).toBeNull()
    await router.push({ query: { ...router.currentRoute.value.query, view: 'resources' } })
    await settle()
    expect(lifecycle.mounted).toHaveLength(4)
    expect(lifecycle.unmounted).toEqual([])
    expect(router.currentRoute.value.query.from).toBe('2026-09-10T23:30:00Z')
    router.back()
    await settle()
    for (const name of ['runtime', 'performance']) expect(root.querySelector(`[data-view="${name}"]`)).not.toBeNull()
    expect(root.querySelectorAll('[data-view="details"]')).toHaveLength(2)
    expect(lifecycle.unmounted).toEqual([])
    app.unmount()
    expect([...lifecycle.unmounted].sort()).toEqual(['details', 'details', 'performance', 'runtime'])
  })
})
