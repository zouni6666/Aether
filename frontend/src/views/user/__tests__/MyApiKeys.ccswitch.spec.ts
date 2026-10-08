import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, nextTick, type App } from 'vue'

import MyApiKeys from '../MyApiKeys.vue'

const toastMock = vi.hoisted(() => ({
  success: vi.fn(),
  error: vi.fn(),
}))

const meApiMock = vi.hoisted(() => ({
  getApiKeys: vi.fn(),
  getRoutingGroups: vi.fn(),
  createApiKey: vi.fn(),
  getFullApiKey: vi.fn(),
  getClientConfig: vi.fn(),
  getAvailableModels: vi.fn(),
  createApiKeyInstallSession: vi.fn(),
  updateApiKey: vi.fn(),
  deleteApiKey: vi.fn(),
  toggleApiKey: vi.fn(),
}))

vi.mock('@/api/me', () => ({
  meApi: meApiMock,
}))

vi.mock('@/composables/useToast', () => ({
  useToast: () => toastMock,
}))

vi.mock('@/components/common', async () => {
  const { defineComponent, h } = await import('vue')

  return {
    LoadingState: defineComponent({
      props: { message: String },
      setup: props => () => h('div', props.message || 'loading'),
    }),
    EmptyState: defineComponent({
      props: { title: String, description: String, icon: [Object, Function] },
      setup: (props, { slots }) => () => h('div', [
        h('div', props.title || ''),
        h('div', props.description || ''),
        slots.actions?.(),
      ]),
    }),
    AlertDialog: defineComponent({
      emits: ['confirm', 'cancel'],
      setup: () => () => null,
    }),
  }
})

vi.mock('@/utils/logger', () => ({
  log: {
    error: vi.fn(),
    warn: vi.fn(),
    info: vi.fn(),
  },
}))

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

function apiKey(overrides: Record<string, unknown> = {}) {
  return {
    id: 'user-key-1',
    name: 'primary',
    key_display: 'sk-user...live',
    is_active: true,
    is_locked: false,
    created_at: '2026-05-29T00:00:00+00:00',
    total_requests: 0,
    total_cost_usd: 0,
    rate_limit: 0,
    concurrent_limit: 0,
    ip_rules: null,
    ...overrides,
  }
}

async function flushPromises() {
  await nextTick()
  await new Promise(resolve => setTimeout(resolve, 0))
  await nextTick()
}

async function mountMyApiKeys() {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp(MyApiKeys)
  app.mount(root)
  mountedApps.push({ app, root })
  await flushPromises()
  return root
}

beforeEach(() => {
  vi.clearAllMocks()
  meApiMock.getRoutingGroups.mockResolvedValue({ items: [], total: 0 })
  meApiMock.getClientConfig.mockResolvedValue({
    base_url: 'https://aether.example.com',
    site_name: 'Aether Local',
  })
  meApiMock.getAvailableModels.mockResolvedValue({
    models: [
      { id: 'gm-1', name: 'claude-haiku-4', display_name: 'Claude Haiku 4', is_active: true },
      { id: 'gm-2', name: 'claude-sonnet-4', display_name: 'Claude Sonnet 4', is_active: true },
      { id: 'gm-3', name: 'claude-opus-4', display_name: 'Claude Opus 4', is_active: true },
      { id: 'gm-4', name: 'gpt-5', display_name: 'GPT 5', is_active: true },
    ],
    total: 4,
  })
  meApiMock.createApiKeyInstallSession.mockResolvedValue({
    install_code: 'install-code',
    expires_at_unix_secs: 1,
    expires_in_seconds: 900,
    target_cli: 'claude_code',
    target_cli_label: 'Claude Code',
    target_system: 'linux',
    target_system_label: 'Linux',
    unix_command: 'curl install',
    powershell_command: 'irm install',
  })
})

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
  document.body.innerHTML = ''
})

describe('MyApiKeys CC Switch import', () => {
  it('opens the import dialog for an existing key without fetching the full key immediately', async () => {
    meApiMock.getApiKeys.mockResolvedValue([apiKey()])

    await mountMyApiKeys()
    document.querySelector<HTMLButtonElement>('[data-testid="ccswitch-open-user-key-1"]')?.click()
    await flushPromises()

    expect(meApiMock.getFullApiKey).not.toHaveBeenCalled()
    expect(document.body.textContent).toContain('导入到 CC Switch')
    expect(document.querySelector<HTMLInputElement>('[data-testid="ccswitch-provider-name"]')?.value).toBe('Aether Local')
    expect(document.querySelector<HTMLElement>('[data-testid="ccswitch-model-select-haiku"]')?.textContent).toContain('claude-haiku-4')
    expect(document.querySelector<HTMLElement>('[data-testid="ccswitch-model-select-sonnet"]')?.textContent).toContain('claude-sonnet-4')
    expect(document.querySelector<HTMLElement>('[data-testid="ccswitch-model-select-opus"]')?.textContent).toContain('claude-opus-4')
  })

  it('switches non-Claude targets to a single default model without changing the site provider name', async () => {
    meApiMock.getApiKeys.mockResolvedValue([apiKey()])

    await mountMyApiKeys()
    document.querySelector<HTMLButtonElement>('[data-testid="ccswitch-open-user-key-1"]')?.click()
    await flushPromises()
    document.querySelector<HTMLButtonElement>('[data-testid="ccswitch-target-codex"]')?.click()
    await flushPromises()

    expect(document.querySelector<HTMLInputElement>('[data-testid="ccswitch-provider-name"]')?.value).toBe('Aether Local')
    expect(document.querySelector<HTMLElement>('[data-testid="ccswitch-model-select-default"]')?.textContent).toContain('gpt-5')
    expect(document.querySelector<HTMLElement>('[data-testid="ccswitch-model-select-haiku"]')).toBeNull()
    expect(document.querySelector<HTMLElement>('[data-testid="ccswitch-model-select-sonnet"]')).toBeNull()
    expect(document.querySelector<HTMLElement>('[data-testid="ccswitch-model-select-opus"]')).toBeNull()
  })

  it('shows a specific message when an existing key cannot return full key material', async () => {
    meApiMock.getApiKeys.mockResolvedValue([apiKey()])
    meApiMock.getFullApiKey.mockRejectedValue({
      response: { data: { detail: '该密钥没有存储完整密钥信息' } },
    })

    await mountMyApiKeys()
    document.querySelector<HTMLButtonElement>('[data-testid="ccswitch-open-user-key-1"]')?.click()
    await flushPromises()
    document.querySelector<HTMLButtonElement>('[data-testid="ccswitch-confirm"]')?.click()
    await flushPromises()

    expect(toastMock.error).toHaveBeenCalledWith('该密钥缺少完整密钥信息，请重新创建 API Key')
  })

  it('does not fall back to an unlisted model when no available models are returned', async () => {
    meApiMock.getApiKeys.mockResolvedValue([apiKey()])
    meApiMock.getAvailableModels.mockResolvedValueOnce({
      models: [],
      total: 0,
    })

    await mountMyApiKeys()
    document.querySelector<HTMLButtonElement>('[data-testid="ccswitch-open-user-key-1"]')?.click()
    await flushPromises()

    const confirmButton = document.querySelector<HTMLButtonElement>('[data-testid="ccswitch-confirm"]')
    expect(document.body.textContent).toContain('暂无可用模型，请联系管理员配置可用模型后再导入。')
    expect(document.querySelector<HTMLElement>('[data-testid="ccswitch-model-select-haiku"]')?.textContent).not.toContain('gpt-5')
    expect(confirmButton?.disabled).toBe(true)

    confirmButton?.click()
    await flushPromises()

    expect(meApiMock.getFullApiKey).not.toHaveBeenCalled()
  })

  it('can open CC Switch import from the newly created key dialog without refetching the key', async () => {
    const createdKey = apiKey({ id: 'created-key-1', name: 'new key', key: 'sk-created-live' })
    meApiMock.getApiKeys.mockResolvedValueOnce([]).mockResolvedValue([createdKey])
    meApiMock.createApiKey.mockResolvedValue(createdKey)

    await mountMyApiKeys()
    document.querySelector<HTMLButtonElement>('[title="创建新 API Key"]')?.click()
    await flushPromises()

    const nameInput = document.querySelector<HTMLInputElement>('#key-name')
    nameInput!.value = 'new key'
    nameInput!.dispatchEvent(new Event('input', { bubbles: true }))
    await flushPromises()

    Array.from(document.querySelectorAll<HTMLButtonElement>('button'))
      .find(button => button.textContent?.trim() === '创建')
      ?.click()
    await flushPromises()

    expect(meApiMock.createApiKey).toHaveBeenCalledOnce()
    expect(meApiMock.createApiKey.mock.calls[0]?.[0]).toMatchObject({ name: 'new key' })
    expect(meApiMock.createApiKey.mock.calls[0]?.[0]).not.toHaveProperty('credential_kind')
    document.querySelector<HTMLButtonElement>('[data-testid="ccswitch-open-created-key"]')?.click()
    await flushPromises()

    expect(meApiMock.getFullApiKey).not.toHaveBeenCalled()
    expect(document.body.textContent).toContain('导入到 CC Switch')
  })

  it('sends the desired inactive state when disabling an active key', async () => {
    meApiMock.getApiKeys.mockResolvedValue([apiKey()])
    meApiMock.toggleApiKey.mockResolvedValue({
      id: 'user-key-1',
      is_active: false,
    })

    await mountMyApiKeys()
    document.querySelector<HTMLButtonElement>('[title="禁用"]')?.click()
    await flushPromises()

    expect(meApiMock.toggleApiKey).toHaveBeenCalledWith('user-key-1', false)
    expect(toastMock.success).toHaveBeenCalledWith('密钥已禁用')
  })
})

describe('MyApiKeys routing groups', () => {
  async function setName(name: string) {
    const input = document.querySelector<HTMLInputElement>('#key-name')!
    input.value = name
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await flushPromises()
  }

  async function save(label: '创建' | '保存') {
    const button = Array.from(document.querySelectorAll<HTMLButtonElement>('button'))
      .find(button => button.textContent?.trim() === label)
    expect(button).toBeDefined()
    button!.click()
    await flushPromises()
  }

  async function chooseGroup(label: string) {
    const trigger = document.querySelector<HTMLButtonElement>('#key-routing-group')!
    trigger.focus()
    trigger.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, cancelable: true }))
    await flushPromises()
    const option = Array.from(document.querySelectorAll<HTMLElement>('[role="listbox"] [role="option"]'))
      .find(option => option.textContent?.trim().startsWith(label))
    expect(option).toBeDefined()
    option!.focus()
    option!.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }))
    await flushPromises()
  }

  it('creates a key with an explicitly selected visible group', async () => {
    meApiMock.getApiKeys.mockResolvedValue([apiKey()])
    meApiMock.getRoutingGroups.mockResolvedValue({
      items: [{ id: 'economy', name: '经济分组', billing_multiplier: 0.5, is_default: false }], total: 1,
    })
    meApiMock.createApiKey.mockResolvedValue(apiKey({ id: 'new-key', key: 'sk-new' }))
    await mountMyApiKeys()
    document.querySelector<HTMLButtonElement>('[title="创建新 API Key"]')!.click()
    await flushPromises()
    await setName('test group')
    await chooseGroup('经济分组')
    expect(document.querySelector('#key-routing-group')?.textContent).toContain('0.5 倍')
    await save('创建')
    expect(meApiMock.createApiKey).toHaveBeenCalledWith(expect.objectContaining({ name: 'test group', routing_group_id: 'economy' }))
  })

  it('creates a key following default when no groups are available', async () => {
    meApiMock.getApiKeys.mockResolvedValue([apiKey()])
    meApiMock.createApiKey.mockResolvedValue(apiKey({ id: 'new-key', key: 'sk-new' }))
    await mountMyApiKeys()
    document.querySelector<HTMLButtonElement>('[title="创建新 API Key"]')!.click()
    await flushPromises()
    expect(document.body.textContent).toContain('暂无可选策略分组')
    await setName('default group')
    await save('创建')
    expect(meApiMock.createApiKey).toHaveBeenCalledWith(expect.objectContaining({ routing_group_id: null }))
  })

  it.each(['hidden', 'failed'] as const)('preserves an existing binding when group options are %s and the name is edited', async (state) => {
    meApiMock.getApiKeys.mockResolvedValue([apiKey({ routing_group_id: 'retained', routing_group_name: '原有分组' })])
    meApiMock.updateApiKey.mockResolvedValue(apiKey())
    if (state === 'failed') meApiMock.getRoutingGroups.mockRejectedValue(new Error('offline'))
    await mountMyApiKeys()
    expect(document.body.textContent?.match(/策略分组：原有分组/g)).toHaveLength(2)
    document.querySelector<HTMLButtonElement>('[title="编辑"]')!.click()
    await flushPromises()
    expect(document.querySelector('#key-routing-group')?.textContent).toContain('原有分组')
    if (state === 'failed') expect(document.body.textContent).toContain('策略分组加载失败')
    await setName('renamed')
    await save('保存')
    expect(meApiMock.updateApiKey).toHaveBeenCalledWith('user-key-1', expect.objectContaining({ name: 'renamed' }))
    expect(meApiMock.updateApiKey.mock.calls[0]?.[1]).not.toHaveProperty('routing_group_id')
  })

  it('allows clearing an unavailable binding to follow default explicitly', async () => {
    meApiMock.getApiKeys.mockResolvedValue([apiKey({ routing_group_id: 'retained', routing_group_name: '原有分组' })])
    meApiMock.updateApiKey.mockResolvedValue(apiKey())
    await mountMyApiKeys()
    document.querySelector<HTMLButtonElement>('[title="编辑"]')!.click()
    await flushPromises()
    await chooseGroup('跟随默认')
    await save('保存')
    expect(meApiMock.updateApiKey).toHaveBeenCalledWith('user-key-1', expect.objectContaining({ routing_group_id: null }))
  })

  it('can retry loading options without replacing the key binding', async () => {
    meApiMock.getApiKeys.mockResolvedValue([apiKey({ routing_group_id: 'retained', routing_group_name: '原有分组' })])
    meApiMock.getRoutingGroups.mockRejectedValueOnce(new Error('offline')).mockResolvedValue({
      items: [{ id: 'economy', name: '经济分组', billing_multiplier: 0.5, is_default: false }], total: 1,
    })
    meApiMock.updateApiKey.mockResolvedValue(apiKey())
    await mountMyApiKeys()
    document.querySelector<HTMLButtonElement>('[title="编辑"]')!.click()
    await flushPromises()
    Array.from(document.querySelectorAll<HTMLButtonElement>('button')).find(button => button.textContent?.trim() === '重试')!.click()
    await flushPromises()
    expect(document.querySelector('#key-routing-group')?.textContent).toContain('原有分组')
    await chooseGroup('经济分组')
    await save('保存')
    expect(meApiMock.updateApiKey).toHaveBeenCalledWith('user-key-1', expect.objectContaining({ routing_group_id: 'economy' }))
  })
})
