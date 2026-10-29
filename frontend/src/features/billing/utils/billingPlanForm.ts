import { translate } from '@/i18n'
import type {
  BillingDurationUnit,
  BillingEntitlement,
  BillingPlan,
  BillingPurchaseLimitScope,
  DailyQuotaEntitlement,
  MembershipGroupEntitlement,
  UsagePolicyEntitlement,
  UsagePolicyMetric,
  UsagePolicyRule,
  UsagePolicyWindow,
  WalletCreditBucket,
  WalletCreditEntitlement,
} from '@/api/billing'

// 套餐表单的类型、默认值、模板与表单和权益之间的转换

export type TemplateKey = 'daily' | 'membership' | 'mixed' | 'traffic'

export type PlanMode = 'empty' | 'wallet' | 'daily' | 'membership' | 'usage' | 'mixed'

export type UsagePolicyWindowKind = UsagePolicyWindow['kind']

export type UsagePolicyRulePreset =
  | 'qps'
  | 'rpm'
  | 'rolling_5h'
  | 'cost_5h'
  | 'cost_daily'
  | 'cost_weekly'
  | 'cost_monthly'
  | 'cost_subscription'
  | 'daily'
  | 'weekly'
  | 'monthly'
  | 'subscription'
  | 'concurrency'

export interface UsagePolicyRuleForm {
  local_id: string
  metric: UsagePolicyMetric
  window_kind: UsagePolicyWindowKind
  rolling_seconds: number
  timezone: string
  week_start: number
  limit: number
}

export interface UsagePolicyForm {
  local_id: string
  policy_id: string
  name: string
  replacement_group: string
  rules: UsagePolicyRuleForm[]
}

export interface PlanModeGuide {
  badge: string
  title: string
  description: string
  notes: string[]
}

export interface PlanFormState {
  title: string
  description: string
  price_amount: number
  price_currency: string
  duration_unit: BillingDurationUnit
  duration_value: number
  enabled: boolean
  sort_order: number
  max_active_per_user: number
  purchase_limit_scope: BillingPurchaseLimitScope
  wallet_credit_enabled: boolean
  wallet_credit_amount_usd: number
  wallet_credit_balance_bucket: WalletCreditBucket
  wallet_credit_replacement_group: string
  daily_quota_enabled: boolean
  daily_quota_usd: number
  reset_timezone: string
  carry_over: boolean
  daily_quota_replacement_group: string
  membership_group_enabled: boolean
  grant_user_groups: string[]
  membership_group_replacement_group: string
  usage_policy_enabled: boolean
  usage_policies: UsagePolicyForm[]
}

export const MAX_USAGE_POLICY_ENTITLEMENTS = 16

export const MAX_USAGE_POLICY_RULES_PER_POLICY = 32

export const MAX_USAGE_POLICY_TOTAL_RULES = 64

export const MAX_USAGE_POLICY_EXACT_INTEGER = Number.MAX_SAFE_INTEGER

export const MIN_USAGE_POLICY_COST_USD = 0.00000001

export const MAX_USAGE_POLICY_COST_USD = Number.MAX_SAFE_INTEGER / 100_000_000

let localIdSequence = 0

export function getPlanModeGuide(mode: PlanMode): PlanModeGuide {
  switch (mode) {
    case 'wallet':
      return {
        badge: '旧余额权益',
        get title() { return translate('ui.legacyWalletCreditPlan') },
        get description() { return translate('ui.walletTopUpsNowReplaceCreditOnlyPlansDisableThis') },
        notes: [
          '新建套餐不再提供余额包模板',
          '钱包入账请使用充值功能',
          '附赠余额仍可放在混合套餐中',
        ],
      }
    case 'daily':
      return {
        badge: '周期额度',
        get title() { return translate('ui.dailyQuotaPlan') },
        get description() { return translate('ui.suitableForMonthlyQuarterlyOrAnnualPlansGrantsASeparate') },
        notes: [
          '默认每天按重置时区刷新',
          '默认用完后拒绝继续消费',
          '同一用户只保留一个有效每日额度套餐',
        ],
      }
    case 'membership':
      return {
        badge: '会员权限',
        get title() { return translate('ui.membershipBenefits') },
        get description() { return translate('ui.suitableForProPlusOrTeamMembershipsAddsGroupPermissions') },
        notes: [
          '不会永久修改用户基础分组',
          '同一用户只保留一个有效会员权益包',
          '适合解锁模型组或高级功能',
        ],
      }
    case 'usage':
      return {
        badge: '流量策略',
        get title() { return translate('ui.usageLimitPlan140e55e') },
        get description() { return translate('ui.limitRequestFrequencyRequestsPerPeriodAndConcurrentRequestsCombine') },
        notes: [
          '任意规则触顶都会拒绝新请求',
          'QPS 和 RPM 使用滚动窗口',
          '策略随套餐权益到期失效',
        ],
      }
    case 'mixed':
      return {
        badge: '混合套餐',
        get title() { return translate('ui.bundledBenefitPlan') },
        get description() { return translate('ui.combinesCreditMembershipPermissionsAndUsageLimitsWithOptionalBonus') },
        notes: [
          '包含每日额度或会员权益时，同类旧套餐会整包失效',
          '附赠余额发放后不随周期结束扣回',
          '限购会同时影响整套组合权益',
        ],
      }
    default:
      return {
        badge: '待配置',
        get title() { return translate('ui.selectAPlanTemplate') },
        get description() { return translate('ui.selectDailyCreditMembershipGroupsUsageLimitsOrABundled') },
        notes: [
          '每日额度和会员权益是周期权益',
          '钱包充值已从套餐中拆出',
          '混合套餐可以附赠余额',
        ],
      }
  }
}

export const weekStartOptions = [
  { value: '1', get label() { return translate('ui.mon') } },
  { value: '2', get label() { return translate('ui.tue') } },
  { value: '3', get label() { return translate('ui.wed') } },
  { value: '4', get label() { return translate('ui.thu') } },
  { value: '5', get label() { return translate('ui.fri') } },
  { value: '6', get label() { return translate('ui.sat') } },
  { value: '7', get label() { return translate('ui.sun') } },
]

export function buildDefaultForm(): PlanFormState {
  return {
    title: '',
    description: '',
    price_amount: 100,
    price_currency: 'CNY',
    duration_unit: 'month',
    duration_value: 1,
    enabled: true,
    sort_order: 0,
    max_active_per_user: 1,
    purchase_limit_scope: 'active_period',
    wallet_credit_enabled: false,
    wallet_credit_amount_usd: 10,
    wallet_credit_balance_bucket: 'recharge',
    wallet_credit_replacement_group: '',
    daily_quota_enabled: false,
    daily_quota_usd: 50,
    reset_timezone: 'Asia/Shanghai',
    carry_over: false,
    daily_quota_replacement_group: '',
    membership_group_enabled: false,
    grant_user_groups: [],
    membership_group_replacement_group: '',
    usage_policy_enabled: false,
    usage_policies: [buildDefaultUsagePolicy()],
  }
}

export function nextLocalId(prefix: string): string {
  localIdSequence += 1
  return `${prefix}-${localIdSequence}`
}

export function buildDefaultUsageRule(preset: UsagePolicyRulePreset = 'rpm'): UsagePolicyRuleForm {
  const rule: UsagePolicyRuleForm = {
    local_id: nextLocalId('rule'),
    metric: 'request_count',
    window_kind: 'rolling',
    rolling_seconds: 60,
    timezone: 'Asia/Shanghai',
    week_start: 1,
    limit: 60,
  }
  switch (preset) {
    case 'qps':
      rule.rolling_seconds = 1
      rule.limit = 5
      break
    case 'rolling_5h':
      rule.rolling_seconds = 18_000
      rule.limit = 1_000
      break
    case 'cost_5h':
      rule.metric = 'actual_cost_usd'
      rule.rolling_seconds = 18_000
      rule.limit = 12.5
      break
    case 'cost_daily':
      rule.metric = 'actual_cost_usd'
      rule.window_kind = 'calendar_day'
      rule.limit = 25
      break
    case 'cost_weekly':
      rule.metric = 'actual_cost_usd'
      rule.window_kind = 'calendar_week'
      rule.limit = 100
      break
    case 'cost_monthly':
      rule.metric = 'actual_cost_usd'
      rule.window_kind = 'calendar_month'
      rule.limit = 300
      break
    case 'cost_subscription':
      rule.metric = 'actual_cost_usd'
      rule.window_kind = 'subscription_period'
      rule.limit = 300
      break
    case 'daily':
      rule.window_kind = 'calendar_day'
      rule.limit = 10_000
      break
    case 'weekly':
      rule.window_kind = 'calendar_week'
      rule.limit = 50_000
      break
    case 'monthly':
      rule.window_kind = 'calendar_month'
      rule.limit = 200_000
      break
    case 'subscription':
      rule.window_kind = 'subscription_period'
      rule.limit = 200_000
      break
    case 'concurrency':
      rule.metric = 'concurrency'
      rule.window_kind = 'concurrent'
      rule.limit = 5
      break
    case 'rpm':
      break
  }
  return rule
}

export function buildDefaultUsagePolicy(): UsagePolicyForm {
  return {
    local_id: nextLocalId('policy'),
    policy_id: '',
    name: '',
    replacement_group: '',
    rules: [buildDefaultUsageRule()],
  }
}

export function formFromPlan(plan: BillingPlan): PlanFormState {
  const next = buildDefaultForm()
  next.title = plan.title
  next.description = plan.description || ''
  next.price_amount = plan.price_amount
  next.price_currency = plan.price_currency.toUpperCase()
  next.duration_unit = plan.duration_unit
  next.duration_value = plan.duration_value
  next.enabled = plan.enabled
  next.sort_order = plan.sort_order
  next.max_active_per_user = plan.max_active_per_user
  next.purchase_limit_scope = plan.purchase_limit_scope || 'active_period'

  for (const entitlement of plan.entitlements || []) {
    if (entitlement.type === 'wallet_credit') {
      const wallet = entitlement as WalletCreditEntitlement
      next.wallet_credit_enabled = true
      next.wallet_credit_amount_usd = Number(wallet.amount_usd || next.wallet_credit_amount_usd)
      next.wallet_credit_balance_bucket = wallet.balance_bucket || 'recharge'
      next.wallet_credit_replacement_group = wallet.replacement_group || ''
    } else if (entitlement.type === 'daily_quota') {
      const quota = entitlement as DailyQuotaEntitlement
      next.daily_quota_enabled = true
      next.daily_quota_usd = Number(quota.daily_quota_usd || next.daily_quota_usd)
      next.reset_timezone = quota.reset_timezone || 'Asia/Shanghai'
      next.carry_over = Boolean(quota.carry_over)
      next.daily_quota_replacement_group = quota.replacement_group || ''
    } else if (entitlement.type === 'membership_group') {
      const membership = entitlement as MembershipGroupEntitlement
      next.membership_group_enabled = true
      next.grant_user_groups = Array.isArray(membership.grant_user_groups)
        ? [...membership.grant_user_groups]
        : []
      next.membership_group_replacement_group = membership.replacement_group || ''
    } else if (entitlement.type === 'usage_policy') {
      const policy = entitlement as UsagePolicyEntitlement
      next.usage_policy_enabled = true
      next.usage_policies.push(usagePolicyFormFromEntitlement(policy))
    }
  }
  if (next.usage_policy_enabled) {
    next.usage_policies = next.usage_policies.slice(1)
  }
  return next
}

export function buildTemplateForm(template: TemplateKey): PlanFormState {
  const next = buildDefaultForm()
  if (template === 'daily') {
    next.title = '100 RMB 月卡'
    next.description = '每日 50 USD 独立额度，周期 1 个月'
    next.daily_quota_enabled = true
    next.daily_quota_usd = 50
    next.max_active_per_user = 1
  } else if (template === 'membership') {
    next.title = 'Pro 月卡'
    next.description = '动态授予 Pro 用户分组 1 个月'
    next.membership_group_enabled = true
    next.max_active_per_user = 1
  } else if (template === 'mixed') {
    next.title = 'Pro 混合月卡'
    next.description = '每日额度、会员分组和流量限制组合'
    next.daily_quota_enabled = true
    next.daily_quota_usd = 50
    next.membership_group_enabled = true
    next.usage_policy_enabled = true
    next.usage_policies = [buildDefaultUsagePolicy()]
    next.max_active_per_user = 1
  } else {
    next.title = '标准流量月卡'
    next.description = '每秒 5 次、每分钟 60 次，最多 5 个并发请求'
    next.usage_policy_enabled = true
    next.usage_policies = [
      {
        local_id: nextLocalId('policy'),
        policy_id: 'standard-traffic',
        name: '标准流量限制',
        replacement_group: '',
        rules: [
          buildDefaultUsageRule('qps'),
          buildDefaultUsageRule('rpm'),
          buildDefaultUsageRule('concurrency'),
        ],
      },
    ]
  }
  return next
}

export function attachReplacementGroup(
  entitlement: BillingEntitlement,
  replacementGroup: string,
): void {
  const normalized = replacementGroup.trim()
  if (normalized) entitlement.replacement_group = normalized
}

export function usagePolicyFormFromEntitlement(policy: UsagePolicyEntitlement): UsagePolicyForm {
  return {
    local_id: nextLocalId('policy'),
    policy_id: policy.policy_id || '',
    name: policy.name || '',
    replacement_group: policy.replacement_group || '',
    rules: (policy.rules || []).map(usagePolicyRuleFormFromRule),
  }
}

export function usagePolicyRuleFormFromRule(rule: UsagePolicyRule): UsagePolicyRuleForm {
  return {
    local_id: nextLocalId('rule'),
    metric: rule.metric,
    window_kind: rule.window.kind,
    rolling_seconds: rule.window.kind === 'rolling' ? Number(rule.window.seconds) : 60,
    timezone: 'timezone' in rule.window ? rule.window.timezone || '' : '',
    week_start: rule.window.kind === 'calendar_week' ? Number(rule.window.week_start || 1) : 1,
    limit: Number(rule.limit),
  }
}

export function buildUsagePolicyEntitlement(policy: UsagePolicyForm): UsagePolicyEntitlement {
  const entitlement: UsagePolicyEntitlement = {
    type: 'usage_policy',
    rules: policy.rules.map(buildUsagePolicyRule),
  }
  const policyId = policy.policy_id.trim()
  const name = policy.name.trim()
  const replacementGroup = policy.replacement_group.trim()
  if (policyId) entitlement.policy_id = policyId
  if (name) entitlement.name = name
  if (replacementGroup) entitlement.replacement_group = replacementGroup
  return entitlement
}

export function buildUsagePolicyRule(rule: UsagePolicyRuleForm): UsagePolicyRule {
  let window: UsagePolicyWindow
  switch (rule.window_kind) {
    case 'rolling':
      window = { kind: 'rolling', seconds: Number(rule.rolling_seconds) }
      break
    case 'calendar_day':
    case 'calendar_month':
      window = rule.timezone.trim()
        ? { kind: rule.window_kind, timezone: rule.timezone.trim() }
        : { kind: rule.window_kind }
      break
    case 'calendar_week':
      window = rule.timezone.trim()
        ? {
            kind: 'calendar_week',
            timezone: rule.timezone.trim(),
            week_start: Number(rule.week_start || 1),
          }
        : {
            kind: 'calendar_week',
            week_start: Number(rule.week_start || 1),
          }
      break
    case 'subscription_period':
      window = { kind: 'subscription_period' }
      break
    case 'concurrent':
      window = { kind: 'concurrent' }
      break
  }
  return {
    metric: rule.metric,
    window,
    limit: Number(rule.limit),
    enforcement: 'hard_cap',
  }
}

export function isCalendarWindow(kind: UsagePolicyWindowKind): boolean {
  return kind === 'calendar_day' || kind === 'calendar_week' || kind === 'calendar_month'
}

export function usageWindowParameterLabel(rule: UsagePolicyRuleForm): string {
  if (rule.window_kind === 'rolling') return translate('ui.windowDurationInSeconds')
  if (rule.window_kind === 'calendar_week') return translate('ui.timeZoneStartOfWeek')
  if (isCalendarWindow(rule.window_kind)) return translate('ui.timeZoneOptional')
  return translate('ui.windowParameters')
}

export function isValidIanaTimezone(value: string): boolean {
  const timezone = value.trim()
  if (!timezone) return true
  try {
    new Intl.DateTimeFormat('en-US', { timeZone: timezone }).format()
    return true
  } catch {
    return false
  }
}

export function resolvePlanModeFromEntitlements(entitlements: BillingEntitlement[] | undefined): PlanMode {
  const items = entitlements || []
  const hasWallet = items.some((entitlement) => entitlement.type === 'wallet_credit')
  const hasDaily = items.some((entitlement) => entitlement.type === 'daily_quota')
  const hasMembership = items.some((entitlement) => entitlement.type === 'membership_group')
  const hasUsage = items.some((entitlement) => entitlement.type === 'usage_policy')
  const enabledCount = [hasWallet, hasDaily, hasMembership, hasUsage].filter(Boolean).length

  if (enabledCount === 0) return 'empty'
  if (enabledCount > 1) return 'mixed'
  if (hasWallet) return 'wallet'
  if (hasDaily) return 'daily'
  if (hasMembership) return 'membership'
  return 'usage'
}

export function formatUsagePolicyBadge(policy: UsagePolicyEntitlement): string {
  const rules = policy.rules || []
  if (rules.length === 0) return translate('ui.usageLimits861d113')
  const first = rules[0]
  const suffix = rules.length > 1 ? ` +${rules.length - 1}` : ''
  if (first.metric === 'concurrency') return `并发 ${first.limit}${suffix}`
  if (first.metric === 'request_count' && first.window.kind === 'rolling' && first.window.seconds === 1) {
    return `QPS ${first.limit}${suffix}`
  }
  if (first.metric === 'request_count' && first.window.kind === 'rolling' && first.window.seconds === 60) {
    return `RPM ${first.limit}${suffix}`
  }
  const windowLabels: Record<UsagePolicyWindowKind, string> = {
    rolling: `${first.window.kind === 'rolling' ? first.window.seconds : 0} 秒`,
    calendar_day: '每日',
    calendar_week: '每周',
    get calendar_month() { return translate('ui.everyMonthOnDay') },
    get subscription_period() { return translate('ui.planPeriod') },
    get concurrent() { return translate('ui.concurrent2b66a2b') },
  }
  const limit = first.metric === 'actual_cost_usd' ? `$${first.limit}` : first.limit
  return `${windowLabels[first.window.kind]} ${limit}${suffix}`
}

export function hasPackageEntitlement(entitlements: BillingEntitlement[] | undefined): boolean {
  return (entitlements || []).some((entitlement) =>
    entitlement.type === 'daily_quota'
    || entitlement.type === 'membership_group'
    || entitlement.type === 'usage_policy'
  )
}
