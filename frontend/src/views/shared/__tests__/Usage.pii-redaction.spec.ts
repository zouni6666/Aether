import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, nextTick, ref, type App, type PropType, type Ref } from 'vue'

import type { UsageRecord } from '@/api/usageRecords'
import Usage from '../Usage.vue'

const mocks = vi.hoisted(() => ({
  route: { path: '/admin/usage', query: {} },
  getAdminActiveRequests: vi.fn(),
  getUserActiveRequests: vi.fn(),
  useUsageData: vi.fn(),
  loadRecords: vi.fn(),
  detailUpdate: { id: 'usage-pii', chatPiiRedaction: true },
}))

vi.mock('vue-router', () => ({
  useRoute: () => mocks.route,
  useRouter: () => ({ replace: vi.fn() }),
}))
vi.mock('@/stores/auth', () => ({ useAuthStore: () => ({ canAccessAdmin: true }) }))
vi.mock('@/api/usage', () => ({
  usageApi: { getActiveRequests: mocks.getAdminActiveRequests },
}))
vi.mock('@/api/me', () => ({
  meApi: {
    getActiveRequests: mocks.getUserActiveRequests,
    getActivityHeatmap: vi.fn().mockResolvedValue(null),
  },
}))
vi.mock('@/api/users', () => ({ usersApi: { getAllUsers: vi.fn().mockResolvedValue([]) } }))
vi.mock('@/features/usage/composables/useUsageData', () => ({ useUsageData: mocks.useUsageData }))
vi.mock('@/features/usage/composables/useDateRange', () => ({ getDateRangeFromPeriod: () => ({ preset: 'today' }) }))
const emptyComponent = vi.hoisted(() => async () => {
  const { defineComponent } = await import('vue')
  return { default: defineComponent({ setup: () => () => null }) }
})
vi.mock('@/features/usage/components/UsageModelTable.vue', emptyComponent)
vi.mock('@/features/usage/components/UsageProviderTable.vue', emptyComponent)
vi.mock('@/features/usage/components/UsageApiFormatTable.vue', emptyComponent)
vi.mock('@/features/usage/components/ActivityHeatmapCard.vue', emptyComponent)
vi.mock('@/features/usage/components/IntervalTimelineCard.vue', emptyComponent)
vi.mock('@/features/usage/components/UsageRecordsTable.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return {
    default: defineComponent({
      props: { records: { type: Array as PropType<UsageRecord[]>, required: true } },
      setup: props => () => h('div', { 'data-pii-record': '' }, String(props.records[0]?.chat_pii_redaction)),
    }),
  }
})
vi.mock('@/features/usage/components/RequestDetailDrawer.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return {
    default: defineComponent({
      emits: ['requestState'],
      setup: (_, { emit }) => () => h('button', {
        'data-detail-update': '',
        onClick: () => emit('requestState', mocks.detailUpdate),
      }),
    }),
  }
})

let records: Ref<UsageRecord[]>
let app: App | undefined
let root: HTMLElement
let header: HTMLElement

beforeEach(() => {
  vi.useFakeTimers()
  mocks.route.path = '/admin/usage'
  mocks.getAdminActiveRequests.mockReset()
  mocks.getUserActiveRequests.mockReset()
  mocks.loadRecords.mockReset().mockResolvedValue(undefined)
  records = ref([{
    id: 'usage-pii',
    model: 'gpt-5',
    input_tokens: 0,
    output_tokens: 0,
    total_tokens: 0,
    cost: 0,
    is_stream: true,
    status: 'pending',
    created_at: '2026-10-12T00:00:00Z',
    chat_pii_redaction: false,
  }])
  mocks.useUsageData.mockReturnValue({
    isLoadingRecords: ref(false),
    providerStats: ref([]),
    apiFormatStats: ref([]),
    currentRecords: records,
    totalRecords: ref(1),
    enhancedModelStats: ref([]),
    availableModels: ref([]),
    availableProviders: ref([]),
    loadStats: vi.fn().mockResolvedValue(false),
    loadRecords: mocks.loadRecords,
  })
  root = document.createElement('div')
  header = document.createElement('div')
  header.id = 'header-actions-right'
  document.body.append(header, root)
})

afterEach(() => {
  app?.unmount()
  app = undefined
  root.remove()
  header.remove()
  vi.useRealTimers()
})

async function mountUsage() {
  app = createApp(Usage)
  app.mount(root)
  await nextTick()
}

describe('usage PII redaction synchronization', () => {
  it.each(['/admin/usage', '/usage'])('applies active markers without refreshing the list on %s', async (path) => {
    mocks.route.path = path
    const getActiveRequests = path.startsWith('/admin')
      ? mocks.getAdminActiveRequests
      : mocks.getUserActiveRequests
    const pending = { ...records.value[0] }
    getActiveRequests
      .mockResolvedValueOnce({ requests: [{ ...pending, status: 'streaming', chat_pii_redaction: true }] })
      .mockResolvedValueOnce({ requests: [{ ...pending, status: 'completed', chat_pii_redaction: false }] })

    await mountUsage()
    const initialListLoads = mocks.loadRecords.mock.calls.length
    expect(root.querySelector('[data-pii-record]')?.textContent).toBe('false')

    await vi.advanceTimersByTimeAsync(1000)
    expect(root.querySelector('[data-pii-record]')?.textContent).toBe('true')
    expect(records.value[0].status).toBe('streaming')

    await vi.advanceTimersByTimeAsync(1000)
    expect(records.value[0].status).toBe('completed')
    expect(root.querySelector('[data-pii-record]')?.textContent).toBe('true')
    expect(mocks.loadRecords).toHaveBeenCalledTimes(initialListLoads)
  })

  it('retains a marker resolved by the detail drawer when later detail reports false', async () => {
    await mountUsage()
    const detailButton = root.querySelector<HTMLButtonElement>('[data-detail-update]')!
    mocks.detailUpdate = { id: 'usage-pii', chatPiiRedaction: true }
    detailButton.click()
    await nextTick()
    expect(root.querySelector('[data-pii-record]')?.textContent).toBe('true')

    mocks.detailUpdate = { id: 'usage-pii', chatPiiRedaction: false }
    detailButton.click()
    await nextTick()
    expect(root.querySelector('[data-pii-record]')?.textContent).toBe('true')
  })
})
