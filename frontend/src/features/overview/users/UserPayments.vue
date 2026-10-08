<template>
  <section
    class="min-w-0 rounded-2xl border bg-card p-4 shadow-sm sm:p-5"
    data-user-payments
  >
    <div class="mb-4">
      <h2 class="text-sm font-semibold">
        {{ t('充值与套餐记录', 'Recharge and plan records') }}
      </h2>
      <p class="mt-1 text-xs text-muted-foreground">
        {{ t('按到账时间统计，赠送到账单独列示', 'Reported by credit time; gift credits are listed separately') }}
      </p>
    </div>
    <div
      class="overflow-x-auto"
      :aria-busy="loading"
    >
      <table class="w-full min-w-[650px] text-sm">
        <thead class="border-y text-xs text-muted-foreground">
          <tr>
            <th class="py-3 text-left font-medium">
              {{ t('订单', 'Order') }}
            </th><th class="px-3 py-3 text-left font-medium">
              {{ t('类型', 'Type') }}
            </th><th class="px-3 py-3 text-left font-medium">
              {{ t('支付方式', 'Payment method') }}
            </th><th class="px-3 py-3 text-right font-medium">
              {{ t('到账金额', 'Credited amount') }}
            </th><th class="py-3 text-right font-medium">
              {{ t('到账时间', 'Credited at') }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="payment in payments?.items || []"
            :key="payment.id"
            class="border-b last:border-0"
          >
            <td class="max-w-56 break-all py-3 font-mono text-xs">
              {{ payment.order_no || payment.id }}
            </td><td class="whitespace-nowrap px-3 py-3">
              {{ kindLabel(payment.kind) }}
            </td><td class="px-3 py-3 text-muted-foreground">
              {{ methodLabel(payment.payment_method) }}
            </td><td class="whitespace-nowrap px-3 py-3 text-right font-medium tabular-nums">
              {{ money(payment.amount) }}
            </td><td class="whitespace-nowrap py-3 text-right text-xs text-muted-foreground">
              {{ timestamp(payment.credited_at, timezone) }}
            </td>
          </tr>
          <tr v-if="!payments?.items.length">
            <td
              colspan="5"
              class="py-12 text-center text-muted-foreground"
            >
              {{ loading ? t('加载中', 'Loading') : !payments ? t('到账记录暂不可用', 'Credit records unavailable') : t('此时间范围暂无到账记录', 'No credited payments in this period') }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <OverviewPagination
      v-if="payments"
      :total="payments.total"
      :offset="offset"
      :limit="limit"
      :loading="loading"
      @page="value => $emit('page', value)"
      @size="value => $emit('size', value)"
    />
  </section>
</template>

<script setup lang="ts">
import type { OverviewPage, OverviewUserPayment } from '@/api/overview'
import OverviewPagination from '../components/OverviewPagination.vue'
import { money, timestamp } from '../format'
import { useOverviewI18n } from '../i18n'
defineProps<{ payments?: OverviewPage<OverviewUserPayment> | null; timezone: string; loading: boolean; limit: number; offset: number }>()
defineEmits<{ page: [value: number]; size: [value: number] }>()
const { t } = useOverviewI18n()
function kindLabel(kind: string) {
  return ({ wallet_recharge: t('钱包充值', 'Wallet recharge'), plan_purchase: t('套餐购买', 'Plan purchase'), gift_credit: t('赠送到账', 'Gift credit') })[kind] || kind
}
function methodLabel(method: string) {
  return ({ alipay: t('支付宝', 'Alipay'), wechat: t('微信支付', 'WeChat Pay'), stripe: 'Stripe', manual: t('人工入账', 'Manual credit'), gift_code: t('兑换码', 'Gift code') })[method] || method || '-'
}
</script>
