import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, nextTick, type App } from 'vue'
import Announcements from '../Announcements.vue'
import type { Announcement } from '@/api/announcements'
import { setI18nLocale } from '@/i18n'

const api = vi.hoisted(() => ({ getAnnouncements: vi.fn(), getUserAnnouncements: vi.fn(), createAnnouncement: vi.fn(), updateAnnouncement: vi.fn(), deleteAnnouncement: vi.fn() }))
const store = vi.hoisted(() => ({ markRead: vi.fn(), refresh: vi.fn(), items: [] as Announcement[], unreadCount: null as number | null }))
const auth = vi.hoisted(() => ({ isAdmin: true, canAccessAdmin: true }))
const route = vi.hoisted(() => ({ query: {} as Record<string, string> }))
const router = vi.hoisted(() => ({ replace: vi.fn() }))
vi.mock('@/api/announcements', () => ({ announcementApi: api }))
vi.mock('@/stores/announcements', async () => {
  const { reactive } = await import('vue')
  return { useAnnouncementStore: () => reactive(store) }
})
vi.mock('@/stores/auth', () => ({ useAuthStore: () => auth }))
vi.mock('vue-router', () => ({ useRoute: () => route, useRouter: () => router }))
vi.mock('@/composables/useToast', () => ({ useToast: () => ({ success: vi.fn(), error: vi.fn() }) }))
vi.mock('@/utils/logger', () => ({ log: { error: vi.fn() } }))
vi.mock('@/components/ui', async () => {
  const { defineComponent, h } = await import('vue')
  const wrapper = (tag = 'div') => defineComponent({ setup: (_, { slots }) => () => h(tag, slots.default?.()) })
  const input = (tag: string) => defineComponent({ props: { modelValue: { type: [String, Number], default: '' } }, emits: ['update:modelValue'], setup: (props, { emit }) => () => h(tag, { value: props.modelValue, onInput: (event: Event) => emit('update:modelValue', (event.target as HTMLInputElement).value) }) })
  return {
    Card: wrapper('section'), Button: wrapper('button'), Badge: wrapper('span'), Label: wrapper('label'), Input: input('input'), Textarea: input('textarea'),
    Table: wrapper('table'), TableHeader: wrapper('thead'), TableBody: wrapper('tbody'), TableRow: wrapper('tr'), TableHead: wrapper('th'), TableCell: wrapper('td'),
    Pagination: wrapper(), RefreshButton: wrapper('button'),
    Dialog: defineComponent({ props: { modelValue: Boolean }, setup: (props, { slots }) => () => props.modelValue ? h('div', { role: 'dialog' }, [slots.header?.(), slots.default?.(), slots.footer?.()]) : null }),
    Switch: defineComponent({ props: { modelValue: Boolean }, emits: ['update:modelValue'], setup: (props, { emit }) => () => h('button', { role: 'switch', 'aria-checked': props.modelValue, onClick: () => emit('update:modelValue', !props.modelValue) }) }),
  }
})
vi.mock('@/components/common', async () => {
  const { defineComponent, h } = await import('vue')
  return { AlertDialog: defineComponent({ props: { modelValue: Boolean }, emits: ['confirm'], setup: (props, { emit }) => () => props.modelValue ? h('button', { 'data-confirm-delete': '', onClick: () => emit('confirm') }, '删除') : null }) }
})

const notice = (overrides: Partial<Announcement> = {}): Announcement => ({
  id: 'notice-1', title: 'Release notice', content: 'The release is available.', type: 'info',
  priority: 0, is_pinned: false, is_active: true, requires_ack: false, is_read: false,
  author: { id: 'admin', username: 'Admin' }, created_at: '2026-09-14T00:00:00Z', updated_at: '2026-09-14T00:00:00Z', ...overrides,
})
const mounted: Array<{ app: App; root: HTMLElement }> = []
async function settle() { await nextTick(); await new Promise(resolve => setTimeout(resolve, 0)); await nextTick() }
async function mount() {
  const root = document.createElement('div')
  document.body.append(root)
  const app = createApp(Announcements)
  app.mount(root)
  mounted.push({ app, root })
  await settle()
  return root
}
function button(root: HTMLElement, text: string) {
  const result = Array.from(root.querySelectorAll<HTMLButtonElement>('button')).find(item => item.textContent?.trim() === text)
  if (!result) throw new Error(`Button not found: ${text}`)
  return result
}
function fill(root: HTMLElement, selector: string, value: string) {
  const input = root.querySelector<HTMLInputElement>(selector)!
  input.value = value
  input.dispatchEvent(new Event('input', { bubbles: true }))
}
function fixture(items: Announcement[]) {
  const response = { items, total: items.length, unread_count: items.filter(item => !item.is_read).length }
  api.getAnnouncements.mockResolvedValue(response)
  api.getUserAnnouncements.mockResolvedValue(response)
}

beforeEach(() => {
  vi.clearAllMocks()
  vi.spyOn(Date, 'now').mockReturnValue(Date.parse('2026-09-15T12:00:00Z'))
  setI18nLocale('zh-CN')
  auth.isAdmin = true
  auth.canAccessAdmin = true
  route.query = {}
  store.items = []
  store.unreadCount = null
  fixture([notice()])
  for (const action of [api.createAnnouncement, api.updateAnnouncement, api.deleteAnnouncement, store.markRead, store.refresh, router.replace]) action.mockResolvedValue(undefined)
})
afterEach(() => {
  for (const { app, root } of mounted.splice(0)) { app.unmount(); root.remove() }
  vi.restoreAllMocks()
})

describe('announcement publication and reading', () => {
  it('opens the administrator create deep link once and preserves unrelated query parameters', async () => {
    route.query = { create: '1', page: '2' }
    const root = await mount()
    expect(api.getUserAnnouncements).toHaveBeenCalledWith({ limit: 20, offset: 0 })
    expect(api.getAnnouncements).not.toHaveBeenCalled()
    expect(router.replace).toHaveBeenCalledWith({ query: { page: '2' } })
    expect(root.querySelector('[role="dialog"]')?.textContent).toContain('发布新的系统公告')
    expect(root.querySelector<HTMLInputElement>('#requires-ack')?.checked).toBe(false)
    expect(root.querySelector('label[for="requires-ack"]')?.textContent).toBe('弹窗提醒')
    expect(root.querySelector('#requires-ack')?.parentElement?.parentElement?.classList.contains('flex-wrap')).toBe(true)
    fill(root, '#title', 'Routine notice')
    fill(root, '#content', 'No interruption expected.')
    await nextTick()
    button(root, '创建').click()
    await settle()
    expect(api.createAnnouncement).toHaveBeenCalledWith(expect.objectContaining({ title: 'Routine notice', requires_ack: false }))
    expect(store.refresh).toHaveBeenCalledTimes(1)
    expect(root.querySelector('[role="dialog"]')).toBeNull()
  })

  it.each([false, true])('uses the member list for non-admin roles (audit access: %s) and never opens publishing', async canAccessAdmin => {
    auth.isAdmin = false
    auth.canAccessAdmin = canAccessAdmin
    route.query = { create: '1' }
    const root = await mount()
    expect(api.getUserAnnouncements).toHaveBeenCalledWith({ limit: 20, offset: 0 })
    expect(api.getAnnouncements).not.toHaveBeenCalled()
    expect(root.querySelector('[role="dialog"]')).toBeNull()
    expect(root.querySelector('[title="新建公告"]')).toBeNull()
    expect(router.replace).toHaveBeenCalledWith({ query: {} })
    root.querySelector<HTMLTableRowElement>('tbody tr')!.click()
    await settle()
    expect(store.markRead).toHaveBeenCalledWith('notice-1')
    expect(root.querySelector('[role="dialog"]')?.textContent).toContain('The release is available.')
  })

  it('marks a valid administrator preview as read once while opening its detail immediately', async () => {
    let finish!: () => void
    store.markRead.mockReturnValueOnce(new Promise<void>(resolve => { finish = resolve }))
    const root = await mount()
    root.querySelector<HTMLTableRowElement>('tbody tr')!.click()
    await nextTick()
    expect(root.querySelector('[role="dialog"]')?.textContent).toContain('The release is available.')
    expect(store.markRead).toHaveBeenCalledWith('notice-1')
    finish()
    await settle()
    button(root, '关闭').click()
    await nextTick()
    root.querySelector<HTMLTableRowElement>('tbody tr')!.click()
    await settle()
    expect(store.markRead).toHaveBeenCalledTimes(1)
    expect(root.textContent).not.toContain('1 条未读')
  })

  it.each([
    { is_active: false },
    { start_time: '2026-09-15T12:00:01Z' },
    { end_time: '2026-09-15T11:59:59Z' },
    { is_read: true },
  ])('does not acknowledge an inactive, scheduled, expired, or already-read preview: %j', async overrides => {
    fixture([notice(overrides)])
    const root = await mount()
    root.querySelector<HTMLTableRowElement>('tbody tr')!.click()
    await settle()
    expect(store.markRead).not.toHaveBeenCalled()
    expect(root.querySelector('[role="dialog"]')?.textContent).toContain('The release is available.')
  })

  it('includes the exact end boundary in the effective window, matching the server', async () => {
    fixture([notice({ end_time: '2026-09-15T12:00:00Z' })])
    const root = await mount()
    root.querySelector<HTMLTableRowElement>('tbody tr')!.click()
    await settle()
    expect(store.markRead).toHaveBeenCalledWith('notice-1')
  })

  it('preserves an existing popup reminder during editing and refreshes shared state after every mutation', async () => {
    fixture([notice({ requires_ack: true })])
    const root = await mount()
    expect(root.textContent).toContain('弹窗提醒')
    expect(root.textContent).not.toContain('必读')
    root.querySelector('tbody .lucide-square-pen')?.closest('button')?.click()
    await nextTick()
    expect(root.querySelector<HTMLInputElement>('#requires-ack')?.checked).toBe(true)
    button(root, '保存').click()
    await settle()
    expect(api.updateAnnouncement).toHaveBeenLastCalledWith('notice-1', expect.objectContaining({ requires_ack: true }))
    expect(store.refresh).toHaveBeenCalledTimes(1)
    root.querySelectorAll<HTMLButtonElement>('tbody [role="switch"]')[0]!.click()
    await settle()
    expect(api.updateAnnouncement).toHaveBeenLastCalledWith('notice-1', { is_pinned: true })
    root.querySelectorAll<HTMLButtonElement>('tbody [role="switch"]')[1]!.click()
    await settle()
    expect(api.updateAnnouncement).toHaveBeenLastCalledWith('notice-1', { is_active: false })
    expect(store.refresh).toHaveBeenCalledTimes(3)
    root.querySelector('tbody .lucide-trash-2')?.closest('button')?.click()
    await nextTick()
    root.querySelector<HTMLButtonElement>('[data-confirm-delete]')!.click()
    await settle()
    expect(api.deleteAnnouncement).toHaveBeenCalledWith('notice-1')
    expect(store.refresh).toHaveBeenCalledTimes(4)
  })

  it('keeps failed read acknowledgements unread and allows a later retry', async () => {
    store.markRead.mockRejectedValueOnce(new Error('Network unavailable'))
    const root = await mount()
    root.querySelector<HTMLTableRowElement>('tbody tr')!.click()
    await settle()
    expect(root.textContent).toContain('1 条未读')
    button(root, '关闭').click()
    await nextTick()
    root.querySelector<HTMLTableRowElement>('tbody tr')!.click()
    await settle()
    expect(store.markRead).toHaveBeenCalledTimes(2)
    expect(root.textContent).not.toContain('1 条未读')
  })

  it('synchronizes the current page after single and bulk reads through the announcement bell', async () => {
    fixture([notice(), notice({ id: 'notice-2', title: 'Second notice' })])
    const root = await mount()
    const { reactive } = await import('vue')
    const shared = reactive(store)
    shared.items = [notice({ is_read: true })]
    shared.unreadCount = 1
    await nextTick()
    expect(root.textContent).toContain('1 条未读')
    expect(root.querySelectorAll('tbody tr')[0]?.textContent).not.toContain('未读')
    expect(root.querySelectorAll('tbody tr')[1]?.textContent).toContain('未读')
    shared.unreadCount = 0
    await nextTick()
    expect(root.textContent).not.toContain('条未读')
    expect(root.querySelector('tbody')?.textContent).not.toContain('未读')
    expect(api.getUserAnnouncements).toHaveBeenCalledTimes(1)
    expect(store.markRead).not.toHaveBeenCalled()
  })
})
