<template>
  <span
    v-if="provider"
    class="text-xs tabular-nums text-muted-foreground"
    :title="error?.message"
  >
    <template v-if="balanceLoading">余额查询中</template>
    <template v-else-if="balance">余额 {{ balances?.formatBalanceDisplay(balance) }}</template>
    <template v-else-if="error">余额暂不可用</template>
    <template v-else-if="provider.billing_type === 'monthly_quota'">
      月度已用 ${{ (provider.monthly_used_usd ?? 0).toFixed(2) }} / ${{ (provider.monthly_quota_usd ?? 0).toFixed(2) }}
    </template>
    <template v-else-if="provider.ops_configured">余额 —</template>
  </span>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue'
import type { ProviderWithEndpointsSummary } from '@/api/endpoints'
import { useSchedulingProviderBalance } from '../composables/useSchedulingProviderBalance'

const props = defineProps<{ provider?: ProviderWithEndpointsSummary }>()
const balances = useSchedulingProviderBalance()
const balance = computed(() => props.provider ? balances?.getProviderBalance(props.provider.id) : null)
const error = computed(() => props.provider ? balances?.getProviderBalanceError(props.provider.id) : null)
const balanceLoading = computed(() => props.provider ? balances?.isBalanceLoading(props.provider.id) : false)
watch(() => [props.provider?.id, props.provider?.ops_configured], () => {
  if (props.provider) balances?.register(props.provider)
}, { immediate: true })
</script>
