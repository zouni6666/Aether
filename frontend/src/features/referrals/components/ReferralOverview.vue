<template>
  <section
    :aria-label="summary ? t('ui.referralAndRuleSummary') : t('ui.allHistoryStatistics')"
    class="space-y-3"
  >
    <div
      v-if="!summary && error"
      role="alert"
      class="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-destructive/30 px-4 py-3 text-sm text-destructive"
    >
      <span>{{ t('ui.globalTotalsAndRulesFailedToLoadListsCanStill') }}</span>
      <Button
        variant="ghost"
        size="sm"
        @click="$emit('reload')"
      >
        {{ t('ui.retryOverview') }}
      </Button>
    </div>
    <div
      v-if="!summary"
      class="grid grid-cols-2 gap-3 sm:grid-cols-3"
    >
      <Card
        v-for="item in amounts"
        :key="item.label"
        class="min-w-0 p-4 sm:px-5"
        :class="item.label === '累计已发' ? 'col-span-2 sm:col-span-1' : ''"
      >
        <div class="flex items-center justify-between gap-3">
          <p class="text-xs text-muted-foreground">
            {{ item.label }}
          </p>
          <component
            :is="item.icon"
            class="h-4 w-4"
            :class="item.emphasized ? 'text-primary' : 'text-muted-foreground/70'"
          />
        </div>
        <p
          class="mt-2 overflow-x-auto overflow-y-hidden whitespace-nowrap font-semibold tabular-nums tracking-tight"
          :class="[String(item.value).length > 12 ? 'text-xl' : 'text-2xl', item.emphasized ? 'text-primary' : 'text-foreground']"
        >
          {{ display(item.value) }}
        </p>
      </Card>
    </div>
    <div
      v-if="summary"
      class="flex flex-wrap items-center gap-x-5 gap-y-2 text-xs text-muted-foreground"
    >
      <span class="inline-flex items-center gap-1.5"><Users class="h-3.5 w-3.5" />{{ t('ui.totalReferrals') }} <strong class="font-medium tabular-nums text-foreground">{{ display(overview?.stats.total_invites) }}</strong></span>
      <span>{{ t('ui.invitationsWithAFirstPayment') }} <strong class="ml-1 font-medium tabular-nums text-foreground">{{ display(overview?.stats.effective_invites) }}</strong></span>
      <span>{{ t('ui.reversed') }} <strong class="ml-1 font-medium tabular-nums text-foreground">{{ display(usd(overview?.stats.reversed_reward_usd || 0)) }}</strong></span>
    </div>
    <div
      v-if="summary"
      class="flex flex-wrap items-center gap-x-3 gap-y-2 text-xs"
    >
      <span class="inline-flex items-center gap-2 font-medium text-foreground"><Settings2 class="h-3.5 w-3.5 text-muted-foreground" />{{ t('ui.currentRules') }}</span>
      <template v-if="overview && !error">
        <Badge
          :variant="overview.rules.enabled && overview.rules.available ? 'success' : 'secondary'"
          class="text-[10px]"
        >
          {{ !overview.rules.available ? t('ui.moduleUnavailable') : overview.rules.enabled ? t('ui.enabled8a4ef3e') : t('ui.disabled3cffa97') }}
        </Badge>
        <span
          v-if="overview.rules.reward_mode !== 'headcount'"
          class="text-muted-foreground"
        >{{ t('ui.percentage') }} {{ overview.rules.recharge_percent }}%</span>
        <span
          v-if="overview.rules.reward_mode !== 'percent'"
          class="text-muted-foreground"
        >{{ t('ui.perUser') }} {{ usd(overview.rules.headcount_amount_usd) }} · {{ trigger(overview.rules.headcount_trigger) }}</span>
      </template>
      <span
        v-else
        class="text-muted-foreground"
      >{{ loading ? t('ui.loading21bd738') : t('ui.rulesUnavailable') }}</span>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed } from 'vue'
import { Badge, Button, Card } from '@/components/ui'
import { Wallet, Clock3, RotateCcw, Users, Settings2 } from 'lucide-vue-next'
import type { AdminReferralOverview } from '@/api/referrals'
import { trigger, usd } from '../presentation'
const props = defineProps<{ overview: AdminReferralOverview | null; loading: boolean; error: boolean; summary?: boolean }>()
defineEmits<{ reload: [] }>()

const { t } = useI18n()
function display(value: string | number | undefined) { return props.loading ? '…' : props.error || !props.overview ? '—' : value }
const amounts = computed(() => {
  const s = props.overview?.stats
  return [
    { get label() { return t('ui.totalGranted') }, value: usd(s?.cumulative_reward_usd || 0), icon: Wallet, emphasized: false },
    { get label() { return t('ui.awaitingGrant') }, value: usd(s?.pending_reward_usd || 0), icon: Clock3, emphasized: false },
    { get label() { return t('ui.outstandingReversal') }, value: usd(s?.pending_reversal_reward_usd || 0), icon: RotateCcw, emphasized: true },
  ]
})

</script>
