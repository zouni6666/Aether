import { createApp, h, nextTick, ref, type App, type VNode } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import ConsumptionTable from '../users/ConsumptionTable.vue'

const api = vi.hoisted(() => ({ consumption: vi.fn() }))
vi.mock('@/api/overview', () => ({ overviewApi: api }))

const amount = { value: '10', currency: 'USD', basis: 'billable', status: 'known' }
const range = { from: '2026-09-10T00:00:00.000Z', to: '2026-09-11T00:00:00.000Z', timezone: 'UTC' }
const meta = {
  range, generated_at: '2026-09-11T00:00:00Z', data_through: null,
  coverage: { status: 'complete', request_count: 10, usage_available_count: 10, pricing_available_count: 10, settled_count: 10 },
}
const mounted: App[] = []
async function settle() { await Promise.resolve(); await nextTick(); await new Promise(resolve => setTimeout(resolve, 0)) }
async function mount(render: () => VNode) {
  const router = createRouter({ history: createMemoryHistory(), routes: [
    { path: '/admin/user-stats/:userId', component: { render: () => null } },
    { path: '/admin/usage', component: { render: () => null } },
    { path: '/admin/health-monitor', component: { render: () => null } },
  ] })
  await router.push({ path: '/admin/user-stats/member-a', query: range })
  const root = document.createElement('div')
  const app = createApp({ render }).use(router)
  app.mount(root)
  mounted.push(app)
  await settle()
  return root
}
beforeEach(() => vi.resetAllMocks())
afterEach(() => { for (const app of mounted.splice(0)) app.unmount() })

describe('revision-driven overview snapshots', () => {
  it('retains consumption while refreshing or after failure but clears another member’s data', async () => {
    api.consumption.mockResolvedValue({ meta, data: { total: 1, limit: 25, offset: 0, items: [{
      id: 'usage-a', request_id: 'request-a', started_at: range.from, user_id: 'member-a', model: 'model-a',
      provider: 'provider-a', status: 'success', settlement_status: 'settled', attribution_kind: 'employee',
      rated_amount: amount, billable_amount: amount, quota_covered_amount: amount, wallet_consumed_amount: amount, wallet_debit_amount: amount,
    }] } })
    const revision = ref(0)
    const userId = ref('member-a')
    const root = await mount(() => h(ConsumptionTable, { userId: userId.value, revision: revision.value }))
    expect(root.textContent).toContain('request-a')
    let rejectRefresh!: (reason: Error) => void
    api.consumption.mockImplementationOnce(() => new Promise((_resolve, reject) => { rejectRefresh = reject }))
    revision.value += 1
    await nextTick()
    expect(root.textContent).toContain('request-a')
    rejectRefresh(new Error('consumption unavailable'))
    await settle()
    expect(root.textContent).toContain('request-a')
    expect(root.querySelector('[role="alert"]')?.textContent).toContain('consumption unavailable')
    api.consumption.mockRejectedValue(new Error('member lookup unavailable'))
    userId.value = 'member-b'
    await settle()
    expect(root.textContent).not.toContain('request-a')
    expect(api.consumption.mock.lastCall?.[0].user_id).toBe('member-b')
  })
})
