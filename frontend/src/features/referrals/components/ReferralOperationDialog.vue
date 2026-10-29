<template>
  <Dialog
    :open="!!operation"
    :title="operation?.action === 'void' ? t('ui.confirmVoidReward') : t('ui.confirmGrantRetry')"
    :persistent="busy"
    :z-index="90"
    size="lg"
    @update:open="!$event && $emit('close')"
  >
    <div
      v-if="operation"
      class="space-y-3 py-3 text-sm"
    >
      <p class="break-all">
        {{ operation.reward.inviter_username || operation.reward.inviter_user_id }} ← {{ operation.reward.invitee_username || operation.reward.invitee_user_id }}
      </p>
      <p class="break-all font-mono text-xs text-muted-foreground">
        {{ t('ui.reward') }} {{ operation.reward.id }}
      </p><p>{{ t('ui.originalReward') }} {{ usd(operation.reward.amount_usd) }} · {{ trigger(operation.reward.trigger_point) }}</p>
      <p
        v-if="loading"
        role="status"
      >
        {{ t('ui.checkingTheLatestStatusAndHistoricalRules') }}
      </p><p
        v-else-if="error"
        role="alert"
        class="text-destructive"
      >
        {{ t('ui.verificationFailedCloseAndTryAgain') }}
      </p>
      <template v-else-if="detail">
        <p>{{ t('ui.currentStatus') }}{{ rewardStatus(detail.reward.status) }}</p><p v-if="detail.rule_snapshot">
          {{ t('ui.rulesAtTheTimecebb87d') }}{{ detail.rule_snapshot.percent_enabled ? t('ui.percentageValue', { value: detail.rule_snapshot.percent_rate }) : '' }} {{ detail.rule_snapshot.headcount_enabled ? t('ui.perUserValueValue', { count: usd(detail.rule_snapshot.headcount_amount_usd), count2: trigger(detail.rule_snapshot.headcount_trigger) }) : '' }}
        </p><p v-else>
          {{ t('ui.historicalRulesWereNotRecordedRetryingWillNotApplyThe') }}
        </p><p
          v-if="detail.source_order && detail.source_order.refunded_amount_usd > 0"
          class="rounded-lg bg-amber-500/10 p-3"
        >
          {{ t('ui.theSourceOrderWasRefundedOnlyTheGiftCreditRemaining') }}
        </p>
      </template>
      <p
        v-if="operation.action === 'void'"
        class="rounded-lg bg-muted/40 p-3"
      >
        {{ t('ui.stopFutureGrantAttemptsForThisRewardChangingRulesWill') }}
      </p>
      <label
        class="block"
        for="referral-operation-note"
      >{{ operation.action === 'void' ? t('ui.reasonForVoidingRequired') : t('ui.operatorNoteOptional') }}</label><Textarea
        id="referral-operation-note"
        v-model="note"
        :disabled="busy"
        :maxlength="1000"
        :placeholder="t('ui.explainTheReasonForFutureReview')"
      />
    </div>
    <template #footer>
      <Button
        :variant="operation?.action === 'void' ? 'destructive' : 'default'"
        :disabled="!canConfirm"
        @click="$emit('confirm', note.trim())"
      >
        {{ busy ? t('ui.processing574ec75') : operation?.action === 'void' ? t('ui.confirmVoid') : t('ui.confirmRetry') }}
      </Button><Button
        variant="outline"
        :disabled="busy"
        @click="$emit('close')"
      >
        {{ t('common.cancel') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, ref, watch } from 'vue'
import type { ReferralRewardDetail, ReferralRewardRecord } from '@/api/referrals'
import { Button, Dialog, Textarea } from '@/components/ui'
import { rewardStatus, trigger, usd } from '../presentation'
export interface RewardOperation { action: 'retry' | 'void'; reward: ReferralRewardRecord }
const props = defineProps<{ operation: RewardOperation | null; detail: ReferralRewardDetail | null; loading: boolean; error: boolean; busy: boolean }>()
defineEmits<{ close: []; confirm: [note: string] }>()

const { t } = useI18n()
const note = ref('')
watch(() => props.operation, () => { note.value = '' })
const canConfirm = computed(() => !!props.operation && !props.busy && !props.loading && !props.error && !!props.detail && props.detail.reward.id === props.operation.reward.id && (props.operation.action === 'retry' ? props.detail.reward.status === 'failed' : ['pending', 'failed'].includes(props.detail.reward.status)) && (props.operation.action !== 'void' || !!note.value.trim()))

</script>
