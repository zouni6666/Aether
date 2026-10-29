import { afterEach, describe, expect, it } from 'vitest'
import { createApp, h, nextTick, ref, type App } from 'vue'

import Drawer from '../drawer/Drawer.vue'

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

function mountDrawer(props: Partial<InstanceType<typeof Drawer>['$props']> & { open: boolean }, onClose: () => void) {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp({
    render: () => h(Drawer, { ...props, onClose }, () => h('p', { class: 'drawer-body' }, 'content')),
  })
  app.mount(root)
  mountedApps.push({ app, root })
  return root
}

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
  document.body.innerHTML = ''
})

describe('Drawer', () => {
  it('teleports the panel to body only while open', async () => {
    const open = ref(false)
    const app = createApp({
      render: () => h(Drawer, { open: open.value }, () => 'body'),
    })
    const root = document.createElement('div')
    document.body.appendChild(root)
    app.mount(root)
    mountedApps.push({ app, root })

    expect(document.body.querySelector('.drawer-panel')).toBeNull()
    open.value = true
    await nextTick()
    expect(document.body.querySelector('.drawer-panel')?.textContent).toBe('body')
  })

  it('emits close on backdrop click unless disabled', async () => {
    let closed = 0
    mountDrawer({ open: true }, () => { closed += 1 })
    await nextTick()
    const backdrop = document.body.querySelector('[data-drawer-backdrop]') as HTMLElement
    backdrop.click()
    expect(closed).toBe(1)

    document.body.innerHTML = ''
    mountDrawer({ open: true, closeOnBackdrop: false }, () => { closed += 1 })
    await nextTick()
    const blockedBackdrop = document.body.querySelector('[data-drawer-backdrop]') as HTMLElement
    blockedBackdrop.click()
    expect(closed).toBe(1)
  })

  it('emits close on Escape', async () => {
    let closed = 0
    mountDrawer({ open: true }, () => { closed += 1 })
    await nextTick()
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(closed).toBe(1)
  })

  it('applies width, layout and panel overrides', async () => {
    mountDrawer({ open: true, width: 'sm:w-[800px]', layout: 'column', panelClass: 'bg-background' }, () => {})
    await nextTick()
    const panel = document.body.querySelector('.drawer-panel') as HTMLElement
    expect(panel.classList.contains('sm:w-[800px]')).toBe(true)
    expect(panel.classList.contains('flex-col')).toBe(true)
    expect(panel.classList.contains('overflow-y-auto')).toBe(false)
    expect(panel.classList.contains('bg-background')).toBe(true)
    expect(panel.classList.contains('bg-card')).toBe(false)
  })
})
