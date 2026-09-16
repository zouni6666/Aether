import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, h, nextTick, ref, type App } from 'vue'
import { getProvidersSummary, type ProviderWithEndpointsSummary } from '@/api/endpoints'
import type { GlobalModelResponse } from '@/api/global-models'
import RoutingSchedulingPolicyEditor from '../components/RoutingSchedulingPolicyEditor.vue'
import {
  createEmptyRoutingGroupConfig,
  getModelPolicy,
  type RoutingGroupConfig,
} from '../utils/routingPolicy'
import { createSchedulingPolicy, writeSchedulingPolicies } from '../utils/schedulingPolicies'

vi.mock('@/api/endpoints', () => ({ getProvidersSummary: vi.fn() }))

const globalModels = ['a', 'b', 'c'].map(name => ({
  id: `id-${name}`, name: `model-${name}`, display_name: `模型 ${name.toUpperCase()}`,
})) as GlobalModelResponse[]
const providerSources = [
  { id: 'provider-a', name: '提供商 A', global_model_ids: ['id-a'] },
  { id: 'provider-b', name: '提供商 B', global_model_ids: ['id-b'] },
  { id: 'provider-shared', name: '共享提供商', global_model_ids: ['id-a', 'id-b'] },
  { id: 'provider-other', name: '无关提供商', global_model_ids: ['id-other'] },
].map((provider, index) => ({
  ...provider,
  provider_priority: index,
  is_active: true,
  api_formats: ['openai:chat'],
})) as ProviderWithEndpointsSummary[]
const mounted: Array<{ app: App, root: HTMLElement }> = []

function mountEditor(selectedModels?: string[], initialModels = globalModels) {
  const initial = createEmptyRoutingGroupConfig()
  const config = ref(selectedModels === undefined ? initial : writeSchedulingPolicies(initial, [{
    ...createSchedulingPolicy(initial, 'selected'),
    models: selectedModels,
  }]))
  const models = ref(initialModels)
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp({
    setup: () => () => h(RoutingSchedulingPolicyEditor, {
      config: config.value,
      globalModels: models.value,
      'onUpdate:config': (value: RoutingGroupConfig) => { config.value = value },
    }),
  })
  app.mount(root)
  mounted.push({ app, root })
  return { root, config, models }
}

function providerNames(root: HTMLElement): string[] {
  return [...root.querySelectorAll('[draggable="true"] .font-medium')]
    .map(element => element.textContent?.trim() ?? '')
}

async function clickButton(root: HTMLElement, label: string) {
  const button = [...root.querySelectorAll<HTMLButtonElement>('button')]
    .find(element => element.getAttribute('aria-label') === label || element.textContent?.trim() === label)
  expect(button, `Missing button: ${label}`).toBeTruthy()
  button!.click()
  await nextTick()
}

async function toggleModel(root: HTMLElement, name: string) {
  if (!root.querySelector('[aria-label="全局模型选择列表"]')) {
    await clickButton(root, '选择适用模型')
  }
  const checkbox = root.querySelector<HTMLInputElement>(`[aria-label="选择模型 ${name}"]`)
  expect(checkbox).toBeTruthy()
  checkbox!.click()
  await nextTick()
}

beforeEach(() => {
  vi.mocked(getProvidersSummary).mockReset()
  vi.mocked(getProvidersSummary).mockResolvedValue({
    items: providerSources, total: providerSources.length, page: 1, page_size: 9999,
  })
  vi.stubGlobal('ResizeObserver', class {
    observe() {}
    unobserve() {}
    disconnect() {}
  })
})

afterEach(() => {
  for (const { app, root } of mounted.splice(0)) {
    app.unmount()
    root.remove()
  }
  vi.unstubAllGlobals()
})

describe('scheduling provider filtering', () => {
  it('filters providers after choosing a global model and restores all-model mode', async () => {
    const { root } = mountEditor()
    await vi.waitFor(() => expect(providerNames(root)).toHaveLength(4))
    await clickButton(root, '区分模型')
    expect(providerNames(root)).toEqual([])
    await toggleModel(root, 'model-a')
    await vi.waitFor(() => expect(providerNames(root)).toEqual(['提供商 A', '共享提供商']))
    await clickButton(root, '全部模型')
    await vi.waitFor(() => expect(providerNames(root)).toHaveLength(4))
  })

  it('shows the union for multiple models once and updates immediately when deselected', async () => {
    const { root } = mountEditor(['model-a'])
    await vi.waitFor(() => expect(providerNames(root)).toEqual(['提供商 A', '共享提供商']))
    await toggleModel(root, 'model-b')
    expect(providerNames(root)).toEqual(['提供商 A', '提供商 B', '共享提供商'])
    await toggleModel(root, 'model-a')
    expect(providerNames(root)).toEqual(['提供商 B', '共享提供商'])
    expect(getProvidersSummary).toHaveBeenCalledTimes(1)
    await toggleModel(root, 'model-b')
    expect(providerNames(root)).toEqual([])
    expect(root.textContent).not.toContain('提供商排序')
  })

  it('keeps the shared ranking attached to every selected model after filtering', async () => {
    const { root, config } = mountEditor(['model-a', 'model-b'])
    await vi.waitFor(() => expect(providerNames(root)).toHaveLength(3))
    const row = [...root.querySelectorAll<HTMLElement>('[draggable="true"]')]
      .find(element => element.textContent?.includes('共享提供商'))!
    const input = row.querySelector<HTMLInputElement>('input[type="number"]')!
    input.value = '7'
    input.dispatchEvent(new Event('change', { bubbles: true }))
    await nextTick()
    for (const model of ['model-a', 'model-b']) {
      expect(getModelPolicy(config.value, model).provider_priority_overrides).toEqual({ 'provider-shared': 7 })
    }
    expect(getModelPolicy(config.value, '*').provider_priority_overrides).toEqual({})
    expect(providerNames(root)).toEqual(['提供商 A', '提供商 B', '共享提供商'])
    expect(getProvidersSummary).toHaveBeenCalledTimes(1)
  })

  it('waits for global model IDs without briefly displaying all providers', async () => {
    const { root, models } = mountEditor(['model-a'], [])
    await vi.waitFor(() => expect(root.textContent).toContain('暂无 Provider'))
    expect(providerNames(root)).toEqual([])
    models.value = globalModels
    await nextTick()
    expect(providerNames(root)).toEqual(['提供商 A', '共享提供商'])
    expect(getProvidersSummary).toHaveBeenCalledTimes(1)
  })

  it.each(['model-c', 'missing-model'])('shows an empty list when no providers match %s', async model => {
    const { root } = mountEditor([model])
    await vi.waitFor(() => expect(root.textContent).toContain('暂无 Provider'))
    expect(providerNames(root)).toEqual([])
  })

  it('drops hidden providers from the temporary multiselection when models change', async () => {
    const { root } = mountEditor(['model-a', 'model-b'])
    await vi.waitFor(() => expect(providerNames(root)).toHaveLength(3))
    await clickButton(root, '多选')
    const checkbox = root.querySelector<HTMLInputElement>('[aria-label="选择 提供商 A"]')!
    checkbox.click()
    await nextTick()
    expect(checkbox.checked).toBe(true)
    await toggleModel(root, 'model-a')
    expect(providerNames(root)).toEqual(['提供商 B', '共享提供商'])
    await toggleModel(root, 'model-a')
    expect(root.querySelector<HTMLInputElement>('[aria-label="选择 提供商 A"]')!.checked).toBe(false)
  })
})
