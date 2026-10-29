<template>
  <HealthMonitorSection
    :title="title || t('ui.modelHealthMonitor')"
    :description="t('ui.basedOnRealRequestStatisticsModelAvailabilityAverageDurationAverage')"
    :icon="Bot"
    :empty-title="t('ui.noModelHealthMonitoringData')"
    :empty-description="t('ui.thisModelHasNoRequestRecordsYet')"
    :entity-label="t('ui.model')"
    :error-message="t('ui.failedToLoadModelHealthMonitoringData')"
    :load="loadModelMonitors"
    @view-details="$emit('viewDetails', $event)"
    @summary-updated="$emit('summaryUpdated', $event)"
  />
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { Bot } from 'lucide-vue-next'
import HealthMonitorSection from './HealthMonitorSection.vue'
import type { HealthMonitorSectionData } from './health-monitor-utils'
import { getModelStatusMonitor, getPublicModelStatusMonitor } from '@/api/endpoints/health'
import type { ModelStatusMonitor } from '@/api/endpoints/types'
import {
  formatCompactNumber,
  type HealthMonitorDetailTarget,
  type HealthMonitorSectionSummary,
} from './health-monitor-utils'

const props = withDefaults(defineProps<{
  title?: string
  isAdmin?: boolean
  showProviderInfo?: boolean
}>(), {
  title: '',
  isAdmin: false,
  showProviderInfo: false,
})

defineEmits<{
  viewDetails: [target: HealthMonitorDetailTarget]
  summaryUpdated: [summary: HealthMonitorSectionSummary]
}>()

const { t } = useI18n()

function getModelMetaText(monitor: ModelStatusMonitor) {
  const attempts = t('heatmap.requests', { count: String(formatCompactNumber(monitor.total_attempts)) })
  if (props.showProviderInfo && typeof monitor.provider_count === 'number') {
    return t('providers.migration.providersa11ac4', { value: String(monitor.provider_count), value_2: String(attempts) })
  }
  return attempts
}

function getModelDetailMetaText(monitor: ModelStatusMonitor) {
  if (props.showProviderInfo && typeof monitor.provider_count === 'number') {
    return t('providers.migration.providers0084fc', { value: String(monitor.provider_count) })
  }
  return null
}

async function loadModelMonitors(lookbackHours: number): Promise<HealthMonitorSectionData<ModelStatusMonitor>> {
  const params = { lookback_hours: lookbackHours, model_limit: 12, per_model_limit: 100 }
  const data = props.isAdmin
    ? await getModelStatusMonitor(params)
    : await getPublicModelStatusMonitor(params)
  return {
    generatedAt: data.generated_at,
    items: (data.models || []).map(monitor => ({
      key: monitor.model,
      title: monitor.display_name || monitor.model,
      metaText: getModelMetaText(monitor),
      entityName: monitor.model,
      monitor,
      raw: monitor,
      detail: {
        kind: 'model',
        value: monitor.model,
        title: monitor.display_name || monitor.model,
        metaText: getModelDetailMetaText(monitor),
        totalAttempts: monitor.total_attempts,
        successCount: monitor.success_count,
        failedCount: monitor.failed_count,
        successRate: monitor.success_rate,
        avgLatencyMs: monitor.avg_latency_ms,
        avgFirstByteMs: monitor.avg_first_byte_ms,
        avgTps: monitor.avg_tps,
        timeline: monitor.timeline || null,
        timelineDetails: monitor.timeline_details || null,
        timeRangeStart: monitor.time_range_start || null,
        timeRangeEnd: monitor.time_range_end || null,
      },
    })),
  }
}

</script>
