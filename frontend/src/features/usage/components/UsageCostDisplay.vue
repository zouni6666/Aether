<template>
  <div
    class="flex flex-col items-end gap-0.5"
    :class="compact ? 'text-[10px]' : 'text-xs'"
  >
    <template v-if="record.usage_available !== false && record.usage_pricing_available !== false">
      <span
        data-usage-cost="base"
        class="text-primary"
        :class="compact ? 'text-sm font-semibold leading-5' : 'font-medium'"
      >{{ formatCurrency(record.cost || 0) }}</span>
      <span
        v-if="billing.cost !== null"
        data-usage-cost="routing-group"
        class="whitespace-nowrap text-muted-foreground"
        title="实际扣费"
      >{{ formatCurrency(billing.cost) }}</span>
      <span
        v-if="showKeyCost"
        data-usage-cost="provider-key"
        class="text-muted-foreground"
        :title="`提供商 Key 成本（Key 倍率 ${record.rate_multiplier}×）`"
      >{{ formatCurrency(record.actual_cost ?? 0) }}</span>
    </template>
    <span
      v-else-if="record.usage_available === false"
      data-usage-unavailable="cost"
      class="text-muted-foreground"
      :class="compact ? 'text-sm font-medium leading-5' : ''"
      title="上游未提供可验证的 token/费用用量"
    >不可用</span>
    <span
      v-else
      data-usage-unpriced="cost"
      class="text-muted-foreground"
      :class="compact ? 'text-sm font-medium leading-5' : ''"
      title="token 用量可验证，但当前计价规则不支持该音频用量分项"
    >未计价</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { UsageRecord } from '../types'
import { formatCurrency } from '@/utils/format'
import { resolveUsageBilling } from '../utils/usageBilling'

const props = defineProps<{
  record: UsageRecord
  showActualCost: boolean
  compact?: boolean
}>()

const billing = computed(() => resolveUsageBilling(props.record))
const showKeyCost = computed(() => props.showActualCost
  && typeof props.record.actual_cost === 'number' && Number.isFinite(props.record.actual_cost)
  && typeof props.record.rate_multiplier === 'number' && Number.isFinite(props.record.rate_multiplier)
  && props.record.rate_multiplier >= 0 && props.record.rate_multiplier !== 1)
</script>
