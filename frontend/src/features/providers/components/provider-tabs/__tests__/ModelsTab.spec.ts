import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, nextTick, reactive, ref, type App } from 'vue'

import ModelsTab from '../ModelsTab.vue'
import { createI18n } from '@/i18n'
import type { Model, ProviderWithEndpointsSummary } from '@/api/endpoints'
import type { EndpointAPIKey } from '@/api/endpoints/keys'

const keyMocks = vi.hoisted(() => ({ getProviderKeys: vi.fn() }))
const testMocks = vi.hoisted(() => ({ startTest: vi.fn() }))

const modelMocks = vi.hoisted(() => ({
  deleteModel: vi.fn(),
  updateModel: vi.fn(),
}))

const toastMocks = vi.hoisted(() => ({
  success: vi.fn(),
  error: vi.fn(),
}))

vi.mock('@/api/endpoints/models', () => modelMocks)
vi.mock('@/api/endpoints/keys', () => keyMocks)
vi.mock('@/composables/useToast', () => ({
  useToast: () => toastMocks,
}))
vi.mock('@/composables/useClipboard', () => ({
  useClipboard: () => ({ copyToClipboard: vi.fn() }),
}))
vi.mock('@/composables/useModelTest', () => ({
  useModelTest: () => ({
    testing: { value: false },
    dialogOpen: { value: false },
    testResult: { value: null },
    testMode: { value: 'global' },
    testTrace: { value: null },
    requestId: { value: null },
    resetState: vi.fn(),
    startTest: testMocks.startTest,
    stopPolling: vi.fn(),
  }),
}))
vi.mock('../ModelTestDialog.vue', () => ({
  default: defineComponent({
    name: 'ModelTestDialogStub',
    setup: () => () => null,
  }),
}))

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

function createProvider(id = 'provider-1'): ProviderWithEndpointsSummary {
  return {
    id,
    name: 'OpenAI Responses',
    provider_type: 'custom',
    provider_priority: 1,
    keep_priority_on_conversion: false,
    enable_format_conversion: false,
    is_active: true,
    total_endpoints: 1,
    active_endpoints: 1,
    total_keys: 1,
    active_keys: 1,
    total_models: 2,
    active_models: 2,
    global_model_ids: [],
    avg_health_score: 1,
    unhealthy_endpoints: 0,
    api_formats: ['openai:chat'],
    endpoint_health_details: [],
    ops_configured: false,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  }
}

function createModel(overrides: Partial<Model> = {}): Model {
  return {
    id: 'model-1',
    provider_id: 'provider-1',
    global_model_id: 'gm-1',
    provider_model_name: 'deepseek-v4-flash',
    global_model_name: 'deepseek-v4-flash',
    global_model_display_name: 'DeepSeek V4 Flash',
    is_active: true,
    is_available: true,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides,
  }
}

const sampleModels = [
  createModel(),
  createModel({
    id: 'model-2',
    provider_model_name: 'kimi-k3',
    global_model_name: 'kimi-k3',
    global_model_display_name: 'Kimi K3',
  }),
]

async function settle() {
  for (let index = 0; index < 5; index += 1) {
    await Promise.resolve()
    await nextTick()
  }
}

function mountTab(options?: {
  models?: Model[]
  provider?: ProviderWithEndpointsSummary
  endpoints?: InstanceType<typeof ModelsTab>['$props']['endpoints']
}) {
  const tabRef = ref<InstanceType<typeof ModelsTab> | null>(null)
  const root = document.createElement('div')
  document.body.appendChild(root)
  const state = reactive({
    models: options?.models ?? sampleModels,
    provider: options?.provider ?? createProvider(),
  })
  const onRefresh = vi.fn()
  const onBatchAssign = vi.fn()
  const onEditModel = vi.fn()
  const app = createApp(defineComponent({
    setup() {
      return () => h(ModelsTab, {
        ref: tabRef,
        provider: state.provider,
        models: state.models,
        endpoints: options?.endpoints ?? [],
        onRefresh,
        onBatchAssign,
        onEditModel,
      })
    },
  }))
  app.use(createI18n())
  app.mount(root)
  mountedApps.push({ app, root })
  const testState = (tabRef.value!.$ as unknown as {
    setupState: {
      testModelConnection: (model: Model) => Promise<void>
      handleSelectTestEndpoint: (id: string) => void
      handleSelectTestKeyIds: (ids: string[]) => void
      handleStartPendingTest: () => Promise<void>
      selectedTestEndpoint: { id: string } | null
      testKeyOptions: Array<{ value: string }>
    }
  }).setupState
  return { root, state, onRefresh, onBatchAssign, onEditModel, testState }
}

beforeEach(() => {
  class ResizeObserverStub {
    observe() {}
    unobserve() {}
    disconnect() {}
  }
  vi.stubGlobal('ResizeObserver', ResizeObserverStub)
  keyMocks.getProviderKeys.mockReset()
  keyMocks.getProviderKeys.mockResolvedValue([])
  testMocks.startTest.mockReset()
  modelMocks.deleteModel.mockReset()
  modelMocks.deleteModel.mockResolvedValue({ message: 'ok' })
  modelMocks.updateModel.mockReset()
  toastMocks.success.mockReset()
  toastMocks.error.mockReset()
})

describe('ModelsTab model tests', () => {
  const endpoints = [
    { id: 'disabled-chat', api_format: 'openai:chat', is_active: false, active_keys: 0 },
    { id: 'active-chat', api_format: 'openai:chat', is_active: true, active_keys: 1 },
    { id: 'unsupported', api_format: 'openai:realtime', is_active: false, active_keys: 1 },
  ] as InstanceType<typeof ModelsTab>['$props']['endpoints']

  it('uses a selected key even when the provider and endpoint are disabled', async () => {
    keyMocks.getProviderKeys.mockResolvedValue([
      { id: 'chat-key', name: 'Chat Key', is_active: true, api_formats: ['openai:chat'], internal_priority: 0 },
      { id: 'disabled-key', is_active: false, api_formats: ['openai:chat'], internal_priority: 1 },
      { id: 'claude-key', is_active: true, api_formats: ['claude:messages'], internal_priority: 2 },
    ] as EndpointAPIKey[])
    const { testState } = mountTab({
      provider: { ...createProvider(), is_active: false },
      endpoints: endpoints?.filter(endpoint => endpoint.is_active === false),
    })

    await testState.testModelConnection(createModel())
    await settle()

    expect(testState.selectedTestEndpoint?.id).toBe('disabled-chat')
    expect(testState.testKeyOptions.map(option => option.value)).toEqual(['chat-key'])
    testState.handleSelectTestEndpoint('unsupported')
    expect(testState.selectedTestEndpoint?.id).toBe('disabled-chat')
    testState.handleSelectTestKeyIds(['chat-key'])
    await testState.handleStartPendingTest()

    expect(toastMocks.error).not.toHaveBeenCalled()
    expect(testMocks.startTest).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({
      endpointId: 'disabled-chat',
      apiKeyIds: ['chat-key'],
    }))
  })

  it('defaults to an active endpoint and leaves key selection to normal scheduling', async () => {
    const { testState } = mountTab({ endpoints })

    await testState.testModelConnection(createModel())
    await settle()
    await testState.handleStartPendingTest()

    expect(testMocks.startTest).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({
      mode: 'global',
      endpointId: 'active-chat',
      apiKeyIds: [],
    }))
  })
})

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
  vi.unstubAllGlobals()
})

describe('ModelsTab model list', () => {
  it('shows models without selection or bulk delete controls', async () => {
    const { root } = mountTab()
    await settle()

    expect(root.querySelectorAll('tbody tr')).toHaveLength(2)
    expect(root.textContent).toContain('DeepSeek V4 Flash')
    expect(root.textContent).toContain('Kimi K3')
    expect(root.querySelector('input[type="checkbox"], [role="checkbox"]')).toBeNull()
    expect(root.querySelector('[data-testid="models-tab-select-all"]')).toBeNull()
    expect(root.querySelector('[data-testid="models-tab-delete-selected"]')).toBeNull()
    expect(root.textContent).not.toContain('已选')
    expect(root.textContent).not.toContain('删除选中')
  })

  it('opens model association from the header', async () => {
    const { root, onBatchAssign } = mountTab()
    await settle()

    const associateButton = Array.from(root.querySelectorAll('button'))
      .find(button => button.textContent?.includes('关联模型'))
    expect(associateButton).toBeTruthy()
    associateButton!.click()
    await settle()

    expect(onBatchAssign).toHaveBeenCalledTimes(1)
    expect(modelMocks.deleteModel).not.toHaveBeenCalled()
  })

  it('keeps the edit and active state actions for each model', async () => {
    const { root, onEditModel } = mountTab({ models: [createModel()] })
    await settle()

    const editButton = root.querySelector('button[title="编辑"]') as HTMLButtonElement
    const toggleButton = root.querySelector('button[title="点击停用"]') as HTMLButtonElement
    expect(root.querySelector('button[title="测试模型"]')).toBeTruthy()
    editButton.click()
    await settle()
    expect(onEditModel).toHaveBeenCalledTimes(1)
    expect(onEditModel.mock.calls[0][0].id).toBe('model-1')

    toggleButton.click()
    await settle()

    expect(modelMocks.updateModel).toHaveBeenCalledWith('provider-1', 'model-1', { is_active: false })
    expect(toastMocks.success).toHaveBeenCalledWith('模型已停用')
    expect(root.querySelector('button[title="点击启用"]')).toBeTruthy()
  })

  it('updates the visible rows when the provider model list refreshes', async () => {
    const { root, state } = mountTab()
    await settle()

    state.models = [sampleModels[1]]
    await settle()

    expect(root.querySelectorAll('tbody tr')).toHaveLength(1)
    expect(root.textContent).not.toContain('DeepSeek V4 Flash')
    expect(root.textContent).toContain('Kimi K3')
    expect(root.querySelector('input[type="checkbox"], [role="checkbox"]')).toBeNull()
  })
})
