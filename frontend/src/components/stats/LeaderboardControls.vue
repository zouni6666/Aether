<template>
  <div class="flex flex-wrap items-center gap-2">
    <TimeRangePicker
      :model-value="timeRange"
      :show-granularity="false"
      :preset-options="timeRangePresetOptions"
      @update:model-value="$emit('update:timeRange', $event)"
    />

    <Select
      :model-value="metric"
      @update:model-value="emitMetric"
    >
      <SelectTrigger class="h-8 text-xs w-28">
        <SelectValue :placeholder="t('stats.metric.placeholder')" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="requests">
          {{ t('stats.metric.requests') }}
        </SelectItem>
        <SelectItem value="tokens">
          Tokens
        </SelectItem>
        <SelectItem value="cost">
          {{ t('stats.metric.cost') }}
        </SelectItem>
      </SelectContent>
    </Select>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { TimeRangePicker } from '@/components/common'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui'
import type { DateRangeParams } from '@/features/usage/types'

type LeaderboardMetric = 'requests' | 'tokens' | 'cost'
type LeaderboardTimeRangePreset = 'today' | 'yesterday' | 'last7days' | 'last30days' | 'last90days' | 'custom'

defineProps<{
  metric: LeaderboardMetric
  timeRange: DateRangeParams
}>()

const emit = defineEmits<{
  (e: 'update:metric', value: LeaderboardMetric): void
  (e: 'update:timeRange', value: DateRangeParams): void
}>()

const { t } = useI18n()

const timeRangePresetOptions: LeaderboardTimeRangePreset[] = [
  'today',
  'yesterday',
  'last7days',
  'last30days',
  'last90days',
  'custom'
]

function emitMetric(value: string) {
  if (value === 'requests' || value === 'tokens' || value === 'cost') {
    emit('update:metric', value)
  }
}

</script>
