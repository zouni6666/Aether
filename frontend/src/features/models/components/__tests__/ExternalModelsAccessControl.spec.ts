import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, nextTick } from 'vue'

const modelsDevMocks = vi.hoisted(() => ({
  getExternalModelsAccessConfig: vi.fn(),
  updateExternalModelsAccessConfig: vi.fn(),
}))

const toastMocks = vi.hoisted(() => ({
  success: vi.fn(),
  error: vi.fn(),
}))

vi.mock('@/api/models-dev', () => modelsDevMocks)

vi.mock('@/composables/useToast', () => ({
  useToast: () => toastMocks,
}))

vi.mock('@/components/ui', async () => {
  const { defineComponent, h } = await import('vue')
  const passthrough = (name: string) => defineComponent({
    name,
    inheritAttrs: false,
    setup(_, { attrs, slots }) {
      return () => h('div', attrs, slots.default?.())
    },
  })

  return {
    Button: defineComponent({
      name: 'ButtonStub',
      inheritAttrs: false,
      props: {
        disabled: { type: Boolean, default: false },
      },
      setup(props, { attrs, slots }) {
        return () => h('button', {
          ...attrs,
          disabled: props.disabled,
          type: 'button',
        }, slots.default?.())
      },
    }),
    Popover: defineComponent({
      name: 'PopoverStub',
      inheritAttrs: false,
      props: {
        open: { type: Boolean, default: false },
      },
      emits: ['update:open'],
      setup(props, { attrs, slots }) {
        return () => h('div', {
          ...attrs,
          'data-popover-open': String(props.open),
        }, slots.default?.())
      },
    }),
    PopoverTrigger: passthrough('PopoverTriggerStub'),
    PopoverContent: passthrough('PopoverContentStub'),
  }
})

vi.mock('lucide-vue-next', async () => {
  const { defineComponent, h } = await import('vue')
  return {
    Globe: defineComponent({
      name: 'GlobeStub',
      setup() {
        return () => h('span', { 'data-testid': 'external-models-access-globe' })
      },
    }),
  }
})

vi.mock('@/features/providers/components/ProxyNodeSelect.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return {
    default: defineComponent({
      name: 'ProxyNodeSelectStub',
      inheritAttrs: false,
      props: {
        modelValue: { type: String, required: true },
        disabled: { type: Boolean, default: false },
      },
      emits: ['update:modelValue'],
      setup(props, { attrs, emit }) {
        return () => h('div', {
          ...attrs,
          'data-model-value': props.modelValue,
          'data-disabled': String(props.disabled),
        }, [
          h('button', {
            type: 'button',
            'data-testid': 'select-proxy-node',
            onClick: () => emit('update:modelValue', 'proxy-1'),
          }),
        ])
      },
    }),
  }
})

import ExternalModelsAccessControl from '@/features/models/components/ExternalModelsAccessControl.vue'
import { createI18n, setI18nLocale } from '@/i18n'

interface MountedControl {
  app: ReturnType<typeof createApp>
  root: HTMLDivElement
}

const mountedControls: MountedControl[] = []

function mountControl(): MountedControl {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp(ExternalModelsAccessControl)
  app.use(createI18n())
  app.mount(root)
  const mounted = { app, root }
  mountedControls.push(mounted)
  return mounted
}

async function flushAsyncState() {
  await Promise.resolve()
  await Promise.resolve()
  await nextTick()
}

beforeEach(() => {
  setI18nLocale('zh-CN')
  modelsDevMocks.getExternalModelsAccessConfig.mockReset()
  modelsDevMocks.getExternalModelsAccessConfig.mockResolvedValue({ proxy_node_id: null })
  modelsDevMocks.updateExternalModelsAccessConfig.mockReset()
  modelsDevMocks.updateExternalModelsAccessConfig.mockImplementation(
    async (proxyNodeId: string | null) => ({
      proxy_node_id: proxyNodeId,
      cache_cleared: true,
    }),
  )
  toastMocks.success.mockReset()
  toastMocks.error.mockReset()
})

afterEach(() => {
  for (const mounted of mountedControls.splice(0)) {
    mounted.app.unmount()
    mounted.root.remove()
  }
})

describe('ExternalModelsAccessControl', () => {
  it('uses the provider-style globe button as the header control', async () => {
    const { root } = mountControl()
    await flushAsyncState()

    const trigger = root.querySelector('[data-testid="external-models-access-trigger"]')
    expect(trigger?.tagName).toBe('BUTTON')
    expect(trigger?.classList.contains('h-8')).toBe(true)
    expect(trigger?.classList.contains('w-8')).toBe(true)
    expect(root.querySelector('[data-testid="external-models-access-globe"]')).not.toBeNull()
    expect(trigger?.getAttribute('aria-label')).toBe('外部模型目录代理节点')
  })

  it('loads the saved node into the shared proxy selector', async () => {
    modelsDevMocks.getExternalModelsAccessConfig.mockResolvedValue({
      proxy_node_id: 'proxy-1',
    })

    const { root } = mountControl()
    await flushAsyncState()

    const select = root.querySelector('[data-testid="external-models-access-select"]')
    expect(select?.getAttribute('data-model-value')).toBe('proxy-1')
    expect(select?.getAttribute('data-disabled')).toBe('false')
    expect(root.querySelector('[data-testid="external-models-access-trigger"]')
      ?.classList.contains('text-blue-500')).toBe(true)
    expect(root.querySelector('[data-testid="external-models-access-clear"]')).not.toBeNull()
  })

  it('immediately saves a selected proxy node', async () => {
    const { root } = mountControl()
    await flushAsyncState()

    ;(root.querySelector('[data-testid="select-proxy-node"]') as HTMLButtonElement).click()
    await flushAsyncState()

    expect(modelsDevMocks.updateExternalModelsAccessConfig).toHaveBeenCalledWith('proxy-1')
    expect(toastMocks.success).toHaveBeenCalledWith('外部模型目录代理节点已保存')
    expect(root.querySelector('[data-testid="external-models-access-select"]')
      ?.getAttribute('data-model-value')).toBe('proxy-1')
  })

  it('clears the selected proxy back to direct access', async () => {
    modelsDevMocks.getExternalModelsAccessConfig.mockResolvedValue({
      proxy_node_id: 'proxy-1',
    })
    const { root } = mountControl()
    await flushAsyncState()

    ;(root.querySelector('[data-testid="external-models-access-clear"]') as HTMLButtonElement).click()
    await flushAsyncState()

    expect(modelsDevMocks.updateExternalModelsAccessConfig).toHaveBeenCalledWith(null)
    expect(root.querySelector('[data-testid="external-models-access-select"]')
      ?.getAttribute('data-model-value')).toBe('')
    expect(root.querySelector('[data-testid="external-models-access-clear"]')).toBeNull()
  })

  it('keeps the globe trigger and selector disabled when config loading fails', async () => {
    modelsDevMocks.getExternalModelsAccessConfig.mockRejectedValue(new Error('load failed'))
    const { root } = mountControl()
    await flushAsyncState()

    expect(root.querySelector('[data-testid="external-models-access-select"]')
      ?.getAttribute('data-disabled')).toBe('true')
    expect((root.querySelector('[data-testid="external-models-access-trigger"]') as HTMLButtonElement)
      .disabled).toBe(true)
    expect(toastMocks.error).toHaveBeenCalledOnce()
  })
})
