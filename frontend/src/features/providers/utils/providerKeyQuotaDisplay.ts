
import { translate as t } from '@/i18n'
import { getI18nLocale } from '@/i18n'
import type { EndpointAPIKey } from '@/api/endpoints'
import type {
  CodexUpstreamMetadata,
  ChatGPTWebUpstreamMetadata,
  GrokUpstreamMetadata,
  KiroUpstreamMetadata,
  WindsurfUpstreamMetadata,
  XaiUpstreamMetadata,
  QuotaResetCreditsSnapshot,
  QuotaStatusSnapshot,
  QuotaWindowSnapshot,
} from '@/api/endpoints/types'
import { getCodexPrimaryQuotaWindow, getCodexQuotaWindowLimitLabel } from '@/features/providers/utils/codexQuotaWindow'
import { formatCompactNumber } from '@/utils/format'
import {
  formatCodexResetCreditCount as formatCodexResetCreditCountLabel,
  getCodexResetCreditAvailableCount as getCodexResetCreditAvailableCountFromSnapshot,
  getCodexResetCreditReservationIdempotencyKey,
  getVisibleCodexResetCreditItems as getVisibleCodexResetCreditItemsFromSnapshot,
  mergeCodexQuotaDisplays,
  readPendingCodexResetCreditIdempotencyKey,
} from '@/features/providers/components/codex-reset-credit-display'

export function quotaSnapshotHasDisplayData(quota: QuotaStatusSnapshot | null | undefined): boolean {
  if (!quota) return false
  return Boolean(
    (typeof quota.code === 'string' && quota.code.trim().toLowerCase() !== 'unknown')
    || quota.updated_at != null
    || quota.observed_at != null
    || quota.usage_ratio != null
    || (Array.isArray(quota.windows) && quota.windows.length > 0)
    || quota.credits,
  )
}

export function getQuotaSnapshotForProvider(
  key: EndpointAPIKey,
  providerType: 'codex' | 'kiro' | 'windsurf' | 'antigravity' | 'chatgpt_web' | 'gemini_cli' | 'claude_code' | 'grok' | 'xai',
): QuotaStatusSnapshot | null {
  const quota = key.status_snapshot?.quota
  if (!quota) return null

  const snapshotProviderType = quota.provider_type?.trim().toLowerCase()
  if (snapshotProviderType) {
    return snapshotProviderType === providerType ? quota : null
  }

  return quotaSnapshotHasDisplayData(quota) ? quota : null
}

export function getQuotaSnapshotUpdatedAt(quota: QuotaStatusSnapshot | null | undefined): number | undefined {
  const updatedAt = quota?.updated_at ?? quota?.observed_at
  return typeof updatedAt === 'number' ? updatedAt : undefined
}

export function getQuotaWindow(
  quota: QuotaStatusSnapshot | null | undefined,
  code: string,
): QuotaWindowSnapshot | null {
  const windows = quota?.windows
  if (!Array.isArray(windows)) return null
  return windows.find(window => String(window?.code || '').trim().toLowerCase() === code.trim().toLowerCase()) ?? null
}

export function getQuotaWindowUsedPercent(window: QuotaWindowSnapshot | null | undefined): number | undefined {
  if (!window) return undefined
  if (typeof window.used_ratio === 'number') {
    return Math.max(Math.min(window.used_ratio * 100, 100), 0)
  }
  if (typeof window.remaining_ratio === 'number') {
    return Math.max(Math.min((1 - window.remaining_ratio) * 100, 100), 0)
  }
  return undefined
}

export function getQuotaWindowRemainingPercent(window: QuotaWindowSnapshot | null | undefined): number | undefined {
  if (!window) return undefined
  if (typeof window.remaining_ratio === 'number') {
    return Math.max(Math.min(window.remaining_ratio * 100, 100), 0)
  }
  if (typeof window.used_ratio === 'number') {
    return Math.max(Math.min((1 - window.used_ratio) * 100, 100), 0)
  }
  return undefined
}

export function getQuotaWindowResetAt(window: QuotaWindowSnapshot | null | undefined): number | undefined {
  return typeof window?.reset_at === 'number' ? window.reset_at : undefined
}

export function getQuotaWindowResetSeconds(window: QuotaWindowSnapshot | null | undefined): number | undefined {
  return typeof window?.reset_seconds === 'number' ? window.reset_seconds : undefined
}

export function getQuotaWindowByScope(
  quota: QuotaStatusSnapshot | null | undefined,
  scope: string,
): QuotaWindowSnapshot[] {
  const windows = quota?.windows
  if (!Array.isArray(windows)) return []
  return windows.filter(window => String(window?.scope || '').trim().toLowerCase() === scope.trim().toLowerCase())
}

export function getQuotaWindowLiveResetSeconds(
  quota: QuotaStatusSnapshot | null | undefined,
  window: QuotaWindowSnapshot | null | undefined,
): number | null {
  if (!window) return null

  const now = Math.floor(Date.now() / 1000)
  if (typeof window.reset_at === 'number') {
    return Math.max(window.reset_at - now, 0)
  }

  if (typeof window.reset_seconds === 'number') {
    const updatedAt = getQuotaSnapshotUpdatedAt(quota)
    const elapsed = typeof updatedAt === 'number' ? Math.max(now - updatedAt, 0) : 0
    return Math.max(window.reset_seconds - elapsed, 0)
  }

  return null
}

export function copyCodexNumberField(
  target: CodexUpstreamMetadata,
  source: CodexUpstreamMetadata,
  field: keyof CodexUpstreamMetadata,
) {
  const value = source[field]
  if (typeof value === 'number' && Number.isFinite(value)) {
    ;(target[field] as number | undefined) = value
  }
}

export function getCodexQuotaDisplayFromMetadata(metadata: CodexUpstreamMetadata | null | undefined): CodexUpstreamMetadata | null {
  if (!metadata) return null

  const display: CodexUpstreamMetadata = {}
  if (metadata.credential_generation?.trim()) {
    display.credential_generation = metadata.credential_generation.trim()
  }
  if (metadata.plan_type) display.plan_type = metadata.plan_type

  const numberFields: (keyof CodexUpstreamMetadata)[] = [
    'updated_at',
    'primary_used_percent',
    'primary_reset_seconds',
    'primary_reset_after_seconds',
    'primary_reset_at',
    'primary_window_minutes',
    'secondary_used_percent',
    'secondary_reset_seconds',
    'secondary_reset_after_seconds',
    'secondary_reset_at',
    'secondary_window_minutes',
    'spark_primary_used_percent',
    'spark_primary_reset_seconds',
    'spark_primary_reset_after_seconds',
    'spark_primary_reset_at',
    'spark_primary_window_minutes',
    'spark_secondary_used_percent',
    'spark_secondary_reset_seconds',
    'spark_secondary_reset_after_seconds',
    'spark_secondary_reset_at',
    'spark_secondary_window_minutes',
    'credits_balance',
  ]
  numberFields.forEach(field => copyCodexNumberField(display, metadata, field))
  if (metadata.has_credits !== undefined) display.has_credits = metadata.has_credits
  if (metadata.reset_credits) display.reset_credits = metadata.reset_credits
  if (metadata.account_quota_reset_reservation) {
    display.account_quota_reset_reservation = metadata.account_quota_reset_reservation
  }

  return Object.keys(display).length > 0 ? display : null
}

export function getCodexQuotaDisplayFromSnapshot(quota: QuotaStatusSnapshot | null | undefined): CodexUpstreamMetadata | null {
  if (!quota) return null

  const display: CodexUpstreamMetadata = {}
  const updatedAt = getQuotaSnapshotUpdatedAt(quota)
  if (updatedAt !== undefined) display.updated_at = updatedAt
  if (quota.plan_type) display.plan_type = quota.plan_type

  const primaryWindow = getCodexPrimaryQuotaWindow(quota.windows)
  const primaryUsedPercent = getQuotaWindowUsedPercent(primaryWindow)
  if (primaryUsedPercent !== undefined) display.primary_used_percent = primaryUsedPercent
  const primaryResetAt = getQuotaWindowResetAt(primaryWindow)
  if (primaryResetAt !== undefined) display.primary_reset_at = primaryResetAt
  const primaryResetSeconds = getQuotaWindowResetSeconds(primaryWindow)
  if (primaryResetSeconds !== undefined) display.primary_reset_seconds = primaryResetSeconds
  if (typeof primaryWindow?.window_minutes === 'number') {
    display.primary_window_minutes = primaryWindow.window_minutes
  }

  const secondaryWindow = getQuotaWindow(quota, '5h')
  const secondaryUsedPercent = getQuotaWindowUsedPercent(secondaryWindow)
  if (secondaryUsedPercent !== undefined) display.secondary_used_percent = secondaryUsedPercent
  const secondaryResetAt = getQuotaWindowResetAt(secondaryWindow)
  if (secondaryResetAt !== undefined) display.secondary_reset_at = secondaryResetAt
  const secondaryResetSeconds = getQuotaWindowResetSeconds(secondaryWindow)
  if (secondaryResetSeconds !== undefined) display.secondary_reset_seconds = secondaryResetSeconds
  if (typeof secondaryWindow?.window_minutes === 'number') {
    display.secondary_window_minutes = secondaryWindow.window_minutes
  }

  const sparkPrimaryWindow = getQuotaWindow(quota, 'spark_5h')
  const sparkPrimaryUsedPercent = getQuotaWindowUsedPercent(sparkPrimaryWindow)
  if (sparkPrimaryUsedPercent !== undefined) display.spark_primary_used_percent = sparkPrimaryUsedPercent
  const sparkPrimaryResetAt = getQuotaWindowResetAt(sparkPrimaryWindow)
  if (sparkPrimaryResetAt !== undefined) display.spark_primary_reset_at = sparkPrimaryResetAt
  const sparkPrimaryResetSeconds = getQuotaWindowResetSeconds(sparkPrimaryWindow)
  if (sparkPrimaryResetSeconds !== undefined) display.spark_primary_reset_seconds = sparkPrimaryResetSeconds
  if (typeof sparkPrimaryWindow?.window_minutes === 'number') {
    display.spark_primary_window_minutes = sparkPrimaryWindow.window_minutes
  }

  const sparkSecondaryWindow = getQuotaWindow(quota, 'spark_weekly')
  const sparkSecondaryUsedPercent = getQuotaWindowUsedPercent(sparkSecondaryWindow)
  if (sparkSecondaryUsedPercent !== undefined) display.spark_secondary_used_percent = sparkSecondaryUsedPercent
  const sparkSecondaryResetAt = getQuotaWindowResetAt(sparkSecondaryWindow)
  if (sparkSecondaryResetAt !== undefined) display.spark_secondary_reset_at = sparkSecondaryResetAt
  const sparkSecondaryResetSeconds = getQuotaWindowResetSeconds(sparkSecondaryWindow)
  if (sparkSecondaryResetSeconds !== undefined) display.spark_secondary_reset_seconds = sparkSecondaryResetSeconds
  if (typeof sparkSecondaryWindow?.window_minutes === 'number') {
    display.spark_secondary_window_minutes = sparkSecondaryWindow.window_minutes
  }
  if (quota.reset_credits) display.reset_credits = quota.reset_credits

  return Object.keys(display).length > 0 ? display : null
}

export function codexDisplayHasResetCredits(display: CodexUpstreamMetadata | null | undefined): boolean {
  const count = display?.reset_credits?.available_count
  return typeof count === 'number' && Number.isFinite(count)
}

export function getCodexQuotaDisplay(key: EndpointAPIKey): CodexUpstreamMetadata | null {
  const snapshotDisplay = getCodexQuotaDisplayFromSnapshot(getQuotaSnapshotForProvider(key, 'codex'))
  const metadataDisplay = getCodexQuotaDisplayFromMetadata(key.upstream_metadata?.codex)
  return mergeCodexQuotaDisplays(snapshotDisplay, metadataDisplay)
}

export function getCodexPrimaryQuotaLabel(key: EndpointAPIKey): string {
  return getCodexQuotaWindowLimitLabel({
    code: 'weekly',
    label: t('providers.antigravity.quota.period.weekly'),
    window_minutes: getCodexQuotaDisplay(key)?.primary_window_minutes,
  }) || t('ui.weeklyLimit')
}

export function hasCodexQuotaDisplayData(key: EndpointAPIKey): boolean {
  const codex = getCodexQuotaDisplay(key)
  return !!codex && (
    codex.primary_used_percent !== undefined
    || codex.secondary_used_percent !== undefined
    || codex.spark_primary_used_percent !== undefined
    || codex.spark_secondary_used_percent !== undefined
    || codexDisplayHasResetCredits(codex)
    || hasPendingCodexResetCredit(key)
  )
}

export function hasCodexSparkQuotaDisplayData(key: EndpointAPIKey): boolean {
  const codex = getCodexQuotaDisplay(key)
  return !!codex && (
    codex.spark_primary_used_percent !== undefined
    || codex.spark_secondary_used_percent !== undefined
  )
}

export function getCodexResetCreditsDisplay(key: EndpointAPIKey): QuotaResetCreditsSnapshot | null {
  return getCodexQuotaDisplay(key)?.reset_credits ?? null
}

export function getCodexResetCreditAvailableCount(key: EndpointAPIKey): number | null {
  return getCodexResetCreditAvailableCountFromSnapshot(getCodexResetCreditsDisplay(key))
}

export function formatCodexResetCreditCount(key: EndpointAPIKey): string {
  return formatCodexResetCreditCountLabel(getCodexResetCreditAvailableCount(key))
}

export function getVisibleCodexResetCreditItems(key: EndpointAPIKey) {
  return getVisibleCodexResetCreditItemsFromSnapshot(getCodexResetCreditsDisplay(key))
}

export function getPendingCodexResetCreditIdempotencyKey(key: EndpointAPIKey): string | null {
  const serverReservation = getCodexResetCreditReservationIdempotencyKey(getCodexQuotaDisplay(key))
  if (serverReservation) return serverReservation
  const credentialGeneration = getCodexCredentialGeneration(key)
  return credentialGeneration === undefined
    ? null
    : readPendingCodexResetCreditIdempotencyKey(key.id, credentialGeneration)
}

export function getCodexCredentialGeneration(key: EndpointAPIKey): string | null | undefined {
  const codex = key.upstream_metadata?.codex
  if (!codex || typeof codex !== 'object') return undefined
  return codex.credential_generation?.trim() || null
}

export function hasPendingCodexResetCredit(key: EndpointAPIKey): boolean {
  return getPendingCodexResetCreditIdempotencyKey(key) !== null
}

export function getKiroQuotaDisplay(key: EndpointAPIKey): KiroUpstreamMetadata | null {
  const quota = getQuotaSnapshotForProvider(key, 'kiro')
  if (!quota) return null

  const display: KiroUpstreamMetadata = {}
  const updatedAt = getQuotaSnapshotUpdatedAt(quota)
  if (updatedAt !== undefined) display.updated_at = updatedAt
  if (quota.plan_type) display.subscription_title = quota.plan_type

  if (String(quota.code || '').trim().toLowerCase() === 'banned') {
    display.is_banned = true
    if (quota.reason) display.ban_reason = quota.reason
    if (updatedAt !== undefined) display.banned_at = updatedAt
  }

  const usageWindow =
    getQuotaWindow(quota, 'usage')
    ?? getQuotaWindowByScope(quota, 'account')[0]
    ?? null
  if (usageWindow) {
    const usedPercent = getQuotaWindowUsedPercent(usageWindow)
    if (usedPercent !== undefined) display.usage_percentage = usedPercent
    if (typeof usageWindow.used_value === 'number') display.current_usage = usageWindow.used_value
    if (typeof usageWindow.limit_value === 'number') display.usage_limit = usageWindow.limit_value
    if (typeof usageWindow.remaining_value === 'number') display.remaining = usageWindow.remaining_value

    const nextResetAt =
      getQuotaWindowResetAt(usageWindow)
      ?? (() => {
        const resetSeconds = getQuotaWindowResetSeconds(usageWindow)
        if (updatedAt === undefined || resetSeconds === undefined) return undefined
        return updatedAt + resetSeconds
      })()
    if (nextResetAt !== undefined) display.next_reset_at = nextResetAt
  }

  return Object.keys(display).length > 0 ? display : null
}

export function hasKiroQuotaDisplayData(key: EndpointAPIKey): boolean {
  const kiro = getKiroQuotaDisplay(key)
  return !!kiro && (kiro.usage_percentage !== undefined || kiro.usage_limit !== undefined)
}

export function getXaiQuotaDisplay(key: EndpointAPIKey): XaiUpstreamMetadata | null {
  const quota = getQuotaSnapshotForProvider(key, 'xai')
  if (!quota) return null

  const display: XaiUpstreamMetadata = {}
  const updatedAt = getQuotaSnapshotUpdatedAt(quota)
  if (updatedAt !== undefined) display.updated_at = updatedAt
  if (quota.plan_type) display.subscription_title = quota.plan_type

  const usageWindow =
    getQuotaWindow(quota, 'usage')
    ?? getQuotaWindowByScope(quota, 'account')[0]
    ?? null
  if (usageWindow) {
    const usedPercent = getQuotaWindowUsedPercent(usageWindow)
    const remainingPercent = getQuotaWindowRemainingPercent(usageWindow)
    if (usedPercent !== undefined) display.usage_percentage = usedPercent
    if (remainingPercent !== undefined) display.remaining_percentage = remainingPercent
    const usageLabel = String(usageWindow.label || '').trim()
    if (usageLabel) display.usage_label = usageLabel
    if (typeof usageWindow.used_value === 'number') display.current_usage = usageWindow.used_value
    if (typeof usageWindow.limit_value === 'number') display.usage_limit = usageWindow.limit_value
    if (typeof usageWindow.remaining_value === 'number') display.remaining = usageWindow.remaining_value
    const nextResetAt =
      getQuotaWindowResetAt(usageWindow)
      ?? (() => {
        const resetSeconds = getQuotaWindowResetSeconds(usageWindow)
        if (updatedAt === undefined || resetSeconds === undefined) return undefined
        return updatedAt + resetSeconds
      })()
    if (nextResetAt !== undefined) display.next_reset_at = nextResetAt
  }

  const prepaidWindow = getQuotaWindow(quota, 'prepaid')
  if (typeof prepaidWindow?.remaining_value === 'number') {
    display.prepaid_balance = prepaidWindow.remaining_value
  }

  const onDemandWindow = getQuotaWindow(quota, 'on_demand')
  if (typeof onDemandWindow?.limit_value === 'number') display.on_demand_cap = onDemandWindow.limit_value
  if (typeof onDemandWindow?.used_value === 'number') display.on_demand_used = onDemandWindow.used_value
  if (typeof onDemandWindow?.remaining_value === 'number') display.on_demand_remaining = onDemandWindow.remaining_value

  return Object.keys(display).length > 0 ? display : null
}

export function hasXaiQuotaDisplayData(key: EndpointAPIKey): boolean {
  const xai = getXaiQuotaDisplay(key)
  return !!xai && (
    xai.usage_percentage !== undefined
    || xai.remaining_percentage !== undefined
    || xai.prepaid_balance !== undefined
    || xai.on_demand_cap !== undefined
  )
}

export function getXaiUsageLabel(key: EndpointAPIKey): string {
  const display = getXaiQuotaDisplay(key)
  if (display?.usage_label) return display.usage_label
  const title = display?.subscription_title
  return title ? t('providers.migration.quotaUsagebd5f27', { value: String(title) }) : t('ui.usageQuota')
}

export function getXaiUsedPercent(key: EndpointAPIKey): number {
  return Math.min(Math.max(100 - getXaiRemainingPercent(key), 0), 100)
}

export function getXaiRemainingPercent(key: EndpointAPIKey): number {
  const xai = getXaiQuotaDisplay(key)
  if (xai?.remaining_percentage != null && Number.isFinite(xai.remaining_percentage)) {
    return Math.min(Math.max(xai.remaining_percentage, 0), 100)
  }
  if (xai?.usage_percentage != null && Number.isFinite(xai.usage_percentage)) {
    return Math.min(Math.max(100 - xai.usage_percentage, 0), 100)
  }
  return 0
}

export function getXaiOnDemandUsedPercent(key: EndpointAPIKey): number {
  const xai = getXaiQuotaDisplay(key)
  if (!xai?.on_demand_cap || xai.on_demand_cap <= 0) return 0
  return Math.max(Math.min(((xai.on_demand_used || 0) / xai.on_demand_cap) * 100, 100), 0)
}

export type GrokQuotaDisplay = GrokUpstreamMetadata & {
  usage_percentage?: number
  usage_limit?: number
  current_usage?: number
  remaining?: number
  next_reset_at?: number
}

export function getGrokQuotaDisplay(key: EndpointAPIKey): GrokQuotaDisplay | null {
  const quota = getQuotaSnapshotForProvider(key, 'grok')
  if (!quota) return null

  const display: GrokQuotaDisplay = {}
  const updatedAt = getQuotaSnapshotUpdatedAt(quota)
  if (updatedAt !== undefined) display.updated_at = updatedAt
  if (quota.plan_type) display.plan_type = quota.plan_type
  if (quota.pool_tier) display.pool_tier = quota.pool_tier

  const code = String(quota.code || '').trim().toLowerCase()
  if (code === 'banned' || code === 'forbidden') {
    display.is_banned = true
    if (quota.reason) display.ban_reason = quota.reason
  }

  const usageWindow =
    getQuotaWindow(quota, 'usage')
    ?? getQuotaWindowByScope(quota, 'account')[0]
    ?? getQuotaWindowByScope(quota, 'model')
      .map(window => ({
        window,
        remainingPercent: getQuotaWindowRemainingPercent(window),
      }))
      .filter((item): item is { window: QuotaWindowSnapshot, remainingPercent: number } => item.remainingPercent !== undefined)
      .sort((a, b) => a.remainingPercent - b.remainingPercent)[0]?.window
    ?? null
  if (usageWindow) {
    const usedPercent = getQuotaWindowUsedPercent(usageWindow)
    if (usedPercent !== undefined) display.usage_percentage = usedPercent
    if (typeof usageWindow.used_value === 'number') display.current_usage = usageWindow.used_value
    if (typeof usageWindow.limit_value === 'number') display.usage_limit = usageWindow.limit_value
    if (typeof usageWindow.remaining_value === 'number') display.remaining = usageWindow.remaining_value

    const nextResetAt =
      getQuotaWindowResetAt(usageWindow)
      ?? (() => {
        const resetSeconds = getQuotaWindowResetSeconds(usageWindow)
        if (updatedAt === undefined || resetSeconds === undefined) return undefined
        return updatedAt + resetSeconds
      })()
    if (nextResetAt !== undefined) display.next_reset_at = nextResetAt
  }

  return Object.keys(display).length > 0 ? display : null
}

export type WindsurfQuotaDisplay = WindsurfUpstreamMetadata & {
  daily_used_percent?: number
  weekly_used_percent?: number
}

export function getWindsurfQuotaDisplay(key: EndpointAPIKey): WindsurfQuotaDisplay | null {
  const quota = getQuotaSnapshotForProvider(key, 'windsurf')
  const upstream = key.upstream_metadata?.windsurf
  if (!quota && !upstream) return null

  const display: WindsurfQuotaDisplay = {}
  const updatedAt = getQuotaSnapshotUpdatedAt(quota) ?? upstream?.updated_at
  if (updatedAt !== undefined) display.updated_at = updatedAt
  if (quota?.plan_type) display.plan_name = quota.plan_type
  else if (upstream?.plan_name) display.plan_name = upstream.plan_name
  if (quota?.reason) display.last_error = quota.reason
  else if (upstream?.last_error) display.last_error = upstream.last_error
  if (typeof quota?.allowed_models_count === 'number') display.allowed_models_count = quota.allowed_models_count
  else if (typeof upstream?.allowed_models_count === 'number') display.allowed_models_count = upstream.allowed_models_count
  if (quota?.rate_limit) display.rate_limit = quota.rate_limit
  else if (upstream?.rate_limit) display.rate_limit = upstream.rate_limit
  if (Array.isArray(upstream?.models)) display.models = upstream.models

  const dailyWindow = getQuotaWindow(quota, 'daily')
  const dailyRemaining = getQuotaWindowRemainingPercent(dailyWindow)
  const dailyUsed = getQuotaWindowUsedPercent(dailyWindow)
  if (dailyRemaining !== undefined) display.daily_remaining_percent = dailyRemaining
  else if (typeof upstream?.daily_remaining_percent === 'number') display.daily_remaining_percent = upstream.daily_remaining_percent
  if (dailyUsed !== undefined) display.daily_used_percent = dailyUsed
  else if (typeof upstream?.daily_remaining_percent === 'number') display.daily_used_percent = Math.max(100 - upstream.daily_remaining_percent, 0)
  const dailyResetAt = getQuotaWindowResetAt(dailyWindow)
  if (dailyResetAt !== undefined) display.daily_reset_at = dailyResetAt
  else if (typeof upstream?.daily_reset_at === 'number') display.daily_reset_at = upstream.daily_reset_at

  const weeklyWindow = getQuotaWindow(quota, 'weekly')
  const weeklyRemaining = getQuotaWindowRemainingPercent(weeklyWindow)
  const weeklyUsed = getQuotaWindowUsedPercent(weeklyWindow)
  if (weeklyRemaining !== undefined) display.weekly_remaining_percent = weeklyRemaining
  else if (typeof upstream?.weekly_remaining_percent === 'number') display.weekly_remaining_percent = upstream.weekly_remaining_percent
  if (weeklyUsed !== undefined) display.weekly_used_percent = weeklyUsed
  else if (typeof upstream?.weekly_remaining_percent === 'number') display.weekly_used_percent = Math.max(100 - upstream.weekly_remaining_percent, 0)
  const weeklyResetAt = getQuotaWindowResetAt(weeklyWindow)
  if (weeklyResetAt !== undefined) display.weekly_reset_at = weeklyResetAt
  else if (typeof upstream?.weekly_reset_at === 'number') display.weekly_reset_at = upstream.weekly_reset_at

  const promptWindow = getQuotaWindow(quota, 'prompt')
  if (typeof promptWindow?.used_value === 'number') display.prompt_used = promptWindow.used_value
  else if (typeof upstream?.prompt_used === 'number') display.prompt_used = upstream.prompt_used
  if (typeof promptWindow?.limit_value === 'number') display.prompt_limit = promptWindow.limit_value
  else if (typeof upstream?.prompt_limit === 'number') display.prompt_limit = upstream.prompt_limit
  if (typeof promptWindow?.remaining_value === 'number') display.prompt_remaining = promptWindow.remaining_value
  else if (typeof upstream?.prompt_remaining === 'number') display.prompt_remaining = upstream.prompt_remaining

  const flexWindow = getQuotaWindow(quota, 'flex')
  if (typeof flexWindow?.used_value === 'number') display.flex_used = flexWindow.used_value
  else if (typeof upstream?.flex_used === 'number') display.flex_used = upstream.flex_used
  if (typeof flexWindow?.limit_value === 'number') display.flex_limit = flexWindow.limit_value
  else if (typeof upstream?.flex_limit === 'number') display.flex_limit = upstream.flex_limit
  if (typeof flexWindow?.remaining_value === 'number') display.flex_remaining = flexWindow.remaining_value
  else if (typeof upstream?.flex_remaining === 'number') display.flex_remaining = upstream.flex_remaining

  return Object.keys(display).length > 0 ? display : null
}

export function hasGrokQuotaDisplayData(key: EndpointAPIKey): boolean {
  const grok = getGrokQuotaDisplay(key)
  return !!grok && (grok.usage_percentage !== undefined || grok.usage_limit !== undefined)
}

export function hasWindsurfQuotaDisplayData(key: EndpointAPIKey): boolean {
  const windsurf = getWindsurfQuotaDisplay(key)
  return !!windsurf && (
    windsurf.daily_remaining_percent !== undefined
    || windsurf.weekly_remaining_percent !== undefined
    || windsurf.prompt_limit !== undefined
    || windsurf.flex_limit !== undefined
    || windsurf.allowed_models_count !== undefined
    || windsurf.rate_limit !== undefined
    || !!windsurf.last_error
    || (Array.isArray(windsurf.models) && windsurf.models.length > 0)
  )
}

export function isWindsurfUnavailableKey(key: EndpointAPIKey): boolean {
  const code = String(getQuotaSnapshotForProvider(key, 'windsurf')?.code || '').trim().toLowerCase()
  return code === 'banned' || code === 'forbidden' || code === 'quarantined'
}

export function getPositiveQuotaNumber(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) && value > 0 ? value : undefined
}

export function windsurfCooldownHasPositiveReset(key: EndpointAPIKey): boolean {
  const quota = getQuotaSnapshotForProvider(key, 'windsurf')
  const rateLimit = quota?.rate_limit
  if (rateLimit && typeof rateLimit === 'object') {
    const retryAfterMs =
      getPositiveQuotaNumber(rateLimit.retry_after_ms)
      ?? getPositiveQuotaNumber(rateLimit.retryAfterMs)
    if (retryAfterMs !== undefined) return true
  }

  const rateLimitWindow = getQuotaWindow(quota, 'rate_limit')
  return (
    getPositiveQuotaNumber(rateLimitWindow?.reset_seconds) !== undefined
    || getPositiveQuotaNumber(rateLimitWindow?.reset_at) !== undefined
  )
}

export function isWindsurfExhaustedKey(key: EndpointAPIKey): boolean {
  const code = String(getQuotaSnapshotForProvider(key, 'windsurf')?.code || '').trim().toLowerCase()
  if (code === 'cooldown') return windsurfCooldownHasPositiveReset(key)
  return code === 'exhausted' || code === 'rate_limited' || code === 'rate_limit'
}

export function getWindsurfModelPreview(key: EndpointAPIKey): string | null {
  const models = getWindsurfQuotaDisplay(key)?.models
  if (!Array.isArray(models) || models.length === 0) return null
  return models
    .slice(0, 3)
    .map(model => (model.label || model.model_uid || '').trim())
    .filter(Boolean)
    .join(' / ') || null
}

export function hasFiniteNumber(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value)
}

export function hasWindsurfPromptQuota(key: EndpointAPIKey): boolean {
  return hasFiniteNumber(getWindsurfQuotaDisplay(key)?.prompt_limit)
}

export function hasWindsurfFlexQuota(key: EndpointAPIKey): boolean {
  return hasFiniteNumber(getWindsurfQuotaDisplay(key)?.flex_limit)
}

export function hasWindsurfModelCount(key: EndpointAPIKey): boolean {
  return hasFiniteNumber(getWindsurfQuotaDisplay(key)?.allowed_models_count)
}

export function hasWindsurfModelPreview(key: EndpointAPIKey): boolean {
  return !!getWindsurfModelPreview(key)
}

export type ChatGPTWebQuotaDisplay = ChatGPTWebUpstreamMetadata & {
  image_quota_remaining_percent?: number
  image_quota_used_percent?: number
}

export function getChatGPTWebQuotaDisplay(key: EndpointAPIKey): ChatGPTWebQuotaDisplay | null {
  const quota = getQuotaSnapshotForProvider(key, 'chatgpt_web')
  if (!quota) return null

  const display: ChatGPTWebQuotaDisplay = {}
  const updatedAt = getQuotaSnapshotUpdatedAt(quota)
  if (updatedAt !== undefined) display.updated_at = updatedAt
  if (quota.plan_type) display.plan_type = quota.plan_type
  if (quota.code === 'exhausted' || quota.code === 'banned') display.image_quota_blocked = true

  const imageWindow =
    getQuotaWindow(quota, 'image_gen')
    ?? getQuotaWindowByScope(quota, 'account')[0]
    ?? null
  if (imageWindow) {
    const remainingValue = typeof imageWindow.remaining_value === 'number' ? imageWindow.remaining_value : undefined
    const limitValue = typeof imageWindow.limit_value === 'number' ? imageWindow.limit_value : undefined
    const usedValue = typeof imageWindow.used_value === 'number' ? imageWindow.used_value : undefined
    const remainingPercent = getQuotaWindowRemainingPercent(imageWindow)
    const usedPercent = getQuotaWindowUsedPercent(imageWindow)

    if (remainingValue !== undefined) display.image_quota_remaining = remainingValue
    if (limitValue !== undefined) display.image_quota_total = limitValue
    if (usedValue !== undefined) display.image_quota_used = usedValue
    if (remainingPercent !== undefined) display.image_quota_remaining_percent = remainingPercent
    if (usedPercent !== undefined) display.image_quota_used_percent = usedPercent
    if (typeof imageWindow.reset_at === 'number') display.image_quota_reset_at = imageWindow.reset_at
    if (typeof imageWindow.reset_seconds === 'number') {
      const resetAt = updatedAt === undefined ? undefined : updatedAt + imageWindow.reset_seconds
      if (resetAt !== undefined && display.image_quota_reset_at === undefined) {
        display.image_quota_reset_at = resetAt
      }
    }
  }

  return Object.keys(display).length > 0 ? display : null
}

export function hasChatGPTWebQuotaDisplayData(key: EndpointAPIKey): boolean {
  const display = getChatGPTWebQuotaDisplay(key)
  return !!display && (
    display.image_quota_remaining_percent !== undefined
    || display.image_quota_total !== undefined
    || display.image_quota_used !== undefined
  )
}

export function getChatGPTWebQuotaUsedPercent(key: EndpointAPIKey): number {
  const display = getChatGPTWebQuotaDisplay(key)
  if (!display) return 0
  if (typeof display.image_quota_used_percent === 'number') return display.image_quota_used_percent
  if (typeof display.image_quota_remaining_percent === 'number') {
    return Math.max(100 - display.image_quota_remaining_percent, 0)
  }
  return 0
}

export function getChatGPTWebQuotaRemainingPercent(key: EndpointAPIKey): number {
  const display = getChatGPTWebQuotaDisplay(key)
  if (!display) return 0
  if (typeof display.image_quota_remaining_percent === 'number') return display.image_quota_remaining_percent
  if (typeof display.image_quota_used_percent === 'number') {
    return Math.max(100 - display.image_quota_used_percent, 0)
  }
  return 0
}

export function formatChatGPTWebUsage(value: number | null | undefined): string {
  if (value === undefined || value === null) return '-'
  if (Math.abs(value - Math.round(value)) < 1e-6) {
    return String(Math.round(value))
  }
  return value.toFixed(1)
}

export function isKiroBannedKey(key: EndpointAPIKey): boolean {
  const quota = getQuotaSnapshotForProvider(key, 'kiro')
  return String(quota?.code || '').trim().toLowerCase() === 'banned'
}

// 格式化封禁/禁止时间（后端返回秒级时间戳，Kiro/Antigravity 通用）
export function formatBanTimestamp(timestamp: number | null | undefined): string {
  if (!timestamp) return ''
  const date = new Date(timestamp * 1000)
  return date.toLocaleString(getI18nLocale(), {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export function isAntigravityForbiddenKey(key: EndpointAPIKey): boolean {
  const quota = getQuotaSnapshotForProvider(key, 'antigravity')
  return String(quota?.code || '').trim().toLowerCase() === 'forbidden'
}

export function getAntigravityForbiddenReason(key: EndpointAPIKey): string | undefined {
  const quota = getQuotaSnapshotForProvider(key, 'antigravity')
  return quota?.reason || undefined
}

export function getAntigravityForbiddenAt(key: EndpointAPIKey): number | undefined {
  return getQuotaSnapshotUpdatedAt(getQuotaSnapshotForProvider(key, 'antigravity'))
}

export function getAntigravityQuotaUpdatedAt(key: EndpointAPIKey): number | undefined {
  return getQuotaSnapshotUpdatedAt(getQuotaSnapshotForProvider(key, 'antigravity'))
}

// 格式化 Kiro 使用量（带单位）
export function formatKiroUsage(value: number | undefined): string {
  if (value === undefined || value === null) return '-'
  const normalized = Number(value)
  if (!Number.isFinite(normalized)) return '-'
  if (normalized >= 1000) return formatCompactNumber(normalized, { fractionDigits: 1 })
  return normalized.toFixed(1)
}

// 格式化 Kiro 订阅类型显示
export function formatKiroSubscription(title: string | undefined): string {
  if (!title) return ''
  // 简化显示：KIRO PRO+ -> Pro+, KIRO FREE -> Free（首字母大写，与 Codex 保持一致）
  const upper = title.toUpperCase()
  if (upper.includes('POWER')) return 'Power'
  if (upper.includes('PRO+')) return 'Pro+'
  if (upper.includes('PRO')) return 'Pro'
  if (upper.includes('FREE')) return 'Free'
  return title
}

export function getKiroSubscriptionTitle(key: EndpointAPIKey): string | undefined {
  return getKiroQuotaDisplay(key)?.subscription_title
}

export function getKiroSubscriptionBadgeLabel(key: EndpointAPIKey): string {
  return formatKiroSubscription(getKiroSubscriptionTitle(key))
}
