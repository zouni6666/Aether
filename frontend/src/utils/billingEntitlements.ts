import { getI18nLocale, translate } from '@/i18n'

import type {
  BillingDurationUnit,
  BillingEntitlement,
  UsagePolicyEntitlement,
  UsagePolicyRule,
  UserPlanEntitlement,
} from '@/api/billing'

type Translator = typeof translate

export function isPlanEntitlementReplacementCandidate(
  entitlement: Pick<UserPlanEntitlement, 'status' | 'expires_at'>,
  nowMs = Date.now(),
): boolean {
  if (entitlement.status !== 'active' || !entitlement.expires_at) return false
  const expiresAtMs = Date.parse(entitlement.expires_at)
  return Number.isFinite(expiresAtMs) && expiresAtMs > nowMs
}

export function entitlementsWillReplaceExisting(
  incoming: BillingEntitlement[] | undefined,
  existing: BillingEntitlement[] | undefined
): boolean {
  const incomingItems = incoming || []
  const existingItems = existing || []
  if (
    hasEntitlementType(incomingItems, 'daily_quota')
    && hasEntitlementType(existingItems, 'daily_quota')
  ) return true
  if (
    hasEntitlementType(incomingItems, 'membership_group')
    && hasEntitlementType(existingItems, 'membership_group')
  ) return true

  const incomingGroups = new Set(entitlementReplacementGroups(incomingItems))
  return incomingGroups.size > 0
    && entitlementReplacementGroups(existingItems).some(group => incomingGroups.has(group))
}

export function entitlementReplacementGroups(
  entitlements: BillingEntitlement[] | undefined
): string[] {
  return [...new Set(
    (entitlements || [])
      .map(item => item.replacement_group?.trim() || '')
      .filter(Boolean)
  )]
}

function hasEntitlementType(
  entitlements: BillingEntitlement[],
  type: BillingEntitlement['type']
): boolean {
  return entitlements.some(item => item.type === type)
}

export function usagePolicyEntitlementLabels(
  policy: UsagePolicyEntitlement,
  t: Translator = translate,
): string[] {
  const labels = policy.rules.map(rule => formatUsagePolicyRuleLabel(rule, t))
  const name = policy.name?.trim()

  if (name) labels.unshift(t('billing.entitlements.namedUsagePolicy', { name }))
  return labels.length > 0 ? labels : [t('billing.entitlements.usageLimits')]
}

export function billingEntitlementLabels(
  items: BillingEntitlement[] | undefined,
  t: Translator = translate,
): string[] {
  return (items || []).flatMap((item) => {
    if (item.type === 'wallet_credit') return t('billing.entitlements.walletCredit', { amount: Number(item.amount_usd || 0).toFixed(2) })
    if (item.type === 'daily_quota') return t('billing.entitlements.dailyQuota', { amount: Number(item.daily_quota_usd || 0).toFixed(2) })
    if (item.type === 'membership_group') return t('billing.entitlements.membership')
    if (item.type === 'usage_policy') return usagePolicyEntitlementLabels(item, t)
    return []
  })
}

function formatUsagePolicyRuleLabel(rule: UsagePolicyRule, t: Translator): string {
  const limit = formatLimit(rule.limit)

  if (rule.metric === 'concurrency') return t('billing.entitlements.concurrency', { limit })
  const amount = rule.metric === 'actual_cost_usd' ? formatUsd(rule.limit) : null
  const timezone = rule.window.kind === 'calendar_day'
    || rule.window.kind === 'calendar_week'
    || rule.window.kind === 'calendar_month'
    ? rule.window.timezone?.trim() || t('billing.entitlements.systemTimezone')
    : ''

  switch (rule.window.kind) {
    case 'rolling':
      if (amount) return t('billing.entitlements.rollingCost', { duration: formatWindowDuration(rule.window.seconds, t), amount })
      if (rule.window.seconds === 1) return t('billing.entitlements.qps', { limit })
      if (rule.window.seconds === 60) return t('billing.entitlements.rpm', { limit })
      return t('billing.entitlements.rollingRequests', { duration: formatWindowDuration(rule.window.seconds, t), limit })
    case 'calendar_day':
      return amount
        ? t('billing.entitlements.dailyCost', { amount, timezone })
        : t('billing.entitlements.dailyRequests', { limit, timezone })
    case 'calendar_week':
      return amount
        ? t('billing.entitlements.weeklyCost', { amount, timezone, weekday: formatWeekday(rule.window.week_start, t) })
        : t('billing.entitlements.weeklyRequests', { limit, timezone, weekday: formatWeekday(rule.window.week_start, t) })
    case 'calendar_month':
      return amount
        ? t('billing.entitlements.monthlyCost', { amount, timezone })
        : t('billing.entitlements.monthlyRequests', { limit, timezone })
    case 'subscription_period':
      return amount
        ? t('billing.entitlements.subscriptionCost', { amount })
        : t('billing.entitlements.subscriptionRequests', { limit })
    case 'concurrent':
      return t('billing.entitlements.concurrentWindow', { limit })
  }
}

function formatWeekday(weekStart: number | undefined, t: Translator): string {
  const weekdays = [
    t('billing.entitlements.monday'),
    t('billing.entitlements.tuesday'),
    t('billing.entitlements.wednesday'),
    t('billing.entitlements.thursday'),
    t('billing.entitlements.friday'),
    t('billing.entitlements.saturday'),
    t('billing.entitlements.sunday'),
  ]
  const normalizedWeekStart = typeof weekStart === 'number'
    && Number.isInteger(weekStart)
    && weekStart >= 1
    && weekStart <= 7
    ? weekStart
    : 1
  return weekdays[normalizedWeekStart - 1]
}

function formatUsd(limit: number): string {
  return `$${Number(limit || 0).toLocaleString(getI18nLocale(), { maximumFractionDigits: 8 })}`
}

function formatWindowDuration(seconds: number, t: Translator): string {
  if (seconds % 86_400 === 0) {
    const count = seconds / 86_400
    return t(count === 1 ? 'billing.entitlements.durationDay' : 'billing.entitlements.durationDays', { count })
  }
  if (seconds % 3_600 === 0) {
    const count = seconds / 3_600
    return t(count === 1 ? 'billing.entitlements.durationHour' : 'billing.entitlements.durationHours', { count })
  }
  if (seconds % 60 === 0) {
    const count = seconds / 60
    return t(count === 1 ? 'billing.entitlements.durationMinute' : 'billing.entitlements.durationMinutes', { count })
  }
  return t(seconds === 1 ? 'billing.entitlements.durationSecond' : 'billing.entitlements.durationSeconds', { count: seconds })
}

function formatLimit(limit: number): string {
  return Number(limit || 0).toLocaleString(getI18nLocale())
}

export function formatBillingPlanDuration(unit: BillingDurationUnit, value: number): string {
  if (getI18nLocale() === 'en-US') {
    const plural = value === 1 ? '' : 's'
    const labels: Record<BillingDurationUnit, string> = {
      day: ` day${plural}`,
      month: ` month${plural}`,
      year: ` year${plural}`,
      custom: ` day${plural}`,
    }
    return `${value}${labels[unit] ?? labels.month}`
  }

  const labels: Record<BillingDurationUnit, string> = {
    get day() { return translate('ui.days') },
    get month() { return translate('ui.months') },
    get year() { return translate('ui.years') },
    get custom() { return translate('ui.days') },
  }
  return `${value}${labels[unit] ?? labels.month}`
}
