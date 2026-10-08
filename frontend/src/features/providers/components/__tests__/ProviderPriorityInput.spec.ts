import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApp, h, nextTick, ref, type App } from 'vue'
import ProviderPriorityInput from '../ProviderPriorityInput.vue'

const mounted: Array<{ app: App, root: HTMLElement }> = []

function mountInput() {
  const priority = ref(10)
  const disabled = ref(false)
  const editContext = ref('group-one:all-models')
  const changed = vi.fn((value: number) => { priority.value = value })
  const rowEvents = { click: vi.fn(), mousedown: vi.fn(), pointerdown: vi.fn(), keydown: vi.fn() }
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp({
    setup: () => () => h('div', {
      onClick: rowEvents.click, onMousedown: rowEvents.mousedown,
      onPointerdown: rowEvents.pointerdown, onKeydown: rowEvents.keydown,
    }, [h(ProviderPriorityInput, {
      providerName: 'Provider One', priority: priority.value, disabled: disabled.value,
      editContext: editContext.value,
      'onUpdate:priority': changed,
    })]),
  })
  app.mount(root)
  mounted.push({ app, root })
  return {
    priority, disabled, editContext, changed, rowEvents,
    button: () => root.querySelector<HTMLButtonElement>('button[aria-label="Provider One 的组内优先级"]'),
    input: () => root.querySelector<HTMLInputElement>('input[aria-label="Provider One 的组内优先级"]'),
  }
}

async function edit(control: ReturnType<typeof mountInput>, value = '24') {
  control.button()!.click()
  await nextTick()
  const input = control.input()!
  expect(input).toBeTruthy()
  input.value = value
  input.dispatchEvent(new Event('input', { bubbles: true }))
  return input
}

afterEach(() => {
  for (const { app, root } of mounted.splice(0)) { app.unmount(); root.remove() }
  vi.restoreAllMocks()
})

describe('ProviderPriorityInput', () => {
  it('shows only a number button until clicked, then focuses and selects the saved value', async () => {
    const select = vi.spyOn(HTMLInputElement.prototype, 'select')
    const control = mountInput()
    expect(control.input()).toBeNull()
    expect(control.button()?.textContent?.trim()).toBe('10')
    control.button()!.click()
    await nextTick()
    expect(document.activeElement).toBe(control.input())
    expect(control.input()?.value).toBe('10')
    expect(select).toHaveBeenCalledOnce()
  })

  it('submits Enter once and restores button focus before ignoring a trailing blur', async () => {
    const control = mountInput()
    const input = await edit(control)
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await nextTick()
    input.dispatchEvent(new Event('blur', { bubbles: true }))
    await nextTick()
    expect(control.changed).toHaveBeenCalledExactlyOnceWith(24)
    expect(control.input()).toBeNull()
    expect(document.activeElement).toBe(control.button())
  })

  it('cancels Escape without submitting and restores the original value and button focus', async () => {
    const control = mountInput()
    const input = await edit(control)
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await nextTick()
    input.dispatchEvent(new Event('blur', { bubbles: true }))
    expect(control.changed).not.toHaveBeenCalled()
    expect(control.input()).toBeNull()
    expect(control.button()?.textContent?.trim()).toBe('10')
    expect(document.activeElement).toBe(control.button())
  })

  it.each(['priority', 'disabled', 'editContext'] as const)('discards unfinished drafts when %s changes', async changedProp => {
    const control = mountInput()
    const oldInput = await edit(control)
    if (changedProp === 'priority') control.priority.value = 99
    else if (changedProp === 'disabled') control.disabled.value = true
    else control.editContext.value = 'group-two:claude-sonnet'
    await nextTick()
    expect(control.input()).toBeNull()
    oldInput.dispatchEvent(new Event('blur', { bubbles: true }))
    oldInput.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(control.changed).not.toHaveBeenCalled()
    expect(control.button()?.textContent?.trim()).toBe(changedProp === 'priority' ? '99' : '10')
    if (changedProp === 'disabled') expect(control.button()?.disabled).toBe(true)
  })

  it('blocks editing while disabled', async () => {
    const control = mountInput()
    control.disabled.value = true
    await nextTick()
    control.button()!.click()
    await nextTick()
    expect(control.input()).toBeNull()
    expect(control.changed).not.toHaveBeenCalled()
  })

  it('stops row click, drag-starting pointer events, and keyboard shortcuts in both display and editing states', async () => {
    const control = mountInput()
    control.button()!.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    control.button()!.dispatchEvent(new MouseEvent('pointerdown', { bubbles: true }))
    control.button()!.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    const input = await edit(control)
    input.click()
    input.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    input.dispatchEvent(new MouseEvent('pointerdown', { bubbles: true }))
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }))
    for (const listener of Object.values(control.rowEvents)) expect(listener).not.toHaveBeenCalled()
  })
})
