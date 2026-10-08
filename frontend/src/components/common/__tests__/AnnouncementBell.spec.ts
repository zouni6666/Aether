import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, h, nextTick, reactive, type App, type ComputedRef } from 'vue'
import AnnouncementBell from '../AnnouncementBell.vue'
import type { Announcement } from '@/api/announcements'
import { setI18nLocale } from '@/i18n'

interface PopoverState { open: ComputedRef<boolean>; toggle: () => void }
vi.mock('@/components/ui/popover', async () => {
  const { cloneVNode, computed, defineComponent, h, inject, provide } = await import('vue')
  return {
    Popover: defineComponent({
      props: { open: Boolean }, emits: ['update:open'],
      setup: (props, { slots, emit }) => {
        provide<PopoverState>('announcement-popover', { open: computed(() => props.open), toggle: () => emit('update:open', !props.open) })
        return () => h('div', slots.default?.())
      },
    }),
    PopoverTrigger: defineComponent({ setup: (_, { slots }) => {
      const state = inject<PopoverState>('announcement-popover')!
      return () => cloneVNode(slots.default!()[0]!, { onClick: state.toggle })
    } }),
    PopoverContent: defineComponent({ setup: (_, { slots }) => {
      const state = inject<PopoverState>('announcement-popover')!
      return () => state.open.value ? h('section', slots.default?.()) : null
    } }),
  }
})
vi.mock('@/components/ui/tooltip', async () => {
  const { defineComponent, h } = await import('vue')
  const passthrough = defineComponent({ setup: (_, { slots }) => () => h('div', slots.default?.()) })
  return { Tooltip: passthrough, TooltipProvider: passthrough, TooltipTrigger: passthrough, TooltipContent: defineComponent({ setup: () => () => null }) }
})

const announcement = (is_read = false): Announcement => ({
  id: is_read ? 'read' : 'unread', title: is_read ? 'Earlier notice' : 'Service update',
  content: '**Scheduled maintenance** with [details](https://example.com).', type: 'info',
  priority: 0, is_pinned: false, is_active: true, requires_ack: false, is_read,
  author: { id: 'admin', username: 'Admin' }, created_at: '2026-09-15T02:00:00Z', updated_at: '2026-09-15T02:00:00Z',
})
type Props = InstanceType<typeof AnnouncementBell>['$props']
const mounted: Array<{ app: App; root: HTMLElement }> = []
function mount(overrides: Partial<Props> = {}) {
  const props = reactive({ open: true, items: [] as Announcement[], unreadCount: 0 as number | null, loading: false, error: null as string | null, hasMore: false, markingAll: false, canManage: false, ...overrides })
  const events = { select: vi.fn(), refresh: vi.fn(), loadMore: vi.fn(), readAll: vi.fn(), create: vi.fn(), open: vi.fn() }
  const root = document.createElement('div')
  document.body.append(root)
  const app = createApp(() => h(AnnouncementBell, {
    ...props,
    'onUpdate:open': value => { events.open(value); props.open = value },
    onSelect: events.select, onRefresh: events.refresh, onLoadMore: events.loadMore,
    onReadAll: events.readAll, onCreate: events.create,
  }))
  app.mount(root)
  mounted.push({ app, root })
  return { root, props, events }
}
function button(root: HTMLElement, label: string) {
  const result = Array.from(root.querySelectorAll<HTMLButtonElement>('button')).find(item => item.getAttribute('aria-label') === label || item.textContent?.trim() === label)
  if (!result) throw new Error(`Button not found: ${label}`)
  return result
}
beforeEach(() => setI18nLocale('zh-CN'))
afterEach(() => { for (const { app, root } of mounted.splice(0)) { app.unmount(); root.remove() } })

describe('AnnouncementBell presentation', () => {
  it('keeps unknown and zero counts hidden, caps the visible badge, and emits controlled open changes', async () => {
    const { root, props, events } = mount({ unreadCount: null, open: false })
    expect(root.querySelector('[data-unread-badge]')).toBeNull()
    expect(button(root, '公告').className).toContain('h-9 w-9')
    button(root, '公告').click()
    await nextTick()
    expect(events.open).toHaveBeenCalledWith(true)
    expect(button(root, '全部标为已读').disabled).toBe(true)
    props.unreadCount = 0
    await nextTick()
    expect(root.querySelector('[data-unread-badge]')).toBeNull()
    props.unreadCount = 137
    await nextTick()
    expect(root.querySelector('[data-unread-badge]')?.textContent).toBe('99+')
    expect(button(root, '公告，137 条未读').getAttribute('title')).toBe('公告，137 条未读')
    expect(button(root, '全部标为已读').disabled).toBe(false)
  })

  it('emits selection without changing reading state and renders plain Markdown summaries', () => {
    const unread = announcement()
    const { root, events } = mount({ items: [unread, announcement(true)], unreadCount: 1 })
    expect(root.textContent).toContain('Scheduled maintenance with details.')
    expect(root.querySelector('a')).toBeNull()
    expect(button(root, 'Service update, 未读')).toBeTruthy()
    expect(button(root, 'Earlier notice, 已读')).toBeTruthy()
    button(root, 'Service update, 未读').click()
    expect(events.select).toHaveBeenCalledWith(unread)
    expect(unread.is_read).toBe(false)
    expect(events.readAll).not.toHaveBeenCalled()
  })

  it('retains rows during refresh failures and provides retry, loading, and pagination states', async () => {
    const { root, props, events } = mount({ items: [announcement()], error: '公告加载失败', hasMore: true })
    expect(root.querySelector('[role="alert"]')?.textContent).toContain('公告加载失败')
    button(root, '重试').click()
    button(root, '加载更多').click()
    expect(events.refresh).toHaveBeenCalledTimes(1)
    expect(events.loadMore).toHaveBeenCalledTimes(1)
    props.loading = true
    await nextTick()
    expect(root.querySelector('[role="status"]')?.textContent).toContain('加载中')
    expect(button(root, 'Service update, 未读')).toBeTruthy()
    expect(button(root, '重试').disabled).toBe(true)
    expect(button(root, '加载更多').disabled).toBe(true)
  })

  it('keeps publishing, refresh, and mark-all commands without a management navigation button', async () => {
    const { root, props, events } = mount({ unreadCount: 2 })
    expect(root.querySelector('[aria-label="发布公告"]')).toBeNull()
    expect(root.querySelector('[aria-label="管理公告"]')).toBeNull()
    props.canManage = true
    await nextTick()
    expect(root.querySelector('[aria-label="管理公告"]')).toBeNull()
    button(root, '发布公告').click()
    button(root, '刷新公告').click()
    button(root, '全部标为已读').click()
    expect(events.create).toHaveBeenCalledTimes(1)
    expect(events.refresh).toHaveBeenCalledTimes(1)
    expect(events.readAll).toHaveBeenCalledTimes(1)
    props.markingAll = true
    await nextTick()
    expect(button(root, '全部标为已读').disabled).toBe(true)
    button(root, '全部标为已读').click()
    expect(events.readAll).toHaveBeenCalledTimes(1)
  })

  it('shows an empty state without inventing an unread count and follows the active locale', async () => {
    const { root } = mount({ unreadCount: null })
    expect(root.textContent).toContain('暂无公告')
    setI18nLocale('en-US')
    await nextTick()
    expect(button(root, 'Announcements')).toBeTruthy()
    expect(root.textContent).toContain('No announcements')
    expect(root.querySelector('[data-unread-badge]')).toBeNull()
  })
})
