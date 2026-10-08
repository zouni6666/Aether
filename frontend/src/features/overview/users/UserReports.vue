<template>
  <section
    class="space-y-4"
    data-user-reports
  >
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 class="text-sm font-semibold">
          {{ t('统计报表', 'Reports') }}
        </h2>
        <p
          v-if="!userId"
          class="mt-1 text-xs text-muted-foreground"
        >
          {{ t('模型用量与消费趋势为全站用量 · 包含独立余额 Key · 不受用户搜索影响', 'Model usage and consumption trends cover installation usage · Includes standalone balance keys · Independent of user search') }}
        </p>
      </div>
      <Button
        variant="outline"
        size="sm"
        :disabled="exporting"
        @click="exportCsv('breakdown', modelQuery)"
      >
        <Download class="mr-2 h-4 w-4" />{{ t('导出模型报表', 'Export model report') }}
      </Button>
    </div>
    <OverviewStatus :error="exportError" />
    <div class="grid min-w-0 gap-4 xl:grid-cols-2">
      <section
        class="min-w-0 rounded-2xl border bg-card p-5 shadow-sm"
        data-user-report="consumption"
      >
        <h3 class="text-sm font-medium">
          {{ t('消费趋势', 'Consumption trend') }}
        </h3>
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('计费消费 · USD', 'Billable consumption · USD') }}
        </p>
        <OverviewStatus
          :error="seriesError"
          @retry="refreshSeries"
        />
        <div
          v-if="series?.data.items.length"
          class="mt-5 h-64 min-w-0"
        >
          <LineChart :data="consumptionChart" />
        </div>
        <div
          v-else
          class="flex h-72 items-center justify-center text-sm text-muted-foreground"
        >
          {{ seriesLoading ? t('加载中', 'Loading') : seriesError ? t('消费报表暂不可用', 'Consumption report unavailable') : t('此时间范围暂无消费', 'No consumption in this period') }}
        </div>
      </section>
      <section
        class="min-w-0 rounded-2xl border bg-card p-5 shadow-sm"
        data-user-report="models"
      >
        <h3 class="text-sm font-medium">
          {{ t('模型用量', 'Model usage') }}
        </h3>
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('按 Token 用量排列 · 前 10 个模型', 'By token usage · Top 10 models') }}
        </p>
        <OverviewStatus
          :error="modelsError"
          @retry="refreshModels"
        />
        <div
          v-if="models?.data.items.length"
          class="mt-5 h-64 min-w-0"
        >
          <BarChart
            :data="modelChart"
            :stacked="false"
            :options="modelOptions"
          />
        </div>
        <div
          v-else
          class="flex h-72 items-center justify-center text-sm text-muted-foreground"
        >
          {{ modelsLoading ? t('加载中', 'Loading') : modelsError ? t('模型报表暂不可用', 'Model report unavailable') : t('此时间范围暂无模型使用', 'No model usage in this period') }}
        </div>
        <div
          v-if="models?.data.items.length"
          class="mt-4 max-h-64 overflow-auto"
        >
          <table class="w-full min-w-[420px] text-xs">
            <thead class="border-b text-muted-foreground">
              <tr>
                <th class="py-2 text-left font-normal">
                  {{ t('模型', 'Model') }}
                </th><th class="px-2 py-2 text-right font-normal">
                  {{ t('请求数', 'Requests') }}
                </th><th class="px-2 py-2 text-right font-normal">
                  Tokens
                </th><th class="py-2 text-right font-normal">
                  {{ t('消费', 'Consumption') }}
                </th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item in models.data.items"
                :key="item.id ?? '__unknown__'"
                class="border-b last:border-0"
              >
                <td class="max-w-40 break-words py-2">
                  {{ item.label || item.id || t('未知模型', 'Unknown model') }}
                </td><td class="px-2 py-2 text-right tabular-nums">
                  {{ count(item.request_count) }}
                </td><td class="px-2 py-2 text-right tabular-nums">
                  {{ count(item.total_tokens) }}
                </td><td class="whitespace-nowrap py-2 text-right tabular-nums">
                  {{ money(item.billable_amount) }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </div>
    <slot name="additional" />
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ChartOptions } from 'chart.js'
import { Download } from 'lucide-vue-next'
import { Button } from '@/components/ui'
import { overviewApi } from '@/api/overview'
import LineChart from '@/components/charts/LineChart.vue'
import BarChart from '@/components/charts/BarChart.vue'
import OverviewStatus from '../components/OverviewStatus.vue'
import { useOverviewQuery } from '../query'
import { useOverviewRequest } from '../useOverviewRequest'
import { useOverviewExport } from '../useOverviewExport'
import { useOverviewI18n } from '../i18n'
import { count, money, timestamp } from '../format'
const props = defineProps<{ userId?: string; revision: number }>()
const { t } = useOverviewI18n()
const { range, relativePreset } = useOverviewQuery()
const requestQuery = computed(() => ({ ...range.value, user_id: props.userId, granularity: Date.parse(range.value.to) - Date.parse(range.value.from) <= 48 * 3_600_000 ? 'hour' as const : 'day' as const }))
const modelQuery = computed(() => ({ ...requestQuery.value, group_by: 'model', sort: 'total_tokens', order: 'desc' as const, limit: 10, offset: 0 }))
const scope = computed(() => JSON.stringify({ user: props.userId, ...(relativePreset.value ? { preset: relativePreset.value, timezone: range.value.timezone } : range.value) }))
const { data: series, loading: seriesLoading, error: seriesError, refresh: refreshSeries } = useOverviewRequest(() => JSON.stringify([requestQuery.value, props.revision]), signal => overviewApi.timeseries(requestQuery.value, signal), { scopeKey: scope })
const { data: models, loading: modelsLoading, error: modelsError, refresh: refreshModels } = useOverviewRequest(() => JSON.stringify([modelQuery.value, props.revision]), signal => overviewApi.breakdown(modelQuery.value, signal), { scopeKey: scope })
const { exporting, exportError, exportCsv } = useOverviewExport()
const consumptionChart = computed(() => ({
  labels: (series.value?.data.items || []).map(item => timestamp(item.bucket_start, range.value.timezone)),
  datasets: [{ label: t('消费 (USD)', 'Consumption (USD)'), data: (series.value?.data.items || []).map(item => {
    const value = item.billable_amount?.value
    return value == null || !Number.isFinite(Number(value)) ? null : Number(value)
  }), borderColor: '#cb694b', backgroundColor: '#cb694b', pointRadius: 2, borderWidth: 2, tension: 0.15 }],
}))
const modelChart = computed(() => ({ labels: (models.value?.data.items || []).map(item => item.label || item.id || t('未知模型', 'Unknown model')), datasets: [{ label: 'Tokens', data: (models.value?.data.items || []).map(item => item.total_tokens), backgroundColor: '#cb694b', borderRadius: 4, maxBarThickness: 24 }] }))
const modelOptions: ChartOptions<'bar'> = { indexAxis: 'y', plugins: { legend: { display: false } }, scales: { x: { beginAtZero: true }, y: { grid: { display: false } } } }
</script>
