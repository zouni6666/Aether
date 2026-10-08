<template>
  <Dialog
    :model-value="true"
    :title="`${user.username} · ${t('账目', 'Account')}`"
    :description="t('金额与套餐历史 · 余额为当前值', 'Balance and plan history · Balances are current')"
    size="4xl"
    placement="right"
    @update:model-value="value => !value && $emit('close')"
  >
    <template #header-actions>
      <RefreshButton
        :loading="loading"
        :title="t('刷新', 'Refresh')"
        @click="refresh"
      />
      <Button
        ref="closeButton"
        variant="ghost"
        size="icon"
        :aria-label="t('关闭账目', 'Close account')"
        @click="$emit('close')"
      >
        <X class="h-4 w-4" />
      </Button>
    </template>
    <div
      class="space-y-6"
      data-user-account-drawer
    >
      <section
        class="space-y-3"
        data-account-transactions
      >
        <h2 class="text-sm font-semibold">
          {{ t('金额变动', 'Balance changes') }}
        </h2>
        <OverviewStatus
          :error="walletError || transactionsError"
          @retry="refreshWallet"
        />
        <dl
          v-if="wallet"
          class="grid grid-cols-1 gap-3 sm:grid-cols-3"
        >
          <div
            v-for="item in balances"
            :key="item.label"
            class="rounded-xl border bg-muted/30 p-3"
          >
            <dt class="text-xs text-muted-foreground">
              {{ item.label }}
            </dt>
            <dd class="mt-1 break-all text-lg font-semibold tabular-nums">
              {{ item.value }}
            </dd>
          </div>
        </dl>
        <template v-if="wallet || walletLoading || transactionsLoading">
          <WalletTransactionsTable
            v-if="!transactionsError || transactions"
            :items="transactions?.items || []"
            :loading="walletLoading || transactionsLoading"
            :timezone="timezone"
          />
          <Pagination
            v-if="transactions"
            :total="transactions.total"
            :current="Math.floor(offset / limit) + 1"
            :page-size="limit"
            :page-size-options="[25, 50, 100]"
            :aria-busy="transactionsLoading"
            @update:current="changePage"
            @update:page-size="resize"
          />
        </template>
        <p
          v-else-if="!walletError"
          class="py-8 text-center text-sm text-muted-foreground"
        >
          {{ t('该用户尚无钱包', 'This user has no wallet yet') }}
        </p>
      </section>
      <section
        class="space-y-3"
        data-account-plans
      >
        <h2 class="text-sm font-semibold">
          {{ t('套餐变动', 'Plan history') }}
        </h2>
        <OverviewStatus
          :error="plansError"
          @retry="refreshPlans"
        />
        <UserPlanList
          v-if="!plansError || plans"
          :entitlements="plans?.items || []"
          :loading="plansLoading"
          :timezone="timezone"
          show-updated-at
        />
      </section>
    </div>
    <template #footer>
      <Button
        variant="outline"
        @click="$emit('close')"
      >
        {{ t('关闭', 'Close') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { X } from 'lucide-vue-next'
import { Button, Dialog, Pagination, RefreshButton } from '@/components/ui'
import { adminWalletApi } from '@/api/admin-wallets'
import { usersApi } from '@/api/users'
import WalletTransactionsTable from '@/features/wallet/components/WalletTransactionsTable.vue'
import UserPlanList from '@/features/users/components/UserPlanList.vue'
import OverviewStatus from '../components/OverviewStatus.vue'
import { useOverviewRequest } from '../useOverviewRequest'
import { useOverviewI18n } from '../i18n'
import { money } from '../format'

const props = defineProps<{ user: { user_id: string; username: string }; timezone: string }>()
defineEmits<{ close: [] }>()
const { t } = useOverviewI18n()
const closeButton = ref<{ $el: HTMLButtonElement } | null>(null)
const previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null
onMounted(() => closeButton.value?.$el.focus({ preventScroll: true }))
onUnmounted(() => {
  void nextTick(() => {
    if (previousFocus?.isConnected) previousFocus.focus({ preventScroll: true })
  })
})
const revision = ref(0)
const limit = ref(25)
const offset = ref(0)
watch(() => props.user.user_id, () => { offset.value = 0 }, { flush: 'sync' })
const {
  data: wallets, loading: walletLoading, error: walletError, refresh: reloadWallet,
} = useOverviewRequest(
  () => JSON.stringify([props.user.user_id, revision.value]),
  signal => adminWalletApi.listWallets({ user_id: props.user.user_id, owner_type: 'user', limit: 1, offset: 0 }, signal),
  { scopeKey: () => props.user.user_id },
)
const walletId = computed(() => wallets.value?.items.find(item => item.user_id === props.user.user_id)?.id)
const {
  data: transactions, loading: transactionsLoading, error: transactionsError, refresh: reloadTransactions,
} = useOverviewRequest(
  () => JSON.stringify([walletId.value, limit.value, offset.value, revision.value]),
  signal => walletId.value
    ? adminWalletApi.getWalletTransactions(walletId.value, { limit: limit.value, offset: offset.value }, signal)
    : Promise.resolve(null),
)
const {
  data: plans, loading: plansLoading, error: plansError, refresh: refreshPlans,
} = useOverviewRequest(
  () => JSON.stringify([props.user.user_id, revision.value]),
  signal => usersApi.listUserPlanEntitlements(props.user.user_id, { include_inactive: true }, signal),
  { scopeKey: () => props.user.user_id },
)
const wallet = computed(() => transactions.value?.wallet || wallets.value?.items.find(item => item.user_id === props.user.user_id))
const balances = computed(() => [
  { label: t('余额', 'Balance'), value: walletMoney(wallet.value?.balance) },
  { label: t('充值', 'Recharge balance'), value: walletMoney(wallet.value?.recharge_balance) },
  { label: t('赠送', 'Gift balance'), value: walletMoney(wallet.value?.gift_balance) },
])
const loading = computed(() => walletLoading.value || transactionsLoading.value || plansLoading.value)
function walletMoney(value?: number) {
  return money({ value: value == null ? null : String(value), currency: wallet.value?.currency || 'USD', basis: 'balance', status: value == null ? 'unknown' : 'known' })
}
function refresh() { revision.value += 1 }
function refreshWallet() {
  if (walletId.value && !walletError.value) void reloadTransactions()
  else void reloadWallet()
}
function changePage(value: number) {
  if (!transactionsLoading.value) offset.value = (value - 1) * limit.value
}
function resize(value: number) {
  if (transactionsLoading.value) return
  limit.value = value
  offset.value = 0
}
</script>
