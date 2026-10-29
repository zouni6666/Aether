<template>
  <div
    role="alert"
    class="flex items-start gap-4 px-6 py-3 rounded-lg border shadow-sm max-w-md"
    :class="variantClasses"
  >
    <!-- 图标带圆形进度环 -->
    <div class="relative shrink-0 w-8 h-8">
      <!-- 进度环背景 -->
      <svg
        v-if="toast.duration && toast.duration > 0"
        class="absolute inset-0 w-8 h-8 -rotate-90"
      >
        <circle
          cx="16"
          cy="16"
          r="14"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          class="opacity-15"
        />
        <circle
          cx="16"
          cy="16"
          r="14"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          :stroke-dasharray="circumference"
          :stroke-dashoffset="strokeDashoffset"
          stroke-linecap="round"
          class="transition-[stroke-dashoffset] duration-75"
          :class="progressColorClass"
        />
      </svg>
      <!-- 图标 -->
      <div
        class="absolute inset-0 flex items-center justify-center"
        :class="iconClasses"
      >
        <component
          :is="icon"
          class="w-4 h-4"
        />
      </div>
    </div>

    <!-- 内容 -->
    <div class="flex-1 min-w-0">
      <p
        v-if="toast.title"
        class="text-sm font-medium"
        :class="titleClasses"
      >
        {{ toast.title }}
      </p>
      <p
        v-if="toast.message"
        class="text-sm break-words"
        :class="messageClasses"
      >
        {{ toast.message }}
      </p>
    </div>

    <!-- 关闭按钮 -->
    <button
      class="shrink-0 p-1 rounded transition-colors opacity-40 hover:opacity-100"
      :class="closeClasses"
      type="button"
      :aria-label="t('ui.close')"
      @click="$emit('remove')"
    >
      <X class="w-3.5 h-3.5" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { ref, computed, onMounted, onUnmounted } from 'vue'
import { CheckCircle2, XCircle, AlertTriangle, Info, X } from 'lucide-vue-next'

interface Toast {
  id: string
  title?: string
  message?: string
  variant?: 'success' | 'error' | 'warning' | 'info'
  duration?: number
}

const props = defineProps<{
  toast: Toast
}>()

const emit = defineEmits<{
  remove: []
}>()

const progress = ref(100)
let startTime = 0
let rafId: number | null = null
let timeoutId: ReturnType<typeof setTimeout> | null = null

// 圆形进度环参数
const circumference = 2 * Math.PI * 14 // r=14

const strokeDashoffset = computed(() => {
  return circumference * (1 - progress.value / 100)
})

const updateProgress = () => {
  if (!props.toast.duration || props.toast.duration <= 0) return

  const elapsed = Date.now() - startTime
  const remaining = Math.max(0, 100 - (elapsed / props.toast.duration) * 100)

  progress.value = remaining

  if (remaining <= 0) {
    emit('remove')
  } else {
    rafId = requestAnimationFrame(updateProgress)
  }
}

onMounted(() => {
  if (props.toast.duration && props.toast.duration > 0) {
    startTime = Date.now()
    rafId = requestAnimationFrame(updateProgress)
    // 保底 timeout，确保即使在后台也能移除
    timeoutId = setTimeout(() => {
      emit('remove')
    }, props.toast.duration + 100)
  }
})

onUnmounted(() => {
  if (rafId) cancelAnimationFrame(rafId)
  if (timeoutId) clearTimeout(timeoutId)
})

const icons = {
  success: CheckCircle2,
  error: XCircle,
  warning: AlertTriangle,
  info: Info
}

const icon = computed(() => icons[props.toast.variant || 'info'])

const variantClasses = computed(() => {
  const variant = props.toast.variant || 'info'
  const classes: Record<string, string> = {
    success: 'border-[#5F8D4E]/30 bg-white dark:bg-[var(--slate-dark)]',
    error: 'border-[var(--error)]/30 bg-white dark:bg-[var(--slate-dark)]',
    warning: 'border-[var(--book-cloth)]/30 bg-white dark:bg-[var(--slate-dark)]',
    info: 'border-[var(--slate-medium)]/20 bg-white dark:bg-[var(--slate-dark)]'
  }
  return classes[variant]
})

const iconClasses = computed(() => {
  const variant = props.toast.variant || 'info'
  const classes: Record<string, string> = {
    success: 'text-[#5F8D4E]',
    error: 'text-[var(--error)]',
    warning: 'text-[var(--book-cloth)]',
    info: 'text-[var(--slate-medium)] dark:text-[var(--cloud-medium)]'
  }
  return classes[variant]
})

const progressColorClass = computed(() => {
  const variant = props.toast.variant || 'info'
  const classes: Record<string, string> = {
    success: 'stroke-[#5F8D4E]',
    error: 'stroke-[var(--error)]',
    warning: 'stroke-[var(--book-cloth)]',
    info: 'stroke-[var(--slate-medium)]'
  }
  return classes[variant]
})

const titleClasses = computed(() => {
  return 'text-[var(--color-text)]'
})

const messageClasses = computed(() => {
  return 'text-[var(--slate-medium)] dark:text-[var(--cloud-medium)]'
})

const closeClasses = computed(() => {
  return 'text-[var(--slate-medium)] hover:bg-[var(--color-border-soft)] dark:text-[var(--cloud-medium)]'
})

const { t } = useI18n()
</script>

<style scoped>
</style>
