import { ref } from 'vue'

export interface ModelsDevPricingSource {
  provider_id: string
  provider_name: string
}

interface StoredModelsDevPricingSources {
  version: 1
  models: Record<string, ModelsDevPricingSource>
}

const STORAGE_KEY = 'aether:models-dev-pricing-sources:v1'
const LEGACY_STORAGE_KEY = 'aether:models-dev-pricing-preferences:v1'
const sources = ref<Record<string, ModelsDevPricingSource>>({})

function parseStoredSources(key: string): Record<string, ModelsDevPricingSource> | null {
  try {
    const stored = JSON.parse(localStorage.getItem(key) || 'null') as unknown
    if (!stored || typeof stored !== 'object') return null
    const document = stored as Partial<StoredModelsDevPricingSources>
    if (document.version !== 1 || !document.models || typeof document.models !== 'object') return null

    const validSources: Record<string, ModelsDevPricingSource> = {}
    for (const [modelId, value] of Object.entries(document.models)) {
      if (!value || typeof value !== 'object') continue
      const source = value as Partial<ModelsDevPricingSource>
      if (
        typeof source.provider_id === 'string'
        && source.provider_id.length > 0
        && typeof source.provider_name === 'string'
        && source.provider_name.length > 0
      ) {
        validSources[modelId] = {
          provider_id: source.provider_id,
          provider_name: source.provider_name,
        }
      }
    }
    return validSources
  } catch {
    return null
  }
}

function writeStoredSources(value: Record<string, ModelsDevPricingSource>): boolean {
  try {
    const document: StoredModelsDevPricingSources = {
      version: 1,
      models: value,
    }
    localStorage.setItem(STORAGE_KEY, JSON.stringify(document))
    return true
  } catch {
    return false
  }
}

function readStoredSources(): Record<string, ModelsDevPricingSource> {
  if (typeof localStorage === 'undefined') return {}

  const currentSources = parseStoredSources(STORAGE_KEY)
  if (currentSources) {
    localStorage.removeItem(LEGACY_STORAGE_KEY)
    return currentSources
  }

  const legacySources = parseStoredSources(LEGACY_STORAGE_KEY)
  if (legacySources && writeStoredSources(legacySources)) {
    localStorage.removeItem(LEGACY_STORAGE_KEY)
  }
  return legacySources ?? {}
}

export function useModelsDevPricingSources() {
  sources.value = readStoredSources()

  function getSource(modelId: string): ModelsDevPricingSource | null {
    return sources.value[modelId] ?? null
  }

  function setSource(modelId: string, source: ModelsDevPricingSource) {
    const nextSources = {
      ...sources.value,
      [modelId]: source,
    }
    sources.value = nextSources
    writeStoredSources(nextSources)
  }

  return {
    getSource,
    setSource,
  }
}