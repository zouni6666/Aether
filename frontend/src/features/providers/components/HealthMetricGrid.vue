<template>
  <div class="grid w-full grid-cols-2 gap-2 sm:grid-cols-4">
    <div
      v-for="metric in metrics"
      :key="metric.label"
      class="rounded-lg border border-border/40 bg-muted/20 px-3 py-2"
    >
      <div class="text-[11px] leading-tight text-muted-foreground">
        {{ metric.label }}
      </div>
      <div
        class="mt-1 text-sm font-semibold tabular-nums"
        :class="metric.valueClass"
      >
        {{ metric.value }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { translate as t } from '@/i18n'

import { computed } from 'vue'
import {
  formatAvailability,
  formatMs,
  formatTps,
  getAvailabilityClass
} from './health-monitor-utils'

const props = defineProps<{
  avgLatencyMs?: number | null
  avgFirstByteMs?: number | null
  avgTps?: number | null
  totalAttempts: number
  successRate: number
}>()

const availabilityItem = computed(() => ({
  total_attempts: props.totalAttempts,
  success_rate: props.successRate
}))

const metrics = computed(() => [
  {
    label: t('ui.averageDuration'),
    value: formatMs(props.avgLatencyMs),
    valueClass: ''
  },
  {
    label: t('ui.averageTtfb'),
    value: formatMs(props.avgFirstByteMs),
    valueClass: ''
  },
  {
    label: t('ui.averageSpeed'),
    value: formatTps(props.avgTps),
    valueClass: ''
  },
  {
    label: t('ui.availability'),
    value: formatAvailability(availabilityItem.value),
    valueClass: getAvailabilityClass(availabilityItem.value)
  }
])
</script>
