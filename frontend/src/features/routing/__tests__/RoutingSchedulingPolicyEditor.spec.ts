import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, h, nextTick, ref, type App, type VNode } from 'vue'
import type { GlobalModelResponse } from '@/api/global-models'
import RoutingSchedulingPolicyEditor from '../components/RoutingSchedulingPolicyEditor.vue'
import {
  createEmptyRoutingGroupConfig,
  getDefaultModelPolicy,
  getModelPolicy,
  getModelScheduling,
  setDefaultProviderPriorityOverrides,
  type RoutingModelPolicy,
  type RoutingGroupConfig,
} from '../utils/routingPolicy'
import { createSchedulingPolicy, readSchedulingPolicies, writeSchedulingPolicies } from '../utils/schedulingPolicies'

vi.mock('../components/RoutingPriorityPolicyEditor.vue', () => ({
  default: {
    props: ['config'],
    emits: ['update:config'],
    setup: (props: { config: RoutingGroupConfig }, { emit }: { emit: (event: string, config: RoutingGroupConfig) => void }) => () => h('button', {
      'aria-label': '调整排序',
      onClick: () => emit('update:config', setDefaultProviderPriorityOverrides(props.config, { provider: 7 })),
    }, '调整排序'),
  },
}))

const mounted: Array<{ app: App, root: HTMLElement }> = []

function mountEditor(
  initial = createEmptyRoutingGroupConfig(),
  layout: 'embedded' | 'config-only' = 'embedded',
  toolbarSlots: Record<string, () => VNode> = {},
  sidebar = false,
  initialSelection?: { id?: string | null; scope: 'all' | 'selected'; modelNames: string[] },
) {
  const config = ref(initial)
  const valid = ref(true)
  const disabled = ref(false)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const reload = vi.fn()
  const selection = vi.fn()
  const editor = ref<InstanceType<typeof RoutingSchedulingPolicyEditor> | null>(null)
  const models = ref(['a', 'b', 'c'].map(name => ({
    id: `id-${name}`, name: `model-${name}`, display_name: `模型 ${name.toUpperCase()}`,
  })) as GlobalModelResponse[])
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp({
    setup: () => () => h(RoutingSchedulingPolicyEditor, {
      ref: editor,
      layout,
      sidebar,
      initialSelection,
      config: config.value,
      disabled: disabled.value,
      globalModels: models.value,
      loadingModels: loading.value,
      modelsError: error.value,
      'onUpdate:config': (value: RoutingGroupConfig) => { config.value = value },
      onValidityChange: (value: boolean) => { valid.value = value },
      onReloadModels: reload,
      onSelectionChange: selection,
    }, toolbarSlots),
  })
  app.mount(root)
  mounted.push({ app, root })
  return { root, config, valid, disabled, loading, error, models, reload, selection, editor }
}

function control<T extends HTMLElement>(root: HTMLElement, label: string): T {
  const element = root.querySelector<T>(`[aria-label="${label}"]`)
  if (!element) throw new Error(`Missing control: ${label}`)
  return element
}

async function clickText(root: HTMLElement, text: string) {
  const element = [...root.querySelectorAll<HTMLButtonElement>('button')]
    .find(button => button.textContent?.trim() === text)
  if (!element) throw new Error(`Missing button: ${text}`)
  element.click()
  await flush()
}

async function select(root: HTMLElement, model: string) {
  const picker = await openModels(root)
  control<HTMLInputElement>(picker, `选择模型 ${model}`).click()
  await flush()
}

async function flush() {
  await nextTick()
  await new Promise(resolve => setTimeout(resolve, 0))
  await nextTick()
}

async function openModels(root: HTMLElement) {
  if (control(root, '全部模型').getAttribute('aria-pressed') === 'true') {
    await clickText(root, '区分模型')
  }
  const activeCard = root.querySelector('[aria-label^="选择调度配置 "][aria-pressed="true"]')?.closest('section')
  const edit = activeCard?.querySelector<HTMLButtonElement>('[aria-label="编辑模型"]')
  if (edit) {
    if (!document.querySelector('[aria-label="编辑适用模型"]')) {
      edit.click()
      await flush()
    }
    return control(document.body, '编辑适用模型')
  }
  if (root.querySelector('[aria-label="全局模型选择列表"]')) return root
  control<HTMLButtonElement>(root, '选择适用模型').click()
  await flush()
  return root
}

beforeEach(() => {
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

describe('RoutingSchedulingPolicyEditor', () => {
  it('opens legacy Key configurations as provider scheduling without exposing a mode switch', async () => {
    const initial = createEmptyRoutingGroupConfig()
    initial.default_policy.priority_mode = 'global_key'
    initial.model_policies = [{ ...getDefaultModelPolicy(initial), key_priority_overrides: { legacy: 3 } }]
    const { root, config, selection } = mountEditor(initial)
    expect(root.querySelector('[aria-label="调度优先级"]')).toBeNull()
    expect([...root.querySelectorAll('button')].some(button => button.textContent?.trim() === 'Key')).toBe(false)
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ priorityMode: 'provider' }))
    await clickText(root, '固定顺序')
    expect(config.value.default_policy.priority_mode).toBe('provider')
    expect(getDefaultModelPolicy(config.value).key_priority_overrides).toEqual({ legacy: 3 })
  })

  it('shows configuration without an inline ranking list and emits its all-model selection', async () => {
    const { root, selection, config, editor } = mountEditor(createEmptyRoutingGroupConfig(), 'config-only')
    expect(root.querySelector('[aria-label="调整排序"]')).toBeNull()
    expect(control(root, '调度范围')).toBeTruthy()
    expect(root.textContent).toContain('全局配置')
    expect(root.textContent).not.toContain('调度设置')
    expect(root.querySelector('h3')).toBeNull()
    const strategy = control(root, '调度策略')
    const scope = control(root, '调度范围')
    expect(scope.compareDocumentPosition(strategy) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({
      policy: expect.objectContaining({ provider_priority_overrides: {} }),
      scope: 'all', modelNames: [], priorityMode: 'provider', schedulingMode: 'cache_affinity',
    }))
    const policy: RoutingModelPolicy = { ...getDefaultModelPolicy(config.value), provider_priority_overrides: { provider: 4 } }
    editor.value!.updateSelectedPolicy(policy)
    await flush()
    expect(getDefaultModelPolicy(config.value).provider_priority_overrides).toEqual({ provider: 4 })
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ policy: expect.objectContaining({ provider_priority_overrides: { provider: 4 } }) }))
  })

  it.each([false, true])('composes group and save slots without duplicating or mixing actions (sidebar: %s)', async sidebar => {
    const saved = vi.fn()
    const initial = createEmptyRoutingGroupConfig()
    const { root, config } = mountEditor(initial, 'config-only', {
      'toolbar-leading': () => h('label', [
        '策略分组', h('select', { 'aria-label': '选择策略分组' }, [h('option', '默认分组')]),
      ]),
      'toolbar-actions': () => h('button', { onClick: saved, 'aria-label': '保存分组配置' }, '保存'),
    }, sidebar)
    const leading = control(root, '选择策略分组')
    const strategy = control(root, '调度策略')
    const scope = control(root, '调度范围')
    const save = control<HTMLButtonElement>(root, '保存分组配置')
    expect(leading.compareDocumentPosition(scope) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
    expect(scope.compareDocumentPosition(strategy) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
    expect(scope.compareDocumentPosition(save) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
    expect(root.querySelectorAll('[aria-label="保存分组配置"]')).toHaveLength(1)
    await clickText(root, '固定顺序')
    expect(config.value.default_policy.scheduling_mode).toBe('fixed_order')
    expect(saved).not.toHaveBeenCalled()
    if (sidebar) {
      await select(root, 'model-a')
      const modelEditor = control(root, '当前配置的适用模型')
      expect(modelEditor.compareDocumentPosition(save) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
      expect(root.querySelectorAll('[aria-label="保存分组配置"]')).toHaveLength(1)
      expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('fixed_order')
    }
    save.click()
    expect(saved).toHaveBeenCalledOnce()
  })

  it('keeps strategy explanations beside the control and updates them with the current selection', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const first = { ...createSchedulingPolicy(initial), models: ['model-a'], schedulingMode: 'fixed_order' as const }
    const second = { ...createSchedulingPolicy(initial), models: ['model-b'], schedulingMode: 'load_balance' as const }
    const { root } = mountEditor(writeSchedulingPolicies(initial, [first, second]), 'config-only')
    expect(control(root, '调度策略说明').getAttribute('title')).toContain('固定顺序：')
    control<HTMLButtonElement>(root, '选择调度配置 2').click()
    await flush()
    expect(control(root, '调度策略说明').getAttribute('title')).toContain('负载均衡：')
    expect(control(root, '选择调度配置 2').textContent).toContain('模型 B')
    await clickText(root, '缓存亲和')
    expect(control(root, '调度策略说明').getAttribute('title')).toContain('缓存亲和：')
    expect(control(root, '选择调度配置 2').textContent).toContain('缓存亲和')
    expect(control(root, '选择调度配置 1').textContent).toContain('固定顺序')
  })

  it('selects a legacy default entry without showing a model picker or changing explicit model policies', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const selected = { ...createSchedulingPolicy(initial), models: ['model-a'], schedulingMode: 'fixed_order' as const }
    const fallback = { ...createSchedulingPolicy(initial, 'all'), schedulingMode: 'cache_affinity' as const }
    const { root, config, selection } = mountEditor(writeSchedulingPolicies(initial, [selected, fallback]), 'config-only')
    control<HTMLButtonElement>(root, '选择调度配置 2').click()
    await flush()
    expect(control(root, '调度配置 2').querySelector('[aria-label="编辑模型"]')).toBeNull()
    expect(control(root, '当前配置的适用模型').textContent).toContain('未单独指定的模型')
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ scope: 'all', modelNames: [], schedulingMode: 'cache_affinity' }))
    await clickText(root, '负载均衡')
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('fixed_order')
    expect(getModelScheduling(config.value, 'model-b').scheduling_mode).toBe('load_balance')
    control<HTMLButtonElement>(root, '选择调度配置 1').click()
    await flush()
    expect(root.querySelectorAll('[aria-label="编辑模型"]')).toHaveLength(1)
    expect(control(root, '已配置模型').textContent).toContain('模型 A')
  })

  it('expands only the selected model editor and keeps every shared ranking attached to its configuration', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const first = { ...createSchedulingPolicy(initial), models: ['model-a', 'model-c'], schedulingMode: 'fixed_order' as const }
    const second = { ...createSchedulingPolicy(initial), models: ['model-b'], schedulingMode: 'load_balance' as const }
    const { root, config, selection, editor } = mountEditor(writeSchedulingPolicies(initial, [first, second]), 'config-only')
    expect(root.querySelectorAll('[aria-label="编辑模型"]')).toHaveLength(2)
    for (const index of [1, 2]) {
      const card = control(root, `调度配置 ${index}`)
      const edit = control<HTMLButtonElement>(card, '编辑模型')
      expect(edit.textContent?.trim()).toBe('')
      expect(edit.querySelector('svg')).not.toBeNull()
      expect(edit.compareDocumentPosition(control(card, `删除调度配置 ${index}`)) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
    }
    expect(control(root, '调度配置 2').querySelector('[aria-label="当前配置的适用模型"]')).toBeNull()
    expect(document.querySelector('[aria-label="全局模型选择列表"]')).toBeNull()
    expect(control(root, '已配置模型').querySelector('[title="model-a"]')?.textContent).toBe('模型 A')
    expect(control(root, '已配置模型').querySelector('[title="model-c"]')?.textContent).toBe('模型 C')
    expect(control(root, '选择调度配置 1').textContent).toContain('模型 A +1')
    expect(root.querySelector('[aria-label="调整排序"]')).toBeNull()
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ id: first.id, modelNames: ['model-a', 'model-c'], schedulingMode: 'fixed_order' }))
    const secondEdit = control<HTMLButtonElement>(control(root, '调度配置 2'), '编辑模型')
    secondEdit.click()
    await flush()
    expect(control(root, '选择调度配置 2').getAttribute('aria-expanded')).toBe('true')
    expect(control(root, '选择调度配置 2').getAttribute('aria-pressed')).toBe('true')
    expect(control(root, '调度配置 1').querySelector('[aria-label="当前配置的适用模型"]')).toBeNull()
    expect(control<HTMLInputElement>(control(document.body, '编辑适用模型'), '选择模型 model-b').checked).toBe(true)
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ id: second.id, modelNames: ['model-b'], schedulingMode: 'load_balance' }))
    control<HTMLButtonElement>(control(document.body, '编辑适用模型'), '完成选择').click()
    await flush()
    expect(document.querySelector('[aria-label="编辑适用模型"]')).toBeNull()
    await vi.waitFor(() => expect(document.activeElement).toBe(secondEdit))
    const policy = { ...getDefaultModelPolicy(initial), provider_priority_overrides: { provider: 6 } }
    editor.value!.updateSelectedPolicy(policy)
    await flush()
    expect(getModelPolicy(config.value, 'model-b').provider_priority_overrides).toEqual({ provider: 6 })
    expect(getModelPolicy(config.value, 'model-a').provider_priority_overrides).toEqual({})
    control<HTMLButtonElement>(root, '选择调度配置 1').click()
    await flush()
    editor.value!.updateSelectedPolicy({ ...policy, provider_priority_overrides: { shared: 3 } })
    await flush()
    expect(getModelPolicy(config.value, 'model-a').provider_priority_overrides).toEqual({ shared: 3 })
    expect(getModelPolicy(config.value, 'model-c').provider_priority_overrides).toEqual({ shared: 3 })
    expect(root.querySelectorAll('[aria-label="编辑模型"]')).toHaveLength(2)
    control<HTMLButtonElement>(root, '删除调度配置 1').click()
    await flush()
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ id: second.id, modelNames: ['model-b'] }))
  })

  it('reports an unfinished model selection as unavailable for directory priority edits', async () => {
    const { root, selection, editor, config } = mountEditor(createEmptyRoutingGroupConfig(), 'config-only')
    await clickText(root, '区分模型')
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ policy: null, scope: 'selected', modelNames: [] }))
    editor.value!.updateSelectedPolicy({ ...getDefaultModelPolicy(config.value), provider_priority_overrides: { unexpected: 7 } })
    await flush()
    expect(config.value.model_policies.some(policy => policy.provider_priority_overrides.unexpected === 7)).toBe(false)
    await select(root, 'model-a')
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ policy: expect.any(Object), scope: 'selected', modelNames: ['model-a'] }))
  })

  it('collapses the selected card without changing the directory target and expands switched or new configurations', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const first = { ...createSchedulingPolicy(initial), models: ['model-a'], schedulingMode: 'fixed_order' as const }
    const second = { ...createSchedulingPolicy(initial), models: ['model-b'], schedulingMode: 'load_balance' as const }
    const { root, selection, config, editor } = mountEditor(writeSchedulingPolicies(initial, [first, second]), 'config-only')
    const firstButton = control<HTMLButtonElement>(root, '选择调度配置 1')
    expect(firstButton.getAttribute('aria-expanded')).toBe('true')
    await openModels(root)
    selection.mockClear()
    firstButton.click()
    await flush()
    expect(firstButton.getAttribute('aria-expanded')).toBe('false')
    expect(firstButton.getAttribute('aria-pressed')).toBe('true')
    expect(document.querySelector('[aria-label="编辑适用模型"]')).toBeNull()
    expect(root.querySelectorAll('[aria-label="编辑模型"]')).toHaveLength(2)
    expect(root.querySelector('[aria-label="调度策略"]')).toBeNull()
    expect(selection).not.toHaveBeenCalled()
    editor.value!.updateSelectedPolicy({ ...getDefaultModelPolicy(initial), provider_priority_overrides: { provider: 8 } })
    await flush()
    expect(getModelPolicy(config.value, 'model-a').provider_priority_overrides).toEqual({ provider: 8 })
    expect(getModelPolicy(config.value, 'model-b').provider_priority_overrides).toEqual({})
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('fixed_order')
    expect(root.querySelector('[aria-label="全局模型选择列表"]')).toBeNull()
    expect(root.querySelector('[aria-label="调度策略"]')).toBeNull()
    firstButton.click()
    await flush()
    expect(firstButton.getAttribute('aria-expanded')).toBe('true')
    expect(control(root, '调度配置 1').contains(control(root, '编辑模型'))).toBe(true)
    expect(control(root, '调度配置 1').contains(control(root, '调度策略'))).toBe(true)
    firstButton.click()
    await flush()
    control<HTMLButtonElement>(root, '选择调度配置 2').click()
    await flush()
    expect(control(root, '选择调度配置 2').getAttribute('aria-expanded')).toBe('true')
    expect(control(root, '选择调度配置 2').getAttribute('aria-pressed')).toBe('true')
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ id: second.id, modelNames: ['model-b'] }))
    control<HTMLButtonElement>(root, '添加调度配置').click()
    await flush()
    expect(control(root, '选择调度配置 3').getAttribute('aria-expanded')).toBe('true')
    expect(control(root, '选择调度配置 3').getAttribute('aria-pressed')).toBe('true')
    expect(control(control(root, '调度配置 3'), '编辑模型')).toBeTruthy()
    expect(control(root, '当前配置的适用模型').textContent).toContain('请选择适用模型')
    expect(document.querySelector('[aria-label="全局模型选择列表"]')).toBeNull()
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ policy: null, modelNames: [] }))
  })

  it('restores the chosen configuration after remount, including regenerated legacy default IDs', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const first = { ...createSchedulingPolicy(initial), models: ['model-a'], schedulingMode: 'fixed_order' as const }
    const second = { ...createSchedulingPolicy(initial), models: ['model-b', 'model-c'], schedulingMode: 'load_balance' as const }
    const saved = writeSchedulingPolicies(initial, [first, second])
    const { root, selection } = mountEditor(saved, 'config-only', {}, false, {
      id: 'previous-generated-id', scope: 'selected', modelNames: ['model-c', 'model-b'],
    })
    expect(control(root, '选择调度配置 2').getAttribute('aria-pressed')).toBe('true')
    expect(control(control(root, '调度配置 2'), '编辑模型')).toBeTruthy()
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ modelNames: ['model-b', 'model-c'] }))

    const legacy = writeSchedulingPolicies(initial, [first, createSchedulingPolicy(initial, 'all')])
    const restored = mountEditor(legacy, 'config-only', {}, false, { id: 'old-default-id', scope: 'all', modelNames: [] })
    expect(control(restored.root, '选择调度配置 2').getAttribute('aria-pressed')).toBe('true')
    expect(control(restored.root, '当前配置的适用模型').textContent).toContain('未单独指定的模型')
    expect(restored.selection).toHaveBeenLastCalledWith(expect.objectContaining({ scope: 'all', modelNames: [] }))
  })

  it('edits scheduling after model selection inside the selected card without changing other configurations', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const first = { ...createSchedulingPolicy(initial), models: ['model-a'], schedulingMode: 'fixed_order' as const }
    const second = { ...createSchedulingPolicy(initial), models: ['model-b'], schedulingMode: 'load_balance' as const }
    const { root, selection, config } = mountEditor(writeSchedulingPolicies(initial, [first, second]), 'config-only')
    const firstCard = control<HTMLElement>(root, '调度配置 1')
    const secondCard = control<HTMLElement>(root, '调度配置 2')
    expect(firstCard.contains(control(root, '调度策略'))).toBe(true)
    expect(secondCard.querySelector('[aria-label="调度策略"]')).toBeNull()
    expect(control(firstCard, '当前配置的适用模型').compareDocumentPosition(control(firstCard, '调度策略')) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
    control<HTMLButtonElement>(secondCard, '选择调度配置 2').click()
    await flush()
    expect(firstCard.querySelector('[aria-label="调度策略"]')).toBeNull()
    const secondStrategy = control(secondCard, '调度策略')
    expect(control(secondCard, '当前配置的适用模型').compareDocumentPosition(secondStrategy) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
    expect([...secondStrategy.querySelectorAll('button')].find(button => button.textContent?.trim() === '负载均衡')?.getAttribute('aria-pressed')).toBe('true')
    await clickText(secondStrategy, '缓存亲和')
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ id: second.id, modelNames: ['model-b'], priorityMode: 'provider' }))
    control<HTMLButtonElement>(firstCard, '选择调度配置 1').click()
    await flush()
    await clickText(control(firstCard, '调度策略'), '负载均衡')
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ id: first.id, modelNames: ['model-a'], schedulingMode: 'load_balance' }))
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('load_balance')
    expect(getModelScheduling(config.value, 'model-b').scheduling_mode).toBe('cache_affinity')
    control<HTMLButtonElement>(secondCard, '选择调度配置 2').click()
    await flush()
    await select(root, 'model-c')
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ id: second.id, modelNames: ['model-b', 'model-c'] }))
  })

  it('keeps live model edits when finishing or escaping and isolates the next configuration picker', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const first = { ...createSchedulingPolicy(initial), models: ['model-a'], schedulingMode: 'fixed_order' as const }
    const second = { ...createSchedulingPolicy(initial), models: ['model-b'], schedulingMode: 'load_balance' as const }
    const { root, config, selection } = mountEditor(writeSchedulingPolicies(initial, [first, second]), 'config-only', {}, true)
    const picker = await openModels(root)
    expect(picker.querySelector('[aria-label="清空已选"]')).toBeNull()
    expect(picker.textContent).not.toMatch(/已选\s*\d/)
    const search = control<HTMLInputElement>(picker, '搜索全局模型')
    search.value = '模型 C'
    search.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    control<HTMLInputElement>(picker, '选择模型 model-c').click()
    await flush()
    expect(getModelScheduling(config.value, 'model-c').scheduling_mode).toBe('fixed_order')
    expect(getModelScheduling(config.value, 'model-b').scheduling_mode).toBe('load_balance')
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ id: first.id, modelNames: ['model-a', 'model-c'] }))
    control<HTMLButtonElement>(picker, '完成选择').click()
    await flush()
    expect(document.querySelector('[aria-label="编辑适用模型"]')).toBeNull()
    expect(readSchedulingPolicies(config.value)[0].models).toEqual(['model-a', 'model-c'])
    expect(control(root, '已配置模型').textContent).toContain('模型 C')
    const reopened = await openModels(root)
    expect(control<HTMLInputElement>(reopened, '选择模型 model-c').checked).toBe(true)
    control<HTMLButtonElement>(root, '选择调度配置 2').click()
    await flush()
    expect(document.querySelector('[aria-label="编辑适用模型"]')).toBeNull()
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ id: second.id, modelNames: ['model-b'] }))
    const nextPicker = await openModels(root)
    expect(document.querySelectorAll('[aria-label="搜索全局模型"]')).toHaveLength(1)
    const nextSearch = control<HTMLInputElement>(nextPicker, '搜索全局模型')
    expect(nextSearch.value).toBe('')
    expect(control<HTMLInputElement>(nextPicker, '选择模型 model-b').checked).toBe(true)
    nextSearch.value = 'model-c'
    nextSearch.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    expect(control<HTMLInputElement>(nextPicker, '选择模型 model-c').disabled).toBe(true)
    expect(control(nextPicker, '全局模型选择列表').textContent).toContain('已用于配置 1')
    nextSearch.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await flush()
    expect(document.querySelector('[aria-label="编辑适用模型"]')).toBeNull()
    expect(readSchedulingPolicies(config.value).map(entry => entry.models)).toEqual([['model-a', 'model-c'], ['model-b']])
    expect(control(root, '选择调度配置 2').getAttribute('aria-pressed')).toBe('true')
    await vi.waitFor(() => expect(document.activeElement).toBe(control(control(root, '调度配置 2'), '编辑模型')))
    const add = control<HTMLButtonElement>(root, '添加调度配置')
    for (const card of root.querySelectorAll('section[aria-label^="调度配置 "]')) {
      expect(card.compareDocumentPosition(add) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
    }
    expect(document.querySelector('[aria-label="完成选择"]')).toBeNull()
  })

  it('keeps live model edits when Escape or saving closes the popover', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const first = { ...createSchedulingPolicy(initial), models: ['model-a'] }
    const { root, config, disabled, selection } = mountEditor(writeSchedulingPolicies(initial, [first]), 'config-only', {}, true)
    await select(root, 'model-c')
    const saved = JSON.stringify(config.value)
    control(document.body, '搜索全局模型').dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await flush()
    expect(document.querySelector('[aria-label="编辑适用模型"]')).toBeNull()
    expect(JSON.stringify(config.value)).toBe(saved)
    await openModels(root)
    selection.mockClear()
    disabled.value = true
    await flush()
    expect(document.querySelector('[aria-label="编辑适用模型"]')).toBeNull()
    expect(control<HTMLButtonElement>(root, '编辑模型').disabled).toBe(true)
    expect(control(root, '选择调度配置 1').getAttribute('aria-pressed')).toBe('true')
    expect(JSON.stringify(config.value)).toBe(saved)
    expect(selection).not.toHaveBeenCalled()
    disabled.value = false
    await flush()
    expect(document.querySelector('[aria-label="编辑适用模型"]')).toBeNull()
    const reopened = await openModels(root)
    expect(control<HTMLInputElement>(reopened, '选择模型 model-a').checked).toBe(true)
    expect(control<HTMLInputElement>(reopened, '选择模型 model-c').checked).toBe(true)
  })

  it('keeps a strategy chosen before model selection and lets unfinished configurations choose their strategy', async () => {
    const { root, config, selection } = mountEditor(createEmptyRoutingGroupConfig(), 'config-only')
    await clickText(root, '固定顺序')
    await clickText(root, '区分模型')
    expect(selection).toHaveBeenLastCalledWith(expect.objectContaining({ policy: null, schedulingMode: 'fixed_order' }))
    await select(root, 'model-a')
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('fixed_order')
    control<HTMLButtonElement>(root, '添加调度配置').click()
    await flush()
    await clickText(root, '负载均衡')
    await select(root, 'model-b')
    expect(getModelScheduling(config.value, 'model-b').scheduling_mode).toBe('load_balance')
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('fixed_order')
    expect(root.querySelectorAll('[aria-label="调度策略"]')).toHaveLength(1)
  })

  it('starts with one all-model configuration and no model picker or add control', async () => {
    const { root, config, valid } = mountEditor()
    expect(control(root, '调度范围').getAttribute('role')).toBe('group')
    expect(control(root, '全部模型').getAttribute('aria-pressed')).toBe('true')
    expect(control(root, '区分模型').getAttribute('aria-pressed')).toBe('false')
    expect(root.querySelectorAll('section[aria-label^="调度配置 "]')).toHaveLength(1)
    expect(root.querySelector('[aria-label="选择适用模型"]')).toBeNull()
    expect(root.querySelector('[aria-label="添加调度配置"]')).toBeNull()
    expect(valid.value).toBe(true)
    await clickText(root, '负载均衡')
    expect(readSchedulingPolicies(config.value)).toHaveLength(1)
    expect(readSchedulingPolicies(config.value)[0]).toMatchObject({ scope: 'all', models: [] })
    expect(getModelScheduling(config.value, 'future-model').scheduling_mode).toBe('load_balance')
  })

  it('selects models first and then configures their shared scheduling and ranking', async () => {
    const { root, config, valid } = mountEditor()
    const focusedControl = control<HTMLButtonElement>(root, '区分模型')
    focusedControl.focus()
    await clickText(root, '区分模型')
    const picker = control<HTMLButtonElement>(root, '选择适用模型')
    expect(control<HTMLButtonElement>(root, '添加调度配置').disabled).toBe(true)
    expect(valid.value).toBe(false)
    const list = control(root, '全局模型选择列表')
    expect(list.getAttribute('role')).toBe('region')
    expect(list.id).toBeTruthy()
    expect(picker.getAttribute('aria-controls')).toBe(list.id)
    expect(picker.getAttribute('aria-expanded')).toBe('true')
    expect(document.querySelector('[role="dialog"][aria-label="选择适用模型"]')).toBeNull()
    expect(document.activeElement).toBe(focusedControl)
    expect(document.querySelector('button[aria-label="指定全局模型"]')).toBeNull()
    expect(control(root, '全部模型').getAttribute('aria-pressed')).toBe('false')
    expect(list.querySelector('[aria-label="全部模型"]')).toBeNull()
    expect(control<HTMLInputElement>(root, '选择模型 model-a').checked).toBe(false)
    await select(root, 'model-a')
    await select(root, 'model-b')
    await clickText(root, '完成选择')
    expect(picker.getAttribute('aria-expanded')).toBe('false')
    expect(root.querySelector('[aria-label="全局模型选择列表"]')).toBeNull()
    expect(document.activeElement).toBe(picker)
    await clickText(root, '固定顺序')
    control<HTMLButtonElement>(root, '调整排序').click()
    await nextTick()
    expect(valid.value).toBe(true)
    expect(readSchedulingPolicies(config.value)).toHaveLength(1)
    for (const model of ['model-a', 'model-b']) {
      expect(getModelScheduling(config.value, model)).toMatchObject({ priority_mode: 'provider', scheduling_mode: 'fixed_order' })
      expect(getModelPolicy(config.value, model).provider_priority_overrides).toEqual({ provider: 7 })
    }
    expect(getModelScheduling(config.value, 'model-c')).toMatchObject({ priority_mode: 'provider', scheduling_mode: 'cache_affinity' })
    expect(getModelPolicy(config.value, '*').provider_priority_overrides).toEqual({})
    expect(control<HTMLButtonElement>(root, '添加调度配置').disabled).toBe(false)
    expect([...root.querySelectorAll('h4')].map(heading => heading.textContent?.trim())).toEqual(['适用模型', '调度设置'])
  })

  it('keeps the inline picker open while editing scheduling outside it', async () => {
    const { root, config } = mountEditor()
    await openModels(root)
    await select(root, 'model-a')
    const list = control(root, '全局模型选择列表')
    const schedulingButton = [...root.querySelectorAll<HTMLButtonElement>('button')]
      .find(button => button.textContent?.trim() === '负载均衡')!
    schedulingButton.focus()
    schedulingButton.click()
    await flush()
    expect(control(root, '全局模型选择列表')).toBe(list)
    expect(control(root, '选择适用模型').getAttribute('aria-expanded')).toBe('true')
    expect(document.activeElement).toBe(schedulingButton)
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('load_balance')
  })

  it('expands a newly mounted empty configuration without moving focus', async () => {
    const { root, valid } = mountEditor()
    await select(root, 'model-a')
    await clickText(root, '完成选择')
    const focusedControl = root.appendChild(document.createElement('button'))
    focusedControl.focus()
    control<HTMLButtonElement>(root, '添加调度配置').click()
    await flush()
    expect(valid.value).toBe(false)
    expect(control(root, '选择适用模型').getAttribute('aria-expanded')).toBe('true')
    expect(control(root, '全局模型选择列表').getAttribute('role')).toBe('region')
    expect(document.activeElement).toBe(focusedControl)
  })

  it('selects or clears only matching search results, keeping hidden selections intact', async () => {
    const { root, config } = mountEditor()
    await openModels(root)
    await select(root, 'model-a')
    const search = control<HTMLInputElement>(root, '搜索全局模型')
    search.value = '模型 B'
    search.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    control<HTMLButtonElement>(root, '全选搜索结果').click()
    await flush()
    expect(readSchedulingPolicies(config.value)[0].models).toEqual(['model-a', 'model-b'])
    control<HTMLButtonElement>(root, '全选搜索结果').click()
    await flush()
    expect(readSchedulingPolicies(config.value)[0].models).toEqual(['model-a'])
  })

  it('keeps list order stable while selecting models and displays their names when collapsed', async () => {
    const { root } = mountEditor()
    await openModels(root)
    const names = () => [...document.querySelectorAll<HTMLInputElement>('[aria-label="全局模型选择列表"] input[aria-label^="选择模型 "]')]
      .map(input => input.getAttribute('aria-label'))
    const originalOrder = names()
    await select(root, 'model-c')
    await select(root, 'model-a')
    expect(names()).toEqual(originalOrder)
    expect(control<HTMLInputElement>(root, '选择模型 model-c').checked).toBe(true)
    expect(control<HTMLInputElement>(root, '选择模型 model-a').checked).toBe(true)
    await clickText(root, '完成选择')
    control<HTMLButtonElement>(root, '收起调度配置 1').click()
    await nextTick()
    expect(control<HTMLButtonElement>(root, '展开调度配置 1').textContent).toContain('模型 C、模型 A')
  })

  it('shows the selected value in the form field and keeps model checkboxes in sync', async () => {
    const { root } = mountEditor()
    await openModels(root)
    const picker = control<HTMLButtonElement>(root, '选择适用模型')
    expect(picker.textContent).toContain('请选择全局模型')
    await select(root, 'model-a')
    expect(picker.textContent).toContain('模型 A')
    await select(root, 'model-b')
    expect(picker.textContent).toContain('模型 A、模型 B')
    await select(root, 'model-c')
    expect(picker.textContent).toContain('已选择 3 个模型')
    await clickText(root, '完成选择')
    await select(root, 'model-b')
    expect(picker.textContent).toContain('模型 A、模型 C')
    expect(control<HTMLInputElement>(root, '选择模型 model-b').checked).toBe(false)
    await clickText(root, '清空已选')
    expect(picker.textContent).toContain('请选择全局模型')
    expect(control<HTMLInputElement>(root, '选择模型 model-a').checked).toBe(false)
    expect(control<HTMLInputElement>(root, '选择模型 model-c').checked).toBe(false)
  })

  it('closes with Escape without losing selections and restores focus to the picker', async () => {
    const { root, config } = mountEditor()
    await openModels(root)
    await select(root, 'model-a')
    const search = control<HTMLInputElement>(root, '搜索全局模型')
    search.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await flush()
    expect(document.querySelector('[aria-label="全局模型选择列表"]')).toBeNull()
    expect(readSchedulingPolicies(config.value)[0].models).toEqual(['model-a'])
    await vi.waitFor(() => expect(document.activeElement).toBe(control(root, '选择适用模型')))
  })

  it('clears selections without discarding scheduling settings', async () => {
    const { root, config, valid } = mountEditor()
    await openModels(root)
    await select(root, 'model-a')
    await clickText(root, '完成选择')
    await clickText(root, '固定顺序')
    await openModels(root)
    await clickText(root, '清空已选')
    expect(valid.value).toBe(false)
    expect(control(root, '选择适用模型').textContent).toContain('请选择全局模型')
    await select(root, 'model-b')
    expect(valid.value).toBe(true)
    expect(getModelScheduling(config.value, 'model-b').scheduling_mode).toBe('fixed_order')
  })

  it('closes the inline picker while saving and does not modify config on a no-op click', async () => {
    const { root, config, disabled } = mountEditor()
    const original = JSON.stringify(config.value)
    await clickText(root, '全部模型')
    await clickText(root, '缓存亲和')
    expect(JSON.stringify(config.value)).toBe(original)
    await openModels(root)
    await select(root, 'model-a')
    disabled.value = true
    await flush()
    expect(document.querySelector('[aria-label="全局模型选择列表"]')).toBeNull()
    expect(control<HTMLButtonElement>(root, '选择适用模型').disabled).toBe(true)
  })

  it('adds another strategy for remaining models and prevents duplicate assignment', async () => {
    const { root, config, valid } = mountEditor()
    await openModels(root)
    await select(root, 'model-a')
    control<HTMLButtonElement>(root, '添加调度配置').click()
    await flush()
    expect(valid.value).toBe(false)
    expect(document.querySelector('input[aria-label="选择模型 model-a"]')).toBeNull()
    const search = control<HTMLInputElement>(root, '搜索全局模型')
    search.value = 'model-a'
    search.dispatchEvent(new Event('input', { bubbles: true }))
    await nextTick()
    expect(control<HTMLInputElement>(root, '选择模型 model-a').disabled).toBe(true)
    search.value = ''
    search.dispatchEvent(new Event('input', { bubbles: true }))
    await nextTick()
    control<HTMLButtonElement>(root, '选择当前列表').click()
    await flush()
    await clickText(root, '完成选择')
    await clickText(root, '负载均衡')
    expect(valid.value).toBe(true)
    const entries = readSchedulingPolicies(config.value)
    expect(entries.map(entry => entry.models)).toEqual([['model-a'], ['model-b', 'model-c']])
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('cache_affinity')
    expect(getModelScheduling(config.value, 'model-b').scheduling_mode).toBe('load_balance')
    expect(control<HTMLButtonElement>(root, '添加调度配置').disabled).toBe(true)
  })

  it('inherits all-model settings on first entering model-specific mode and restores both drafts', async () => {
    const { root, config, valid } = mountEditor()
    await clickText(root, '负载均衡')
    control<HTMLButtonElement>(root, '调整排序').click()
    await flush()
    await clickText(root, '区分模型')
    expect(valid.value).toBe(false)
    expect(control(root, '选择适用模型').textContent).toContain('请选择全局模型')
    await select(root, 'model-a')
    expect(readSchedulingPolicies(config.value)).toHaveLength(1)
    expect(getModelScheduling(config.value, 'model-a')).toMatchObject({ priority_mode: 'provider', scheduling_mode: 'load_balance' })
    expect(getModelPolicy(config.value, 'model-a').provider_priority_overrides).toEqual({ provider: 7 })
    expect(getModelScheduling(config.value, 'new-model').scheduling_mode).toBe('cache_affinity')
    await clickText(root, '固定顺序')
    await clickText(root, '全部模型')
    expect(valid.value).toBe(true)
    expect(config.value.rules).toEqual([])
    expect(config.value.model_policies.map(policy => policy.model)).toEqual(['*'])
    expect(root.querySelector('[aria-label="添加调度配置"]')).toBeNull()
    expect(getModelScheduling(config.value, 'new-model').scheduling_mode).toBe('load_balance')
    await clickText(root, '区分模型')
    expect(readSchedulingPolicies(config.value)[0].models).toEqual(['model-a'])
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('fixed_order')
    expect(getModelScheduling(config.value, 'new-model').scheduling_mode).toBe('cache_affinity')
    await openModels(root)
    expect(control<HTMLInputElement>(root, '选择模型 model-a').checked).toBe(true)
  })

  it('uses the first model-specific configuration when first switching multiple entries to all models', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const first = { ...createSchedulingPolicy(initial), models: ['model-a'], schedulingMode: 'fixed_order' as const }
    const second = { ...createSchedulingPolicy(initial), models: ['model-b'], schedulingMode: 'load_balance' as const }
    const { root, config } = mountEditor(writeSchedulingPolicies(initial, [first, second]))
    await clickText(root, '全部模型')
    expect(readSchedulingPolicies(config.value)).toHaveLength(1)
    expect(readSchedulingPolicies(config.value)[0]).toMatchObject({ scope: 'all', models: [], schedulingMode: 'fixed_order' })
    expect(root.querySelectorAll('section[aria-label^="调度配置 "]')).toHaveLength(1)
    expect(config.value.rules).toEqual([])
    expect(config.value.model_policies.map(policy => policy.model)).toEqual(['*'])
    expect(getModelScheduling(config.value, 'model-b').scheduling_mode).toBe('fixed_order')
    await clickText(root, '区分模型')
    expect(readSchedulingPolicies(config.value).map(entry => entry.models)).toEqual([['model-a'], ['model-b']])
    expect(getModelScheduling(config.value, 'model-b').scheduling_mode).toBe('load_balance')
  })

  it('restores an unfinished model-specific draft after temporarily using all models', async () => {
    const { root, config, valid } = mountEditor()
    await select(root, 'model-a')
    await clickText(root, '固定顺序')
    control<HTMLButtonElement>(root, '添加调度配置').click()
    await flush()
    expect(valid.value).toBe(false)
    await clickText(root, '全部模型')
    expect(valid.value).toBe(true)
    await clickText(root, '区分模型')
    expect(valid.value).toBe(false)
    expect(root.querySelectorAll('section[aria-label^="调度配置 "]')).toHaveLength(2)
    expect(control<HTMLButtonElement>(root, '添加调度配置').disabled).toBe(true)
    control<HTMLButtonElement>(root, '展开调度配置 2').click()
    await flush()
    expect(control(root, '选择适用模型').textContent).toContain('请选择全局模型')
    await select(root, 'model-b')
    expect(valid.value).toBe(true)
    expect(readSchedulingPolicies(config.value).map(entry => entry.models)).toEqual([['model-a'], ['model-b']])
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('fixed_order')
  })

  it('offers all models independently of the catalog and automatically covers future models', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const entry = { ...createSchedulingPolicy(initial), models: ['model-a'], schedulingMode: 'load_balance' as const }
    const { root, config, models, loading, error, valid } = mountEditor(writeSchedulingPolicies(initial, [entry]))
    control<HTMLButtonElement>(root, '调整排序').click()
    await nextTick()
    models.value = []
    loading.value = true
    await flush()
    expect(control<HTMLButtonElement>(root, '全部模型').disabled).toBe(false)
    loading.value = false
    error.value = '模型加载失败'
    await flush()
    await clickText(root, '全部模型')
    expect(valid.value).toBe(true)
    expect(control(root, '全部模型').getAttribute('aria-pressed')).toBe('true')
    expect(root.querySelector('[aria-label="选择适用模型"]')).toBeNull()
    expect(root.querySelector('[aria-label="添加调度配置"]')).toBeNull()
    const saved = JSON.stringify(config.value)
    error.value = null
    models.value = [{ id: 'id-new', name: 'new-model', display_name: '新增模型' }] as GlobalModelResponse[]
    await flush()
    expect(JSON.stringify(config.value)).toBe(saved)
    expect(readSchedulingPolicies(JSON.parse(saved))[0]).toMatchObject({ scope: 'all', models: [] })
    expect(config.value.rules).toEqual([])
    expect(config.value.model_policies.map(policy => policy.model)).toEqual(['*'])
    expect(getModelScheduling(config.value, 'new-model').scheduling_mode).toBe('load_balance')
    expect(getDefaultModelPolicy(config.value).provider_priority_overrides).toEqual({ provider: 7 })
    expect(control(root, '全部模型').getAttribute('aria-pressed')).toBe('true')
    expect(root.querySelector('[aria-label="全局模型选择列表"]')).toBeNull()
  })

  it('keeps selecting the current list distinct from the all-model scope', async () => {
    const { root, config, models } = mountEditor()
    await openModels(root)
    control<HTMLButtonElement>(root, '选择当前列表').click()
    await flush()
    expect(control(root, '全部模型').getAttribute('aria-pressed')).toBe('false')
    expect(readSchedulingPolicies(config.value)[0]).toMatchObject({ scope: 'selected', models: ['model-a', 'model-b', 'model-c'] })
    await clickText(root, '完成选择')
    await clickText(root, '负载均衡')
    const saved = JSON.stringify(config.value)
    models.value.push({ id: 'id-new', name: 'new-model', display_name: '新增模型' } as GlobalModelResponse)
    await flush()
    expect(JSON.stringify(config.value)).toBe(saved)
    expect(getModelScheduling(config.value, 'new-model').scheduling_mode).toBe('cache_affinity')
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('load_balance')
    await openModels(root)
    expect(control<HTMLInputElement>(root, '选择模型 new-model').checked).toBe(false)
  })

  it('preserves a legacy all-model fallback and allows more model-specific configurations', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const selected = { ...createSchedulingPolicy(initial), models: ['model-a'], schedulingMode: 'fixed_order' as const }
    const fallback = { ...createSchedulingPolicy(initial, 'all'), schedulingMode: 'load_balance' as const }
    const { root, config } = mountEditor(writeSchedulingPolicies(initial, [selected, fallback]))
    const saved = JSON.stringify(config.value)
    expect(control(root, '区分模型').getAttribute('aria-pressed')).toBe('true')
    expect(control(root, '展开调度配置 2').textContent).toContain('默认配置')
    expect(control<HTMLButtonElement>(root, '添加调度配置').disabled).toBe(false)
    control<HTMLButtonElement>(root, '展开调度配置 2').click()
    await flush()
    expect(root.querySelector('[aria-label="选择适用模型"]')).toBeNull()
    expect(JSON.stringify(config.value)).toBe(saved)
    control<HTMLButtonElement>(root, '添加调度配置').click()
    await flush()
    await select(root, 'model-b')
    await clickText(root, '缓存亲和')
    expect(readSchedulingPolicies(config.value)).toHaveLength(3)
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('fixed_order')
    expect(getModelScheduling(config.value, 'model-b').scheduling_mode).toBe('cache_affinity')
    expect(getModelScheduling(config.value, 'future-model').scheduling_mode).toBe('load_balance')
  })

  it('uses the legacy fallback when switching mixed configurations to all models', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const selected = { ...createSchedulingPolicy(initial), models: ['model-a'], schedulingMode: 'fixed_order' as const }
    const fallback = { ...createSchedulingPolicy(initial, 'all'), schedulingMode: 'load_balance' as const }
    const { root, config } = mountEditor(writeSchedulingPolicies(initial, [selected, fallback]))
    await clickText(root, '全部模型')
    expect(readSchedulingPolicies(config.value)).toHaveLength(1)
    expect(config.value.rules).toEqual([])
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('load_balance')
    expect(getModelScheduling(config.value, 'model-b').scheduling_mode).toBe('load_balance')
    await clickText(root, '区分模型')
    expect(readSchedulingPolicies(config.value)).toHaveLength(2)
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('fixed_order')
    expect(getModelScheduling(config.value, 'model-b').scheduling_mode).toBe('load_balance')
  })

  it('keeps scope and ranking edits when moving between strategy cards', async () => {
    const { root, config } = mountEditor()
    await clickText(root, '固定顺序')
    await openModels(root)
    await select(root, 'model-a')
    control<HTMLButtonElement>(root, '添加调度配置').click()
    await nextTick()
    await select(root, 'model-b')
    control<HTMLButtonElement>(root, '展开调度配置 1').click()
    await nextTick()
    await select(root, 'model-c')
    control<HTMLButtonElement>(root, '调整排序').click()
    await nextTick()
    const entries = readSchedulingPolicies(config.value)
    expect(entries[0]).toMatchObject({ models: ['model-a', 'model-c'], schedulingMode: 'fixed_order' })
    expect(getModelPolicy(config.value, 'model-c').provider_priority_overrides).toEqual({ provider: 7 })
    expect(getModelPolicy(config.value, 'model-b').provider_priority_overrides).toEqual({})
  })

  it('releases models and removes rules when a strategy is deleted', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const first = { ...createSchedulingPolicy(initial), models: ['model-a'] }
    const second = { ...createSchedulingPolicy(initial), models: ['model-b'] }
    const { root, config } = mountEditor(writeSchedulingPolicies(initial, [first, second]))
    control<HTMLButtonElement>(root, '删除调度配置 2').click()
    await nextTick()
    expect(config.value.rules).toHaveLength(1)
    expect(config.value.model_policies.map(policy => policy.model)).toEqual(['model-a'])
    await openModels(root)
    expect(control<HTMLInputElement>(root, '选择模型 model-b').disabled).toBe(false)
    await select(root, 'model-b')
    expect(readSchedulingPolicies(config.value)[0].models).toEqual(['model-a', 'model-b'])
  })

  it('returns to all-model mode when deleting the last selected entry beside a legacy fallback', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const selected = { ...createSchedulingPolicy(initial), models: ['model-a'], schedulingMode: 'fixed_order' as const }
    const fallback = { ...createSchedulingPolicy(initial, 'all'), schedulingMode: 'load_balance' as const }
    const { root, config, valid } = mountEditor(writeSchedulingPolicies(initial, [selected, fallback]))
    control<HTMLButtonElement>(root, '删除调度配置 1').click()
    await flush()
    expect(valid.value).toBe(true)
    expect(control(root, '全部模型').getAttribute('aria-pressed')).toBe('true')
    expect(root.querySelector('[aria-label="添加调度配置"]')).toBeNull()
    expect(readSchedulingPolicies(config.value)).toHaveLength(1)
    expect(config.value.rules).toEqual([])
    expect(getModelScheduling(config.value, 'model-a').scheduling_mode).toBe('load_balance')
    await clickText(root, '区分模型')
    expect(valid.value).toBe(false)
    expect(root.querySelectorAll('section[aria-label^="调度配置 "]')).toHaveLength(1)
    expect(control<HTMLInputElement>(root, '选择模型 model-a').checked).toBe(false)
    await select(root, 'model-b')
    expect(getModelScheduling(config.value, 'model-b').scheduling_mode).toBe('load_balance')
  })

  it('searches global model names and display names without losing selected models', async () => {
    const { root, config } = mountEditor()
    await openModels(root)
    await select(root, 'model-a')
    const search = control<HTMLInputElement>(root, '搜索全局模型')
    search.value = '模型 B'
    search.dispatchEvent(new Event('input', { bubbles: true }))
    await nextTick()
    expect(root.querySelector('input[aria-label="选择模型 model-a"]')).toBeNull()
    await select(root, 'model-b')
    expect(readSchedulingPolicies(config.value)[0].models).toEqual(['model-a', 'model-b'])
    search.value = ''
    search.dispatchEvent(new Event('input', { bubbles: true }))
    await nextTick()
    expect(control<HTMLInputElement>(root, '选择模型 model-a').checked).toBe(true)
    await select(root, 'model-a')
    expect(readSchedulingPolicies(config.value)[0].models).toEqual(['model-b'])
  })

  it('retains group-wide failover changes when the shared ranking is edited', async () => {
    const { root, config } = mountEditor()
    config.value.default_policy.max_transfer_count = 9
    config.value.default_policy.cancel_on_client_disconnect = true
    await nextTick()
    await openModels(root)
    await select(root, 'model-a')
    control<HTMLButtonElement>(root, '调整排序').click()
    await nextTick()
    expect(config.value.default_policy.max_transfer_count).toBe(9)
    expect(config.value.default_policy.cancel_on_client_disconnect).toBe(true)
  })

  it('reports loading failures without clearing previously selected models', async () => {
    const initial = createEmptyRoutingGroupConfig()
    const entry = { ...createSchedulingPolicy(initial), models: ['removed-model'] }
    const { root, config, models, loading, error, reload, valid } = mountEditor(writeSchedulingPolicies(initial, [entry]))
    models.value = []
    loading.value = true
    await nextTick()
    await openModels(root)
    expect(document.body.textContent).toContain('正在加载全局模型')
    loading.value = false
    error.value = '模型加载失败'
    await nextTick()
    expect(document.body.textContent).toContain('模型加载失败')
    await clickText(root, '重试')
    expect(reload).toHaveBeenCalledOnce()
    expect(readSchedulingPolicies(config.value)[0].models).toEqual(['removed-model'])
    expect(valid.value).toBe(true)
    expect(control<HTMLButtonElement>(root, '添加调度配置').disabled).toBe(true)
  })

  it('disables configuration controls while saving', async () => {
    const { root, config, disabled } = mountEditor()
    const previous = JSON.stringify(config.value)
    disabled.value = true
    await nextTick()
    await clickText(root, '负载均衡')
    await clickText(root, '区分模型')
    await flush()
    expect(control<HTMLButtonElement>(root, '全部模型').disabled).toBe(true)
    expect(control<HTMLButtonElement>(root, '区分模型').disabled).toBe(true)
    expect(control(root, '全部模型').getAttribute('aria-pressed')).toBe('true')
    expect(root.querySelector('[aria-label="选择适用模型"]')).toBeNull()
    expect(document.querySelector('[aria-label="全局模型选择列表"]')).toBeNull()
    expect(root.querySelector('fieldset')?.disabled).toBe(true)
    expect(JSON.stringify(config.value)).toBe(previous)
  })

  it('keeps model-specific drafts unchanged when scope switching is disabled', async () => {
    const { root, config, disabled } = mountEditor()
    await select(root, 'model-a')
    const previous = JSON.stringify(config.value)
    disabled.value = true
    await flush()
    await clickText(root, '全部模型')
    expect(control(root, '区分模型').getAttribute('aria-pressed')).toBe('true')
    expect(control<HTMLButtonElement>(root, '添加调度配置').disabled).toBe(true)
    expect(JSON.stringify(config.value)).toBe(previous)
  })
})
