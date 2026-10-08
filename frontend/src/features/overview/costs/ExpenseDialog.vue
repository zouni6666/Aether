<template>
  <Dialog
    :open="open"
    :title="t('登记提供商支出', 'Record provider expense')"
    :description="t('记录已支付的充值、套餐或其他费用', 'Record a paid recharge, subscription or other expense')"
    size="lg"
    :persistent="saving"
    @update:open="value => !saving && emit('update:open', value)"
  >
    <form
      id="provider-expense-form"
      class="grid gap-4 py-2 sm:grid-cols-2"
      @submit.prevent="submit"
    >
      <label class="grid gap-2 text-sm sm:col-span-2">
        {{ t('提供商', 'Provider') }}
        <select
          v-model="draft.provider_id"
          required
          class="h-10 min-w-0 rounded-lg border bg-background px-3"
          :disabled="saving"
        >
          <option
            value=""
            disabled
          >{{ t('选择提供商', 'Select a provider') }}</option>
          <option
            v-for="provider in providers"
            :key="provider.provider_id"
            :value="provider.provider_id"
          >{{ provider.provider_name }}</option>
        </select>
      </label>
      <label class="grid gap-2 text-sm">
        {{ t('支出类型', 'Expense type') }}
        <select
          v-model="draft.kind"
          class="h-10 rounded-lg border bg-background px-3"
          :disabled="saving"
        >
          <option value="recharge">{{ t('余额充值', 'Balance recharge') }}</option>
          <option value="subscription">{{ t('套餐 / 续费', 'Subscription / renewal') }}</option>
          <option value="other">{{ t('其他支出', 'Other expense') }}</option>
        </select>
      </label>
      <label class="grid gap-2 text-sm">
        {{ t('币种', 'Currency') }}
        <input
          v-model="draft.currency"
          maxlength="3"
          pattern="[A-Za-z]{3}"
          required
          class="h-10 min-w-0 rounded-lg border bg-background px-3 uppercase"
          :disabled="saving"
        >
      </label>
      <label class="grid gap-2 text-sm">
        {{ t('实付金额', 'Amount paid') }}
        <input
          v-model="draft.amount"
          inputmode="decimal"
          placeholder="0.00"
          required
          class="h-10 min-w-0 rounded-lg border bg-background px-3"
          :disabled="saving"
        >
      </label>
      <label class="grid gap-2 text-sm">
        {{ t('付款时间', 'Paid at') }} · {{ timezone }}
        <input
          v-model="draft.paid_at"
          type="datetime-local"
          required
          class="h-10 min-w-0 rounded-lg border bg-background px-3"
          :disabled="saving"
        >
      </label>
      <template v-if="draft.kind === 'subscription'">
        <label class="grid gap-2 text-sm">
          {{ t('套餐开始（选填）', 'Plan starts (optional)') }}
          <input
            v-model="draft.period_start"
            type="date"
            class="h-10 min-w-0 rounded-lg border bg-background px-3"
            :disabled="saving"
          >
        </label>
        <label class="grid gap-2 text-sm">
          {{ t('套餐到期（选填）', 'Plan expires (optional)') }}
          <input
            v-model="draft.period_end"
            type="date"
            class="h-10 min-w-0 rounded-lg border bg-background px-3"
            :disabled="saving"
          >
        </label>
      </template>
      <label class="grid gap-2 text-sm sm:col-span-2">
        {{ t('账单 / 交易号（选填）', 'Invoice / reference (optional)') }}
        <input
          v-model="draft.external_reference"
          maxlength="200"
          class="h-10 min-w-0 rounded-lg border bg-background px-3"
          :disabled="saving"
        >
      </label>
      <label class="grid gap-2 text-sm sm:col-span-2">
        {{ t('备注（选填）', 'Note (optional)') }}
        <textarea
          v-model="draft.note"
          maxlength="1000"
          rows="2"
          class="min-w-0 resize-y rounded-lg border bg-background p-3"
          :disabled="saving"
        />
      </label>
      <p
        v-if="error"
        class="text-sm text-destructive sm:col-span-2"
        role="alert"
      >
        {{ error }}
      </p>
    </form>
    <template #footer>
      <Button
        type="submit"
        form="provider-expense-form"
        :disabled="saving || !providers.length"
      >
        {{ saving ? t('保存中…', 'Saving…') : t('保存记录', 'Save record') }}
      </Button>
      <Button
        variant="outline"
        :disabled="saving"
        @click="emit('update:open', false)"
      >
        {{ t('取消', 'Cancel') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import { Button, Dialog } from '@/components/ui'
import { providerFinanceApi, type ProviderAccount, type ProviderExpenseKind } from '@/api/providerFinance'
import { useOverviewI18n } from '../i18n'
import { zonedInput, zonedInstant } from '../query'

const props = defineProps<{ open: boolean; providers: ProviderAccount[]; timezone: string }>()
const emit = defineEmits<{ 'update:open': [value: boolean]; saved: [] }>()
const { t } = useOverviewI18n()
const saving = ref(false)
const error = ref('')
const draft = reactive({ provider_id: '', kind: 'recharge' as ProviderExpenseKind, amount: '', currency: 'USD', paid_at: '', period_start: '', period_end: '', note: '', external_reference: '' })
let pending: { body: string; id: string } | null = null
watch(() => props.open, open => {
  if (open && !draft.paid_at) draft.paid_at = zonedInput(new Date(), props.timezone)
})
async function submit() {
  if (saving.value) return
  error.value = ''
  const paidAt = zonedInstant(draft.paid_at, props.timezone)
  const start = draft.kind === 'subscription' && draft.period_start ? zonedInstant(`${draft.period_start}T00:00`, props.timezone) : null
  const end = draft.kind === 'subscription' && draft.period_end ? zonedInstant(`${draft.period_end}T00:00`, props.timezone) : null
  if (!draft.provider_id || !paidAt || !/^\d{1,12}(\.\d{1,8})?$/.test(draft.amount.trim()) || Number(draft.amount) <= 0 || !/^[A-Za-z]{3}$/.test(draft.currency)) {
    error.value = t('请填写提供商、有效付款时间、正数金额及三位币种代码。', 'Enter a provider, valid payment time, positive amount and three-letter currency code.')
    return
  }
  if (draft.kind === 'subscription' && ((Boolean(start) !== Boolean(end)) || (draft.period_start && !start) || (draft.period_end && !end) || (start && end && start >= end))) {
    error.value = t('请同时填写套餐开始和到期时间，且到期时间晚于开始时间。', 'Enter both plan dates, with expiry after the start.')
    return
  }
  const input = { provider_id: draft.provider_id, kind: draft.kind, amount: draft.amount.trim(), currency: draft.currency.toUpperCase(), paid_at: paidAt, period_start: start, period_end: end, note: draft.note.trim() || null, external_reference: draft.external_reference.trim() || null }
  const body = JSON.stringify(input)
  if (!pending || pending.body !== body) pending = { body, id: crypto.randomUUID() }
  saving.value = true
  try {
    await providerFinanceApi.record({ ...input, client_request_id: pending.id })
    pending = null
    Object.assign(draft, { provider_id: '', kind: 'recharge', amount: '', currency: 'USD', paid_at: '', period_start: '', period_end: '', note: '', external_reference: '' })
    emit('saved')
    emit('update:open', false)
  } catch (cause) {
    const failure = cause as { response?: { data?: { detail?: string; message?: string } }; message?: string }
    error.value = failure.response?.data?.detail || failure.response?.data?.message || failure.message || t('保存失败，请重试', 'Save failed; try again')
  } finally { saving.value = false }
}
</script>
