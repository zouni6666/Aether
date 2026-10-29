<template>
  <Drawer
    :open="open && !!refund"
    width="sm:w-[760px] lg:w-[860px] sm:max-w-[95vw]"
    panel-class="bg-background text-foreground"
    backdrop-class="bg-black/35 backdrop-blur-sm"
    :z-index="80"
    @close="$emit('close')"
  >
    <template v-if="refund">
      <div class="sticky top-0 z-10 border-b border-border bg-background/95 backdrop-blur px-4 py-3 sm:px-6 sm:py-4">
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <h3 class="text-lg font-semibold text-foreground leading-tight">
              {{ t('ui.refundApprovals') }}
            </h3>
            <p class="text-xs text-muted-foreground">
              {{ t('ui.refund') }} {{ refund.refund_no }}
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
        <div class="rounded-2xl border border-border/60 bg-muted/30 p-4">
          <div class="grid gap-3 sm:grid-cols-2">
            <div>
              <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t('ui.owner1e9b3dc') }}
              </div>
              <div class="mt-1 text-sm font-medium">
                {{ ownerDisplayName(refund.owner_name, refund.owner_type) }}
              </div>
              <div class="mt-1 text-xs text-muted-foreground">
                {{ ownerTypeLabel(refund.owner_type) }}
              </div>
            </div>
            <div>
              <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t('ui.amount') }}
              </div>
              <div class="mt-1 text-sm font-semibold tabular-nums">
                {{ formatCurrency(refund.amount_usd) }}
              </div>
            </div>
            <div>
              <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t('ui.refundMode') }}
              </div>
              <div class="mt-1 text-sm">
                {{ refundModeLabel(refund.refund_mode) }}
              </div>
            </div>
            <div>
              <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t('ui.status6320b4a') }}
              </div>
              <div class="mt-1">
                <Badge :variant="refundStatusBadge(refund.status)">
                  {{ refundStatusLabel(refund.status) }}
                </Badge>
              </div>
            </div>
            <div class="sm:col-span-2">
              <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t('ui.requestReason') }}
              </div>
              <div class="mt-1 text-sm text-foreground whitespace-pre-wrap break-words">
                {{ refund.reason || '-' }}
              </div>
            </div>
            <div
              v-if="refund.failure_reason"
              class="sm:col-span-2"
            >
              <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t('ui.failureReason') }}
              </div>
              <div class="mt-1 text-sm text-rose-600 whitespace-pre-wrap break-words">
                {{ refund.failure_reason }}
              </div>
            </div>
          </div>
        </div>

        <div
          v-if="canFailRefund(refund)"
          class="rounded-xl border border-border/60 p-4 space-y-2"
        >
          <Label>{{ t('ui.rejectionReason') }}</Label>
          <Input
            id="refund-fail-reason"
            v-model="failReason"
            :placeholder="t('ui.enterAReasonForRejection')"
          />
        </div>

        <div
          v-if="canCompleteRefund(refund.status)"
          class="rounded-xl border border-border/60 p-4 space-y-3"
        >
          <div class="space-y-1.5">
            <Label>{{ t('ui.gatewayRefundIdOptional') }}</Label>
            <Input v-model="gatewayRefundId" />
          </div>
          <div class="space-y-1.5">
            <Label>{{ t('ui.transferReceiptReferenceNumberOptional') }}</Label>
            <Input v-model="payoutReference" />
          </div>
        </div>
      </div>

      <div class="sticky bottom-0 border-t border-border bg-background/95 backdrop-blur px-4 py-3 sm:px-6 sm:py-4">
        <div class="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
          <Button
            variant="outline"
            @click="$emit('close')"
          >
            {{ t('ui.close') }}
          </Button>
          <Button
            v-if="canProcessRefund(refund.status)"
            variant="outline"
            :disabled="submitting"
            @click="$emit('process', refund)"
          >
            {{ submitting ? t('ui.processingc1fd812') : t('ui.processRefund') }}
          </Button>
          <Button
            v-if="canCompleteRefund(refund.status)"
            :disabled="submitting"
            @click="$emit('complete', { gateway_refund_id: gatewayRefundId || undefined, payout_reference: payoutReference || undefined })"
          >
            {{ submitting ? t('ui.submitting') : t('ui.confirmCompletion') }}
          </Button>
          <Button
            v-if="canFailRefund(refund)"
            variant="destructive"
            :disabled="submitting"
            @click="$emit('fail', failReason.trim())"
          >
            {{ submitting ? t('ui.submitting') : t('ui.rejectRefund') }}
          </Button>
        </div>
      </div>
    </template>
  </Drawer>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { ref, watch } from 'vue'
import { X } from 'lucide-vue-next'
import { Badge, Button, Input, Label } from '@/components/ui'
import Drawer from '@/components/ui/drawer/Drawer.vue'
import type { AdminGlobalRefund } from '@/api/admin-wallets'
import {
  formatWalletCurrency as formatCurrency,
  ownerDisplayName,
  ownerTypeLabel,
  refundModeLabel,
  refundStatusBadge,
  refundStatusLabel,
} from '../utils/walletDisplay'
import { canCompleteRefund, canFailRefund, canProcessRefund } from '../utils/refundActions'

export interface WalletRefundCompletePayload {
  gateway_refund_id?: string
  payout_reference?: string
}

/**
 * 退款审批抽屉：展示退款信息，并承载驳回原因与完成凭证两份表单。
 * 推荐奖励复核与实际的 API 调用由父组件处理，这里只在按钮点击时把表单值抛出。
 */
const props = defineProps<{
  open: boolean
  refund: AdminGlobalRefund | null
  submitting?: boolean
}>()

defineEmits<{
  close: []
  process: [refund: AdminGlobalRefund]
  fail: [reason: string]
  complete: [payload: WalletRefundCompletePayload]
}>()

const { t } = useI18n()

const failReason = ref('')
const gatewayRefundId = ref('')
const payoutReference = ref('')

watch(() => [props.open, props.refund?.id] as const, ([open]) => {
  if (!open) return
  failReason.value = ''
  gatewayRefundId.value = ''
  payoutReference.value = ''
}, { immediate: true })

</script>
