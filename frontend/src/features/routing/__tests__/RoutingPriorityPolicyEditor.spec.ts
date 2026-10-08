import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, h, nextTick, ref, type App } from 'vue'
import client from '@/api/client'
import { getProvidersSummary, type ProviderWithEndpointsSummary } from '@/api/endpoints'
import RoutingPriorityPolicyEditor from '../components/RoutingPriorityPolicyEditor.vue'
import { createEmptyRoutingGroupConfig, getDefaultModelPolicy, type RoutingGroupConfig } from '../utils/routingPolicy'

vi.mock('@/api/endpoints', () => ({ getProvidersSummary: vi.fn() }))
vi.mock('@/api/client', () => ({ default: { get: vi.fn() } }))

const providerSources = ['A', 'B', 'C', 'D'].map((name, index) => ({
  id: name,
  name: `提供商 ${name}`,
  global_model_ids: index === 3 ? ['other-model'] : ['selected-model'],
  provider_priority: [0, 1, 1, 2][index],
  is_active: true,
  active_keys: 2,
  total_keys: 3,
  avg_health_score: 0.9,
  api_formats: ['openai:chat'],
  pool_advanced: name === 'C' ? { global_priority: 1 } : null,
})) as ProviderWithEndpointsSummary[]

const mounted: Array<{ app: App, root: HTMLElement }> = []

function mountEditor(options: { modelIds?: string[], keyMode?: boolean } = {}) {
  const initial = createEmptyRoutingGroupConfig()
  if (options.keyMode) initial.default_policy.priority_mode = 'global_key'
  initial.model_policies = [{
    ...getDefaultModelPolicy(initial),
    provider_priority_overrides: { outside: 99 },
    key_priority_overrides_by_format: { 'openai:chat': { outside: 99 }, 'claude:messages': { 'claude-key': 45 } },
    pool_priority_overrides: { 'other-pool': 88 },
  }]
  const config = ref(initial)
  const modelIds = ref(options.modelIds)
  const revision = ref(0)
  const inspect = vi.fn()
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp({
    setup: () => () => h(RoutingPriorityPolicyEditor, {
      config: config.value,
      providerModelIds: modelIds.value,
      refreshRevision: revision.value,
      onInspectProvider: inspect,
      'onUpdate:config': (value: RoutingGroupConfig) => { config.value = value },
    }),
  })
  app.mount(root)
  mounted.push({ app, root })
  return { root, config, revision, inspect, modelIds }
}

function control<T extends HTMLElement>(root: HTMLElement, label: string): T {
  const element = root.querySelector<T>(`[aria-label="${label}"]`)
  expect(element, label).toBeTruthy()
  return element!
}

function rowNames(root: HTMLElement): string[] {
  return [...root.querySelectorAll('[draggable="true"] .font-medium')].map(element => element.textContent!.trim())
}

async function search(root: HTMLElement, value: string, keyMode = false) {
  const input = control<HTMLInputElement>(root, keyMode ? '搜索调度 Key' : '搜索调度提供商')
  input.value = value
  input.dispatchEvent(new Event('input', { bubbles: true }))
  await nextTick()
}

async function click(root: HTMLElement, label: string) {
  control<HTMLButtonElement>(root, label).click()
  await nextTick()
}

beforeEach(() => {
  vi.mocked(getProvidersSummary).mockReset()
  vi.mocked(getProvidersSummary).mockResolvedValue({ items: providerSources, total: 4, page: 1, page_size: 9999 })
  vi.mocked(client.get).mockReset()
  vi.stubGlobal('ResizeObserver', class { observe() {} unobserve() {} disconnect() {} })
})

afterEach(() => {
  for (const { app, root } of mounted.splice(0)) { app.unmount(); root.remove() }
  vi.unstubAllGlobals()
})

describe('RoutingPriorityPolicyEditor ordering', () => {
  it('moves a searched provider through the full order while preserving untouched priority ties and hidden overrides', async () => {
    const { root, config } = mountEditor()
    await vi.waitFor(() => expect(rowNames(root)).toHaveLength(4))
    await search(root, '提供商 D')
    expect(rowNames(root)).toEqual(['提供商 D'])
    expect(control<HTMLButtonElement>(root, '置顶 提供商 D').disabled).toBe(false)
    await click(root, '置顶 提供商 D')
    expect(getDefaultModelPolicy(config.value).provider_priority_overrides).toEqual({ outside: 99, D: 0, A: 1, B: 2, C: 2 })
    await search(root, '')
    expect(rowNames(root)).toEqual(['提供商 D', '提供商 A', '提供商 B', '提供商 C'])
    await click(root, '置底 提供商 D')
    expect(rowNames(root)).toEqual(['提供商 A', '提供商 B', '提供商 C', '提供商 D'])
    expect(getDefaultModelPolicy(config.value).provider_priority_overrides).toEqual({ outside: 99, A: 0, B: 1, C: 1, D: 2 })
  })

  it('uses actual positions for up/down controls in a filtered list', async () => {
    const { root } = mountEditor()
    await vi.waitFor(() => expect(rowNames(root)).toHaveLength(4))
    await search(root, '提供商 B')
    await click(root, '下移 提供商 B')
    await search(root, '')
    expect(rowNames(root)).toEqual(['提供商 A', '提供商 C', '提供商 B', '提供商 D'])
    await search(root, '提供商 B')
    await click(root, '上移 提供商 B')
    await search(root, '')
    expect(rowNames(root)).toEqual(['提供商 A', '提供商 B', '提供商 C', '提供商 D'])
  })

  it('allows equal numeric priorities and retains overrides outside the model selection when reordered', async () => {
    const { root, config } = mountEditor({ modelIds: ['selected-model'] })
    await vi.waitFor(() => expect(rowNames(root)).toHaveLength(3))
    const input = control<HTMLInputElement>(root, '提供商 A 优先级')
    input.value = '1'
    input.dispatchEvent(new Event('change', { bubbles: true }))
    await nextTick()
    expect(getDefaultModelPolicy(config.value).provider_priority_overrides).toEqual({ outside: 99, A: 1 })
    await click(root, '置顶 提供商 C')
    expect(getDefaultModelPolicy(config.value).provider_priority_overrides).toEqual({ outside: 99, C: 0, A: 1, B: 1 })
  })

  it('keeps the full selected group when search hides some selected providers', async () => {
    const { root, config } = mountEditor()
    await vi.waitFor(() => expect(rowNames(root)).toHaveLength(4))
    const selectButton = [...root.querySelectorAll('button')].find(button => button.textContent?.trim() === '多选')!
    selectButton.click()
    await nextTick()
    await click(root, '选择 提供商 B')
    await click(root, '选择 提供商 D')
    await search(root, 'D')
    await click(root, '置顶 提供商 D')
    await search(root, '')
    expect(rowNames(root)).toEqual(['提供商 B', '提供商 D', '提供商 A', '提供商 C'])
    expect(getDefaultModelPolicy(config.value).provider_priority_overrides).toEqual({ outside: 99, B: 0, D: 1, A: 2, C: 3 })
  })

  it('drags across filtered rows without losing hidden providers', async () => {
    const { root, config } = mountEditor()
    await vi.waitFor(() => expect(rowNames(root)).toHaveLength(4))
    await search(root, 'D')
    root.querySelector('[draggable="true"]')!.dispatchEvent(new Event('dragstart', { bubbles: true }))
    await search(root, 'A')
    root.querySelector('[draggable="true"]')!.dispatchEvent(new Event('drop', { bubbles: true }))
    await nextTick()
    await search(root, '')
    expect(rowNames(root)).toEqual(['提供商 D', '提供商 A', '提供商 B', '提供商 C'])
    expect(getDefaultModelPolicy(config.value).provider_priority_overrides).toEqual({ outside: 99, D: 0, A: 1, B: 2, C: 2 })
  })

  it('uses provider ordering for legacy Key configs without requesting keys or dropping their history', async () => {
    const { root, config } = mountEditor({ keyMode: true, modelIds: ['selected-model'] })
    await vi.waitFor(() => expect(rowNames(root)).toEqual(['提供商 A', '提供商 B', '提供商 C']))
    expect(root.textContent).toContain('提供商排序')
    expect(root.textContent).not.toContain('Key 排序')
    expect(root.querySelector('[aria-label="搜索调度 Key"]')).toBeNull()
    expect(vi.mocked(client.get)).not.toHaveBeenCalled()
    const keyHistory = JSON.stringify(getDefaultModelPolicy(config.value).key_priority_overrides_by_format)
    await click(root, '置顶 提供商 C')
    expect(config.value.default_policy.priority_mode).toBe('provider')
    expect(rowNames(root)).toEqual(['提供商 C', '提供商 A', '提供商 B'])
    expect(JSON.stringify(getDefaultModelPolicy(config.value).key_priority_overrides_by_format)).toBe(keyHistory)
  })

  it('keeps model scope and group exclusions visible for legacy Key configurations', async () => {
    const { root, config, modelIds } = mountEditor({ keyMode: true, modelIds: [] })
    await vi.waitFor(() => expect(root.textContent).toContain('暂无 Provider'))
    expect(rowNames(root)).toEqual([])
    config.value.disabled_providers = ['A', 'C']
    modelIds.value = ['selected-model']
    await vi.waitFor(() => expect(rowNames(root)).toEqual(['提供商 A', '提供商 B', '提供商 C']))
    const rows = [...root.querySelectorAll<HTMLElement>('[draggable="true"]')]
    expect(rows.find(row => row.textContent?.includes('提供商 A'))?.textContent).toContain('本组禁用')
    expect(rows.find(row => row.textContent?.includes('提供商 C'))?.textContent).toContain('本组禁用')
    expect(rows.find(row => row.textContent?.includes('提供商 B'))?.textContent).not.toContain('本组禁用')
    expect(rows.every(row => !row.textContent?.includes('停用'))).toBe(true)
  })

  it('shows the selected model policy membership while retaining legacy exclusions as defaults', async () => {
    const { root, config } = mountEditor()
    config.value.disabled_providers = ['A', 'C']
    config.value.model_policies[0].provider_enabled_overrides = { A: true, B: false }
    await vi.waitFor(() => expect(rowNames(root)).toHaveLength(4))
    const rows = [...root.querySelectorAll<HTMLElement>('[draggable="true"]')]
    expect(rows.find(row => row.textContent?.includes('提供商 A'))?.textContent).not.toContain('本组禁用')
    expect(rows.find(row => row.textContent?.includes('提供商 B'))?.textContent).toContain('本组禁用')
    expect(rows.find(row => row.textContent?.includes('提供商 C'))?.textContent).toContain('本组禁用')
    await click(root, '置顶 提供商 C')
    expect(getDefaultModelPolicy(config.value).provider_enabled_overrides).toEqual({ A: true, B: false })
    expect(config.value.disabled_providers).toEqual(['A', 'C'])
  })

  it('emits provider inspection and refreshes health and keys without altering the draft', async () => {
    const { root, config, revision, inspect } = mountEditor()
    await vi.waitFor(() => expect(rowNames(root)).toHaveLength(4))
    await click(root, '查看提供商 提供商 B')
    expect(inspect).toHaveBeenCalledWith('B')
    await click(root, '置顶 提供商 D')
    const before = JSON.stringify(config.value)
    vi.mocked(getProvidersSummary).mockResolvedValue({
      items: providerSources.map(provider => ({ ...provider, active_keys: 0, avg_health_score: 0.5 })),
      total: 4, page: 1, page_size: 9999,
    })
    revision.value += 1
    await vi.waitFor(() => expect(root.textContent).toContain('可用 Key 0 / 3'))
    expect(root.textContent).toContain('健康度 50%')
    expect(JSON.stringify(config.value)).toBe(before)
  })
})
