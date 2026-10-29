<template>
  <form
    class="min-w-0"
    @submit.prevent="$emit('query')"
  >
    <div class="flex flex-wrap items-center gap-2">
      <ReferralUserSelect
        v-model="filters.inviter"
        class="w-full sm:w-36"
        :label="t('ui.inviter')"
      />
      <ReferralUserSelect
        v-model="filters.invitee"
        class="w-full sm:w-36"
        :label="t('ui.invitee')"
      />
      <Input
        v-model="filters.order_no"
        class="w-full sm:w-40"
        :placeholder="t('ui.orderNumber')"
        :aria-label="t('ui.orderNumber')"
      />
      <select
        v-if="full"
        v-model="filters.status"
        class="h-9 w-full rounded-md border border-border bg-background px-3 text-sm sm:w-auto"
        :aria-label="t('ui.issuanceStatus')"
      >
        <option value="all">
          {{ t('ui.allIssuanceStatuses') }}
        </option>
        <option value="pending">
          {{ t('ui.pendingIssue') }}
        </option>
        <option value="applying">
          {{ t('ui.processing694b71b') }}
        </option>
        <option value="failed">
          {{ t('ui.grantFailed') }}
        </option>
        <option value="applied">
          {{ t('ui.issued') }}
        </option>
        <option value="reversed">
          {{ t('ui.issuedAndFullyReversed') }}
        </option>
        <option value="voided">
          {{ t('ui.voided') }}
        </option>
      </select>
      <div class="flex flex-wrap items-center gap-1 sm:ml-auto">
        <Button
          type="submit"
          size="sm"
          :aria-busy="loading"
        >
          <Search class="mr-1.5 h-3.5 w-3.5" />{{ t('ui.search') }}
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          @click="$emit('reset')"
        >
          {{ t('ui.reset') }}
        </Button>
        <Button
          v-if="full"
          type="button"
          variant="ghost"
          size="sm"
          :aria-label="t('ui.moreFilters')"
          :aria-expanded="expanded"
          @click="expanded = !expanded"
        >
          <SlidersHorizontal class="mr-1.5 h-3.5 w-3.5" />{{ t('ui.moreFilters') }}<span
            v-if="advancedCount"
            class="ml-1 text-primary"
          >{{ advancedCount }}</span>
        </Button>
      </div>
    </div>
    <div
      v-if="full && expanded"
      class="mt-3 grid gap-2 border-t border-border pt-3 sm:grid-cols-2 lg:grid-cols-4"
    >
      <label class="space-y-1 text-xs text-muted-foreground">
        <span>{{ t('ui.referralRelationships') }}</span><Input
          v-model="filters.referral_id"
          :placeholder="t('ui.relationshipId')"
          :aria-label="t('ui.relationshipId')"
        />
      </label>
      <label class="space-y-1 text-xs text-muted-foreground">
        <span>{{ t('ui.rewardType') }}</span>
        <select
          v-model="filters.reward_type"
          class="h-9 w-full rounded-md border border-border bg-background px-3 text-sm text-foreground"
          :aria-label="t('ui.rewardType')"
        >
          <option value="all">{{ t('ui.allTypes') }}</option><option value="percent">{{ t('ui.percentageReward') }}</option><option value="headcount">{{ t('ui.perUserReward') }}</option>
        </select>
      </label>
      <label class="space-y-1 text-xs text-muted-foreground">
        <span>{{ t('ui.triggerCondition') }}</span>
        <select
          v-model="filters.trigger_point"
          class="h-9 w-full rounded-md border border-border bg-background px-3 text-sm text-foreground"
          :aria-label="t('ui.triggerCondition')"
        >
          <option value="all">{{ t('ui.allTriggerConditions') }}</option><option value="registration">{{ t('auth.register.submit') }}</option><option value="email_verified">{{ t('ui.emailVerification') }}</option><option value="first_paid_order">{{ t('ui.firstActualPayment') }}</option><option value="paid_order">{{ t('ui.actualPayment') }}</option>
        </select>
      </label>
      <label class="space-y-1 text-xs text-muted-foreground">
        <span>{{ t('ui.reversalStatus') }}</span>
        <select
          v-model="filters.pending_reversal"
          class="h-9 w-full rounded-md border border-border bg-background px-3 text-sm text-foreground"
          :aria-label="t('ui.pendingReversalFilter')"
        >
          <option value="all">{{ t('ui.allReversalStatuses') }}</option><option value="true">{{ t('ui.hasPendingReversals') }}</option><option value="false">{{ t('ui.noPendingReversals') }}</option>
        </select>
      </label>
    </div>
  </form>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, ref } from 'vue'
import { Search, SlidersHorizontal } from 'lucide-vue-next'
import { Button, Input } from '@/components/ui'
import ReferralUserSelect from './ReferralUserSelect.vue'
export interface RewardFilters { inviter: string; invitee: string; order_no: string; referral_id: string; reward_type: string; status: string; trigger_point: string; pending_reversal: string }
defineProps<{ loading: boolean; full?: boolean }>()
defineEmits<{ query: []; reset: [] }>()

const { t } = useI18n()
const filters = defineModel<RewardFilters>('filters', { required: true })
const expanded = ref(false)
const advancedCount = computed(() => [filters.value.referral_id.trim(), ...[filters.value.reward_type, filters.value.trigger_point, filters.value.pending_reversal].map(value => value === 'all' ? '' : value)].filter(Boolean).length)

</script>
