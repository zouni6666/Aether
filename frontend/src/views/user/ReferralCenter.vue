<template>
  <div class="max-w-6xl space-y-6 pb-8">
    <div>
      <h1 class="text-2xl font-semibold text-foreground">
        {{ t('nav.myReferral') }}
      </h1>
      <p class="mt-1 text-sm text-muted-foreground">
        {{ t('ui.shareYourInviteLinkToInviteFriendsToSignUp') }}
      </p>
    </div>

    <Card
      v-if="loading"
      class="flex items-center gap-3 p-6 text-sm text-muted-foreground"
      role="status"
    >
      <Loader2
        class="h-4 w-4 animate-spin"
        aria-hidden="true"
      /> {{ t('site.privacy.loading') }}
    </Card>

    <template v-else-if="dashboard">
      <Card class="overflow-hidden">
        <div class="space-y-6 p-5 sm:p-6">
          <div class="flex items-start gap-3">
            <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary">
              <Gift
                class="h-5 w-5"
                aria-hidden="true"
              />
            </div>
            <div>
              <h2 class="text-lg font-semibold">
                {{ t('ui.inviteFriends') }}
              </h2>
              <p class="mt-1 text-sm text-muted-foreground">
                {{ t('ui.sendThisLinkToFriendsYourInviteCodeIsIncluded') }}
              </p>
            </div>
          </div>

          <div>
            <Label for="referral-invitation-link"> {{ t('ui.inviteLink') }} </Label>
            <div class="mt-2 flex min-w-0 flex-col gap-3 sm:flex-row">
              <Input
                id="referral-invitation-link"
                :model-value="invitationLink"
                readonly
                class="min-w-0 font-mono text-sm sm:flex-1"
                @focus="selectInvitationLink"
              />
              <Button
                type="button"
                class="shrink-0 gap-2"
                @click="copyToClipboard(invitationLink)"
              >
                <Copy
                  class="h-4 w-4"
                  aria-hidden="true"
                /> {{ t('ui.copyInviteLink') }}
              </Button>
            </div>
          </div>
        </div>

        <div class="flex flex-wrap items-center gap-x-4 gap-y-2 border-t border-border/60 bg-muted/30 px-5 py-3 sm:px-6">
          <span class="text-sm text-muted-foreground">{{ t('auth.register.inviteCodeLabel') }}</span>
          <code class="break-all font-mono text-sm font-semibold tracking-wide">{{ dashboard.invite_code }}</code>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            class="gap-2"
            @click="copyToClipboard(dashboard.invite_code)"
          >
            <Copy
              class="h-3.5 w-3.5"
              aria-hidden="true"
            /> {{ t('ui.copyInviteCode') }}
          </Button>
          <span class="text-xs text-muted-foreground">{{ t('ui.yourInviteCodeIsIncludedInTheLinkAbove') }}</span>
        </div>
      </Card>

      <Card class="p-5 sm:p-6">
        <div class="mb-5">
          <h2 class="text-base font-semibold">
            {{ t('ui.referralsAndRewards') }}
          </h2>
          <p class="mt-1 text-sm text-muted-foreground">
            {{ t('ui.eligibleRewardsAreCreditedToYourGiftBalanceAndCan') }}
          </p>
        </div>

        <dl class="grid grid-cols-2 gap-x-6 gap-y-5 sm:grid-cols-3 lg:grid-cols-5">
          <div>
            <dt class="text-xs text-muted-foreground">
              {{ t('ui.totalReferrals') }}
            </dt>
            <dd class="mt-2 text-2xl font-semibold tabular-nums">
              {{ dashboard.summary.total_invites }}
            </dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">
              {{ t('ui.effectiveReferrals') }}
            </dt>
            <dd class="mt-2 text-2xl font-semibold tabular-nums">
              {{ dashboard.summary.effective_invites }}
            </dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">
              {{ t('ui.issuedRewards') }}
            </dt>
            <dd class="mt-2 break-all text-2xl font-semibold tabular-nums text-primary">
              {{ formatUsd(dashboard.summary.paid_reward_usd) }}
            </dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">
              {{ t('ui.pendingRewards') }}
            </dt>
            <dd class="mt-2 break-all text-2xl font-semibold tabular-nums">
              {{ formatUsd(dashboard.summary.pending_reward_usd) }}
            </dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">
              {{ t('ui.reversedRewards') }}
            </dt>
            <dd class="mt-2 break-all text-2xl font-semibold tabular-nums">
              {{ formatUsd(dashboard.summary.reversed_reward_usd) }}
            </dd>
          </div>
        </dl>

        <div
          v-if="dashboard.summary.total_invites === 0"
          class="mt-5 flex items-start gap-3 rounded-xl bg-muted/40 p-4"
        >
          <Users
            class="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground"
            aria-hidden="true"
          />
          <div>
            <p class="text-sm font-medium">
              {{ t('ui.noReferralsYet') }}
            </p>
            <p class="mt-1 text-sm text-muted-foreground">
              {{ t('ui.copyTheLinkAboveToInviteYourFirstFriend') }}
            </p>
          </div>
        </div>

        <div class="mt-5 space-y-1 border-t border-border/60 pt-4 text-xs leading-relaxed text-muted-foreground">
          <p>{{ t('ui.effectiveReferralsInvitedUsersWhoHaveCompletedTheirFirstPayment') }}</p>
          <p>{{ t('ui.pendingRewardsHaveNotBeenCreditedYetSeeTheWallet') }}</p>
        </div>
      </Card>
    </template>

    <Card
      v-else
      class="space-y-3 p-6"
    >
      <p class="text-sm text-muted-foreground">
        {{ t('ui.referralDataIsCurrentlyUnavailable') }}
      </p>
      <Button
        type="button"
        variant="outline"
        size="sm"
        class="gap-2"
        @click="loadReferralDashboard"
      >
        <RefreshCw
          class="h-4 w-4"
          aria-hidden="true"
        /> {{ t('ui.reload') }}
      </Button>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { computed, onMounted, ref } from 'vue'
import { Copy, Gift, Loader2, RefreshCw, Users } from 'lucide-vue-next'
import { referralApi, type ReferralDashboardResponse } from '@/api/referrals'
import { Button, Card, Input, Label } from '@/components/ui'
import { useClipboard } from '@/composables/useClipboard'
import { useToast } from '@/composables/useToast'

const dashboard = ref<ReferralDashboardResponse | null>(null)
const loading = ref(false)
const { copyToClipboard } = useClipboard()
const { error: showError } = useToast()

// 后端默认返回相对路径，分享时需补全当前站点的协议、域名和端口。
const invitationLink = computed(() => dashboard.value
  ? new URL(dashboard.value.invitation_link, window.location.origin).href
  : '')

function selectInvitationLink(event: FocusEvent) {
  (event.target as HTMLInputElement).select()
}

function formatUsd(value: number): string {
  return `$${Number(value || 0).toFixed(2)}`
}

async function loadReferralDashboard() {
  loading.value = true
  try {
    dashboard.value = await referralApi.getMyReferral()
  } catch {
    dashboard.value = null
    showError(t('ui.failedToLoadReferralData'))
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void loadReferralDashboard()
})

</script>
