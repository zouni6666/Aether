import { createApp, nextTick, type Component } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import UserStats from '@/views/admin/UserStats.vue'
import UserAnalysisDetail from '@/views/admin/UserAnalysisDetail.vue'
import { setI18nLocale } from '@/i18n'

const api = vi.hoisted(() => ({ users: vi.fn(), user: vi.fn(), timeseries: vi.fn(), breakdown: vi.fn(), consumption: vi.fn(), exportCsv: vi.fn() }))
const accountApi = vi.hoisted(() => ({ wallets: vi.fn(), transactions: vi.fn(), plans: vi.fn() }))
const usageStats = vi.hoisted(() => ({ selectUser: vi.fn() }))
vi.mock('@/api/overview', () => ({ overviewApi: api }))
vi.mock('@/api/admin-wallets', () => ({ adminWalletApi: { listWallets: accountApi.wallets, getWalletTransactions: accountApi.transactions } }))
vi.mock('@/api/users', () => ({ usersApi: { listUserPlanEntitlements: accountApi.plans } }))
vi.mock('@/components/charts/BarChart.vue', () => ({ default: { render: () => null } }))
vi.mock('@/components/charts/LineChart.vue', () => ({ default: { render: () => null } }))
vi.mock('@/features/overview/users/UserUsageStats.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return { default: defineComponent({
    setup: (_, { slots }) => () => h('div', { 'data-user-usage-stats': '' }, slots['user-leaderboard']?.({ selectUser: usageStats.selectUser })),
  }) }
})
const range = 'from=2026-09-01T00:00:00Z&to=2026-09-02T00:00:00Z&timezone=UTC'
const meta = {
  schema_version: 1, metric_version: 'overview-v2', scope: { kind: 'installation' },
  range: { from: '2026-09-01T00:00:00Z', to: '2026-09-02T00:00:00Z', timezone: 'UTC', time_basis: 'request_started_at' },
  generated_at: '2026-09-02T00:01:00Z', data_through: '2026-09-02T00:00:00Z', read_revision: 'test-1',
  coverage: { status: 'complete', request_count: 1, usage_available_count: 1, pricing_available_count: 1, settled_count: 1, attribution_available_count: 1 },
}
const amount = (value: string | null) => ({ value, currency: 'USD', basis: 'billable', status: value == null ? 'unknown' : 'known' })
const metrics = { request_count: 120, successful_request_count: 118, failed_request_count: 2, total_tokens: 8800, billable_amount: amount('432.25'), quota_covered_amount: amount('300'), wallet_debit_amount: amount('132.25') }
const finance = { wallet_balance: amount('765.50'), recharge_balance: amount('700'), gift_balance: amount('65.5'), recharge_amount: amount('1000'), recharge_count: 4, plan_purchase_amount: amount('99'), plan_purchase_count: 1, gift_credit_amount: amount('20'), gift_credit_count: 1, balance_time_basis: 'current', payment_time_basis: 'credited_at' }
const employee = { user_id: 'employee-0', username: 'Zero Usage Employee', email: 'zero@example.test', is_active: true, last_used_at: null, active_days: 0, request_count: 0, total_tokens: 0, billable_amount: amount('0'), finance }
const wallet = { id: 'wallet-0', user_id: employee.user_id, owner_type: 'user', balance: 765.5, recharge_balance: 700, gift_balance: 65.5, currency: 'USD' }
const transaction = { id: 'tx-0', category: 'adjust', reason_code: 'adjust_admin', amount: -10, balance_before: 775.5, balance_after: 765.5, recharge_balance_before: 710, recharge_balance_after: 700, gift_balance_before: 65.5, gift_balance_after: 65.5, description: 'balance correction', created_at: '2026-08-01T08:00:00Z' }
const plan = { id: 'plan-history-0', user_id: employee.user_id, plan_id: 'plan-0', plan_title: 'Old monthly plan', payment_order_id: 'order-0', status: 'revoked', active: false, starts_at: '2026-07-01T00:00:00Z', expires_at: '2026-08-01T00:00:00Z', updated_at: '2026-08-01T00:00:00Z', entitlements: [{ type: 'daily_quota', daily_quota_usd: 20 }] }
const cleanup: (() => void)[] = []
async function settle() { await Promise.resolve(); await Promise.resolve(); await nextTick(); await new Promise(resolve => setTimeout(resolve, 0)) }
async function mount(component: Component, path: string) {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/admin/user-stats', component: UserStats }, { path: '/admin/user-stats/:userId', component: UserAnalysisDetail }, { path: '/admin/usage', component: { render: () => null } }] })
  await router.push(path)
  const root = document.createElement('div')
  const app = createApp(component).use(router)
  app.mount(root)
  cleanup.push(() => app.unmount())
  await settle()
  return { root, router }
}
function button(root: Element, label: string) {
  const result = [...root.querySelectorAll<HTMLButtonElement>('button')].find(item => item.textContent?.includes(label) || item.getAttribute('aria-label')?.includes(label))
  if (!result) throw new Error(`Missing button: ${label}`)
  return result
}
function section(root: Element, selector: string) {
  const result = root.querySelector(selector)
  if (!result) throw new Error(`Missing section: ${selector}`)
  return result
}
beforeEach(() => {
  vi.clearAllMocks()
  setI18nLocale('zh-CN')
  api.users.mockImplementation(async query => ({ meta, data: { total: 62, limit: query.limit, offset: query.offset, items: [employee], summary: { ...metrics, user_count: 62, active_user_count: 7 }, finance_summary: finance } }))
  api.user.mockImplementation(async (id, query) => ({ meta, data: { user: { id, username: id, email: 'member@example.test', is_active: true }, summary: metrics, finance, payments: { items: [{ id: 'payment-1', order_no: 'credited-order', kind: 'wallet_recharge', amount: amount('1000'), payment_method: 'alipay', credited_at: '2026-09-01T01:00:00Z' }], total: 80, limit: query.payment_limit, offset: query.payment_offset } } }))
  api.timeseries.mockResolvedValue({ meta, data: { items: [], granularity: 'hour' } })
  api.breakdown.mockResolvedValue({ meta, data: { items: [], total: 0, limit: 10, offset: 0 } })
  api.consumption.mockImplementation(async query => ({ meta, data: { items: [{ id: 'usage-1', request_id: 'request-1', started_at: '2026-09-01T02:00:00Z', model: 'test-model', provider: 'provider', status: 'success', settlement_status: 'settled', billable_amount: amount('10'), quota_covered_amount: amount('8'), wallet_debit_amount: amount('2') }], total: 75, limit: query.limit, offset: query.offset } }))
  accountApi.wallets.mockResolvedValue({ items: [wallet], total: 1, limit: 1, offset: 0 })
  accountApi.transactions.mockImplementation(async (_id, query) => ({ wallet, items: [transaction], total: 60, ...query }))
  accountApi.plans.mockResolvedValue({ items: [plan], total: 1 })
})
afterEach(() => { cleanup.splice(0).forEach(fn => fn()); vi.useRealTimers(); setI18nLocale('zh-CN') })

describe('enterprise user accounts', () => {
  it('shows account and usage statistics together on one page', async () => {
    const { root } = await mount(UserStats, `/admin/user-stats?${range}`)
    expect(root.querySelector('[data-user-usage-stats]')).not.toBeNull()
    expect(section(root, '[data-user-accounts]').textContent).toContain(employee.username)
    expect(section(root, '[data-user-reports]').querySelector('[data-user-accounts]')).not.toBeNull()
    expect(root.querySelectorAll('[data-user-accounts]')).toHaveLength(1)
    expect(section(root, '[data-user-accounts]').textContent).toContain('用户排行与账目')
    expect(root.querySelector('[role="tablist"]')).toBeNull()
    expect(root.querySelector('button[data-value="accounts"]')).toBeNull()
    expect(api.users).toHaveBeenCalledTimes(1)
  })

  it('shows full-roster financial totals, keeps zero-use users and delegates sorting/pagination', async () => {
    const { root } = await mount(UserStats, `/admin/user-stats?${range}&attribution_kind=standalone&model=legacy`)
    expect(root.textContent).toContain('Zero Usage Employee')
    expect(section(root, '[data-user-summary="consumption"]').textContent).toContain('432.25')
    expect(section(root, '[data-user-summary="activity"]').textContent).toMatch(/7\s*\/ 62/)
    expect(root.querySelector('select[aria-label="归属"]')).toBeNull()
    expect(api.users.mock.lastCall?.[0]).toMatchObject({ sort: 'billable_amount', order: 'desc', limit: 25, offset: 0 })
    expect(api.users.mock.lastCall?.[0]).not.toHaveProperty('attribution_kind')
    expect(api.users.mock.lastCall?.[0]).not.toHaveProperty('model')
    expect(section(root, '[data-user-rank]').textContent?.trim()).toBe('1')
    button(root, '第 2 页').click()
    await settle()
    expect(api.users.mock.lastCall?.[0]).toMatchObject({ offset: 25 })
    expect(section(root, '[data-user-rank]').textContent?.trim()).toBe('26')
    button(root, 'Tokens').click()
    await settle()
    expect(api.users.mock.lastCall?.[0]).toMatchObject({ offset: 0, sort: 'total_tokens', order: 'desc' })
    expect(section(root, '[data-user-rank]').textContent?.trim()).toBe('1')
    expect(root.querySelector('a[href*="employee-0"]')).toBeNull()
    expect(root.querySelector('a[href*="/admin/usage"]')).toBeNull()
    expect([...section(root, '[data-user-accounts]').querySelectorAll('a, button')].some(item => item.textContent?.trim() === '使用记录')).toBe(false)
  })
  it('opens trends from the merged table and labels ascending rows as positions', async () => {
    const { root } = await mount(UserStats, `/admin/user-stats?${range}`)
    const accounts = section(root, '[data-user-accounts]')
    button(accounts, '使用趋势').click()
    expect(usageStats.selectUser).toHaveBeenCalledWith(expect.objectContaining({ user_id: employee.user_id, username: employee.username }))
    expect(accountApi.wallets).not.toHaveBeenCalled()
    button(accounts, '消费').click()
    await settle()
    expect(api.users.mock.lastCall?.[0]).toMatchObject({ sort: 'billable_amount', order: 'asc', offset: 0 })
    expect(accounts.querySelector('thead')?.textContent).toContain('序号')
    expect(accounts.querySelector('thead')?.textContent).not.toContain('排名')
  })
  it('includes accounts in reports and opens reusable account history without leaving the page', async () => {
    const { root, router } = await mount(UserStats, `/admin/user-stats?${range}`)
    const reports = section(root, '[data-user-reports]')
    const accounts = section(root, '[data-user-accounts]')
    expect(reports.contains(accounts)).toBe(true)
    expect(accountApi.wallets).not.toHaveBeenCalled()
    expect(accountApi.plans).not.toHaveBeenCalled()
    const before = router.currentRoute.value.fullPath
    const accountButton = button(accounts, '账目')
    expect(accountButton.textContent?.trim()).toBe('')
    expect(accountButton.querySelector('svg')).not.toBeNull()
    accountButton.click()
    await settle()
    await settle()
    const dialog = section(document.body, '[data-user-account-drawer]')
    expect(router.currentRoute.value.fullPath).toBe(before)
    expect(accountApi.wallets).toHaveBeenCalledWith({ user_id: employee.user_id, owner_type: 'user', limit: 1, offset: 0 }, expect.any(AbortSignal))
    expect(accountApi.plans).toHaveBeenCalledWith(employee.user_id, { include_inactive: true }, expect.any(AbortSignal))
    expect(dialog.textContent).toContain('balance correction')
    expect(dialog.textContent).toContain('775.5000 → 765.5000')
    expect(dialog.textContent).toContain('Old monthly plan')
    expect(dialog.textContent).toContain('已撤销')
    expect(dialog.textContent).toContain('每日额度')
    expect(dialog.textContent).not.toContain('撤销套餐')
    expect(dialog.textContent).not.toContain('发放套餐')
    button(dialog, '第 2 页').click()
    await settle()
    expect(accountApi.transactions.mock.lastCall?.slice(0, 2)).toEqual(['wallet-0', { limit: 25, offset: 25 }])
    expect(accountApi.plans).toHaveBeenCalledTimes(1)
    expect(api.users).toHaveBeenCalledTimes(1)
    expect(api.timeseries).toHaveBeenCalledTimes(1)
    document.body.querySelector<HTMLButtonElement>('[aria-label="关闭账目"]')?.click()
    await settle()
    expect(document.querySelector('[data-user-account-drawer]')).toBeNull()
  })
  it('keeps plan history available without a wallet and retries failed wallet lookup', async () => {
    accountApi.wallets.mockRejectedValueOnce(new Error('wallet unavailable')).mockResolvedValue({ items: [], total: 0 })
    const { root } = await mount(UserStats, `/admin/user-stats?${range}`)
    button(section(root, '[data-user-accounts]'), '账目').click()
    await settle()
    const dialog = section(document.body, '[data-user-account-drawer]')
    expect(dialog.textContent).toContain('wallet unavailable')
    expect(dialog.textContent).toContain('Old monthly plan')
    button(dialog, '重试').click()
    await settle()
    expect(dialog.textContent).toContain('该用户尚无钱包')
    expect(accountApi.transactions).not.toHaveBeenCalled()
  })
  it('discards closed account requests before opening another user', async () => {
    let resolveOld: (value: unknown) => void = () => {}
    accountApi.wallets.mockImplementationOnce(() => new Promise(resolve => { resolveOld = resolve }))
    const second = { ...employee, user_id: 'employee-1', username: 'Second employee' }
    api.users.mockResolvedValue({ meta, data: { total: 2, limit: 25, offset: 0, items: [employee, second], summary: metrics, finance_summary: finance } })
    const { root } = await mount(UserStats, `/admin/user-stats?${range}`)
    const accounts = section(root, '[data-user-accounts]')
    button(accounts, employee.username).click()
    await settle()
    const oldSignal = accountApi.wallets.mock.lastCall?.[1] as AbortSignal
    document.body.querySelector<HTMLButtonElement>('[aria-label="关闭账目"]')?.click()
    await settle()
    expect(oldSignal.aborted).toBe(true)
    accountApi.wallets.mockResolvedValue({ items: [{ ...wallet, id: 'wallet-1', user_id: second.user_id }], total: 1 })
    accountApi.transactions.mockResolvedValue({ wallet: { ...wallet, id: 'wallet-1', user_id: second.user_id }, items: [{ ...transaction, description: 'second account only' }], total: 1 })
    button(accounts, second.username).click()
    await settle()
    resolveOld({ items: [wallet], total: 1 })
    await settle()
    const dialog = section(document.body, '[data-user-account-drawer]')
    expect(dialog.textContent).toContain('second account only')
    expect(accountApi.transactions.mock.calls.every(call => call[0] === 'wallet-1')).toBe(true)
  })
  it('loads accounts independently of pending and failed reports', async () => {
    api.timeseries.mockImplementation(() => new Promise(() => {}))
    api.breakdown.mockRejectedValue(new Error('models unavailable'))
    const { root } = await mount(UserStats, `/admin/user-stats?${range}`)
    expect(root.textContent).toContain('Zero Usage Employee')
    expect(section(root, '[data-user-summary="recharge"]').textContent).toContain('1,000.00')
    expect(section(root, '[data-user-report="models"]').textContent).toContain('模型报表暂不可用')
    expect(root.textContent).toContain('全站用量')
    expect(root.textContent).not.toContain('全体用户')
  })
  it('keeps missing finance unknown instead of showing a zero balance', async () => {
    api.users.mockResolvedValue({ meta, data: { total: 1, limit: 25, offset: 0, items: [{ ...employee, finance: null }], summary: { ...metrics, user_count: 1, active_user_count: 0 }, finance_summary: null } })
    const { root } = await mount(UserStats, `/admin/user-stats?${range}`)
    expect(section(root, '[data-user-summary="balance"]').textContent).toMatch(/余额\s*-/)
    expect(section(root, '[data-user-summary="balance"]').textContent).not.toContain('$0')
  })
  it('searches account totals without refetching the installation reports', async () => {
    const { root } = await mount(UserStats, `/admin/user-stats?${range}`)
    const input = root.querySelector<HTMLInputElement>('input[aria-label="搜索用户"]')
    if (!input) throw new Error('Missing user search')
    input.value = 'Zero'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    input.closest('form')?.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }))
    await settle()
    expect(api.users.mock.lastCall?.[0]).toMatchObject({ search: 'Zero', offset: 0 })
    expect(api.timeseries).toHaveBeenCalledTimes(1)
    expect(api.breakdown).toHaveBeenCalledTimes(1)
    expect(root.textContent).toContain('上方汇总与账目仅包含匹配的用户')
  })
  it('shows all sections and paginates payments independently from usage', async () => {
    const { root } = await mount(UserAnalysisDetail, `/admin/user-stats/member?${range}&tab=usage&model=legacy`)
    expect(root.querySelector('[role="tablist"]')).toBeNull()
    expect(root.textContent).toContain('credited-order')
    expect(root.textContent).toContain('request-1')
    expect(section(root, '[data-user-consumption]').textContent).not.toContain('规则计价')
    expect(section(root, '[data-user-consumption]').textContent).toContain('额度抵扣')
    expect(api.user.mock.lastCall?.[1]).toMatchObject({ user_id: 'member', payment_limit: 25, payment_offset: 0 })
    expect(api.consumption.mock.lastCall?.[0]).not.toHaveProperty('model')
    section(root, '[data-user-payments]').querySelector<HTMLButtonElement>('[aria-label="下一页"]')?.click()
    await settle()
    expect(api.user.mock.lastCall?.[1]).toMatchObject({ payment_offset: 25 })
    expect(api.consumption).toHaveBeenCalledTimes(1)
    expect(api.timeseries).toHaveBeenCalledTimes(1)
    section(root, '[data-user-consumption]').querySelector<HTMLButtonElement>('[aria-label="下一页"]')?.click()
    await settle()
    expect(api.consumption.mock.lastCall?.[0]).toMatchObject({ offset: 25 })
    expect(api.user).toHaveBeenCalledTimes(2)
    expect(api.breakdown.mock.lastCall?.[0]).toMatchObject({ user_id: 'member', group_by: 'model', sort: 'total_tokens' })
  })
  it('clears money and resets payment pagination when the selected period changes', async () => {
    const { root, router } = await mount(UserAnalysisDetail, `/admin/user-stats/member?${range}`)
    section(root, '[data-user-payments]').querySelector<HTMLButtonElement>('[aria-label="下一页"]')?.click()
    await settle()
    api.user.mockImplementation(() => new Promise(() => {}))
    await router.replace('/admin/user-stats/member?from=2026-08-01T00:00:00Z&to=2026-08-02T00:00:00Z&timezone=UTC')
    await settle()
    expect(api.user.mock.lastCall?.[1]).toMatchObject({ payment_offset: 0, from: '2026-08-01T00:00:00.000Z' })
    expect(root.textContent).not.toContain('credited-order')
    expect(section(root, '[data-user-summary="consumption"]').textContent).not.toContain('432.25')
  })
  it('refreshes each section once on activation and every 10 seconds, then stops', async () => {
    const { root } = await mount(UserStats, '/admin/user-stats?relative_preset=today&timezone=UTC')
    vi.useFakeTimers()
    vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('visible')
    root.querySelector<HTMLButtonElement>('[aria-label="开启自动刷新（每 10 秒）"]')?.click()
    await nextTick()
    expect(api.users).toHaveBeenCalledTimes(2)
    expect(api.timeseries).toHaveBeenCalledTimes(2)
    expect(api.breakdown).toHaveBeenCalledTimes(2)
    await vi.advanceTimersByTimeAsync(10_000)
    expect(api.users).toHaveBeenCalledTimes(3)
    expect(api.timeseries).toHaveBeenCalledTimes(3)
    expect(api.breakdown).toHaveBeenCalledTimes(3)
    root.querySelector<HTMLButtonElement>('[aria-label="关闭自动刷新（每 10 秒）"]')?.click()
    await vi.advanceTimersByTimeAsync(20_000)
    expect(api.users).toHaveBeenCalledTimes(3)
    vi.restoreAllMocks()
  })
  it('exports the user report for the selected period and matching user search', async () => {
    api.exportCsv.mockRejectedValue(new Error('export check'))
    const { root } = await mount(UserStats, `/admin/user-stats?${range}&search=alice&offset=25`)
    button(root, '导出用户报表').click()
    await settle()
    expect(api.exportCsv.mock.lastCall?.[0]).toBe('users')
    expect(api.exportCsv.mock.lastCall?.[1]).toMatchObject({ search: 'alice', from: '2026-09-01T00:00:00.000Z', to: '2026-09-02T00:00:00.000Z', timezone: 'UTC', sort: 'billable_amount' })
    expect(root.textContent).toContain('export check')
  })
})
