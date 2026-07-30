import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, nextTick, type App } from 'vue'

import type { FailoverRulesConfig, ProviderWithEndpointsSummary } from '@/api/endpoints/types'
import FailoverRulesDialog from '../FailoverRulesDialog.vue'

const endpointMocks = vi.hoisted(() => ({
  updateProvider: vi.fn(),
}))

vi.mock('@/api/endpoints', () => ({
  updateProvider: endpointMocks.updateProvider,
}))

vi.mock('@/composables/useToast', () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
  }),
}))

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

function makeProvider(failoverRules: FailoverRulesConfig | null): ProviderWithEndpointsSummary {
  return {
    id: 'provider-1',
    failover_rules: failoverRules,
  } as ProviderWithEndpointsSummary
}

function mountDialog(failoverRules: FailoverRulesConfig | null = null) {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp(FailoverRulesDialog, {
    open: true,
    provider: makeProvider(failoverRules),
    'onUpdate:open': vi.fn(),
  })
  app.mount(root)
  mountedApps.push({ app, root })
}

async function settle() {
  for (let index = 0; index < 4; index += 1) {
    await Promise.resolve()
    await nextTick()
  }
}

function transportErrorSwitch(): HTMLButtonElement {
  const control = document.body.querySelector<HTMLButtonElement>(
    '[role="switch"][aria-label="传输错误时继续尝试下一候选"]',
  )
  if (!control) throw new Error('Missing transport-error failover switch')
  return control
}

function clickSave() {
  const button = [...document.body.querySelectorAll<HTMLButtonElement>('button')]
    .find(candidate => candidate.textContent?.trim() === '保存')
  if (!button) throw new Error('Missing save button')
  button.click()
}

beforeEach(() => {
  endpointMocks.updateProvider.mockReset()
  endpointMocks.updateProvider.mockResolvedValue(makeProvider(null))
})

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
  document.body.innerHTML = ''
})

describe('FailoverRulesDialog transport errors', () => {
  it('continues failover by default and explains why HTTP status rules do not apply', async () => {
    mountDialog()
    await settle()

    expect(transportErrorSwitch().getAttribute('aria-checked')).toBe('true')
    expect(document.body.textContent).toContain('没有可用于故障转移判断的上游 HTTP 状态码')
    expect(document.body.textContent).toContain('DNS 解析')
    expect(document.body.textContent).toContain('TCP 连接')
    expect(document.body.textContent).toContain('TLS 握手')
    expect(document.body.textContent).toContain('响应提交前的连接重置或超时')
    expect(document.body.textContent).toContain('响应开始发送后无法切换候选')
  })

  it('persists stop_on_transport_errors when continuing is disabled', async () => {
    mountDialog()
    await settle()

    transportErrorSwitch().click()
    await nextTick()
    expect(transportErrorSwitch().getAttribute('aria-checked')).toBe('false')

    clickSave()
    await settle()

    expect(endpointMocks.updateProvider).toHaveBeenCalledWith('provider-1', {
      failover_rules: {
        stop_on_transport_errors: true,
      },
    })
  })

  it('loads a stop rule and removes only that rule when continuing is enabled', async () => {
    mountDialog({
      max_retries: 2,
      stop_on_transport_errors: true,
    })
    await settle()

    expect(transportErrorSwitch().getAttribute('aria-checked')).toBe('false')
    transportErrorSwitch().click()
    await nextTick()

    clickSave()
    await settle()

    expect(endpointMocks.updateProvider).toHaveBeenCalledWith('provider-1', {
      failover_rules: {
        max_retries: 2,
      },
    })
  })
})
