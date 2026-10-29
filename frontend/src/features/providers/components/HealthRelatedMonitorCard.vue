<template>
  <HealthMonitorItemCard
    :icon="iconComponent"
    :title="monitor.display_name || monitor.key"
    :subtitle="monitor.meta_text"
    :meta-text="metaText"
    :monitor="monitor"
    :lookback-hours="lookbackHours"
    :generated-at="generatedAt"
    :entity-label="entityLabel"
    :entity-name="monitor.key"
    :show-detail-button="showDetailButton"
    @view-details="$emit('viewDetails', monitor)"
  />
</template>

<script setup lang="ts">
import { translate as t } from '@/i18n'

import { computed } from 'vue'
import { Activity, Bot, Server } from 'lucide-vue-next'
import type { HealthRelatedMonitor } from '@/api/endpoints/types'
import HealthMonitorItemCard from './HealthMonitorItemCard.vue'
import { formatCompactNumber } from './health-monitor-utils'

const props = withDefaults(defineProps<{
  monitor: HealthRelatedMonitor
  lookbackHours: number
  generatedAt?: string | null
  showDetailButton?: boolean
}>(), {
  generatedAt: null,
  showDetailButton: true
})

defineEmits<{
  viewDetails: [monitor: HealthRelatedMonitor]
}>()

const iconComponent = computed(() => {
  switch (props.monitor.kind) {
    case 'model':
      return Bot
    case 'provider':
      return Server
    default:
      return Activity
  }
})

const entityLabel = computed(() => {
  switch (props.monitor.kind) {
    case 'model':
      return t('ui.model')
    case 'provider':
      return t('nav.providers')
    default:
      return t('ui.endpoint52fff16')
  }
})

const metaText = computed(() => {
  const attempts = t('heatmap.requests', { count: String(formatCompactNumber(props.monitor.total_attempts)) })
  if (props.monitor.meta_text?.includes('次请求')) return props.monitor.meta_text
  return props.monitor.meta_text
    ? `${props.monitor.meta_text} / ${attempts}`
    : attempts
})
</script>
