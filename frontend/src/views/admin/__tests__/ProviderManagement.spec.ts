import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, h, nextTick, type App, type PropType } from 'vue'
import { createMemoryHistory, createRouter, RouterView, type Router } from 'vue-router'
import type { ProviderWithEndpointsSummary } from '@/api/endpoints'
import { createI18n } from '@/i18n'
import ProviderManagement from '../ProviderManagement.vue'
import { createSchedulingPolicy, writeSchedulingPolicies } from '@/features/routing/utils/schedulingPolicies'
import { createEmptyRoutingGroupConfig, getDefaultModelPolicy, getModelPolicy, setModelProviderPriorityOverrides, type RoutingGroupConfig, type RoutingModelPolicy } from '@/features/routing/utils/routingPolicy'

const workspace = vi.hoisted(() => ({
  groups: {} as Record<string, RoutingGroupConfig>,
  busy: false,
  selectionReady: true,
  updatePriorityPolicy: vi.fn(),
  updateDraftConfig: vi.fn(),
  refreshGroups: vi.fn().mockResolvedValue(undefined),
  ensureSaved: vi.fn().mockResolvedValue(true),
}))

const apiMocks = vi.hoisted(() => ({
  getProvidersSummary: vi.fn(),
  getGlobalModels: vi.fn(),
  getProvider: vi.fn(),
  updateProvider: vi.fn(),
}))

const toastMocks = vi.hoisted(() => ({ success: vi.fn(), error: vi.fn(), info: vi.fn() }))

vi.mock('@/api/endpoints', async (importOriginal) => ({
  ...await importOriginal<typeof import('@/api/endpoints')>(),
  ...apiMocks,
}))

vi.mock('@/composables/useConfirm', () => ({
  useConfirm: () => ({ confirmDanger: vi.fn().mockResolvedValue(false) }),
}))

vi.mock('@/composables/useToast', () => ({
  useToast: () => toastMocks,
}))

vi.mock('@/features/providers/composables/useProviderBalance', () => ({
  useProviderBalance: () => ({
    loadArchitectureSchemas: vi.fn(),
    loadBalances: vi.fn(),
    getProviderBalance: () => ({ available: 125, currency: 'USD' }),
    getProviderBalanceBreakdown: () => null,
    getProviderBalanceError: () => null,
    isBalanceLoading: () => false,
    getProviderCheckin: () => null,
    getProviderCookieExpired: () => null,
    formatBalanceDisplay: () => '$125.00',
    formatResetCountdown: () => '',
    getProviderBalanceExtra: () => [],
    getQuotaUsedColorClass: () => '',
    startTick: vi.fn(),
    stopTick: vi.fn(),
  }),
}))

vi.mock('@/features/providers/components', async () => {
  const { defineComponent, h } = await import('vue')
  return {
    ProviderFormDialog: defineComponent({
      props: {
        modelValue: Boolean,
        provider: { type: Object as PropType<ProviderWithEndpointsSummary | null>, default: null },
        routingGroupId: { type: String, default: '' },
        routingGroupName: { type: String, default: '' },
      },
      emits: ['provider-updated', 'update:modelValue'],
      setup: (props, { emit }) => () => props.modelValue
        ? h('button', {
            'data-save-edited-provider': '',
            'data-routing-group-id': props.routingGroupId,
            'data-routing-group-name': props.routingGroupName,
            onClick: () => {
              emit('provider-updated', { ...props.provider, name: 'Edited provider name' })
              emit('update:modelValue', false)
            },
          }, '保存提供商编辑')
        : null,
    }),
    ProviderAuthDialog: { render: () => null },
  }
})

vi.mock('@/features/providers/components/ProviderBatchActionDialog.vue', () => ({
  default: { render: () => null },
}))

vi.mock('@/features/providers/components/ProviderDetailDrawer.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return {
    __esModule: true,
    default: defineComponent({
      props: {
        open: Boolean,
        providerId: { type: String, default: '' },
        initialProvider: { type: Object as PropType<ProviderWithEndpointsSummary | null>, default: null },
      },
      emits: ['edit'],
      setup: (props, { emit }) => () => props.open
        ? h('div', {
            'data-provider-detail': props.providerId,
            'data-initial-provider-name': props.initialProvider?.name,
          }, [
            h('button', {
              'data-edit-provider': '',
              onClick: () => emit('edit', props.initialProvider ?? createProvider({ id: props.providerId })),
            }, '编辑提供商'),
          ])
        : null,
    }),
  }
})

vi.mock('@/features/providers/components/ProviderSchedulingView.vue', async () => {
  const { computed, defineComponent, h, ref, shallowRef, watch } = await import('vue')
  const { useRoute } = await import('vue-router')
  const { createEmptyRoutingGroupConfig } = await import('@/features/routing/utils/routingPolicy')
  const { readSchedulingPolicies, writeSchedulingPolicies } = await import('@/features/routing/utils/schedulingPolicies')
  return {
    __esModule: true,
    default: defineComponent({
      props: { providerRevision: { type: Number, default: 0 } },
      emits: ['inspect-provider', 'context-change'],
      setup: (_props, { emit, expose, slots }) => {
        const route = useRoute()
        const groupId = computed(() => typeof route.query.group === 'string' ? route.query.group : 'group-a')
        const config = shallowRef<RoutingGroupConfig>(createEmptyRoutingGroupConfig())
        const selectedPolicyIndex = ref(0)
        watch(groupId, id => { config.value = workspace.groups[id] ?? createEmptyRoutingGroupConfig(); selectedPolicyIndex.value = 0 }, { immediate: true })
        const policies = computed(() => readSchedulingPolicies(config.value))
        const activePolicy = computed(() => {
          const entry = policies.value[selectedPolicyIndex.value]!
          return {
            policy: workspace.selectionReady ? entry.policy : null,
            priorityMode: entry.priorityMode,
            schedulingMode: entry.schedulingMode,
            scope: workspace.selectionReady ? entry.scope : null,
            modelNames: entry.models,
          }
        })
        const providerModelIds = computed(() => activePolicy.value.scope === 'selected'
          ? activePolicy.value.modelNames.map(name => ({ 'Model One': 'model-1', 'Model Two': 'model-2', 'Model Three': 'model-3' })[name]).filter((id): id is string => Boolean(id))
          : undefined)
        watch([groupId, config, activePolicy, providerModelIds], () => emit('context-change', {
          groupId: groupId.value === 'new' ? null : groupId.value,
          groupName: `Group ${groupId.value}`,
          config: config.value,
          busy: workspace.busy,
          activePolicy: activePolicy.value,
          providerModelIds: providerModelIds.value,
          priorityMode: activePolicy.value.priorityMode,
          schedulingMode: activePolicy.value.schedulingMode,
        }), { immediate: true })
        expose({
          updateDraftConfig(value: RoutingGroupConfig) {
            workspace.updateDraftConfig(value)
            workspace.groups[groupId.value] = value
            config.value = value
          },
          updatePriorityPolicy(value: RoutingModelPolicy) {
            workspace.updatePriorityPolicy(value)
            const updated = policies.value.map((entry, index) => index === selectedPolicyIndex.value ? { ...entry, policy: value } : entry)
            const nextConfig = writeSchedulingPolicies(config.value, updated)
            workspace.groups[groupId.value] = nextConfig
            config.value = nextConfig
          },
          refreshGroups: workspace.refreshGroups,
          ensureSaved: workspace.ensureSaved,
        })
        return () => h('section', { 'data-scheduling-group': groupId.value }, [
          h('button', {
            'data-inspect-scheduled-provider': '',
            onClick: () => emit('inspect-provider', 'provider-1'),
          }, '查看调度提供商'),
          ...policies.value.map((_entry, index) => h('button', { 'data-select-policy': index, onClick: () => { selectedPolicyIndex.value = index } }, `配置 ${index + 1}`)),
          slots.default?.(),
        ])
      },
    }),
  }
})

function createProvider(overrides: Partial<ProviderWithEndpointsSummary> = {}): ProviderWithEndpointsSummary {
  return {
    id: 'provider-1',
    name: 'Provider One',
    description: 'Primary provider',
    provider_type: 'custom',
    provider_priority: 10,
    keep_priority_on_conversion: false,
    enable_format_conversion: true,
    is_active: true,
    total_endpoints: 2,
    active_endpoints: 1,
    total_keys: 3,
    active_keys: 2,
    total_models: 4,
    active_models: 3,
    global_model_ids: ['model-1'],
    avg_health_score: 0.8,
    unhealthy_endpoints: 0,
    api_formats: ['openai:chat'],
    endpoint_health_details: [{
      api_format: 'openai:chat',
      health_score: 0.8,
      is_active: true,
      total_keys: 3,
      active_keys: 2,
    }],
    ops_configured: true,
    created_at: '2026-09-07T00:00:00Z',
    updated_at: '2026-09-07T00:00:00Z',
    ...overrides,
  }
}

let mountedApp: App | null = null
let mountedRoot: HTMLElement | null = null
let mountedRouter: Router | null = null

async function settle() {
  for (let index = 0; index < 8; index += 1) {
    await Promise.resolve()
    await nextTick()
  }
}

async function mountView(path = '/admin/providers') {
  const root = document.createElement('div')
  document.body.appendChild(root)
  mountedRoot = root
  mountedRouter = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/admin/providers', name: 'ProviderManagement', component: ProviderManagement }],
  })
  await mountedRouter.push(path)
  await mountedRouter.isReady()
  mountedApp = createApp({ render: () => h(RouterView) })
  mountedApp.use(mountedRouter)
  mountedApp.use(createI18n())
  mountedApp.mount(root)
  await settle()
  await vi.waitFor(() => expect(root.querySelector('[data-scheduling-group]')).not.toBeNull())
  await settle()
  return root
}

function unmountView() {
  mountedApp?.unmount()
  mountedRoot?.remove()
  mountedApp = null
  mountedRoot = null
  mountedRouter = null
}

function findButton(root: HTMLElement, title: string): HTMLButtonElement {
  const button = root.querySelector<HTMLButtonElement>(`button[title="${title}"]`)
  expect(button, `Missing button: ${title}`).not.toBeNull()
  return button!
}

async function openPriorityInput(root: HTMLElement, providerName: string): Promise<HTMLInputElement> {
  const label = `${providerName} 的组内优先级`
  const button = root.querySelector<HTMLButtonElement>(`button[aria-label="${label}"]`)
  expect(button, `Missing priority button: ${providerName}`).not.toBeNull()
  button!.click()
  await nextTick()
  const input = root.querySelector<HTMLInputElement>(`input[aria-label="${label}"]`)
  expect(input, `Missing priority input: ${providerName}`).not.toBeNull()
  return input!
}


beforeEach(() => {
  vi.clearAllMocks()
  workspace.groups = { 'group-a': createEmptyRoutingGroupConfig(), 'group-b': createEmptyRoutingGroupConfig() }
  workspace.busy = false
  workspace.selectionReady = true
  apiMocks.getProvidersSummary.mockResolvedValue({
    items: [createProvider()],
    total: 1,
  })
  apiMocks.getGlobalModels.mockResolvedValue({ models: [{ id: 'model-1', name: 'Model One' }] })
  apiMocks.getProvider.mockResolvedValue(createProvider())
  apiMocks.updateProvider.mockResolvedValue(createProvider({ is_active: false }))
})

const originalElementFromPoint = Object.getOwnPropertyDescriptor(document, 'elementFromPoint')

afterEach(() => {
  unmountView()
  if (originalElementFromPoint) {
    Object.defineProperty(document, 'elementFromPoint', originalElementFromPoint)
  } else {
    Reflect.deleteProperty(document, 'elementFromPoint')
  }
})

describe('ProviderManagement provider directory', () => {
  it('filters and paginates the current search locally', async () => {
    apiMocks.getProvidersSummary.mockResolvedValue({ items: Array.from({ length: 30 }, (_, index) => createProvider({ id: `provider-${index + 1}` })), total: 30 })
    const root = await mountView()
    const search = root.querySelector<HTMLInputElement>('#provider-search')!
    search.value = 'Provider'
    search.dispatchEvent(new Event('input', { bubbles: true }))
    await settle()
    const secondPage = root.querySelector<HTMLButtonElement>('button[aria-label="第 2 页"]')!
    secondPage.click()
    await settle()

    expect(search.value).toBe('Provider')
    expect(root.querySelector('[aria-current="page"]')?.textContent?.trim()).toBe('2')
    expect(apiMocks.getProvidersSummary).toHaveBeenCalledTimes(1)
    expect(apiMocks.getProvidersSummary).toHaveBeenCalledWith({ page: 1, page_size: 10000 }, expect.any(Object))
  })

  it.each(['table', 'mobile'] as const)('supports note editing, status actions, and details from the %s list', async layout => {
    const root = await mountView()
    const row = providerElements(root, layout)[0]!
    row.querySelector<HTMLElement>('[title="Primary provider"]')!.click()
    await settle()

    const input = row.querySelector<HTMLInputElement>('[data-desc-editor] input')!
    expect(input.value).toBe('Primary provider')
    input.value = 'Updated note'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await settle()

    expect(apiMocks.updateProvider).toHaveBeenCalledWith('provider-1', { description: 'Updated note' })
    expect(root.textContent).toContain('Updated note')
    expect(root.querySelector('[data-provider-detail]')).toBeNull()

    findButton(row, '全局停用提供商').click()
    await settle()
    expect(apiMocks.updateProvider).toHaveBeenCalledWith('provider-1', { is_active: false })
    expect(root.querySelector('[data-provider-detail]')).toBeNull()

    findButton(row, '查看详情').click()
    await vi.waitFor(() => {
      expect(root.querySelector('[data-provider-detail="provider-1"]')).not.toBeNull()
    })
  })

  it('uses account labels and handles providers without endpoints', async () => {
    apiMocks.getProvidersSummary.mockResolvedValue({
      items: [createProvider({ provider_type: 'codex', is_active: false, endpoint_health_details: [] })],
      total: 1,
    })
    const root = await mountView()

    expect(providerElements(root)[0]?.textContent).toContain('账号')
    expect(root.textContent).toContain('暂无端点')
    expect(findButton(root, '全局启用提供商')).not.toBeNull()
  })

  it('does not display provider rows during loading or with an empty result', async () => {
    let resolveRequest!: (value: { items: ProviderWithEndpointsSummary[]; total: number }) => void
    apiMocks.getProvidersSummary.mockReturnValue(new Promise((resolve) => {
      resolveRequest = resolve
    }))
    const root = await mountView()
    expect(root.querySelector('[data-provider-sort-id]')).toBeNull()
    expect(findButton(root, '刷新').disabled).toBe(true)

    resolveRequest({ items: [], total: 0 })
    await settle()
    expect(root.querySelector('[data-provider-sort-id]')).toBeNull()
    expect(root.textContent).toContain('暂无提供商，点击右上角添加')
    expect(findButton(root, '刷新').disabled).toBe(false)
  })


})

describe('ProviderManagement group directory', () => {
  it('opens the unified provider directory directly for a group', async () => {
    const root = await mountView('/admin/providers?group=group-b')
    expect(root.querySelector('[data-scheduling-group="group-b"]')).not.toBeNull()
    expect(root.querySelector('table')).not.toBeNull()
    expect(root.querySelector('[aria-label="提供商视图"]')).toBeNull()
    expect(apiMocks.getProvidersSummary).toHaveBeenCalledWith({ page: 1, page_size: 10000 }, expect.any(Object))
    expect(root.textContent).toContain('$125.00')
  })

  it.each(['table', 'mobile list'] as const)('keeps the %s group action before details and isolated from global provider state', async layout => {
    const providers = mockSortableProviders()
    providers[3]!.is_active = false
    workspace.groups['group-b'] = {
      ...setModelProviderPriorityOverrides(createEmptyRoutingGroupConfig(), '*', { 'provider-4': 0, 'provider-2': 1, 'provider-3': 2, 'provider-1': 3 }),
      disabled_providers: ['provider-4'],
    }
    const root = await mountView('/admin/providers?group=group-a')
    expect(providerOrder(root)).toEqual(['provider-1', 'provider-2', 'provider-3', 'provider-4'])
    await mountedRouter!.push('/admin/providers?group=group-b')
    await settle()
    expect(providerOrder(root)).toEqual(['provider-4', 'provider-2', 'provider-3', 'provider-1'])
    const row = [...root.querySelectorAll<HTMLElement>('[data-provider-sort-id="provider-4"]')]
      .find(element => layout === 'table' ? element.closest('table') : !element.closest('table'))!
    const toggle = row.querySelector<HTMLButtonElement>('[aria-label="Provider 4 本组启用"]')!
    expect(toggle).not.toBeNull()
    expect(toggle.nextElementSibling).toBe(findButton(row, '查看详情'))
    expect(toggle.getAttribute('aria-pressed')).toBe('false')
    expect(toggle.title).toBe('本组启用提供商')
    expect(toggle.textContent?.trim()).toBe('')
    expect(row.querySelector('[role="switch"]')).toBeNull()
    expect(row.textContent).toContain('本组禁用')
    expect(row.textContent).toContain('全局停用')
    toggle.dispatchEvent(new MouseEvent('pointerdown', { bubbles: true }))
    toggle.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    toggle.click()
    await settle()
    expect(toggle.getAttribute('aria-pressed')).toBe('true')
    expect(toggle.title).toBe('本组禁用提供商')
    expect(row.textContent).toContain('本组启用')
    expect(row.textContent).not.toContain('本组禁用')
    expect(row.textContent).toContain('全局停用')
    expect(workspace.groups['group-b']!.disabled_providers).toEqual(['provider-4'])
    expect(getDefaultModelPolicy(workspace.groups['group-b']!).provider_enabled_overrides).toEqual({ 'provider-4': true })
    expect(workspace.groups['group-a']!.disabled_providers).toEqual([])
    expect(workspace.updateDraftConfig).not.toHaveBeenCalled()
    expect(workspace.updatePriorityPolicy).toHaveBeenCalledOnce()
    expect(providers[3]!.is_active).toBe(false)
    expect(apiMocks.updateProvider).not.toHaveBeenCalled()
    expect(apiMocks.getProvidersSummary).toHaveBeenCalledTimes(1)
    expect(root.querySelector('[data-provider-detail]')).toBeNull()
  })

  it('disables group actions while the selected group is busy', async () => {
    workspace.busy = true
    const root = await mountView()
    const toggles = root.querySelectorAll<HTMLButtonElement>('[aria-label="Provider One 本组启用"]')
    expect(toggles.length).toBeGreaterThan(0)
    for (const toggle of toggles) {
      expect(toggle.disabled).toBe(true)
      toggle.click()
    }
    await settle()
    expect(workspace.updateDraftConfig).not.toHaveBeenCalled()
    expect(apiMocks.updateProvider).not.toHaveBeenCalled()
    expect(root.querySelector('[data-provider-detail]')).toBeNull()
  })

  it('edits the selected group priority from a row without opening details', async () => {
    mockSortableProviders()
    const root = await mountView()
    expect(root.querySelector('input[aria-label$="的组内优先级"]')).toBeNull()
    const input = await openPriorityInput(root, 'Provider 4')
    expect(root.querySelector('[data-provider-detail]')).toBeNull()
    input.value = '0'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    expect(workspace.updatePriorityPolicy).not.toHaveBeenCalled()
    input.blur()
    await settle()
    expect(providerOrder(root)).toEqual(['provider-4', 'provider-1', 'provider-2', 'provider-3'])
    expect(getModelPolicy(workspace.groups['group-a']!, '*').provider_priority_overrides['provider-4']).toBe(0)
    expect(apiMocks.updateProvider).not.toHaveBeenCalled()
    expect(root.querySelector('[data-provider-detail]')).toBeNull()
  })

  it('keeps globally or group-disabled providers visible and toggles only group state', async () => {
    const providers = mockSortableProviders()
    providers[0]!.is_active = false
    workspace.groups['group-a']!.disabled_providers = ['provider-2']
    const root = await mountView()
    expect(providerOrder(root)).toEqual(['provider-1', 'provider-2', 'provider-3', 'provider-4'])
    const globallyDisabled = providerElements(root).find(row => row.dataset.providerSortId === 'provider-1')!
    const groupDisabled = providerElements(root).find(row => row.dataset.providerSortId === 'provider-2')!
    expect(globallyDisabled.textContent).toContain('全局停用')
    expect(globallyDisabled.textContent).toContain('本组启用')
    expect(groupDisabled.textContent).toContain('全局启用')
    expect(groupDisabled.textContent).toContain('本组禁用')

    groupDisabled.querySelector<HTMLButtonElement>('[aria-label="Provider 2 本组启用"]')!.click()
    await settle()
    expect(providerOrder(root)).toEqual(['provider-1', 'provider-2', 'provider-3', 'provider-4'])
    expect(groupDisabled.textContent).toContain('本组启用')
    expect(workspace.groups['group-a']!.disabled_providers).toEqual(['provider-2'])
    expect(getDefaultModelPolicy(workspace.groups['group-a']!).provider_enabled_overrides).toEqual({ 'provider-2': true })

    globallyDisabled.querySelector<HTMLButtonElement>('[aria-label="Provider 1 本组启用"]')!.click()
    await settle()
    expect(providerOrder(root)).toEqual(['provider-1', 'provider-2', 'provider-3', 'provider-4'])
    expect(globallyDisabled.textContent).toContain('本组禁用')
    expect(globallyDisabled.textContent).toContain('全局停用')
    expect(providers[0]!.is_active).toBe(false)
    expect(apiMocks.getProvidersSummary).toHaveBeenCalledTimes(1)
  })

  it('changes priority for every model in the top selected configuration while preserving other configurations', async () => {
    const providers = mockSortableProviders()
    providers[1]!.global_model_ids = ['model-2']
    providers[3]!.global_model_ids = ['model-3']
    const config = createEmptyRoutingGroupConfig()
    workspace.groups['group-a'] = writeSchedulingPolicies(config, [
      { ...createSchedulingPolicy(config), models: ['Model One', 'Model Two'] },
      { ...createSchedulingPolicy(config), models: ['Model Three'] },
    ])
    const root = await mountView()
    expect(providerOrder(root)).toEqual(['provider-1', 'provider-2', 'provider-3', 'provider-4'])
    expect(root.querySelector('[title="筛选模型"]')).toBeNull()
    const input = await openPriorityInput(root, 'Provider 3')
    input.value = '0'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await settle()
    expect(providerOrder(root)).toEqual(['provider-3', 'provider-1', 'provider-2', 'provider-4'])
    for (const model of ['Model One', 'Model Two']) {
      expect(getModelPolicy(workspace.groups['group-a']!, model).provider_priority_overrides['provider-3']).toBe(0)
    }
    expect(getModelPolicy(workspace.groups['group-a']!, 'Model Three').provider_priority_overrides).toEqual({})
    expect(workspace.updatePriorityPolicy).toHaveBeenCalledOnce()
    expect(workspace.updateDraftConfig).not.toHaveBeenCalled()

    root.querySelector<HTMLButtonElement>('[data-select-policy="1"]')!.click()
    await settle()
    expect(providerOrder(root)).toEqual(['provider-1', 'provider-2', 'provider-3', 'provider-4'])
    const otherInput = await openPriorityInput(root, 'Provider 4')
    otherInput.value = '2'
    otherInput.dispatchEvent(new Event('input', { bubbles: true }))
    otherInput.blur()
    await settle()
    expect(getModelPolicy(workspace.groups['group-a']!, 'Model Three').provider_priority_overrides).toEqual({ 'provider-4': 2 })
    expect(getModelPolicy(workspace.groups['group-a']!, 'Model One').provider_priority_overrides).toEqual({ 'provider-3': 0 })
    root.querySelector<HTMLButtonElement>('[data-select-policy="0"]')!.click()
    await settle()
    expect(providerOrder(root)).toEqual(['provider-3', 'provider-1', 'provider-2', 'provider-4'])
  })

  it.each(['table', 'mobile list'] as const)('keeps model configuration membership independent in the %s', async layout => {
    const providers = mockSortableProviders()
    const config = createEmptyRoutingGroupConfig()
    workspace.groups['group-a'] = writeSchedulingPolicies(config, [
      { ...createSchedulingPolicy(config), models: ['Model One', 'Model Two'] },
      { ...createSchedulingPolicy(config), models: ['Model Three'] },
      createSchedulingPolicy(config, 'all'),
    ])
    const root = await mountView()
    const row = [...root.querySelectorAll<HTMLElement>('[data-provider-sort-id="provider-1"]')]
      .find(element => layout === 'table' ? element.closest('table') : !element.closest('table'))!
    const toggle = row.querySelector<HTMLButtonElement>('[aria-label="Provider 1 本组启用"]')!
    toggle.click()
    await settle()
    expect(toggle.getAttribute('aria-pressed')).toBe('false')
    expect(row.textContent).toContain('本组禁用')
    for (const model of ['Model One', 'Model Two']) {
      expect(getModelPolicy(workspace.groups['group-a']!, model).provider_enabled_overrides).toEqual({ 'provider-1': false })
    }
    expect(getModelPolicy(workspace.groups['group-a']!, 'Model Three').provider_enabled_overrides).toEqual({})
    expect(getDefaultModelPolicy(workspace.groups['group-a']!).provider_enabled_overrides).toEqual({})
    expect(workspace.groups['group-a']!.disabled_providers).toEqual([])

    root.querySelector<HTMLButtonElement>('[data-select-policy="1"]')!.click()
    await settle()
    expect(toggle.getAttribute('aria-pressed')).toBe('true')
    expect(row.textContent).toContain('本组启用')
    expect(row.textContent).not.toContain('本组禁用')
    root.querySelector<HTMLButtonElement>('[data-select-policy="0"]')!.click()
    await settle()
    expect(toggle.getAttribute('aria-pressed')).toBe('false')
    toggle.click()
    await settle()
    expect(getModelPolicy(workspace.groups['group-a']!, 'Model One').provider_enabled_overrides).toEqual({ 'provider-1': true })
    expect(getModelPolicy(workspace.groups['group-a']!, 'Model Three').provider_enabled_overrides).toEqual({})
    expect(workspace.updateDraftConfig).not.toHaveBeenCalled()
    expect(providers[0]!.is_active).toBe(true)
    expect(apiMocks.updateProvider).not.toHaveBeenCalled()
  })

  it('drags the shared model configuration without splitting its models', async () => {
    mockSortableProviders()
    const config = createEmptyRoutingGroupConfig()
    workspace.groups['group-a'] = writeSchedulingPolicies(config, [
      { ...createSchedulingPolicy(config), models: ['Model One', 'Model Two'] },
    ])
    const root = await mountView()
    const { handle } = startProviderDrag(root, 'provider-4', 'provider-1')
    await dropProvider(handle)
    expect(providerOrder(root)).toEqual(['provider-4', 'provider-1', 'provider-2', 'provider-3'])
    const first = getModelPolicy(workspace.groups['group-a']!, 'Model One').provider_priority_overrides
    expect(first['provider-4']).toBe(0)
    expect(getModelPolicy(workspace.groups['group-a']!, 'Model Two').provider_priority_overrides).toEqual(first)
    expect(workspace.updatePriorityPolicy).toHaveBeenCalledOnce()
  })

  it('keeps full-directory pagination when the selected configuration changes', async () => {
    const providers = Array.from({ length: 22 }, (_, index) => createProvider({
      id: `provider-${index + 1}`, name: `Provider ${index + 1}`, provider_priority: index,
      global_model_ids: [index < 11 ? 'model-1' : 'model-2'],
    }))
    apiMocks.getProvidersSummary.mockResolvedValue({ items: providers, total: providers.length })
    const config = createEmptyRoutingGroupConfig()
    workspace.groups['group-a'] = writeSchedulingPolicies(config, [
      { ...createSchedulingPolicy(config), models: ['Model One'] },
      { ...createSchedulingPolicy(config), models: ['Model Two'] },
    ])
    localStorage.setItem('provider-management-page-size', '10')
    const root = await mountView()
    const secondPage = root.querySelector<HTMLButtonElement>('button[aria-label="第 2 页"]')!
    secondPage.click()
    await settle()
    expect(providerOrder(root)).toEqual(providers.slice(10, 20).map(provider => provider.id))
    root.querySelector<HTMLButtonElement>('[data-select-policy="1"]')!.click()
    await settle()
    expect(root.querySelector('[aria-current="page"]')?.textContent?.trim()).toBe('1')
    expect(providerOrder(root)).toEqual(providers.slice(0, 10).map(provider => provider.id))
    expect(apiMocks.getProvidersSummary).toHaveBeenCalledTimes(1)
  })

  it('uses provider ranking for legacy Key groups while retaining group enablement', async () => {
    mockSortableProviders()
    workspace.groups['group-a']!.default_policy.priority_mode = 'global_key'
    const root = await mountView()
    const priorityButton = root.querySelector<HTMLButtonElement>('button[aria-label="Provider 1 的组内优先级"]')!
    expect(priorityButton.disabled).toBe(false)
    expect(root.querySelector('[data-provider-drag-handle]')).not.toBeNull()
    const input = await openPriorityInput(root, 'Provider 1')
    input.value = '7'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await settle()
    expect(workspace.groups['group-a']!.default_policy.priority_mode).toBe('provider')
    expect(getDefaultModelPolicy(workspace.groups['group-a']!).provider_priority_overrides['provider-1']).toBe(7)
    const toggle = providerElements(root)[0]!.querySelector<HTMLButtonElement>('[aria-label="Provider 1 本组启用"]')!
    expect(toggle.disabled).toBe(false)
    toggle.click()
    await settle()
    expect(getDefaultModelPolicy(workspace.groups['group-a']!).provider_enabled_overrides['provider-1']).toBe(false)
    expect(workspace.groups['group-a']!.disabled_providers).toEqual([])
    expect(workspace.updatePriorityPolicy).toHaveBeenCalled()
  })

  it('disables ranking and membership before a configuration is selected without blocking provider creation', async () => {
    mockSortableProviders()
    workspace.selectionReady = false
    const root = await mountView()
    expect(providerOrder(root)).toEqual(['provider-1', 'provider-2', 'provider-3', 'provider-4'])
    const priorityButton = providerElements(root)[0]!.querySelector<HTMLButtonElement>('button[aria-label$="的组内优先级"]')!
    expect(priorityButton.disabled).toBe(true)
    const handle = root.querySelector<HTMLButtonElement>('[data-provider-drag-handle]')
    expect(handle == null || handle.disabled).toBe(true)
    const toggle = providerElements(root)[0]!.querySelector<HTMLButtonElement>('[aria-label="Provider 1 本组启用"]')!
    expect(toggle.disabled).toBe(true)
    toggle.click()
    await settle()
    expect(workspace.groups['group-a']!.disabled_providers).toEqual([])
    findButton(root, '新增提供商').click()
    await settle()
    expect(root.querySelector('[data-routing-group-id="group-a"]')).not.toBeNull()
    expect(workspace.updatePriorityPolicy).not.toHaveBeenCalled()
  })

  it('keeps all providers manageable and sortable when the selected model has no matches', async () => {
    mockSortableProviders()
    const config = createEmptyRoutingGroupConfig()
    workspace.groups['group-a'] = writeSchedulingPolicies(config, [
      { ...createSchedulingPolicy(config), models: ['Missing Model'] },
    ])
    const root = await mountView()
    expect(providerOrder(root)).toEqual(['provider-1', 'provider-2', 'provider-3', 'provider-4'])
    const priorityButton = providerElements(root)[0]!.querySelector<HTMLButtonElement>('button[aria-label$="的组内优先级"]')!
    expect(priorityButton.disabled).toBe(false)
    expect(providerElements(root)[0]!.querySelector<HTMLButtonElement>('[data-provider-drag-handle]')?.disabled).toBe(false)
    const input = await openPriorityInput(root, 'Provider 1')
    input.value = '0'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await settle()
    expect(workspace.updatePriorityPolicy).toHaveBeenCalledOnce()
    expect(getModelPolicy(workspace.groups['group-a']!, 'Missing Model').provider_priority_overrides['provider-1']).toBe(0)
    const toggle = providerElements(root)[0]!.querySelector<HTMLButtonElement>('[aria-label="Provider 1 本组启用"]')!
    expect(toggle.disabled).toBe(false)
    toggle.click()
    await settle()
    expect(getModelPolicy(workspace.groups['group-a']!, 'Missing Model').provider_enabled_overrides).toEqual({ 'provider-1': false })
    expect(workspace.groups['group-a']!.disabled_providers).toEqual([])
    expect(providerOrder(root)).toEqual(['provider-1', 'provider-2', 'provider-3', 'provider-4'])
    expect(workspace.updatePriorityPolicy).toHaveBeenCalledTimes(2)
  })

  it('collects every API page before applying group priority', async () => {
    apiMocks.getProvidersSummary
      .mockResolvedValueOnce({ items: [createProvider({ id: 'provider-1' })], total: 2 })
      .mockResolvedValueOnce({ items: [createProvider({ id: 'provider-2', name: 'Second Provider' })], total: 2 })
    workspace.groups['group-a'] = setModelProviderPriorityOverrides(createEmptyRoutingGroupConfig(), '*', { 'provider-2': 0 })
    const root = await mountView()
    expect(apiMocks.getProvidersSummary).toHaveBeenNthCalledWith(2, { page: 2, page_size: 10000 }, expect.any(Object))
    expect(providerOrder(root)).toEqual(['provider-2', 'provider-1'])
  })

  it('passes the current group to creation and captures it until the dialog closes', async () => {
    const root = await mountView('/admin/providers?group=group-b')
    findButton(root, '新增提供商').click()
    await settle()
    const form = root.querySelector<HTMLElement>('[data-save-edited-provider]')!
    expect(form.dataset.routingGroupId).toBe('group-b')
    expect(form.dataset.routingGroupName).toBe('Group group-b')
    await mountedRouter!.push('/admin/providers?group=group-a')
    await settle()
    expect(form.dataset.routingGroupId).toBe('group-b')
  })

  it('does not create a provider before the new group has been saved', async () => {
    const root = await mountView('/admin/providers?group=new')
    findButton(root, '新增提供商').click()
    await settle()
    expect(root.querySelector('[data-save-edited-provider]')).toBeNull()
    expect(toastMocks.info).toHaveBeenCalledWith('请先保存新分组，再添加提供商')
    expect(workspace.ensureSaved).not.toHaveBeenCalled()
  })

  it('edits provider priority and membership in an unsaved group without saving it', async () => {
    const providers = mockSortableProviders()
    workspace.groups.new = {
      ...createEmptyRoutingGroupConfig(),
      disabled_providers: ['provider-4'],
    }
    const savedGroups = {
      'group-a': structuredClone(workspace.groups['group-a']),
      'group-b': structuredClone(workspace.groups['group-b']),
    }
    const root = await mountView('/admin/providers?group=new')
    expect(root.querySelector('[data-scheduling-group="new"]')).not.toBeNull()
    expect(providerOrder(root)).toEqual(['provider-1', 'provider-2', 'provider-3', 'provider-4'])

    const input = await openPriorityInput(root, 'Provider 4')
    expect(input.disabled).toBe(false)
    input.value = '0'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await settle()

    expect(getDefaultModelPolicy(workspace.groups.new!).provider_priority_overrides['provider-4']).toBe(0)
    expect(providerOrder(root)).toEqual(['provider-4', 'provider-1', 'provider-2', 'provider-3'])
    const row = providerElements(root)[0]!
    const toggle = row.querySelector<HTMLButtonElement>('[aria-label="Provider 4 本组启用"]')!
    expect(toggle.disabled).toBe(false)
    expect(toggle.getAttribute('aria-pressed')).toBe('false')
    toggle.click()
    await settle()

    expect(toggle.getAttribute('aria-pressed')).toBe('true')
    expect(workspace.groups.new!.disabled_providers).toEqual(['provider-4'])
    expect(getDefaultModelPolicy(workspace.groups.new!).provider_enabled_overrides).toEqual({ 'provider-4': true })
    expect(getDefaultModelPolicy(workspace.groups.new!).provider_priority_overrides['provider-4']).toBe(0)
    expect(workspace.updatePriorityPolicy).toHaveBeenCalledTimes(2)
    expect(workspace.updateDraftConfig).not.toHaveBeenCalled()
    expect(workspace.groups['group-a']).toEqual(savedGroups['group-a'])
    expect(workspace.groups['group-b']).toEqual(savedGroups['group-b'])
    expect(workspace.ensureSaved).not.toHaveBeenCalled()
    expect(workspace.refreshGroups).not.toHaveBeenCalled()
    expect(apiMocks.updateProvider).not.toHaveBeenCalled()
    expect(providers.map(provider => provider.provider_priority)).toEqual([10, 20, 30, 40])
    expect(mountedRouter!.currentRoute.value.query.group).toBe('new')
    expect(root.querySelector('[data-provider-detail]')).toBeNull()
  })

  it('opens details and applies edited snapshots for providers outside the loaded directory', async () => {
    apiMocks.getProvidersSummary.mockResolvedValue({ items: [], total: 0 })
    const root = await mountView('/admin/providers?group=group-b')
    root.querySelector<HTMLButtonElement>('[data-inspect-scheduled-provider]')!.click()
    await vi.waitFor(() => expect(root.querySelector('[data-provider-detail="provider-1"]')).not.toBeNull())
    const drawer = root.querySelector<HTMLElement>('[data-provider-detail="provider-1"]')!
    expect(drawer.hasAttribute('data-initial-provider-name')).toBe(false)
    root.querySelector<HTMLButtonElement>('[data-edit-provider]')!.click()
    await vi.waitFor(() => expect(root.querySelector('[data-save-edited-provider]')).not.toBeNull())
    root.querySelector<HTMLButtonElement>('[data-save-edited-provider]')!.click()
    await settle()
    expect(drawer.dataset.initialProviderName).toBe('Edited provider name')
    expect(mountedRouter!.currentRoute.value.query).toEqual({ group: 'group-b' })
  })
})

function mockSortableProviders() {
  const providers = [1, 2, 3, 4].map(index => createProvider({
    id: `provider-${index}`,
    name: `Provider ${index}`,
    provider_priority: index * 10,
  }))
  apiMocks.getProvidersSummary.mockResolvedValue({ items: providers, total: providers.length })
  return providers
}

function providerElements(root: HTMLElement, layout: 'table' | 'mobile' = 'table'): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>('[data-provider-sort-id]')]
    .filter(element => layout === 'table' ? element.closest('table') : !element.closest('table'))
}

function providerOrder(root: HTMLElement): string[] {
  return providerElements(root).map(element => element.dataset.providerSortId!)
}

function pointerEvent(type: string, clientX: number, clientY: number, options: { button?: number; pointerType?: string } = {}) {
  const event = new MouseEvent(type, { bubbles: true, cancelable: true, clientX, clientY, button: options.button ?? 0 })
  Object.defineProperties(event, {
    pointerId: { value: 1 },
    isPrimary: { value: true },
    pointerType: { value: options.pointerType ?? 'mouse' },
  })
  return event
}

function startProviderDrag(root: HTMLElement, sourceId: string, targetId: string, pointerType = 'mouse', layout: 'table' | 'mobile' = 'table') {
  const elements = providerElements(root, layout)
  const handle = elements.find(element => element.dataset.providerSortId === sourceId)!
    .querySelector<HTMLButtonElement>('[data-provider-drag-handle]')!
  const target = elements.find(element => element.dataset.providerSortId === targetId)!
  const hitTest = vi.fn((): Element | null => target)
  Object.defineProperty(document, 'elementFromPoint', { configurable: true, value: hitTest })
  handle.dispatchEvent(pointerEvent('pointerdown', 40, 100, { pointerType }))
  window.dispatchEvent(pointerEvent('pointermove', 100, 200, { pointerType }))
  return { handle, target, hitTest }
}

async function dropProvider(handle: HTMLButtonElement) {
  window.dispatchEvent(pointerEvent('pointerup', 100, 200))
  handle.click()
  await settle()
}

describe('ProviderManagement group priority ordering', () => {
  it('drags table rows, synchronizes the mobile list, and updates only the current group draft', async () => {
    const providers = mockSortableProviders()
    const root = await mountView()
    const { handle, target } = startProviderDrag(root, 'provider-1', 'provider-3')
    await settle()
    expect(target.classList.contains('ring-2')).toBe(true)
    expect(providerElements(root)[0]?.classList.contains('opacity-40')).toBe(true)

    await dropProvider(handle)
    const expected = ['provider-2', 'provider-3', 'provider-1', 'provider-4']
    expect(providerOrder(root)).toEqual(expected)
    const mobileOrder = [...root.querySelectorAll<HTMLElement>('[data-provider-sort-id]')]
      .filter(element => !element.closest('table'))
      .map(element => element.dataset.providerSortId)
    expect(mobileOrder).toEqual(expected)
    expect(root.querySelector('[data-provider-detail]')).toBeNull()
    expect(apiMocks.updateProvider).not.toHaveBeenCalled()
    expect(providers.map(provider => provider.provider_priority)).toEqual([10, 20, 30, 40])
    expect(getModelPolicy(workspace.groups['group-a']!, '*').provider_priority_overrides).toEqual({ 'provider-2': 0, 'provider-3': 1, 'provider-1': 2, 'provider-4': 3 })
    expect(workspace.updatePriorityPolicy).toHaveBeenCalledOnce()

    expect(providerOrder(root)).toEqual(expected)
    expect(apiMocks.getProvidersSummary).toHaveBeenCalledTimes(1)
  })

  it('supports mobile touch dragging and retains the group draft across resource refresh and remounts', async () => {
    mockSortableProviders()
    let root = await mountView()
    const { handle } = startProviderDrag(root, 'provider-4', 'provider-1', 'touch', 'mobile')
    await dropProvider(handle)
    const expected = ['provider-4', 'provider-1', 'provider-2', 'provider-3']
    expect(providerOrder(root)).toEqual(expected)
    expect(JSON.parse(localStorage.getItem('aether-provider-display-order') ?? '[]')).toEqual([])

    findButton(root, '刷新').click()
    await settle()
    expect(providerOrder(root)).toEqual(expected)

    unmountView()
    root = await mountView()
    expect(providerOrder(root)).toEqual(expected)
  })

  it.each(['escape', 'pointercancel', 'outside'] as const)('cancels a drag without saving when cancelled by %s', async (reason) => {
    mockSortableProviders()
    const root = await mountView()
    const original = providerOrder(root)
    const { handle, hitTest } = startProviderDrag(root, 'provider-1', 'provider-3')
    if (reason === 'escape') {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    } else if (reason === 'pointercancel') {
      window.dispatchEvent(pointerEvent('pointercancel', 100, 200))
    } else {
      hitTest.mockReturnValue(null)
    }
    await dropProvider(handle)

    expect(providerOrder(root)).toEqual(original)
    expect(localStorage.getItem('aether-provider-display-order')).toBeNull()
    expect(workspace.updateDraftConfig).not.toHaveBeenCalled()
    expect(workspace.updatePriorityPolicy).not.toHaveBeenCalled()
    expect(root.querySelector('.opacity-40')).toBeNull()
    expect(root.querySelector('[data-provider-detail]')).toBeNull()
  })

  it('does not reorder on a handle click or a secondary mouse button', async () => {
    mockSortableProviders()
    const root = await mountView()
    const original = providerOrder(root)
    const handle = providerElements(root)[0]!.querySelector<HTMLButtonElement>('[data-provider-drag-handle]')!
    handle.dispatchEvent(pointerEvent('pointerdown', 40, 100))
    window.dispatchEvent(pointerEvent('pointermove', 42, 101))
    window.dispatchEvent(pointerEvent('pointerup', 42, 101))
    handle.click()
    handle.dispatchEvent(pointerEvent('pointerdown', 40, 100, { button: 2 }))
    window.dispatchEvent(pointerEvent('pointermove', 100, 200))
    window.dispatchEvent(pointerEvent('pointerup', 100, 200))
    await settle()

    expect(providerOrder(root)).toEqual(original)
    expect(root.querySelector('[data-provider-detail]')).toBeNull()
  })

  it('keeps hidden providers in the complete group order while dragging a filtered result', async () => {
    const providers = mockSortableProviders()
    providers[0]!.description = 'filtered'
    providers[2]!.description = 'filtered'
    const root = await mountView()
    const search = root.querySelector<HTMLInputElement>('#provider-search')!
    search.value = 'filtered'
    search.dispatchEvent(new Event('input', { bubbles: true }))
    await settle()
    expect(providerOrder(root)).toEqual(['provider-1', 'provider-3'])
    const { handle } = startProviderDrag(root, 'provider-1', 'provider-3')
    await dropProvider(handle)
    expect(providerOrder(root)).toEqual(['provider-3', 'provider-1'])
    findButton(root, '重置筛选').click()
    await settle()
    expect(providerOrder(root)).toEqual(['provider-2', 'provider-3', 'provider-1', 'provider-4'])
    expect(apiMocks.getProvidersSummary).toHaveBeenCalledTimes(1)
  })

  it('supports keyboard ordering and keeps focus on the moved handle', async () => {
    mockSortableProviders()
    const root = await mountView()
    const handle = providerElements(root)[0]!.querySelector<HTMLButtonElement>('[data-provider-drag-handle]')!
    handle.focus()
    handle.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, cancelable: true }))
    await settle()

    expect(providerOrder(root)).toEqual(['provider-2', 'provider-1', 'provider-3', 'provider-4'])
    expect(document.activeElement).toBe(handle)
    expect(root.querySelector('[role="status"]')?.textContent).toContain('调度顺序已调整')
    expect(root.querySelector('[data-provider-detail]')).toBeNull()

    handle.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true, cancelable: true }))
    await settle()
    expect(providerOrder(root)).toEqual(['provider-1', 'provider-2', 'provider-3', 'provider-4'])
  })

  it('keeps the saved ordering across local pages after reordering the current page', async () => {
    const providers = Array.from({ length: 12 }, (_, index) => createProvider({
      id: `provider-${index + 1}`,
      name: `Provider ${index + 1}`,
      provider_priority: (index + 1) * 10,
    }))
    apiMocks.getProvidersSummary.mockImplementation(async (query: { page?: number, page_size?: number } = {}) => {
      const page = query.page ?? 1
      const pageSize = query.page_size ?? 20
      return {
        items: providers.slice((page - 1) * pageSize, page * pageSize),
        total: providers.length,
      }
    })
    localStorage.setItem('provider-management-page-size', '10')
    const root = await mountView()

    const firstDrag = startProviderDrag(root, 'provider-2', 'provider-1')
    await dropProvider(firstDrag.handle)
    expect(providerOrder(root).slice(0, 2)).toEqual(['provider-2', 'provider-1'])

    const requestsBeforePaging = apiMocks.getProvidersSummary.mock.calls.length
    const secondPage = root.querySelector<HTMLButtonElement>('button[aria-label="第 2 页"]')!
    secondPage.click()
    await settle()
    expect(providerOrder(root)).toEqual(['provider-11', 'provider-12'])
    expect(apiMocks.getProvidersSummary).toHaveBeenCalledTimes(requestsBeforePaging)

    const firstPage = root.querySelector<HTMLButtonElement>('button[aria-label="第 1 页"]')!
    firstPage.click()
    await settle()
    expect(providerOrder(root).slice(0, 2)).toEqual(['provider-2', 'provider-1'])
    expect(workspace.updatePriorityPolicy).toHaveBeenCalledOnce()
  })

  it('ignores a legacy local display order in favor of the selected group priorities', async () => {
    mockSortableProviders()
    localStorage.setItem('aether-provider-display-order', JSON.stringify(['deleted-provider', 'provider-3', 'provider-1']))
    const root = await mountView()
    expect(providerOrder(root)).toEqual(['provider-1', 'provider-2', 'provider-3', 'provider-4'])
  })

  it('keeps the dragged provider first after switching to a smaller page size', async () => {
    const providers = Array.from({ length: 12 }, (_, index) => createProvider({
      id: `provider-${index + 1}`,
      name: `Provider ${index + 1}`,
      provider_priority: (index + 1) * 10,
    }))
    apiMocks.getProvidersSummary.mockImplementation(async (query: { page?: number, page_size?: number } = {}) => {
      const page = query.page ?? 1
      const pageSize = query.page_size ?? 20
      return {
        items: providers.slice((page - 1) * pageSize, page * pageSize),
        total: providers.length,
      }
    })

    localStorage.setItem('provider-management-page-size', '50')
    let root = await mountView()
    const { handle } = startProviderDrag(root, 'provider-12', 'provider-1')
    await dropProvider(handle)
    expect(providerOrder(root)[0]).toBe('provider-12')

    unmountView()
    localStorage.setItem('provider-management-page-size', '10')
    root = await mountView()

    expect(providerOrder(root)).toEqual([
      'provider-12',
      'provider-1', 'provider-2', 'provider-3', 'provider-4', 'provider-5',
      'provider-6', 'provider-7', 'provider-8', 'provider-9',
    ])
  })
})
