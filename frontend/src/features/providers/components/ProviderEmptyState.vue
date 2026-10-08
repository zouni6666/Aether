<template>
  <div class="flex flex-col items-center justify-center py-16 text-center">
    <div class="mb-2 text-muted-foreground">
      {{ legacyT(hasActiveFilters ? '未找到匹配当前筛选条件的提供商' : title || '暂无提供商，点击右上角添加') }}
    </div>
    <p
      v-if="!hasActiveFilters && description"
      class="px-4 text-xs text-muted-foreground"
    >
      {{ legacyT(description) }}
    </p>
    <Button
      v-if="hasActiveFilters"
      variant="outline"
      size="sm"
      @click="$emit('resetFilters')"
    >
      {{ legacyT('清除筛选') }}
    </Button>
  </div>
</template>

<script setup lang="ts">
import Button from '@/components/ui/button.vue'
import { useI18n } from '@/i18n'

defineProps<{
  hasActiveFilters: boolean
  title?: string
  description?: string
}>()

defineEmits<{
  resetFilters: []
}>()

const { legacyT } = useI18n()
</script>
