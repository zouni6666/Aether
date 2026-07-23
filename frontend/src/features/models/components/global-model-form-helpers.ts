import type { GlobalModelCreate, GlobalModelResponse, GlobalModelUpdate } from '@/api/global-models'
import type { TieredPricingConfig } from '@/api/endpoints/types'
import type { ModelsDevModelItem } from '@/api/models-dev'

export const EMBEDDING_API_FORMATS = [
  'openai:embedding',
  'gemini:embedding',
  'jina:embedding',
  'doubao:embedding',
  'aliyun:multimodal_embedding',
] as const

export const RERANK_API_FORMATS = [
  'openai:rerank',
  'jina:rerank',
] as const

export interface GlobalModelFormPayloadState {
  name: string
  display_name: string
  default_price_per_request?: number
  supported_capabilities?: string[]
  config?: Record<string, unknown>
  is_active?: boolean
}

export function cloneTieredPricingConfig(
  pricing: TieredPricingConfig,
): TieredPricingConfig {
  // Presets live inside a Vue ref and can therefore be reactive proxies, which
  // structuredClone cannot clone directly. Pricing configs are JSON payloads.
  return JSON.parse(JSON.stringify(pricing)) as TieredPricingConfig
}

export function findGlobalModelByName<T extends { name: string }>(
  models: T[],
  modelName: string,
): T | undefined {
  const normalizedName = modelName.trim().toLowerCase()
  return models.find(model => model.name.trim().toLowerCase() === normalizedName)
}

function normalizeJsonValue(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(normalizeJsonValue)
  }
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .filter(([, entryValue]) => entryValue !== undefined)
        .sort(([leftKey], [rightKey]) => leftKey.localeCompare(rightKey))
        .map(([key, entryValue]) => [key, normalizeJsonValue(entryValue)]),
    )
  }
  return value
}

export function tieredPricingConfigsEqual(
  currentPricing: TieredPricingConfig | null | undefined,
  onlinePricing: TieredPricingConfig | null | undefined,
): boolean {
  return JSON.stringify(normalizeJsonValue(currentPricing ?? null))
    === JSON.stringify(normalizeJsonValue(onlinePricing ?? null))
}

export interface GlobalModelPriceSyncEntry {
  model: GlobalModelResponse
  onlineModel: ModelsDevModelItem
}

export interface GlobalModelPriceSyncPlan {
  syncable: GlobalModelPriceSyncEntry[]
  unchanged: GlobalModelPriceSyncEntry[]
  unavailable: GlobalModelResponse[]
}

export interface ModelsDevPricingPreference {
  enabled: true
  provider_id: string
  provider_name: string
}

const MODELS_DEV_PRICING_CONFIG_KEY = 'models_dev_pricing'

export function readModelsDevPricingPreference(
  config: Record<string, unknown> | null | undefined,
): ModelsDevPricingPreference | null {
  const value = config?.[MODELS_DEV_PRICING_CONFIG_KEY]
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  const preference = value as Record<string, unknown>
  if (preference.enabled !== true) return null
  if (typeof preference.provider_id !== 'string' || !preference.provider_id.trim()) return null
  return {
    enabled: true,
    provider_id: preference.provider_id.trim(),
    provider_name: typeof preference.provider_name === 'string' && preference.provider_name.trim()
      ? preference.provider_name.trim()
      : preference.provider_id.trim(),
  }
}

export function mergeModelsDevPricingPreference(
  config: Record<string, unknown> | null | undefined,
  preference: ModelsDevPricingPreference | null,
): Record<string, unknown> | null {
  const mergedConfig = { ...(config || {}) }
  if (preference) {
    mergedConfig[MODELS_DEV_PRICING_CONFIG_KEY] = preference
  } else {
    delete mergedConfig[MODELS_DEV_PRICING_CONFIG_KEY]
  }
  return Object.keys(mergedConfig).length > 0 ? mergedConfig : null
}

export function buildGlobalModelPriceSyncPlan(
  models: GlobalModelResponse[],
  onlineModels: ModelsDevModelItem[],
): GlobalModelPriceSyncPlan {
  const onlineModelsByName = new Map<string, ModelsDevModelItem>()
  for (const onlineModel of onlineModels) {
    const normalizedName = onlineModel.modelId.trim().toLowerCase()
    if (!onlineModelsByName.has(normalizedName)) {
      onlineModelsByName.set(normalizedName, onlineModel)
    }
  }

  const plan: GlobalModelPriceSyncPlan = {
    syncable: [],
    unchanged: [],
    unavailable: [],
  }
  for (const model of models) {
    const onlineModel = onlineModelsByName.get(model.name.trim().toLowerCase())
    if (!onlineModel?.tieredPricing) {
      plan.unavailable.push(model)
    } else if (tieredPricingConfigsEqual(model.default_tiered_pricing, onlineModel.tieredPricing)) {
      plan.unchanged.push({ model, onlineModel })
    } else {
      plan.syncable.push({ model, onlineModel })
    }
  }
  return plan
}

function cleanGlobalModelConfig(form: GlobalModelFormPayloadState): Record<string, unknown> | undefined {
  return form.config && Object.keys(form.config).length > 0 ? form.config : undefined
}

export function buildGlobalModelCreatePayload(
  form: GlobalModelFormPayloadState,
  defaultTieredPricing: TieredPricingConfig,
): GlobalModelCreate {
  return {
    name: form.name ?? '',
    display_name: form.display_name ?? '',
    config: cleanGlobalModelConfig(form),
    default_price_per_request: form.default_price_per_request ?? undefined,
    default_tiered_pricing: defaultTieredPricing,
    supported_capabilities: form.supported_capabilities?.length ? form.supported_capabilities : undefined,
    is_active: form.is_active,
  }
}

export function buildGlobalModelUpdatePayload(
  form: GlobalModelFormPayloadState,
  defaultTieredPricing: TieredPricingConfig,
): GlobalModelUpdate {
  return {
    display_name: form.display_name,
    config: cleanGlobalModelConfig(form) || null,
    default_price_per_request: form.default_price_per_request ?? null,
    default_tiered_pricing: defaultTieredPricing,
    supported_capabilities: form.supported_capabilities?.length ? form.supported_capabilities : null,
    is_active: form.is_active,
  }
}
