import { translate } from '@/i18n'
import type { PoolKeyBatchUpdatePatch } from '@/api/endpoints/pool'

export interface PoolKeyBatchEditState {
  applyApiFormats: boolean
  apiFormats: string[]
  applyActive: boolean
  isActive: boolean
  applyInternalPriority: boolean
  internalPriority: string
  applyRpmLimit: boolean
  rpmLimit: string
  applyConcurrentLimit: boolean
  concurrentLimit: string
  applyCacheTtl: boolean
  cacheTtlMinutes: string
  applyProbeInterval: boolean
  maxProbeIntervalMinutes: string
  applyNote: boolean
  note: string
  applyAutoFetchModels: boolean
  autoFetchModels: boolean
  includePatterns: string
  excludePatterns: string
  applyAllowedModels: boolean
  unrestrictedModels: boolean
  selectedModels: string[]
  lockSelectedModels: boolean
}

export interface PoolKeyBatchPatchBuildResult {
  patch: PoolKeyBatchUpdatePatch | null
  fieldLabels: string[]
  error: string | null
  errorField?: 'allowed_models'
}

function uniqueTrimmed(values: string[]): string[] {
  return [...new Set(values.map(value => value.trim()).filter(Boolean))]
}

export function parsePoolKeyModelPatterns(value: string): string[] {
  return uniqueTrimmed(value.split(/[,\n]/))
}

function parseIntegerField(
  value: string,
  label: string,
  min: number,
  max?: number,
  nullable = false,
): { value?: number | null; error?: string } {
  const normalized = value.trim()
  if (!normalized) {
    return nullable ? { value: null } : { error: translate('feedback.pool.requiredField', { label }) }
  }
  const parsed = Number(normalized)
  if (!Number.isInteger(parsed) || parsed < min || (max !== undefined && parsed > max)) {
    const range = max === undefined ? translate('feedback.pool.minimumValue', { min }) : `${min}-${max}`
    return { error: translate('feedback.pool.integerRange', { label, range }) }
  }
  return { value: parsed }
}

export function buildPoolKeyBatchUpdatePatch(
  state: PoolKeyBatchEditState,
): PoolKeyBatchPatchBuildResult {
  const patch: PoolKeyBatchUpdatePatch = {}
  const fieldLabels: string[] = []

  if (state.applyApiFormats) {
    const apiFormats = uniqueTrimmed(state.apiFormats)
    if (apiFormats.length === 0) {
      return { patch: null, fieldLabels, error: translate('ui.selectAtLeastOneSupportedApi') }
    }
    patch.api_formats = apiFormats
    fieldLabels.push(translate('ui.supportedApisdd7863a'))
  }

  if (state.applyActive) {
    patch.is_active = state.isActive
    fieldLabels.push(translate('ui.enablementStatus'))
  }

  if (state.applyInternalPriority) {
    const parsed = parseIntegerField(state.internalPriority, translate('ui.priority565d646'), 0)
    if (parsed.error) return { patch: null, fieldLabels, error: parsed.error }
    patch.internal_priority = parsed.value as number
    fieldLabels.push(translate('ui.priority565d646'))
  }

  if (state.applyRpmLimit) {
    const parsed = parseIntegerField(state.rpmLimit, translate('ui.rpmLimit50b7143'), 1, 10000, true)
    if (parsed.error) return { patch: null, fieldLabels, error: parsed.error }
    patch.rpm_limit = parsed.value
    fieldLabels.push(translate('ui.rpmLimit50b7143'))
  }

  if (state.applyConcurrentLimit) {
    const parsed = parseIntegerField(state.concurrentLimit, translate('ui.concurrentRequestLimit'), 0, undefined, true)
    if (parsed.error) return { patch: null, fieldLabels, error: parsed.error }
    patch.concurrent_limit = parsed.value
    fieldLabels.push(translate('ui.concurrentRequestLimit'))
  }

  if (state.applyCacheTtl) {
    const parsed = parseIntegerField(state.cacheTtlMinutes, translate('ui.cacheTtld8a1617'), 0, 60)
    if (parsed.error) return { patch: null, fieldLabels, error: parsed.error }
    patch.cache_ttl_minutes = parsed.value as number
    fieldLabels.push(translate('ui.cacheTtld8a1617'))
  }

  if (state.applyProbeInterval) {
    const parsed = parseIntegerField(state.maxProbeIntervalMinutes, translate('ui.circuitProbe'), 0, 32)
    if (parsed.error) return { patch: null, fieldLabels, error: parsed.error }
    patch.max_probe_interval_minutes = parsed.value as number
    fieldLabels.push(translate('ui.circuitProbe'))
  }

  if (state.applyNote) {
    patch.note = state.note.trim() || null
    fieldLabels.push(translate('ui.notes'))
  }

  if (state.applyAutoFetchModels) {
    patch.auto_fetch_models = state.autoFetchModels
    if (state.autoFetchModels) {
      patch.model_include_patterns = parsePoolKeyModelPatterns(state.includePatterns)
      patch.model_exclude_patterns = parsePoolKeyModelPatterns(state.excludePatterns)
    }
    fieldLabels.push(translate('ui.autoFetchUpstreamModels'))
  }

  if (state.applyAllowedModels) {
    const selectedModels = uniqueTrimmed(state.selectedModels)
    if (!state.unrestrictedModels && selectedModels.length === 0) {
      return { patch: null, fieldLabels, error: translate('ui.selectAtLeastOneAvailableModel'), errorField: 'allowed_models' }
    }
    patch.allowed_models = state.unrestrictedModels ? null : selectedModels
    patch.locked_models = state.unrestrictedModels || !state.lockSelectedModels
      ? []
      : selectedModels
    fieldLabels.push(translate('ui.allowedModels'))
  }

  if (fieldLabels.length === 0) {
    return { patch: null, fieldLabels, error: translate('ui.enableAtLeastOneFieldForBulkEditing') }
  }

  return { patch, fieldLabels, error: null }
}
