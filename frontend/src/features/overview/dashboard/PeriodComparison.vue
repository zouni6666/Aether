<template>
  <section class="space-y-3 border-b pb-4">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <h2 class="text-sm font-semibold">
        {{ t('较前一等长时段', 'Versus previous equal period') }}
      </h2><span
        v-if="data"
        class="text-xs text-muted-foreground"
      >{{ timestamp(data.meta.range.from, range.timezone) }} - {{ timestamp(data.meta.range.to, range.timezone) }}</span>
    </div>
    <p
      v-if="error"
      role="alert"
      class="text-xs text-destructive"
    >
      {{ t('对比数据暂不可用', 'Comparison unavailable') }}
    </p><p
      v-else-if="loading && !data"
      class="text-xs text-muted-foreground"
    >
      {{ t('加载中', 'Loading') }}
    </p>
    <dl
      v-if="data"
      class="grid grid-cols-2 gap-4 sm:grid-cols-4"
    >
      <div
        v-for="item in changes"
        :key="item.label"
      >
        <dt class="text-xs text-muted-foreground">
          {{ item.label }}
        </dt><dd class="mt-1 text-sm font-semibold tabular-nums">
          {{ item.value }}
        </dd>
      </div>
    </dl>
    <p
      v-if="data"
      class="text-xs text-muted-foreground"
    >
      {{ t('对比更新', 'Comparison updated') }} {{ timestamp(data.meta.generated_at, range.timezone) }} · {{ data.meta.coverage.status === 'complete' ? t('数据完整', 'Complete') : t('部分数据', 'Partial data') }}
    </p>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { overviewApi, type OverviewMetrics, type OverviewRange } from '@/api/overview'
import { useOverviewRequest } from '../useOverviewRequest'
import { useOverviewI18n } from '../i18n'
import { timestamp } from '../format'
const props = defineProps<{ current: OverviewMetrics; range: OverviewRange; revision: number }>()
const { t } = useOverviewI18n()
const previousRange = computed(() => ({ from: new Date(Date.parse(props.range.from) - (Date.parse(props.range.to) - Date.parse(props.range.from))).toISOString(), to: props.range.from, timezone: props.range.timezone }))
const { data, loading, error } = useOverviewRequest(() => [previousRange.value, props.revision], signal => overviewApi.summary(previousRange.value, signal), { scopeKey: () => JSON.stringify(previousRange.value) })
function delta(current: number | null | undefined, previous: number | null | undefined) {
  if (current == null || previous == null) return '-'
  if (!previous) return current ? t('前期为零', 'Previous period zero') : '0.0%'
  const value = (current - previous) / previous * 100
  return `${value > 0 ? '+' : ''}${value.toFixed(1)}%`
}
const changes = computed(() => {
  const previous = data.value?.data
  return [{ label: t('请求数', 'Requests'), value: delta(props.current.request_count, previous?.request_count) }, { label: 'Tokens', value: delta(props.current.total_tokens, previous?.total_tokens) }, { label: t('计费消耗', 'Billable consumption'), value: delta(props.current.billable_amount.value == null ? null : Number(props.current.billable_amount.value), previous?.billable_amount.value == null ? null : Number(previous.billable_amount.value)) }, { label: t('有使用成员', 'Members with usage'), value: delta(props.current.usage_active_users, previous?.usage_active_users) }]
})
</script>
