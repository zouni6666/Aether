import { effectScope, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { useSidebarResize } from '../useSidebarResize'

describe('useSidebarResize', () => {
  let scope = effectScope()

  function setup() {
    return scope.run(() => useSidebarResize())!
  }

  function pointerEvent(type: string, clientX: number, pointerId = 1, button = 0) {
    const event = new MouseEvent(type, { clientX, button, cancelable: true })
    Object.defineProperty(event, 'pointerId', { value: pointerId })
    return event as PointerEvent
  }

  beforeEach(() => {
    localStorage.clear()
    scope = effectScope()
  })

  afterEach(() => {
    scope.stop()
  })

  it('restores saved width and normalizes invalid values', () => {
    localStorage.setItem('aether-sidebar-width', '350')
    const sidebar = setup()
    expect(sidebar.sidebarWidth.value).toBe(350)
    sidebar.sidebarWidth.value = 900
    expect(sidebar.sidebarWidth.value).toBe(480)
    sidebar.sidebarWidth.value = Number.NaN
    expect(sidebar.sidebarWidth.value).toBe(260)
  })

  it('resizes, clamps and persists the width', async () => {
    const sidebar = setup()
    sidebar.startSidebarResize(pointerEvent('pointerdown', 260))
    expect(sidebar.sidebarResizing.value).toBe(true)
    window.dispatchEvent(pointerEvent('pointermove', 320, 2))
    expect(sidebar.sidebarWidth.value).toBe(260)
    window.dispatchEvent(pointerEvent('pointermove', 320))
    expect(sidebar.sidebarWidth.value).toBe(320)
    await nextTick()
    expect(localStorage.getItem('aether-sidebar-width')).toBe('320')
    window.dispatchEvent(pointerEvent('pointermove', 100))
    expect(sidebar.sidebarWidth.value).toBe(200)
    window.dispatchEvent(pointerEvent('pointermove', 1000))
    expect(sidebar.sidebarWidth.value).toBe(480)
    window.dispatchEvent(pointerEvent('pointerup', 1000))
    expect(sidebar.sidebarResizing.value).toBe(false)
    window.dispatchEvent(pointerEvent('pointermove', 300))
    expect(sidebar.sidebarWidth.value).toBe(480)
  })

  it('restores body styles and removes listeners when disposed', () => {
    document.body.style.cursor = 'default'
    document.body.style.userSelect = 'text'
    const sidebar = setup()
    sidebar.startSidebarResize(pointerEvent('pointerdown', 260))
    scope.stop()
    expect(document.body.style.cursor).toBe('default')
    expect(document.body.style.userSelect).toBe('text')
    window.dispatchEvent(pointerEvent('pointermove', 350))
    expect(sidebar.sidebarWidth.value).toBe(260)
  })

  it('supports keyboard resizing and ignores secondary mouse buttons', () => {
    const sidebar = setup()
    sidebar.startSidebarResize(pointerEvent('pointerdown', 260, 1, 2))
    expect(sidebar.sidebarResizing.value).toBe(false)
    sidebar.handleSidebarResizeKey(new KeyboardEvent('keydown', { key: 'ArrowRight' }))
    expect(sidebar.sidebarWidth.value).toBe(270)
    sidebar.handleSidebarResizeKey(new KeyboardEvent('keydown', { key: 'ArrowLeft' }))
    expect(sidebar.sidebarWidth.value).toBe(260)
  })
})
