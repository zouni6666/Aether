<template>
  <main class="space-y-5 px-4 pb-8 sm:px-6 lg:px-0">
    <RouterLink
      :to="link('/admin/user-stats')"
      class="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-primary"
    >
      <ArrowLeft class="h-3.5 w-3.5" />{{ t('用户分析', 'User analysis') }}
    </RouterLink>
    <OverviewToolbar
      :title="data?.data.user.username || t('用户账目', 'User account')"
      :range="range"
      :preset="relativePreset"
      :show-range="false"
      :refresh-active="autoRefresh"
      :refresh-title="refreshTitle"
      presets-only
      @update:range="selectRange"
      @refresh="toggleAutoRefresh"
    >
      <template #subtitle>
        <p
          v-if="data"
          class="mt-1 break-words text-xs text-muted-foreground"
        >
          {{ data.data.user.email }} · {{ data.data.user.is_active ? t('已启用', 'Enabled') : t('已停用', 'Disabled') }}
        </p>
      </template>
    </OverviewToolbar>
    <OverviewStatus
      :error="error"
      @retry="refreshAll"
    />
    <UserFinanceSummary
      :summary="data?.data.summary"
      :finance="data?.data.finance"
      detail
    />
    <p class="text-xs text-muted-foreground">
      {{ t('消费与到账按所选时间统计，余额为当前值', 'Consumption and credits follow the selected period; balances are current') }}
    </p>
    <UserPayments
      :payments="data?.data.payments"
      :timezone="range.timezone"
      :loading="loading"
      :limit="paymentLimit"
      :offset="paymentOffset"
      @page="value => paymentOffset = value"
      @size="resizePayments"
    />
    <ConsumptionTable
      :user-id="userId"
      :revision="revision"
    />
    <UserReports
      :user-id="userId"
      :revision="revision"
    />
  </main>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import { overviewApi, type OverviewRange } from '@/api/overview'
import OverviewToolbar from '@/features/overview/components/OverviewToolbar.vue'
import OverviewStatus from '@/features/overview/components/OverviewStatus.vue'
import UserFinanceSummary from '@/features/overview/users/UserFinanceSummary.vue'
import UserPayments from '@/features/overview/users/UserPayments.vue'
import UserReports from '@/features/overview/users/UserReports.vue'
import ConsumptionTable from '@/features/overview/users/ConsumptionTable.vue'
import { useUserAnalysisRefresh } from '@/features/overview/users/useUserAnalysisRefresh'
import { useOverviewQuery } from '@/features/overview/query'
import { useOverviewRequest } from '@/features/overview/useOverviewRequest'
import { useOverviewI18n } from '@/features/overview/i18n'
const { t } = useOverviewI18n()
const { route, range, relativePreset, refreshRange, setRange, link } = useOverviewQuery('today', { rolling: true })
const userId = computed(() => String(route.params.userId))
const { revision, autoRefresh, refreshTitle, refreshAll, toggleAutoRefresh } = useUserAnalysisRefresh(refreshRange)
const paymentLimit = ref(25)
const paymentOffset = ref(0)
const scope = computed(() => JSON.stringify({ user: userId.value, ...(relativePreset.value ? { preset: relativePreset.value, timezone: range.value.timezone } : range.value) }))
watch(scope, () => { paymentOffset.value = 0 }, { flush: 'sync' })
const requestQuery = computed(() => ({ ...range.value, user_id: userId.value, payment_limit: paymentLimit.value, payment_offset: paymentOffset.value }))
const { data, loading, error } = useOverviewRequest(() => JSON.stringify([requestQuery.value, revision.value]), signal => overviewApi.user(userId.value, requestQuery.value, signal), { scopeKey: scope })
function selectRange(value: OverviewRange, preset?: string) { paymentOffset.value = 0; void setRange(value, preset) }
function resizePayments(value: number) { paymentLimit.value = value; paymentOffset.value = 0 }
</script>
