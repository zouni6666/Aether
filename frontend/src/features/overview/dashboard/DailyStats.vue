<template>
  <section class="min-w-0 space-y-3 border-t pt-5">
    <h2 class="text-sm font-semibold">
      {{ t('每日统计', 'Daily statistics') }}
    </h2>
    <div class="divide-y sm:hidden">
      <div
        v-for="row in rows"
        :key="row.bucket_start"
        class="space-y-2 py-3"
      >
        <div class="flex flex-wrap items-center justify-between gap-2 text-xs">
          <RouterLink
            :to="dailyLink(row.bucket_start)"
            class="font-medium hover:text-primary hover:underline"
          >
            {{ chartDate(row.bucket_start, range.timezone) }}
          </RouterLink>
          <span class="tabular-nums">{{ money(row.billable_amount) }}<span
            v-if="amountStatus(row.billable_amount, t)"
            class="ml-1 text-amber-700 dark:text-amber-400"
          >{{ amountStatus(row.billable_amount, t) }}</span></span>
        </div>
        <dl class="grid grid-cols-2 gap-x-4 gap-y-2 text-xs">
          <div class="flex flex-wrap justify-between gap-x-2">
            <dt class="text-muted-foreground">
              {{ t('请求', 'Requests') }}
            </dt><dd class="tabular-nums">
              {{ count(row.request_count) }}
            </dd>
          </div>
          <div class="flex flex-wrap justify-between gap-x-2">
            <dt class="text-muted-foreground">
              Tokens
            </dt><dd class="tabular-nums">
              {{ count(row.total_tokens) }}
            </dd>
          </div>
          <div class="flex flex-wrap justify-between gap-x-2">
            <dt class="text-muted-foreground">
              {{ t('响应', 'Response') }}
            </dt><dd class="tabular-nums">
              {{ latency(row.latency_ms.avg) }}
            </dd>
          </div>
          <div class="flex flex-wrap justify-between gap-x-2">
            <dt class="text-muted-foreground">
              {{ t('成功率', 'Success rate') }}
            </dt><dd class="tabular-nums">
              {{ percent(row.success_rate.value) }}
            </dd>
          </div>
        </dl>
      </div>
      <div class="space-y-2 py-3 text-xs">
        <div class="flex flex-wrap justify-between gap-2 font-medium">
          <span>{{ t('周期合计', 'Period total') }}</span><span>{{ money(data.summary.billable_amount) }}<span
            v-if="amountStatus(data.summary.billable_amount, t)"
            class="ml-1 text-amber-700 dark:text-amber-400"
          >{{ amountStatus(data.summary.billable_amount, t) }}</span></span>
        </div>
        <div class="flex flex-wrap justify-between gap-2 text-muted-foreground">
          <span>{{ t('请求', 'Requests') }} {{ count(data.summary.request_count) }}</span><span>Tokens {{ count(data.summary.total_tokens) }}</span>
        </div>
      </div>
    </div>
    <div class="hidden overflow-x-auto sm:block">
      <table class="w-full min-w-[600px] text-left text-xs tabular-nums">
        <thead class="border-b text-muted-foreground">
          <tr>
            <th class="py-3 font-medium">
              {{ t('日期', 'Date') }}
            </th>
            <th class="px-3 py-3 text-right font-medium">
              {{ t('请求次数', 'Requests') }}
            </th>
            <th class="px-3 py-3 text-right font-medium">
              Tokens
            </th>
            <th class="px-3 py-3 text-right font-medium">
              {{ t('费用', 'Cost') }}
            </th>
            <th class="px-3 py-3 text-right font-medium">
              {{ t('平均响应', 'Avg response') }}
            </th>
            <th class="py-3 pl-3 text-right font-medium">
              {{ t('成功率', 'Success rate') }}
            </th>
          </tr>
        </thead>
        <tbody class="divide-y">
          <tr v-if="!rows.length">
            <td
              colspan="6"
              class="py-8 text-center text-muted-foreground"
            >
              {{ t('暂无数据', 'No data') }}
            </td>
          </tr>
          <tr
            v-for="row in rows"
            :key="row.bucket_start"
          >
            <td class="py-3">
              <RouterLink
                :to="dailyLink(row.bucket_start)"
                class="font-medium hover:text-primary hover:underline"
              >
                {{ chartDate(row.bucket_start, range.timezone) }}
              </RouterLink>
            </td>
            <td class="px-3 py-3 text-right">
              {{ count(row.request_count) }}
            </td>
            <td class="px-3 py-3 text-right">
              {{ count(row.total_tokens) }}
            </td>
            <td class="px-3 py-3 text-right">
              {{ money(row.billable_amount) }}
              <span
                v-if="amountStatus(row.billable_amount, t)"
                class="block text-[10px] text-amber-700 dark:text-amber-400"
              >{{ amountStatus(row.billable_amount, t) }}</span>
            </td>
            <td class="px-3 py-3 text-right">
              {{ latency(row.latency_ms.avg) }}
            </td>
            <td class="py-3 pl-3 text-right">
              {{ percent(row.success_rate.value) }}
            </td>
          </tr>
        </tbody>
        <tfoot class="border-t bg-muted/30 font-medium">
          <tr>
            <td class="py-3">
              {{ t('周期合计', 'Period total') }}
            </td>
            <td class="px-3 py-3 text-right">
              {{ count(data.summary.request_count) }}
            </td>
            <td class="px-3 py-3 text-right">
              {{ count(data.summary.total_tokens) }}
            </td>
            <td class="px-3 py-3 text-right">
              {{ money(data.summary.billable_amount) }}
              <span
                v-if="amountStatus(data.summary.billable_amount, t)"
                class="block text-[10px] text-amber-700 dark:text-amber-400"
              >{{ amountStatus(data.summary.billable_amount, t) }}</span>
            </td>
            <td class="px-3 py-3 text-right">
              {{ latency(data.summary.latency_ms.avg) }}
            </td>
            <td class="py-3 pl-3 text-right">
              {{ percent(data.summary.success_rate.value) }}
            </td>
          </tr>
        </tfoot>
      </table>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import type { OverviewDashboardCharts, OverviewRange } from '@/api/overview'
import { count, money, percent } from '../format'
import { rangeQuery } from '../query'
import { useOverviewI18n } from '../i18n'
import { amountStatus } from './amount'
import { chartDate, dayRange } from './charts'

const props = defineProps<{ data: OverviewDashboardCharts; range: OverviewRange }>()
const { t } = useOverviewI18n()
const rows = computed(() => [...props.data.series].sort((a, b) => b.bucket_start.localeCompare(a.bucket_start)))
const latency = (value: number | null) => value == null ? '-' : `${count(value)} ms`
function dailyLink(bucket: string) { return { path: '/admin/usage', query: rangeQuery(dayRange(bucket, props.range)) } }
</script>
