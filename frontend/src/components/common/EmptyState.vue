<template>
  <div :class="containerClasses">
    <!-- 图标 -->
    <div :class="iconContainerClasses">
      <component
        :is="icon"
        v-if="icon"
        :class="iconClasses"
      />
      <component
        :is="defaultIcon"
        v-else
        :class="iconClasses"
      />
    </div>

    <!-- 标题 -->
    <h3
      v-if="displayTitle"
      :class="titleClasses"
    >
      {{ displayTitle }}
    </h3>

    <!-- 描述 -->
    <p
      v-if="displayDescription"
      :class="descriptionClasses"
    >
      {{ displayDescription }}
    </p>

    <!-- 自定义内容插槽 -->
    <div
      v-if="$slots.default"
      class="mt-4"
    >
      <slot />
    </div>

    <!-- 操作按钮 -->
    <div
      v-if="$slots.actions || displayActionText"
      class="mt-6 flex flex-wrap items-center justify-center gap-3"
    >
      <slot name="actions">
        <Button
          v-if="displayActionText"
          :variant="actionVariant"
          :size="actionSize"
          @click="handleAction"
        >
          <component
            :is="actionIcon"
            v-if="actionIcon"
            class="mr-2 h-4 w-4"
          />
          {{ displayActionText }}
        </Button>
      </slot>
    </div>

    <!-- 次要操作 -->
    <div
      v-if="$slots.secondary"
      class="mt-3"
    >
      <slot name="secondary" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import Button from '@/components/ui/button.vue'
import {
  FileQuestion,
  Search,
  Inbox,
  AlertCircle,
  PackageOpen,
  Filter
} from 'lucide-vue-next'
import type { Component } from 'vue'
import { useI18n } from '@/i18n'

type EmptyStateType = 'default' | 'search' | 'filter' | 'error' | 'empty' | 'notFound'
type ButtonVariant = 'default' | 'outline' | 'secondary' | 'ghost' | 'link' | 'destructive'
type ButtonSize = 'sm' | 'default' | 'lg' | 'icon'

interface Props {
  /** 空状态类型 */
  type?: EmptyStateType
  /** 自定义图标组件 */
  icon?: Component
  /** 标题 */
  title?: string
  /** 描述文本 */
  description?: string
  /** 操作按钮文本 */
  actionText?: string
  /** 操作按钮图标 */
  actionIcon?: Component
  /** 操作按钮变体 */
  actionVariant?: ButtonVariant
  /** 操作按钮大小 */
  actionSize?: ButtonSize
  /** 大小 */
  size?: 'sm' | 'md' | 'lg'
  /** 对齐方式 */
  align?: 'left' | 'center' | 'right'
}

interface Emits {
  (e: 'action'): void
}

const props = withDefaults(defineProps<Props>(), {
  type: 'default',
  icon: undefined,
  title: undefined,
  description: undefined,
  actionText: undefined,
  actionIcon: undefined,
  actionVariant: 'default',
  actionSize: 'default',
  size: 'md',
  align: 'center'
})

const emit = defineEmits<Emits>()
const { t } = useI18n()

// 根据类型获取默认配置
const typeConfig = computed(() => {
  const configs = {
    default: {
      icon: Inbox,
      title: t('stats.empty.title'),
      description: t('ui.thereIsNothingToDisplayRightNow')
    },
    search: {
      icon: Search,
      title: t('ui.noResultsFound'),
      description: t('ui.tryADifferentKeyword')
    },
    filter: {
      icon: Filter,
      title: t('ui.noMatchingResults'),
      description: t('ui.noDataMatchesTheCurrentFilters')
    },
    error: {
      icon: AlertCircle,
      title: t('ui.failedToLoad'),
      description: t('ui.anErrorOccurredWhileLoadingData')
    },
    empty: {
      icon: PackageOpen,
      title: t('ui.nothingHereYet'),
      description: t('ui.thereIsNoContentYet')
    },
    notFound: {
      icon: FileQuestion,
      title: t('ui.notFound'),
      description: t('ui.theRequestedResourceDoesNotExist')
    }
  }

  return configs[props.type]
})

// 默认图标
const defaultIcon = computed(() => typeConfig.value.icon)
const displayTitle = computed(() => (props.title ?? typeConfig.value.title))
const displayDescription = computed(() => (props.description ?? typeConfig.value.description))
const displayActionText = computed(() => props.actionText ? props.actionText : '')

// 容器样式
const containerClasses = computed(() => {
  const classes = ['empty-state']

  // 大小
  if (props.size === 'sm') {
    classes.push('empty-state-sm', 'py-6')
  } else if (props.size === 'lg') {
    classes.push('empty-state-lg', 'py-16')
  } else {
    classes.push('empty-state-md', 'py-12')
  }

  // 对齐
  if (props.align === 'left') {
    classes.push('text-left')
  } else if (props.align === 'right') {
    classes.push('text-right')
  } else {
    classes.push('text-center')
  }

  return classes.join(' ')
})

// 图标容器样式
const iconContainerClasses = computed(() => {
  const classes = [
    'empty-state-icon-container',
    'rounded-full',
    'inline-flex',
    'items-center',
    'justify-center',
    'mb-4'
  ]

  // 大小和颜色
  if (props.type === 'error') {
    classes.push('bg-red-100', 'dark:bg-red-900/30')
  } else if (props.type === 'search' || props.type === 'filter') {
    classes.push('bg-blue-100', 'dark:bg-blue-900/30')
  } else {
    classes.push('bg-muted')
  }

  // 尺寸
  if (props.size === 'sm') {
    classes.push('w-12', 'h-12')
  } else if (props.size === 'lg') {
    classes.push('w-20', 'h-20')
  } else {
    classes.push('w-16', 'h-16')
  }

  return classes.join(' ')
})

// 图标样式
const iconClasses = computed(() => {
  const classes = []

  // 颜色
  if (props.type === 'error') {
    classes.push('text-red-600', 'dark:text-red-400')
  } else if (props.type === 'search' || props.type === 'filter') {
    classes.push('text-blue-600', 'dark:text-blue-400')
  } else {
    classes.push('text-muted-foreground')
  }

  // 尺寸
  if (props.size === 'sm') {
    classes.push('w-6', 'h-6')
  } else if (props.size === 'lg') {
    classes.push('w-10', 'h-10')
  } else {
    classes.push('w-8', 'h-8')
  }

  return classes.join(' ')
})

// 标题样式
const titleClasses = computed(() => {
  const classes = ['font-semibold', 'text-foreground', 'mb-2']

  if (props.size === 'sm') {
    classes.push('text-base')
  } else if (props.size === 'lg') {
    classes.push('text-2xl')
  } else {
    classes.push('text-lg')
  }

  return classes.join(' ')
})

// 描述样式
const descriptionClasses = computed(() => {
  const classes = ['text-muted-foreground', 'max-w-md']

  if (props.align === 'center') {
    classes.push('mx-auto')
  }

  if (props.size === 'sm') {
    classes.push('text-xs')
  } else if (props.size === 'lg') {
    classes.push('text-base')
  } else {
    classes.push('text-sm')
  }

  return classes.join(' ')
})

// 处理操作
function handleAction() {
  emit('action')
}
</script>

<style scoped>
.empty-state {
  @apply flex flex-col items-center justify-center;
}

.empty-state-icon-container {
  @apply transition-transform duration-200;
}

.empty-state:hover .empty-state-icon-container {
  @apply scale-105;
}
</style>
