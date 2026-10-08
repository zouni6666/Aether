import { createApp, nextTick, type App } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import RuntimeView from '../operations/RuntimeView.vue'
import { setI18nLocale } from '@/i18n'

const api = vi.hoisted(() => ({ live: vi.fn() }))
vi.mock('@/api/overview', () => ({ overviewApi: api }))
vi.mock('../components/OverviewStatus.vue', () => ({ default: { render: () => null } }))

let app: App | undefined
afterEach(() => { app?.unmount(); vi.resetAllMocks() })
async function mount() {
  setI18nLocale('zh-CN')
  const root = document.createElement('div')
  app = createApp(RuntimeView, { revision: 0 })
  app.mount(root)
  for (let index = 0; index < 5; index += 1) { await Promise.resolve(); await nextTick() }
  return root
}
function metric(root: HTMLElement, label: string) { return root.querySelector(`[data-operations-metric="${label}"]`) }

describe('operations live throughput', () => {
  it('shows recent throughput and current-node concurrency from a single live snapshot', async () => {
    api.live.mockResolvedValue({ data: {
      metrics: {
        local: { inFlight: 8 }, tunnel: { activeStreams: 3 },
        process: {
          systemCpuUsageBasisPoints: 2640, systemMemoryUsageBasisPoints: 3750,
          processCpuUsageBasisPoints: 25000, processMemoryBytes: 512 * 1024 ** 2,
          systemMemoryUsedBytes: 6 * 1024 ** 3, systemMemoryTotalBytes: 16 * 1024 ** 3,
        },
      },
      recent_activity: { meta: {}, data: {
        requests_per_minute: 240, tokens_per_minute: 12000,
        requests_per_second: 4, request_count: 240,
        success_rate: { value: 0.95 }, failed_request_count: 12,
      } },
    } })
    const root = await mount()
    expect(root.querySelectorAll('[data-operations-hardware] [data-operations-metric]')).toHaveLength(4)
    expect(root.querySelectorAll('[data-operations-gateway] [data-operations-metric]')).toHaveLength(6)
    expect(metric(root, '主机 CPU')?.textContent).toContain('26.4%')
    expect(metric(root, '主机内存')?.textContent).toContain('37.5%')
    expect(metric(root, '主机内存')?.textContent).toContain('6 GiB / 16 GiB')
    expect(metric(root, '进程 CPU')?.textContent).toContain('250%')
    expect(metric(root, '进程内存')?.textContent).toContain('512 MiB')
    expect(metric(root, 'RPM')?.textContent).toContain('240')
    expect(metric(root, 'TPM')?.textContent).toContain('12,000')
    expect(metric(root, '请求成功率')?.textContent).toContain('95.00%')
    expect(metric(root, '失败请求')?.textContent).toContain('12')
    expect(metric(root, '当前并发')?.textContent).toContain('8')
    expect(metric(root, '活跃流')?.textContent).toContain('3')
    expect(metric(root, 'RPM')?.textContent).toContain('最近 60 秒')
    expect(metric(root, '当前并发')?.textContent).toContain('当前节点')
    expect(api.live).toHaveBeenCalledTimes(1)
  })

  it('keeps unavailable values distinct from measured zeroes', async () => {
    api.live.mockResolvedValue({ data: {
      metrics: { process: { systemCpuUsageBasisPoints: 0, systemMemoryTotalBytes: 0, systemMemoryUsageBasisPoints: 0 }, local: {}, tunnel: {} },
      recent_activity: { data: { requests_per_minute: 0, tokens_per_minute: null, success_rate: { value: null }, failed_request_count: 0 } },
    } })
    const root = await mount()
    expect(metric(root, 'RPM')?.querySelector('span[title]')?.textContent).toBe('0')
    expect(metric(root, 'TPM')?.querySelector('span[title]')?.textContent).toBe('-')
    expect(metric(root, '请求成功率')?.querySelector('span[title]')?.textContent).toBe('-')
    expect(metric(root, '当前并发')?.querySelector('span[title]')?.textContent).toBe('-')
    expect(metric(root, '活跃流')?.querySelector('span[title]')?.textContent).toBe('-')
    expect(metric(root, '主机 CPU')?.querySelector('span[title]')?.textContent).toBe('0%')
    for (const label of ['主机内存', '进程 CPU', '进程内存']) {
      expect(metric(root, label)?.querySelector('span[title]')?.textContent).toBe('-')
    }
    expect(api.live).toHaveBeenCalledTimes(1)
  })
})
