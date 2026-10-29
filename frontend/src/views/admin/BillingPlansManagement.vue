<template>
  <PageContainer padding="lg">
    <PageHeader
      :title="t('nav.billingManagement')"
      :description="t('ui.configureCreditUsageLimitsMembershipBenefitsAndBundledPlans')"
    >
      <template #actions>
        <Button
          size="sm"
          @click="openCreateDialog"
        >
          <Plus class="mr-2 h-4 w-4" /> {{ t('ui.createPlan') }}
        </Button>
      </template>
    </PageHeader>

    <div class="mt-6 space-y-6">
      <div
        v-if="loading"
        class="py-16"
      >
        <LoadingState :message="t('ui.loadingPlans')" />
      </div>

      <CardSection
        v-else
        :title="t('ui.plans')"
        :description="t('ui.enabledPlansAppearInTheUserPlanCatalog')"
      >
        <div
          v-if="plans.length === 0"
          class="py-12"
        >
          <EmptyState
            :title="t('ui.noPlans')"
            :description="t('ui.createAPlanToMakeItAvailableForUsersTo')"
          />
        </div>

        <div
          v-else
          class="overflow-x-auto"
        >
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead class="w-[24%]">
                  {{ t('ui.plan') }}
                </TableHead>
                <TableHead class="w-[10%] whitespace-nowrap">
                  {{ t('ui.price') }}
                </TableHead>
                <TableHead class="w-[18%] whitespace-nowrap">
                  {{ t('ui.benefitValidity') }}
                </TableHead>
                <TableHead class="w-[20%]">
                  {{ t('ui.benefits') }}
                </TableHead>
                <TableHead class="w-[6%] whitespace-nowrap text-center">
                  {{ t('common.sort') }}
                </TableHead>
                <TableHead class="w-[7%] whitespace-nowrap text-center">
                  {{ t('ui.status6320b4a') }}
                </TableHead>
                <TableHead class="w-[15%] whitespace-nowrap text-right">
                  {{ t('ui.actionsed31fbb') }}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for="plan in plans"
                :key="plan.id"
              >
                <TableCell>
                  <div class="text-sm font-medium">
                    {{ plan.title }}
                  </div>
                  <div
                    v-if="plan.description"
                    class="mt-0.5 max-w-[280px] truncate text-xs text-muted-foreground"
                  >
                    {{ plan.description }}
                  </div>
                </TableCell>
                <TableCell class="whitespace-nowrap text-sm tabular-nums">
                  {{ formatPlanPriceAmount(plan) }} {{ plan.price_currency }}
                </TableCell>
                <TableCell>
                  <div class="whitespace-nowrap text-sm">
                    {{ formatPlanPeriod(plan) }}
                  </div>
                  <div class="mt-0.5 text-xs text-muted-foreground">
                    {{ planDurationHint(plan) }} · {{ planPurchaseLimitHint(plan) }}
                  </div>
                </TableCell>
                <TableCell>
                  <div class="flex flex-wrap items-center gap-1">
                    <span
                      v-for="item in entitlementBadges(plan)"
                      :key="item"
                      class="inline-flex whitespace-nowrap rounded-md border border-border/60 bg-muted/40 px-2 py-0.5 text-xs"
                    >
                      {{ item }}
                    </span>
                  </div>
                </TableCell>
                <TableCell class="text-center text-sm tabular-nums text-muted-foreground">
                  {{ plan.sort_order }}
                </TableCell>
                <TableCell class="text-center">
                  <span
                    class="inline-flex items-center gap-1.5 text-xs"
                    :class="plan.enabled ? 'text-emerald-500' : 'text-muted-foreground'"
                  >
                    <span
                      class="h-1.5 w-1.5 rounded-full"
                      :class="plan.enabled ? 'bg-emerald-500' : 'bg-muted-foreground/40'"
                    />
                    {{ plan.enabled ? t('ui.enableddfb8022') : t('ui.disableda8c3698') }}
                  </span>
                </TableCell>
                <TableCell class="whitespace-nowrap text-right">
                  <div class="inline-flex items-center">
                    <Button
                      variant="ghost"
                      size="sm"
                      :disabled="deletingPlanId === plan.id"
                      @click="togglePlanStatus(plan)"
                    >
                      {{ plan.enabled ? t('ui.disable4e6fd0e') : t('ui.enablef4f0ead') }}
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      :disabled="deletingPlanId === plan.id"
                      @click="openEditDialog(plan)"
                    >
                      {{ t('ui.edit0518365') }}
                    </Button>
                    <DropdownMenu>
                      <DropdownMenuTrigger as-child>
                        <Button
                          variant="ghost"
                          size="sm"
                          class="h-9 w-9 p-0"
                          :disabled="deletingPlanId === plan.id"
                        >
                          <MoreHorizontal class="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem
                          class="text-destructive focus:text-destructive"
                          :disabled="deletingPlanId === plan.id"
                          @select="deletePlan(plan)"
                        >
                          <Trash2 class="mr-2 h-4 w-4" />
                          {{ deletingPlanId === plan.id ? t('ui.deleting') : t('ui.deletePlan') }}
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>
      </CardSection>
    </div>

    <BillingPlanFormDialog
      :open="dialogOpen"
      :plan="editingPlan"
      :user-groups="userGroups"
      @update:open="dialogOpen = $event"
      @saved="loadPlans"
    />
  </PageContainer>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { MoreHorizontal, Plus, Trash2 } from 'lucide-vue-next'
import {
  adminBillingPlansApi,
  type BillingPlan,
} from '@/api/billing'
import { usersApi, type UserGroup } from '@/api/users'
import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui'
import { EmptyState, LoadingState } from '@/components/common'
import { CardSection, PageContainer, PageHeader } from '@/components/layout'
import { useToast } from '@/composables/useToast'
import { useI18n } from '@/i18n'
import { parseApiError } from '@/utils/errorParser'
import { log } from '@/utils/logger'
import { formatBillingPlanDuration } from '@/utils/billingEntitlements'
import BillingPlanFormDialog from '@/features/billing/components/BillingPlanFormDialog.vue'
import { formatUsagePolicyBadge, resolvePlanModeFromEntitlements } from '@/features/billing/utils/billingPlanForm'

const { success, error: showError } = useToast()
const { t } = useI18n()

const loading = ref(true)
const deletingPlanId = ref<string | null>(null)
const dialogOpen = ref(false)
const plans = ref<BillingPlan[]>([])
const editingPlan = ref<BillingPlan | null>(null)
const userGroups = ref<UserGroup[]>([])

onMounted(() => {
  void Promise.all([loadPlans(), loadUserGroups()]).finally(() => {
    loading.value = false
  })
})

async function loadPlans() {
  try {
    const response = await adminBillingPlansApi.list()
    plans.value = [...response.items].sort((left, right) =>
      left.sort_order === right.sort_order
        ? left.price_amount - right.price_amount
        : left.sort_order - right.sort_order
    )
  } catch (err) {
    log.error('加载套餐失败:', err)
    showError(parseApiError(err, t('ui.failedToLoadPlans')))
  }
}

async function loadUserGroups() {
  try {
    const response = await usersApi.listUserGroups()
    userGroups.value = response.items
  } catch (err) {
    log.error('加载用户分组失败:', err)
    showError(parseApiError(err, t('ui.failedToLoadUserGroups')))
  }
}

function openCreateDialog() {
  editingPlan.value = null
  dialogOpen.value = true
}

function openEditDialog(plan: BillingPlan) {
  editingPlan.value = plan
  dialogOpen.value = true
}

async function togglePlanStatus(plan: BillingPlan) {
  try {
    await adminBillingPlansApi.setStatus(plan.id, !plan.enabled)
    success(plan.enabled ? t('ui.planDisabled') : t('ui.planEnabled'))
    await loadPlans()
  } catch (err) {
    log.error('更新套餐状态失败:', err)
    showError(parseApiError(err, '更新套餐状态失败'))
  }
}

async function deletePlan(plan: BillingPlan) {
  if (deletingPlanId.value) return
  const confirmed = window.confirm(
    t('ui.deletePlanValuePlansWithExistingOrdersOrEntitlementsCannot', { value: plan.title })
  )
  if (!confirmed) return

  deletingPlanId.value = plan.id
  try {
    await adminBillingPlansApi.delete(plan.id)
    success(t('ui.planDeleted'))
    await loadPlans()
  } catch (err) {
    log.error('删除套餐失败:', err)
    showError(parseApiError(err, '删除套餐失败'))
  } finally {
    deletingPlanId.value = null
  }
}

function formatPlanPriceAmount(plan: BillingPlan): string {
  return Number(plan.price_amount || 0).toFixed(2)
}

function formatPlanPeriod(plan: BillingPlan): string {
  return formatBillingPlanDuration(plan.duration_unit, plan.duration_value)
}

function planPurchaseLimitHint(plan: BillingPlan): string {
  if (plan.purchase_limit_scope === 'unlimited') return t('ui.noPurchaseLimit')
  if (plan.purchase_limit_scope === 'lifetime') {
    return `每人最多购买 ${plan.max_active_per_user} 次`
  }
  return `最多同时生效 ${plan.max_active_per_user} 份`
}

function planDurationHint(plan: BillingPlan): string {
  if (plan.purchase_limit_scope === 'unlimited') return t('ui.noRepeatPurchaseLimit')
  if (plan.purchase_limit_scope === 'lifetime') return t('ui.purchaseCountsDoNotResetPeriodically')
  const mode = resolvePlanModeFromEntitlements(plan.entitlements)
  if (mode === 'wallet') return t('ui.legacyWalletCreditPlanDisablingIsRecommended')
  if (mode === 'daily') return t('ui.dailyCreditPeriod')
  if (mode === 'membership') return t('ui.membershipPeriod')
  if (mode === 'usage') return t('ui.usageLimitPeriod')
  if (mode === 'mixed') return t('ui.bundledBenefitPeriod')
  return t('ui.noBenefitsConfigured')
}

function groupName(groupId: string): string {
  return userGroups.value.find((group) => group.id === groupId)?.name || groupId
}

function entitlementBadges(plan: BillingPlan): string[] {
  return (plan.entitlements || []).map((entitlement) => {
    if (entitlement.type === 'wallet_credit') {
      return `附赠余额 $${Number(entitlement.amount_usd || 0).toFixed(2)}`
    }
    if (entitlement.type === 'daily_quota') {
      return `每日 $${Number(entitlement.daily_quota_usd || 0).toFixed(2)}`
    }
    if (entitlement.type === 'membership_group') {
      const groups = entitlement.grant_user_groups.map(groupName).join(', ')
      return `会员组 ${groups}`
    }
    if (entitlement.type === 'usage_policy') {
      return formatUsagePolicyBadge(entitlement)
    }
    return t('ui.unknownBenefit')
  })
}

</script>
