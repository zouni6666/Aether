<template>
  <HealthMonitorSection
    :title="title || t('nav.healthMonitor')"
    :description="t('ui.basedOnRealRequestStatisticsEndpointAvailabilityAverageDurationAverage')"
    :icon="Activity"
    :empty-title="t('ui.noHealthMonitoringData')"
    :empty-description="t('ui.thisEndpointHasNoRequestRecordsYet')"
    :entity-label="t('ui.endpoint52fff16')"
    :error-message="t('ui.failedToLoadHealthMonitoringData')"
    :load="loadEndpointMonitors"
    @view-details="$emit('viewDetails', $event)"
    @summary-updated="$emit('summaryUpdated', $event)"
  >
    <template #timeline="{ item, lookbackHours }">
      <div class="mt-2">
        <EndpointHealthTimeline
          :monitor="item.raw"
          :lookback-hours="lookbackHours"
        />
      </div>
    </template>
  </HealthMonitorSection>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { Activity } from 'lucide-vue-next'
import HealthMonitorSection from './HealthMonitorSection.vue'
import type { HealthMonitorSectionData } from './health-monitor-utils'
import EndpointHealthTimeline from './EndpointHealthTimeline.vue'
import { getEndpointStatusMonitor, getPublicEndpointStatusMonitor } from '@/api/endpoints/health'
import type { EndpointStatusMonitor, PublicEndpointStatusMonitor } from '@/api/endpoints/types'
import { formatApiFormat } from '@/api/endpoints/types/api-format'
import {
  formatCompactNumber,
  type HealthMonitorDetailTarget,
  type HealthMonitorSectionSummary,
} from './health-monitor-utils'

type EndpointMonitor = EndpointStatusMonitor | PublicEndpointStatusMonitor

const props = withDefaults(defineProps<{
  title?: string
  isAdmin?: boolean
  showProviderInfo?: boolean
}>(), {
  title: '',
  isAdmin: false,
  showProviderInfo: false
})

defineEmits<{
  viewDetails: [target: HealthMonitorDetailTarget]
  summaryUpdated: [summary: HealthMonitorSectionSummary]
}>()

const { t } = useI18n()

function hasProviderInfo(monitor: EndpointMonitor): monitor is EndpointStatusMonitor {
  return 'provider_count' in monitor && typeof monitor.provider_count === 'number'
}

function hasApiPath(monitor: EndpointMonitor): monitor is PublicEndpointStatusMonitor {
  return 'api_path' in monitor && typeof monitor.api_path === 'string' && monitor.api_path.length > 0
}

function getEndpointMetaText(monitor: EndpointMonitor) {
  const attempts = t('heatmap.requests', { count: String(formatCompactNumber(monitor.total_attempts)) })
  if (props.showProviderInfo && hasProviderInfo(monitor)) {
    return t('providers.migration.providersKeys6fdfef', { value: String(monitor.provider_count), value_2: String(monitor.key_count), value_3: String(attempts) })
  }
  if (hasApiPath(monitor)) {
    return `${monitor.api_path} / ${attempts}`
  }
  return attempts
}

function getEndpointDetailMetaText(monitor: EndpointMonitor) {
  if (props.showProviderInfo && hasProviderInfo(monitor)) {
    return t('providers.migration.providersKeys168008', { value: String(monitor.provider_count), value_2: String(monitor.key_count) })
  }
  if (hasApiPath(monitor)) {
    return monitor.api_path
  }
  return null
}

async function loadEndpointMonitors(lookbackHours: number): Promise<HealthMonitorSectionData<EndpointMonitor>> {
  const params = { lookback_hours: lookbackHours, per_format_limit: 100 }
  const data = props.isAdmin
    ? await getEndpointStatusMonitor(params)
    : await getPublicEndpointStatusMonitor(params)
  const monitors: EndpointMonitor[] = (data.formats || []).filter(monitor => monitor.total_attempts > 0)
  return {
    generatedAt: data.generated_at,
    items: monitors.map((monitor, index) => ({
      key: `${monitor.api_format}-${index}`,
      title: formatApiFormat(monitor.api_format),
      metaText: getEndpointMetaText(monitor),
      entityName: formatApiFormat(monitor.api_format),
      monitor,
      raw: monitor,
      detail: {
        kind: 'endpoint',
        value: monitor.api_format,
        title: formatApiFormat(monitor.api_format),
        metaText: getEndpointDetailMetaText(monitor),
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
        timeRangeEnd: monitor.time_range_end || null
      },
    })),
  }
}

</script>
