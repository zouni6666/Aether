<template>
  <PageContainer>
    <PageHeader
      :title="t('nav.billingCenter')"
      :description="t('ui.buyCreditMembershipOrUsagePlans')"
    />

    <div class="mt-6 space-y-6">
      <div
        v-if="loading"
        class="py-16"
      >
        <LoadingState :message="t('ui.loadingPlans')" />
      </div>

      <template v-else>
        <CardSection
          :title="t('ui.currentBenefits')"
          :description="t('ui.onlyActivePlanBenefitsAreShown')"
        >
          <div
            v-if="activeEntitlements.length"
            class="grid grid-cols-1 gap-3 lg:grid-cols-2"
          >
            <div
              v-for="item in activeEntitlements"
              :key="item.id"
              class="rounded-lg border border-border/60 bg-muted/20 p-4"
            >
              <div class="flex items-start justify-between gap-3">
                <div>
                  <div class="font-medium">
                    {{ planTitle(item.plan_id) }}
                  </div>
                  <div class="mt-1 text-xs text-muted-foreground">
                    {{ formatDateShort(item.starts_at) }} - {{ formatDateShort(item.expires_at) }}
                  </div>
                </div>
                <Badge variant="success">
                  {{ t('ui.active') }}
                </Badge>
              </div>
              <div class="mt-3 flex flex-wrap gap-1.5">
                <Badge
                  v-for="(label, index) in entitlementLabels(item.entitlements)"
                  :key="`${label}-${index}`"
                  variant="outline"
                >
                  {{ label }}
                </Badge>
              </div>
            </div>
          </div>
          <EmptyState
            v-else
            :title="t('ui.noActivePlans')"
            :description="t('ui.activeBenefitsWillAppearHereAfterYouPurchaseAPlan')"
          />
        </CardSection>

        <CardSection
          :title="t('ui.availablePlans')"
          :description="t('ui.benefitsAreIssuedAutomaticallyByCallbackAfterSuccessfulPayment')"
        >
          <div class="grid grid-cols-1 gap-4 xl:grid-cols-3">
            <Card
              v-for="plan in purchaseablePlans"
              :key="plan.id"
              class="flex flex-col p-5"
            >
              <div class="flex items-start justify-between gap-3">
                <div>
                  <h3 class="text-base font-semibold">
                    {{ plan.title }}
                  </h3>
                  <p class="mt-1 min-h-[32px] text-xs text-muted-foreground">
                    {{ plan.description || t('ui.standardPlan') }}
                  </p>
                </div>
                <Badge variant="outline">
                  {{ formatBillingPlanDuration(plan.duration_unit, plan.duration_value) }}
                </Badge>
              </div>

              <div class="mt-5">
                <span class="text-3xl font-semibold tabular-nums">
                  {{ Number(plan.price_amount || 0).toFixed(2) }}
                </span>
                <span class="ml-1 text-sm text-muted-foreground">
                  {{ plan.price_currency }}
                </span>
              </div>

              <div class="mt-5 flex flex-wrap gap-1.5">
                <Badge
                  v-for="(label, index) in entitlementLabels(plan.entitlements)"
                  :key="`${label}-${index}`"
                  variant="outline"
                >
                  {{ label }}
                </Badge>
              </div>

              <div
                v-if="replacementNotice(plan)"
                class="mt-4 rounded-lg border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-xs leading-5 text-amber-200"
              >
                {{ replacementNotice(plan) }}
              </div>

              <div class="mt-5 flex-1" />

              <div class="mt-5 space-y-3">
                <Select v-model="selectedChannel">
                  <SelectTrigger>
                    <SelectValue
                      :placeholder="checkoutOptions.length ? t('ui.selectPaymentChannel') : t('ui.noPaymentChannelsAvailable')"
                    />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem
                      v-for="option in checkoutOptions"
                      :key="option.key"
                      :value="option.key"
                    >
                      {{ option.display_name }}
                    </SelectItem>
                  </SelectContent>
                </Select>
                <Button
                  class="w-full"
                  :disabled="
                    checkoutLoadingPlanId === plan.id
                      || checkoutOptions.length === 0
                      || !selectedChannel
                  "
                  @click="checkoutPlan(plan)"
                >
                  <CreditCard class="mr-2 h-4 w-4" />
                  {{ checkoutLoadingPlanId === plan.id ? t('ui.creatingOrder') : t('ui.buyPlan') }}
                </Button>
              </div>
            </Card>

            <div
              v-if="purchaseablePlans.length === 0"
              class="xl:col-span-3"
            >
              <EmptyState
                :title="t('ui.noPlansAvailableForPurchase')"
                :description="t('ui.plansWillAppearHereAfterAnAdministratorPublishesThem')"
              />
            </div>
          </div>
        </CardSection>

        <Card
          v-if="latestCheckout"
          class="p-4"
        >
          <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
            <div>
              <div class="text-sm font-medium">
                {{ t('ui.latestOrder') }}<span class="font-mono">{{ latestCheckout.order.order_no }}</span>
              </div>
              <div class="mt-1 text-xs text-muted-foreground">
                {{ t('ui.amountDue') }} {{ latestCheckout.order.pay_amount ?? '-' }} {{ latestCheckout.order.pay_currency || '' }}
              </div>
            </div>
            <Button
              v-if="latestCheckoutActionLabel"
              variant="outline"
              @click="openLatestPayment"
            >
              {{ latestCheckoutActionLabel }}
            </Button>
          </div>
        </Card>
      </template>
    </div>

    <StripePaymentDialog
      v-model:open="stripeDialogOpen"
      :instructions="stripePaymentInstructions"
      :title="t('ui.planStripePayment')"
      :description="t('ui.afterPaymentPlanBenefitsAreIssuedAutomaticallyByTheStripe')"
      :confirm-text="t('ui.payForPlan')"
      @success="handleStripePaymentSuccess"
    />
  </PageContainer>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { CreditCard } from 'lucide-vue-next'
import {
  billingApi,
  type BillingEntitlement,
  type BillingCheckoutResponse,
  type BillingPlan,
  type UserPlanEntitlement,
} from '@/api/billing'
import { walletApi, type WalletRechargeOption } from '@/api/wallet'
import {
  Badge,
  Button,
  Card,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui'
import { EmptyState, LoadingState } from '@/components/common'
import StripePaymentDialog from '@/features/wallet/components/StripePaymentDialog.vue'
import { CardSection, PageContainer, PageHeader } from '@/components/layout'
import { useToast } from '@/composables/useToast'
import { useI18n } from '@/i18n'
import { parseApiError } from '@/utils/errorParser'
import {
  entitlementReplacementGroups,
  entitlementsWillReplaceExisting,
  isPlanEntitlementReplacementCandidate,
  usagePolicyEntitlementLabels,
  formatBillingPlanDuration,
} from '@/utils/billingEntitlements'
import { log } from '@/utils/logger'
import { safePaymentTargetUrl } from '@/features/wallet/utils/paymentUrl'
import {
  getPaymentInstructionString,
  getStripePaymentInstructions,
  type PaymentInstructionMap,
} from '@/features/wallet/utils/paymentInstructions'
import { formatDateShort } from '@/utils/format'

const { success, error: showError } = useToast()
const { t } = useI18n()

const loading = ref(true)
const plans = ref<BillingPlan[]>([])
const entitlements = ref<UserPlanEntitlement[]>([])
const rechargeOptions = ref<WalletRechargeOption[]>([])
const selectedChannel = ref('')
const checkoutLoadingPlanId = ref<string | null>(null)
const latestCheckout = ref<BillingCheckoutResponse | null>(null)
const stripeDialogOpen = ref(false)
const stripePaymentInstructions = ref<PaymentInstructionMap | null>(null)

const checkoutOptions = computed(() =>
  rechargeOptions.value
    .filter((option) => Boolean(paymentOptionProvider(option)) && Boolean(option.payment_channel))
    .map((option, index) => ({
      ...option,
      key: [
        paymentOptionProvider(option),
        option.payment_method,
        option.payment_channel || '',
        index,
      ].join(':'),
    }))
)

const selectedCheckoutOption = computed(() => {
  if (checkoutOptions.value.length === 0) return null
  return checkoutOptions.value.find(option => option.key === selectedChannel.value)
    || checkoutOptions.value[0]
})

const replacementCandidateEntitlements = computed(() =>
  entitlements.value.filter((item) =>
    isPlanEntitlementReplacementCandidate(item)
  )
)

const activeEntitlements = computed(() =>
  entitlements.value.filter((item) =>
    item.active !== false
    && item.status === 'active'
    && hasPackageEntitlement(item.entitlements)
  )
)

const purchaseablePlans = computed(() =>
  plans.value.filter((plan) => hasPackageEntitlement(plan.entitlements))
)

const latestPaymentUrl = computed(() => {
  const value = latestCheckout.value?.payment_instructions?.payment_url
  return typeof value === 'string' && value ? value : ''
})

const latestCheckoutActionLabel = computed(() => {
  const instructions = latestCheckout.value?.payment_instructions
  if (getStripePaymentInstructions(instructions)) return t('ui.openStripeCheckout')
  if (latestPaymentUrl.value) return t('ui.openPaymentLink')
  return ''
})

watch(checkoutOptions, (options) => {
  const keys = options.map(option => option.key)
  if (!keys.includes(selectedChannel.value)) {
    selectedChannel.value = keys[0] || ''
  }
}, { immediate: true })

onMounted(async () => {
  await Promise.all([
    loadPlans(),
    loadEntitlements(),
    loadRechargeOptions(),
  ])
  loading.value = false
})

async function loadPlans() {
  try {
    const response = await billingApi.listPlans()
    plans.value = response.items
  } catch (err) {
    log.error('加载套餐失败:', err)
    showError(parseApiError(err, t('ui.failedToLoadPlans')))
  }
}

async function loadEntitlements() {
  try {
    const response = await billingApi.listEntitlements()
    entitlements.value = response.items
  } catch (err) {
    log.error('加载套餐权益失败:', err)
    showError(parseApiError(err, t('ui.failedToLoadPlanBenefits')))
  }
}

async function loadRechargeOptions() {
  try {
    const response = await walletApi.listRechargeOptions()
    rechargeOptions.value = response.items
    if (!selectedChannel.value && checkoutOptions.value.length > 0) {
      selectedChannel.value = checkoutOptions.value[0].key
    }
  } catch (err) {
    log.error('加载支付通道失败:', err)
    showError(parseApiError(err, t('ui.failedToLoadPaymentChannels')))
  }
}

async function checkoutPlan(plan: BillingPlan) {
  if (hasMatchingActivePlan(plan)) {
    const confirmed = window.confirm(t('ui.thisPurchaseWillReplaceConflictingPlansAndAllTheirBundled'))
    if (!confirmed) return
  }
  const option = selectedCheckoutOption.value
  if (!option) {
    showError(t('ui.selectAPaymentMethod'))
    return
  }
  const provider = paymentOptionProvider(option)
  if (!provider) {
    showError(t('ui.invalidPaymentMethodConfiguration'))
    return
  }
  checkoutLoadingPlanId.value = plan.id
  try {
    const response = await billingApi.checkout(plan.id, {
      payment_method: option.payment_method || provider,
      payment_provider: provider,
      payment_channel: option.payment_channel,
    })
    latestCheckout.value = response
    success(
      response.reused_pending_order
        ? t('ui.aPendingPaymentOrderAlreadyExistsOpenedTheExistingPayment')
        : t('ui.planOrderCreated')
    )
    submitPaymentInstructions(response.payment_instructions)
  } catch (err) {
    log.error('创建套餐订单失败:', err)
    showError(parseApiError(err, t('ui.failedToCreatePlanOrder')))
  } finally {
    checkoutLoadingPlanId.value = null
  }
}

function openLatestPayment() {
  submitPaymentInstructions(latestCheckout.value?.payment_instructions || { payment_url: latestPaymentUrl.value })
}

function submitPaymentInstructions(instructions: Record<string, unknown> | null | undefined) {
  if (!instructions) return
  const stripeInstructions = getStripePaymentInstructions(instructions)
  if (stripeInstructions) {
    stripePaymentInstructions.value = instructions
    stripeDialogOpen.value = true
    return
  }
  const paymentUrl = getPaymentInstructionString(instructions, 'payment_url')
  if (!paymentUrl) return
  const safePaymentUrl = safePaymentTargetUrl(paymentUrl)
  if (!safePaymentUrl) {
    showError(t('ui.thePaymentGatewayReturnedAnUnsafePaymentUrl'))
    return
  }
  const paymentParams = instructions.payment_params
  if (paymentParams && typeof paymentParams === 'object' && !Array.isArray(paymentParams)) {
    submitPaymentForm(safePaymentUrl, paymentParams as Record<string, unknown>)
    return
  }
  const opened = window.open(safePaymentUrl, '_blank', 'noopener,noreferrer')
  if (!opened) {
    window.location.href = safePaymentUrl
  }
}

function submitPaymentForm(url: string, params: Record<string, unknown>) {
  const safeUrl = safePaymentTargetUrl(url)
  if (!safeUrl) {
    showError(t('ui.thePaymentGatewayReturnedAnUnsafePaymentUrl'))
    return
  }
  const form = document.createElement('form')
  form.action = safeUrl
  form.method = 'POST'
  if (!isSafariBrowser()) {
    form.target = '_blank'
  }
  Object.entries(params).forEach(([key, value]) => {
    if (value === null || value === undefined) return
    const input = document.createElement('input')
    input.type = 'hidden'
    input.name = key
    input.value = String(value)
    form.appendChild(input)
  })
  document.body.appendChild(form)
  form.submit()
  document.body.removeChild(form)
}

function isSafariBrowser(): boolean {
  return navigator.userAgent.includes('Safari') && !navigator.userAgent.includes('Chrome')
}

async function handleStripePaymentSuccess() {
  success(t('ui.paymentCompleteRefreshingPlanStatus'))
  await Promise.all([loadEntitlements(), loadPlans()])
}

function paymentOptionProvider(option: WalletRechargeOption): string {
  return (option.payment_provider || option.provider || option.payment_method || '').trim()
}

function planTitle(planId: string): string {
  return plans.value.find((plan) => plan.id === planId)?.title || planId
}

function hasMatchingActivePlan(plan: BillingPlan): boolean {
  return replacementCandidateEntitlements.value.some((item) =>
    entitlementsWillReplaceExisting(plan.entitlements, item.entitlements)
  )
}

function replacementNotice(plan: BillingPlan): string {
  const labels = replacementClassLabels(plan.entitlements)
  if (labels.length === 0) return ''
  if (hasMatchingActivePlan(plan)) {
    return t('ui.youHaveAnActivePlanThatConflictsWithThisOne')
  }
  return `若已有冲突的有效${labels.join('和')}，购买成功后旧套餐会整体失效。`
}

function entitlementLabels(items: BillingEntitlement[]): string[] {
  return (items || []).flatMap((item) => {
    if (item.type === 'wallet_credit') {
      return `附赠余额 $${Number(item.amount_usd || 0).toFixed(2)}`
    }
    if (item.type === 'daily_quota') {
      return `每日 $${Number(item.daily_quota_usd || 0).toFixed(2)}`
    }
    if (item.type === 'membership_group') {
      return `会员组 ${item.grant_user_groups.join(', ')}`
    }
    if (item.type === 'usage_policy') {
      return usagePolicyEntitlementLabels(item)
    }
    return []
  })
}

function hasPackageEntitlement(items: BillingEntitlement[] | undefined): boolean {
  return (items || []).some((item) =>
    item.type === 'daily_quota'
    || item.type === 'membership_group'
    || item.type === 'usage_policy'
  )
}

function hasDailyQuotaEntitlement(items: BillingEntitlement[] | undefined): boolean {
  return (items || []).some((item) => item.type === 'daily_quota')
}

function hasMembershipEntitlement(items: BillingEntitlement[] | undefined): boolean {
  return (items || []).some((item) => item.type === 'membership_group')
}

function replacementClassLabels(items: BillingEntitlement[] | undefined): string[] {
  const labels: string[] = []
  if (hasDailyQuotaEntitlement(items)) labels.push('每日额度套餐')
  if (hasMembershipEntitlement(items)) labels.push('会员权益包')
  labels.push(...entitlementReplacementGroups(items).map(group => `互斥组「${group}」`))
  return labels
}

</script>
