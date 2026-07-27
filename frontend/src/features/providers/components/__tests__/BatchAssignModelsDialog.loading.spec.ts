import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, nextTick, type App } from 'vue'

import BatchAssignModelsDialog from '../BatchAssignModelsDialog.vue'

const globalModelMocks = vi.hoisted(() => ({
  getGlobalModels: vi.fn(),
}))

const endpointMocks = vi.hoisted(() => ({
  getProviderModels: vi.fn(),
  getProviderKeys: vi.fn(),
  batchAssignModelsToProvider: vi.fn(),
  deleteModel: vi.fn(),
}))

vi.mock('@/api/endpoints/global-models', () => globalModelMocks)
vi.mock('@/api/endpoints', () => endpointMocks)
vi.mock('@/composables/useToast', () => ({
  useToast: () => ({
    error: vi.fn(),
    success: vi.fn(),
    warning: vi.fn(),
  }),
}))
vi.mock('@/composables/useConfirm', () => ({
  useConfirm: () => ({
    confirmWarning: vi.fn().mockResolvedValue(true),
  }),
}))
vi.mock('@/features/providers/composables/useUpstreamModelsCache', () => ({
  useUpstreamModelsCache: () => ({
    fetchModels: vi.fn(),
  }),
}))
vi.mock('@/components/ui/dialog/Dialog.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return {
    default: defineComponent({
      name: 'DialogStub',
      setup: (_props, { slots }) => () => h('section', [slots.default?.(), slots.footer?.()]),
    }),
  }
})
vi.mock('@/components/ui', async () => {
  const { defineComponent } = await import('vue')
  const passthrough = (name: string) => defineComponent({
    name,
    inheritAttrs: false,
    setup: (_props, { slots }) => () => slots.default?.(),
  })
  return {
    DropdownMenu: passthrough('DropdownMenuStub'),
    DropdownMenuTrigger: passthrough('DropdownMenuTriggerStub'),
    DropdownMenuContent: passthrough('DropdownMenuContentStub'),
    DropdownMenuItem: passthrough('DropdownMenuItemStub'),
  }
})

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

async function settle() {
  for (let index = 0; index < 5; index += 1) {
    await Promise.resolve()
    await nextTick()
  }
}

beforeEach(() => {
  globalModelMocks.getGlobalModels.mockReset()
  globalModelMocks.getGlobalModels.mockResolvedValue({ models: [], total: 0 })
  endpointMocks.getProviderModels.mockReset()
  endpointMocks.getProviderModels.mockResolvedValue([])
  endpointMocks.getProviderKeys.mockReset()
  endpointMocks.getProviderKeys.mockResolvedValue([])
  endpointMocks.batchAssignModelsToProvider.mockReset()
  endpointMocks.deleteModel.mockReset()
})

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
})

describe('BatchAssignModelsDialog loading', () => {
  it('loads model choices when lazily mounted in the open state', async () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const app = createApp(defineComponent({
      setup() {
        return () => h(BatchAssignModelsDialog, {
          open: true,
          providerId: 'provider-1',
          providerName: 'Provider One',
        })
      },
    }))
    app.mount(root)
    mountedApps.push({ app, root })

    await settle()

    expect(globalModelMocks.getGlobalModels).toHaveBeenCalledOnce()
    expect(globalModelMocks.getGlobalModels).toHaveBeenCalledWith({ limit: 1000 })
    expect(endpointMocks.getProviderModels).toHaveBeenCalledWith('provider-1')
    expect(endpointMocks.getProviderKeys).toHaveBeenCalledWith('provider-1')
  })
})
