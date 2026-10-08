<template>
  <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
    <section
      class="rounded-2xl border bg-card p-5 shadow-sm"
      data-user-summary="consumption"
    >
      <p class="text-xs font-medium text-muted-foreground">
        {{ t('消费', 'Consumption') }}
      </p>
      <p class="mt-3 break-all text-2xl font-semibold tabular-nums">
        {{ money(summary?.billable_amount) }}
      </p>
      <p class="mt-3 flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground tabular-nums">
        <span class="whitespace-nowrap">{{ t('额度', 'Quota') }} {{ money(summary?.quota_covered_amount) }}</span>
        <span class="whitespace-nowrap">{{ t('余额', 'Balance') }} {{ money(summary?.wallet_debit_amount) }}</span>
      </p>
    </section>
    <section
      class="rounded-2xl border bg-card p-5 shadow-sm"
      data-user-summary="recharge"
    >
      <p class="text-xs font-medium text-muted-foreground">
        {{ t('充值', 'Recharges') }}
      </p>
      <p class="mt-3 break-all text-2xl font-semibold tabular-nums">
        {{ money(finance?.recharge_amount) }}
      </p>
      <p class="mt-3 flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground tabular-nums">
        <span class="whitespace-nowrap">{{ t('套餐', 'Plans') }} {{ money(finance?.plan_purchase_amount) }}</span>
        <span class="whitespace-nowrap">{{ t('赠送', 'Gifts') }} {{ money(finance?.gift_credit_amount) }}</span>
      </p>
    </section>
    <section
      class="rounded-2xl border bg-card p-5 shadow-sm"
      data-user-summary="balance"
    >
      <p class="text-xs font-medium text-muted-foreground">
        {{ t('余额', 'Balance') }}
      </p>
      <p class="mt-3 break-all text-2xl font-semibold tabular-nums">
        {{ money(finance?.wallet_balance) }}
      </p>
      <p class="mt-3 flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground tabular-nums">
        <span class="whitespace-nowrap">{{ t('充值', 'Recharges') }} {{ money(finance?.recharge_balance) }}</span>
        <span class="whitespace-nowrap">{{ t('赠送', 'Gifts') }} {{ money(finance?.gift_balance) }}</span>
      </p>
    </section>
    <section
      class="rounded-2xl border bg-card p-5 shadow-sm"
      data-user-summary="activity"
    >
      <p class="text-xs font-medium text-muted-foreground">
        {{ detail ? t('请求', 'Requests') : t('用户', 'Users') }}
      </p>
      <p
        class="mt-3 text-2xl font-semibold tabular-nums"
        :title="detail ? undefined : t('活跃用户 / 总用户', 'Active users / Total users')"
      >
        {{ count(detail ? summary?.request_count : activeUserCount) }}<span
          v-if="!detail"
          class="ml-2 text-sm font-normal text-muted-foreground"
        >/ {{ count(userCount) }}</span>
      </p>
      <p class="mt-3 flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground tabular-nums">
        <span class="whitespace-nowrap">{{ detail ? t('成功', 'Successful') : t('请求', 'Requests') }} {{ count(detail ? summary?.successful_request_count : summary?.request_count) }}</span>
        <span class="whitespace-nowrap">Tokens {{ count(summary?.total_tokens) }}</span>
      </p>
    </section>
  </div>
</template>

<script setup lang="ts">
import type { OverviewMetrics, OverviewUserFinance } from '@/api/overview'
import { count, money } from '../format'
import { useOverviewI18n } from '../i18n'
defineProps<{ summary?: OverviewMetrics | null; finance?: OverviewUserFinance | null; userCount?: number; activeUserCount?: number; detail?: boolean }>()
const { t } = useOverviewI18n()
</script>
