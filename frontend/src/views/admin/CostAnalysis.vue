<template>
  <main class="space-y-6 px-4 pb-8 sm:px-6 lg:px-0">
    <OverviewToolbar
      :title="t('成本分析', 'Cost analysis')"
      :range="range"
      :preset="relativePreset"
      :refresh-active="autoRefresh"
      :refresh-title="refreshTitle"
      :show-range="false"
      presets-only
      :loading="loading"
      @update:range="setRange"
      @refresh="toggleAutoRefresh"
    >
      <Button
        variant="outline"
        size="sm"
        :disabled="!accounts?.items.length"
        @click="recording = true"
      >
        <Plus class="mr-1.5 h-3.5 w-3.5" />{{ t('登记支出', 'Record expense') }}
      </Button>
    </OverviewToolbar>
    <div
      class="grid grid-cols-2 gap-3 xl:grid-cols-4"
      data-cost-summary
    >
      <Card class="min-w-0 space-y-2 p-4 sm:p-5">
        <p class="text-xs text-muted-foreground">
          {{ t('已登记支出', 'Recorded expenses') }}
        </p>
        <div class="min-h-8 text-xl font-semibold tabular-nums">
          <p
            v-for="item in totalAmounts('amount')"
            :key="item.currency"
          >
            {{ item.text }}
          </p>
        </div>
        <p class="text-xs text-muted-foreground">
          {{ t('所选时段', 'Selected range') }} · {{ count(expenses?.total) }} {{ t('笔', 'entries') }}
        </p>
      </Card>
      <Card class="min-w-0 space-y-2 p-4 sm:p-5">
        <p class="text-xs text-muted-foreground">
          {{ t('提供商充值', 'Provider recharges') }}
        </p>
        <div class="min-h-8 text-xl font-semibold tabular-nums">
          <p
            v-for="item in totalAmounts('recharge_amount')"
            :key="item.currency"
          >
            {{ item.text }}
          </p>
        </div>
        <p class="text-xs text-muted-foreground">
          {{ t('已登记的余额充值', 'Recorded balance top-ups') }}
        </p>
      </Card>
      <Card class="min-w-0 space-y-2 p-4 sm:p-5">
        <p class="text-xs text-muted-foreground">
          {{ t('套餐与续费', 'Subscriptions and renewals') }}
        </p>
        <div class="min-h-8 text-xl font-semibold tabular-nums">
          <p
            v-for="item in totalAmounts('subscription_amount')"
            :key="item.currency"
          >
            {{ item.text }}
          </p>
        </div>
        <p class="text-xs text-muted-foreground">
          {{ t('已登记的套餐实付金额', 'Recorded subscription payments') }}
        </p>
      </Card>
      <Card class="min-w-0 space-y-2 p-4 sm:p-5">
        <p class="text-xs text-muted-foreground">
          {{ t('提供商调用', 'Provider requests') }}
        </p>
        <p class="min-h-8 text-xl font-semibold tabular-nums">
          {{ count(costs?.data.summary.request_count) }}
        </p>
        <p class="text-xs text-muted-foreground">
          {{ count(costs?.data.summary.total_tokens) }} Tokens
        </p>
      </Card>
    </div>
    <p class="text-xs text-muted-foreground">
      {{ t('支出以登记的付款记录为准，不同币种分别汇总；套餐额度与上游余额单独展示。', 'Expenses reflect recorded payments, grouped by currency. Plan quotas and upstream balances are shown separately.') }}
    </p>

    <section
      class="space-y-4"
      data-provider-usage
    >
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 class="text-sm font-semibold">
            {{ t('提供商用量', 'Provider usage') }}
          </h2><p class="mt-1 text-xs text-muted-foreground">
            {{ t('所选时段 · 按请求量排序', 'Selected range · Most requests first') }}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          :disabled="exporting || !providers?.data.total"
          @click="exportCsv('breakdown', providerQuery)"
        >
          <Download class="mr-1.5 h-3.5 w-3.5" />{{ t('导出用量报表', 'Export usage') }}
        </Button>
      </div>
      <OverviewStatus
        :loading="providerLoading && !providers"
        :error="providerError || exportError"
        @retry="refreshProviders"
      />
      <Card class="min-w-0 overflow-hidden">
        <div class="overflow-x-auto px-4 sm:px-5">
          <table class="w-full min-w-[760px] text-sm">
            <thead class="border-b text-xs text-muted-foreground">
              <tr>
                <th class="py-3 text-left font-medium">
                  {{ t('提供商', 'Provider') }}
                </th>
                <th class="px-3 py-3 text-right font-medium">
                  {{ t('请求量', 'Requests') }}
                </th>
                <th class="px-3 py-3 text-right font-medium">
                  {{ t('输入 Token', 'Input tokens') }}
                </th>
                <th class="px-3 py-3 text-right font-medium">
                  {{ t('输出 Token', 'Output tokens') }}
                </th>
                <th class="px-3 py-3 text-right font-medium">
                  {{ t('已登记支出', 'Recorded expenses') }}
                </th>
                <th class="py-3 text-right font-medium">
                  {{ t('使用记录', 'Usage records') }}
                </th>
              </tr>
            </thead>
            <tbody class="divide-y">
              <tr
                v-for="provider in providers?.data.items || []"
                :key="provider.id || 'unattributed'"
              >
                <td class="max-w-64 break-words py-4 font-medium">
                  {{ provider.label || t('未归属提供商', 'Unattributed provider') }}<p class="mt-1 text-xs font-normal text-muted-foreground">
                    {{ providerType(provider.id) }}
                  </p>
                </td>
                <td class="px-3 py-4 text-right tabular-nums">
                  {{ count(provider.request_count) }}
                </td>
                <td class="px-3 py-4 text-right tabular-nums">
                  {{ count(provider.input_tokens) }}
                </td>
                <td class="px-3 py-4 text-right tabular-nums">
                  {{ count(provider.output_tokens) }}
                </td>
                <td class="px-3 py-4 text-right tabular-nums">
                  <p
                    v-for="item in providerAmounts(provider.id)"
                    :key="item.currency"
                    class="whitespace-nowrap"
                  >
                    {{ item.text }}
                  </p>
                </td>
                <td class="py-4 text-right">
                  <RouterLink
                    v-if="provider.id"
                    :to="link('/admin/usage', { provider_id: provider.id })"
                    class="inline-flex items-center gap-1 text-xs text-primary hover:underline"
                  >
                    {{ t('查看', 'View') }}<ArrowUpRight class="h-3.5 w-3.5" />
                  </RouterLink><span v-else>-</span>
                </td>
              </tr>
              <tr v-if="!providers?.data.items.length">
                <td
                  colspan="6"
                  class="py-12 text-center text-muted-foreground"
                >
                  {{ providerLoading ? t('加载提供商用量…', 'Loading provider usage…') : providers ? t('此时段暂无调用记录', 'No requests in this range') : t('暂未取得用量', 'Usage unavailable') }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <div
          v-if="providers"
          class="px-4 pb-3 sm:px-5"
        >
          <OverviewPagination
            :total="providers.data.total"
            :limit="providerLimit"
            :offset="providerOffset"
            :loading="providerLoading"
            @page="value => providerOffset = value"
            @size="value => { providerLimit = value; providerOffset = 0 }"
          />
        </div>
      </Card>
    </section>

    <OverviewStatus
      :loading="accountLoading && !accounts"
      :error="accountError"
      @retry="refreshAccounts"
    />
    <ProviderAccounts
      v-if="accounts"
      :accounts="accounts.items"
      :observed-at="accounts.observed_at"
      :timezone="range.timezone"
    />

    <OverviewStatus
      :loading="expenseLoading && !expenses"
      :error="expenseError"
      @retry="refreshExpenses"
    />
    <ExpenseLedger
      :data="expenses"
      :range="range"
      :offset="expenseOffset"
      :loading="expenseLoading"
      :can-record="Boolean(accounts?.items.length)"
      @record="recording = true"
      @page="selectExpensePage"
      @size="value => { expenseLimit = value; expenseOffset = 0 }"
      @changed="expensesChanged"
    />

    <section
      class="space-y-4"
      data-cost-reports
    >
      <h2 class="text-sm font-semibold">
        {{ t('用量报表', 'Usage reports') }}
      </h2>
      <OverviewStatus
        :loading="costLoading && !costs"
        :error="costError"
        @retry="refreshCosts"
      />
      <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <Card class="min-w-0 space-y-4 p-4 sm:p-5">
          <h3 class="text-sm font-medium">
            {{ t('调用趋势', 'Request trend') }}
          </h3><div
            v-if="hasRequests"
            class="h-60"
          >
            <LineChart :data="requestChart" />
          </div><p
            v-else
            class="flex h-60 items-center justify-center text-sm text-muted-foreground"
          >
            {{ costs ? t('此时段暂无调用', 'No requests in this range') : t('暂无报表数据', 'Report unavailable') }}
          </p>
        </Card>
        <Card class="min-w-0 space-y-4 p-4 sm:p-5">
          <h3 class="text-sm font-medium">
            {{ t('Token 用量趋势', 'Token usage trend') }}
          </h3><div
            v-if="hasRequests"
            class="h-60"
          >
            <LineChart :data="tokenChart" />
          </div><p
            v-else
            class="flex h-60 items-center justify-center text-sm text-muted-foreground"
          >
            {{ costs ? t('此时段暂无用量', 'No usage in this range') : t('暂无报表数据', 'Report unavailable') }}
          </p>
        </Card>
      </div>
    </section>
    <ExpenseDialog
      v-model:open="recording"
      :providers="accounts?.items || []"
      :timezone="range.timezone"
      @saved="expensesChanged"
    />
  </main>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import { ArrowUpRight, Download, Plus } from 'lucide-vue-next'
import { Button, Card } from '@/components/ui'
import LineChart from '@/components/charts/LineChart.vue'
import { overviewApi } from '@/api/overview'
import { providerFinanceApi, type ProviderExpenseTotals } from '@/api/providerFinance'
import OverviewToolbar from '@/features/overview/components/OverviewToolbar.vue'
import OverviewStatus from '@/features/overview/components/OverviewStatus.vue'
import OverviewPagination from '@/features/overview/components/OverviewPagination.vue'
import ExpenseDialog from '@/features/overview/costs/ExpenseDialog.vue'
import ExpenseLedger from '@/features/overview/costs/ExpenseLedger.vue'
import ProviderAccounts from '@/features/overview/costs/ProviderAccounts.vue'
import { useOverviewQuery } from '@/features/overview/query'
import { useOverviewAutoRefresh } from '@/features/overview/useOverviewAutoRefresh'
import { useOverviewRequest } from '@/features/overview/useOverviewRequest'
import { useOverviewExport } from '@/features/overview/useOverviewExport'
import { useOverviewI18n } from '@/features/overview/i18n'
import { count, money, timestamp } from '@/features/overview/format'
const { t } = useOverviewI18n()
const { range, relativePreset, refreshRange, setRange, link } = useOverviewQuery('last30days', { rolling: true })
const { revision, autoRefresh, refreshTitle, toggleAutoRefresh } = useOverviewAutoRefresh(refreshRange)
const recording = ref(false)
const providerOffset = ref(0)
const expenseOffset = ref(0)
const providerLimit = ref(25)
const expenseLimit = ref(25)
const selectedScope = computed(() => JSON.stringify(relativePreset.value ? { preset: relativePreset.value, timezone: range.value.timezone } : range.value))
watch(selectedScope, () => { providerOffset.value = 0; expenseOffset.value = 0 })
const costQuery = computed(() => ({ ...range.value, granularity: 'day' as const }))
const providerQuery = computed(() => ({ ...range.value, group_by: 'provider', sort: 'request_count', order: 'desc' as const, limit: providerLimit.value, offset: providerOffset.value }))
const expenseQuery = computed(() => ({ ...range.value, limit: expenseLimit.value, offset: expenseOffset.value }))
const { data: costs, loading: costLoading, error: costError, refresh: refreshCosts } = useOverviewRequest(() => JSON.stringify([costQuery.value, revision.value]), signal => overviewApi.costs(costQuery.value, signal), { scopeKey: selectedScope })
const { data: providers, loading: providerLoading, error: providerError, refresh: refreshProviders } = useOverviewRequest(() => JSON.stringify([providerQuery.value, revision.value]), signal => overviewApi.breakdown(providerQuery.value, signal), { scopeKey: () => JSON.stringify([selectedScope.value, providerOffset.value]) })
const { data: accounts, loading: accountLoading, error: accountError, refresh: refreshAccounts } = useOverviewRequest(revision, signal => providerFinanceApi.accounts(signal))
const { data: expenses, loading: expenseLoading, error: expenseError, refresh: refreshExpenses } = useOverviewRequest(() => JSON.stringify([expenseQuery.value, revision.value]), signal => providerFinanceApi.expenses(expenseQuery.value, signal), { scopeKey: selectedScope })
const { exporting, exportError, exportCsv } = useOverviewExport()
const loading = computed(() => costLoading.value || providerLoading.value || accountLoading.value || expenseLoading.value)
function expensesChanged() { if (expenseOffset.value) expenseOffset.value = 0; else void refreshExpenses() }
function selectExpensePage(offset: number) { if (expenseOffset.value === offset) void refreshExpenses(); else expenseOffset.value = offset }
function amount(value: string, currency: string) { return money({ value, currency, basis: 'provider_expense', status: 'known' }) }
function totalAmounts(key: keyof Pick<ProviderExpenseTotals, 'amount' | 'recharge_amount' | 'subscription_amount'>) {
  if (!expenses.value) return [{ currency: '', text: '-' }]
  if (!expenses.value.totals.length) return [{ currency: '', text: t('暂无记录', 'No records') }]
  return expenses.value.totals.map(row => ({ currency: row.currency, text: `${amount(row[key], row.currency)} ${row.currency}` }))
}
function providerAmounts(id: string | null) {
  if (!expenses.value || !id) return [{ currency: '', text: '-' }]
  const amounts = expenses.value.providers.filter(item => item.provider_id === id)
  return amounts.length ? amounts.map(item => ({ currency: item.currency, text: `${amount(item.amount, item.currency)} ${item.currency}` })) : [{ currency: '', text: t('未登记', 'Unrecorded') }]
}
function providerType(id: string | null) {
  const type = accounts.value?.items.find(account => account.provider_id === id)?.billing_type
  return type === 'monthly_quota' ? t('周期套餐', 'Period plan') : type === 'free_tier' ? t('免费额度', 'Free tier') : type === 'pay_as_you_go' ? t('按量付费', 'Pay as you go') : ''
}
const points = computed(() => costs.value?.data.timeseries || [])
const hasRequests = computed(() => points.value.some(point => point.request_count > 0))
const labels = computed(() => points.value.map(point => timestamp(point.bucket_start, range.value.timezone)))
const requestChart = computed(() => ({ labels: labels.value, datasets: [{ label: t('请求数', 'Requests'), data: points.value.map(point => point.request_count), borderColor: '#ce6545', backgroundColor: '#ce654518', fill: true, pointRadius: 1, borderWidth: 2, tension: 0.2 }] }))
const tokenChart = computed(() => ({ labels: labels.value, datasets: [{ label: t('输入 Token', 'Input tokens'), data: points.value.map(point => point.input_tokens), borderColor: '#0d9488', pointRadius: 1, borderWidth: 2, tension: 0.2 }, { label: t('输出 Token', 'Output tokens'), data: points.value.map(point => point.output_tokens), borderColor: '#d97706', pointRadius: 1, borderWidth: 2, tension: 0.2 }] }))
</script>
