import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, nextTick, ref, type App, type Ref } from 'vue'

import type { RequestDetail } from '@/api/dashboard'
import RequestDetailDrawer from '../RequestDetailDrawer.vue'

const apiMocks = vi.hoisted(() => ({
  getRequestDetail: vi.fn(),
}))

vi.mock('@/api/dashboard', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/dashboard')>()
  return {
    ...actual,
    dashboardApi: {
      ...actual.dashboardApi,
      getRequestDetail: apiMocks.getRequestDetail,
    },
  }
})

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
  apiMocks.getRequestDetail.mockReset()
})

function buildEmbeddingDetail(): RequestDetail {
  return {
    id: 'usage-embedding-1',
    request_id: 'req-embedding-1',
    user: {
      id: 'user-1',
      username: 'embedding-user',
      email: 'embedding@example.com',
    },
    api_key: {
      id: 'key-1',
      name: 'test-key',
      display: 'test-key',
    },
    provider: 'embedding-provider',
    model: 'embedding-model',
    tokens: { input: 100, output: 0, total: 100 },
    cost: { input: 0.00001, output: 0, total: 0.00001 },
    request_type: 'embedding',
    is_stream: false,
    status: 'completed',
    status_code: 200,
    response_time_ms: 10,
    created_at: '2026-07-16T00:00:00Z',
    request_headers: { 'content-type': 'application/json' },
    settlement: {
      settlement_snapshot: {
        pricing_snapshot: {
          pricing_source: 'global_default',
          tiered_pricing: {
            tiers: [{ up_to: null, input_price_per_1m: 0.1 }],
          },
        },
      },
    },
  }
}

describe('RequestDetailDrawer settlement pricing', () => {
  it('renders an input-only embedding tier without treating the missing output price as zero', async () => {
    apiMocks.getRequestDetail.mockResolvedValue(buildEmbeddingDetail())

    let isOpen!: Ref<boolean>
    const Host = defineComponent({
      setup() {
        isOpen = ref(false)
        return () => h(RequestDetailDrawer, {
          isOpen: isOpen.value,
          requestId: 'usage-embedding-1',
        })
      },
    })

    const root = document.createElement('div')
    document.body.appendChild(root)
    const app = createApp(Host)
    app.mount(root)
    mountedApps.push({ app, root })

    isOpen.value = true
    await nextTick()

    await vi.waitFor(() => {
      expect(document.body.textContent).toContain('输入 $0.1/M')
      expect(document.body.textContent).toContain('输出 -')
    })
    expect(document.body.textContent).not.toContain('输出 $0/M')
  })
})
