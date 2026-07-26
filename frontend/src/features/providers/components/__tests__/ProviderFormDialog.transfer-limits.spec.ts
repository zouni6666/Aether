import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, nextTick, type App } from 'vue'

import type { ProviderWithEndpointsSummary } from '@/api/endpoints/types'
import ProviderFormDialog from '../ProviderFormDialog.vue'

const endpointMocks = vi.hoisted(() => ({
  createProvider: vi.fn(),
  updateProvider: vi.fn(),
}))

vi.mock('@/api/endpoints', () => ({
  createProvider: endpointMocks.createProvider,
  updateProvider: endpointMocks.updateProvider,
  normalizePoolAdvancedConfig: (value: unknown) => {
    if (value == null || value === false) return null
    if (value === true) return {}
    if (typeof value !== 'object' || Array.isArray(value)) return null
    return { ...value }
  },
}))

vi.mock('@/composables/useToast', () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
  }),
}))

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

function makeProvider(
  overrides: Partial<ProviderWithEndpointsSummary> = {},
): ProviderWithEndpointsSummary {
  return {
    id: 'provider-1',
    name: 'Provider One',
    provider_type: 'custom',
    provider_priority: 100,
    keep_priority_on_conversion: false,
    enable_format_conversion: true,
    max_transfer_count: 0,
    max_transfer_timeout_seconds: 0,
    is_active: true,
    total_endpoints: 0,
    active_endpoints: 0,
    total_keys: 0,
    active_keys: 0,
    total_models: 0,
    active_models: 0,
    global_model_ids: [],
    avg_health_score: 1,
    unhealthy_endpoints: 0,
    api_formats: [],
    endpoint_health_details: [],
    ops_configured: false,
    created_at: '2026-07-26T00:00:00Z',
    updated_at: '2026-07-26T00:00:00Z',
    ...overrides,
  }
}

function mountDialog(provider?: ProviderWithEndpointsSummary | null) {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp(ProviderFormDialog, {
    modelValue: true,
    provider,
    'onUpdate:modelValue': vi.fn(),
  })
  app.mount(root)
  mountedApps.push({ app, root })
}

async function settle() {
  for (let index = 0; index < 4; index += 1) {
    await Promise.resolve()
    await nextTick()
  }
}

async function setInput(selector: string, value: string) {
  const input = document.body.querySelector<HTMLInputElement>(selector)
  if (!input) throw new Error(`Missing input: ${selector}`)
  input.value = value
  input.dispatchEvent(new Event('input', { bubbles: true }))
  await nextTick()
}

function clickButton(text: string) {
  const button = [...document.body.querySelectorAll<HTMLButtonElement>('button')]
    .find(candidate => candidate.textContent?.trim() === text)
  if (!button) throw new Error(`Missing button: ${text}`)
  button.click()
}

beforeEach(() => {
  endpointMocks.createProvider.mockReset()
  endpointMocks.createProvider.mockResolvedValue({ id: 'provider-new', name: 'New Provider' })
  endpointMocks.updateProvider.mockReset()
  endpointMocks.updateProvider.mockResolvedValue(makeProvider())
})

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
  document.body.innerHTML = ''
})

describe('ProviderFormDialog transfer limits', () => {
  it('loads and submits configured limits in edit mode', async () => {
    mountDialog(makeProvider({
      max_transfer_count: 10,
      max_transfer_timeout_seconds: 60,
    }))
    await settle()

    expect(document.body.querySelector<HTMLInputElement>('#max-transfer-count')?.value).toBe('10')
    expect(document.body.querySelector<HTMLInputElement>('#max-transfer-timeout-seconds')?.value).toBe('60')

    await setInput('#max-transfer-count', '12')
    await setInput('#max-transfer-timeout-seconds', '45')
    clickButton('保存')
    await settle()

    expect(endpointMocks.updateProvider).toHaveBeenCalledWith(
      'provider-1',
      expect.objectContaining({
        max_transfer_count: 12,
        max_transfer_timeout_seconds: 45,
      }),
    )
  })

  it('defaults missing legacy values to explicit zero', async () => {
    mountDialog(makeProvider({
      max_transfer_count: undefined,
      max_transfer_timeout_seconds: undefined,
    }))
    await settle()

    expect(document.body.querySelector<HTMLInputElement>('#max-transfer-count')?.value).toBe('0')
    expect(document.body.querySelector<HTMLInputElement>('#max-transfer-timeout-seconds')?.value).toBe('0')

    clickButton('保存')
    await settle()

    expect(endpointMocks.updateProvider).toHaveBeenCalledWith(
      'provider-1',
      expect.objectContaining({
        max_transfer_count: 0,
        max_transfer_timeout_seconds: 0,
      }),
    )
  })

  it('hides the controls when creating while still sending zero defaults', async () => {
    mountDialog(null)
    await settle()

    expect(document.body.querySelector('#max-transfer-count')).toBeNull()
    expect(document.body.querySelector('#max-transfer-timeout-seconds')).toBeNull()

    await setInput('#name', 'New Provider')
    clickButton('创建')
    await settle()

    expect(endpointMocks.createProvider).toHaveBeenCalledWith(
      expect.objectContaining({
        max_transfer_count: 0,
        max_transfer_timeout_seconds: 0,
      }),
    )
  })
})
