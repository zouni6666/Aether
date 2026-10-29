
import { translate as t } from '@/i18n'
import type {
  ProviderKeyStatusSnapshot,
  QuotaStatusSnapshot,
  QuotaWindowSnapshot,
} from '@/api/endpoints/types/statusSnapshot'
import type { UpstreamMetadata } from '@/api/endpoints/types/provider'
import { getCodexQuotaWindowPresentation } from '@/features/providers/utils/codexQuotaWindow'

export interface ProviderKeyQuotaCarrier {
  account_quota?: string | null
  status_snapshot?: ProviderKeyStatusSnapshot | null
  upstream_metadata?: UpstreamMetadata | null
}

function normalizeText(value: unknown): string | null {
  if (typeof value !== 'string') return null
  const text = value.trim()
  return text || null
}

function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 0
  if (value < 0) return 0
  if (value > 100) return 100
  return value
}

function formatPercent(value: number): string {
  return `${clampPercent(value).toFixed(1)}%`
}

function getQuotaSnapshot(
  input: ProviderKeyQuotaCarrier,
): QuotaStatusSnapshot | null {
  return input.status_snapshot?.quota ?? null
}

function getQuotaProviderType(
  quota: QuotaStatusSnapshot | null | undefined,
  fallbackProviderType?: string | null,
): string {
  const snapshotProviderType = normalizeText(quota?.provider_type)?.toLowerCase()
  if (snapshotProviderType) return snapshotProviderType
  return normalizeText(fallbackProviderType)?.toLowerCase() || ''
}

function getQuotaWindows(
  quota: QuotaStatusSnapshot | null | undefined,
): QuotaWindowSnapshot[] {
  return Array.isArray(quota?.windows) ? quota.windows : []
}

function getQuotaWindowRemainingPercent(
  window: QuotaWindowSnapshot | null | undefined,
): number | null {
  if (!window) return null
  if (typeof window.remaining_ratio === 'number') {
    return clampPercent(window.remaining_ratio * 100)
  }
  if (typeof window.used_ratio === 'number') {
    return clampPercent((1 - window.used_ratio) * 100)
  }
  if (typeof window.limit_value === 'number' && window.limit_value > 0) {
    if (typeof window.remaining_value === 'number') {
      return clampPercent((window.remaining_value / window.limit_value) * 100)
    }
    if (typeof window.used_value === 'number') {
      return clampPercent((1 - (window.used_value / window.limit_value)) * 100)
    }
  }
  return null
}

function getQuotaWindow(
  quota: QuotaStatusSnapshot | null | undefined,
  code: string,
): QuotaWindowSnapshot | null {
  const normalizedCode = code.trim().toLowerCase()
  return getQuotaWindows(quota).find(window => normalizeText(window.code)?.toLowerCase() === normalizedCode) ?? null
}

function finiteNumber(value: unknown): number | null {
  if (typeof value === 'number') return Number.isFinite(value) ? value : null
  if (typeof value === 'string') {
    const parsed = Number(value.trim())
    return Number.isFinite(parsed) ? parsed : null
  }
  return null
}

function firstFiniteNumber(...values: unknown[]): number | null {
  for (const value of values) {
    const parsed = finiteNumber(value)
    if (parsed != null) return parsed
  }
  return null
}

function objectValue(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null
}

function positiveNumber(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) && value > 0 ? value : null
}

function windsurfCooldownHasPositiveReset(quota: QuotaStatusSnapshot): boolean {
  const rateLimit = quota.rate_limit
  if (rateLimit && typeof rateLimit === 'object') {
    const retryAfterMs = positiveNumber(rateLimit.retry_after_ms) ?? positiveNumber(rateLimit.retryAfterMs)
    if (retryAfterMs != null) return true
  }

  const rateLimitWindow = getQuotaWindow(quota, 'rate_limit')
  return (
    positiveNumber(rateLimitWindow?.reset_seconds) != null
    || positiveNumber(rateLimitWindow?.reset_at) != null
  )
}

function getQuotaWindowsByScope(
  quota: QuotaStatusSnapshot | null | undefined,
  scope: string,
): QuotaWindowSnapshot[] {
  const normalizedScope = scope.trim().toLowerCase()
  return getQuotaWindows(quota).filter(window => normalizeText(window.scope)?.toLowerCase() === normalizedScope)
}

function formatQuotaValue(value: number | null | undefined): string {
  const normalized = Number(value)
  if (!Number.isFinite(normalized)) return '0'
  const rounded = Math.round(normalized)
  if (Math.abs(normalized - rounded) < 1e-6) {
    return String(rounded)
  }
  return normalized.toFixed(1)
}

function getQuotaWindowValueText(window: QuotaWindowSnapshot | null | undefined): string | null {
  if (!window || typeof window.limit_value !== 'number' || window.limit_value <= 0) return null
  if (typeof window.remaining_value === 'number') {
    return `${formatQuotaValue(window.remaining_value)}/${formatQuotaValue(window.limit_value)}`
  }
  if (typeof window.used_value === 'number') {
    return `${formatQuotaValue(Math.max(window.limit_value - window.used_value, 0))}/${formatQuotaValue(window.limit_value)}`
  }
  return null
}

function getGeminiCliCreditsTextFromQuota(quota: QuotaStatusSnapshot | null | undefined): string | null {
  const credits = quota?.credits
  if (!credits) return null
  const remaining = firstFiniteNumber(credits.remaining, credits.balance)
  if (remaining != null) return t('providers.migration.aiCreditsRemainingd6f546', { value: String(formatQuotaValue(remaining)) })
  if (credits.unlimited === true) return t('providers.migration.unlimitedAiCreditsa560b5')
  if (credits.has_credits === true) return t('providers.migration.aiCreditsAvailable6fb50e')
  if (credits.has_credits === false) return t('providers.migration.aiCreditsExhaustedb52d7e')
  return null
}

export function getGeminiCliAccountCreditsText(
  input: ProviderKeyQuotaCarrier,
  fallbackProviderType?: string | null,
): string | null {
  const quota = getQuotaSnapshot(input)
  if (getQuotaProviderType(quota, fallbackProviderType) !== 'gemini_cli') return null

  const quotaText = getGeminiCliCreditsTextFromQuota(quota)
  if (quotaText) return quotaText

  const metadata = input.upstream_metadata?.gemini_cli
  const credits = objectValue(metadata?.credits)
  const paidTier = objectValue(metadata?.paidTier)
  const currentTier = objectValue(metadata?.currentTier)
  const remaining = firstFiniteNumber(
    credits?.remaining,
    credits?.remainingCredits,
    credits?.available,
    credits?.availableCredits,
    credits?.balance,
    paidTier?.availableCredits,
    paidTier?.remainingCredits,
    currentTier?.availableCredits,
    currentTier?.remainingCredits,
  )
  if (remaining != null) return t('providers.migration.aiCreditsRemainingd6f546', { value: String(formatQuotaValue(remaining)) })

  const hasCredits = typeof credits?.has_credits === 'boolean'
    ? credits.has_credits
    : typeof paidTier?.hasCredits === 'boolean'
      ? paidTier.hasCredits
      : typeof currentTier?.hasCredits === 'boolean'
        ? currentTier.hasCredits
        : null
  if (hasCredits === true) return t('providers.migration.aiCreditsAvailable6fb50e')
  if (hasCredits === false) return t('providers.migration.aiCreditsExhaustedb52d7e')

  const unlimited = credits?.unlimited === true || paidTier?.unlimited === true || currentTier?.unlimited === true
  return unlimited ? t('providers.migration.unlimitedAiCreditsa560b5') : null
}

const GROK_QUOTA_MODE_LABELS: Record<string, string> = {
  quota_auto: 'Auto',
  auto: 'Auto',
  quota_fast: 'Fast',
  fast: 'Fast',
  quota_expert: 'Expert',
  expert: 'Expert',
  quota_heavy: 'Heavy',
  heavy: 'Heavy',
  quota_grok_4_3: 'Grok 4.3',
  'grok-420-computer-use-sa': 'Grok 4.3',
}

function getGrokQuotaWindowLabel(window: QuotaWindowSnapshot): string {
  const rawCode = normalizeText(window.code)?.replace(/^model:/i, '') || ''
  const rawLabel = normalizeText(window.label) || normalizeText(window.model) || rawCode
  const normalized = (rawLabel || rawCode).trim().toLowerCase()
  return GROK_QUOTA_MODE_LABELS[normalized] || GROK_QUOTA_MODE_LABELS[rawCode.toLowerCase()] || rawLabel || rawCode || t('ui.mode')
}

function getCodexQuotaText(quota: QuotaStatusSnapshot): string | null {
  const parts: string[] = []
  for (const window of getQuotaWindows(quota)) {
    const presentation = getCodexQuotaWindowPresentation(window)
    const remainingPercent = getQuotaWindowRemainingPercent(window)
    if (!presentation || remainingPercent == null) continue
    parts.push(t('providers.migration.remaining9f01ec', { value: String(presentation.label), value_2: String(formatPercent(remainingPercent)) }))
  }
  if (parts.length > 0) return parts.join(' | ')

  if (quota.credits?.has_credits === true && typeof quota.credits.balance === 'number') {
    return t('providers.migration.credits57495d', { value: String(quota.credits.balance.toFixed(2)) })
  }
  if (quota.credits?.has_credits === true) return t('providers.migration.creditsAvailable02d50c')
  if (quota.credits?.has_credits === false) return t('providers.migration.noCreditsAvailable2c5d3a')

  return normalizeText(quota.label)
}

function getKiroQuotaText(quota: QuotaStatusSnapshot): string | null {
  const code = normalizeText(quota.code)?.toLowerCase()
  if (code === 'banned') {
    return normalizeText(quota.label) || t('ui.accountBanned210dc8d')
  }

  const window = getQuotaWindow(quota, 'usage') ?? getQuotaWindowsByScope(quota, 'account')[0] ?? null
  const remainingPercent = getQuotaWindowRemainingPercent(window)
  if (typeof window?.remaining_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0 && window.remaining_value <= 0) {
    return t('providers.migration.remaining3e9a48', { value: String(formatQuotaValue(window.remaining_value)), value_2: String(formatQuotaValue(window.limit_value)) })
  }
  if (remainingPercent != null) {
    if (typeof window?.used_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0) {
      return t('providers.migration.remaining40a2ab', { value: String(formatPercent(remainingPercent)), value_2: String(formatQuotaValue(window.used_value)), value_3: String(formatQuotaValue(window.limit_value)) })
    }
    return t('providers.migration.remaining43f955', { value: String(formatPercent(remainingPercent)) })
  }

  if (typeof window?.remaining_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0) {
    return t('providers.migration.remaining3e9a48', { value: String(formatQuotaValue(window.remaining_value)), value_2: String(formatQuotaValue(window.limit_value)) })
  }

  return normalizeText(quota.label)
}

function getXaiQuotaText(quota: QuotaStatusSnapshot): string | null {
  const parts: string[] = []
  const usageText = getKiroQuotaText(quota)
  if (usageText) parts.push(usageText)

  const prepaid = getQuotaWindow(quota, 'prepaid')
  if (typeof prepaid?.remaining_value === 'number') {
    parts.push(t('providers.migration.prepaidRemaining925e85', { value: String(formatQuotaValue(prepaid.remaining_value)) }))
  }

  const onDemand = getQuotaWindow(quota, 'on_demand')
  const onDemandRemaining = getQuotaWindowRemainingPercent(onDemand)
  if (onDemandRemaining != null) {
    const valueText = getQuotaWindowValueText(onDemand)
    parts.push(t('providers.migration.onDemandRemainingee3571', { value: String(formatPercent(onDemandRemaining)), value_2: String(valueText ? ` (${valueText})` : '') }))
  } else if (typeof onDemand?.remaining_value === 'number') {
    parts.push(t('providers.migration.onDemandRemaining91ba64', { value: String(formatQuotaValue(onDemand.remaining_value)) }))
  }

  if (parts.length > 0) return parts.join(' | ')
  return normalizeText(quota.label)
}

function getGrokQuotaText(quota: QuotaStatusSnapshot): string | null {
  const code = normalizeText(quota.code)?.toLowerCase()
  if (code === 'banned') {
    return normalizeText(quota.label) || t('ui.accountBanned210dc8d')
  }
  if (code === 'forbidden') {
    return normalizeText(quota.label) || t('ui.accessRestricted')
  }

  const modelWindows = getQuotaWindowsByScope(quota, 'model')
  const modelParts = modelWindows
    .map((window) => {
      const remainingPercent = getQuotaWindowRemainingPercent(window)
      if (remainingPercent == null) return null
      const valueText = getQuotaWindowValueText(window)
      return t('providers.migration.remaining1ee0b3', { value: String(getGrokQuotaWindowLabel(window)), value_2: String(formatPercent(remainingPercent)), value_3: String(valueText ? ` (${valueText})` : '') })
    })
    .filter((value): value is string => value != null)

  if (modelParts.length > 0) return modelParts.join(' | ')

  const window = getQuotaWindow(quota, 'usage') ?? getQuotaWindowsByScope(quota, 'account')[0] ?? null
  const remainingPercent = getQuotaWindowRemainingPercent(window)
  if (typeof window?.remaining_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0 && window.remaining_value <= 0) {
    return t('providers.migration.remaining3e9a48', { value: String(formatQuotaValue(window.remaining_value)), value_2: String(formatQuotaValue(window.limit_value)) })
  }
  if (remainingPercent != null) {
    const valueText = getQuotaWindowValueText(window)
    if (valueText) {
      return t('providers.migration.remaining2fddcd', { value: String(formatPercent(remainingPercent)), value_2: String(valueText) })
    }
    return t('providers.migration.remaining43f955', { value: String(formatPercent(remainingPercent)) })
  }

  if (typeof window?.remaining_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0) {
    return t('providers.migration.remaining3e9a48', { value: String(formatQuotaValue(window.remaining_value)), value_2: String(formatQuotaValue(window.limit_value)) })
  }

  return normalizeText(quota.label)
}

function getWindsurfQuotaText(quota: QuotaStatusSnapshot): string | null {
  const code = normalizeText(quota.code)?.toLowerCase()
  if (code === 'banned' || code === 'forbidden' || code === 'quarantined') {
    return normalizeText(quota.label) || t('ui.accountUnavailable')
  }
  if (code === 'cooldown' && windsurfCooldownHasPositiveReset(quota)) {
    return normalizeText(quota.label) || t('ui.coolingDown')
  }
  if (code === 'rate_limited' || code === 'rate_limit') {
    return normalizeText(quota.label) || t('ui.rateLimited6d5ac6f')
  }
  if (code === 'exhausted') {
    return normalizeText(quota.label) || t('providers.migration.quotaExhaustedbe24f8')
  }

  const parts: string[] = []
  const dailyRemaining = getQuotaWindowRemainingPercent(getQuotaWindow(quota, 'daily'))
  const weeklyRemaining = getQuotaWindowRemainingPercent(getQuotaWindow(quota, 'weekly'))
  if (dailyRemaining != null) parts.push(t('providers.migration.dailyRemaining0a4f9b', { value: String(formatPercent(dailyRemaining)) }))
  if (weeklyRemaining != null) parts.push(t('providers.migration.weeklyRemaininge92827', { value: String(formatPercent(weeklyRemaining)) }))

  for (const [label, code] of [
    ['Prompt', 'prompt'],
    ['Flex', 'flex'],
  ] as const) {
    const window = getQuotaWindow(quota, code)
    if (!window) continue
    if (typeof window.remaining_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0) {
      parts.push(t('providers.migration.remainingee3ffa', { value: String(label), value_2: String(formatQuotaValue(window.remaining_value)), value_3: String(formatQuotaValue(window.limit_value)) }))
      continue
    }
    if (typeof window.used_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0) {
      parts.push(t('providers.migration.remainingee3ffa', { value: String(label), value_2: String(formatQuotaValue(Math.max(window.limit_value - window.used_value, 0))), value_3: String(formatQuotaValue(window.limit_value)) }))
      continue
    }
    const remainingPercent = getQuotaWindowRemainingPercent(window)
    if (remainingPercent != null) {
      parts.push(t('providers.migration.remaining45c77d', { value: String(label), value_2: String(formatPercent(remainingPercent)) }))
    }
  }

  if (typeof quota.allowed_models_count === 'number') {
    parts.push(t('providers.migration.availableModelsed776b', { value: String(quota.allowed_models_count) }))
  }

  if (parts.length > 0) return parts.join(' | ')

  if (code === 'cooldown') {
    return normalizeText(quota.label) || t('ui.coolingDown')
  }

  return normalizeText(quota.label)
}

function getAntigravityQuotaText(quota: QuotaStatusSnapshot): string | null {
  const code = normalizeText(quota.code)?.toLowerCase()
  if (code === 'forbidden') {
    return normalizeText(quota.label) || t('ui.accessRestricted')
  }

  const remainingList = getQuotaWindowsByScope(quota, 'model')
    .map(getQuotaWindowRemainingPercent)
    .filter((value): value is number => value != null)

  if (remainingList.length === 0) return normalizeText(quota.label)

  const minimumRemaining = Math.min(...remainingList)
  if (remainingList.length === 1) {
    return t('providers.migration.remaining43f955', { value: String(formatPercent(minimumRemaining)) })
  }
  return t('providers.migration.lowestRemainingModelsb37cfc', { value: String(formatPercent(minimumRemaining)), value_2: String(remainingList.length) })
}

function getGeminiCliQuotaText(quota: QuotaStatusSnapshot): string | null {
  const creditsText = getGeminiCliCreditsTextFromQuota(quota)
  const modelWindows = getQuotaWindowsByScope(quota, 'model')
  const activeCoolingModels = modelWindows
    .filter((window) => {
      if (window.is_exhausted === true) return true
      if (typeof window.used_ratio === 'number') return window.used_ratio >= 1.0 - 1e-6
      return false
    })
    .filter((window) => {
      if (typeof window.reset_at !== 'number') return true
      return window.reset_at > Math.floor(Date.now() / 1000)
    })
    .map((window) => normalizeText(window.label) || normalizeText(window.model) || t('ui.model'))

  if (activeCoolingModels.length === 1) {
    return t('providers.migration.coolingDown071d3a', { value: String(activeCoolingModels[0]) })
  }
  if (activeCoolingModels.length > 1) {
    return t('providers.migration.modelsCoolingDown639cf8', { value: String(activeCoolingModels.length) })
  }

  const remainingList = modelWindows
    .map(getQuotaWindowRemainingPercent)
    .filter((value): value is number => value != null)
  if (remainingList.length === 0) return creditsText || normalizeText(quota.label)

  const minimumRemaining = Math.min(...remainingList)
  if (creditsText) return creditsText
  if (remainingList.length === 1) {
    return t('providers.migration.remaining43f955', { value: String(formatPercent(minimumRemaining)) })
  }
  return t('providers.migration.lowestRemainingModelsb37cfc', { value: String(formatPercent(minimumRemaining)), value_2: String(remainingList.length) })
}

function getChatGPTWebQuotaText(quota: QuotaStatusSnapshot): string | null {
  const window = getQuotaWindow(quota, 'image_gen') ?? getQuotaWindowsByScope(quota, 'account')[0] ?? null
  if (!window) return normalizeText(quota.label)

  const remainingPercent = getQuotaWindowRemainingPercent(window)
  if (typeof window.remaining_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0) {
    return t('providers.migration.imagesRemainingfb537f', { value: String(formatQuotaValue(window.remaining_value)), value_2: String(formatQuotaValue(window.limit_value)) })
  }
  if (remainingPercent != null) {
    return t('providers.migration.imagesRemaining41c714', { value: String(formatPercent(remainingPercent)) })
  }

  if (typeof window.remaining_value === 'number') {
    return t('providers.migration.imagesRemaining41c714', { value: String(formatQuotaValue(window.remaining_value)) })
  }

  return normalizeText(quota.label)
}

export function getLegacyAccountQuotaText(
  input: ProviderKeyQuotaCarrier,
): string | null {
  return normalizeText(input.account_quota)
}

export function getQuotaSnapshotFallbackText(
  input: ProviderKeyQuotaCarrier,
  fallbackProviderType?: string | null,
): string | null {
  const quota = getQuotaSnapshot(input)
  if (!quota) return null

  const providerType = getQuotaProviderType(quota, fallbackProviderType)
  switch (providerType) {
    case 'codex':
      return getCodexQuotaText(quota)
    case 'kiro':
      return getKiroQuotaText(quota)
    case 'xai':
      return getXaiQuotaText(quota)
    case 'grok':
      return getGrokQuotaText(quota)
    case 'windsurf':
      return getWindsurfQuotaText(quota)
    case 'antigravity':
      return getAntigravityQuotaText(quota)
    case 'gemini_cli':
      return getGeminiCliQuotaText(quota)
    case 'chatgpt_web':
      return getChatGPTWebQuotaText(quota)
    default:
      return normalizeText(quota.label)
  }
}

export function getQuotaDisplayText(
  input: ProviderKeyQuotaCarrier,
  fallbackProviderType?: string | null,
): string | null {
  return getQuotaSnapshotFallbackText(input, fallbackProviderType) || getLegacyAccountQuotaText(input)
}
