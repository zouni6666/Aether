<template>
  <section
    id="usage-records"
    class="min-w-0 scroll-mt-6 space-y-4 rounded-2xl border bg-card p-4 shadow-sm sm:p-5"
    data-user-consumption
  >
    <div class="flex flex-wrap items-center justify-between gap-3">
      <h2 class="text-sm font-semibold">
        {{ t('使用记录', 'Usage records') }}
      </h2><Button
        variant="outline"
        size="icon"
        class="h-8 w-8"
        :disabled="exporting"
        :title="t('导出完整筛选结果', 'Export all matching results')"
        :aria-label="t('导出完整筛选结果', 'Export all matching results')"
        @click="exportCsv('consumption', requestQuery)"
      >
        <Download class="h-4 w-4" />
      </Button>
    </div>
    <OverviewStatus
      :loading="loading && !data"
      :error="error || exportError"
      @retry="refresh"
    />
    <template v-if="data">
      <div class="overflow-x-auto">
        <table class="w-full min-w-[820px] text-sm">
          <thead class="border-y text-xs text-muted-foreground">
            <tr>
              <th class="whitespace-nowrap py-3 text-left font-medium">
                {{ t('请求 / 时间', 'Request / time') }}
              </th><th class="px-3 py-3 text-left font-medium">
                {{ t('模型', 'Model') }}
              </th><th class="px-3 py-3 text-left font-medium">
                {{ t('状态', 'Status') }}
              </th><th
                v-for="label in amountLabels"
                :key="label"
                class="whitespace-nowrap px-3 py-3 text-right font-medium"
              >
                {{ label }}
              </th>
            </tr>
          </thead><tbody>
            <tr
              v-for="item in data.data.items"
              :key="item.id"
              class="border-b last:border-0"
            >
              <td class="max-w-60 py-3">
                <RouterLink
                  :to="link('/admin/usage', { user_id: userId, detail_id: item.id, request_id: item.request_id })"
                  class="block truncate font-mono text-xs text-primary hover:underline"
                  :title="item.request_id"
                >
                  {{ item.request_id }}
                </RouterLink><p class="mt-1 whitespace-nowrap text-xs text-muted-foreground">
                  {{ timestamp(item.started_at, range.timezone) }}
                </p>
              </td><td class="max-w-52 break-words px-3 py-3">
                {{ item.model || '-' }}<p class="mt-1 text-xs text-muted-foreground">
                  {{ item.provider || '-' }}
                </p>
              </td><td class="px-3 py-3">
                <span :class="item.status === 'failed' ? 'text-destructive' : ''">{{ statusLabel(item.status) }}</span><p class="mt-1 text-xs text-muted-foreground">
                  {{ statusLabel(item.settlement_status) }}
                </p>
              </td><td
                v-for="key in amounts"
                :key="key"
                class="whitespace-nowrap px-3 py-3 text-right tabular-nums"
                :title="item[key]?.value ?? t('未知', 'Unknown')"
              >
                {{ money(item[key]) }}<span
                  v-if="item[key]?.status === 'known_subtotal'"
                  class="block text-xs text-muted-foreground"
                >{{ t('已知小计', 'Known subtotal') }}</span>
              </td>
            </tr><tr v-if="!data.data.items.length">
              <td
                colspan="6"
                class="py-12 text-center text-muted-foreground"
              >
                {{ t('此时间范围暂无消费记录', 'No consumption in this range') }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <OverviewPagination
        :total="data.data.total"
        :offset="requestQuery.offset"
        :limit="requestQuery.limit"
        :loading="loading"
        @page="offset => patch({ offset })"
        @size="limit => patch({ limit, offset: undefined })"
      />
    </template>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { Download } from 'lucide-vue-next'
import { Button } from '@/components/ui'
import { overviewApi } from '@/api/overview'
import OverviewStatus from '../components/OverviewStatus.vue'
import OverviewPagination from '../components/OverviewPagination.vue'
import { useOverviewQuery } from '../query'
import { useOverviewRequest } from '../useOverviewRequest'
import { useOverviewExport } from '../useOverviewExport'
import { useOverviewI18n } from '../i18n'
import { money, timestamp } from '../format'
const props = defineProps<{ userId: string; revision: number }>()
const { t } = useOverviewI18n()
const { query, range, relativePreset, patch, link } = useOverviewQuery()
const requestQuery = computed(() => ({ ...range.value, user_id: props.userId, limit: Math.max(1, Math.min(100, query.value.limit || 25)), offset: query.value.offset || 0, sort: 'started_at', order: 'desc' as const }))
const scope = computed(() => JSON.stringify({ ...requestQuery.value, ...(relativePreset.value ? { from: undefined, to: undefined, preset: relativePreset.value } : {}) }))
const { data, loading, error, refresh } = useOverviewRequest(() => JSON.stringify([requestQuery.value, props.revision]), signal => overviewApi.consumption(requestQuery.value, signal), { scopeKey: scope })
const { exporting, exportError, exportCsv } = useOverviewExport()
const amounts = ['billable_amount', 'quota_covered_amount', 'wallet_debit_amount'] as const
const amountLabels = computed(() => [t('消费', 'Consumption'), t('额度抵扣', 'Quota covered'), t('余额扣款', 'Balance debited')])
function statusLabel(status: string) {
  const labels: Record<string, string> = { success: t('成功', 'Successful'), completed: t('已完成', 'Completed'), failed: t('失败', 'Failed'), cancelled: t('已取消', 'Cancelled'), settled: t('已结算', 'Settled'), pending: t('待结算', 'Pending'), unknown: t('未知', 'Unknown') }
  return labels[status] || status || '-'
}
</script>
