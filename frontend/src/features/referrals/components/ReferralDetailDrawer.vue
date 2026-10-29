<template>
  <Dialog
    :open="!!id"
    :title="t('ui.referralRewardDetails')"
    placement="right"
    size="2xl"
    @update:open="!$event && $emit('close')"
  >
    <div
      v-if="loading"
      role="status"
      class="flex items-center justify-center gap-2 py-16 text-sm text-muted-foreground"
    >
      <LoaderCircle class="h-4 w-4 animate-spin" />{{ t('ui.loadingHistoricalDetails') }}
    </div>
    <div
      v-else-if="error"
      role="alert"
      class="flex items-center gap-2 py-8 text-sm text-destructive"
    >
      <AlertCircle class="h-4 w-4" />{{ t('ui.failedToLoadDetails') }}<Button
        size="sm"
        variant="outline"
        @click="$emit('reload')"
      >
        {{ t('ui.retryLoadingDetails') }}
      </Button>
    </div>
    <div
      v-else-if="detail"
      class="space-y-5 py-3 text-sm"
    >
      <section class="overflow-hidden rounded-2xl border bg-muted/15">
        <div class="flex flex-wrap items-center gap-2 border-b px-4 py-3">
          <Badge :variant="statusVariant(detail.reward.status)">
            {{ rewardStatus(detail.reward.status) }}
          </Badge><Badge
            v-if="reversalStatus(detail.reward) !== '无冲回'"
            variant="outline"
          >
            {{ reversalStatus(detail.reward) }}
          </Badge><span class="text-xs text-muted-foreground">{{ rewardType(detail.reward.reward_type) }} · {{ trigger(detail.reward.trigger_point) }}</span>
        </div>
        <div class="grid grid-cols-2 gap-4 p-4 sm:grid-cols-3">
          <div class="col-span-2 min-w-0 sm:col-span-1">
            <p class="text-xs text-muted-foreground">
              {{ t('ui.originalReward') }}
            </p><p class="mt-1 break-all text-2xl font-semibold tracking-tight tabular-nums">
              {{ usd(detail.reward.amount_usd) }}
            </p>
          </div>
          <div class="min-w-0">
            <p class="text-xs text-muted-foreground">
              {{ t('ui.reversed') }}
            </p><p class="mt-1 break-all text-xl font-semibold tabular-nums">
              {{ usd(detail.reward.reversed_amount_usd) }}
            </p>
          </div>
          <div class="min-w-0">
            <p class="text-xs text-muted-foreground">
              {{ t('ui.outstandingReversal') }}
            </p><p
              class="mt-1 break-all text-xl font-semibold tabular-nums"
              :class="detail.reward.pending_reversal_amount_usd > 0 ? 'text-amber-700 dark:text-amber-400' : 'text-muted-foreground'"
            >
              {{ usd(detail.reward.pending_reversal_amount_usd) }}
            </p>
          </div>
        </div>
      </section>
      <section class="rounded-xl border p-4">
        <h3 class="mb-3 flex items-center gap-2 font-semibold">
          <Users class="h-4 w-4 text-muted-foreground" />{{ t('ui.referralParties') }}
        </h3>
        <div class="grid gap-3 sm:grid-cols-2">
          <div class="flex items-start gap-3">
            <span
              class="avatar bg-primary/10 text-primary"
              aria-hidden="true"
            >{{ initial(detail.reward.inviter_username || detail.reward.inviter_user_id) }}</span><div class="min-w-0 flex-1">
              <p class="text-xs text-muted-foreground">
                {{ t('ui.inviter') }}
              </p><p class="mt-0.5 break-all font-medium">
                {{ detail.reward.inviter_username || detail.reward.inviter_user_id }}
              </p>
            </div><Button
              variant="ghost"
              size="sm"
              class="copy-button"
              :aria-label="t('ui.copyInviterId')"
              @click="$emit('copy', detail.reward.inviter_user_id)"
            >
              <Copy class="h-3.5 w-3.5" />
            </Button>
          </div>
          <div class="flex items-start gap-3">
            <span
              class="avatar bg-muted text-muted-foreground"
              aria-hidden="true"
            >{{ initial(detail.reward.invitee_username || detail.reward.invitee_user_id) }}</span><div class="min-w-0 flex-1">
              <p class="text-xs text-muted-foreground">
                {{ t('ui.invitee') }}
              </p><p class="mt-0.5 break-all font-medium">
                {{ detail.reward.invitee_username || detail.reward.invitee_user_id }}
              </p>
            </div><Button
              variant="ghost"
              size="sm"
              class="copy-button"
              :aria-label="t('ui.copyInviteeId')"
              @click="$emit('copy', detail.reward.invitee_user_id)"
            >
              <Copy class="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
        <div class="mt-4 flex flex-wrap items-center justify-between gap-2 border-t pt-3">
          <p
            v-if="detail.relationship"
            class="text-xs leading-5 text-muted-foreground"
          >
            {{ t('ui.bind') }} {{ date(detail.relationship.created_at_unix_secs) }}<br>{{ detail.relationship.first_paid_order_id ? t('ui.firstPaymentMadePaymentRecorded') : t('ui.noFirstPayment') }}
          </p><p
            v-else
            class="text-xs text-muted-foreground"
          >
            {{ t('ui.historicalReferralRelationshipNotFound') }}
          </p><RouterLink
            v-if="detail.reward.inviter_wallet_id"
            class="detail-link"
            :to="{ path: '/admin/wallets', query: { tab: 'wallets', wallet_id: detail.reward.inviter_wallet_id } }"
          >
            <Wallet class="h-3.5 w-3.5" />{{ t('ui.openInviterWallet') }}<ArrowUpRight class="h-3 w-3" />
          </RouterLink>
        </div>
      </section>
      <div class="grid gap-4 sm:grid-cols-2">
        <section class="rounded-xl border p-4">
          <h3 class="mb-3 flex items-center gap-2 font-semibold">
            <ReceiptText class="h-4 w-4 text-muted-foreground" />{{ t('ui.rewardSource') }}
          </h3>
          <template v-if="detail.source_order">
            <p class="break-all font-medium">
              {{ detail.source_order.order_no }}
            </p><p class="mt-1 text-xs text-muted-foreground">
              {{ orderStatus(detail.source_order.status) }}
            </p><dl class="mt-3 space-y-2 text-xs">
              <div class="flex flex-wrap justify-between gap-1">
                <dt class="text-muted-foreground">
                  {{ t('ui.orderAmountf827fdb') }}
                </dt><dd class="tabular-nums">
                  {{ usd(detail.source_order.amount_usd) }}
                </dd>
              </div><div class="flex flex-wrap justify-between gap-1">
                <dt class="text-muted-foreground">
                  {{ t('ui.totalRefundedcbcd6d2') }}
                </dt><dd class="tabular-nums">
                  {{ usd(detail.source_order.refunded_amount_usd) }}
                </dd>
              </div>
            </dl><RouterLink
              class="detail-link mt-3"
              :to="{ path: '/admin/wallets', query: { tab: 'orders', order_id: detail.source_order.id } }"
            >
              {{ t('ui.openSourceOrder') }}<ArrowUpRight class="h-3 w-3" />
            </RouterLink>
          </template>
          <p
            v-else
            class="break-all text-muted-foreground"
          >
            {{ source(detail.reward) }}
          </p>
        </section>
        <section class="rounded-xl border p-4">
          <h3 class="mb-3 flex items-center gap-2 font-semibold">
            <History class="h-4 w-4 text-muted-foreground" />{{ t('ui.rulesAtTheTime') }}
          </h3>
          <template v-if="detail.rule_snapshot">
            <p
              v-if="detail.rule_snapshot.percent_enabled"
              class="font-medium"
            >
              {{ t('ui.percentageReward') }} {{ detail.rule_snapshot.percent_rate }}%
            </p><p
              v-if="detail.rule_snapshot.headcount_enabled"
              class="mt-1 break-all font-medium"
            >
              {{ t('ui.perUser') }} {{ usd(detail.rule_snapshot.headcount_amount_usd) }} · {{ trigger(detail.rule_snapshot.headcount_trigger) }}
            </p>
          </template><p
            v-else
            class="text-muted-foreground"
          >
            {{ t('ui.historicalRulesWereNotRecorded') }}
          </p>
        </section>
      </div>
      <section>
        <h3 class="mb-3 flex items-center justify-between font-semibold">
          <span class="flex items-center gap-2"><ListOrdered class="h-4 w-4 text-muted-foreground" />{{ t('ui.grantAndReversalLedger') }}</span><span class="text-xs font-normal text-muted-foreground">{{ detail.ledger_entries.length }} {{ t('ui.itemsf004f1d') }}</span>
        </h3>
        <p
          v-if="!detail.ledger_entries.length"
          class="rounded-xl border border-dashed p-4 text-xs text-muted-foreground"
        >
          {{ t('ui.noAssociatedLedgerEntriesRecorded') }}
        </p>
        <div
          v-else
          class="divide-y overflow-hidden rounded-xl border"
        >
          <div
            v-for="entry in detail.ledger_entries"
            :key="entry.id"
            class="space-y-2 p-3"
          >
            <div class="flex items-start justify-between gap-3">
              <div>
                <p class="font-medium">
                  {{ entry.reason_code === 'referral_reward' ? t('ui.rewardIssued') : ['referral_reward_reversal', 'referral_reversal'].includes(entry.reason_code) ? t('ui.rewardReversed') : entry.reason_code }}
                </p><p class="mt-1 text-xs text-muted-foreground">
                  {{ date(entry.created_at_unix_secs) }}
                </p>
              </div><p class="break-all text-right font-semibold tabular-nums">
                {{ usd(entry.amount_usd) }}
              </p>
            </div>
            <div class="flex flex-wrap items-center justify-between gap-2">
              <details class="min-w-0 text-xs text-muted-foreground">
                <summary class="cursor-pointer hover:text-foreground">
                  {{ t('ui.ledgerId') }}
                </summary><p class="mt-2 break-all font-mono">
                  {{ entry.id }}
                </p>
              </details><div class="flex items-center gap-2">
                <Button
                  size="sm"
                  variant="ghost"
                  class="copy-button"
                  :aria-label="t('ui.copyLedgerId')"
                  @click="$emit('copy', entry.id)"
                >
                  <Copy class="h-3.5 w-3.5" />
                </Button><RouterLink
                  v-if="detail.reward.inviter_wallet_id"
                  class="detail-link"
                  :to="{ path: '/admin/wallets', query: { tab: 'ledger', wallet_id: detail.reward.inviter_wallet_id, transaction_id: entry.id } }"
                >
                  {{ t('ui.openLedgerEntry') }}<ArrowUpRight class="h-3 w-3" />
                </RouterLink>
              </div>
            </div>
          </div>
        </div>
      </section>
      <section>
        <h3 class="mb-2 flex items-center gap-2 font-semibold">
          <Undo2 class="h-4 w-4 text-muted-foreground" />{{ t('ui.relatedRefunds') }}
        </h3>
        <p
          v-if="!detail.refunds.length"
          class="rounded-xl border border-dashed p-4 text-xs text-muted-foreground"
        >
          {{ t('ui.noRelatedRefunds') }}
        </p>
        <div
          v-else
          class="divide-y overflow-hidden rounded-xl border"
        >
          <div
            v-for="refund in detail.refunds"
            :key="refund.id"
            class="space-y-2 p-3"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <p class="break-all font-medium">
                  {{ refund.refund_no }}
                </p><p class="mt-1 text-xs text-muted-foreground">
                  {{ refundStatus(refund.status) }} · {{ date(refund.created_at_unix_secs) }}
                </p>
              </div><p class="break-all text-right font-semibold tabular-nums">
                {{ usd(refund.refund_amount_usd) }}
              </p>
            </div><RouterLink
              class="detail-link"
              :to="{ path: '/admin/wallets', query: { tab: 'refunds', wallet_id: refund.wallet_id, refund_id: refund.id } }"
            >
              {{ t('ui.openRefundDetails') }}<ArrowUpRight class="h-3 w-3" />
            </RouterLink>
          </div>
        </div>
      </section>
      <details class="rounded-xl border">
        <summary class="cursor-pointer px-4 py-3 text-xs font-medium text-muted-foreground hover:text-foreground">
          {{ t('ui.recordInformationAndOperatorNotes') }}
        </summary>
        <dl class="grid grid-cols-[5rem_minmax(0,1fr)] gap-x-3 gap-y-3 border-t p-4 text-xs">
          <template v-if="detail.reward.status === 'failed'">
            <dt class="text-muted-foreground">
              {{ t('ui.failureReason') }}
            </dt><dd>{{ t('ui.noFailureReasonRecorded') }}</dd>
          </template>
          <dt class="text-muted-foreground">
            {{ t('ui.rewardId') }}
          </dt><dd class="flex min-w-0 items-start gap-1">
            <span class="min-w-0 flex-1 break-all font-mono">{{ detail.reward.id }}</span><Button
              variant="ghost"
              size="sm"
              class="copy-button"
              :aria-label="t('ui.copyRewardId')"
              @click="$emit('copy', detail.reward.id)"
            >
              <Copy class="h-3.5 w-3.5" />
            </Button>
          </dd>
          <dt class="text-muted-foreground">
            {{ t('ui.relationshipId') }}
          </dt><dd class="flex min-w-0 items-start gap-1">
            <span class="min-w-0 flex-1 break-all font-mono">{{ detail.reward.referral_id }}</span><Button
              variant="ghost"
              size="sm"
              class="copy-button"
              :aria-label="t('ui.copyReferralId')"
              @click="$emit('copy', detail.reward.referral_id)"
            >
              <Copy class="h-3.5 w-3.5" />
            </Button>
          </dd>
          <dt class="text-muted-foreground">
            {{ t('ui.createdAt') }}
          </dt><dd>{{ date(detail.reward.created_at_unix_secs) }}</dd><dt class="text-muted-foreground">
            {{ t('ui.updatedAt') }}
          </dt><dd>{{ date(detail.reward.updated_at_unix_secs) }}</dd><dt class="text-muted-foreground">
            {{ t('auth.role.admin') }}
          </dt><dd class="break-all">
            {{ detail.reward.admin_operator_id || t('ui.notRecorded') }}
          </dd><dt class="text-muted-foreground">
            {{ t('ui.adminNotes') }}
          </dt><dd class="whitespace-pre-wrap break-all">
            {{ detail.reward.admin_note || t('ui.none484d556') }}
          </dd>
        </dl>
      </details>
    </div>
    <template #footer>
      <Button
        variant="outline"
        @click="$emit('close')"
      >
        {{ t('ui.closeDetails') }}
      </Button><Button
        v-if="detail?.reward.status === 'failed'"
        :disabled="!!mutating"
        @click="$emit('operate', 'retry', detail.reward)"
      >
        {{ t('ui.retryGrant') }}
      </Button><Button
        v-if="detail && ['pending', 'failed'].includes(detail.reward.status)"
        variant="outline"
        class="text-destructive hover:border-destructive/40 hover:bg-destructive/5 hover:text-destructive"
        :disabled="!!mutating"
        @click="$emit('operate', 'void', detail.reward)"
      >
        {{ t('ui.void') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { AlertCircle, ArrowUpRight, Copy, History, ListOrdered, LoaderCircle, ReceiptText, Undo2, Users, Wallet } from 'lucide-vue-next'
import { RouterLink } from 'vue-router'
import type { ReferralRewardDetail, ReferralRewardRecord } from '@/api/referrals'
import { Badge, Button, Dialog } from '@/components/ui'
import { date, orderStatus, refundStatus, reversalStatus, rewardStatus, rewardType, source, statusVariant, trigger, usd } from '../presentation'

defineProps<{ id: string | null; detail: ReferralRewardDetail | null; loading: boolean; error: boolean; mutating: string | null }>()
defineEmits<{ close: []; reload: []; copy: [value: string]; operate: [action: 'retry' | 'void', reward: ReferralRewardRecord] }>()

const { t } = useI18n()
const initial = (name: string) => Array.from(name)[0]?.toLocaleUpperCase() || '?'

</script>

<style scoped>
.avatar { @apply flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-xs font-semibold; }
.copy-button { @apply h-7 w-7 shrink-0 rounded-md p-0 text-muted-foreground; }
.detail-link { @apply inline-flex items-center gap-1 text-xs font-medium text-primary hover:underline; }
</style>
