<template>
  <Dialog
    :model-value="modelValue"
    :persistent="persistent"
    size="xl"
    no-padding
    @update:model-value="emit('update:modelValue', $event)"
  >
    <template #header>
      <div class="shrink-0 bg-card px-4 pt-4 sm:px-6 sm:pt-5">
        <div class="flex items-center justify-between gap-3">
          <div class="flex items-center gap-2 text-xs text-muted-foreground">
            <Bell class="h-3.5 w-3.5 text-primary" />
            <span>{{ text('系统公告', 'Announcement') }}</span>
          </div>
          <button
            v-if="!persistent"
            type="button"
            class="-mr-1 flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-muted-foreground transition hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            :aria-label="text('关闭', 'Close')"
            :title="text('关闭', 'Close')"
            @click="emit('update:modelValue', false)"
          >
            <X class="h-4 w-4" />
          </button>
        </div>
      </div>
    </template>

    <div class="bg-card px-4 pb-5 pt-3 sm:px-6">
      <header class="mb-5 border-b border-border pb-4">
        <h2
          class="text-lg font-semibold leading-7 tracking-normal text-foreground [overflow-wrap:anywhere]"
          translate="no"
        >
          {{ announcement?.title }}
        </h2>
        <div
          v-if="announcement"
          class="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs leading-5 text-muted-foreground"
        >
          <span
            class="min-w-0 [overflow-wrap:anywhere]"
            translate="no"
          >{{ announcement.author.username }}</span>
          <time :datetime="announcement.created_at">{{ publishedAt }}</time>
        </div>
      </header>
      <!-- eslint-disable vue/no-v-html -->
      <article
        v-if="announcement"
        class="announcement-body min-w-0 text-sm text-foreground"
        translate="no"
        v-html="content"
      />
      <!-- eslint-enable vue/no-v-html -->
    </div>

    <template #footer>
      <div class="w-full space-y-3">
        <p
          v-if="error"
          role="alert"
          class="text-sm text-destructive [overflow-wrap:anywhere]"
        >
          {{ error }}
        </p>
        <div class="flex flex-wrap items-center justify-end gap-2">
          <Button
            v-if="!persistent"
            variant="outline"
            size="sm"
            class="border-border bg-card font-medium"
            @click="emit('update:modelValue', false)"
          >
            {{ text('关闭', 'Close') }}
          </Button>
          <Button
            v-if="confirmRead"
            size="sm"
            class="gap-2 font-medium text-primary-foreground"
            :disabled="confirming"
            @click="emit('confirm')"
          >
            <Loader2
              v-if="confirming"
              class="h-4 w-4 animate-spin"
            />
            <Check
              v-else
              class="h-4 w-4"
            />
            {{ confirming ? t('common.confirming') : t('common.confirmRead') }}
          </Button>
        </div>
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Bell, Check, Loader2, X } from 'lucide-vue-next'
import { marked } from 'marked'
import type { Announcement } from '@/api/announcements'
import { Button, Dialog } from '@/components/ui'
import { useI18n } from '@/i18n'
import { sanitizeMarkdown } from '@/utils/sanitize'

const props = defineProps<{
  modelValue: boolean
  announcement: Announcement | null
  persistent?: boolean
  confirmRead?: boolean
  confirming?: boolean
  error?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [open: boolean]
  confirm: []
}>()

const { locale, t } = useI18n()
const text = (zh: string, en: string) => locale.value === 'zh-CN' ? zh : en
const content = computed(() => sanitizeMarkdown(marked.parse(props.announcement?.content || '', { async: false })))
const publishedAt = computed(() => {
  const date = new Date(props.announcement?.created_at || '')
  return Number.isNaN(date.getTime()) ? '-' : date.toLocaleString(locale.value, {
    year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit',
  })
})
</script>

<style scoped>
.announcement-body {
  font-family: var(--sans-serif);
  line-height: 1.8;
  overflow-wrap: anywhere;
}

.announcement-body :deep(*) {
  letter-spacing: 0;
}

.announcement-body :deep(p) {
  font-family: inherit;
  line-height: inherit;
}

.announcement-body :deep(:is(p, ul, ol, blockquote, pre, table, hr)) {
  margin-block: 1em;
}

.announcement-body :deep(:is(h1, h2, h3, h4, h5, h6)) {
  margin-block: 1.5em 0.5em;
  font-weight: 600;
  line-height: 1.5;
  color: var(--foreground);
}

.announcement-body :deep(:is(h1, h2)) {
  font-size: 1rem;
}

.announcement-body :deep(:is(h3, h4, h5, h6)) {
  font-size: 0.875rem;
}

.announcement-body :deep(ul) {
  list-style: disc;
  padding-left: 1.5em;
}

.announcement-body :deep(ol) {
  list-style: decimal;
  padding-left: 1.5em;
}

.announcement-body :deep(li + li) {
  margin-top: 0.25em;
}

.announcement-body :deep(a) {
  color: var(--primary);
  text-decoration: underline;
  text-underline-offset: 3px;
}

.announcement-body :deep(blockquote) {
  border-left: 2px solid var(--border);
  padding-left: 1em;
  color: var(--muted-foreground);
}

.announcement-body :deep(:is(code, pre)) {
  border-radius: 4px;
  background: var(--muted);
  color: var(--foreground);
  font-family: var(--monospace);
  font-size: 0.8125rem;
}

.announcement-body :deep(code) {
  padding: 0.125em 0.3em;
}

.announcement-body :deep(pre) {
  max-width: 100%;
  overflow-x: auto;
  padding: 0.75rem 1rem;
}

.announcement-body :deep(pre code) {
  padding: 0;
}

.announcement-body :deep(table) {
  display: block;
  max-width: 100%;
  overflow-x: auto;
  border-collapse: collapse;
}

.announcement-body :deep(:is(th, td)) {
  min-width: 6rem;
  border: 1px solid var(--border);
  padding: 0.5em 0.75em;
  text-align: left;
}

.announcement-body :deep(th) {
  background: var(--muted);
  font-weight: 500;
}

.announcement-body :deep(img) {
  max-width: 100%;
  height: auto;
}

.announcement-body :deep(> :first-child) {
  margin-top: 0;
}

.announcement-body :deep(> :last-child) {
  margin-bottom: 0;
}
</style>
