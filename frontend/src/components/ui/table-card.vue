<template>
  <Card class="overflow-hidden">
    <!-- 标题和操作栏 -->
    <div
      v-if="$slots.header || title"
      class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60"
    >
      <slot name="header">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <!-- 左侧：标题 -->
          <div class="min-w-0">
            <h3 class="text-sm sm:text-base font-semibold shrink-0">
              {{ title }}
            </h3>
            <p
              v-if="description"
              class="mt-1 text-xs text-muted-foreground"
            >
              {{ description }}
            </p>
          </div>

          <!-- 右侧：操作区 -->
          <div
            v-if="$slots.actions"
            class="flex flex-wrap items-center gap-2"
          >
            <slot name="actions" />
          </div>
        </div>
      </slot>
    </div>

    <!-- 表格内容 -->
    <slot />

    <!-- 分页 -->
    <slot name="pagination" />
  </Card>
</template>

<script setup lang="ts">
import { Card } from '@/components/ui'

interface Props {
  title?: string
  description?: string
}

defineProps<Props>()
</script>
