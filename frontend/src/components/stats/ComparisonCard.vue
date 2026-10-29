<template>
  <Card class="p-4 space-y-2">
    <div class="text-xs text-muted-foreground">
      {{ label }}
    </div>
    <div class="text-lg font-semibold">
      {{ value }}
    </div>
    <div
      class="text-xs"
      :class="changeClass"
    >
      <span v-if="changePercent !== null">{{ changePercent }}%</span>
      <span v-else>--</span>
      <span class="ml-1 text-muted-foreground">{{ t('ui.vsComparisonPeriod') }}</span>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed } from 'vue'
import { Card } from '@/components/ui'

interface Props {
  label: string
  value: string
  changePercent: number | null
}

const props = defineProps<Props>()

const { t } = useI18n()

const changeClass = computed(() => {
  if (props.changePercent === null) return 'text-muted-foreground'
  if (props.changePercent > 0) return 'text-emerald-500'
  if (props.changePercent < 0) return 'text-rose-500'
  return 'text-muted-foreground'
})

</script>
