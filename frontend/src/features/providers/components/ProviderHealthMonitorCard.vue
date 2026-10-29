<template>
  <HealthMonitorSection
    :title="title || t('ui.providerHealthMonitor')"
    :description="t('ui.onlyActiveProvidersAreShownClickDetailsToViewLinked')"
    :icon="Server"
    :empty-title="t('ui.noProviderHealthData')"
    :empty-description="t('ui.noProvidersHaveRequestRecordsYet')"
    :entity-label="t('nav.providers')"
    :error-message="t('ui.failedToLoadProviderHealthMonitoringData')"
    :load="loadProviderMonitors"
    @view-details="$emit('viewDetails', $event)"
    @summary-updated="$emit('summaryUpdated', $event)"
  />
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { Server } from 'lucide-vue-next'
import HealthMonitorSection from './HealthMonitorSection.vue'
import type { HealthMonitorSectionData } from './health-monitor-utils'
import { getProviderStatusMonitor } from '@/api/endpoints/health'
import type { ProviderStatusMonitor } from '@/api/endpoints/types'
import {
  formatCompactNumber,
  type HealthMonitorDetailTarget,
  type HealthMonitorSectionSummary,
} from './health-monitor-utils'

withDefaults(defineProps<{
  title?: string
}>(), {
  title: ''
})

defineEmits<{
  viewDetails: [target: HealthMonitorDetailTarget]
  summaryUpdated: [summary: HealthMonitorSectionSummary]
}>()

const { t } = useI18n()

async function loadProviderMonitors(lookbackHours: number): Promise<HealthMonitorSectionData<ProviderStatusMonitor>> {
  const data = await getProviderStatusMonitor({
    lookback_hours: lookbackHours,
    provider_limit: 50,
    per_provider_model_limit: 12,
    per_model_limit: 100
  })
  const providers = (data.providers || []).filter(provider => provider.total_attempts > 0)
  return {
    generatedAt: data.generated_at,
    items: providers.map(provider => ({
      key: provider.provider_id,
      title: provider.provider_name,
      metaText: t('providers.migration.modelsRequests310deb', { value: String(provider.model_count), value_2: String(formatCompactNumber(provider.total_attempts)) }),
      entityName: provider.provider_name,
      monitor: provider,
      raw: provider,
      detail: {
        kind: 'provider',
        value: provider.provider_name,
        title: provider.provider_name,
        metaText: null,
        totalAttempts: provider.total_attempts,
        successCount: provider.success_count,
        failedCount: provider.failed_count,
        successRate: provider.success_rate,
        avgLatencyMs: provider.avg_latency_ms,
        avgFirstByteMs: provider.avg_first_byte_ms,
        avgTps: provider.avg_tps,
        timeline: provider.timeline || null,
        timelineDetails: provider.timeline_details || null,
        timeRangeStart: provider.time_range_start || null,
        timeRangeEnd: provider.time_range_end || null
      },
    })),
  }
}

</script>
