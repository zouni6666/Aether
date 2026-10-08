<template>
  <div class="space-y-6">
    <section
      class="space-y-4"
      data-operations-details
    >
      <div class="flex flex-wrap items-center justify-between gap-2">
        <h2 class="text-sm font-semibold">
          {{ t('请求分析', 'Request analysis') }}
        </h2>
        <p class="text-xs text-muted-foreground">
          {{ t('所选时段 · 失败请求多的优先展示', 'Selected range · Most failures first') }}
        </p>
      </div>
      <OverviewStatus
        :loading="loading && !data"
        :error="error"
        @retry="refresh"
      />
      <div
        v-if="data"
        class="grid grid-cols-1 items-start gap-4 min-[1920px]:grid-cols-2"
      >
        <PerformanceAnalysisCard
          :title="t('提供商分析', 'Provider analysis')"
          dimension="provider"
          :rows="providerRows"
          :total-requests="data.data.summary.request_count"
          :total-failures="data.data.summary.failed_request_count ?? null"
          :live-window-seconds="activity?.observed_window_seconds"
        />
        <PerformanceAnalysisCard
          :title="t('模型分析', 'Model analysis')"
          dimension="model"
          :rows="modelRows"
          :total-requests="data.data.summary.request_count"
          :total-failures="data.data.summary.failed_request_count ?? null"
          :live-window-seconds="activity?.observed_window_seconds"
        />
      </div>
    </section>
    <section
      v-if="data"
      class="space-y-4"
      data-operations-charts
    >
      <h2 class="text-sm font-semibold">
        {{ t('趋势与分布', 'Trends and distribution') }}
      </h2>
      <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <Card class="min-w-0 space-y-3 p-4 sm:p-5">
          <h3 class="text-sm font-semibold">
            {{ t('请求与失败趋势', 'Request and failure trend') }}
          </h3>
          <div
            v-if="data.data.timeseries.length"
            class="h-64"
          >
            <LineChart :data="requestChart" />
          </div>
          <p
            v-else
            class="flex h-64 items-center justify-center text-sm text-muted-foreground"
          >
            {{ t('此时间范围暂无请求', 'No requests in this range') }}
          </p>
        </Card>
        <Card class="min-w-0 space-y-3 p-4 sm:p-5">
          <h3 class="text-sm font-semibold">
            {{ t('响应时间趋势', 'Response latency trend') }}
          </h3>
          <div
            v-if="data.data.timeseries.length"
            class="h-64"
          >
            <LineChart :data="latencyChart" />
          </div>
          <p
            v-else
            class="flex h-64 items-center justify-center text-sm text-muted-foreground"
          >
            {{ t('此时间范围暂无时延样本', 'No latency samples in this range') }}
          </p>
        </Card>
        <Card class="min-w-0 space-y-3 p-4 sm:p-5 lg:col-span-2">
          <div class="flex items-center justify-between gap-3">
            <h3 class="text-sm font-semibold">
              {{ t('失败原因分布', 'Failure reasons') }}
            </h3>
            <RouterLink
              :to="link('/admin/usage', { status: 'failed' })"
              class="inline-flex items-center gap-1 text-xs text-primary hover:underline"
            >
              {{ t('失败请求', 'Failed requests') }}<ArrowUpRight class="h-3.5 w-3.5" />
            </RouterLink>
          </div>
          <div
            v-if="data.data.errors.length"
            class="min-w-0"
            :style="{ height: `${Math.max(180, Math.min(400, data.data.errors.length * 36))}px` }"
          >
            <BarChart
              :data="failureChart"
              :options="failureChartOptions"
              :stacked="false"
            />
          </div>
          <p
            v-else
            class="flex h-32 items-center justify-center text-sm text-muted-foreground"
          >
            {{ t('此时间范围未记录失败请求', 'No failed requests recorded in this range') }}
          </p>
        </Card>
      </div>
    </section>
    <details
      class="rounded-2xl border bg-card"
      data-operations-diagnostics
    >
      <summary class="cursor-pointer px-4 py-3 text-sm font-medium sm:px-5">
        {{ t('技术诊断', 'Technical diagnostics') }}
      </summary>
      <div class="space-y-6 border-t p-4 sm:p-5">
        <section
          v-if="data"
          class="space-y-4"
        >
          <div class="flex items-center justify-between gap-3">
            <h3 class="text-sm font-semibold">
              {{ t('性能诊断', 'Performance diagnostics') }}
            </h3>
            <Badge variant="outline">
              {{ t('所选时段', 'Selected range') }}
            </Badge>
          </div>
          <OverviewMetrics
            :metrics="data.data.summary"
            mode="performance"
          />
          <OverviewStatus :meta="data.meta" />
        </section>
        <slot name="diagnostics" />
      </div>
    </details>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import type { ChartOptions } from 'chart.js'
import { ArrowUpRight } from 'lucide-vue-next'
import { Badge, Card } from '@/components/ui'
import BarChart from '@/components/charts/BarChart.vue'
import LineChart from '@/components/charts/LineChart.vue'
import { overviewApi, type OverviewExecutionActivity, type OverviewQuery } from '@/api/overview'
import OverviewStatus from '../components/OverviewStatus.vue'
import OverviewMetrics from '../components/OverviewMetrics.vue'
import PerformanceAnalysisCard from './PerformanceAnalysisCard.vue'
import { mergeLiveAnalysis, type AnalysisRow } from './analysisHelpers'
import { useOverviewQuery } from '../query'
import { useOverviewRequest } from '../useOverviewRequest'
import { useOverviewI18n } from '../i18n'
import { timestamp } from '../format'

const props = defineProps<{ revision: number; activity?: OverviewExecutionActivity | null }>()
const { t } = useOverviewI18n()
const { range, relativePreset, link } = useOverviewQuery('today')
// This overview always analyses all requests. Old diagnostic filters in links
// must not silently hide providers or models from the automatic reports.
const analysisQuery = computed<OverviewQuery>(() => ({
  ...range.value,
  granularity: Date.parse(range.value.to) - Date.parse(range.value.from) <= 48 * 3_600_000 ? 'hour' : 'day',
}))
const scopeKey = computed(() => JSON.stringify(relativePreset.value
  ? { timezone: range.value.timezone, granularity: analysisQuery.value.granularity, preset: relativePreset.value }
  : analysisQuery.value))
const { data, loading, error, refresh } = useOverviewRequest(
  () => JSON.stringify([analysisQuery.value, props.revision]),
  signal => overviewApi.performance(analysisQuery.value, signal),
  { scopeKey },
)
const historicalProviders = computed<AnalysisRow[] | null>(() => data.value?.data.providers?.providers.map(row => ({
  id: row.provider_id || null,
  label: row.provider || t('未归属提供商', 'Unattributed provider'),
  requestCount: row.request_count,
  failedCount: row.error_count ?? null,
  successRate: row.success_rate ?? null,
  firstByteMs: row.avg_first_byte_time_ms ?? null,
  outputTps: row.avg_output_tps ?? null,
})) ?? null)
const historicalModels = computed<AnalysisRow[] | null>(() => data.value?.data.models?.map(row => ({
  id: row.model || null,
  label: row.model || t('未记录模型', 'Unrecorded model'),
  requestCount: row.request_count,
  failedCount: row.error_count ?? null,
  successRate: row.success_rate ?? null,
  firstByteMs: row.avg_first_byte_time_ms ?? null,
  outputTps: row.avg_output_tps ?? null,
})) ?? null)
const completeActivity = computed(() => props.activity?.coverage === 'complete' && props.activity.scope.kind === 'node' ? props.activity : null)
const providerRows = computed(() => mergeLiveAnalysis(historicalProviders.value,
  completeActivity.value?.providers.map(row => ({ id: row.provider_id || null, label: row.provider || row.provider_id || t('未归属提供商', 'Unattributed provider'), concurrency: row.current_concurrency, rpm: row.requests_per_minute })) ?? null))
const modelRows = computed(() => mergeLiveAnalysis(historicalModels.value,
  completeActivity.value?.models.map(row => ({ id: row.model || null, label: row.model || t('未记录模型', 'Unrecorded model'), concurrency: row.current_concurrency, rpm: row.requests_per_minute })) ?? null))
const chartLabels = computed(() => data.value?.data.timeseries.map(point => timestamp(point.bucket_start, range.value.timezone)) || [])
const requestChart = computed(() => ({
  labels: chartLabels.value,
  datasets: [
    { label: t('请求数', 'Requests'), data: data.value?.data.timeseries.map(point => point.request_count) || [], borderColor: '#0d9488', pointRadius: 1, borderWidth: 2, tension: 0.1 },
    { label: t('失败请求', 'Failed requests'), data: data.value?.data.timeseries.map(point => point.failed_request_count) || [], borderColor: '#db7d60', pointRadius: 1, borderWidth: 2, tension: 0.1 },
  ],
}))
const latencyChart = computed(() => ({
  labels: chartLabels.value,
  datasets: (['p50', 'p95', 'p99'] as const).map((key, index) => ({ label: `${key.toUpperCase()} (ms)`, data: data.value?.data.timeseries.map(point => point.latency_ms?.[key] ?? null) || [], borderColor: ['#0d9488', '#d97706', '#e11d48'][index], pointRadius: 1, borderWidth: 2, tension: 0.1 })),
}))
const failureChart = computed(() => {
  const failures = [...(data.value?.data.errors || [])].sort((a, b) => b.count - a.count)
  return {
    labels: failures.map(row => row.reason),
    datasets: [{ label: t('失败请求', 'Failed requests'), data: failures.map(row => row.count), backgroundColor: '#db7d60', borderRadius: 4, maxBarThickness: 24 }],
  }
})
const failureChartOptions: ChartOptions<'bar'> = {
  indexAxis: 'y',
  plugins: { legend: { display: false } },
  scales: {
    x: { beginAtZero: true, ticks: { precision: 0 }, grid: { color: 'rgba(156, 163, 175, 0.1)' } },
    y: { grid: { display: false } },
  },
}
</script>
