<template>
  <aside
    id="announcements-section"
    class="flex min-h-0 min-w-0 flex-col"
    :aria-label="text('系统公告', 'System announcements')"
  >
    <div class="mb-4 flex h-5 shrink-0 items-center justify-between gap-2">
      <h3 class="text-sm font-medium text-foreground">
        {{ text('系统公告', 'System announcements') }}
      </h3>
      <RouterLink
        :to="authStore.canAccessAdmin ? '/admin/announcements' : '/dashboard/announcements'"
        class="shrink-0 text-[11px] text-muted-foreground transition hover:text-foreground"
      >
        {{ text('查看全部', 'View all') }}
      </RouterLink>
    </div>
    <Card class="flex min-h-0 flex-1 flex-col overflow-hidden p-0">
      <div
        v-if="displayItems.length"
        class="min-h-0 max-h-72 flex-1 overflow-y-auto overscroll-contain min-[1440px]:max-h-none"
        :aria-busy="announcementStore.loading"
      >
        <ol class="px-4 py-1">
          <li
            v-for="item in displayItems"
            :key="item.announcement.id"
            class="relative border-l border-border/60 py-3 pl-4 last:border-transparent"
          >
            <span
              aria-hidden="true"
              class="absolute -left-[3.5px] top-[18px] h-1.5 w-1.5 rounded-full ring-4 ring-card"
              :class="item.announcement.is_read ? 'bg-border' : 'bg-primary'"
            />
            <button
              type="button"
              class="block w-full min-w-0 text-left transition hover:text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-4"
              :aria-label="`${item.announcement.title}, ${item.announcement.is_read ? text('已读', 'Read') : text('未读', 'Unread')}`"
              @click="openAnnouncement?.(item.announcement)"
            >
              <time
                :datetime="item.announcement.created_at"
                class="block text-[10px] leading-4 text-muted-foreground"
              >{{ formatTime(item.announcement.created_at) }}</time>
              <span
                translate="no"
                class="mt-1 block truncate text-xs font-medium leading-5"
                :class="item.announcement.is_read ? 'text-muted-foreground' : 'text-foreground'"
              >{{ item.announcement.title }}</span>
              <span
                translate="no"
                class="mt-0.5 line-clamp-2 text-[11px] leading-[18px] text-muted-foreground [overflow-wrap:anywhere]"
              >{{ item.summary }}</span>
            </button>
          </li>
        </ol>
      </div>
      <div
        v-else-if="announcementStore.loading"
        role="status"
        class="flex min-h-28 flex-1 items-center justify-center gap-2 px-4 py-6 text-xs text-muted-foreground"
      >
        <Loader2 class="h-4 w-4 animate-spin" />
        {{ text('加载中', 'Loading') }}
      </div>
      <div
        v-else
        class="flex min-h-28 flex-1 flex-col items-center justify-center gap-2 px-4 py-6 text-xs text-muted-foreground"
      >
        <p>{{ announcementStore.error ? text('公告暂时无法加载', 'Announcements are unavailable') : text('暂无公告', 'No announcements') }}</p>
        <button
          v-if="announcementStore.error"
          type="button"
          class="underline underline-offset-4 hover:text-foreground"
          @click="announcementStore.loadList()"
        >
          {{ text('重新加载', 'Reload') }}
        </button>
      </div>
    </Card>
  </aside>
</template>

<script setup lang="ts">
import { computed, inject, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { Loader2 } from 'lucide-vue-next'
import { marked } from 'marked'
import { Card } from '@/components/ui'
import { openAnnouncementKey } from '@/components/common/announcementContext'
import { useAuthStore } from '@/stores/auth'
import { useAnnouncementStore } from '@/stores/announcements'
import { useI18n } from '@/i18n'
import { sanitizeMarkdown } from '@/utils/sanitize'

const authStore = useAuthStore()
const announcementStore = useAnnouncementStore()
const openAnnouncement = inject(openAnnouncementKey, undefined)
const { locale } = useI18n()
const text = (zh: string, en: string) => locale.value === 'zh-CN' ? zh : en
const displayItems = computed(() => announcementStore.items.map(announcement => {
  const template = document.createElement('template')
  template.innerHTML = sanitizeMarkdown(marked.parse(announcement.content, { async: false }))
  return { announcement, summary: (template.content.textContent || '').replace(/\s+/g, ' ').trim().slice(0, 160) }
}))

function formatTime(value: string) {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? '—' : date.toLocaleString(locale.value, {
    month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
  })
}

onMounted(() => {
  void announcementStore.loadList()
})
</script>
