<template>
  <div class="flex flex-col items-center justify-center py-16 text-center">
    <div class="mb-2 text-muted-foreground">
      {{ hasActiveFilters ? t('ui.noProvidersMatchTheCurrentFilters') : (title || t('ui.noProvidersYetAddOneFromTheTopRight')) }}
    </div>
    <p
      v-if="!hasActiveFilters && description"
      class="px-4 text-xs text-muted-foreground"
    >
      {{ description }}
    </p>
    <Button
      v-if="hasActiveFilters"
      variant="outline"
      size="sm"
      @click="$emit('resetFilters')"
    >
      {{ t('ui.clearFilters') }}
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

const { t } = useI18n()

</script>
