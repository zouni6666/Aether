<template>
  <div class="space-y-5 px-4 pb-8 sm:px-6 lg:px-0">
    <div class="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
      <div>
        <h1 class="text-lg font-semibold">
          运维总览
        </h1>
        <p class="text-xs text-muted-foreground">
          统一查看流量、吞吐、延迟、错误、上游健康、缓存与审计风险
        </p>
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <Badge variant="outline">
          自动刷新 10s
        </Badge>
        <span class="text-xs text-muted-foreground">
          更新 {{ lastUpdatedLabel }}
        </span>
        <RefreshButton
          :loading="refreshing"
          title="刷新运维总览"
          @click="refreshAll"
        />
        <TimeRangePicker
          v-model="timeRange"
          :allow-hourly="true"
        />
      </div>
    </div>

    <div
      v-if="loadWarning"
      class="rounded-lg border border-yellow-300/70 bg-yellow-50/80 px-3 py-2 text-xs text-yellow-900 dark:border-yellow-900/60 dark:bg-yellow-950/30 dark:text-yellow-100"
    >
      {{ loadWarning }}
    </div>

    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4 2xl:grid-cols-8">
      <Card
        v-for="card in kpiCards"
        :key="card.title"
        class="min-h-[118px] p-4"
      >
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <p class="truncate text-xs text-muted-foreground">
              {{ card.title }}
            </p>
            <div
              class="mt-2 truncate text-2xl font-semibold tabular-nums"
              :class="card.valueClass"
            >
              {{ card.value }}
            </div>
          </div>
          <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-border/60 bg-muted/30">
            <component
              :is="card.icon"
              class="h-4 w-4"
              :class="card.iconClass"
            />
          </div>
        </div>
        <p class="mt-3 line-clamp-2 text-xs text-muted-foreground">
          {{ card.hint }}
        </p>
      </Card>
    </div>

    <div class="grid grid-cols-1 gap-4 xl:grid-cols-3">
      <Card class="overflow-hidden xl:col-span-2">
        <div class="border-b border-border/70 bg-muted/20 px-4 py-3">
          <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <h2 class="text-sm font-semibold">
                流量与吞吐
              </h2>
              <p class="text-xs text-muted-foreground">
                请求、Token 与费用趋势，按当前时间窗口聚合
              </p>
            </div>
            <Badge variant="outline">
              {{ timeRange.granularity || 'day' }}
            </Badge>
          </div>
        </div>
        <div class="p-4">
          <div
            v-if="trendLoading"
            class="py-12"
          >
            <LoadingState message="加载流量趋势中" />
          </div>
          <div
            v-else
            class="h-[302px]"
          >
            <LineChart
              :data="trafficChartData"
              :options="trafficChartOptions"
            />
          </div>
        </div>
      </Card>

      <Card class="overflow-hidden">
        <div class="border-b border-border/70 bg-muted/20 px-4 py-3">
          <div class="flex items-center justify-between gap-3">
            <div>
              <h2 class="text-sm font-semibold">
                实时并发
              </h2>
              <p class="text-xs text-muted-foreground">
                网关、全局锁与代理通道
              </p>
            </div>
            <Badge :variant="distributedGateVariant">
              {{ distributedGateText }}
            </Badge>
          </div>
        </div>
        <div class="space-y-4 p-4">
          <div class="grid grid-cols-2 gap-3 text-sm">
            <MetricCell
              label="本机处理中"
              :value="formatMetricNumber(gatewayMetrics?.local.inFlight)"
            />
            <MetricCell
              label="本机可接入"
              :value="formatMetricNumber(gatewayMetrics?.local.availablePermits)"
            />
            <MetricCell
              label="全局处理中"
              :value="formatMetricNumber(gatewayMetrics?.distributed.inFlight)"
            />
            <MetricCell
              label="全局可接入"
              :value="formatMetricNumber(gatewayMetrics?.distributed.availablePermits)"
            />
            <MetricCell
              label="代理活跃流"
              :value="formatMetricNumber(currentActiveStreams)"
            />
            <MetricCell
              label="代理连接"
              :value="formatMetricNumber(currentProxyConnections)"
            />
          </div>
          <div class="rounded-lg border border-border/60 bg-background/45 px-3 py-3">
            <div class="flex items-center justify-between gap-3 text-xs">
              <span class="text-muted-foreground">队列利用率</span>
              <span class="font-medium tabular-nums">{{ tunnelQueueUtilizationText }}</span>
            </div>
            <div class="mt-2 h-2 overflow-hidden rounded-full bg-muted">
              <div
                class="h-full rounded-full bg-primary"
                :style="{ width: tunnelQueueUtilizationWidth }"
              />
            </div>
            <div class="mt-3 grid grid-cols-2 gap-2 text-xs text-muted-foreground">
              <span>拒绝 {{ formatMetricNumber(tunnelQueueRejectedTotal) }}</span>
              <span>选择压力 {{ formatMetricNumber(tunnelSelectionPressureTotal) }}</span>
            </div>
          </div>
        </div>
      </Card>
    </div>

    <div class="grid grid-cols-1 gap-4 xl:grid-cols-3">
      <Card class="overflow-hidden xl:col-span-2">
        <div class="border-b border-border/70 bg-muted/20 px-4 py-3">
          <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <h2 class="text-sm font-semibold">
                延迟与首字
              </h2>
              <p class="text-xs text-muted-foreground">
                P50/P90/P99 请求耗时与 TTFT 趋势
              </p>
            </div>
            <RouterLink
              to="/admin/performance-analysis"
              class="text-xs font-medium text-primary hover:underline"
            >
              查看性能分析
            </RouterLink>
          </div>
        </div>
        <div class="p-4">
          <div
            v-if="percentileLoading"
            class="py-12"
          >
            <LoadingState message="加载延迟百分位中" />
          </div>
          <div
            v-else
            class="h-[288px]"
          >
            <LineChart
              :data="latencyChartData"
              :options="latencyChartOptions"
            />
          </div>
        </div>
      </Card>

      <Card class="overflow-hidden">
        <div class="border-b border-border/70 bg-muted/20 px-4 py-3">
          <h2 class="text-sm font-semibold">
            SLA 与错误
          </h2>
          <p class="text-xs text-muted-foreground">
            请求错误、上游错误与熔断风险
          </p>
        </div>
        <div class="space-y-4 p-4">
          <div class="grid grid-cols-2 gap-3">
            <MetricCell
              label="成功率"
              :value="formatPercent(providerPerformance?.summary.success_rate)"
              :value-class="slaValueClass"
            />
            <MetricCell
              label="错误率"
              :value="formatErrorRate(providerPerformance?.summary.success_rate)"
              :value-class="errorRateValueClass"
            />
            <MetricCell
              label="请求错误"
              :value="formatMetricNumber(summaryStats?.error_requests)"
            />
            <MetricCell
              label="熔断打开"
              :value="formatMetricNumber(resilienceStatus?.error_statistics.open_circuit_breakers)"
            />
          </div>

          <div>
            <div class="mb-2 flex items-center justify-between gap-3">
              <span class="text-xs font-medium text-muted-foreground">错误分类</span>
              <RouterLink
                to="/admin/audit-logs"
                class="text-xs font-medium text-primary hover:underline"
              >
                审计
              </RouterLink>
            </div>
            <div class="h-[154px]">
              <DoughnutChart
                :data="errorDistributionChartData"
                :options="errorDistributionOptions"
              />
            </div>
          </div>
        </div>
      </Card>
    </div>

    <div class="grid grid-cols-1 gap-4 xl:grid-cols-3">
      <Card class="overflow-hidden xl:col-span-2">
        <div class="border-b border-border/70 bg-muted/20 px-4 py-3">
          <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <h2 class="text-sm font-semibold">
                上游健康与吞吐
              </h2>
              <p class="text-xs text-muted-foreground">
                Provider 维度的 TPS、TTFT、慢请求与错误样本
              </p>
            </div>
            <RouterLink
              to="/admin/health-monitor"
              class="text-xs font-medium text-primary hover:underline"
            >
              健康监控
            </RouterLink>
          </div>
        </div>
        <div class="overflow-x-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>上游</TableHead>
                <TableHead class="text-right">
                  请求
                </TableHead>
                <TableHead class="text-right">
                  成功率
                </TableHead>
                <TableHead class="text-right">
                  TPS
                </TableHead>
                <TableHead class="text-right">
                  TTFT
                </TableHead>
                <TableHead class="text-right">
                  P99
                </TableHead>
                <TableHead class="text-right">
                  慢请求
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for="provider in providerRows"
                :key="provider.provider_id"
              >
                <TableCell>
                  <div class="max-w-[220px] truncate text-sm font-medium">
                    {{ provider.provider }}
                  </div>
                </TableCell>
                <TableCell class="text-right tabular-nums">
                  {{ formatMetricNumber(provider.request_count) }}
                </TableCell>
                <TableCell
                  class="text-right tabular-nums"
                  :class="successRateClass(provider.success_rate)"
                >
                  {{ formatPercent(provider.success_rate) }}
                </TableCell>
                <TableCell class="text-right tabular-nums">
                  {{ formatTps(provider.avg_output_tps) }}
                </TableCell>
                <TableCell class="text-right tabular-nums">
                  {{ formatMs(provider.avg_first_byte_time_ms) }}
                </TableCell>
                <TableCell class="text-right tabular-nums">
                  {{ formatMs(provider.p99_response_time_ms) }}
                </TableCell>
                <TableCell class="text-right tabular-nums">
                  {{ formatMetricNumber(provider.slow_request_count) }}
                </TableCell>
              </TableRow>
              <TableRow v-if="providerRows.length === 0">
                <TableCell
                  colspan="7"
                  class="py-8 text-center text-sm text-muted-foreground"
                >
                  当前时间窗口暂无上游样本
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>
      </Card>

      <Card class="overflow-hidden">
        <div class="border-b border-border/70 bg-muted/20 px-4 py-3">
          <div class="flex items-center justify-between gap-3">
            <div>
              <h2 class="text-sm font-semibold">
                资源、数据流与缓存
              </h2>
              <p class="text-xs text-muted-foreground">
                代理节点资源、WebSocket 流量、Redis Key 分类与缓存命中
              </p>
            </div>
            <RouterLink
              to="/admin/cache-monitoring"
              class="text-xs font-medium text-primary hover:underline"
            >
              缓存页
            </RouterLink>
          </div>
        </div>
        <div class="space-y-4 p-4">
          <div class="grid grid-cols-2 gap-3">
            <MetricCell
              label="CPU"
              :value="formatPercentResource(resourceSnapshot?.avgCpuPercent)"
              :value-class="resourceToneClass(resourceSnapshot?.avgCpuPercent, 70, 90)"
            />
            <MetricCell
              label="内存"
              :value="formatPercentResource(resourceSnapshot?.avgMemoryPercent)"
              :value-class="resourceToneClass(resourceSnapshot?.avgMemoryPercent, 75, 90)"
            />
            <MetricCell
              label="WS 入站"
              :value="formatBytes(resourceSnapshot?.wsInBytes)"
            />
            <MetricCell
              label="WS 出站"
              :value="formatBytes(resourceSnapshot?.wsOutBytes)"
            />
            <MetricCell
              label="Redis 状态"
              :value="redisStatusText"
              :value-class="redisStatusClass"
            />
            <MetricCell
              label="Redis Keys"
              :value="formatMetricNumber(redisCategories?.total_keys)"
            />
            <MetricCell
              label="亲和缓存"
              :value="formatMetricNumber(cacheStats?.affinity_stats.total_affinities)"
            />
            <MetricCell
              label="命中率"
              :value="formatPercent(cacheHitRate)"
            />
          </div>

          <div class="space-y-2">
            <div
              v-for="item in redisCategoryRows"
              :key="item.key"
              class="flex items-center justify-between gap-3 rounded-lg border border-border/50 bg-background/45 px-3 py-2 text-xs"
            >
              <div class="min-w-0">
                <div class="truncate font-medium">
                  {{ item.name }}
                </div>
                <div class="truncate text-muted-foreground">
                  {{ item.pattern }}
                </div>
              </div>
              <span class="font-semibold tabular-nums">{{ formatMetricNumber(item.count) }}</span>
            </div>
          </div>
        </div>
      </Card>
    </div>

    <div class="grid grid-cols-1 gap-4 xl:grid-cols-3">
      <Card class="overflow-hidden xl:col-span-2">
        <div class="border-b border-border/70 bg-muted/20 px-4 py-3">
          <div class="flex items-center justify-between gap-3">
            <div>
              <h2 class="text-sm font-semibold">
                错误审计
              </h2>
              <p class="text-xs text-muted-foreground">
                最近错误、熔断事件和可疑活动入口
              </p>
            </div>
            <RouterLink
              to="/admin/usage?status=error"
              class="text-xs font-medium text-primary hover:underline"
            >
              使用记录
            </RouterLink>
          </div>
        </div>
        <div class="divide-y divide-border/50">
          <div
            v-for="error in recentErrors"
            :key="error.error_id"
            class="grid grid-cols-1 gap-2 px-4 py-3 text-sm lg:grid-cols-[180px_1fr_120px]"
          >
            <div class="font-medium">
              {{ error.error_type }}
            </div>
            <div class="min-w-0 text-xs text-muted-foreground">
              <div class="truncate">
                {{ error.context.error_message || error.operation }}
              </div>
              <div class="mt-1 truncate">
                {{ error.context.provider_name || error.context.provider_id || '-' }} / {{ error.context.model || '-' }}
              </div>
            </div>
            <div class="text-xs text-muted-foreground lg:text-right">
              {{ formatShortDate(error.timestamp) }}
            </div>
          </div>
          <div
            v-if="recentErrors.length === 0"
            class="px-4 py-8 text-center text-sm text-muted-foreground"
          >
            当前没有近期错误
          </div>
        </div>
      </Card>

      <Card class="overflow-hidden">
        <div class="border-b border-border/70 bg-muted/20 px-4 py-3">
          <h2 class="text-sm font-semibold">
            运维入口
          </h2>
          <p class="text-xs text-muted-foreground">
            常用排障视角
          </p>
        </div>
        <div class="grid grid-cols-1 gap-2 p-4">
          <RouterLink
            v-for="link in opsLinks"
            :key="link.to"
            :to="link.to"
            class="flex items-center justify-between rounded-lg border border-border/60 bg-background/45 px-3 py-3 text-sm transition-colors hover:border-primary/50 hover:bg-primary/5"
          >
            <span class="font-medium">{{ link.label }}</span>
            <component
              :is="link.icon"
              class="h-4 w-4 text-muted-foreground"
            />
          </RouterLink>
        </div>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineComponent, h, onMounted, onUnmounted, ref, watch, type Component } from 'vue'
import { RouterLink } from 'vue-router'
import type { ChartData, ChartOptions } from 'chart.js'
import {
  Activity,
  AlertTriangle,
  BarChart3,
  CircleDollarSign,
  Database,
  Gauge,
  ListChecks,
  RefreshCw,
  ShieldCheck,
  Timer,
  Zap,
} from 'lucide-vue-next'
import { Badge, Card, RefreshButton, Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui'
import { LoadingState, TimeRangePicker } from '@/components/common'
import LineChart from '@/components/charts/LineChart.vue'
import DoughnutChart from '@/components/charts/DoughnutChart.vue'
import { adminApi, type ErrorDistributionItem, type PercentileItem, type ProviderPerformanceResponse } from '@/api/admin'
import { cacheApi, redisCacheApi, type CacheStats, type RedisCacheCategoriesResponse } from '@/api/cache'
import { monitoringApi, type AdminMonitoringRecentError, type AdminMonitoringResilienceStatus, type AdminMonitoringSystemStatus, type GatewayMetricsSummary } from '@/api/monitoring'
import { proxyNodesApi, type ProxyNode, type ProxyNodeMetricsResponse } from '@/api/proxy-nodes'
import { usageApi, type UsageStats } from '@/api/usage'
import { getDateRangeFromPeriod } from '@/features/usage/composables'
import type { DateRangeParams } from '@/features/usage/types'
import { formatByteSize, formatCurrency, formatNumber, formatTokens } from '@/utils/format'
import { log } from '@/utils/logger'

interface MetricCellProps {
  label: string
  value: string
  valueClass?: string
}

const MetricCell = defineComponent<MetricCellProps>({
  name: 'MetricCell',
  props: {
    label: { type: String, required: true },
    value: { type: String, required: true },
    valueClass: { type: String, default: '' },
  },
  setup(props) {
    return () => h('div', { class: 'rounded-lg border border-border/60 bg-background/45 px-3 py-3' }, [
      h('div', { class: 'text-xs text-muted-foreground' }, props.label),
      h('div', { class: ['mt-1 truncate text-lg font-semibold tabular-nums', props.valueClass] }, props.value),
    ])
  },
})

const AUTO_REFRESH_MS = 10_000
const DEFAULT_SLOW_THRESHOLD_MS = 10_000

interface ResourceSnapshot {
  totalNodes: number
  onlineNodes: number
  avgCpuPercent: number | null
  avgMemoryPercent: number | null
  wsInBytes: number | null
  wsOutBytes: number | null
}

const timeRange = ref<DateRangeParams>({
  ...getDateRangeFromPeriod('today'),
  granularity: 'hour',
})
const summaryStats = ref<UsageStats | null>(null)
const timeSeries = ref<Array<Record<string, unknown>>>([])
const percentiles = ref<PercentileItem[]>([])
const providerPerformance = ref<ProviderPerformanceResponse | null>(null)
const errorDistribution = ref<ErrorDistributionItem[]>([])
const systemStatus = ref<AdminMonitoringSystemStatus | null>(null)
const resilienceStatus = ref<AdminMonitoringResilienceStatus | null>(null)
const gatewayMetrics = ref<GatewayMetricsSummary | null>(null)
const cacheStats = ref<CacheStats | null>(null)
const redisCategories = ref<RedisCacheCategoriesResponse | null>(null)
const resourceSnapshot = ref<ResourceSnapshot | null>(null)
const lastUpdatedAt = ref<string | null>(null)
const loadWarning = ref<string | null>(null)
const refreshing = ref(false)
const trendLoading = ref(false)
const percentileLoading = ref(false)
let refreshTimer: ReturnType<typeof setInterval> | null = null
let requestId = 0

const timeRangeParams = computed(() => ({
  start_date: timeRange.value.start_date,
  end_date: timeRange.value.end_date,
  preset: timeRange.value.preset,
  timezone: timeRange.value.timezone,
  tz_offset_minutes: timeRange.value.tz_offset_minutes,
  granularity: timeRange.value.granularity || 'day',
}))

function numeric(value: unknown): number {
  if (typeof value === 'number' && Number.isFinite(value)) return value
  if (typeof value === 'string') {
    const parsed = Number(value)
    return Number.isFinite(parsed) ? parsed : 0
  }
  return 0
}

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  return value as Record<string, unknown>
}

function numberField(record: Record<string, unknown> | null | undefined, key: string): number | null {
  if (!record) return null
  const value = record[key]
  if (typeof value === 'number' && Number.isFinite(value)) return value
  if (typeof value === 'string') {
    const parsed = Number(value)
    return Number.isFinite(parsed) ? parsed : null
  }
  return null
}

function averageFinite(values: Array<number | null | undefined>): number | null {
  const numbers = values.filter((value): value is number => typeof value === 'number' && Number.isFinite(value))
  if (!numbers.length) return null
  return numbers.reduce((total, value) => total + value, 0) / numbers.length
}

function memoryUsedPercent(node: ProxyNode): number | null {
  const metadata = asRecord(node.proxy_metadata)
  const resource = asRecord(metadata?.resource_usage)
  const hardware = asRecord(node.hardware_info)
  const explicitPercent = numberField(resource, 'memory_used_percent')
  if (explicitPercent != null) return explicitPercent
  const usedBytes = numberField(resource, 'memory_used_bytes')
  const totalBytes = numberField(resource, 'memory_total_bytes')
    ?? numberField(hardware, 'memory_total_bytes')
    ?? numberField(hardware, 'total_memory_bytes')
  if (usedBytes == null || totalBytes == null || totalBytes <= 0) return null
  return usedBytes / totalBytes * 100
}

function cpuUsedPercent(node: ProxyNode): number | null {
  const metadata = asRecord(node.proxy_metadata)
  const resource = asRecord(metadata?.resource_usage)
  return numberField(resource, 'system_cpu_usage_percent')
    ?? numberField(resource, 'process_cpu_usage_percent')
}

async function loadResourceSnapshot(): Promise<ResourceSnapshot | null> {
  const now = Math.floor(Date.now() / 1000)
  const from = now - 3600
  const [nodesResult, fleetResult] = await Promise.allSettled([
    proxyNodesApi.listProxyNodes({ limit: 200 }),
    proxyNodesApi.listFleetMetrics({ from, to: now, step: '1m' }),
  ])

  const nodes = nodesResult.status === 'fulfilled' ? nodesResult.value.items : []
  const fleet: ProxyNodeMetricsResponse | null = fleetResult.status === 'fulfilled'
    ? fleetResult.value
    : null
  const onlineNodes = nodes.filter(node => node.status === 'online' || node.tunnel_connected)

  return {
    totalNodes: nodes.length,
    onlineNodes: onlineNodes.length,
    avgCpuPercent: averageFinite(onlineNodes.map(cpuUsedPercent)),
    avgMemoryPercent: averageFinite(onlineNodes.map(memoryUsedPercent)),
    wsInBytes: fleet?.summary.ws_in_bytes_delta ?? null,
    wsOutBytes: fleet?.summary.ws_out_bytes_delta ?? null,
  }
}

function formatMetricNumber(value: number | null | undefined): string {
  if (value == null || Number.isNaN(value)) return '-'
  if (!Number.isInteger(value)) return value.toFixed(value < 10 ? 2 : 1)
  return formatNumber(value)
}

function formatPercent(value: number | null | undefined): string {
  if (value == null || Number.isNaN(value)) return '-'
  const ratio = value <= 1 ? value * 100 : value
  return `${ratio.toFixed(2)}%`
}

function formatErrorRate(successRate: number | null | undefined): string {
  if (successRate == null || Number.isNaN(successRate)) return '-'
  const rate = successRate <= 1 ? successRate * 100 : successRate
  return `${Math.max(0, 100 - rate).toFixed(2)}%`
}

function formatMs(value: number | null | undefined): string {
  if (value == null || Number.isNaN(value)) return '-'
  if (value >= 1000) return `${(value / 1000).toFixed(value >= 10_000 ? 1 : 2)}s`
  return `${Math.round(value)}ms`
}

function formatTps(value: number | null | undefined): string {
  if (value == null || Number.isNaN(value)) return '-'
  return `${value.toFixed(value < 10 ? 2 : value < 100 ? 1 : 0)} tps`
}

function formatPercentResource(value: number | null | undefined): string {
  if (value == null || Number.isNaN(value)) return '-'
  return `${value.toFixed(value < 10 ? 1 : 0)}%`
}

function formatBytes(value: number | null | undefined): string {
  if (value == null || Number.isNaN(value)) return '-'
  return formatByteSize(value)
}

function resourceToneClass(value: number | null | undefined, warning: number, critical: number): string {
  if (value == null || Number.isNaN(value)) return ''
  if (value >= critical) return 'text-red-600 dark:text-red-400'
  if (value >= warning) return 'text-amber-600 dark:text-amber-400'
  return 'text-green-600 dark:text-green-400'
}

function formatShortDate(value?: string | null): string {
  if (!value) return '-'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '-'
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function successRateClass(value: number | null | undefined): string {
  if (value == null || Number.isNaN(value)) return ''
  const rate = value <= 1 ? value * 100 : value
  if (rate >= 95) return 'text-green-600 dark:text-green-400'
  if (rate >= 80) return 'text-amber-600 dark:text-amber-400'
  return 'text-red-600 dark:text-red-400'
}

function average(values: Array<number | null | undefined>): number | null {
  const numbers = values.filter((value): value is number => typeof value === 'number' && Number.isFinite(value))
  if (!numbers.length) return null
  return numbers.reduce((total, value) => total + value, 0) / numbers.length
}

function sumSeries(field: string): number {
  return timeSeries.value.reduce((total, item) => total + numeric(item[field]), 0)
}

async function refreshAll() {
  const currentRequestId = ++requestId
  refreshing.value = true
  trendLoading.value = timeSeries.value.length === 0
  percentileLoading.value = percentiles.value.length === 0

  const params = timeRangeParams.value
  const results = await Promise.allSettled([
    usageApi.getUsageStats(params, { skipCache: true }),
    adminApi.getTimeSeries(params),
    adminApi.getPercentiles(params),
    adminApi.getProviderPerformance({
      ...params,
      granularity: params.granularity === 'hour' ? 'hour' : 'day',
      limit: 8,
      slow_threshold_ms: DEFAULT_SLOW_THRESHOLD_MS,
    }),
    adminApi.getErrorDistribution(params),
    monitoringApi.getSystemStatus(),
    monitoringApi.getResilienceStatus(),
    monitoringApi.getGatewayMetricsSummary(),
    cacheApi.getStats(),
    redisCacheApi.getCategories(),
    loadResourceSnapshot(),
  ])

  if (currentRequestId !== requestId) return

  const failed: string[] = []
  const [
    summaryResult,
    timeSeriesResult,
    percentilesResult,
    providerPerformanceResult,
    errorDistributionResult,
    systemStatusResult,
    resilienceResult,
    gatewayResult,
    cacheResult,
    redisResult,
    resourceResult,
  ] = results

  if (summaryResult.status === 'fulfilled') summaryStats.value = summaryResult.value
  else failed.push('统计摘要')

  if (timeSeriesResult.status === 'fulfilled') timeSeries.value = timeSeriesResult.value
  else failed.push('流量趋势')

  if (percentilesResult.status === 'fulfilled') percentiles.value = percentilesResult.value
  else failed.push('延迟百分位')

  if (providerPerformanceResult.status === 'fulfilled') providerPerformance.value = providerPerformanceResult.value
  else failed.push('上游性能')

  if (errorDistributionResult.status === 'fulfilled') errorDistribution.value = errorDistributionResult.value.distribution
  else failed.push('错误分类')

  if (systemStatusResult.status === 'fulfilled') systemStatus.value = systemStatusResult.value
  else failed.push('系统状态')

  if (resilienceResult.status === 'fulfilled') resilienceStatus.value = resilienceResult.value
  else failed.push('韧性状态')

  if (gatewayResult.status === 'fulfilled') gatewayMetrics.value = gatewayResult.value
  else failed.push('网关指标')

  if (cacheResult.status === 'fulfilled') cacheStats.value = cacheResult.value
  else failed.push('缓存统计')

  if (redisResult.status === 'fulfilled') redisCategories.value = redisResult.value
  else failed.push('Redis 分类')

  if (resourceResult.status === 'fulfilled') resourceSnapshot.value = resourceResult.value

  results.forEach((result, index) => {
    if (result.status === 'rejected') {
      log.error(`运维总览加载失败 ${index}`, result.reason)
    }
  })

  loadWarning.value = failed.length ? `部分数据加载失败：${failed.join('、')}` : null
  lastUpdatedAt.value = new Date().toISOString()
  refreshing.value = false
  trendLoading.value = false
  percentileLoading.value = false
}

const lastUpdatedLabel = computed(() => formatShortDate(lastUpdatedAt.value))

const totalRequests = computed(() => summaryStats.value?.total_requests ?? sumSeries('total_requests'))
const totalTokens = computed(() => summaryStats.value?.total_tokens ?? sumSeries('total_tokens'))
const totalCost = computed(() => summaryStats.value?.total_cost ?? sumSeries('total_cost'))
const avgResponseMs = computed(() => providerPerformance.value?.summary.avg_response_time_ms ?? (summaryStats.value?.avg_response_time ? summaryStats.value.avg_response_time * 1000 : null))
const avgFirstByteMs = computed(() => providerPerformance.value?.summary.avg_first_byte_time_ms ?? average(percentiles.value.map(item => item.p50_first_byte_time_ms)))
const avgOutputTps = computed(() => providerPerformance.value?.summary.avg_output_tps ?? null)
const windowSeconds = computed(() => {
  const start = timeRange.value.start_date ? new Date(timeRange.value.start_date).getTime() : NaN
  const end = timeRange.value.end_date ? new Date(timeRange.value.end_date).getTime() : NaN
  if (Number.isFinite(start) && Number.isFinite(end) && end > start) {
    return (end - start) / 1000
  }
  switch (timeRange.value.preset) {
    case 'today': return Math.max(1, (Date.now() - new Date().setHours(0, 0, 0, 0)) / 1000)
    case 'yesterday': return 86_400
    case 'last7days': return 7 * 86_400
    case 'last30days': return 30 * 86_400
    case 'last90days': return 90 * 86_400
    default: return Math.max(1, timeSeries.value.length * 86_400)
  }
})
const qps = computed(() => totalRequests.value / windowSeconds.value)
const rpm = computed(() => qps.value * 60)
const tokensPerMinute = computed(() => totalTokens.value / Math.max(1, windowSeconds.value / 60))
const slaRate = computed(() => providerPerformance.value?.summary.success_rate ?? null)

const kpiCards = computed<Array<{
  title: string
  value: string
  hint: string
  icon: Component
  iconClass: string
  valueClass?: string
}>>(() => [
  {
    title: 'QPS',
    value: qps.value.toFixed(qps.value < 10 ? 2 : 1),
    hint: `RPM ${rpm.value.toFixed(rpm.value < 100 ? 1 : 0)} · 请求 ${formatMetricNumber(totalRequests.value)}`,
    icon: Activity,
    iconClass: 'text-sky-500',
  },
  {
    title: '吞吐 Tokens',
    value: formatTokens(totalTokens.value),
    hint: `${formatMetricNumber(tokensPerMinute.value)} TPM`,
    icon: Zap,
    iconClass: 'text-amber-500',
  },
  {
    title: 'SLA',
    value: formatPercent(slaRate.value),
    hint: `错误率 ${formatErrorRate(slaRate.value)}`,
    icon: ShieldCheck,
    iconClass: 'text-emerald-500',
    valueClass: successRateClass(slaRate.value),
  },
  {
    title: '请求时长',
    value: formatMs(avgResponseMs.value),
    hint: `P99 ${formatMs(providerPerformance.value?.summary.p99_response_time_ms)}`,
    icon: Gauge,
    iconClass: 'text-violet-500',
  },
  {
    title: 'TTFT',
    value: formatMs(avgFirstByteMs.value),
    hint: `P99 首字 ${formatMs(providerPerformance.value?.summary.p99_first_byte_time_ms)}`,
    icon: Timer,
    iconClass: 'text-blue-500',
  },
  {
    title: '输出 TPS',
    value: formatTps(avgOutputTps.value),
    hint: `慢请求 ${formatMetricNumber(providerPerformance.value?.summary.slow_request_count)}`,
    icon: BarChart3,
    iconClass: 'text-cyan-500',
  },
  {
    title: '上游错误',
    value: formatMetricNumber(resilienceStatus.value?.error_statistics.total_errors),
    hint: `打开熔断 ${formatMetricNumber(resilienceStatus.value?.error_statistics.open_circuit_breakers)}`,
    icon: AlertTriangle,
    iconClass: 'text-red-500',
  },
  {
    title: '费用',
    value: formatCurrency(totalCost.value),
    hint: `缓存读 ${formatTokens(cacheStats.value?.affinity_stats.cache_hits ?? 0)} 次`,
    icon: CircleDollarSign,
    iconClass: 'text-green-500',
  },
])

const trafficChartData = computed<ChartData<'line'>>(() => ({
  labels: timeSeries.value.map(item => String(item.date ?? item.bucket ?? item.time ?? '')),
  datasets: [
    {
      label: '请求',
      data: timeSeries.value.map(item => numeric(item.total_requests ?? item.requests)),
      borderColor: 'rgb(14, 165, 233)',
      backgroundColor: 'rgba(14, 165, 233, 0.12)',
      tension: 0.25,
      pointRadius: 2,
      yAxisID: 'y',
    },
    {
      label: 'Tokens',
      data: timeSeries.value.map(item => numeric(item.total_tokens)),
      borderColor: 'rgb(245, 158, 11)',
      backgroundColor: 'rgba(245, 158, 11, 0.12)',
      tension: 0.25,
      pointRadius: 2,
      yAxisID: 'y1',
    },
  ],
}))

const trafficChartOptions: ChartOptions<'line'> = {
  interaction: { mode: 'index', intersect: false },
  scales: {
    y: { position: 'left' },
    y1: {
      position: 'right',
      grid: { drawOnChartArea: false },
    },
  },
}

const latencyChartData = computed<ChartData<'line'>>(() => ({
  labels: percentiles.value.map(item => item.date),
  datasets: [
    {
      label: 'P90 请求',
      data: percentiles.value.map(item => item.p90_response_time_ms ?? null),
      borderColor: 'rgb(124, 58, 237)',
      tension: 0.25,
      pointRadius: 2,
    },
    {
      label: 'P99 请求',
      data: percentiles.value.map(item => item.p99_response_time_ms ?? null),
      borderColor: 'rgb(239, 68, 68)',
      tension: 0.25,
      pointRadius: 2,
    },
    {
      label: 'P90 首字',
      data: percentiles.value.map(item => item.p90_first_byte_time_ms ?? null),
      borderColor: 'rgb(14, 165, 233)',
      tension: 0.25,
      pointRadius: 2,
    },
  ],
}))

const latencyChartOptions: ChartOptions<'line'> = {
  scales: {
    y: {
      ticks: {
        callback: value => formatMs(Number(value)),
      },
    },
  },
}

const errorDistributionChartData = computed<ChartData<'doughnut'>>(() => {
  const rows = errorDistribution.value.length
    ? errorDistribution.value
    : [{ category: '无错误', count: 1 }]
  return {
    labels: rows.map(item => item.category),
    datasets: [
      {
        data: rows.map(item => item.count),
        backgroundColor: [
          'rgba(239, 68, 68, 0.82)',
          'rgba(245, 158, 11, 0.82)',
          'rgba(14, 165, 233, 0.82)',
          'rgba(99, 102, 241, 0.82)',
          'rgba(34, 197, 94, 0.82)',
        ],
      },
    ],
  }
})

const errorDistributionOptions: ChartOptions<'doughnut'> = {
  plugins: {
    legend: {
      position: 'bottom',
    },
    tooltip: {
      callbacks: {
        label: context => `${context.label}: ${formatMetricNumber(context.raw as number)}`,
      },
    },
  },
}

const distributedGateVariant = computed<'warning' | 'outline'>(() => (
  gatewayMetrics.value?.distributed.unavailable ? 'warning' : 'outline'
))
const distributedGateText = computed(() => (
  gatewayMetrics.value?.distributed.unavailable ? '全局不可用' : '全局在线'
))
const currentActiveStreams = computed(() => gatewayMetrics.value?.tunnel.activeStreams ?? systemStatus.value?.tunnel.active_streams ?? null)
const currentProxyConnections = computed(() => gatewayMetrics.value?.tunnel.proxyConnections ?? systemStatus.value?.tunnel.proxy_connections ?? null)
const tunnelQueueRejectedTotal = computed(() => (
  (gatewayMetrics.value?.tunnel.outboundQueueRejectedFullTotal ?? 0)
  + (gatewayMetrics.value?.tunnel.outboundQueueRejectedClosedTotal ?? 0)
))
const tunnelSelectionPressureTotal = computed(() => (
  (gatewayMetrics.value?.tunnel.proxyConnectionCongestedTotal ?? 0)
  + (gatewayMetrics.value?.tunnel.selectionRetryTotal ?? 0)
  + (gatewayMetrics.value?.tunnel.selectionUnavailableTotal ?? 0)
))
const tunnelQueueUtilization = computed(() => {
  const depth = gatewayMetrics.value?.tunnel.outboundQueueDepthTotal
  const capacity = gatewayMetrics.value?.tunnel.outboundQueueCapacityTotal
  if (depth == null || capacity == null || capacity <= 0) return null
  return Math.max(0, Math.min(100, depth / capacity * 100))
})
const tunnelQueueUtilizationText = computed(() => (
  tunnelQueueUtilization.value == null ? '-' : `${Math.round(tunnelQueueUtilization.value)}%`
))
const tunnelQueueUtilizationWidth = computed(() => (
  tunnelQueueUtilization.value == null ? '0%' : `${tunnelQueueUtilization.value}%`
))

const slaValueClass = computed(() => successRateClass(providerPerformance.value?.summary.success_rate))
const errorRateValueClass = computed(() => {
  const rate = providerPerformance.value?.summary.success_rate
  if (rate == null) return ''
  const successPercent = rate <= 1 ? rate * 100 : rate
  const errorPercent = Math.max(0, 100 - successPercent)
  if (errorPercent <= 5) return 'text-green-600 dark:text-green-400'
  if (errorPercent <= 20) return 'text-amber-600 dark:text-amber-400'
  return 'text-red-600 dark:text-red-400'
})

const providerRows = computed(() => providerPerformance.value?.providers.slice(0, 8) ?? [])
const cacheHitRate = computed(() => cacheStats.value?.affinity_stats.cache_hit_rate ?? null)
const redisStatusText = computed(() => {
  if (!redisCategories.value) return '-'
  return redisCategories.value.available ? '在线' : '未启用'
})
const redisStatusClass = computed(() => {
  if (!redisCategories.value) return ''
  return redisCategories.value.available ? 'text-green-600 dark:text-green-400' : 'text-amber-600 dark:text-amber-400'
})
const redisCategoryRows = computed(() => (
  redisCategories.value?.categories
    .slice()
    .sort((left, right) => right.count - left.count)
    .slice(0, 6) ?? []
))
const recentErrors = computed<AdminMonitoringRecentError[]>(() => resilienceStatus.value?.recent_errors.slice(0, 6) ?? [])

const opsLinks = [
  { label: '性能分析', to: '/admin/performance-analysis', icon: Gauge },
  { label: '健康监控', to: '/admin/health-monitor', icon: ShieldCheck },
  { label: '使用记录', to: '/admin/usage', icon: ListChecks },
  { label: '缓存监控', to: '/admin/cache-monitoring', icon: Database },
  { label: '审计日志', to: '/admin/audit-logs', icon: AlertTriangle },
  { label: '异步任务', to: '/admin/async-tasks', icon: RefreshCw },
]

watch(timeRange, () => {
  void refreshAll()
}, { deep: true })

onMounted(() => {
  void refreshAll()
  refreshTimer = setInterval(() => {
    void refreshAll()
  }, AUTO_REFRESH_MS)
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
})
</script>
