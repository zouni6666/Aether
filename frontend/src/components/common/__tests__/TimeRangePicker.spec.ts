import { createApp, defineComponent, h, nextTick, ref, type App } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import TimeRangePicker from '../TimeRangePicker.vue'
import type { DateRangeParams } from '@/features/usage/types'

vi.mock('@/components/ui', async () => {
  const { defineComponent, h } = await import('vue')
  const passthrough = defineComponent({ setup: (_, { slots }) => () => slots.default?.() })
  return {
    Select: defineComponent({
      props: { modelValue: String }, emits: ['update:modelValue'],
      setup: (props, { emit, slots }) => () => h('select', {
        value: props.modelValue,
        onChange: (event: Event) => emit('update:modelValue', (event.target as HTMLSelectElement).value),
      }, slots.default?.()),
    }),
    SelectItem: defineComponent({ props: { value: String }, setup: (props, { slots }) => () => h('option', { value: props.value }, slots.default?.()) }),
    SelectContent: passthrough, SelectTrigger: defineComponent({ render: () => null }), SelectValue: passthrough,
    Input: defineComponent({
      props: { modelValue: String, type: String }, emits: ['update:modelValue'],
      setup: (props, { emit }) => () => h('input', {
        type: props.type, value: props.modelValue,
        onInput: (event: Event) => emit('update:modelValue', (event.target as HTMLInputElement).value),
      }),
    }),
  }
})

let app: App | undefined
afterEach(() => { app?.unmount(); app = undefined })

describe('time range mode changes', () => {
  it('keeps a fixed range read-only in preset-only mode without emitting a replacement on mount', async () => {
    const range = ref<DateRangeParams>({ from: '2026-09-18T00:00:00Z', to: '2026-09-18T01:00:00Z', timezone: 'UTC' })
    const update = vi.fn((value: DateRangeParams) => { range.value = value })
    const root = document.createElement('div')
    app = createApp(defineComponent({ setup: () => () => h(TimeRangePicker, {
      modelValue: range.value, showGranularity: false, presetOnly: true,
      presetOptions: ['last1hour', 'today', 'last24hours', 'last7days'],
      'onUpdate:modelValue': update,
    }) }))
    app.mount(root)
    await nextTick()
    expect(update).not.toHaveBeenCalled()
    expect(root.querySelector('input')).toBeNull()
    expect(Array.from(root.querySelectorAll('option'), option => option.value)).toEqual(['last1hour', 'today', 'last24hours', 'last7days'])
    expect(range.value).toMatchObject({ from: '2026-09-18T00:00:00Z', to: '2026-09-18T01:00:00Z', timezone: 'UTC' })
    const select = root.querySelector('select')!
    select.value = 'last1hour'
    select.dispatchEvent(new Event('change'))
    await nextTick()
    expect(update).toHaveBeenCalledTimes(1)
    expect(range.value.preset).toBe('last1hour')
    expect(root.querySelector('input')).toBeNull()
  })

  it('does not repeat a preset selection when the parent synchronizes its range', async () => {
    const range = ref<DateRangeParams>({ preset: 'today', timezone: 'UTC', granularity: 'day' })
    const update = vi.fn()
    const root = document.createElement('div')
    app = createApp(defineComponent({ setup: () => () => h(TimeRangePicker, {
      modelValue: range.value, showGranularity: false, presetOnly: true,
      presetOptions: ['last1hour', 'today', 'last24hours'],
      'onUpdate:modelValue': update,
    }) }))
    app.mount(root)
    await nextTick()
    range.value = { ...range.value }
    await nextTick()
    expect(update).not.toHaveBeenCalled()
    range.value = { ...range.value, preset: 'last24hours' }
    await nextTick()
    expect(root.querySelector('select')?.value).toBe('last24hours')
    expect(update).not.toHaveBeenCalled()
  })

  it('keeps valid calendar dates after precise range → preset → custom', async () => {
    const range = ref<DateRangeParams>({ from: '2026-09-18T00:00:00Z', to: '2026-09-18T01:00:00Z', timezone: 'UTC' })
    const root = document.createElement('div')
    app = createApp(defineComponent({ setup: () => () => h(TimeRangePicker, {
      modelValue: range.value, showGranularity: false,
      'onUpdate:modelValue': value => { range.value = value },
    }) }))
    app.mount(root)
    await nextTick()
    expect(root.querySelector<HTMLInputElement>('input')?.type).toBe('datetime-local')
    const select = root.querySelector('select')!
    select.value = 'today'
    select.dispatchEvent(new Event('change'))
    await nextTick()
    select.value = 'custom'
    select.dispatchEvent(new Event('change'))
    await nextTick()
    expect(range.value).toMatchObject({ start_date: '2026-09-18', end_date: '2026-09-18' })
    expect(range.value.from).toBeUndefined()
    expect(Array.from(root.querySelectorAll('input')).map(input => [input.type, input.value])).toEqual([
      ['date', '2026-09-18'], ['date', '2026-09-18'],
    ])
  })
})
