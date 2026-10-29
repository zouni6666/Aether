<template>
  <Drawer
    :open="open && !!localWallet"
    width="sm:w-[760px] lg:w-[860px] sm:max-w-[95vw]"
    panel-class="bg-background text-foreground"
    backdrop-class="bg-black/35 backdrop-blur-sm"
    :z-index="80"
    @close="handleClose"
  >
    <template v-if="localWallet">
      <div class="sticky top-0 z-10 border-b border-border bg-background/95 backdrop-blur px-4 py-3 sm:px-6 sm:py-4">
        <div class="flex items-start justify-between gap-3">
          <div class="flex items-center gap-3 min-w-0">
            <div
              class="flex h-10 w-10 items-center justify-center rounded-xl shrink-0"
              :class="accentClasses"
            >
              <Wallet class="h-5 w-5" />
            </div>
            <div class="min-w-0">
              <div class="flex items-center gap-1.5">
                <h3 class="text-lg font-semibold text-foreground leading-tight">
                  {{ contextLabel || t('ui.walletDetails') }}
                </h3>
                <Badge
                  :variant="walletStatusBadge(localWallet.status)"
                  class="w-fit px-2 py-0.5 text-[11px] leading-none"
                >
                  {{ walletStatusLabel(localWallet.status) }}
                </Badge>
              </div>
              <p class="text-xs text-muted-foreground">
                {{ ownerName || '-' }} <span v-if="ownerSubtitle">· {{ ownerSubtitle }}</span>
              </p>
            </div>
          </div>
          <Button
            variant="ghost"
            size="icon"
            class="h-9 w-9 shrink-0"
            :title="t('ui.close')"
            @click="handleClose"
          >
            <X class="h-4 w-4" />
          </Button>
        </div>
      </div>

      <div class="p-4 sm:p-6 space-y-5">
        <div class="rounded-2xl border border-border/60 bg-muted/30 p-4">
          <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            <div class="rounded-xl bg-background/80 p-3">
              <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t('ui.totalAvailableBalance') }}
              </div>
              <div
                class="mt-1 text-lg font-semibold"
                :class="totalAvailableAmount !== null && totalAvailableAmount < 0 ? 'text-rose-600' : 'text-foreground'"
              >
                {{ totalAvailableAmount === null ? t('ui.unlimited4ea2cac') : `$${formatFixed(totalAvailableAmount, 2)}` }}
              </div>
            </div>
            <div class="rounded-xl bg-background/80 p-3">
              <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t('ui.planQuotaToday') }}
              </div>
              <div class="mt-1 text-lg font-semibold text-foreground">
                {{ isApiKeyWallet ? t('ui.notApplicable') : `$${formatFixed(packageBalanceAmount, 2)}` }}
              </div>
              <div
                v-if="dailyQuota?.has_active"
                class="mt-1 text-[11px] text-muted-foreground"
              >
                {{ t('ui.used131ab90') }}{{ formatFixed(dailyQuota.used_usd, 2) }} / ${{ formatFixed(dailyQuota.total_usd, 2) }}
              </div>
            </div>
            <div class="rounded-xl bg-background/80 p-3">
              <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t('ui.walletBalance') }}
              </div>
              <div
                class="mt-1 text-lg font-semibold"
                :class="walletBalanceAmount < 0 ? 'text-rose-600' : 'text-foreground'"
              >
                ${{ formatFixed(walletBalanceAmount, 2) }}
              </div>
            </div>
            <div class="rounded-xl bg-background/80 p-3">
              <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t('ui.topUpBalance') }}
              </div>
              <div class="mt-1 text-lg font-semibold text-foreground">
                ${{ formatFixed(localWallet.recharge_balance, 2) }}
              </div>
            </div>
            <div class="rounded-xl bg-background/80 p-3">
              <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t('ui.grantBalance') }}
              </div>
              <div class="mt-1 text-lg font-semibold text-foreground">
                {{ isApiKeyWallet ? t('ui.notSupported') : `$${formatFixed(localWallet.gift_balance, 2)}` }}
              </div>
            </div>
            <div class="rounded-xl bg-background/80 p-3">
              <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t('ui.cumulativeSpending') }}
              </div>
              <div class="mt-1 text-lg font-semibold text-foreground">
                ${{ formatFixed(localWallet.total_consumed, 2) }}
              </div>
            </div>
          </div>
          <p
            v-if="!isApiKeyWallet"
            class="mt-3 text-xs text-muted-foreground"
          >
            {{ t('ui.chargesUseTheDailyPlanQuotaFirstThenTheTop') }}
          </p>
        </div>

        <Tabs v-model="activeTab">
          <TabsList :class="tabsListClass">
            <TabsTrigger value="actions">
              {{ t('ui.walletOperation') }}
            </TabsTrigger>
            <TabsTrigger value="transactions">
              {{ t('ui.walletLedger') }}
            </TabsTrigger>
            <TabsTrigger
              v-if="showRefunds"
              value="refunds"
            >
              {{ t('ui.refundApprovals') }}
            </TabsTrigger>
          </TabsList>

          <TabsContent
            value="actions"
            class="mt-4 space-y-4"
          >
            <div
              v-if="!isApiKeyWallet"
              class="space-y-2"
            >
              <Label class="text-sm font-medium"> {{ t('ui.operationType') }} </Label>
              <Select v-model="moneyActionType">
                <SelectTrigger class="h-11">
                  <SelectValue :placeholder="t('ui.selectOperationType')" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="recharge">
                    {{ t('ui.manualTopUp') }}
                  </SelectItem>
                  <SelectItem value="adjust">
                    {{ t('ui.manualAdjustment') }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div class="space-y-2">
              <Label
                for="wallet-action-amount"
                class="text-sm font-medium"
              >{{ t('ui.amountUsd') }}</Label>
              <Input
                id="wallet-action-amount"
                :model-value="actionAmount ?? ''"
                type="number"
                step="0.01"
                :placeholder="isApiKeyWallet || moneyActionType === 'adjust' ? t('ui.positiveAmountsAddFundsNegativeAmountsDeductFundsAsAn') : t('ui.enterAPositiveAmount')"
                class="h-11"
                @update:model-value="(value) => actionAmount = parseNumberInput(value, { allowFloat: true })"
              />
              <p class="text-xs text-muted-foreground">
                {{
                  isApiKeyWallet || moneyActionType === 'adjust'
                    ? t('ui.adjustmentDeductionsUseTheSelectedAccountFirstThenTheOther')
                    : t('ui.manualTopUpsOnlyAcceptPositiveAmounts')
                }}
              </p>
            </div>

            <div class="space-y-2">
              <Label
                for="wallet-action-description"
                class="text-sm font-medium"
              >{{ t('ui.description') }}</Label>
              <Input
                id="wallet-action-description"
                v-model="actionDescription"
                type="text"
                :placeholder="t('ui.addANoteForFinancialTracking')"
                class="h-11"
              />
            </div>

            <div
              v-if="moneyActionType === 'adjust' && !isApiKeyWallet"
              class="space-y-2"
            >
              <Label class="text-sm font-medium"> {{ t('ui.adjustmentAccount') }} </Label>
              <Select v-model="adjustBalanceType">
                <SelectTrigger class="h-11">
                  <SelectValue :placeholder="t('ui.selectAdjustmentAccount')" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="recharge">
                    {{ t('ui.topUpBalanceRefundable') }}
                  </SelectItem>
                  <SelectItem
                    v-if="!isApiKeyWallet"
                    value="gift"
                  >
                    {{ t('ui.giftCreditNonRefundable') }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div
              v-if="!isApiKeyWallet"
              class="rounded-xl border border-border/60 p-3 text-xs text-muted-foreground"
            >
              {{ t('ui.manualTopUpsAddToTheUserSTopUp') }}
            </div>

            <div class="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
              <Button
                variant="outline"
                class="h-10 px-5"
                @click="handleClose"
              >
                {{ t('ui.close') }}
              </Button>
              <Button
                class="h-10 px-5"
                :disabled="submitMoneyDisabled"
                @click="submitMoneyAction"
              >
                {{ submittingMoneyAction ? t('ui.processingc1fd812') : submitMoneyLabel }}
              </Button>
            </div>
          </TabsContent>

          <TabsContent
            value="transactions"
            class="mt-4 space-y-3"
          >
            <div class="flex items-center justify-between gap-3">
              <div class="text-sm text-muted-foreground">
                {{ t('ui.total76e547a') }} {{ txTotal }} {{ t('ui.itemsf004f1d') }}
              </div>
              <RefreshButton
                :loading="loadingTx"
                @click="loadTransactions"
              />
            </div>

            <WalletTransactionsTable
              :items="txItems"
              :loading="loadingTx"
            />

            <Pagination
              :current="txPage"
              :total="txTotal"
              :page-size="txPageSize"
              @update:current="handleTxPageChange"
              @update:page-size="handleTxPageSizeChange"
            />
          </TabsContent>

          <TabsContent
            v-if="showRefunds"
            value="refunds"
            class="mt-4 space-y-3"
          >
            <div class="flex items-center justify-between gap-3">
              <div class="text-sm text-muted-foreground">
                {{ t('ui.total76e547a') }} {{ refundTotal }} {{ t('ui.itemsf004f1d') }}
              </div>
              <RefreshButton
                :loading="loadingRefunds"
                @click="loadRefunds"
              />
            </div>

            <div
              v-if="refundActionType && actionRefund"
              class="rounded-xl border border-border/60 p-4 space-y-3"
            >
              <div class="text-sm font-semibold">
                {{ refundActionType === 'fail' ? t('ui.rejectRefund') : t('ui.completeRefund') }} - {{ actionRefund.refund_no }}
              </div>
              <template v-if="refundActionType === 'fail'">
                <div class="space-y-1.5">
                  <Label>{{ t('ui.rejectionReason') }}</Label>
                  <Input
                    v-model="refundFailReason"
                    :placeholder="t('ui.enterAReasonForRejection')"
                  />
                </div>
              </template>
              <template v-else>
                <div class="space-y-1.5">
                  <Label>{{ t('ui.gatewayRefundIdOptional') }}</Label>
                  <Input v-model="refundGatewayRefundId" />
                </div>
                <div class="space-y-1.5">
                  <Label>{{ t('ui.transferReceiptReferenceNumberOptional') }}</Label>
                  <Input v-model="refundPayoutReference" />
                </div>
              </template>
              <div class="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
                <Button
                  variant="outline"
                  @click="resetRefundActionForm"
                >
                  {{ t('common.cancel') }}
                </Button>
                <Button
                  v-if="refundActionType === 'fail'"
                  variant="destructive"
                  :disabled="submittingRefundAction"
                  @click="submitFailRefund"
                >
                  {{ submittingRefundAction ? t('ui.submitting') : t('ui.confirmRejection') }}
                </Button>
                <Button
                  v-else
                  :disabled="submittingRefundAction"
                  @click="submitCompleteRefund"
                >
                  {{ submittingRefundAction ? t('ui.submitting') : t('ui.confirmCompletion') }}
                </Button>
              </div>
            </div>

            <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
              <div class="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{{ t('ui.refundId') }}</TableHead>
                      <TableHead>{{ t('ui.amount') }}</TableHead>
                      <TableHead>{{ t('ui.mode') }}</TableHead>
                      <TableHead>{{ t('ui.status6320b4a') }}</TableHead>
                      <TableHead>{{ t('ui.reasonf029d9c') }}</TableHead>
                      <TableHead class="text-right">
                        {{ t('ui.actionsed31fbb') }}
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="refund in refundItems"
                      :key="refund.id"
                    >
                      <TableCell class="font-mono text-xs whitespace-nowrap">
                        {{ refund.refund_no }}
                      </TableCell>
                      <TableCell class="tabular-nums whitespace-nowrap">
                        ${{ formatFixed(refund.amount_usd, 4) }}
                      </TableCell>
                      <TableCell>
                        {{ refundModeLabel(refund.refund_mode) }}
                      </TableCell>
                      <TableCell>
                        <Badge :variant="refundStatusBadge(refund.status)">
                          {{ refundStatusLabel(refund.status) }}
                        </Badge>
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground max-w-[220px] truncate">
                        {{ refund.reason || refund.failure_reason || '-' }}
                      </TableCell>
                      <TableCell class="text-right">
                        <div class="flex justify-end gap-2">
                          <Button
                            v-if="canProcessRefund(refund.status)"
                            size="sm"
                            variant="outline"
                            :disabled="submittingRefundAction"
                            @click="processRefund(refund)"
                          >
                            {{ t('ui.processing') }}
                          </Button>
                          <Button
                            v-if="canCompleteRefund(refund.status)"
                            size="sm"
                            :disabled="submittingRefundAction"
                            @click="openCompleteRefund(refund)"
                          >
                            {{ t('ui.completed') }}
                          </Button>
                          <Button
                            v-if="canFailRefund(refund)"
                            size="sm"
                            variant="destructive"
                            :disabled="submittingRefundAction"
                            @click="openFailRefund(refund)"
                          >
                            {{ t('ui.reject') }}
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                    <TableRow v-if="!loadingRefunds && refundItems.length === 0">
                      <TableCell
                        colspan="6"
                        class="py-10"
                      >
                        <EmptyState
                          :title="t('ui.noRefundRequests')"
                          :description="t('ui.noRefundRequestsForThisWallet')"
                        />
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </div>
            </div>

            <Pagination
              :current="refundPage"
              :total="refundTotal"
              :page-size="refundPageSize"
              @update:current="handleRefundPageChange"
              @update:page-size="handleRefundPageSizeChange"
            />
          </TabsContent>
        </Tabs>
      </div>
    </template>
  </Drawer>
  <RefundReferralReview
    ref="referralReview"
    :active="open"
    :context-key="`${wallet?.id || ''}:${actionRefund?.id || ''}`"
  />
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import RefundReferralReview from '@/features/wallet/components/RefundReferralReview.vue'
import { computed, ref, watch } from 'vue'
import {
  Badge,
  Button,
  Input,
  Label,
  Pagination,
  RefreshButton,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/components/ui'
import { EmptyState } from '@/components/common'
import WalletTransactionsTable from './WalletTransactionsTable.vue'
import {
  adminWalletApi,
  type AdminWallet,
} from '@/api/admin-wallets'
import type { RefundRequest, WalletTransaction } from '@/api/wallet'
import { parseApiError } from '@/utils/errorParser'
import { parseNumberInput } from '@/utils/form'
import {
  refundModeLabel,
  refundStatusBadge,
  refundStatusLabel,
  walletStatusBadge,
  walletStatusLabel,
} from '@/features/wallet/utils/walletDisplay'
import { useToast } from '@/composables/useToast'
import { canCompleteRefund, canFailRefund, canProcessRefund } from '../utils/refundActions'
import Drawer from '@/components/ui/drawer/Drawer.vue'
import { useConfirm } from '@/composables/useConfirm'
import { Wallet, X } from 'lucide-vue-next'
import { log } from '@/utils/logger'

const props = withDefaults(
  defineProps<{
    open: boolean
    wallet: AdminWallet | null
    ownerName?: string
    ownerSubtitle?: string
    contextLabel?: string
    accent?: 'emerald' | 'blue'
    showRefunds?: boolean
  }>(),
  {
    ownerName: '',
    ownerSubtitle: '',
    contextLabel: undefined,
    accent: 'emerald',
    showRefunds: true,
  }
)

const emit = defineEmits<{
  close: []
  changed: []
}>()

const { t } = useI18n()

const referralReview = ref<InstanceType<typeof RefundReferralReview> | null>(null)

const { success, error } = useToast()
const { confirm } = useConfirm()

const activeTab = ref<'actions' | 'transactions' | 'refunds'>('actions')
const localWallet = ref<AdminWallet | null>(null)

const moneyActionType = ref<'recharge' | 'adjust'>('adjust')
const actionAmount = ref<number | undefined>(undefined)
const actionDescription = ref('')
const adjustBalanceType = ref<'recharge' | 'gift'>('recharge')
const submittingMoneyAction = ref(false)

const loadingTx = ref(false)
const txItems = ref<WalletTransaction[]>([])
const txTotal = ref(0)
const txPage = ref(1)
const txPageSize = ref(20)

const loadingRefunds = ref(false)
const refundItems = ref<RefundRequest[]>([])
const refundTotal = ref(0)
const refundPage = ref(1)
const refundPageSize = ref(20)
const submittingRefundAction = ref(false)

const refundActionType = ref<'fail' | 'complete' | null>(null)
const actionRefund = ref<RefundRequest | null>(null)
const refundFailReason = ref('')
const refundGatewayRefundId = ref('')
const refundPayoutReference = ref('')

const accentClasses = computed(() => {
  return props.accent === 'blue' ? 'bg-blue-500/10 text-blue-600' : 'bg-emerald-500/10 text-emerald-600'
})
const isApiKeyWallet = computed(() => localWallet.value?.owner_type === 'api_key')
const dailyQuota = computed(() => localWallet.value?.daily_quota ?? null)
const packageBalanceAmount = computed(() => toFiniteNumber(localWallet.value?.package_balance, 0))
const walletBalanceAmount = computed(() => toFiniteNumber(localWallet.value?.wallet_balance ?? localWallet.value?.balance, 0))
const totalAvailableAmount = computed(() => {
  if (!localWallet.value) return 0
  if (localWallet.value.unlimited || localWallet.value.total_available_balance === null) return null
  return toFiniteNumber(
    localWallet.value.total_available_balance,
    walletBalanceAmount.value + packageBalanceAmount.value
  )
})
const showRefunds = computed(() => props.showRefunds)
const tabsListClass = computed(() => {
  return [
    'tabs-button-list',
    'grid',
    'w-full',
    showRefunds.value ? 'grid-cols-3' : 'grid-cols-2',
  ]
})
const submitMoneyDisabled = computed(() => {
  if (submittingMoneyAction.value) return true
  if (!actionAmount.value) return true
  if (moneyActionType.value === 'recharge') {
    return actionAmount.value <= 0
  }
  return actionAmount.value === 0
})
const submitMoneyLabel = computed(() => {
  if (isApiKeyWallet.value) return t('ui.confirmAdjustment')
  return moneyActionType.value === 'recharge' ? t('ui.confirmTopUp') : t('ui.confirmAdjustment')
})

watch(
  () => [props.open, props.wallet?.id] as const,
  async ([open]) => {
    if (!open || !props.wallet) {
      return
    }
    localWallet.value = { ...props.wallet }
    resetActionForm()
    resetRefundActionForm()
    activeTab.value = 'actions'
    txPage.value = 1
    refundPage.value = 1
    await refreshDrawerData()
  },
  { immediate: true }
)

function handleClose() {
  emit('close')
}

function resetActionForm() {
  moneyActionType.value = isApiKeyWallet.value ? 'adjust' : 'recharge'
  actionAmount.value = undefined
  actionDescription.value = ''
  adjustBalanceType.value = 'recharge'
}

watch(
  () => [moneyActionType.value, isApiKeyWallet.value] as const,
  () => {
    if (isApiKeyWallet.value && moneyActionType.value !== 'adjust') {
      moneyActionType.value = 'adjust'
      return
    }
    if (moneyActionType.value !== 'adjust') {
      adjustBalanceType.value = 'recharge'
      return
    }
    if (isApiKeyWallet.value && adjustBalanceType.value === 'gift') {
      adjustBalanceType.value = 'recharge'
    }
  }
)

function resetRefundActionForm() {
  refundActionType.value = null
  actionRefund.value = null
  refundFailReason.value = ''
  refundGatewayRefundId.value = ''
  refundPayoutReference.value = ''
}

async function loadTransactions() {
  if (!localWallet.value) return
  loadingTx.value = true
  try {
    const offset = (txPage.value - 1) * txPageSize.value
    const resp = await adminWalletApi.getWalletTransactions(localWallet.value.id, {
      limit: txPageSize.value,
      offset,
    })
    localWallet.value = resp.wallet
    txItems.value = resp.items
    txTotal.value = resp.total
  } catch (err) {
    log.error('加载钱包流水失败:', err)
    error(parseApiError(err, t('ui.failedToLoadWalletTransactions')))
  } finally {
    loadingTx.value = false
  }
}

async function loadRefunds() {
  if (!showRefunds.value || !localWallet.value) {
    refundItems.value = []
    refundTotal.value = 0
    return
  }
  loadingRefunds.value = true
  try {
    const offset = (refundPage.value - 1) * refundPageSize.value
    const resp = await adminWalletApi.getWalletRefunds(localWallet.value.id, {
      limit: refundPageSize.value,
      offset,
    })
    localWallet.value = resp.wallet
    refundItems.value = resp.items
    refundTotal.value = resp.total
    if (actionRefund.value) {
      const latest = refundItems.value.find((item) => item.id === actionRefund.value?.id)
      if (latest) actionRefund.value = latest
    }
  } catch (err) {
    log.error('加载钱包退款失败:', err)
    error(parseApiError(err, '加载钱包退款失败'))
  } finally {
    loadingRefunds.value = false
  }
}

function handleTxPageChange(page: number) {
  txPage.value = page
  void loadTransactions()
}

function handleTxPageSizeChange(size: number) {
  txPageSize.value = size
  txPage.value = 1
  void loadTransactions()
}

function handleRefundPageChange(page: number) {
  if (!showRefunds.value) return
  refundPage.value = page
  void loadRefunds()
}

function handleRefundPageSizeChange(size: number) {
  if (!showRefunds.value) return
  refundPageSize.value = size
  refundPage.value = 1
  void loadRefunds()
}

async function refreshDrawerData() {
  if (showRefunds.value) {
    await Promise.all([loadTransactions(), loadRefunds()])
    return
  }
  await loadTransactions()
}

async function submitRecharge() {
  if (!localWallet.value) return
  if (!actionAmount.value || actionAmount.value <= 0) {
    error('人工充值金额必须大于 0')
    return
  }

  const rechargeBefore = localWallet.value.recharge_balance
  const rechargeAfter = rechargeBefore + actionAmount.value
  const totalBefore = localWallet.value.balance
  const totalAfter = totalBefore + actionAmount.value
  const confirmed = await confirm({
    get title() { return t('ui.confirmManualTopUp') },
    message: `将为 ${props.ownerName || '该钱包'} 充值 **$${formatFixed(actionAmount.value, 4)}**\n该账户**充值余额**将从 **$${formatFixed(rechargeBefore, 4)}** 变为 **$${formatFixed(rechargeAfter, 4)}**，**钱包余额**将从 **$${formatFixed(totalBefore, 4)}** 变为 **$${formatFixed(totalAfter, 4)}**`,
    get confirmText() { return t('ui.confirmTopUp') },
    variant: 'warning',
  })
  if (!confirmed) return

  submittingMoneyAction.value = true
  try {
    const response = await adminWalletApi.rechargeWallet(localWallet.value.id, {
      amount_usd: actionAmount.value,
      payment_method: 'admin_manual',
      description: actionDescription.value || `管理员为 ${props.ownerName || '钱包'} 人工充值`,
    })
    localWallet.value = response.wallet
    success('人工充值已入账')
    resetActionForm()
    await refreshDrawerData()
    emit('changed')
  } catch (err) {
    log.error('钱包人工充值失败:', err)
    error(parseApiError(err, '人工充值失败'))
  } finally {
    submittingMoneyAction.value = false
  }
}

function previewAdjustResult(
  rechargeBefore: number,
  giftBefore: number,
  amount: number,
  balanceType: 'recharge' | 'gift'
) {
  let rechargeAfter = rechargeBefore
  let giftAfter = giftBefore

  if (amount > 0) {
    if (balanceType === 'gift') {
      giftAfter += amount
    } else {
      rechargeAfter += amount
    }
    return {
      rechargeAfter,
      giftAfter,
      totalAfter: rechargeAfter + giftAfter,
    }
  }

  let remaining = Math.abs(amount)
  const consumePositiveBucket = (value: number) => {
    const available = Math.max(value, 0)
    const used = Math.min(available, remaining)
    remaining -= used
    return value - used
  }

  if (balanceType === 'gift') {
    giftAfter = consumePositiveBucket(giftAfter)
    rechargeAfter = consumePositiveBucket(rechargeAfter)
  } else {
    rechargeAfter = consumePositiveBucket(rechargeAfter)
    giftAfter = consumePositiveBucket(giftAfter)
  }

  if (remaining > 0) {
    rechargeAfter -= remaining
  }

  return {
    rechargeAfter,
    giftAfter,
    totalAfter: rechargeAfter + giftAfter,
  }
}

async function submitAdjust() {
  if (!localWallet.value) return
  if (!actionAmount.value || actionAmount.value === 0) {
    error('调账金额不能为 0')
    return
  }

  if (isApiKeyWallet.value && adjustBalanceType.value === 'gift') {
    error('独立密钥钱包不支持赠款调账')
    return
  }

  const rechargeBefore = localWallet.value.recharge_balance
  const giftBefore = localWallet.value.gift_balance
  const currentBucketBalance = adjustBalanceType.value === 'gift' ? giftBefore : rechargeBefore
  const preview = previewAdjustResult(
    rechargeBefore,
    giftBefore,
    actionAmount.value,
    adjustBalanceType.value
  )
  const afterBalance = adjustBalanceType.value === 'gift' ? preview.giftAfter : preview.rechargeAfter
  const totalBefore = localWallet.value.balance
  const totalAfter = preview.totalAfter
  const balanceTypeLabel = adjustBalanceType.value === 'gift' ? '赠款余额' : '充值余额'
  const isDeduct = actionAmount.value < 0
  const detailLine = isDeduct
    ? `该账户**充值余额**将从 **$${formatFixed(rechargeBefore, 4)}** 变为 **$${formatFixed(preview.rechargeAfter, 4)}**，**赠款余额**将从 **$${formatFixed(giftBefore, 4)}** 变为 **$${formatFixed(preview.giftAfter, 4)}**`
    : `该账户**${balanceTypeLabel}**将从 **$${formatFixed(currentBucketBalance, 4)}** 变为 **$${formatFixed(afterBalance, 4)}**`
  const confirmed = await confirm({
    title: '确认钱包调账',
    message: `将对 ${props.ownerName || '该钱包'} 的**${balanceTypeLabel}**${actionAmount.value > 0 ? '增加' : '扣减'} **$${formatFixed(Math.abs(actionAmount.value), 4)}**\n${detailLine}，**钱包余额**将从 **$${formatFixed(totalBefore, 4)}** 变为 **$${formatFixed(totalAfter, 4)}**`,
    get confirmText() { return t('ui.confirmAdjustment') },
    variant: 'warning',
  })
  if (!confirmed) return

  submittingMoneyAction.value = true
  try {
    const response = await adminWalletApi.adjustWallet(localWallet.value.id, {
      amount_usd: actionAmount.value,
      balance_type: adjustBalanceType.value,
      description: actionDescription.value || `管理员为 ${props.ownerName || '钱包'} 执行钱包调账`,
    })
    localWallet.value = response.wallet
    success('钱包调账已完成')
    resetActionForm()
    await refreshDrawerData()
    emit('changed')
  } catch (err) {
    log.error('钱包调账失败:', err)
    error(parseApiError(err, '钱包调账失败'))
  } finally {
    submittingMoneyAction.value = false
  }
}

async function submitMoneyAction() {
  if (!isApiKeyWallet.value && moneyActionType.value === 'recharge') {
    await submitRecharge()
    return
  }
  await submitAdjust()
}

async function processRefund(refund: RefundRequest) {
  if (!localWallet.value) return
  const walletId = localWallet.value.id
  const refundId = refund.id
  const canReject = canFailRefund(refund)
  submittingRefundAction.value = true
  try {
    const review = await referralReview.value?.review(walletId, refundId, 'process', canReject)
    if (review?.decision === 'reject') { openFailRefund(refund); return }
    if (review?.decision !== 'approve') return
    const resp = await adminWalletApi.processRefund(walletId, refundId, { referral_shortfall_confirmation: review.token })
    localWallet.value = resp.wallet
    success(t('ui.refundIsNowProcessing'))
    await refreshDrawerData()
    emit('changed')
  } catch (err) {
    log.error('处理退款失败:', err)
    error(parseApiError(err, '处理退款失败'))
  } finally {
    submittingRefundAction.value = false
  }
}

function openFailRefund(refund: RefundRequest) {
  refundActionType.value = 'fail'
  actionRefund.value = refund
  refundFailReason.value = ''
}

function openCompleteRefund(refund: RefundRequest) {
  refundActionType.value = 'complete'
  actionRefund.value = refund
  refundGatewayRefundId.value = ''
  refundPayoutReference.value = ''
}

async function submitFailRefund() {
  if (!localWallet.value || !actionRefund.value) return
  if (!refundFailReason.value.trim()) {
    error(t('ui.enterAReasonForRejection'))
    return
  }

  submittingRefundAction.value = true
  try {
    const resp = await adminWalletApi.failRefund(localWallet.value.id, actionRefund.value.id, {
      reason: refundFailReason.value.trim(),
    })
    localWallet.value = resp.wallet
    success(t('ui.refundRejected'))
    resetRefundActionForm()
    await refreshDrawerData()
    emit('changed')
  } catch (err) {
    log.error('驳回退款失败:', err)
    error(parseApiError(err, '驳回退款失败'))
  } finally {
    submittingRefundAction.value = false
  }
}

async function submitCompleteRefund() {
  if (!localWallet.value || !actionRefund.value) return

  const walletId = localWallet.value.id
  const selectedRefund = actionRefund.value
  const refundId = selectedRefund.id
  const canReject = canFailRefund(selectedRefund)
  const payload = {
    gateway_refund_id: refundGatewayRefundId.value || undefined,
    payout_reference: refundPayoutReference.value || undefined,
  }
  submittingRefundAction.value = true
  try {
    const review = await referralReview.value?.review(walletId, refundId, 'complete', canReject)
    if (review?.decision === 'reject') { openFailRefund(selectedRefund); return }
    if (review?.decision !== 'approve') return
    const response = await adminWalletApi.completeRefund(walletId, refundId, {
      referral_shortfall_confirmation: review.token,
      ...payload,
    })
    success(response.referral_reversal
      ? `退款已完成；该订单累计冲回 $${response.referral_reversal.reversed_amount_usd.toFixed(2)}，当前待冲回 $${response.referral_reversal.pending_reversal_amount_usd.toFixed(2)}`
      : t('ui.refundCompleted'))
    resetRefundActionForm()
    await refreshDrawerData()
    emit('changed')
  } catch (err) {
    log.error('完成退款失败:', err)
    error(parseApiError(err, '完成退款失败'))
  } finally {
    submittingRefundAction.value = false
  }
}

function toFiniteNumber(value: unknown, fallback = 0): number {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : fallback
}

function formatFixed(value: unknown, digits: number): string {
  return toFiniteNumber(value).toFixed(digits)
}

const contextLabel = computed(() => props.contextLabel ?? t('ui.walletDetails'))
</script>

