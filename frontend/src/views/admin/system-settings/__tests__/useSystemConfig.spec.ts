import { beforeEach, describe, expect, it, vi } from 'vitest'

const { getAllSystemConfigsMock, updateSystemConfigMock } = vi.hoisted(() => ({
  getAllSystemConfigsMock: vi.fn(),
  updateSystemConfigMock: vi.fn(),
}))

vi.mock('@/api/admin', () => ({
  adminApi: {
    getAllSystemConfigs: getAllSystemConfigsMock,
    updateSystemConfig: updateSystemConfigMock,
    getSystemVersion: vi.fn(),
  },
}))

vi.mock('@/composables/useToast', () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
  }),
}))

vi.mock('@/composables/useSiteInfo', () => ({
  useSiteInfo: () => ({
    refreshSiteInfo: vi.fn(),
  }),
}))

vi.mock('@/utils/logger', () => ({
  log: {
    error: vi.fn(),
  },
}))

import { useSystemConfig } from '../composables/useSystemConfig'

describe('useSystemConfig', () => {
  beforeEach(() => {
    getAllSystemConfigsMock.mockReset()
    updateSystemConfigMock.mockReset()
  })

  it('loads config keys in one request and keeps change detection disabled until the baseline is ready', async () => {
    let resolveConfigs!: (value: Array<{ key: string, value: unknown, is_set?: boolean }>) => void
    getAllSystemConfigsMock.mockImplementation(() => new Promise((resolve) => {
      resolveConfigs = resolve
    }))

    const state = useSystemConfig()
    const loadPromise = state.loadSystemConfig()

    expect(getAllSystemConfigsMock).toHaveBeenCalledTimes(1)
    expect(getAllSystemConfigsMock).toHaveBeenCalledWith({ cacheTtlMs: 30_000 })

    state.systemConfig.value.request_record_level = 'headers'
    expect(state.systemConfigLoading.value).toBe(true)
    expect(state.hasLogConfigChanges.value).toBe(false)

    resolveConfigs([
      { key: 'request_record_level', value: 'basic' },
      { key: 'proxy_node_metrics_cleanup_batch_size', value: 5000 },
    ])
    await loadPromise

    expect(state.systemConfigLoading.value).toBe(false)
    expect(state.systemConfig.value.request_record_level).toBe('basic')
    expect(state.hasLogConfigChanges.value).toBe(false)

    state.systemConfig.value.request_record_level = 'full'
    expect(state.hasLogConfigChanges.value).toBe(true)
  })

  it('uses backend-compatible defaults when config rows have not been persisted yet', async () => {
    getAllSystemConfigsMock.mockResolvedValue([])

    const state = useSystemConfig()
    await state.loadSystemConfig()

    expect(state.systemConfig.value.request_record_level).toBe('basic')
    expect(state.systemConfig.value.request_record_compact_base64_images).toBe(false)
    expect(state.systemConfig.value.request_record_max_body_size_kb).toBe(0)
    expect(state.systemConfig.value).not.toHaveProperty('max_request_body_size')
    expect(state.systemConfig.value).not.toHaveProperty('max_response_body_size')
  })

  it('saves only the proxy node and ignores retired DNS allowlist settings', async () => {
    getAllSystemConfigsMock.mockResolvedValue([
      { key: 'system_proxy_node_id', value: 'node-1' },
      { key: 'execution_extra_trusted_dns_hosts', value: ['custom.example.com'] },
    ])
    updateSystemConfigMock.mockResolvedValue(undefined)
    const state = useSystemConfig()
    await state.loadSystemConfig()
    expect(state.systemConfig.value).not.toHaveProperty('execution_extra_trusted_dns_hosts')
    state.systemConfig.value.system_proxy_node_id = 'node-2'
    expect(state.hasProxyConfigChanges.value).toBe(true)
    await state.saveProxyConfig()
    expect(updateSystemConfigMock).toHaveBeenCalledTimes(1)
    expect(updateSystemConfigMock).toHaveBeenCalledWith(
      'system_proxy_node_id', 'node-2', '系统默认代理节点 ID'
    )
    expect(state.hasProxyConfigChanges.value).toBe(false)
  })

  it('loads and saves request-record body storage controls', async () => {
    getAllSystemConfigsMock.mockResolvedValue([
      { key: 'request_record_compact_base64_images', value: true },
      { key: 'request_record_max_body_size_kb', value: 512 },
    ])
    updateSystemConfigMock.mockResolvedValue({})

    const state = useSystemConfig()
    await state.loadSystemConfig()

    expect(state.systemConfig.value.request_record_compact_base64_images).toBe(true)
    expect(state.systemConfig.value.request_record_max_body_size_kb).toBe(512)
    state.systemConfig.value.request_record_max_body_size_kb = 256
    expect(state.hasLogConfigChanges.value).toBe(true)

    await state.saveLogConfig()

    expect(updateSystemConfigMock).toHaveBeenCalledWith(
      'request_record_compact_base64_images',
      true,
      '请求记录 Base64 图片精简开关'
    )
    expect(updateSystemConfigMock).toHaveBeenCalledWith(
      'request_record_max_body_size_kb',
      256,
      '请求记录单个 Body 大小上限（KB），0 表示不限制'
    )
    expect(state.hasLogConfigChanges.value).toBe(false)
  })
})
