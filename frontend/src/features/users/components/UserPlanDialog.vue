<template>
  <Dialog
    :model-value="open"
    size="xl"
    @update:model-value="(value) => !value && $emit('close')"
  >
    <template #header>
      <div class="border-b border-border px-6 py-4">
        <div class="flex items-center gap-3">
          <div class="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-lg bg-kraft/10">
            <PackageCheck class="h-5 w-5 text-kraft" />
          </div>
          <div class="min-w-0 flex-1">
            <h3 class="text-lg font-semibold leading-tight text-foreground">
              {{ t('ui.userPlans') }}
            </h3>
            <p class="text-xs text-muted-foreground">
              {{ userName || '-' }} · {{ t('ui.viewCurrentPlansAndGrantManually') }}
            </p>
          </div>
        </div>
      </div>
    </template>

    <div class="max-h-[64vh] space-y-4 overflow-y-auto">
      <div class="rounded-lg border border-amber-500/20 bg-amber-500/10 px-3 py-2.5 text-xs text-amber-100/90">
        {{ t('ui.grantedPlansTakeEffectImmediatelyExistingPlansAreReplacedEntirely') }}
      </div>

      <section class="space-y-2.5">
        <div class="flex items-center justify-between gap-3">
          <h4 class="text-sm font-semibold text-foreground">
            {{ t('ui.currentActivePlans') }}
          </h4>
          <Button
            variant="ghost"
            size="sm"
            class="h-7 px-2 text-[11px]"
            :disabled="loadingEntitlements || !userId"
            @click="userId && $emit('refresh-entitlements', userId)"
          >
            {{ loadingEntitlements ? t('common.loading') : t('ui.refresh') }}
          </Button>
        </div>

        <UserPlanList
          :entitlements="entitlements"
          :loading="loadingEntitlements"
          :format-date-time="formatDateTime"
          :entitlement-labels="entitlementLabels"
          :empty-text="t('ui.noActivePlans705f0e1')"
        >
          <template #actions="{ item }">
            <Button
              v-if="item.active"
              variant="destructive"
              size="sm"
              class="h-7 px-2 text-[11px]"
              :disabled="revokingEntitlementId === item.id"
              @click="$emit('revoke', item)"
            >
              {{ revokingEntitlementId === item.id ? t('ui.revoking') : t('ui.revokePlan') }}
            </Button>
          </template>
        </UserPlanList>
      </section>

      <section class="space-y-3 rounded-lg border border-border bg-card/70 p-4">
        <div class="space-y-1">
          <h4 class="text-sm font-semibold text-foreground">
            {{ t('ui.grantPlan1428b3a') }}
          </h4>
          <p class="text-xs text-muted-foreground">
            {{ t('ui.grantBenefitsWithoutChargingTheUserExistingPlansWithConflicting') }}
          </p>
        </div>

        <Select
          :model-value="selectedPlanId"
          @update:model-value="$emit('update:selectedPlanId', $event)"
        >
          <SelectTrigger
            class="h-9 rounded-md bg-muted/50 px-3"
            :disabled="loadingPlans || plans.length === 0"
          >
            <SelectValue :placeholder="loadingPlans ? t('ui.loadingPlans5a289c6') : t('ui.selectAPlanToGrant')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem
              v-for="plan in plans"
              :key="plan.id"
              :value="plan.id"
            >
              <div class="flex min-w-0 items-center gap-2">
                <span class="truncate">{{ plan.title }}</span>
                <span class="shrink-0 text-xs text-muted-foreground">
                  {{ formatPlanPrice(plan) }} · {{ formatPlanDuration(plan) }}
                </span>
                <span
                  v-if="!plan.enabled"
                  class="shrink-0 text-[10px] text-amber-400"
                >
                  {{ t('ui.unlisted') }}
                </span>
              </div>
            </SelectItem>
          </SelectContent>
        </Select>

        <Textarea
          :model-value="grantReason"
          class="min-h-[60px] resize-y rounded-md bg-muted/50 text-sm"
          maxlength="512"
          :placeholder="t('ui.notesOptionalEGManualCompensationOrCampaignGift')"
          @update:model-value="$emit('update:grantReason', $event)"
        />

        <div class="flex justify-end">
          <Button
            size="sm"
            :disabled="granting || !userId || !selectedPlanId"
            @click="$emit('grant')"
          >
            {{ granting ? t('ui.granting') : t('ui.grantPlan1428b3a') }}
          </Button>
        </div>
      </section>
    </div>

    <template #footer>
      <Button
        variant="outline"
        class="h-10 px-5"
        @click="$emit('close')"
      >
        {{ t('ui.close') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { PackageCheck } from 'lucide-vue-next'
import UserPlanList from './UserPlanList.vue'
import {
  Button,
  Dialog,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Textarea,
} from '@/components/ui'
import { useI18n } from '@/i18n'
import type { AdminUserPlanEntitlement } from '@/api/users'
import type { BillingEntitlement, BillingPlan } from '@/api/billing'

defineProps<{
  open: boolean
  userId?: string | null
  userName?: string
  entitlements: AdminUserPlanEntitlement[]
  plans: BillingPlan[]
  selectedPlanId: string
  grantReason: string
  loadingEntitlements: boolean
  loadingPlans: boolean
  granting: boolean
  revokingEntitlementId?: string | null
  formatDateTime: (value?: string | null) => string
  formatPlanPrice: (plan: BillingPlan) => string
  formatPlanDuration: (plan: BillingPlan) => string
  entitlementLabels: (items: BillingEntitlement[] | undefined) => string[]
}>()

defineEmits<{
  close: []
  'update:selectedPlanId': [value: string]
  'update:grantReason': [value: string]
  'refresh-entitlements': [userId: string]
  grant: []
  revoke: [entitlement: AdminUserPlanEntitlement]
}>()

const { t } = useI18n()
</script>
