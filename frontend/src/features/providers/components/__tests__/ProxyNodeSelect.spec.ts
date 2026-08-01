import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h } from 'vue'

import ProxyNodeSelect from '@/features/providers/components/ProxyNodeSelect.vue'
import { createI18n } from '@/i18n'

const proxyNodesStore = vi.hoisted(() => ({
  loading: false,
  nodes: [],
  onlineNodes: [],
  ensureLoaded: vi.fn(() => Promise.resolve()),
}))

vi.mock('@/stores/proxy-nodes', () => ({
  useProxyNodesStore: () => proxyNodesStore,
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
    Select: defineComponent({
      name: 'SelectStub',
      props: {
        disabled: { type: Boolean, default: false },
      },
      setup(props, { slots }) {
        return () => h('div', {
          'data-testid': 'proxy-select',
          'data-disabled': String(props.disabled),
        }, slots.default?.())
      },
    }),
    SelectTrigger: passthrough('SelectTriggerStub'),
    SelectValue: passthrough('SelectValueStub'),
    SelectContent: passthrough('SelectContentStub'),
    SelectItem: defineComponent({
      name: 'SelectItemStub',
      props: {
        value: { type: String, required: true },
      },
      setup(props, { slots }) {
        return () => h('div', { 'data-select-value': props.value }, slots.default?.())
      },
    }),
  }
})

function mountSelect(props: Record<string, unknown> = {}) {
  const root = document.createElement('div')
  const app = createApp(defineComponent({
    setup() {
      return () => h(ProxyNodeSelect, { modelValue: '', ...props })
    },
  }))
  app.use(createI18n())
  app.mount(root)
  return { app, root }
}

beforeEach(() => {
  proxyNodesStore.ensureLoaded.mockClear()
})

describe('ProxyNodeSelect', () => {
  it('loads proxy nodes when mounted', () => {
    const { app } = mountSelect()

    expect(proxyNodesStore.ensureLoaded).toHaveBeenCalledTimes(1)

    app.unmount()
  })

  it('stays disabled when there are no proxy nodes', () => {
    const { app, root } = mountSelect()

    expect(root.querySelector('[data-testid="proxy-select"]')
      ?.getAttribute('data-disabled')).toBe('true')

    app.unmount()
  })

  it('applies an accessible label to the select trigger', () => {
    const { app, root } = mountSelect({
      triggerAriaLabel: 'External model catalog access',
    })

    expect(root.querySelector('[aria-label="External model catalog access"]')).not.toBeNull()

    app.unmount()
  })
})
