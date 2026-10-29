<template>
  <span
    v-if="provider"
    class="text-xs tabular-nums text-muted-foreground"
    :title="error?.message"
  >
    <template v-if="balanceLoading">{{ t('ui.checkingBalance') }}</template>
    <template v-else-if="balance">{{ t('ui.balance') }} {{ balances?.formatBalanceDisplay(balance) }}</template>
    <template v-else-if="error">{{ t('ui.balanceTemporarilyUnavailable') }}</template>
    <template v-else-if="provider.billing_type === 'monthly_quota'"> {{ t('ui.monthlyUsage') }}{{ (provider.monthly_used_usd ?? 0).toFixed(2) }} / ${{ (provider.monthly_quota_usd ?? 0).toFixed(2) }}
    </template>
    <template v-else-if="provider.ops_configured">{{ t('ui.balance168e5e0') }}</template>
  </span>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, watch } from 'vue'
import type { ProviderWithEndpointsSummary } from '@/api/endpoints'
import { useSchedulingProviderBalance } from '../composables/useSchedulingProviderBalance'

const props = defineProps<{ provider?: ProviderWithEndpointsSummary }>()

const { t } = useI18n()
const balances = useSchedulingProviderBalance()
const balance = computed(() => props.provider ? balances?.getProviderBalance(props.provider.id) : null)
const error = computed(() => props.provider ? balances?.getProviderBalanceError(props.provider.id) : null)
const balanceLoading = computed(() => props.provider ? balances?.isBalanceLoading(props.provider.id) : false)
watch(() => [props.provider?.id, props.provider?.ops_configured], () => {
  if (props.provider) balances?.register(props.provider)
}, { immediate: true })

</script>
