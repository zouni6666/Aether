import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import type { Announcement, AnnouncementListResponse } from '@/api/announcements'

const api = vi.hoisted(() => ({
  getUserAnnouncements: vi.fn(),
  getUnreadCount: vi.fn(),
  getRequiredUnreadAnnouncements: vi.fn(),
  markAsRead: vi.fn(),
  markAllAsRead: vi.fn(),
}))

vi.mock('@/api/announcements', () => ({ announcementApi: api }))

import { useAnnouncementStore } from '@/stores/announcements'

function notice(id: string, overrides: Partial<Announcement> = {}): Announcement {
  return {
    id,
    title: `Notice ${id}`,
    content: `Content ${id}`,
    type: 'info',
    priority: 0,
    is_pinned: false,
    is_active: true,
    requires_ack: false,
    is_read: false,
    author: { id: 'author', username: 'Author' },
    created_at: '2026-09-15T00:00:00Z',
    updated_at: '2026-09-15T00:00:00Z',
    ...overrides,
  }
}

function page(items: Announcement[], total: number, unread_count: number): AnnouncementListResponse {
  return { items, total, unread_count }
}

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (error: Error) => void
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise
    reject = rejectPromise
  })
  return { promise, resolve, reject }
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.resetAllMocks()
  api.getUserAnnouncements.mockResolvedValue(page([], 0, 0))
  api.getUnreadCount.mockResolvedValue({ unread_count: 0 })
  api.getRequiredUnreadAnnouncements.mockResolvedValue(page([], 0, 0))
  api.markAsRead.mockResolvedValue({ message: 'read' })
  api.markAllAsRead.mockResolvedValue({ message: 'read all' })
})

describe('announcement store loading', () => {
  it('does not fetch while logged out and clears all personal state on logout', async () => {
    const store = useAnnouncementStore()
    await Promise.all([store.refresh(), store.loadList(), store.markRead('notice'), store.markAllRead()])
    expect(api.getUserAnnouncements).not.toHaveBeenCalled()
    expect(api.getUnreadCount).not.toHaveBeenCalled()
    expect(api.markAsRead).not.toHaveBeenCalled()
    expect(api.markAllAsRead).not.toHaveBeenCalled()

    store.resetSession('alice')
    api.getUserAnnouncements.mockResolvedValue(page([notice('alice-notice')], 50, 40))
    await store.loadList()
    store.resetSession(null)
    expect(store.items).toEqual([])
    expect(store.requiredItems).toEqual([])
    expect(store.unreadCount).toBeNull()
    expect(store.total).toBe(0)
    expect(store.hasMore).toBe(false)
    expect(store.loading).toBe(false)
    expect(store.error).toBeNull()
  })

  it('shares one status fetch across header consumers and uses the global count', async () => {
    const desktop = useAnnouncementStore()
    const mobile = useAnnouncementStore()
    expect(mobile).toBe(desktop)
    desktop.resetSession('alice')
    const count = deferred<{ unread_count: number }>()
    const required = deferred<AnnouncementListResponse>()
    api.getUnreadCount.mockReturnValue(count.promise)
    api.getRequiredUnreadAnnouncements.mockReturnValue(required.promise)

    const first = desktop.refreshStatus()
    const second = mobile.refreshStatus()
    expect(api.getUnreadCount).toHaveBeenCalledTimes(1)
    expect(api.getRequiredUnreadAnnouncements).toHaveBeenCalledTimes(1)
    count.resolve({ unread_count: 87 })
    required.resolve(page([notice('required', { requires_ack: true }), notice('normal')], 2, 87))
    await Promise.all([first, second])
    expect(desktop.unreadCount).toBe(87)
    expect(mobile.requiredItems.map(item => item.id)).toEqual(['required'])
    expect(desktop.items).toEqual([])

    api.getUserAnnouncements.mockResolvedValue(page([notice('first-page-item')], 100, 87))
    await mobile.loadList()
    expect(desktop.unreadCount).toBe(87)
    expect(desktop.items).toHaveLength(1)
    expect(desktop.total).toBe(100)
    expect(desktop.hasMore).toBe(true)
  })

  it('loads further pages using loaded offsets while retaining server totals and unread counts', async () => {
    const store = useAnnouncementStore()
    store.resetSession('alice')
    const items = Array.from({ length: 45 }, (_, index) => notice(`notice-${index}`))
    api.getUserAnnouncements
      .mockResolvedValueOnce(page(items.slice(0, 20), 45, 38))
      .mockResolvedValueOnce(page(items.slice(20, 40), 45, 37))
      .mockResolvedValueOnce(page(items.slice(40), 45, 36))

    await store.loadList()
    expect(store.hasMore).toBe(true)
    await store.loadList(true)
    expect(store.items).toHaveLength(40)
    expect(store.total).toBe(45)
    expect(store.unreadCount).toBe(37)
    await store.loadList(true)
    expect(store.items.map(item => item.id)).toEqual(items.map(item => item.id))
    expect(store.unreadCount).toBe(36)
    expect(store.hasMore).toBe(false)
    await store.loadList(true)
    expect(api.getUserAnnouncements.mock.calls.map(([query]) => query)).toEqual([
      { limit: 20, offset: 0 },
      { limit: 20, offset: 20 },
      { limit: 20, offset: 40 },
    ])
  })

  it('advances by consumed page rows even when later pages repeat announcements', async () => {
    const store = useAnnouncementStore()
    store.resetSession('alice')
    const items = Array.from({ length: 39 }, (_, index) => notice(`notice-${index}`))
    api.getUserAnnouncements
      .mockResolvedValueOnce(page(items.slice(0, 20), 41, 35))
      .mockResolvedValueOnce(page([items[18]!, items[19]!, ...items.slice(20, 38)], 41, 34))
      .mockResolvedValueOnce(page(items.slice(38), 41, 33))

    await store.loadList()
    await store.loadList(true)
    expect(store.items).toHaveLength(38)
    expect(store.hasMore).toBe(true)
    await store.loadList(true)

    expect(api.getUserAnnouncements.mock.calls.map(([query]) => query.offset)).toEqual([0, 20, 40])
    expect(store.items.map(item => item.id)).toEqual(items.map(item => item.id))
    expect(store.total).toBe(41)
    expect(store.unreadCount).toBe(33)
    expect(store.hasMore).toBe(false)
    await store.loadList(true)
    expect(api.getUserAnnouncements).toHaveBeenCalledTimes(3)
  })

  it('retains loaded rows and counts when load-more fails, then retries the same offset', async () => {
    const store = useAnnouncementStore()
    store.resetSession('alice')
    api.getUserAnnouncements.mockResolvedValueOnce(page([notice('first')], 2, 2))
    await store.loadList()
    api.getUserAnnouncements.mockRejectedValueOnce(new Error('unavailable'))
    await store.loadList(true)
    expect(store.items.map(item => item.id)).toEqual(['first'])
    expect(store.total).toBe(2)
    expect(store.unreadCount).toBe(2)
    expect(store.hasMore).toBe(true)
    expect(store.loading).toBe(false)
    expect(store.error).toBeTruthy()
    api.getUserAnnouncements.mockResolvedValueOnce(page([notice('second')], 2, 2))
    await store.loadList(true)
    expect(api.getUserAnnouncements).toHaveBeenLastCalledWith({ limit: 20, offset: 1 })
    expect(store.items.map(item => item.id)).toEqual(['first', 'second'])
    expect(store.error).toBeNull()
  })

  it('retains prior status when polling fails and exposes the failure', async () => {
    const store = useAnnouncementStore()
    store.resetSession('alice')
    api.getUnreadCount.mockResolvedValueOnce({ unread_count: 30 })
    api.getRequiredUnreadAnnouncements.mockResolvedValueOnce(page([notice('required', { requires_ack: true })], 1, 30))
    await store.refreshStatus()
    api.getUnreadCount.mockRejectedValueOnce(new Error('unavailable'))
    api.getRequiredUnreadAnnouncements.mockRejectedValueOnce(new Error('unavailable'))
    await store.refreshStatus()
    expect(store.unreadCount).toBe(30)
    expect(store.requiredItems.map(item => item.id)).toEqual(['required'])
    expect(store.error).toBeTruthy()
  })
})

describe('announcement acknowledgements', () => {
  it('updates list and badge while refilling a required queue larger than twenty', async () => {
    const store = useAnnouncementStore()
    store.resetSession('alice')
    const all = Array.from({ length: 23 }, (_, index) => notice(`required-${index}`, { requires_ack: true }))
    const acknowledged = new Set<string>()
    const unread = () => all.filter(item => !acknowledged.has(item.id))
    api.getUserAnnouncements.mockImplementation(async () => page(all.slice(0, 20), 23, unread().length))
    api.getUnreadCount.mockImplementation(async () => ({ unread_count: unread().length }))
    api.getRequiredUnreadAnnouncements.mockImplementation(async () => page(unread().slice(0, 20), Math.min(unread().length, 20), unread().length))
    api.markAsRead.mockImplementation(async (id: string) => { acknowledged.add(id); return { message: 'read' } })

    await Promise.all([store.loadList(), store.refreshStatus()])
    expect(store.requiredItems).toHaveLength(20)
    for (const [index, item] of all.entries()) {
      expect(store.requiredItems[0]?.id).toBe(item.id)
      await store.markRead(item.id)
      expect(store.unreadCount).toBe(22 - index)
      expect(store.requiredItems.some(required => required.id === item.id)).toBe(false)
    }
    expect(store.requiredItems).toEqual([])
    expect(store.items.every(item => item.is_read)).toBe(true)
    expect(api.markAsRead.mock.calls.map(([id]) => id)).toEqual(all.map(item => item.id))
    expect(api.getRequiredUnreadAnnouncements).toHaveBeenCalledTimes(24)
  })

  it('deduplicates one pending acknowledgement and leaves unread state on failure', async () => {
    const store = useAnnouncementStore()
    store.resetSession('alice')
    const item = notice('required', { requires_ack: true })
    store.items = [item]
    store.requiredItems = [item]
    store.unreadCount = 7
    const write = deferred<{ message: string }>()
    api.markAsRead.mockReturnValueOnce(write.promise)
    const first = store.markRead(item.id)
    const second = store.markRead(item.id)
    const failures = Promise.allSettled([first, second])
    expect(api.markAsRead).toHaveBeenCalledTimes(1)
    expect(store.unreadCount).toBe(7)
    write.reject(new Error('write failed'))
    expect((await failures).every(result => result.status === 'rejected')).toBe(true)
    expect(store.items[0]?.is_read).toBe(false)
    expect(store.requiredItems.map(required => required.id)).toEqual([item.id])
    expect(store.unreadCount).toBe(7)
    expect(api.getUnreadCount).not.toHaveBeenCalled()
  })

  it('does not allow pre-acknowledgement list or status responses to resurrect an unread item', async () => {
    const store = useAnnouncementStore()
    store.resetSession('alice')
    const item = notice('required', { requires_ack: true })
    store.items = [item]
    store.requiredItems = [item]
    store.unreadCount = 1
    const oldList = deferred<AnnouncementListResponse>()
    const oldCount = deferred<{ unread_count: number }>()
    const oldRequired = deferred<AnnouncementListResponse>()
    api.getUserAnnouncements.mockReturnValueOnce(oldList.promise)
    api.getUnreadCount.mockReturnValueOnce(oldCount.promise)
    api.getRequiredUnreadAnnouncements.mockReturnValueOnce(oldRequired.promise)
    const listRequest = store.loadList()
    const statusRequest = store.refreshStatus()

    await store.markRead(item.id)
    expect(store.items[0]?.is_read).toBe(true)
    expect(store.unreadCount).toBe(0)
    oldList.resolve(page([item], 1, 1))
    oldCount.resolve({ unread_count: 1 })
    oldRequired.resolve(page([item], 1, 1))
    await Promise.all([listRequest, statusRequest])
    expect(store.items[0]?.is_read).toBe(true)
    expect(store.requiredItems).toEqual([])
    expect(store.unreadCount).toBe(0)
    expect(store.loading).toBe(false)
  })

  it('only clears all reads after a successful write and preserves state after a rejected write', async () => {
    const store = useAnnouncementStore()
    store.resetSession('alice')
    const item = notice('required', { requires_ack: true })
    store.items = [item]
    store.requiredItems = [item]
    store.unreadCount = 60
    const rejected = deferred<{ message: string }>()
    api.markAllAsRead.mockReturnValueOnce(rejected.promise)
    const failed = expect(store.markAllRead()).rejects.toThrow('write failed')
    await store.markAllRead()
    expect(api.markAllAsRead).toHaveBeenCalledTimes(1)
    expect(store.markingAll).toBe(true)
    expect(store.unreadCount).toBe(60)
    rejected.reject(new Error('write failed'))
    await failed
    expect(store.markingAll).toBe(false)
    expect(store.unreadCount).toBe(60)
    expect(store.items[0]?.is_read).toBe(false)
    expect(store.requiredItems).toHaveLength(1)

    api.getUnreadCount.mockRejectedValueOnce(new Error('refresh failed'))
    api.getRequiredUnreadAnnouncements.mockRejectedValueOnce(new Error('refresh failed'))
    await store.markAllRead()
    expect(store.items[0]?.is_read).toBe(true)
    expect(store.requiredItems).toEqual([])
    expect(store.unreadCount).toBe(0)
    expect(store.markingAll).toBe(false)
    expect(store.error).toBeTruthy()
  })
})

describe('announcement session isolation', () => {
  it('ignores old account list and status responses even after switching back to that account', async () => {
    const store = useAnnouncementStore()
    store.resetSession('alice')
    const oldList = deferred<AnnouncementListResponse>()
    const oldCount = deferred<{ unread_count: number }>()
    const oldRequired = deferred<AnnouncementListResponse>()
    api.getUserAnnouncements.mockReturnValueOnce(oldList.promise)
    api.getUnreadCount.mockReturnValueOnce(oldCount.promise)
    api.getRequiredUnreadAnnouncements.mockReturnValueOnce(oldRequired.promise)
    const listRequest = store.loadList()
    const statusRequest = store.refreshStatus()
    store.resetSession('bob')
    store.resetSession('alice')
    const current = notice('new-session-notice', { requires_ack: true })
    api.getUserAnnouncements.mockResolvedValueOnce(page([current], 1, 1))
    api.getUnreadCount.mockResolvedValueOnce({ unread_count: 1 })
    api.getRequiredUnreadAnnouncements.mockResolvedValueOnce(page([current], 1, 1))
    await Promise.all([store.loadList(), store.refreshStatus()])

    oldList.resolve(page([notice('old-secret')], 99, 90))
    oldCount.resolve({ unread_count: 90 })
    oldRequired.resolve(page([notice('old-required', { requires_ack: true })], 1, 90))
    await Promise.all([listRequest, statusRequest])
    expect(store.items.map(item => item.id)).toEqual([current.id])
    expect(store.requiredItems.map(item => item.id)).toEqual([current.id])
    expect(store.total).toBe(1)
    expect(store.unreadCount).toBe(1)
    expect(store.error).toBeNull()
  })

  it.each(['one', 'all'] as const)('ignores a late %s acknowledgement from a previous session', async operation => {
    const store = useAnnouncementStore()
    store.resetSession('alice')
    const item = notice('same-notice', { requires_ack: true })
    store.items = [item]
    store.requiredItems = [item]
    store.unreadCount = 1
    const write = deferred<{ message: string }>()
    if (operation === 'one') api.markAsRead.mockReturnValueOnce(write.promise)
    else api.markAllAsRead.mockReturnValueOnce(write.promise)
    const request = operation === 'one' ? store.markRead(item.id) : store.markAllRead()
    store.resetSession('bob')
    store.items = [{ ...item }]
    store.requiredItems = [{ ...item }]
    store.unreadCount = 5
    write.resolve({ message: 'read' })
    await request
    expect(store.items[0]?.is_read).toBe(false)
    expect(store.requiredItems.map(required => required.id)).toEqual([item.id])
    expect(store.unreadCount).toBe(5)
    expect(store.markingAll).toBe(false)
    expect(api.getUnreadCount).not.toHaveBeenCalled()
  })
})
