<template>
  <div class="px-4 sm:px-6 py-4 bg-muted/15 border-t border-border/40">
    <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between mb-4">
      <div>
        <div class="flex items-center gap-2">
          <h4 class="text-sm font-semibold">
            {{ t('ui.nodeData') }}
          </h4>
          <Badge
            variant="outline"
            class="text-[10px] px-1.5 py-0"
          >
            {{ t('ui.last24Hoursab2c188') }}
          </Badge>
        </div>
        <p class="text-xs text-muted-foreground mt-1">
          {{ loadedText }}
        </p>
      </div>
      <Button
        variant="ghost"
        size="sm"
        class="h-8 px-2 text-xs self-start sm:self-auto"
        :disabled="state?.loading"
        @click="$emit('refresh')"
      >
        <RefreshCw
          class="h-3.5 w-3.5 mr-1"
          :class="state?.loading ? 'animate-spin' : ''"
        />
        {{ t('ui.refresh') }}
      </Button>
    </div>

    <div
      v-if="state?.loading && !state.metrics"
      class="py-8 flex items-center justify-center gap-2 text-sm text-muted-foreground"
    >
      <Loader2 class="h-4 w-4 animate-spin" />
      {{ t('ui.loadingNodeData') }}
    </div>

    <div
      v-else-if="state?.error"
      class="rounded-lg border border-destructive/30 bg-destructive/5 px-4 py-3 text-sm text-destructive flex items-start gap-2"
    >
      <AlertTriangle class="h-4 w-4 mt-0.5 shrink-0" />
      <span>{{ state.error }}</span>
    </div>

    <div
      v-else
      class="space-y-4"
    >
      <div class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-6 gap-2">
        <div
          v-for="item in summaryStats"
          :key="item.label"
          class="rounded-lg border border-border/50 bg-background/70 px-3 py-2 min-w-0"
        >
          <div class="text-[11px] text-muted-foreground truncate">
            {{ item.label }}
          </div>
          <div
            class="text-sm font-semibold tabular-nums truncate mt-0.5"
            :class="item.tone"
          >
            {{ item.value }}
          </div>
          <div
            v-if="item.hint"
            class="text-[10px] text-muted-foreground truncate mt-0.5"
          >
            {{ item.hint }}
          </div>
        </div>
      </div>

      <div class="grid gap-4 lg:grid-cols-[minmax(0,1.35fr)_minmax(280px,0.9fr)]">
        <section class="rounded-lg border border-border/50 bg-background/70 p-3 min-w-0">
          <div class="flex items-center justify-between gap-3 mb-3">
            <div>
              <h5 class="text-xs font-semibold">
                {{ t('ui.uptimeSamples') }}
              </h5>
              <p class="text-[11px] text-muted-foreground mt-0.5">
                {{ t('ui.1hBucketsColorChangesWithUptimeAndErrorCount') }}
              </p>
            </div>
            <span class="text-[11px] text-muted-foreground tabular-nums shrink-0">{{ t('ui.valuePoints', { count: formatNumber(bucketItems.length) }) }}</span>
          </div>
          <div
            v-if="bucketItems.length > 0"
            class="h-20 flex items-end gap-1 overflow-hidden"
          >
            <div
              v-for="bucket in bucketItems"
              :key="bucket.bucket_start_unix_secs"
              class="flex-1 min-w-[4px] rounded-t-sm"
              :style="bucketBarStyle(bucket)"
              :title="bucketTitle(bucket)"
            />
          </div>
          <div
            v-else
            class="h-20 rounded-md bg-muted/30 flex items-center justify-center text-xs text-muted-foreground"
          >
            {{ t('ui.noSampleData') }}
          </div>
        </section>

        <section class="rounded-lg border border-border/50 bg-background/70 p-3 min-w-0">
          <div class="flex items-center justify-between gap-2 mb-3">
            <h5 class="text-xs font-semibold">
              {{ t('ui.recentEvents') }}
            </h5>
            <span class="text-[11px] text-muted-foreground tabular-nums">{{ recentEvents.length }}/8</span>
          </div>
          <div
            v-if="recentEvents.length === 0"
            class="h-20 rounded-md bg-muted/30 flex items-center justify-center text-xs text-muted-foreground"
          >
            {{ t('ui.noKeyEvents') }}
          </div>
          <div
            v-else
            class="space-y-1.5 max-h-28 overflow-y-auto pr-1"
          >
            <div
              v-for="event in recentEvents"
              :key="event.id"
              class="flex items-center gap-2 text-xs min-w-0"
            >
              <Badge
                :variant="eventTypeVariant(event.event_type)"
                class="text-[10px] px-1.5 py-0 shrink-0"
              >
                {{ eventTypeLabel(event.event_type) }}
              </Badge>
              <span
                class="text-muted-foreground truncate flex-1"
                :title="eventTooltip(event)"
              >{{ eventDetail(event) }}</span>
              <span class="text-[10px] text-muted-foreground/70 tabular-nums shrink-0">{{ formatTime(event.created_at || null) }}</span>
            </div>
          </div>
        </section>
      </div>

      <div class="grid gap-4 lg:grid-cols-2">
        <section class="rounded-lg border border-border/50 bg-background/70 p-3 min-w-0">
          <h5 class="text-xs font-semibold mb-3">
            {{ t('ui.hardwareAndResources') }}
          </h5>
          <div class="grid grid-cols-2 sm:grid-cols-3 gap-x-4 gap-y-2">
            <div
              v-for="item in resourceItems"
              :key="item.label"
              class="min-w-0"
            >
              <div class="text-[11px] text-muted-foreground truncate">
                {{ item.label }}
              </div>
              <div
                class="text-xs font-medium tabular-nums truncate mt-0.5"
                :class="item.tone"
              >
                {{ item.value }}
              </div>
              <div
                v-if="item.hint"
                class="text-[10px] text-muted-foreground truncate mt-0.5"
              >
                {{ item.hint }}
              </div>
            </div>
          </div>
        </section>

        <section class="rounded-lg border border-border/50 bg-background/70 p-3 min-w-0">
          <h5 class="text-xs font-semibold mb-3">
            {{ t('ui.realtimeSnapshot') }}
          </h5>
          <div class="grid grid-cols-2 sm:grid-cols-3 gap-x-4 gap-y-2">
            <div
              v-for="item in snapshotItems"
              :key="item.label"
              class="min-w-0"
            >
              <div class="text-[11px] text-muted-foreground truncate">
                {{ item.label }}
              </div>
              <div class="text-xs font-medium tabular-nums truncate mt-0.5">
                {{ item.value }}
              </div>
            </div>
          </div>
        </section>

        <section class="rounded-lg border border-border/50 bg-background/70 p-3 min-w-0">
          <h5 class="text-xs font-semibold mb-3">
            {{ t('ui.tunnelCounters') }}
          </h5>
          <div
            v-if="tunnelMetrics"
            class="grid grid-cols-2 sm:grid-cols-4 gap-x-4 gap-y-2"
          >
            <div
              v-for="item in tunnelCounterItems"
              :key="item.label"
              class="min-w-0"
            >
              <div class="text-[11px] text-muted-foreground truncate">
                {{ item.label }}
              </div>
              <div class="text-xs font-medium tabular-nums truncate mt-0.5">
                {{ item.value }}
              </div>
            </div>
          </div>
          <div
            v-else
            class="rounded-md bg-muted/30 py-4 text-center text-xs text-muted-foreground"
          >
            {{ t('ui.noTunnelCounters') }}
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { AlertTriangle, Loader2, RefreshCw } from 'lucide-vue-next'
import { Badge, Button } from '@/components/ui'
import { useI18n } from '@/i18n'
import { formatProxyNodeTime } from './proxy-node-display'
import { formatCompactNumber, formatByteSize } from '@/utils/format'
import type {
  ProxyNode,
  ProxyNodeEvent,
  ProxyNodeMetricsBucket,
  ProxyNodeMetricsResponse,
} from '@/api/proxy-nodes'

interface ProxyNodeDetailState {
  loading: boolean
  error: string | null
  node: ProxyNode | null
  metrics: ProxyNodeMetricsResponse | null
  events: ProxyNodeEvent[]
  loadedAt: number | null
}

const props = defineProps<{
  node: ProxyNode
  state?: ProxyNodeDetailState | null
}>()

defineEmits<{
  refresh: []
}>()

const { t, locale } = useI18n()

const detailNode = computed(() => props.state?.node ?? props.node)
const metrics = computed(() => props.state?.metrics ?? null)
const metricsSummary = computed(() => metrics.value?.summary ?? null)
const bucketItems = computed(() => metrics.value?.items.slice(-24) ?? [])
const recentEvents = computed(() => props.state?.events.slice(0, 8) ?? [])
const metadata = computed(() => asRecord(detailNode.value.proxy_metadata))
const hardwareInfo = computed(() => asRecord(detailNode.value.hardware_info))
const resourceUsage = computed(() => asRecord(metadata.value?.resource_usage))
const tunnelMetrics = computed(() => asRecord(metadata.value?.tunnel_metrics))

const loadedText = computed(() => {
  if (props.state?.loading) return t('ui.refreshingSampleData')
  if (!props.state?.loadedAt) return t('ui.expandingLoads24HourMetricsAndRecentKeyEventsAutomatically')
  return t('feedback.proxy.refreshedAt', { time: formatLoadedAt(props.state.loadedAt) })
})

const summaryStats = computed(() => {
  const summary = metricsSummary.value
  const totalBytes = (summary?.ws_in_bytes_delta ?? 0) + (summary?.ws_out_bytes_delta ?? 0)
  return [
    {
      label: t('ui.uptime'),
      value: formatPercent(summary?.uptime_ratio ?? null),
      hint: t('feedback.proxy.samples', { count: formatNumber(summary?.uptime_samples ?? 0), total: formatNumber(summary?.samples ?? 0) }),
      tone: uptimeTone(summary?.uptime_ratio ?? null),
    },
    {
      label: t('ui.heartbeatRtt'),
      value: formatMs(summary?.heartbeat_rtt_ms_avg ?? null),
      hint: t('feedback.proxy.peak', { value: formatMs(summary?.heartbeat_rtt_ms_max ?? null) }),
      tone: '',
    },
    {
      label: t('ui.peakConcurrency'),
      value: formatNumber(summary?.active_connections_max ?? 0),
      hint: t('feedback.proxy.average', { value: formatDecimal(summary?.active_connections_avg ?? null) }),
      tone: '',
    },
    {
      label: t('ui.disconnects'),
      value: formatNumber(summary?.disconnects_delta ?? 0),
      hint: '24h delta',
      tone: (summary?.disconnects_delta ?? 0) > 0 ? 'text-yellow-600 dark:text-yellow-400' : '',
    },
    {
      label: t('ui.connectionErrors'),
      value: formatNumber(summary?.connect_errors_delta ?? 0),
      hint: '24h delta',
      tone: (summary?.connect_errors_delta ?? 0) > 0 ? 'text-destructive' : '',
    },
    {
      label: t('ui.wsTraffic'),
      value: formatByteSize(totalBytes),
      hint: t('feedback.proxy.frames', { count: formatNumber((summary?.ws_in_frames_delta ?? 0) + (summary?.ws_out_frames_delta ?? 0)) }),
      tone: '',
    },
  ]
})

const snapshotItems = computed(() => {
  const node = detailNode.value
  return [
    { label: t('ui.currentConcurrency'), value: formatNumber(node.active_connections ?? 0) },
    { label: t('ui.heartbeatInterval'), value: `${node.heartbeat_interval ?? '-'}s` },
    { label: t('ui.lastHeartbeat'), value: formatTime(node.last_heartbeat_at) },
    { label: t('ui.tunnelConnection'), value: node.tunnel_connected ? t('ui.connected5be0323') : t('ui.notConnected3d52574') },
    { label: t('ui.connectedAt'), value: formatTime(node.tunnel_connected_at) },
    { label: t('ui.capacityEstimate'), value: node.estimated_max_concurrency == null ? '-' : formatNumber(node.estimated_max_concurrency) },
    { label: t('ui.configVersionfeb7d97'), value: `v${node.config_version}` },
    { label: t('ui.registeredBy'), value: node.registered_by || '-' },
    { label: t('ui.proxyVersion'), value: stringField(metadata.value, 'version') || '-' },
  ]
})

const resourceItems = computed(() => {
  const hardware = hardwareInfo.value
  const resource = resourceUsage.value
  const memoryTotalBytes = numberField(resource, 'memory_total_bytes') ?? memoryMbToBytes(numberField(hardware, 'total_memory_mb'))
  const memoryUsedBytes = numberField(resource, 'memory_used_bytes')
  const memoryUsedPercent = numberField(resource, 'memory_used_percent') ?? percentFromBytes(memoryUsedBytes, memoryTotalBytes)
  const processMemoryBytes = numberField(resource, 'process_memory_bytes')
  const processMemoryPercent = numberField(resource, 'process_memory_percent')
  const systemCpu = numberField(resource, 'system_cpu_usage_percent')
  const processCpu = numberField(resource, 'process_cpu_usage_percent')
  const load1 = numberField(resource, 'load_average_1m')
  const load5 = numberField(resource, 'load_average_5m')
  const load15 = numberField(resource, 'load_average_15m')
  const cpuCores = numberField(hardware, 'cpu_cores')
  return [
    {
      label: t('ui.systemCpu'),
      value: formatUsagePercent(systemCpu),
      hint: t('ui.hostOverall'),
      tone: usageTone(systemCpu, 70, 90),
    },
    {
      label: t('ui.systemMemory'),
      value: formatUsagePercent(memoryUsedPercent),
      hint: memoryUsedBytes == null || memoryTotalBytes == null
        ? ''
        : `${formatByteSize(memoryUsedBytes)} / ${formatByteSize(memoryTotalBytes)}`,
      tone: usageTone(memoryUsedPercent, 75, 90),
    },
    {
      label: 'Proxy CPU',
      value: formatUsagePercent(processCpu),
      hint: t('ui.currentProcess'),
      tone: usageTone(processCpu, 70, 90),
    },
    {
      label: t('ui.proxyMemory'),
      value: processMemoryBytes == null ? '-' : formatByteSize(processMemoryBytes),
      hint: processMemoryPercent == null ? '' : formatUsagePercent(processMemoryPercent),
      tone: usageTone(processMemoryPercent, 5, 15),
    },
    {
      label: t('ui.load'),
      value: formatLoadAverage(load1, load5, load15),
      hint: cpuCores == null ? '1/5/15m' : t('feedback.proxy.cores', { count: formatNumber(cpuCores) }),
      tone: loadTone(load1, cpuCores),
    },
    {
      label: t('ui.uptime5986e9d'),
      value: formatDurationSeconds(numberField(resource, 'process_uptime_secs')),
      hint: t('ui.proxyProcess'),
      tone: '',
    },
    {
      label: t('ui.cpuCores'),
      value: cpuCores == null ? '-' : formatNumber(cpuCores),
      hint: stringField(hardware, 'os_info') || '',
      tone: '',
    },
    {
      label: t('ui.physicalMemory'),
      value: memoryTotalBytes == null ? '-' : formatByteSize(memoryTotalBytes),
      hint: t('ui.collectedAtStartup'),
      tone: '',
    },
    {
      label: t('ui.fdLimit'),
      value: formatOptionalNumber(numberField(hardware, 'fd_limit')),
      hint: detailNode.value.estimated_max_concurrency == null
        ? ''
        : t('feedback.proxy.capacity', { value: formatNumber(detailNode.value.estimated_max_concurrency) }),
      tone: '',
    },
  ]
})

const tunnelCounterItems = computed(() => [
  { label: t('ui.connectAttempts'), value: formatTunnelNumber('connect_attempts') },
  { label: t('ui.connectSuccesses'), value: formatTunnelNumber('connect_successes') },
  { label: t('ui.connectErrors'), value: formatTunnelNumber('connect_errors') },
  { label: t('ui.disconnects'), value: formatTunnelNumber('disconnects') },
  { label: t('ui.lastConnected'), value: formatUnixSecs(numberField(tunnelMetrics.value, 'last_connected_at_unix_secs')) },
  { label: t('ui.lastDisconnected'), value: formatUnixSecs(numberField(tunnelMetrics.value, 'last_disconnected_at_unix_secs')) },
  { label: t('ui.lastOnlineDuration'), value: formatDurationMs(numberField(tunnelMetrics.value, 'last_connected_duration_ms')) },
  { label: t('ui.totalOnlineDuration'), value: formatDurationMs(numberField(tunnelMetrics.value, 'connected_duration_total_ms')) },
  { label: t('ui.heartbeatSent'), value: formatTunnelNumber('heartbeat_sent') },
  { label: t('ui.heartbeatAck'), value: formatTunnelNumber('heartbeat_ack') },
  { label: t('ui.latestRtt'), value: formatMs(numberField(tunnelMetrics.value, 'heartbeat_rtt_last_ms')) },
  { label: t('ui.averageRtt'), value: formatMs(numberField(tunnelMetrics.value, 'heartbeat_rtt_avg_ms')) },
  { label: t('ui.wsInbound'), value: formatByteSize(numberField(tunnelMetrics.value, 'ws_in_bytes') ?? 0) },
  { label: t('ui.wsOutbound'), value: formatByteSize(numberField(tunnelMetrics.value, 'ws_out_bytes') ?? 0) },
  { label: t('ui.inboundFrames'), value: formatTunnelNumber('ws_in_frames') },
  { label: t('ui.outboundFrames'), value: formatTunnelNumber('ws_out_frames') },
])

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  return value as Record<string, unknown>
}

function numberField(record: Record<string, unknown> | null, key: string): number | null {
  const value = record?.[key]
  if (typeof value === 'number' && Number.isFinite(value)) return value
  if (typeof value === 'string' && value.trim() !== '') {
    const parsed = Number(value)
    if (Number.isFinite(parsed)) return parsed
  }
  return null
}

function stringField(record: Record<string, unknown> | null, key: string): string | null {
  const value = record?.[key]
  if (typeof value !== 'string') return null
  const trimmed = value.trim()
  return trimmed || null
}

function formatTunnelNumber(key: string) {
  return formatNumber(numberField(tunnelMetrics.value, key) ?? 0)
}

function formatNumber(value: number) {
  if (!Number.isFinite(value)) return '-'
  return formatCompactNumber(Math.round(value), { fractionDigits: 1 })
}

function formatOptionalNumber(value: number | null) {
  return value == null ? '-' : formatNumber(value)
}

function formatDecimal(value: number | null) {
  if (value == null || !Number.isFinite(value)) return '-'
  if (value === 0) return '0'
  if (value < 10) return value.toFixed(1)
  return formatNumber(value)
}

function formatPercent(value: number | null) {
  if (value == null || !Number.isFinite(value)) return '-'
  return `${(value * 100).toFixed(value >= 0.995 || value === 0 ? 0 : 1)}%`
}

function formatUsagePercent(value: number | null) {
  if (value == null || !Number.isFinite(value)) return '-'
  if (value >= 100) return `${Math.round(value)}%`
  if (value >= 10) return `${value.toFixed(0)}%`
  return `${value.toFixed(1)}%`
}

function formatMs(value: number | null) {
  if (value == null || !Number.isFinite(value) || value <= 0) return '-'
  if (value >= 1000) return `${(value / 1000).toFixed(1)}s`
  return `${Math.round(value)}ms`
}

function formatDurationSeconds(value: number | null) {
  if (value == null || !Number.isFinite(value) || value <= 0) return '-'
  return formatDurationMs(value * 1000)
}

function formatDurationMs(value: number | null) {
  if (value == null || !Number.isFinite(value) || value <= 0) return '-'
  const totalSeconds = Math.floor(value / 1000)
  if (totalSeconds < 60) return `${totalSeconds}s`
  const totalMinutes = Math.floor(totalSeconds / 60)
  if (totalMinutes < 60) return `${totalMinutes}m`
  const totalHours = Math.floor(totalMinutes / 60)
  if (totalHours < 24) return `${totalHours}h ${totalMinutes % 60}m`
  const days = Math.floor(totalHours / 24)
  return `${days}d ${totalHours % 24}h`
}

function formatUnixSecs(value: number | null) {
  if (value == null || !Number.isFinite(value) || value <= 0) return '-'
  return formatTime(new Date(value * 1000).toISOString())
}

function formatLoadAverage(one: number | null, five: number | null, fifteen: number | null) {
  if (one == null && five == null && fifteen == null) return '-'
  return [one, five, fifteen]
    .map(value => (value == null || !Number.isFinite(value)) ? '-' : value.toFixed(value >= 10 ? 0 : 2))
    .join(' / ')
}

function memoryMbToBytes(value: number | null) {
  if (value == null || !Number.isFinite(value) || value <= 0) return null
  return value * 1024 * 1024
}

function percentFromBytes(value: number | null, total: number | null) {
  if (value == null || total == null || !Number.isFinite(value) || !Number.isFinite(total) || total <= 0) return null
  return value * 100 / total
}

function usageTone(value: number | null, warnAt: number, badAt: number) {
  if (value == null || !Number.isFinite(value)) return ''
  if (value >= badAt) return 'text-destructive'
  if (value >= warnAt) return 'text-yellow-600 dark:text-yellow-400'
  return ''
}

function loadTone(load: number | null, cpuCores: number | null) {
  if (load == null || cpuCores == null || !Number.isFinite(load) || !Number.isFinite(cpuCores) || cpuCores <= 0) return ''
  const ratio = load / cpuCores
  if (ratio >= 1.5) return 'text-destructive'
  if (ratio >= 1) return 'text-yellow-600 dark:text-yellow-400'
  return ''
}

function formatTime(iso: string | null) {
  return formatProxyNodeTime(iso, locale.value)
}

function formatLoadedAt(value: number) {
  return new Date(value).toLocaleTimeString(locale.value, { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

function bucketBarHeight(bucket: ProxyNodeMetricsBucket) {
  if (bucket.samples <= 0 || bucket.uptime_ratio == null) return '10%'
  return `${Math.max(10, Math.round(bucket.uptime_ratio * 100))}%`
}

function bucketBarStyle(bucket: ProxyNodeMetricsBucket) {
  return {
    height: bucketBarHeight(bucket),
    minHeight: bucket.samples > 0 ? '6px' : '4px',
    backgroundColor: bucketBarColor(bucket),
  }
}

function bucketBarColor(bucket: ProxyNodeMetricsBucket) {
  if (bucket.samples <= 0) return 'rgba(148, 163, 184, 0.35)'
  if (bucket.error_events_delta > 0 || bucket.connect_errors_delta > 0) return 'rgba(220, 38, 38, 0.86)'
  if ((bucket.uptime_ratio ?? 0) < 0.98 || bucket.disconnects_delta > 0) return 'rgba(217, 119, 6, 0.86)'
  return 'rgba(22, 163, 74, 0.86)'
}

function bucketTitle(bucket: ProxyNodeMetricsBucket) {
  const parts = [
    formatBucketTime(bucket.bucket_start),
    `${t('ui.uptime')} ${formatPercent(bucket.uptime_ratio)}`,
    `RTT ${formatMs(bucket.heartbeat_rtt_ms_avg)}`,
    `${t('ui.disconnected')} ${formatNumber(bucket.disconnects_delta)}`,
    `${t('ui.error')} ${formatNumber(bucket.connect_errors_delta + bucket.error_events_delta)}`,
  ]
  return parts.join(locale.value === 'en-US' ? ', ' : '，')
}

function formatBucketTime(value: string | null) {
  if (!value) return '-'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '-'
  return date.toLocaleString(locale.value, { month: '2-digit', day: '2-digit', hour: '2-digit' })
}

function uptimeTone(value: number | null) {
  if (value == null) return ''
  if (value < 0.95) return 'text-destructive'
  if (value < 0.99) return 'text-yellow-600 dark:text-yellow-400'
  return 'text-primary'
}

function eventTypeLabel(type: string) {
  switch (type) {
    case 'connected': return t('ui.connected')
    case 'disconnected': return t('ui.disconnected')
    case 'error': return t('ui.error')
    case 'tunnel_err': return t('ui.tunnelError')
    default: return type
  }
}

function eventTypeVariant(type: string) {
  switch (type) {
    case 'connected': return 'success' as const
    case 'disconnected':
    case 'error':
    case 'tunnel_err':
      return 'destructive' as const
    default: return 'secondary' as const
  }
}

function eventDetail(event: ProxyNodeEvent) {
  const metadata = asRecord(event.event_metadata)
  if (event.event_type === 'tunnel_err') {
    const category = stringField(metadata, 'category')
    const message = stringField(metadata, 'message')
    const summary = (category ? tunnelErrorSummary(category) : '') || stringField(metadata, 'summary') || category
    if (summary && message) return `${summary}：${message}`
    return summary || message || event.detail || '-'
  }
  return event.detail || stringField(metadata, 'message') || stringField(metadata, 'category') || '-'
}

function eventTooltip(event: ProxyNodeEvent) {
  const metadata = asRecord(event.event_metadata)
  const parts = [eventDetail(event)]
  const category = stringField(metadata, 'category')
  const action = (category ? tunnelErrorAction(category) : '') || stringField(metadata, 'operator_action')
  if (action) parts.push(t('feedback.proxy.suggestion', { action }))
  const component = stringField(metadata, 'component')
  const severity = stringField(metadata, 'severity')
  if (component || severity) parts.push([component, severity].filter(Boolean).join(' / '))
  return parts.filter(Boolean).join('\n')
}

function tunnelErrorSummary(category: string) {
  switch (category) {
    case 'stale_timeout': return t('ui.timedOutWaitingForInboundTunnelFrames')
    case 'ws_write_error': return t('ui.websocketWriteFailedPeerResetOrClosedTheConnection')
    case 'ws_ping_error': return t('ui.websocketKeepalivePingFailed')
    case 'ws_read_error': return t('ui.websocketReadFailed')
    case 'tunnel_connect_error': return t('ui.tunnelConnectionFailed')
    case 'frame_decode_error': return t('ui.tunnelFrameDecodingFailed')
    case 'stream_dispatch_timeout': return t('ui.requestStreamDispatchTimedOut')
    case 'heartbeat_ack_empty': return t('ui.heartbeatAcknowledgementWasEmpty')
    case 'heartbeat_ack_parse': return t('ui.failedToParseHeartbeatAcknowledgement')
    case 'writer_task_panic': return t('ui.tunnelWriterTaskExitedUnexpectedly')
    case 'writer_task_cancelled': return t('ui.tunnelWriterTaskWasCancelled')
    case 'dispatcher_error': return t('ui.tunnelDispatcherError')
    default: return ''
  }
}

function tunnelErrorAction(category: string) {
  switch (category) {
    case 'stale_timeout': return t('ui.checkGatewayReverseProxyIdleTimeoutCrossRegionNetworkJitter')
    case 'ws_write_error': return t('ui.checkWhetherTheGatewayRestartedLoadBalancerNatFirewallReset')
    case 'ws_ping_error': return t('ui.checkWhetherAnIntermediateProxyClearsIdleWebsocketsOrThe')
    case 'ws_read_error': return t('ui.compareGatewayLogsAndNetworkMonitoringAtTheSameTime')
    case 'tunnel_connect_error': return t('ui.checkTheAetherUrlDnsTlsManagementTokenAndAether')
    case 'frame_decode_error': return t('ui.checkProxyGatewayVersionCompatibilityAndConfirmNoIntermediateLayer')
    case 'stream_dispatch_timeout': return t('ui.checkProxyCpuMemoryConcurrencyLimitsAndSlowUpstreamProvider')
    case 'heartbeat_ack_empty':
    case 'heartbeat_ack_parse':
      return t('ui.checkGatewayHeartbeatHandlingLogsAndProxyGatewayVersionCompatibility')
    case 'writer_task_panic':
    case 'writer_task_cancelled':
      return t('ui.reviewThePreviousWriteOrPingErrorAndConfirmThe')
    case 'dispatcher_error': return t('ui.checkRequestStreamLogsAndGatewayTunnelLogsFromThe')
    default: return ''
  }
}
</script>
