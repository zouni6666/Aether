import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApp, h, nextTick, ref, type App } from 'vue'
import ProviderGroupControls from '../ProviderGroupControls.vue'

let app: App | null = null
let root: HTMLElement | null = null

function mountControls(disabled = false, showPriority = true) {
  const priority = ref(10)
  const enabled = ref(true)
  const busy = ref(disabled)
  const priorityChanged = vi.fn((value: number) => { priority.value = value })
  const rowClicked = vi.fn()
  const rowMouseDown = vi.fn()
  const rowPointerDown = vi.fn()
  root = document.createElement('div')
  document.body.appendChild(root)
  app = createApp({
    setup: () => () => h('div', { onClick: rowClicked, onMousedown: rowMouseDown, onPointerdown: rowPointerDown }, [
      h(ProviderGroupControls, {
        providerName: 'Provider One', priority: priority.value, enabled: enabled.value, disabled: busy.value, showPriority,
        'onUpdate:priority': priorityChanged,
      }),
    ]),
  })
  app.mount(root)
  return {
    priority, enabled, busy, priorityChanged, rowClicked, rowMouseDown, rowPointerDown,
    button: () => root!.querySelector<HTMLButtonElement>('button[aria-label="Provider One 的组内优先级"]'),
    input: () => root!.querySelector<HTMLInputElement>('input[aria-label="Provider One 的组内优先级"]'),
  }
}

async function beginEditing(controls: ReturnType<typeof mountControls>): Promise<HTMLInputElement> {
  controls.button()?.click()
  await nextTick()
  const input = controls.input()
  expect(input).toBeTruthy()
  return input!
}

function setInput(input: HTMLInputElement, value: string): void {
  input.value = value
  input.dispatchEvent(new Event('input', { bubbles: true }))
}

afterEach(() => { app?.unmount(); root?.remove(); app = null; root = null })

describe('ProviderGroupControls', () => {
  it.each([0, 24, 2147483647])('submits valid group priority %i once on blur', async priority => {
    const controls = mountControls()
    expect(controls.input()).toBeNull()
    const input = await beginEditing(controls)
    setInput(input, String(priority))
    input.dispatchEvent(new Event('blur', { bubbles: true }))
    await nextTick()
    expect(controls.priorityChanged).toHaveBeenCalledExactlyOnceWith(priority)
    expect(controls.button()?.textContent).toContain(String(priority))
    expect(controls.input()).toBeNull()
  })

  it.each(['', '-1', '1.5', '2147483648'])('restores the saved priority for invalid input %j', async value => {
    const controls = mountControls()
    const input = await beginEditing(controls)
    setInput(input, value)
    input.dispatchEvent(new Event('blur', { bubbles: true }))
    await nextTick()
    expect(controls.priorityChanged).not.toHaveBeenCalled()
    expect(controls.button()?.textContent).toContain('10')
    expect(controls.input()).toBeNull()
  })

  it.each([true, false])('shows enablement %s as a read-only badge when priority is hidden', async enabled => {
    const controls = mountControls(false, false)
    controls.enabled.value = enabled
    await nextTick()
    expect(root!.textContent?.trim()).toBe(enabled ? '本组启用' : '本组禁用')
    expect(root!.querySelector('button, input, [role="switch"]')).toBeNull()
  })

  it('keeps priority editing from triggering row actions', async () => {
    const controls = mountControls()
    controls.button()!.dispatchEvent(new MouseEvent('pointerdown', { bubbles: true }))
    controls.button()!.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    const input = await beginEditing(controls)
    input.dispatchEvent(new MouseEvent('pointerdown', { bubbles: true }))
    input.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    input.click()
    await nextTick()
    expect(controls.priorityChanged).not.toHaveBeenCalled()
    expect(controls.rowClicked).not.toHaveBeenCalled()
    expect(controls.rowMouseDown).not.toHaveBeenCalled()
    expect(controls.rowPointerDown).not.toHaveBeenCalled()
  })

  it('disables priority editing while the group is busy', async () => {
    const controls = mountControls(true)
    expect(controls.button()?.disabled).toBe(true)
    controls.button()?.click()
    await nextTick()
    expect(controls.input()).toBeNull()
    expect(controls.priorityChanged).not.toHaveBeenCalled()
    expect(root!.textContent).toContain('本组启用')
  })
})
