import { createApp, h, nextTick, reactive } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { setI18nLocale } from '@/i18n'
import UserUsageStats from '../users/UserUsageStats.vue'

const api = vi.hoisted(() => ({
  users: vi.fn(), groups: vi.fn(),
  userLeaderboard: vi.fn(), groupLeaderboard: vi.fn(), series: vi.fn(),
}))
vi.mock('@/api/users', () => ({ usersApi: { getAllUsers: api.users, listUserGroups: api.groups } }))
vi.mock('@/api/admin', () => ({ adminApi: { getLeaderboardUsers: api.userLeaderboard, getLeaderboardUserGroups: api.groupLeaderboard, getTimeSeries: api.series } }))
vi.mock('@/components/charts/LineChart.vue', () => ({ default: { render: () => null } }))
vi.mock('@/components/common', () => ({ EmptyState: { render: () => null }, LoadingState: { render: () => null }, TimeRangePicker: { render: () => null } }))
vi.mock('@/components/ui', async importOriginal => {
  const original = await importOriginal<object>()
  const { defineComponent, h } = await import('vue')
  return {
    ...original,
    Select: defineComponent({
      props: { modelValue: String }, emits: ['update:modelValue'],
      setup: (props, { slots, emit }) => () => h('select', {
        value: props.modelValue,
        onChange: (event: Event) => emit('update:modelValue', (event.target as HTMLSelectElement).value),
      }, slots.default?.()),
    }),
    SelectTrigger: { render: () => null },
    SelectContent: defineComponent({ inheritAttrs: false, setup: (_, { slots }) => () => slots.default?.() }),
    SelectItem: defineComponent({ props: { value: String }, setup: (props, { slots }) => () => h('option', { value: props.value }, slots.default?.()) }),
  }
})

let unmount = () => {}
const range = { from: '2026-09-10T15:20:42.000Z', to: '2026-09-10T16:20:42.000Z', timezone: 'Asia/Shanghai' }
let props = reactive({ range: { ...range }, revision: 0 })
async function settle() {
  for (let index = 0; index < 10; index += 1) await Promise.resolve()
  await nextTick()
}
async function mount() {
  const root = document.createElement('div')
  const app = createApp({
    render: () => h(UserUsageStats, props, {
      'user-leaderboard': ({ selectUser }: { selectUser: (user: { user_id: string; username: string }) => void }) => h('button', {
        'data-select-account': '',
        onClick: () => selectUser({ user_id: 'user-3', username: 'Charlie' }),
      }, 'Merged user accounts'),
    }),
  })
  app.mount(root)
  unmount = () => app.unmount()
  await settle()
  return root
}
async function select(root: HTMLElement, index: number, value: string) {
  const control = root.querySelectorAll('select')[index]
  control.value = value
  control.dispatchEvent(new Event('change'))
  await nextTick()
  await vi.advanceTimersByTimeAsync(120)
  await settle()
}
beforeEach(() => {
  vi.useFakeTimers()
  vi.clearAllMocks()
  setI18nLocale('en-US')
  props = reactive({ range: { ...range }, revision: 0 })
  api.users.mockResolvedValue([{ id: 'user-1', username: 'Alice', is_active: true, groups: [{ id: 'group-1' }] }, { id: 'user-2', username: 'Bob', is_active: true, groups: [] }])
  api.groups.mockResolvedValue({ items: [{ id: 'group-1', name: 'Engineering' }, { id: 'group-2', name: 'Support' }] })
  api.userLeaderboard.mockResolvedValue({ items: [], total: 0 })
  api.groupLeaderboard.mockResolvedValue({ items: [], total: 0 })
  api.series.mockResolvedValue([])
})
afterEach(() => { unmount(); vi.useRealTimers(); setI18nLocale('zh-CN') })

describe('user and group usage statistics', () => {
  it('uses the merged account table without fetching another user leaderboard', async () => {
    const root = await mount()
    expect(root.textContent).toContain('Merged user accounts')
    expect(api.userLeaderboard).not.toHaveBeenCalled()
    expect(api.groupLeaderboard).not.toHaveBeenCalled()
    expect(root.textContent).not.toContain('User summary')
    expect(root.textContent).not.toContain('User-group summary')

    root.querySelector<HTMLButtonElement>('[data-select-account]')!.click()
    await nextTick()
    await vi.advanceTimersByTimeAsync(120)
    await settle()
    expect(root.querySelectorAll('select')[1].value).toBe('user-3')
    expect(root.querySelector('h3')?.parentElement?.textContent).toContain('Charlie')
    expect(api.series).toHaveBeenLastCalledWith(expect.objectContaining({ ...range, user_id: 'user-3' }), { skipCache: true })

    await select(root, 0, 'user_group')
    expect(root.querySelector('[data-select-account]')).toBeNull()
    await select(root, 0, 'user')
    expect(root.querySelectorAll('select')[1].value).toBe('user-3')
    expect(root.querySelector('h3')?.parentElement?.textContent).toContain('Charlie')
    expect(api.userLeaderboard.mock.calls.every(([params]) => params.user_group_id)).toBe(true)
  })

  it('applies group scope to trends and member rankings, and can compare groups', async () => {
    const root = await mount()
    await select(root, 0, 'user_group')
    expect(api.groupLeaderboard).toHaveBeenCalled()
    expect(api.userLeaderboard).toHaveBeenLastCalledWith(expect.objectContaining({ ...range, user_group_id: 'group-1', limit: 10 }), { skipCache: true })
    await select(root, 2, 'group-2')
    expect(api.series).toHaveBeenCalledWith(expect.objectContaining({ ...range, user_group_id: 'group-2' }), { skipCache: true })
    expect(root.textContent).toContain('Group member leaderboard')
  })

  it('keeps the ungrouped sentinel across scoped queries without fetching a fictitious group', async () => {
    const root = await mount()
    await select(root, 0, 'user_group')
    await select(root, 1, '__ungrouped__')
    expect(api.series).toHaveBeenLastCalledWith(expect.objectContaining({ user_group_id: '__ungrouped__' }), { skipCache: true })
    expect(api.userLeaderboard).toHaveBeenLastCalledWith(expect.objectContaining({ user_group_id: '__ungrouped__' }), { skipCache: true })
  })

  it('uses the page range and refresh for trends and comparisons without losing selection', async () => {
    const root = await mount()
    await select(root, 2, 'user-2')
    api.userLeaderboard.mockClear()
    api.series.mockClear()
    props.range = { from: '2026-09-03T02:47:00.000Z', to: '2026-10-03T02:47:00.000Z', timezone: 'Asia/Shanghai' }
    await nextTick()
    await vi.advanceTimersByTimeAsync(120)
    expect(api.userLeaderboard).not.toHaveBeenCalled()
    expect(api.series).toHaveBeenCalledWith(expect.objectContaining({ ...props.range, user_id: 'user-2' }), { skipCache: true })
    props.revision += 1
    await nextTick()
    await vi.advanceTimersByTimeAsync(120)
    expect(api.userLeaderboard).not.toHaveBeenCalled()
    expect(api.series).toHaveBeenCalledTimes(4)
    expect(root.querySelector('h1')).toBeNull()
    expect(root.querySelectorAll('select')[2].value).toBe('user-2')
  })

  it('discards the old trend when the page range changes during a request', async () => {
    let resolveOld: (value: unknown) => void = () => {}
    api.series.mockImplementationOnce(() => new Promise(resolve => { resolveOld = resolve }))
    const root = await mount()
    props.range = { ...range, from: '2026-09-10T16:00:00.000Z' }
    await nextTick()
    resolveOld([{ date: 'old', total_cost: 999 }])
    await settle()
    expect(root.textContent).not.toContain('999')
    await vi.advanceTimersByTimeAsync(120)
    expect(api.series.mock.lastCall?.[0]).toMatchObject(props.range)
  })

  it('reports failed requests and retries them without unmounting the usage section', async () => {
    api.series.mockRejectedValueOnce(new Error('Trend unavailable'))
    const root = await mount()
    expect(root.textContent).toContain('Trend unavailable')
    const retry = [...root.querySelectorAll('button')].find(item => item.textContent?.includes('Retry'))
    expect(retry).toBeDefined()
    retry!.click()
    await settle()
    expect(api.series).toHaveBeenCalledTimes(2)
    expect(api.userLeaderboard).not.toHaveBeenCalled()
    expect(root.textContent).not.toContain('Trend unavailable')
  })

  it('retries failed group rankings while preserving scoped member rankings', async () => {
    api.groupLeaderboard.mockRejectedValueOnce(new Error('Group rankings unavailable'))
    const root = await mount()
    await select(root, 0, 'user_group')
    expect(root.textContent).toContain('Group rankings unavailable')
    const retry = [...root.querySelectorAll('button')].find(item => item.textContent?.includes('Retry'))
    retry!.click()
    await settle()
    expect(api.groupLeaderboard).toHaveBeenCalledTimes(2)
    expect(api.userLeaderboard).toHaveBeenLastCalledWith(expect.objectContaining({ user_group_id: 'group-1' }), { skipCache: true })
    expect(root.textContent).not.toContain('Group rankings unavailable')
  })
})
