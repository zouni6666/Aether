<template>
  <dl class="grid grid-cols-2 gap-3 xl:grid-cols-4">
    <div
      v-for="item in items"
      :key="item.key"
      :data-metric="item.key"
      class="min-w-0 rounded-lg border bg-card p-3 sm:p-5"
    >
      <dt class="flex min-h-5 items-start justify-between gap-2 text-xs font-medium text-muted-foreground">
        <span class="break-words">{{ item.label }}</span>
        <component
          :is="item.icon"
          class="h-4 w-4 shrink-0"
          :class="item.color"
        />
      </dt>
      <dd class="mt-4 min-h-7 font-semibold tabular-nums">
        <MetricValue :value="item.value" />
      </dd>
      <p
        v-if="item.note"
        class="mt-1 text-xs text-amber-700 dark:text-amber-400"
      >
        {{ item.note }}
      </p>
      <div class="mt-3 border-t pt-3 text-xs text-muted-foreground">
        <div class="flex flex-wrap items-baseline justify-between gap-x-2 gap-y-1">
          <dt>{{ item.totalLabel }}</dt>
          <dd class="break-all font-medium tabular-nums text-foreground">
            {{ item.totalValue }}
          </dd>
        </div>
        <p
          v-if="item.totalNote"
          class="mt-1 text-amber-700 dark:text-amber-400"
        >
          {{ item.totalNote }}
        </p>
      </div>
    </div>
  </dl>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Activity, Coins, Users, Zap } from 'lucide-vue-next'
import type { OverviewDashboard } from '@/api/overview'
import MetricValue from '../components/MetricValue.vue'
import { count, money } from '../format'
import { useOverviewI18n } from '../i18n'
import { amountStatus, amountValue } from './amount'

const props = defineProps<{ data: OverviewDashboard }>()
const { t } = useOverviewI18n()
const items = computed(() => {
  const today = props.data.today.data
  const total = props.data.total.data
  const cost = (amount: typeof today.billable_amount) => amountValue(amount) === null ? '-' : money(amount)
  return [
    { key: 'requests', label: t('今日请求', 'Today\'s requests'), value: count(today.request_count), totalLabel: t('总请求', 'Total requests'), totalValue: count(total.request_count), icon: Activity, color: 'text-blue-600 dark:text-blue-400' },
    { key: 'tokens', label: t('今日 Token', 'Today\'s tokens'), value: count(today.total_tokens), totalLabel: t('总 Token', 'Total tokens'), totalValue: count(total.total_tokens), icon: Zap, color: 'text-violet-600 dark:text-violet-400' },
    { key: 'cost', label: t('今日费用', 'Today\'s cost'), value: cost(today.billable_amount), note: amountStatus(today.billable_amount, t), totalLabel: t('总费用', 'Total cost'), totalValue: cost(total.billable_amount), totalNote: amountStatus(total.billable_amount, t), icon: Coins, color: 'text-emerald-600 dark:text-emerald-400' },
    { key: 'employees', label: t('今日活跃成员', 'Active members today'), value: count(today.usage_active_users), totalLabel: t('启用成员', 'Enabled members'), totalValue: count(today.enabled_users ?? total.enabled_users), icon: Users, color: 'text-amber-600 dark:text-amber-400' },
  ]
})
</script>
