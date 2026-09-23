import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, nextTick, ref, type App } from 'vue'

import type { PoolAdvancedConfig } from '@/api/endpoints/types/provider'
import PoolSchedulingDialog from '../PoolSchedulingDialog.vue'

const endpointMocks = vi.hoisted(() => ({
  getPoolSchedulingPresets: vi.fn(),
  getProvider: vi.fn(),
  updateProvider: vi.fn(),
}))

vi.mock('@/api/endpoints', () => ({
  getProvider: endpointMocks.getProvider,
  updateProvider: endpointMocks.updateProvider,
}))

vi.mock('@/api/endpoints/pool', () => ({
  getPoolSchedulingPresets: endpointMocks.getPoolSchedulingPresets,
}))

vi.mock('@/composables/useToast', () => ({
  useToast: () => ({ success: vi.fn(), error: vi.fn() }),
}))

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

async function settle(): Promise<void> {
  for (let index = 0; index < 4; index += 1) {
    await Promise.resolve()
    await nextTick()
  }
}

function mountDialog(providerType = 'codex', currentConfig: PoolAdvancedConfig | null = null) {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const open = ref(false)
  const TestHost = defineComponent({
    setup() {
      void nextTick(() => { open.value = true })
      return () => h(PoolSchedulingDialog, {
        modelValue: open.value,
        providerId: 'provider-1',
        providerType,
        currentConfig,
        'onUpdate:modelValue': (value: boolean) => { open.value = value },
      })
    },
  })
  const app = createApp(TestHost)
  app.mount(root)
  mountedApps.push({ app, root })
  return open
}

function quotaSwitch() {
  return document.body.querySelector<HTMLButtonElement>('#pool-reserve-minimum-quota')
}

async function saveDialog() {
  const saveButton = [...document.body.querySelectorAll<HTMLButtonElement>('button')]
    .find(button => button.textContent?.trim() === '保存')
  expect(saveButton).toBeDefined()
  saveButton?.click()
  await settle()
}

beforeEach(() => {
  endpointMocks.getPoolSchedulingPresets.mockReset()
  endpointMocks.getProvider.mockReset()
  endpointMocks.updateProvider.mockReset()
  endpointMocks.getPoolSchedulingPresets.mockResolvedValue([{
    name: 'lru',
    label: 'LRU 轮转',
    description: '最久未使用的 Key 优先',
    providers: [],
    default_enabled: true,
    modes: null,
    default_mode: null,
    mutex_group: 'distribution_mode',
  }])
  endpointMocks.getProvider.mockResolvedValue({ id: 'provider-1', pool_advanced: {} })
  endpointMocks.updateProvider.mockResolvedValue({ id: 'provider-1' })
})

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
  document.body.innerHTML = ''
})

describe('PoolSchedulingDialog minimum quota reserve', () => {
  it('defaults to disabled and saves enabling it while preserving the latest settings', async () => {
    endpointMocks.getProvider.mockResolvedValue({
      id: 'provider-1',
      pool_advanced: { rate_limit_cooldown_seconds: 900, skip_exhausted_accounts: false },
    })
    mountDialog('codex', { rate_limit_cooldown_seconds: 300 })
    await settle()

    expect(quotaSwitch()?.getAttribute('aria-checked')).toBe('false')
    expect(document.body.textContent).toContain('剩余额度不高于 1%')
    quotaSwitch()?.click()
    await nextTick()
    await saveDialog()

    expect(endpointMocks.updateProvider).toHaveBeenCalledWith('provider-1', {
      pool_advanced: {
        rate_limit_cooldown_seconds: 900,
        skip_exhausted_accounts: false,
        reserve_minimum_quota: true,
        scheduling_presets: [{ preset: 'lru', enabled: true }],
      },
    })
  })

  it('loads an enabled reserve and saves disabling it', async () => {
    mountDialog('codex', { reserve_minimum_quota: true })
    await settle()

    expect(quotaSwitch()?.getAttribute('aria-checked')).toBe('true')
    quotaSwitch()?.click()
    await nextTick()
    await saveDialog()

    expect(endpointMocks.updateProvider).toHaveBeenCalledWith('provider-1', {
      pool_advanced: {
        reserve_minimum_quota: false,
        scheduling_presets: [{ preset: 'lru', enabled: true }],
      },
    })
  })

  it('discards unsaved reserve changes when the dialog is reopened', async () => {
    const open = mountDialog()
    await settle()
    quotaSwitch()?.click()
    await nextTick()
    expect(quotaSwitch()?.getAttribute('aria-checked')).toBe('true')

    open.value = false
    await settle()
    open.value = true
    await settle()

    expect(quotaSwitch()?.getAttribute('aria-checked')).toBe('false')
    expect(endpointMocks.updateProvider).not.toHaveBeenCalled()
  })

  it.each(['openai', 'kiro'])('does not expose or add the reserve setting for %s', async (providerType) => {
    mountDialog(providerType)
    await settle()

    expect(quotaSwitch()).toBeNull()
    await saveDialog()

    expect(endpointMocks.updateProvider).toHaveBeenCalledWith('provider-1', {
      pool_advanced: {
        scheduling_presets: [{ preset: 'lru', enabled: true }],
      },
    })
  })
})
