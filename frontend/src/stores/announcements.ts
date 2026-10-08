import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { announcementApi, type Announcement } from '@/api/announcements'
import { getI18nLocale } from '@/i18n'

export const useAnnouncementStore = defineStore('announcements', () => {
  const items = ref<Announcement[]>([])
  const requiredItems = ref<Announcement[]>([])
  const unreadCount = ref<number | null>(null)
  const total = ref(0)
  const nextOffset = ref(0)
  const loading = ref(false)
  const markingAll = ref(false)
  const listError = ref<string | null>(null)
  const statusError = ref<string | null>(null)
  const error = computed(() => listError.value || statusError.value)
  const hasMore = computed(() => nextOffset.value < total.value)
  let sessionId: string | null = null
  let sessionVersion = 0
  let revision = 0
  let listRequestId = 0
  let listLoaded = false
  let statusPromise: Promise<void> | null = null
  const reads = new Map<string, Promise<void>>()
  const message = (zh: string, en: string) => getI18nLocale() === 'zh-CN' ? zh : en

  function resetSession(userId: string | null) {
    sessionId = userId
    sessionVersion += 1
    revision += 1
    listRequestId += 1
    statusPromise = null
    reads.clear()
    items.value = []
    requiredItems.value = []
    unreadCount.value = null
    total.value = 0
    nextOffset.value = 0
    loading.value = false
    markingAll.value = false
    listError.value = null
    statusError.value = null
    listLoaded = false
  }

  async function refreshStatus() {
    if (!sessionId) return
    if (statusPromise) return statusPromise
    const version = revision
    const request = (async () => {
      const [count, required] = await Promise.allSettled([
        announcementApi.getUnreadCount(),
        announcementApi.getRequiredUnreadAnnouncements(),
      ])
      if (version !== revision) return
      if (count.status === 'fulfilled') unreadCount.value = count.value.unread_count
      if (required.status === 'fulfilled') {
        requiredItems.value = required.value.items.filter(item => item.requires_ack && !item.is_read)
      }
      statusError.value = count.status === 'rejected' || required.status === 'rejected'
        ? message('公告暂时无法更新', 'Announcements could not be refreshed') : null
    })()
    statusPromise = request
    try {
      await request
    } finally {
      if (statusPromise === request) statusPromise = null
    }
  }

  async function loadList(more = false) {
    if (!sessionId || (more && (loading.value || !hasMore.value))) return
    const requestId = ++listRequestId
    const version = revision
    const offset = more ? nextOffset.value : 0
    loading.value = true
    listError.value = null
    try {
      const response = await announcementApi.getUserAnnouncements({ limit: 20, offset })
      if (requestId !== listRequestId || version !== revision) return
      items.value = more
        ? Array.from(new Map([...items.value, ...response.items].map(item => [item.id, item])).values())
        : response.items
      total.value = response.total
      nextOffset.value = offset + response.items.length
      unreadCount.value = response.unread_count
      listLoaded = true
    } catch {
      if (requestId === listRequestId && version === revision) {
        listError.value = message('公告加载失败，请重试', 'Announcements could not be loaded. Please retry.')
      }
    } finally {
      if (requestId === listRequestId) loading.value = false
    }
  }

  async function refresh() {
    await Promise.all([refreshStatus(), listLoaded ? loadList() : Promise.resolve()])
  }

  // Invalidate reads started before an acknowledgement or account switch.
  function invalidateReads() {
    revision += 1
    listRequestId += 1
    loading.value = false
    statusPromise = null
  }

  async function markRead(id: string) {
    if (!sessionId) return
    const pending = reads.get(id)
    if (pending) return pending
    const owner = sessionVersion
    const request = (async () => {
      await announcementApi.markAsRead(id)
      if (sessionVersion !== owner) return
      invalidateReads()
      const wasUnread = items.value.some(item => item.id === id && item.is_read === false)
        || requiredItems.value.some(item => item.id === id)
      items.value = items.value.map(item => item.id === id ? { ...item, is_read: true } : item)
      requiredItems.value = requiredItems.value.filter(item => item.id !== id)
      if (wasUnread && unreadCount.value !== null) unreadCount.value = Math.max(0, unreadCount.value - 1)
      await refreshStatus()
    })()
    reads.set(id, request)
    try {
      await request
    } finally {
      if (reads.get(id) === request) reads.delete(id)
    }
  }

  async function markAllRead() {
    if (!sessionId || markingAll.value) return
    const owner = sessionVersion
    markingAll.value = true
    try {
      await announcementApi.markAllAsRead()
      if (sessionVersion !== owner) return
      invalidateReads()
      items.value = items.value.map(item => ({ ...item, is_read: true }))
      requiredItems.value = []
      unreadCount.value = 0
      await refreshStatus()
    } finally {
      if (sessionVersion === owner) markingAll.value = false
    }
  }

  return { items, requiredItems, unreadCount, total, loading, markingAll, error, hasMore,
    resetSession, refreshStatus, loadList, refresh, markRead, markAllRead }
})
