import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, nextTick, type App } from 'vue'
import type { ProviderWithEndpointsSummary } from '@/api/endpoints/types'
import ProviderFormDialog from '../ProviderFormDialog.vue'

const api = vi.hoisted(() => ({ createProvider: vi.fn(), updateProvider: vi.fn() }))
vi.mock('@/api/endpoints', () => ({ ...api, normalizePoolAdvancedConfig: () => null }))
vi.mock('@/composables/useToast', () => ({ useToast: () => ({ success: vi.fn(), error: vi.fn() }) }))

let app: App | null = null
let root: HTMLElement | null = null

async function settle() {
  for (let index = 0; index < 4; index += 1) { await Promise.resolve(); await nextTick() }
}

async function mountDialog(props: { routingGroupId?: string; routingGroupName?: string; provider?: ProviderWithEndpointsSummary } = {}) {
  root = document.createElement('div')
  document.body.appendChild(root)
  app = createApp(ProviderFormDialog, { modelValue: true, ...props })
  app.mount(root)
  await settle()
}

async function submit(text: string) {
  const button = [...document.body.querySelectorAll<HTMLButtonElement>('button')].find(item => item.textContent?.trim() === text)
  expect(button).toBeDefined()
  button!.click()
  await settle()
}

async function nameProvider() {
  const input = document.body.querySelector<HTMLInputElement>('#name')!
  input.value = 'New Provider'
  input.dispatchEvent(new Event('input', { bubbles: true }))
  await nextTick()
}

beforeEach(() => {
  vi.clearAllMocks()
  api.createProvider.mockResolvedValue({ id: 'new-provider' })
  api.updateProvider.mockResolvedValue({ id: 'existing-provider' })
})

afterEach(() => { app?.unmount(); root?.remove(); document.body.innerHTML = ''; app = null; root = null })

describe('ProviderFormDialog routing group assignment', () => {
  it('explains the creation scope and submits the selected routing group', async () => {
    await mountDialog({ routingGroupId: 'group-b', routingGroupName: '备用策略' })
    expect(document.body.textContent).toContain('备用策略')
    expect(document.body.textContent).toContain('其他策略分组中默认禁用')
    await nameProvider()
    await submit('创建')
    expect(api.createProvider).toHaveBeenCalledWith(expect.objectContaining({ name: 'New Provider', routing_group_id: 'group-b' }))
    expect(api.updateProvider).not.toHaveBeenCalled()
  })

  it('keeps creation without a group compatible with the existing API', async () => {
    await mountDialog()
    expect(document.body.textContent).not.toContain('其他策略分组中默认禁用')
    await nameProvider()
    await submit('创建')
    expect(api.createProvider).toHaveBeenCalledOnce()
    expect(api.createProvider.mock.calls[0]![0].routing_group_id).toBeUndefined()
  })

  it('does not change group membership when editing an existing provider', async () => {
    await mountDialog({
      routingGroupId: 'group-b', routingGroupName: '备用策略',
      provider: { id: 'existing-provider', name: 'Existing', provider_type: 'custom', provider_priority: 10, is_active: true } as ProviderWithEndpointsSummary,
    })
    expect(document.body.textContent).not.toContain('其他策略分组中默认禁用')
    await submit('保存')
    expect(api.updateProvider).toHaveBeenCalledWith('existing-provider', expect.objectContaining({ name: 'Existing' }))
    expect(api.updateProvider.mock.calls[0]![1]).not.toHaveProperty('routing_group_id')
    expect(api.createProvider).not.toHaveBeenCalled()
  })
})
