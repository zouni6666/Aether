<template>
  <Dialog
    :open="open"
    :title="t('ui.refundRewardReview')"
    size="lg"
    :z-index="90"
    @update:open="onOpenChange"
  >
    <div
      v-if="preview"
      class="space-y-4"
    >
      <p class="text-sm text-muted-foreground">
        {{ preview.stage === 'process' ? t('ui.reviewReferralRewardReversalsBeforeApprovingTheRefund') : t('ui.referralRewardReversalsHaveBeenCheckedAgainBeforeCompletingThe') }}
      </p>
      <div
        v-for="(item, index) in preview.rewards"
        :key="`${item.inviter_user_id}-${index}`"
        class="rounded-lg border p-3 text-sm"
      >
        <p class="font-medium">
          {{ t('ui.referrer') }}{{ item.inviter_username || item.inviter_user_id }}
        </p>
        <dl class="mt-2 grid grid-cols-2 gap-2">
          <dt>{{ t('ui.rewardsToRecoverThroughThisRefund') }}</dt><dd>{{ usd(item.expected_reversal_usd) }}</dd>
          <dt>{{ t('ui.availableGiftCredit') }}</dt><dd>{{ usd(item.available_gift_usd) }}</dd>
          <dt>{{ t('ui.estimatedRecoverableAmount') }}</dt><dd>{{ usd(item.deductible_usd) }}</dd>
          <dt>{{ t('ui.estimatedPendingReversal') }}</dt><dd>{{ usd(item.shortfall_usd) }}</dd>
        </dl>
      </div>
      <p
        v-if="preview.rewards.length === 0"
        class="text-sm"
      >
        {{ t('ui.thisRefundHasNoReferralRewardsToReverse') }}
      </p>
      <div
        v-if="hasShortfall"
        class="rounded-lg border border-amber-500/40 bg-amber-500/10 p-3 text-sm"
        role="alert"
      >
        <p>{{ t('ui.theReferrerSGiftCreditIsInsufficientAnEstimated') }} {{ usd(preview.total_shortfall_usd) }} {{ t('ui.cannotBeRecoveredImmediatelyTheAdministratorDecidesWhetherToProceed') }}</p>
        <p class="mt-1 text-muted-foreground">
          {{ t('ui.theAmountToRecoverIncludesPreviouslyUnreversedRewardsAfterApproval') }}
        </p>
        <label class="mt-3 flex items-start gap-2">
          <Checkbox
            :checked="acknowledged"
            @update:checked="acknowledged = !!$event"
          />
          <span>{{ t('ui.iUnderstandTheBalanceIsInsufficientAndAgreeToProceed') }}</span>
        </label>
      </div>
    </div>
    <template #footer>
      <Button
        :disabled="!canApprove"
        @click="resolveReview('approve')"
      >
        {{ preview?.stage === 'process' ? t('ui.approveAndProcessRefund') : t('ui.agreeToCompleteRefund') }}
      </Button>
      <Button
        v-if="allowReject"
        variant="destructive"
        @click="resolveReview('reject')"
      >
        {{ t('ui.switchToRejectingRefund') }}
      </Button>
      <Button
        variant="outline"
        @click="resolveReview('cancel')"
      >
        {{ t('common.cancel') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, onUnmounted, ref, watch } from 'vue'
import { Button, Checkbox, Dialog } from '@/components/ui'
import { adminWalletApi, type ReferralRefundPreview } from '@/api/admin-wallets'

const props = withDefaults(defineProps<{ active?: boolean; contextKey?: string }>(), { active: true, contextKey: '' })

const { t } = useI18n()

type Decision = { decision: 'approve' | 'reject' | 'cancel'; token?: string }
const open = ref(false)
const preview = ref<ReferralRefundPreview | null>(null)
const acknowledged = ref(false)
const allowReject = ref(true)
let generation = 0
let controller: AbortController | null = null
let pending: ((value: Decision) => void) | null = null
// 与服务端金额容差一致，浮点尾差不会产生无法取得确认凭据的审批。
const hasShortfall = computed(() => !!preview.value && preview.value.total_shortfall_usd > 0.00000001)
const canApprove = computed(() => !!preview.value && (!hasShortfall.value || (acknowledged.value && !!preview.value.confirmation_token)))
const usd = (value: number) => `$${value.toFixed(value !== 0 && Math.abs(value) < 0.01 ? 8 : 2)}`

async function review(walletId: string, refundId: string, stage: 'process' | 'complete', canReject = true): Promise<Decision> {
  cancelReview()
  if (!props.active) return { decision: 'cancel' }
  const currentGeneration = generation
  const requestController = new AbortController()
  controller = requestController
  acknowledged.value = false
  allowReject.value = canReject
  preview.value = null
  try {
    const latest = await adminWalletApi.getReferralRefundPreview(walletId, refundId, stage, requestController.signal)
    if (generation !== currentGeneration || !props.active) return { decision: 'cancel' }
    preview.value = latest
    open.value = true
    return new Promise(resolve => { pending = resolve })
  } catch (error) {
    if (generation !== currentGeneration || requestController.signal.aborted) return { decision: 'cancel' }
    throw error
  } finally {
    if (controller === requestController) controller = null
  }
}
function cancelReview() {
  generation += 1
  controller?.abort()
  controller = null
  resolveReview('cancel')
}
function resolveReview(decision: Decision['decision']) {
  if (decision === 'approve' && !canApprove.value) return
  const resolve = pending
  pending = null
  open.value = false
  resolve?.({ decision, token: decision === 'approve' ? preview.value?.confirmation_token || undefined : undefined })
}
function onOpenChange(value: boolean) { if (!value) cancelReview() }
watch(() => [props.active, props.contextKey], () => cancelReview())
onUnmounted(cancelReview)
defineExpose({ review })

</script>
