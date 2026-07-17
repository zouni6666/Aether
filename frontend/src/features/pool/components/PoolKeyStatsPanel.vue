<template>
  <div
    v-if="cycle && cycleMetricRows.length > 0"
    :class="cycleContainerClass"
    :data-testid="variant === 'desktop' ? 'pool-stats-cycle-text' : 'pool-mobile-stats-cycle-text'"
  >
    <div
      v-for="row in cycleMetricRows"
      :key="`${row.key}-${variant}-cycle-row`"
      class="truncate"
      :title="`${row.label} ${row.valueText}`"
    >
      <span
        :data-testid="variant === 'desktop' ? `pool-stats-cycle-${row.key}` : undefined"
      >
        {{ row.label }} {{ row.valueText }}
      </span>
    </div>
  </div>

  <div
    v-else-if="cycle"
    :class="cycleContainerClass"
    :data-testid="variant === 'desktop' ? 'pool-stats-cycle-empty' : 'pool-mobile-stats-cycle-empty'"
  >
    <div class="flex min-h-16 items-center justify-center text-muted-foreground">
      —
    </div>
  </div>

  <div
    v-else
    :class="accountContainerClass"
    :data-testid="variant === 'desktop' ? 'pool-stats-account-total' : undefined"
  >
    <div
      class="invisible h-4"
      aria-hidden="true"
    >
      -
    </div>
    <div
      v-for="metric in accountMetrics"
      :key="`${metric.key}-${variant}-account-total`"
      :class="accountMetricRowClass"
    >
      <span class="text-muted-foreground truncate">{{ metric.label }}</span>
      <span
        :class="accountValueClass"
        :title="metric.value"
      >
        {{ metric.value }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type {
  PoolCodexCycleStatsGroup,
  PoolStatsMetric,
  PoolStatsMetricKey,
} from '@/features/pool/utils/poolStatsDisplay'

const props = withDefaults(defineProps<{
  cycle: boolean
  cycleGroups: PoolCodexCycleStatsGroup[]
  accountMetrics: PoolStatsMetric[]
  variant?: 'desktop' | 'mobile'
}>(), {
  variant: 'desktop',
})

const CYCLE_METRIC_KEYS: PoolStatsMetricKey[] = ['request_count', 'total_tokens', 'total_cost_usd']
const CYCLE_METRIC_LABELS: Record<PoolStatsMetricKey, string> = {
  request_count: '请求',
  total_tokens: 'Token',
  total_cost_usd: '费用',
}

function missingMetric(key: PoolStatsMetricKey): PoolStatsMetric {
  return {
    key,
    label: CYCLE_METRIC_LABELS[key],
    value: '—',
    missing: true,
    numericValue: null,
  }
}

function metricForGroup(
  group: PoolCodexCycleStatsGroup | undefined,
  key: PoolStatsMetricKey,
): PoolStatsMetric {
  return group?.metrics.find(metric => metric.key === key) ?? missingMetric(key)
}

const cycleMetricRows = computed(() => {
  const smallGroup = props.cycleGroups.length > 1 ? props.cycleGroups[0] : undefined
  const largeGroup = props.cycleGroups.at(-1)
  if (!largeGroup) return []

  return CYCLE_METRIC_KEYS.map((key) => {
    const smallMetric = metricForGroup(smallGroup, key)
    const largeMetric = metricForGroup(largeGroup, key)
    const hasComparison = Boolean(smallGroup)
    return {
      key,
      label: CYCLE_METRIC_LABELS[key],
      valueText: hasComparison ? `${smallMetric.value}/${largeMetric.value}` : largeMetric.value,
    }
  })
})

const cycleContainerClass = computed(() => [
  'mx-auto w-[132px] space-y-0.5 text-[10px] leading-4 text-foreground/90 tabular-nums',
  props.variant === 'mobile' ? 'py-0.5' : '',
].filter(Boolean).join(' '))

const accountContainerClass = computed(() => props.variant === 'desktop'
  ? 'grid min-h-16 w-[188px] grid-rows-4 gap-0 mx-auto text-[10px] leading-4'
  : ''
)

const accountMetricRowClass = computed(() => props.variant === 'desktop'
  ? 'grid grid-cols-[64px_124px] items-center'
  : 'grid h-4 w-[188px] grid-cols-[64px_124px] items-center text-left'
)

const accountValueClass = computed(() => [
  'min-w-0 truncate text-center text-foreground/90',
  props.variant === 'desktop' ? 'tabular-nums' : 'font-medium',
].join(' '))
</script>
