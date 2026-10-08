<template>
  <section class="min-w-0 space-y-3">
    <div class="flex items-center justify-between gap-3">
      <h2 class="text-sm font-semibold">
        {{ title || t('使用趋势', 'Usage trend') }}
      </h2><select
        v-model="metric"
        :aria-label="t('趋势指标', 'Trend metric')"
        class="h-8 rounded-md border bg-background px-2 text-xs"
      >
        <option value="requests">
          {{ t('请求数', 'Requests') }}
        </option><option value="tokens">
          Tokens
        </option><option value="billable">
          {{ t('计费消耗', 'Billable consumption') }}
        </option>
      </select>
    </div>
    <div
      v-if="!points.length"
      class="flex h-64 items-center justify-center text-sm text-muted-foreground"
    >
      {{ t('此时间范围暂无数据', 'No data in this range') }}
    </div>
    <div
      v-else
      class="h-64 min-w-0"
    >
      <LineChart :data="chart" />
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { OverviewSeriesPoint } from '@/api/overview'
import LineChart from '@/components/charts/LineChart.vue'
import { timestamp } from '../format'
import { useOverviewI18n } from '../i18n'
const props = defineProps<{ points: OverviewSeriesPoint[]; timezone: string; title?: string }>()
const { t } = useOverviewI18n()
const metric = ref('requests')
const chart = computed(() => ({ labels: props.points.map(p => timestamp(p.bucket_start, props.timezone)), datasets: [{
  label: metric.value === 'tokens' ? 'Tokens' : metric.value === 'billable' ? t('计费消耗 (USD)', 'Billable consumption (USD)') : t('请求数', 'Requests'),
  data: props.points.map(p => metric.value === 'tokens' ? p.total_tokens : metric.value === 'billable' ? (p.billable_amount?.value == null ? null : Number(p.billable_amount.value)) : p.request_count),
  borderColor: '#0d9488', backgroundColor: '#0d9488', pointRadius: props.points.length > 60 ? 0 : 2, borderWidth: 2, tension: 0.15,
}] }))
</script>
