import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { setI18nLocale } from '@/i18n'
import type { ProxyNode, ProxyNodeEvent } from '@/api/proxy-nodes'
import {
  formatProxyNodeTime,
  proxyNodeEventDetail,
  proxyNodeEventTypeLabel,
  proxyNodeSchedulingBadge,
  proxyNodeStatusLabel,
  proxyNodeStatusTitle,
} from '../proxy-node-display'

beforeEach(() => setI18nLocale('zh-CN'))
afterEach(() => {
  setI18nLocale('zh-CN')
  vi.useRealTimers()
})

const node = (overrides: Omit<Partial<ProxyNode>, 'status'> & { status?: string }) => ({ status: 'online', ...overrides }) as ProxyNode

describe('proxy node display translations', () => {
  it('updates known status labels and preserves unknown server status and detail', () => {
    expect(proxyNodeStatusLabel(node({}))).toBe('在线')
    setI18nLocale('en-US')
    expect(proxyNodeStatusLabel(node({}))).toBe('Online')
    expect(proxyNodeStatusLabel(node({ tunnel_mode: true }))).toBe('Tunnel online')
    expect(proxyNodeStatusTitle(node({}))).toBe('The node is currently marked online')
    expect(proxyNodeSchedulingBadge(node({ remote_config: { scheduling_state: 'draining' } } as Partial<ProxyNode>))?.label).toBe('Draining')
    expect(proxyNodeEventTypeLabel('connected')).toBe('Connected')
    expect(proxyNodeStatusLabel(node({ status: '保存' }))).toBe('保存')
    expect(proxyNodeEventTypeLabel('删除')).toBe('删除')
    expect(proxyNodeEventDetail({ detail: '  12 分钟  ' } as ProxyNodeEvent)).toBe('  12 分钟  ')
  })

  it('formats relative time using the requested locale independently of the app locale', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-10-29T00:00:00Z'))
    expect(formatProxyNodeTime('2026-10-28T23:55:00Z', 'en-US')).toBe('5m ago')
    expect(formatProxyNodeTime('2026-10-28T23:55:00Z', 'zh-CN')).toBe('5分钟前')
    expect(formatProxyNodeTime('2026-10-28T22:00:00Z', 'en-US')).toBe('2h ago')
    expect(formatProxyNodeTime('2026-10-28T23:59:45Z', 'en-US')).toBe('Just now')
    expect(formatProxyNodeTime('invalid-date', 'en-US')).toBe('-')
  })
})
