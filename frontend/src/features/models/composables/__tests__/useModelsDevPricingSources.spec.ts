import { beforeEach, describe, expect, it } from 'vitest'

import { useModelsDevPricingSources } from '../useModelsDevPricingSources'

const STORAGE_KEY = 'aether:models-dev-pricing-sources:v1'
const LEGACY_STORAGE_KEY = 'aether:models-dev-pricing-preferences:v1'

describe('useModelsDevPricingSources', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  it('stores the provider used by a manual pricing action', () => {
    const { getSource, setSource } = useModelsDevPricingSources()

    setSource('model-1', {
      provider_id: 'openai',
      provider_name: 'OpenAI',
    })

    expect(getSource('model-1')).toEqual({
      provider_id: 'openai',
      provider_name: 'OpenAI',
    })
    expect(JSON.parse(localStorage.getItem(STORAGE_KEY) || 'null')).toEqual({
      version: 1,
      models: {
        'model-1': {
          provider_id: 'openai',
          provider_name: 'OpenAI',
        },
      },
    })
  })

  it('migrates the previous provider record without retaining its automatic preference key', () => {
    localStorage.setItem(LEGACY_STORAGE_KEY, JSON.stringify({
      version: 1,
      models: {
        'model-1': {
          provider_id: 'anthropic',
          provider_name: 'Anthropic',
        },
      },
    }))

    const { getSource } = useModelsDevPricingSources()

    expect(getSource('model-1')).toEqual({
      provider_id: 'anthropic',
      provider_name: 'Anthropic',
    })
    expect(localStorage.getItem(LEGACY_STORAGE_KEY)).toBeNull()
    expect(localStorage.getItem(STORAGE_KEY)).not.toBeNull()
  })

  it.each([
    '{broken',
    JSON.stringify({ version: 2, models: {} }),
    JSON.stringify({ version: 1, models: [] }),
  ])('ignores incompatible or malformed source documents', (stored) => {
    localStorage.setItem(STORAGE_KEY, stored)

    const { getSource } = useModelsDevPricingSources()

    expect(getSource('model-1')).toBeNull()
  })
})