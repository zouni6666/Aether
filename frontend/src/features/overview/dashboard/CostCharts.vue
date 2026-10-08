<template>
  <div class="grid min-w-0 grid-cols-1 gap-6 lg:grid-cols-2">
    <section class="min-w-0 space-y-3">
      <h2 class="text-sm font-semibold">
        {{ t('每日模型费用', 'Daily model cost') }}
      </h2>
      <div class="h-[280px] min-w-0">
        <BarChart
          v-if="modelChart.datasets.length && hasModelAmount"
          :data="modelChart"
          :options="modelOptions"
        />
        <div
          v-else
          class="flex h-full items-center justify-center rounded-md border border-dashed text-sm text-muted-foreground"
        >
          {{ emptyCostLabel }}
        </div>
      </div>
    </section>
    <section class="min-w-0 space-y-3">
      <h2 class="text-sm font-semibold">
        {{ t('提供商费用分布', 'Provider cost distribution') }}
      </h2>
      <div class="h-[280px] min-w-0">
        <DoughnutChart
          v-if="slices.length"
          :data="providerChart"
          :options="providerOptions"
        />
        <div
          v-else
          class="flex h-full items-center justify-center rounded-md border border-dashed text-sm text-muted-foreground"
        >
          {{ emptyCostLabel }}
        </div>
      </div>
    </section>
    <p
      v-if="partial"
      class="text-xs text-amber-700 lg:col-span-2 dark:text-amber-400"
    >
      {{ t('费用为已知小计，分布占比按已知金额计算', 'Costs are known subtotals; distribution shares use known amounts') }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ChartData, ChartOptions } from 'chart.js'
import type { OverviewDashboardCharts } from '@/api/overview'
import BarChart from '@/components/charts/BarChart.vue'
import DoughnutChart from '@/components/charts/DoughnutChart.vue'
import { money } from '../format'
import { useOverviewI18n } from '../i18n'
import { amountValue } from './amount'
import { chartColors, chartDate, modelDatasets, providerSlices } from './charts'

const props = defineProps<{ data: OverviewDashboardCharts; timezone: string }>()
const { t } = useOverviewI18n()
const currency = computed(() => props.data.summary.billable_amount.currency)
const formatCost = (value: number) => money({ value: String(value), currency: currency.value, basis: 'billable', status: 'known' })
const partial = computed(() => {
  const rows = [props.data.summary, ...props.data.models, ...props.data.providers]
  return rows.some(row => amountValue(row.billable_amount) !== null)
    && rows.some(row => row.billable_amount.status !== 'known')
})
const emptyCostLabel = computed(() => props.data.summary.request_count === 0 ? t('暂无使用数据', 'No usage data')
  : amountValue(props.data.summary.billable_amount) === null ? t('费用尚未确认', 'Cost not yet known') : t('此周期暂无计费费用', 'No billable cost in this period'))
const modelChart = computed<ChartData<'bar'>>(() => ({
  labels: props.data.series.map(day => chartDate(day.bucket_start, props.timezone)),
  datasets: modelDatasets(props.data, t('未知模型', 'Unknown model'), t('其他模型', 'Other models')),
}))
const hasModelAmount = computed(() => modelChart.value.datasets.some(dataset => dataset.data.some(value => typeof value === 'number' && value !== 0)))
const modelOptions = computed<ChartOptions<'bar'>>(() => ({
  responsive: true, maintainAspectRatio: false,
  interaction: { mode: 'index', intersect: false },
  scales: {
    x: { stacked: true, grid: { display: false }, ticks: { font: { size: 10 }, maxRotation: 0 } },
    y: { stacked: true, ticks: { font: { size: 10 } }, title: { display: true, text: `${t('费用', 'Cost')} (${currency.value})`, font: { size: 10 } } },
  },
  plugins: {
    legend: { position: 'bottom', labels: { font: { size: 10 }, boxWidth: 10, padding: 10,
      generateLabels: chart => chart.data.datasets.map((dataset, index) => ({ text: shorten(dataset.label || ''), fillStyle: chartColors[index % chartColors.length], hidden: !chart.isDatasetVisible(index), datasetIndex: index })),
    } },
    tooltip: { callbacks: {
      label: context => `${context.dataset.label}: ${typeof context.raw === 'number' ? formatCost(context.raw) : '-'}`,
      footer: items => `${t('已显示合计', 'Visible subtotal')}: ${formatCost(items.reduce((sum, item) => sum + (typeof item.raw === 'number' ? item.raw : 0), 0))}`,
    } },
  },
}))
const slices = computed(() => providerSlices(props.data.providers, t('未知提供商', 'Unknown provider'), t('其他提供商', 'Other providers')))
const providerChart = computed<ChartData<'doughnut'>>(() => ({
  labels: slices.value.map(slice => slice.label),
  datasets: [{ data: slices.value.map(slice => slice.value), backgroundColor: slices.value.map((_, index) => chartColors[index % chartColors.length]), borderWidth: 2, borderColor: 'rgba(255,255,255,0.15)' }],
}))
const providerOptions = computed<ChartOptions<'doughnut'>>(() => ({
  responsive: true, maintainAspectRatio: false, cutout: '60%',
  plugins: {
    legend: { position: 'bottom', labels: { font: { size: 10 }, boxWidth: 10, padding: 10,
      generateLabels: chart => slices.value.map((slice, index) => ({ text: shorten(slice.label), fillStyle: chartColors[index % chartColors.length], hidden: !chart.getDataVisibility(index), index })),
    } },
    tooltip: { callbacks: { label: context => {
      const value = typeof context.raw === 'number' ? context.raw : 0
      const total = slices.value.reduce((sum, slice) => sum + slice.value, 0)
      return `${context.label}: ${formatCost(value)} (${total > 0 ? (value / total * 100).toFixed(1) : '0'}%)`
    } } },
  },
}))
function shorten(value: string) { return value.length > 25 ? `${value.slice(0, 24)}...` : value }
</script>
