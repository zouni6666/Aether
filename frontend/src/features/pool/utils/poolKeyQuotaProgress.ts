import { translate } from '@/i18n'
import { getCodexResetCountdown } from '@/composables/useCountdownTimer'
import type { MessageKey } from '@/i18n/messages'
import type { PoolKeyDetail } from '@/api/endpoints/pool'
import type { QuotaStatusSnapshot, QuotaWindowSnapshot } from '@/api/endpoints/types'
import { resolveAntigravityQuotaGroupLabel } from '@/features/providers/utils/antigravityQuota'
import { getCodexQuotaWindowPresentation } from '@/features/providers/utils/codexQuotaWindow'
import { getLegacyAccountQuotaText } from '@/features/providers/utils/providerKeyQuota'

export type PoolQuotaTranslator = (key: MessageKey) => string

export interface QuotaProgressItem {
  label: string
  remainingPercent: number
  numericOnly?: boolean
  sortOrder?: number
  detail?: string
  resetAtSeconds?: number | null
  resetSeconds?: number | null
  updatedAtSeconds?: number | null
  allowDynamicReset?: boolean
}

export interface QuotaProgressDisplayItem {
  label: string
  remainingPercent: number
  resetText: string
  meterText: string
  barClass: string
  meterClass: string
  numericOnly?: boolean
}

export function normalizeQuotaLabel(label: string): string {
  // Keep legacy parser categories stable; translate only at the display boundary.
  const normalized = label.trim()
  if (!normalized) return '额度'
  if (/spark\s*5h/i.test(normalized) || normalized.includes('Spark5H')) return 'Spark5H'
  if (/spark/i.test(normalized) && normalized.includes('周')) return 'Spark周'
  if (normalized.includes('5H')) return '5H'
  if (normalized.includes('周')) return '周'
  if (normalized.includes('月')) return '月'
  if (normalized.includes('最低剩余')) return '最低'
  if (normalized === '剩余' || normalized.includes('剩余')) return '剩余'
  if (normalized === '额度') return '额度'
  return normalized
}

export function getQuotaProgressLabel(label: string): string {
  if (label === '日') return translate('ui.day85217f7')
  if (label === '5H') return '5H'
  if (label === '周') return translate('providers.antigravity.quota.period.weekly')
  if (label === '月') return translate('ui.month')
  if (label === '周额度') return translate('providers.antigravity.quota.period.weekly')
  if (label === '月额度') return translate('ui.month')
  if (label === '额度') return translate('poolQuota.claudeCode.unknownWindow')
  if (label === 'Spark5H') return 'Spark5H'
  if (label === 'Spark周') return translate('ui.sparkWeekly')
  if (label === 'Spark月') return translate('ui.sparkMonthly')
  if (label === '生图') return translate('ui.imageGeneration79f2577')
  if (label === '速率') return translate('ui.ratea918a34')
  if (label === '模型') return translate('ui.model')
  if (label === '最低') return translate('ui.lowest')
  if (label === '剩余') return translate('ui.remainingd6822b0')
  return label
}

export function getQuotaProgressCountdown(item: QuotaProgressItem, tick: number) {
  const staticResetLabels = ['日', '5H', '周', '月', '周额度', '月额度', '额度', 'Spark5H', 'Spark周', 'Spark月', 'Auto', 'Fast', 'Expert', 'Heavy', 'Grok 4.3', '生图']
  if (!item.allowDynamicReset && !staticResetLabels.includes(item.label)) return null
  if (item.resetAtSeconds == null && item.resetSeconds == null) return null
  return getCodexResetCountdown(
    item.resetAtSeconds,
    item.resetSeconds,
    item.updatedAtSeconds,
    tick,
    item.remainingPercent
  )
}

export function getQuotaProgressCountdownText(item: QuotaProgressItem, tick: number): string {
  const status = getQuotaProgressCountdown(item, tick)
  if (!status) return ''
  return status.isExpired ? '' : translate('feedback.pool.resetsAfter', { time: status.text })
}

export function formatCompactQuotaCountdownText(text: string): string {
  const normalized = text.trim()
  const dayMatch = normalized.match(/^(\d+)天\s+(.+?)(?:\s+后重置)?$/)
  if (dayMatch) {
    return translate('time.countdownDays', { days: dayMatch[1], time: dayMatch[2] })
  }
  return normalized.replace(/\s+后重置$/, '').replace(/^Resets in\s+/i, '')
}

export function shouldHideQuotaProgressDetailText(text: string | null | undefined): boolean {
  const normalized = (text ?? '').trim()
  return normalized.includes('已重置') || /^reset(?:\b|$)/i.test(normalized)
}

export function getQuotaProgressResetDisplayText(item: QuotaProgressItem, tick: number): string {
  const status = getQuotaProgressCountdown(item, tick)
  return status && !status.isExpired ? status.text : ''
}

export function getQuotaProgressMeterDisplayText(
  item: QuotaProgressItem,
  remainingPercent = item.remainingPercent,
  suppressDetail = false,
): string {
  // 窗口已重置时忽略重置前的旧用量文本，直接显示归一化后的剩余百分比。
  const detail = suppressDetail ? '' : (item.detail?.trim() || '')
  if (!shouldHideQuotaProgressDetailText(detail) && detail) return detail
  return `${remainingPercent.toFixed(1)}%`
}

export function getQuotaLabelOrder(label: string): number {
  if (label === 'Auto') return 0
  if (label === 'Fast') return 1
  if (label === 'Expert') return 2
  if (label === 'Heavy') return 3
  if (label === 'Grok 4.3') return 4
  if (label === '日') return 5
  if (label === '5H') return 6
  if (label === '周') return 7
  if (label === '月') return 8
  if (label === 'Spark5H') return 9
  if (label === 'Spark周') return 10
  if (label === 'Spark月') return 11
  if (label === 'Prompt') return 12
  if (label === 'Flex') return 13
  if (label === '剩余') return 14
  if (label === '额度') return 14
  if (label === '最低') return 15
  if (label === '生图') return 16
  if (label === '速率') return 17
  if (label === '模型') return 18
  return 20
}

export function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 0
  if (value < 0) return 0
  if (value > 100) return 100
  return value
}

export function normalizeUnixSeconds(raw: number | null | undefined): number | null {
  const value = Number(raw ?? 0)
  if (!Number.isFinite(value) || value <= 0) return null
  if (value > 1_000_000_000_000) return Math.floor(value / 1000)
  return Math.floor(value)
}

export function normalizeRemainingSeconds(raw: number | null | undefined): number | null {
  const value = Number(raw ?? NaN)
  if (!Number.isFinite(value) || value < 0) return null
  return Math.floor(value)
}

export function getQuotaSnapshot(key: PoolKeyDetail): QuotaStatusSnapshot | null {
  const quota = key.status_snapshot?.quota
  if (!quota) return null
  return quota
}

export function getQuotaSnapshotProviderType(key: PoolKeyDetail, fallbackProviderType: string): string {
  const snapshotProviderType = String(getQuotaSnapshot(key)?.provider_type || '').trim().toLowerCase()
  if (snapshotProviderType) return snapshotProviderType
  return fallbackProviderType
}

export function getCodexQuotaSnapshot(key: PoolKeyDetail, fallbackProviderType: string): QuotaStatusSnapshot | null {
  const quota = getQuotaSnapshot(key)
  if (!quota) return null
  return getQuotaSnapshotProviderType(key, fallbackProviderType) === 'codex' ? quota : null
}

export function getQuotaSnapshotUpdatedAtSeconds(quota: QuotaStatusSnapshot | null | undefined): number | null {
  return normalizeUnixSeconds(quota?.updated_at ?? quota?.observed_at ?? null)
}

export function getQuotaSnapshotResetAtSeconds(quota: QuotaStatusSnapshot | null | undefined): number | null {
  return normalizeUnixSeconds(quota?.reset_at ?? null)
}

export function getQuotaSnapshotResetSeconds(quota: QuotaStatusSnapshot | null | undefined): number | null {
  return normalizeRemainingSeconds(quota?.reset_seconds ?? null)
}

export function getQuotaSnapshotWindow(
  quota: QuotaStatusSnapshot | null | undefined,
  code: string,
): QuotaWindowSnapshot | null {
  const windows = quota?.windows
  if (!Array.isArray(windows)) return null

  const normalizedCode = code.trim().toLowerCase()
  return windows.find(window => String(window?.code || '').trim().toLowerCase() === normalizedCode) ?? null
}

export function getQuotaSnapshotWindowsByScope(
  quota: QuotaStatusSnapshot | null | undefined,
  scope: string,
): QuotaWindowSnapshot[] {
  const windows = quota?.windows
  if (!Array.isArray(windows)) return []

  const normalizedScope = scope.trim().toLowerCase()
  return windows.filter(window => String(window?.scope || '').trim().toLowerCase() === normalizedScope)
}

export function getQuotaWindowUsedPercent(window: QuotaWindowSnapshot | null | undefined): number | null {
  if (!window) return null
  if (typeof window.used_ratio === 'number') {
    return clampPercent(window.used_ratio * 100)
  }
  if (typeof window.remaining_ratio === 'number') {
    return clampPercent((1 - window.remaining_ratio) * 100)
  }
  if (typeof window.limit_value === 'number' && window.limit_value > 0) {
    if (typeof window.remaining_value === 'number') {
      return clampPercent((1 - (window.remaining_value / window.limit_value)) * 100)
    }
    if (typeof window.used_value === 'number') {
      return clampPercent((window.used_value / window.limit_value) * 100)
    }
  }
  return null
}

export function getQuotaWindowRemainingPercent(window: QuotaWindowSnapshot | null | undefined): number | null {
  if (!window) return null
  if (typeof window.remaining_ratio === 'number') {
    return clampPercent(window.remaining_ratio * 100)
  }
  const usedPercent = getQuotaWindowUsedPercent(window)
  return usedPercent == null ? null : clampPercent(100 - usedPercent)
}

export function formatQuotaValue(value: number | null | undefined): string {
  const normalized = Number(value)
  if (!Number.isFinite(normalized)) return '0'
  const rounded = Math.round(normalized)
  if (Math.abs(normalized - rounded) < 1e-6) {
    return String(rounded)
  }
  return normalized.toFixed(1)
}

export function getQuotaWindowValueText(window: QuotaWindowSnapshot | null | undefined): string | undefined {
  if (!window || typeof window.limit_value !== 'number' || window.limit_value <= 0) return undefined
  if (typeof window.remaining_value === 'number') {
    return `${formatQuotaValue(window.remaining_value)}/${formatQuotaValue(window.limit_value)}`
  }
  if (typeof window.used_value === 'number') {
    return `${formatQuotaValue(Math.max(window.limit_value - window.used_value, 0))}/${formatQuotaValue(window.limit_value)}`
  }
  return undefined
}

export function resolvePoolKeyPlanType(key: PoolKeyDetail): string | null {
  const direct = key.oauth_plan_type?.trim()
  if (direct) return direct
  const quota = getQuotaSnapshot(key)
  const quotaPlan = quota?.plan_type?.trim()
  if (quotaPlan) return quotaPlan
  const quotaPoolTier = quota?.pool_tier?.trim()
  return quotaPoolTier || null
}

export const GROK_QUOTA_MODE_LABELS: Record<string, string> = {
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

export function getGrokQuotaWindowLabel(window: QuotaWindowSnapshot): string {
  const code = String(window.code || '').trim().replace(/^model:/i, '')
  const label = String(window.label || window.model || code).trim()
  const normalized = (label || code).toLowerCase()
  return GROK_QUOTA_MODE_LABELS[normalized] || GROK_QUOTA_MODE_LABELS[code.toLowerCase()] || label || code || translate('ui.mode')
}

export function getGeminiCliQuotaWindowLabel(window: QuotaWindowSnapshot): string {
  const code = String(window.code || '').trim().replace(/^model:/i, '')
  const label = String(window.label || window.model || code).trim()
  return label || code || '模型'
}

function formatRemainingQuotaValue(value: string): string {
  return translate('feedback.pool.remainingValue', { value })
}

function formatQuotaItemCount(count: number): string {
  return translate('feedback.pool.itemCount', { count })
}

export function buildQuotaProgressItemsFromSnapshot(
  key: PoolKeyDetail,
  fallbackProviderType: string,
  translate: PoolQuotaTranslator,
): QuotaProgressItem[] {
  const quota = getQuotaSnapshot(key)
  if (!quota) return []

  const providerType = getQuotaSnapshotProviderType(key, fallbackProviderType)

  if (providerType === 'codex') {
    const quotaResetAtSeconds = getQuotaSnapshotResetAtSeconds(quota)
    const quotaResetSeconds = getQuotaSnapshotResetSeconds(quota)
    return (quota.windows ?? [])
      .map((window): QuotaProgressItem | null => {
        const presentation = getCodexQuotaWindowPresentation(window)
        const remainingPercent = getQuotaWindowRemainingPercent(window)
        if (!presentation || remainingPercent == null) return null
        return {
          label: presentation.label,
          sortOrder: presentation.sortOrder,
          remainingPercent,
          resetAtSeconds: normalizeUnixSeconds(window.reset_at ?? quotaResetAtSeconds ?? null),
          resetSeconds: normalizeRemainingSeconds(window.reset_seconds ?? quotaResetSeconds ?? null),
          updatedAtSeconds: getQuotaSnapshotUpdatedAtSeconds(quota),
          allowDynamicReset: true,
        }
      })
      .filter((item): item is QuotaProgressItem => item != null)
  }

  if (providerType === 'kiro' || providerType === 'xai') {
    const quotaResetAtSeconds = getQuotaSnapshotResetAtSeconds(quota)
    const quotaResetSeconds = getQuotaSnapshotResetSeconds(quota)
    const window = getQuotaSnapshotWindow(quota, 'usage')
      ?? getQuotaSnapshotWindowsByScope(quota, 'account')[0]
      ?? null
    const remainingPercent = getQuotaWindowRemainingPercent(window)
    if (remainingPercent == null) return []

    const detail = typeof window?.used_value === 'number' && typeof window?.limit_value === 'number'
      ? `${formatQuotaValue(window.used_value)}/${formatQuotaValue(window.limit_value)}`
      : undefined

    return [{
      label: normalizeQuotaLabel(String(window?.label || '').trim() || '剩余'),
      remainingPercent,
      detail,
      resetAtSeconds: normalizeUnixSeconds(window?.reset_at ?? quotaResetAtSeconds ?? null),
      resetSeconds: normalizeRemainingSeconds(window?.reset_seconds ?? quotaResetSeconds ?? null),
      updatedAtSeconds: getQuotaSnapshotUpdatedAtSeconds(quota),
      allowDynamicReset: true,
    }]
  }

  if (providerType === 'grok') {
    const quotaResetAtSeconds = getQuotaSnapshotResetAtSeconds(quota)
    const quotaResetSeconds = getQuotaSnapshotResetSeconds(quota)
    const modelWindows = getQuotaSnapshotWindowsByScope(quota, 'model')
    if (modelWindows.length > 0) {
      return modelWindows
        .map((window): QuotaProgressItem | null => {
          const remainingPercent = getQuotaWindowRemainingPercent(window)
          if (remainingPercent == null) return null
          return {
            label: getGrokQuotaWindowLabel(window),
            remainingPercent,
            detail: getQuotaWindowValueText(window),
            resetAtSeconds: normalizeUnixSeconds(window?.reset_at ?? quotaResetAtSeconds ?? null),
            resetSeconds: normalizeRemainingSeconds(window?.reset_seconds ?? quotaResetSeconds ?? null),
            updatedAtSeconds: getQuotaSnapshotUpdatedAtSeconds(quota),
          }
        })
        .filter((item): item is QuotaProgressItem => item != null)
    }

    const window = getQuotaSnapshotWindow(quota, 'usage')
      ?? getQuotaSnapshotWindowsByScope(quota, 'account')[0]
      ?? null
    const remainingPercent = getQuotaWindowRemainingPercent(window)
    if (remainingPercent == null) return []

    return [{
      label: '剩余',
      remainingPercent,
      detail: getQuotaWindowValueText(window),
      resetAtSeconds: normalizeUnixSeconds(window?.reset_at ?? quotaResetAtSeconds ?? null),
      resetSeconds: normalizeRemainingSeconds(window?.reset_seconds ?? quotaResetSeconds ?? null),
      updatedAtSeconds: getQuotaSnapshotUpdatedAtSeconds(quota),
    }]
  }

  if (providerType === 'windsurf') {
    const items: QuotaProgressItem[] = []
    for (const [label, code] of [
      ['日', 'daily'],
      ['周', 'weekly'],
      ['Prompt', 'prompt'],
      ['Flex', 'flex'],
    ] as const) {
      const window = getQuotaSnapshotWindow(quota, code)
      const remainingPercent = getQuotaWindowRemainingPercent(window)
      if (remainingPercent == null) continue
      const detail = typeof window?.used_value === 'number' && typeof window?.limit_value === 'number'
        ? `${formatQuotaValue(window.used_value)}/${formatQuotaValue(window.limit_value)}`
        : typeof window?.remaining_value === 'number' && typeof window?.limit_value === 'number'
          ? formatRemainingQuotaValue(`${formatQuotaValue(window.remaining_value)}/${formatQuotaValue(window.limit_value)}`)
          : undefined
      items.push({
        label,
        remainingPercent,
        detail,
        resetAtSeconds: normalizeUnixSeconds(window?.reset_at ?? null),
        resetSeconds: normalizeRemainingSeconds(window?.reset_seconds ?? null),
        updatedAtSeconds: getQuotaSnapshotUpdatedAtSeconds(quota),
      })
    }

    const rateLimitWindow = getQuotaSnapshotWindow(quota, 'rate_limit')
    if (rateLimitWindow) {
      items.push({
        label: '速率',
        remainingPercent: rateLimitWindow.is_exhausted ? 0 : 100,
        resetAtSeconds: normalizeUnixSeconds(rateLimitWindow.reset_at ?? null),
        resetSeconds: normalizeRemainingSeconds(rateLimitWindow.reset_seconds ?? null),
        updatedAtSeconds: getQuotaSnapshotUpdatedAtSeconds(quota),
      })
    }

    if (typeof quota.allowed_models_count === 'number' && Number.isFinite(quota.allowed_models_count)) {
      items.push({
        label: '模型',
        remainingPercent: 100,
        detail: formatQuotaItemCount(quota.allowed_models_count),
        resetAtSeconds: null,
        resetSeconds: null,
        updatedAtSeconds: getQuotaSnapshotUpdatedAtSeconds(quota),
      })
    }

    return items
  }

  if (providerType === 'antigravity') {
    const windows = getQuotaSnapshotWindowsByScope(quota, 'quota_group')
    if (windows.length === 0) return []
    return windows
      .map((window, index): QuotaProgressItem | null => {
        const remainingPercent = getQuotaWindowRemainingPercent(window)
        if (remainingPercent == null) return null
        const label = resolveAntigravityQuotaGroupLabel(window, translate)
        if (!label) return null
        return {
          label,
          sortOrder: index,
          remainingPercent,
          resetAtSeconds: normalizeUnixSeconds(window.reset_at ?? quota.reset_at ?? null),
          resetSeconds: normalizeRemainingSeconds(window.reset_seconds ?? quota.reset_seconds ?? null),
          updatedAtSeconds: getQuotaSnapshotUpdatedAtSeconds(quota),
          allowDynamicReset: true,
        }
      })
      .filter((item): item is QuotaProgressItem => item != null)
  }

  if (providerType === 'claude_code') {
    const quotaResetAtSeconds = getQuotaSnapshotResetAtSeconds(quota)
    const quotaResetSeconds = getQuotaSnapshotResetSeconds(quota)
    const windowPresentations: Record<string, { labelKey: MessageKey, sortOrder: number }> = {
      '5h': { labelKey: 'poolQuota.claudeCode.window5h', sortOrder: 0 },
      weekly: { labelKey: 'poolQuota.claudeCode.weekly', sortOrder: 1 },
      weekly_sonnet: { labelKey: 'poolQuota.claudeCode.weeklySonnet', sortOrder: 2 },
      weekly_fable: { labelKey: 'poolQuota.claudeCode.weeklyFable', sortOrder: 3 },
    }
    return (quota.windows ?? [])
      .map((window): QuotaProgressItem | null => {
        const remainingPercent = getQuotaWindowRemainingPercent(window)
        if (remainingPercent == null) return null
        const presentation = windowPresentations[String(window.code || '')]
        return {
          label: translate(presentation?.labelKey ?? 'poolQuota.claudeCode.unknownWindow'),
          sortOrder: presentation?.sortOrder ?? 9,
          remainingPercent,
          resetAtSeconds: normalizeUnixSeconds(window.reset_at ?? quotaResetAtSeconds ?? null),
          resetSeconds: normalizeRemainingSeconds(window.reset_seconds ?? quotaResetSeconds ?? null),
          updatedAtSeconds: getQuotaSnapshotUpdatedAtSeconds(quota),
          allowDynamicReset: true,
        }
      })
      .filter((item): item is QuotaProgressItem => item != null)
  }

  if (providerType === 'gemini_cli') {
    const windows = getQuotaSnapshotWindowsByScope(quota, 'model')
    if (windows.length === 0) return []

    const quotaResetAtSeconds = getQuotaSnapshotResetAtSeconds(quota)
    const quotaResetSeconds = getQuotaSnapshotResetSeconds(quota)
    return windows
      .map((window): QuotaProgressItem | null => {
        const remainingPercent = getQuotaWindowRemainingPercent(window)
          ?? (window?.is_exhausted === true ? 0 : null)
        if (remainingPercent == null) return null
        return {
          label: getGeminiCliQuotaWindowLabel(window),
          remainingPercent,
          detail: getQuotaWindowValueText(window),
          resetAtSeconds: normalizeUnixSeconds(window?.reset_at ?? quotaResetAtSeconds ?? null),
          resetSeconds: normalizeRemainingSeconds(window?.reset_seconds ?? quotaResetSeconds ?? null),
          updatedAtSeconds: getQuotaSnapshotUpdatedAtSeconds(quota),
          allowDynamicReset: true,
        }
      })
      .filter((item): item is QuotaProgressItem => item != null)
  }

  if (providerType === 'chatgpt_web') {
    const window = getQuotaSnapshotWindow(quota, 'image_gen')
      ?? getQuotaSnapshotWindowsByScope(quota, 'account')[0]
      ?? null
    const remainingPercent = getQuotaWindowRemainingPercent(window)
    if (remainingPercent == null) return []

    const remainingValue = typeof window?.remaining_value === 'number' ? window.remaining_value : null
    const limitValue = typeof window?.limit_value === 'number' ? window.limit_value : null
    const usedValue = typeof window?.used_value === 'number' ? window.used_value : null
    const detail = remainingValue != null && limitValue != null
      ? `${formatQuotaValue(remainingValue)}/${formatQuotaValue(limitValue)}`
      : usedValue != null && limitValue != null
        ? `${formatQuotaValue(Math.max(limitValue - usedValue, 0))}/${formatQuotaValue(limitValue)}`
        : remainingValue != null
          ? formatRemainingQuotaValue(formatQuotaValue(remainingValue))
          : undefined

    return [{
      label: '生图',
      remainingPercent,
      detail,
      resetAtSeconds: normalizeUnixSeconds(window?.reset_at ?? null),
      resetSeconds: normalizeRemainingSeconds(window?.reset_seconds ?? null),
      updatedAtSeconds: getQuotaSnapshotUpdatedAtSeconds(quota),
    }]
  }

  return []
}

export function resolveCodexQuotaCountdown(
  key: PoolKeyDetail,
  label: string,
  fallbackProviderType: string,
): Pick<QuotaProgressItem, 'resetAtSeconds' | 'resetSeconds' | 'updatedAtSeconds'> | null {
  const codexWindowCodeByLabel: Record<string, string> = {
    '5H': '5h',
    '周': 'weekly',
    Spark5H: 'spark_5h',
    Spark周: 'spark_weekly',
  }
  const windowCode = codexWindowCodeByLabel[label]
  if (!windowCode) return null

  const codexSnapshot = getCodexQuotaSnapshot(key, fallbackProviderType)
  const snapshotWindow = getQuotaSnapshotWindow(codexSnapshot, windowCode)
  if (!snapshotWindow) return null

  const resetAtSeconds = normalizeUnixSeconds(snapshotWindow.reset_at ?? null)
  const resetSeconds = normalizeRemainingSeconds(snapshotWindow.reset_seconds ?? null)
  const updatedAtSeconds = getQuotaSnapshotUpdatedAtSeconds(codexSnapshot)

  if (resetAtSeconds == null && resetSeconds == null) return null
  return { resetAtSeconds, resetSeconds, updatedAtSeconds }
}

export function parseQuotaResetRemainingSeconds(detail: string | undefined): number | null {
  if (!detail) return null
  const text = detail.replace(/\s+/g, '')

  if (text.includes('已重置')) return 0
  if (text.includes('即将重置')) return 1
  if (!text.includes('后重置')) return null

  const dayMatch = text.match(/(\d+)天/)
  const hourMatch = text.match(/(\d+)小时/)
  const minuteMatch = text.match(/(\d+)分钟/)
  const secondMatch = text.match(/(\d+)秒/)

  const days = dayMatch ? Number(dayMatch[1]) : 0
  const hours = hourMatch ? Number(hourMatch[1]) : 0
  const minutes = minuteMatch ? Number(minuteMatch[1]) : 0
  const seconds = secondMatch ? Number(secondMatch[1]) : 0
  const total = days * 86400 + hours * 3600 + minutes * 60 + seconds

  if (total <= 0) return 1
  return total
}

export function parseQuotaProgressItems(
  key: PoolKeyDetail,
  fallbackProviderType: string,
  translate: PoolQuotaTranslator,
): QuotaProgressItem[] {
  const snapshotItems = buildQuotaProgressItemsFromSnapshot(key, fallbackProviderType, translate)
  if (snapshotItems.length > 0) {
    return snapshotItems.sort((a, b) => {
      const orderDiff = (a.sortOrder ?? getQuotaLabelOrder(a.label)) - (b.sortOrder ?? getQuotaLabelOrder(b.label))
      if (orderDiff !== 0) return orderDiff
      return a.label.localeCompare(b.label, 'zh-Hans-CN')
    })
  }

  if (getQuotaSnapshot(key)) return []

  const quotaText = getLegacyAccountQuotaText(key)
  if (!quotaText) return []

  const segments = quotaText
    .split('|')
    .map(s => s.trim())
    .filter(Boolean)

  const items: QuotaProgressItem[] = []
  for (const segment of segments) {
    const match = segment.match(/^(.*?)(-?\d+(?:\.\d+)?)%\s*(.*)$/)
    if (!match) continue

    const [, rawLabel, rawPercent, rawTail] = match
    const remainingPercent = clampPercent(Number(rawPercent))
    const label = normalizeQuotaLabel(rawLabel)
    const detail = rawTail.trim().replace(/^[()]+|[()]+$/g, '').trim()
    const codexCountdown = resolveCodexQuotaCountdown(key, label, fallbackProviderType)
    let resetAtSeconds = codexCountdown?.resetAtSeconds ?? null
    let resetSeconds = codexCountdown?.resetSeconds ?? null
    let updatedAtSeconds = codexCountdown?.updatedAtSeconds ?? null

    if (resetAtSeconds == null && resetSeconds == null) {
      const resetRemainingSeconds = parseQuotaResetRemainingSeconds(detail || undefined)
      resetAtSeconds = resetRemainingSeconds == null
        ? null
        : Math.floor(Date.now() / 1000) + resetRemainingSeconds
      resetSeconds = null
      updatedAtSeconds = null
    }

    items.push({
      label,
      remainingPercent,
      detail: detail || undefined,
      resetAtSeconds,
      resetSeconds,
      updatedAtSeconds,
    })
  }

  return items.sort((a, b) => {
    const orderDiff = getQuotaLabelOrder(a.label) - getQuotaLabelOrder(b.label)
    if (orderDiff !== 0) return orderDiff
    return a.label.localeCompare(b.label, 'zh-Hans-CN')
  })
}

export function getQuotaRemainingClassByRemaining(remaining: number): string {
  if (remaining <= 10) return 'text-red-600 dark:text-red-400'
  if (remaining <= 30) return 'text-yellow-600 dark:text-yellow-400'
  return 'text-green-600 dark:text-green-400'
}

export function getQuotaRemainingBarColorByRemaining(remaining: number): string {
  if (remaining <= 10) return 'bg-red-500 dark:bg-red-400'
  if (remaining <= 30) return 'bg-yellow-500 dark:bg-yellow-400'
  return 'bg-green-500 dark:bg-green-400'
}
