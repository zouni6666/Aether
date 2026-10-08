<template>
  <Popover
    :open="open"
    @update:open="emit('update:open', $event)"
  >
    <PopoverTrigger as-child>
      <button
        type="button"
        class="relative flex h-9 w-9 shrink-0 items-center justify-center rounded-lg transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        :class="open ? 'bg-muted text-foreground' : 'text-muted-foreground hover:bg-muted hover:text-foreground'"
        :title="bellLabel"
        :aria-label="bellLabel"
      >
        <Bell class="h-4 w-4" />
        <span
          v-if="unreadCount !== null && unreadCount > 0"
          data-unread-badge
          aria-hidden="true"
          class="absolute -right-1 -top-1 flex h-4 min-w-4 items-center justify-center rounded-full bg-primary px-1 text-[10px] font-medium leading-none text-primary-foreground ring-2 ring-background"
        >{{ unreadCount > 99 ? '99+' : unreadCount }}</span>
      </button>
    </PopoverTrigger>
    <PopoverContent
      align="end"
      :side-offset="8"
      :collision-padding="8"
      :aria-label="text('公告', 'Announcements')"
      class="w-[22rem] max-w-[calc(100vw-1rem)] overflow-hidden rounded-lg border-border bg-card p-0 text-card-foreground shadow-xl shadow-black/10"
    >
      <div class="flex min-w-0 items-center justify-between gap-2 border-b border-border bg-[color-mix(in_srgb,var(--muted)_45%,var(--card))] px-3 py-2">
        <h2 class="min-w-0 text-sm font-semibold">
          {{ text('公告', 'Announcements') }}
        </h2>
        <TooltipProvider :delay-duration="150">
          <div class="flex shrink-0 items-center gap-0.5">
            <Tooltip
              v-for="action in actions"
              :key="action.name"
            >
              <TooltipTrigger as-child>
                <button
                  type="button"
                  class="flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground transition hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-40"
                  :title="action.label"
                  :aria-label="action.label"
                  :disabled="action.disabled"
                  @click="action.run"
                >
                  <component
                    :is="action.icon"
                    class="h-4 w-4"
                    :class="{ 'animate-spin': action.spinning }"
                  />
                </button>
              </TooltipTrigger>
              <TooltipContent>{{ action.label }}</TooltipContent>
            </Tooltip>
          </div>
        </TooltipProvider>
      </div>

      <div
        v-if="error"
        role="alert"
        class="flex items-start justify-between gap-3 border-b border-border bg-[color-mix(in_srgb,var(--destructive)_6%,var(--card))] px-3 py-2 text-xs"
      >
        <span class="min-w-0 break-words text-destructive">{{ error }}</span>
        <button
          type="button"
          class="shrink-0 font-medium text-foreground underline underline-offset-2 disabled:opacity-40"
          :disabled="loading"
          @click="emit('refresh')"
        >
          {{ text('重试', 'Retry') }}
        </button>
      </div>

      <div
        class="max-h-[min(28rem,65dvh)] overflow-y-auto overscroll-contain"
        :aria-busy="loading"
      >
        <ul
          v-if="items.length"
          class="divide-y divide-border"
        >
          <li
            v-for="item in displayItems"
            :key="item.announcement.id"
          >
            <button
              type="button"
              class="flex w-full min-w-0 gap-2.5 px-3 py-3 text-left transition hover:bg-accent hover:text-accent-foreground focus-visible:bg-accent focus-visible:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring"
              :class="item.announcement.is_read ? 'text-muted-foreground' : 'bg-[color-mix(in_srgb,var(--primary)_5%,var(--card))] text-card-foreground'"
              :aria-label="`${item.announcement.title}, ${item.announcement.is_read ? text('已读', 'Read') : text('未读', 'Unread')}`"
              @click="emit('select', item.announcement)"
            >
              <span
                aria-hidden="true"
                class="mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full"
                :class="item.announcement.is_read ? 'bg-transparent' : 'bg-primary'"
              />
              <span class="min-w-0 flex-1 space-y-1">
                <span
                  translate="no"
                  class="block break-words text-sm leading-5 [overflow-wrap:anywhere]"
                  :class="{ 'font-medium': !item.announcement.is_read }"
                >{{ item.announcement.title }}</span>
                <span
                  translate="no"
                  class="line-clamp-2 break-words text-xs leading-5 text-muted-foreground [overflow-wrap:anywhere]"
                >{{ item.summary }}</span>
                <time
                  :datetime="item.announcement.created_at"
                  class="block text-[11px] text-muted-foreground"
                >{{ formatTime(item.announcement.created_at) }}</time>
              </span>
            </button>
          </li>
        </ul>
        <div
          v-else-if="!loading && !error"
          class="px-4 py-10 text-center text-sm text-muted-foreground"
        >
          {{ text('暂无公告', 'No announcements') }}
        </div>
        <div
          v-if="loading"
          role="status"
          class="flex items-center justify-center gap-2 px-4 py-6 text-xs text-muted-foreground"
        >
          <Loader2 class="h-4 w-4 animate-spin" />
          {{ text('加载中', 'Loading') }}
        </div>
      </div>
      <button
        v-if="hasMore"
        type="button"
        class="flex min-h-10 w-full items-center justify-center gap-1 border-t border-border px-3 py-2 text-xs font-medium text-muted-foreground transition hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring disabled:opacity-40"
        :disabled="loading"
        @click="emit('loadMore')"
      >
        {{ text('加载更多', 'Load more') }}
        <ChevronDown class="h-3.5 w-3.5" />
      </button>
    </PopoverContent>
  </Popover>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Bell, CheckCheck, ChevronDown, Loader2, Plus, RefreshCw } from 'lucide-vue-next'
import { marked } from 'marked'
import type { Announcement } from '@/api/announcements'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { getI18nLocale } from '@/i18n'
import { sanitizeMarkdown } from '@/utils/sanitize'

const props = defineProps<{
  open: boolean
  items: Announcement[]
  unreadCount: number | null
  loading: boolean
  error: string | null
  hasMore: boolean
  markingAll: boolean
  canManage: boolean
}>()

const emit = defineEmits<{
  'update:open': [open: boolean]
  select: [announcement: Announcement]
  refresh: []
  loadMore: []
  readAll: []
  create: []
}>()

const text = (zh: string, en: string) => getI18nLocale() === 'zh-CN' ? zh : en
const bellLabel = computed(() => props.unreadCount !== null && props.unreadCount > 0
  ? text(`公告，${props.unreadCount} 条未读`, `Announcements, ${props.unreadCount} unread`)
  : text('公告', 'Announcements'))
const actions = computed(() => [
  { name: 'refresh', label: text('刷新公告', 'Refresh announcements'), icon: RefreshCw, disabled: props.loading, spinning: props.loading, run: () => emit('refresh') },
  { name: 'readAll', label: text('全部标为已读', 'Mark all as read'), icon: props.markingAll ? Loader2 : CheckCheck, disabled: props.markingAll || props.unreadCount === null || props.unreadCount === 0, spinning: props.markingAll, run: () => emit('readAll') },
  ...(props.canManage ? [
    { name: 'create', label: text('发布公告', 'Publish announcement'), icon: Plus, disabled: false, spinning: false, run: () => emit('create') },
  ] : []),
])
const displayItems = computed(() => props.items.map(announcement => {
  const template = document.createElement('template')
  template.innerHTML = sanitizeMarkdown(marked.parse(announcement.content, { async: false }))
  return { announcement, summary: (template.content.textContent || '').replace(/\s+/g, ' ').trim().slice(0, 160) }
}))

function formatTime(value: string) {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? '-' : date.toLocaleString(getI18nLocale(), {
    month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
  })
}
</script>
