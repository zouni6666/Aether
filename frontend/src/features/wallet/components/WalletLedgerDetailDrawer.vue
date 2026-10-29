<template>
  <Drawer
    :open="open && !!ledger"
    width="sm:w-[760px] lg:w-[860px] sm:max-w-[95vw]"
    panel-class="bg-background text-foreground"
    backdrop-class="bg-black/35 backdrop-blur-sm"
    :z-index="80"
    @close="$emit('close')"
  >
    <template v-if="ledger">
      <div class="sticky top-0 z-10 border-b border-border bg-background/95 backdrop-blur px-4 py-3 sm:px-6 sm:py-4">
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <h3 class="text-lg font-semibold text-foreground leading-tight">
              {{ t('ui.transactionDetails') }}
            </h3>
            <p class="text-xs text-muted-foreground">
              {{ t('ui.financialTransactionAuditDetails') }}
            </p>
          </div>
          <Button
            variant="ghost"
            size="icon"
            class="h-9 w-9 shrink-0"
            :title="t('ui.close')"
            @click="$emit('close')"
          >
            <X class="h-4 w-4" />
          </Button>
        </div>
      </div>

      <div class="p-4 sm:p-6 space-y-5">
        <div class="rounded-2xl border border-border/60 bg-muted/30 p-4 space-y-3">
          <div class="flex flex-wrap items-center justify-between gap-2">
            <div class="flex items-center gap-2">
              <Badge variant="outline">
                {{ walletTransactionCategoryLabel(ledger.category) }}
              </Badge>
              <Badge variant="secondary">
                {{ walletTransactionReasonLabel(ledger.reason_code) }}
              </Badge>
            </div>
            <span
              class="text-sm font-semibold tabular-nums"
              :class="ledger.amount >= 0 ? 'text-emerald-600' : 'text-rose-600'"
            >
              {{ ledger.amount >= 0 ? '+' : '' }}{{ ledger.amount.toFixed(4) }}
            </span>
          </div>
          <div class="text-xs text-muted-foreground">
            {{ formatDateTime(ledger.created_at) }}
          </div>
        </div>

        <div class="grid gap-3 sm:grid-cols-2">
          <div class="rounded-xl border border-border/60 p-3">
            <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
              {{ t('ui.owner1e9b3dc') }}
            </div>
            <div class="mt-1 text-sm font-medium">
              {{ ownerDisplayName(ledger.owner_name, ledger.owner_type) }}
            </div>
            <div class="mt-1 text-xs text-muted-foreground flex items-center gap-2">
              <span>{{ ownerTypeLabel(ledger.owner_type) }}</span>
              <Badge
                v-if="ledger.wallet_status"
                variant="outline"
                class="text-[10px]"
              >
                {{ walletStatusLabel(ledger.wallet_status) }}
              </Badge>
            </div>
          </div>
          <div class="rounded-xl border border-border/60 p-3">
            <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
              {{ t('ui.balanceChanges') }}
            </div>
            <div class="mt-1 text-sm font-medium tabular-nums">
              {{ ledger.balance_before.toFixed(4) }} → {{ ledger.balance_after.toFixed(4) }}
            </div>
            <div
              v-if="ledger.recharge_balance_before !== null && ledger.recharge_balance_before !== undefined && ledger.gift_balance_before !== null && ledger.gift_balance_before !== undefined"
              class="mt-1 text-xs text-muted-foreground tabular-nums"
            >
              {{ t('ui.paid') }} {{ Number(ledger.recharge_balance_before).toFixed(4) }}→{{ Number(ledger.recharge_balance_after ?? 0).toFixed(4) }} {{ t('ui.gift') }} {{ Number(ledger.gift_balance_before).toFixed(4) }}→{{ Number(ledger.gift_balance_after ?? 0).toFixed(4) }}
            </div>
          </div>
        </div>

        <div class="grid gap-3 sm:grid-cols-2">
          <div class="rounded-xl border border-border/60 p-3">
            <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
              {{ t('ui.referenceType') }}
            </div>
            <div class="mt-1 text-sm font-medium break-all">
              {{ walletLinkTypeLabel(ledger.link_type) }}
            </div>
          </div>
          <div class="rounded-xl border border-border/60 p-3">
            <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
              {{ t('ui.transactionId') }}
            </div>
            <div class="mt-1 text-sm font-mono break-all">
              {{ ledger.id }}
            </div>
          </div>
        </div>

        <div
          v-if="ledger.link_type === 'payment_order'"
          class="grid gap-3 sm:grid-cols-2"
        >
          <div class="rounded-xl border border-border/60 p-3">
            <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
              {{ t('ui.paymentMethod') }}
            </div>
            <div class="mt-1 text-sm font-medium">
              <span v-if="loadingOrderNo">{{ t('common.loading') }}</span>
              <span v-else>{{ paymentMethod ? paymentMethodLabel(paymentMethod) : '-' }}</span>
            </div>
          </div>
          <div class="rounded-xl border border-border/60 p-3">
            <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
              {{ t('ui.topUpOrderNumber') }}
            </div>
            <div class="mt-1 text-sm font-mono break-all">
              <span v-if="loadingOrderNo">{{ t('common.loading') }}</span>
              <span v-else>{{ paymentOrderNo || '-' }}</span>
            </div>
          </div>
        </div>

        <div class="rounded-xl border border-border/60 p-3">
          <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
            {{ t('ui.performedBy') }}
          </div>
          <div class="mt-1 text-sm font-medium">
            {{ ledger.operator_name || (ledger.operator_id ? t('ui.deletedUser') : t('ui.systemccc7d7c')) }}
          </div>
          <div class="mt-1 text-xs text-muted-foreground">
            ID: {{ ledger.operator_id || '-' }}
          </div>
          <div
            v-if="ledger.operator_email"
            class="mt-1 text-xs text-muted-foreground"
          >
            {{ t('ui.email6a174ee') }} {{ ledger.operator_email }}
          </div>
        </div>

        <div class="rounded-xl border border-border/60 p-3">
          <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
            {{ t('ui.description') }}
          </div>
          <div class="mt-1 text-sm text-foreground whitespace-pre-wrap break-words">
            {{ ledger.description || '-' }}
          </div>
        </div>
      </div>
    </template>
  </Drawer>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { ref, watch } from 'vue'
import { X } from 'lucide-vue-next'
import { Badge, Button } from '@/components/ui'
import Drawer from '@/components/ui/drawer/Drawer.vue'
import { adminPaymentsApi } from '@/api/admin-payments'
import type { AdminLedgerTransaction } from '@/api/admin-wallets'
import { formatDateTime } from '@/utils/format'
import { log } from '@/utils/logger'
import {
  ownerDisplayName,
  ownerTypeLabel,
  paymentMethodLabel,
  walletLinkTypeLabel,
  walletStatusLabel,
  walletTransactionCategoryLabel,
  walletTransactionReasonLabel,
} from '../utils/walletDisplay'

/** 资金流水详情抽屉；关联充值订单的流水会额外解析订单号与支付方式 */
const props = defineProps<{
  open: boolean
  ledger: AdminLedgerTransaction | null
}>()

defineEmits<{
  close: []
}>()

const { t } = useI18n()

const loadingOrderNo = ref(false)
const paymentOrderNo = ref<string | null>(null)
const paymentMethod = ref<string | null>(null)
let detailSequence = 0

async function resolveRechargeOrderNo(tx: AdminLedgerTransaction) {
  const generation = ++detailSequence
  loadingOrderNo.value = false
  if (tx.link_type !== 'payment_order' || !tx.link_id) {
    paymentOrderNo.value = null
    paymentMethod.value = null
    return
  }

  if (tx.link_id.startsWith('po_')) {
    paymentOrderNo.value = tx.link_id
    paymentMethod.value = null
    return
  }

  loadingOrderNo.value = true
  try {
    const resp = await adminPaymentsApi.getOrder(tx.link_id)
    if (generation !== detailSequence || !props.open || props.ledger?.id !== tx.id) return
    paymentOrderNo.value = resp.order.order_no || null
    paymentMethod.value = resp.order.payment_method || null
  } catch (error) {
    if (generation !== detailSequence) return
    log.error('加载关联充值订单失败:', error)
    paymentOrderNo.value = null
    paymentMethod.value = null
  } finally {
    if (generation === detailSequence) loadingOrderNo.value = false
  }
}

watch(() => [props.open, props.ledger] as const, ([open, ledger]) => {
  if (!open || !ledger) {
    detailSequence++
    return
  }
  paymentOrderNo.value = null
  paymentMethod.value = null
  void resolveRechargeOrderNo(ledger)
}, { immediate: true })

</script>
