import { createApp, nextTick, type App } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, describe, expect, it } from 'vitest'
import RuntimeFocus from '../operations/RuntimeFocus.vue'
import { buildGatewayMetricsSummary } from '@/api/monitoring'
import type { OverviewLive } from '@/api/overview'
import { setI18nLocale } from '@/i18n'

let app: App | undefined
afterEach(() => app?.unmount())

function snapshot(): OverviewLive {
  return {
    observed_at: '2026-09-20T00:00:00Z', window_seconds: 60, node_id: 'test-node',
    scope: { kind: 'node' }, metrics: buildGatewayMetricsSummary(''), resilience: null,
    unavailable_sections: [],
  }
}

async function mount(value: OverviewLive) {
  setI18nLocale('zh-CN')
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:pathMatch(.*)*', component: { render: () => null } }] })
  await router.push('/admin/operations')
  await router.isReady()
  const root = document.createElement('div')
  app = createApp(RuntimeFocus, { snapshot: value })
  app.use(router)
  app.mount(root)
  await nextTick()
  return root
}

function valueOf(root: HTMLElement, key: string) {
  return root.querySelector(`[data-runtime-focus-item="${key}"] dd`)?.textContent
}

describe('operations processing pipeline', () => {
  it('keeps unavailable samples distinct from measured zeroes', async () => {
    const sample = snapshot()
    sample.metrics!.requestCandidateQueue.depth = 0
    sample.metrics!.postgres.waitingConnections = 0
    const root = await mount(sample)
    expect(root.querySelectorAll('[data-runtime-focus-group]')).toHaveLength(3)
    expect(valueOf(root, 'candidate-queue')).toBe('0')
    expect(valueOf(root, 'database-waiting')).toBe('0')
    expect(valueOf(root, 'lock-waiting')).toBe('-')
    expect(valueOf(root, 'open-circuits')).toBe('-')
    expect(valueOf(root, 'usage-lag')).toBe('-')
    expect(root.querySelector('[data-runtime-focus-queue-state]')?.textContent).toBe('未采集')
    expect(root.textContent).not.toContain('正常')
  })

  it('separates undelivered, unacknowledged and dead-letter entries, preserving a real zero', async () => {
    const sample = snapshot()
    Object.assign(sample.metrics!.usageQueue, { enabled: true, configured: true, unavailable: false, groupLag: 7, groupPending: 0, dlqLength: 3 })
    sample.resilience = {
      timestamp: '2026-09-20T00:00:00Z', health_score: 75, status: 'degraded', recent_errors: [], recommendations: [],
      error_statistics: { total_errors: 9, active_keys: 4, degraded_keys: 1, unhealthy_keys: 0, open_circuit_breakers: 2, circuit_breakers: {} },
    }
    const root = await mount(sample)
    const usage = root.querySelector('[data-runtime-focus-group="usage"]')!
    expect([...usage.querySelectorAll('dt')].map(item => item.textContent)).toEqual(['待消费', '待确认', '死信'])
    expect([...usage.querySelectorAll('dd')].map(item => item.textContent)).toEqual(['7', '0', '3'])
    expect(usage.querySelector('[data-runtime-focus-queue-state]')).toBeNull()
    expect(valueOf(root, 'open-circuits')).toBe('2')
    expect(valueOf(root, 'degraded-credentials')).toBe('1')
    expect(root.querySelector('a')?.getAttribute('href')).toContain('/admin/health-monitor')
  })

  it.each([
    [{ enabled: false }, '未启用'],
    [{ configured: false }, '未配置'],
    [{ unavailable: true }, '未采集'],
  ])('does not present queue defaults as measurements when %s', async (flags, label) => {
    const sample = snapshot()
    Object.assign(sample.metrics!.usageQueue, { enabled: true, configured: true, unavailable: false, groupLag: 0, groupPending: 0, dlqLength: 0 }, flags)
    const root = await mount(sample)
    expect(root.querySelector('[data-runtime-focus-queue-state]')?.textContent).toBe(label)
    for (const key of ['usage-lag', 'usage-pending', 'usage-dead-letters']) expect(valueOf(root, key)).toBe('-')
  })

  it('uses the supplied Prometheus snapshot when structured metrics are absent', async () => {
    const sample = snapshot()
    sample.metrics = null
    sample.metrics_text = 'usage_queue_group_lag 4\nusage_queue_group_pending 2\nusage_queue_dlq_length 0\n'
    const root = await mount(sample)
    expect(valueOf(root, 'usage-lag')).toBe('4')
    expect(valueOf(root, 'usage-pending')).toBe('2')
    expect(valueOf(root, 'usage-dead-letters')).toBe('0')
  })
})
