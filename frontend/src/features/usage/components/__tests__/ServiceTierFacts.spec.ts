import { afterEach, describe, expect, it } from 'vitest'
import { createApp, h, type App } from 'vue'

import ServiceTierFacts from '../ServiceTierFacts.vue'

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
})

describe('ServiceTierFacts', () => {
  it('renders all three facts and marks a missing actual tier explicitly', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const app = createApp({
      render: () => h(ServiceTierFacts, {
        requested: 'priority',
        actual: null,
        billing: 'flex',
      }),
    })
    app.mount(root)
    mountedApps.push({ app, root })

    expect(root.querySelector('[data-testid="service-tier-facts"]')).not.toBeNull()
    expect([...root.querySelectorAll('dt')].map(node => node.textContent?.trim())).toEqual([
      '请求层级',
      '实际层级',
      '计费层级',
    ])
    expect([...root.querySelectorAll('dd')].map(node => node.textContent?.trim())).toEqual([
      'Fast',
      '-',
      'flex',
    ])
    expect([...root.querySelectorAll('dd')].map(node => node.getAttribute('title'))).toEqual([
      'Fast',
      '-',
      'flex',
    ])
  })

  it('uses the same Fast label for raw priority and fast facts', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const app = createApp({
      render: () => h(ServiceTierFacts, {
        requested: 'priority',
        actual: 'fast',
        billing: 'priority',
      }),
    })
    app.mount(root)
    mountedApps.push({ app, root })

    expect([...root.querySelectorAll('dd')].map(node => node.textContent?.trim())).toEqual([
      'Fast',
      'Fast',
      'Fast',
    ])
    expect([...root.querySelectorAll('dd')].map(node => node.getAttribute('title'))).toEqual([
      'Fast',
      'Fast',
      'Fast',
    ])
  })

  it('renders the processing-tier multiplier with the billing tier label', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const app = createApp({
      render: () => h(ServiceTierFacts, {
        requested: 'priority',
        actual: 'fast',
        billing: 'fast',
        priceMultiplier: 2.5,
      }),
    })
    app.mount(root)
    mountedApps.push({ app, root })

    const multiplier = root.querySelector('[data-testid="service-tier-price-multiplier"]')
    expect(multiplier?.textContent).toContain('Fast 倍率')
    expect(multiplier?.textContent).toContain('2.5×')
  })

  it('does not render an empty or invalid processing-tier multiplier', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const app = createApp({
      render: () => h(ServiceTierFacts, {
        requested: 'priority',
        actual: null,
        billing: null,
        priceMultiplier: null,
      }),
    })
    app.mount(root)
    mountedApps.push({ app, root })

    expect(root.querySelector('[data-testid="service-tier-price-multiplier"]')).toBeNull()
  })
})
