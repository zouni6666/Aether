import { getI18nLocale, translate, type Locale, type MessageKey } from '@/i18n'
import type { PoolKeyDetail } from '@/api/endpoints/pool'
import { buildPoolMobileTagItems } from '@/features/pool/utils/poolMobilePresentation'
import type { PoolMobileTagItem, PoolMobileTagTone } from '@/features/pool/utils/poolMobilePresentation'
import { getOAuthOrgBadge } from '@/features/providers/utils/oauthIdentity'
import { formatOAuthPlanType } from '@/features/providers/utils/oauthPlanType'
import { getProviderAuthLabel } from '@/features/providers/utils/providerKeyAuth'
import {
  getAccountStatusDisplay,
  getOAuthRefreshButtonTitle as resolveOAuthRefreshButtonTitle,
  getOAuthStatusDisplayWithFallback,
  getOAuthStatusTitle as resolveOAuthStatusTitle,
} from '@/features/providers/utils/providerKeyStatus'
import { getLegacyAccountQuotaText } from '@/features/providers/utils/providerKeyQuota'
import { getQuotaSnapshot, resolvePoolKeyPlanType } from './poolKeyQuotaProgress'

export type PoolKeyScore = NonNullable<PoolKeyDetail['pool_score']>

export const poolScoreHardStateOptions = [
  { value: 'all', get label() { return translate('ui.allStatuses') } },
  { value: 'available', get label() { return translate('ui.available4d99c97') } },
  { value: 'unknown', get label() { return translate('chart.unknown') } },
  { value: 'cooldown', get label() { return translate('ui.cooldown') } },
  { value: 'quota_exhausted', get label() { return translate('ui.quotaExhausted') } },
  { value: 'auth_invalid', get label() { return translate('ui.invalidAuthorization') } },
  { value: 'banned', get label() { return translate('ui.banned') } },
  { value: 'inactive', get label() { return translate('ui.disable') } },
]

export const poolScoreProbeStatusOptions = [
  { value: 'all', get label() { return translate('ui.allProbes') } },
  { value: 'never', get label() { return translate('ui.notProbed') } },
  { value: 'ok', get label() { return translate('ui.normal') } },
  { value: 'failed', get label() { return translate('ui.failed28384d7') } },
  { value: 'stale', get label() { return translate('ui.expired97399bd') } },
  { value: 'in_progress', get label() { return translate('ui.probing') } },
]

export const COOLDOWN_REASON_MAP: Record<string, string> = {
  get rate_limited_429() { return translate('ui.429RateLimited') },
  get forbidden_403() { return translate('ui.403Forbidden') },
  get overloaded_529() { return translate('ui.529Overloaded') },
  get auth_failed_401() { return translate('ui.401Unauthorized') },
  get payment_required_402() { return translate('ui.402PaymentRequired') },
  get server_error_500() { return translate('ui.500ServerError') },
  get request_timeout_408() { return translate('ui.408RequestTimeout') },
  get conflict_409() { return translate('ui.409Conflict') },
  get locked_423() { return translate('ui.423Locked') },
  too_early_425: '425 Too Early',
  get bad_gateway_502() { return translate('ui.502BadGateway') },
  get service_unavailable_503() { return translate('ui.503ServiceUnavailable') },
  get gateway_timeout_504() { return translate('ui.504GatewayTimeout') },
}

export function formatCooldownReason(reason: string): string {
  return COOLDOWN_REASON_MAP[reason] || reason
}

export type PoolStatusVariant = 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning' | 'dark'

export function isHealthDerivedSchedulingReason(reason: string | null | undefined): boolean {
  const normalized = String(reason || '').trim().toLowerCase()
  return normalized === 'health_low'
    || normalized === 'health_degraded'
    || normalized === 'health'
    || normalized === 'circuit_open'
    || normalized === 'circuit_breaker'
}

export function isHealthDerivedSchedulingLabel(label: string | null | undefined): boolean {
  const normalized = String(label || '').trim()
  return normalized === '健康低'
    || normalized === '健康度较低'
    || normalized === '降级'
    || normalized === '熔断'
    || normalized === '熔断中'
}

export function getVisibleSchedulingReason(key: PoolKeyDetail): string | null {
  const reason = String(key.scheduling_reason || '').trim()
  if (!reason || isHealthDerivedSchedulingReason(reason)) return null
  return reason
}

export function getVisibleSchedulingReasons(key: PoolKeyDetail) {
  return (key.scheduling_reasons ?? []).filter((item) => {
    const source = String(item.source || '').trim().toLowerCase()
    return source !== 'health'
      && !isHealthDerivedSchedulingReason(item.code)
      && !isHealthDerivedSchedulingLabel(item.label)
  })
}

export function getSchedulingStatus(key: PoolKeyDetail, tick: number): 'available' | 'degraded' | 'blocked' {
  if (getAccountAlertLabel(key)) return 'blocked'
  if (getBlockingOAuthStatusLabel(key, tick)) return 'blocked'

  const status = key.scheduling_status
  if (
    (status === 'available' || status === 'degraded' || status === 'blocked')
    && !isHealthDerivedSchedulingReason(key.scheduling_reason)
    && !isHealthDerivedSchedulingLabel(key.scheduling_label)
  ) {
    return status
  }

  if (!key.is_active) return 'blocked'
  if (key.cooldown_reason) return 'degraded'
  if (key.cost_limit != null && key.cost_limit > 0 && key.cost_window_usage >= key.cost_limit) return 'blocked'
  return 'available'
}

export function compactPoolStatusLabel(label: string | null | undefined): string | null {
  const rawLabel = String(label || '')
  const normalized = rawLabel.trim()
  if (!normalized) return null

  const knownLabels: Record<string, MessageKey> = {
    'Token 失效': 'ui.invalid',
    'Token 过期': 'ui.expired',
    Token失效: 'ui.invalid',
    Token过期: 'ui.expired',
    账号已封禁: 'ui.accountBanned',
    账号封禁: 'ui.accountBanned',
    工作区已停用: 'ui.workspaceDisabled',
    工作区停用: 'ui.workspaceDisabled',
    账号访问受限: 'ui.accessRestricted',
    访问受限: 'ui.accessRestricted',
    账号停用: 'ui.accountDisabled',
    需要验证: 'ui.verificationRequired',
    账号隔离: 'ui.accountIsolation',
    账号异常: 'ui.accountIssue',
    健康度较低: 'ui.lowHealth',
  }
  const key = knownLabels[normalized]
  // Unknown labels may be server or user content. Do not truncate them or
  // attempt to infer a translation from their text.
  return key ? translate(key) : rawLabel
}

export function getOAuthStatusBadgeLabel(status: ReturnType<typeof getVisibleOAuthState>): string | null {
  if (!status) return null
  if (status.requiresReauth) return translate('ui.renewalFailed')
  if (status.isInvalid) return translate('ui.invalid')
  if (status.isExpired) return translate('ui.expired')
  if (status.kind === 'missing_refresh_token') return translate('ui.notAdded')
  if (status.kind === 'unknown_expiration') return translate('chart.unknown')
  if (status.isExpiringSoon) return translate('ui.expiringSoon')
  return translate('ui.valid')
}

export function getBlockingOAuthStatusLabel(key: PoolKeyDetail, tick: number): string | null {
  const oauthState = getVisibleOAuthState(key, tick)
  if (!oauthState?.isInvalid && !oauthState?.isExpired) return null
  return getOAuthStatusBadgeLabel(oauthState)
}

export function isPoolKeyCostExhausted(key: PoolKeyDetail): boolean {
  return key.cost_limit != null
    && key.cost_limit > 0
    && key.cost_window_usage >= key.cost_limit
}

const schedulingLabelKeys: Partial<Record<string, MessageKey>> = {
  available: 'ui.available4d99c97',
  inactive: 'ui.disabledbc5a87a',
  manual_disabled: 'ui.disable',
  cooldown: 'ui.coolingDown',
  account_quota_exhausted: 'ui.quotaExhausted',
  cost_exhausted: 'ui.limitExceeded',
  invalid: 'ui.invalid',
  expired: 'ui.expired',
  banned: 'ui.accountBanned',
  forbidden: 'ui.accessRestricted',
  quarantined: 'ui.accountIsolation',
  rate_limited: 'ui.rateLimited6d5ac6f',
  oauth_token_invalid: 'ui.invalid',
  oauth_token_expired: 'ui.expired',
  account_banned: 'ui.accountBanned',
  account_suspended: 'ui.accountBanned',
  account_disabled: 'ui.accountDisabled',
  workspace_deactivated: 'ui.workspaceDisabled',
  account_forbidden: 'ui.accessRestricted',
  account_verification: 'ui.verificationRequired',
  account_quarantined: 'ui.accountIsolation',
  account_blocked: 'ui.accountIssue',
  quota_exhausted: 'ui.quotaExhausted',
}

function getKnownSchedulingLabel(code: string | null | undefined): string | null {
  const key = schedulingLabelKeys[String(code || '').trim().toLowerCase()]
  return key ? translate(key) : null
}

export function getSchedulingBadgeLabel(key: PoolKeyDetail, tick: number): string {
  const accountAlert = getAccountAlertLabel(key)
  if (accountAlert) return compactPoolStatusLabel(accountAlert) || accountAlert
  const oauthAlert = getBlockingOAuthStatusLabel(key, tick)
  if (oauthAlert) return oauthAlert

  const knownLabel = getKnownSchedulingLabel(getVisibleSchedulingReason(key))
  if (knownLabel) {
    if (key.scheduling_reason === 'available' && isPoolKeyCostExhausted(key)) return translate('ui.limitExceeded')
    return knownLabel
  }

  const rawLabel = String(key.scheduling_label || '')
  const normalizedLabel = rawLabel.trim()
  if (
    normalizedLabel
    && !isHealthDerivedSchedulingReason(key.scheduling_reason)
    && !isHealthDerivedSchedulingLabel(rawLabel)
  ) {
    if ((normalizedLabel === '可用' || key.scheduling_reason === 'available') && isPoolKeyCostExhausted(key)) {
      return translate('ui.limitExceeded')
    }
    if (normalizedLabel === '禁用' || normalizedLabel === '停用') return translate('ui.disable')
    return compactPoolStatusLabel(rawLabel) || rawLabel
  }

  if (!key.is_active) return translate('ui.disabledbc5a87a')
  if (key.cooldown_reason) return translate('ui.coolingDown')
  return translate('ui.available4d99c97')
}

export function getSchedulingBadgeVariant(key: PoolKeyDetail, tick: number): PoolStatusVariant {
  if (getAccountAlertLabel(key)) return 'destructive'
  if (getBlockingOAuthStatusLabel(key, tick)) return 'destructive'

  const reason = getVisibleSchedulingReason(key)
  if (reason === 'manual_disabled' || reason === 'inactive') return 'secondary'
  if (reason === 'account_blocked' || reason === 'account_quota_exhausted' || reason === 'cost_exhausted') return 'destructive'
  if (reason === 'cooldown') return 'warning'
  if (reason === 'cost_soft' || reason === 'cost') return 'warning'
  if (isPoolKeyCostExhausted(key)) return 'destructive'
  if (reason === 'available') return 'default'
  if (!reason && !key.is_active) return 'secondary'

  const status = getSchedulingStatus(key, tick)
  if (status === 'blocked') return 'destructive'
  if (status === 'degraded') return 'warning'
  return 'default'
}

export function getSchedulingTitle(key: PoolKeyDetail, tick: number): string {
  const accountAlertTitle = getAccountAlertTitle(key)
  if (accountAlertTitle) return accountAlertTitle
  if (getBlockingOAuthStatusLabel(key, tick)) return getOAuthStatusTitle(key, tick)

  const reasons = getVisibleSchedulingReasons(key)
  if (reasons.length > 0) {
    return reasons.map((item) => {
      const ttl = item.ttl_seconds && item.ttl_seconds > 0 ? ` (${formatTTL(item.ttl_seconds)})` : ''
      const detail = item.detail ? ` - ${item.detail}` : ''
      return `${getKnownSchedulingLabel(item.code) ?? item.label}${ttl}${detail}`
    }).join('\n')
  }

  if (key.cooldown_reason) {
    const ttl = key.cooldown_ttl_seconds ? ` (${formatTTL(key.cooldown_ttl_seconds)})` : ''
    return `${formatCooldownReason(key.cooldown_reason)}${ttl}`
  }
  if (isPoolKeyCostExhausted(key)) return translate('ui.limitExceeded')
  return getSchedulingBadgeLabel(key, tick)
}

export function formatTTL(seconds: number): string {
  if (seconds <= 0) return ''
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return m > 0 ? `${m}m ${s}s` : `${s}s`
}

export function getRowClass(key: PoolKeyDetail, tick: number): string {
  const status = getSchedulingStatus(key, tick)
  if (!key.is_active || status === 'blocked') return 'bg-muted/50 opacity-60'
  return ''
}

export function getAuthTypeChipLabel(key: PoolKeyDetail): string {
  return getProviderAuthLabel(key)
}

export function getMobileOAuthTone(key: PoolKeyDetail, tick: number): PoolMobileTagTone | null {
  const oauthState = getVisibleOAuthState(key, tick)
  if (!oauthState) return null
  if (oauthState.isInvalid || oauthState.isExpired) return 'danger'
  if (oauthState.isExpiringSoon) return 'warning'
  return 'muted'
}

export function getMobileTagItems(key: PoolKeyDetail, tick: number): PoolMobileTagItem[] {
  const accountAlert = getAccountAlertLabel(key)
  const oauthState = getVisibleOAuthState(key, tick)
  const orgBadge = getOAuthOrgBadge(key)
  const planType = resolvePoolKeyPlanType(key)

  return buildPoolMobileTagItems({
    accountStatusLabel: compactPoolStatusLabel(accountAlert),
    accountStatusTone: accountAlert ? 'danger' : null,
    oauthStatusLabel: getOAuthStatusBadgeLabel(oauthState),
    oauthStatusTone: getMobileOAuthTone(key, tick),
    priorityLabel: `P${key.internal_priority ?? 50}`,
    authLabel: getAuthTypeChipLabel(key),
    planLabel: planType ? formatOAuthPlanType(planType) : null,
    orgLabel: orgBadge?.label ?? null,
    proxyLabel: key.proxy?.node_id ? translate('ui.dedicatedProxy') : null,
  })
}

export function getMobileTagClass(item: PoolMobileTagItem): string {
  if (item.tone === 'danger') {
    return 'border-red-500/30 bg-red-500/10 text-red-700 dark:text-red-300'
  }
  if (item.tone === 'warning') {
    return 'border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300'
  }
  if (item.tone === 'accent') {
    return 'border-blue-500/30 bg-blue-500/10 text-blue-700 dark:text-blue-300'
  }
  if (item.tone === 'muted') {
    return 'border-border/60 bg-background/70 text-muted-foreground'
  }
  return 'border-border/60 bg-background/80 text-foreground/80'
}

export function getVisibleOAuthState(key: PoolKeyDetail, tick: number) {
  return getOAuthStatusDisplayWithFallback(key, tick)
}

export function getOAuthRefreshButtonTitle(key: PoolKeyDetail, tick: number): string {
  return resolveOAuthRefreshButtonTitle(key, tick)
}

export function getOAuthStatusTitle(key: PoolKeyDetail, tick: number): string {
  return resolveOAuthStatusTitle(key, tick)
}

export const _accountAlertCache = new WeakMap<PoolKeyDetail, Partial<Record<Locale, string | null>>>()

export function getQuotaAlertSnapshotState(key: PoolKeyDetail): { label: string, title: string } | null {
  const quota = getQuotaSnapshot(key)
  if (!quota) return null

  const code = String(quota.code || '').trim().toLowerCase()
  if (!['banned', 'forbidden', 'quarantined', 'rate_limited', 'exhausted'].includes(code)) return null

  const knownLabel = getKnownSchedulingLabel(code === 'exhausted' ? 'account_quota_exhausted' : code)
  const label = knownLabel ?? String(quota.label || '').trim()

  const reason = String(quota.reason || '').trim()
  return {
    label,
    title: reason ? `${label}: ${reason}` : label,
  }
}

export function getAccountAlertLabel(key: PoolKeyDetail): string | null {
  const locale = getI18nLocale()
  const cached = _accountAlertCache.get(key)?.[locale]
  if (cached !== undefined) return cached

  const account = getAccountStatusDisplay(key)
  let result: string | null = (account.blocked || account.label)
    ? getKnownSchedulingLabel(account.code) ?? account.label
    : null
  const quotaAlert = getQuotaAlertSnapshotState(key)
  if (!result && quotaAlert) result = quotaAlert.label
  if (!result && !getQuotaSnapshot(key)) {
    const quotaText = getLegacyAccountQuotaText(key)
    if (quotaText === '账号已封禁' || quotaText === '封禁') result = translate('ui.accountBanned')
    else if (quotaText === '访问受限') result = translate('ui.accessRestricted')
  }

  _accountAlertCache.set(key, { ..._accountAlertCache.get(key), [locale]: result })
  return result
}

export function getAccountAlertTitle(key: PoolKeyDetail): string {
  const label = getAccountAlertLabel(key)
  if (!label) return ''

  const account = getAccountStatusDisplay(key)
  if (account.blocked && account.label) {
    return account.reason ? `${label}: ${account.reason}` : label
  }

  const quotaAlert = getQuotaAlertSnapshotState(key)
  if (quotaAlert?.title) return quotaAlert.title

  const quotaText = getLegacyAccountQuotaText(key)
  if (quotaText) return `${label}: ${quotaText}`
  return label
}

export function getQuotaTextClass(quotaText: string): string {
  if (quotaText.includes('封禁') || quotaText.includes('受限')) {
    return 'text-[11px] text-destructive leading-4'
  }
  return 'text-[11px] text-foreground/90 leading-4'
}

export function formatPoolScore(value: number | null | undefined): string {
  const n = Number(value)
  if (!Number.isFinite(n)) return '-'
  return n.toFixed(3)
}

export function formatPoolScoreReason(value: PoolKeyScore['score_reason'] | null | undefined): string {
  if (!value) return translate('ui.noCalculationResults')
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return String(value)
  }
}

export function getPoolScoreHardStateLabel(value: PoolKeyScore['hard_state'] | null | undefined): string {
  if (!value) return '-'
  return poolScoreHardStateOptions.find(item => item.value === value)?.label || value
}

export function getPoolScoreProbeStatusLabel(value: PoolKeyScore['probe_status'] | null | undefined): string {
  if (!value) return '-'
  return poolScoreProbeStatusOptions.find(item => item.value === value)?.label || value
}

export function formatUnixSeconds(seconds: number | null | undefined): string {
  const raw = Number(seconds ?? 0)
  if (!Number.isFinite(raw) || raw <= 0) return '-'
  return formatRelativeTime(new Date(raw * 1000).toISOString())
}

export function formatRelativeTime(isoStr: string): string {
  const date = new Date(isoStr)
  const pad = (n: number) => String(n).padStart(2, '0')
  const M = pad(date.getMonth() + 1)
  const D = pad(date.getDate())
  const h = pad(date.getHours())
  const m = pad(date.getMinutes())
  return `${M}-${D} ${h}:${m}`
}

export function formatPoolKeyImportedAt(key: PoolKeyDetail): string {
  const value = key.imported_at || key.created_at
  return value ? formatRelativeTime(value) : '-'
}
