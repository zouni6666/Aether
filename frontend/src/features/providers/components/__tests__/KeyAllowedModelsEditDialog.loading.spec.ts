import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, nextTick, type App } from 'vue'

import KeyAllowedModelsEditDialog from '../KeyAllowedModelsEditDialog.vue'
import type { EndpointAPIKey } from '@/api/endpoints'

const endpointMocks = vi.hoisted(() => ({
  getProviderModels: vi.fn(),
  updateProviderKey: vi.fn(),
}))

vi.mock('@/api/endpoints', () => endpointMocks)
vi.mock('@/composables/useToast', () => ({
  useToast: () => ({
    error: vi.fn(),
    success: vi.fn(),
    warning: vi.fn(),
  }),
}))
vi.mock('@/composables/useConfirm', () => ({
  useConfirm: () => ({
    confirmWarning: vi.fn().mockResolvedValue(true),
  }),
}))
vi.mock('@/features/providers/composables/useUpstreamModelsCache', () => ({
  useUpstreamModelsCache: () => ({
    fetchModels: vi.fn(),
  }),
}))
vi.mock('@/components/ui', async () => {
  const { defineComponent, h } = await import('vue')
  const Dialog = defineComponent({
    name: 'DialogStub',
    setup: (_props, { slots }) => () => h('section', [slots.default?.(), slots.footer?.()]),
  })
  const passthrough = (name: string) => defineComponent({
    name,
    inheritAttrs: false,
    setup: (_props, { attrs, slots }) => () => h('div', attrs, slots.default?.()),
  })
  return {
    Dialog,
    Button: passthrough('ButtonStub'),
    Input: passthrough('InputStub'),
  }
})

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

const apiKey: EndpointAPIKey = {
  id: 'key-1',
  provider_id: 'provider-1',
  api_formats: ['openai:chat'],
  api_key_masked: 'sk-***',
  auth_type: 'api_key',
  name: 'Primary key',
  internal_priority: 0,
  allowed_models: ['gpt-5'],
  cache_ttl_minutes: 0,
  max_probe_interval_minutes: 5,
  health_score: 1,
  consecutive_failures: 0,
  request_count: 0,
  success_count: 0,
  error_count: 0,
  success_rate: 1,
  avg_response_time_ms: 0,
  is_active: true,
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
}

async function settle() {
  for (let index = 0; index < 5; index += 1) {
    await Promise.resolve()
    await nextTick()
  }
}

beforeEach(() => {
  endpointMocks.getProviderModels.mockReset()
  endpointMocks.getProviderModels.mockResolvedValue([{
    provider_model_name: 'gpt-5',
    global_model_name: 'gpt-5',
    global_model_display_name: 'GPT-5',
  }])
  endpointMocks.updateProviderKey.mockReset()
})

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
})

describe('KeyAllowedModelsEditDialog loading', () => {
  it('loads saved permissions when lazily mounted in the open state', async () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const app = createApp(KeyAllowedModelsEditDialog, {
      open: true,
      apiKey,
      providerId: 'provider-1',
    })
    app.mount(root)
    mountedApps.push({ app, root })

    await settle()

    expect(endpointMocks.getProviderModels).toHaveBeenCalledOnce()
    expect(endpointMocks.getProviderModels).toHaveBeenCalledWith('provider-1', { limit: 1000 })
    expect(root.textContent).toContain('提供商模型')
    expect(root.textContent).toContain('已选 1 个')
    expect(root.querySelector('.bg-primary.border-primary')).toBeTruthy()
  })
})