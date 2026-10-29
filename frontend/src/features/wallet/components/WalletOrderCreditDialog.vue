<template>
  <Dialog
    :model-value="open"
    @update:model-value="(value: boolean) => { if (!value) $emit('close') }"
  >
    <template #header>
      <div class="px-6 py-4 border-b border-border">
        <h3 class="text-lg font-semibold">
          {{ t('ui.manualCredit') }}
        </h3>
        <p class="text-xs text-muted-foreground mt-1">
          {{ t('ui.order43177a6') }} {{ order?.order_no || '-' }}
        </p>
      </div>
    </template>
    <div class="space-y-4">
      <div class="space-y-1.5">
        <Label>{{ t('ui.gatewayOrderIdOptional') }}</Label>
        <Input v-model="form.gateway_order_id" />
      </div>
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
        <div class="space-y-1.5">
          <Label>{{ t('ui.amountPaidOptional') }}</Label>
          <Input
            v-model.number="form.pay_amount"
            type="number"
            min="0.01"
            step="0.01"
          />
        </div>
        <div class="space-y-1.5">
          <Label>{{ t('ui.currencyOptional') }}</Label>
          <Input v-model="form.pay_currency" />
        </div>
        <div class="space-y-1.5">
          <Label>{{ t('ui.exchangeRateOptional') }}</Label>
          <Input
            v-model.number="form.exchange_rate"
            type="number"
            min="0.000001"
            step="0.000001"
          />
        </div>
      </div>
    </div>
    <template #footer>
      <Button
        variant="outline"
        @click="$emit('close')"
      >
        {{ t('common.cancel') }}
      </Button>
      <Button
        :disabled="submitting"
        @click="submit"
      >
        {{ submitting ? t('ui.submitting') : t('ui.confirmCredit') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { reactive, watch } from 'vue'
import { Button, Dialog, Input, Label } from '@/components/ui'
import type { PaymentOrder } from '@/api/wallet'

export interface WalletOrderCreditPayload {
  gateway_order_id?: string
  pay_amount?: number
  pay_currency?: string
  exchange_rate?: number
}

/** 充值订单人工到账表单：打开时按订单预填，提交时把整理后的字段抛给父组件 */
const props = defineProps<{
  open: boolean
  order: PaymentOrder | null
  submitting?: boolean
}>()

const emit = defineEmits<{
  close: []
  submit: [payload: WalletOrderCreditPayload]
}>()

const { t } = useI18n()

const form = reactive({
  gateway_order_id: '',
  pay_amount: undefined as number | undefined,
  pay_currency: '',
  exchange_rate: undefined as number | undefined,
})

watch(() => [props.open, props.order?.id] as const, ([open]) => {
  if (!open) return
  form.gateway_order_id = props.order?.gateway_order_id || ''
  form.pay_amount = props.order?.pay_amount || undefined
  form.pay_currency = props.order?.pay_currency || ''
  form.exchange_rate = props.order?.exchange_rate || undefined
}, { immediate: true })

function submit() {
  emit('submit', {
    gateway_order_id: form.gateway_order_id || undefined,
    pay_amount: form.pay_amount,
    pay_currency: form.pay_currency || undefined,
    exchange_rate: form.exchange_rate,
  })
}

</script>
