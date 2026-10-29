import { createApp, h, nextTick, type App, type Component } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createI18n, setI18nLocale } from '@/i18n'
import AlertDialog from '../AlertDialog.vue'
import EmptyState from '../EmptyState.vue'
import JsonImportInput from '../JsonImportInput.vue'
import MultiSelect from '../MultiSelect.vue'
import RefreshButton from '@/components/ui/refresh-button.vue'

vi.mock('@/components/ui', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/components/ui')>()
  const { defineComponent, h } = await import('vue')
  return {
    ...actual,
    Dialog: defineComponent({
      inheritAttrs: false,
      setup: (_, { slots }) => () => h('div', [slots.header?.(), slots.default?.(), slots.footer?.()]),
    }),
  }
})

const mounted: Array<{ app: App; root: HTMLElement }> = []
function mount(component: Component, props: Record<string, unknown>) {
  const root = document.createElement('div')
  document.body.append(root)
  const app = createApp({ render: () => h(component, props) })
  app.use(createI18n())
  app.mount(root)
  mounted.push({ app, root })
  return root
}

beforeEach(() => setI18nLocale('en-US'))
afterEach(() => {
  for (const { app, root } of mounted.splice(0)) {
    app.unmount()
    root.remove()
  }
  setI18nLocale('zh-CN')
})

describe('localized component defaults', () => {
  it('switches refresh button defaults without changing explicit titles', async () => {
    const defaults = mount(RefreshButton, {})
    const custom = mount(RefreshButton, { title: '保存' })
    expect(defaults.querySelector('button')?.title).toBe('Refresh')
    expect(custom.querySelector('button')?.title).toBe('保存')
    setI18nLocale('zh-CN')
    await nextTick()
    expect(defaults.querySelector('button')?.title).toBe('刷新')
    expect(custom.querySelector('button')?.title).toBe('保存')
  })

  it('switches empty state defaults while preserving supplied content and empty titles', async () => {
    const defaults = mount(EmptyState, {})
    const custom = mount(EmptyState, { title: '保存', description: '删除', actionText: '12 分钟' })
    const blank = mount(EmptyState, { title: '' })
    expect(defaults.textContent).toContain('No data')
    expect(custom.textContent).toContain('保存')
    expect(custom.textContent).toContain('删除')
    expect(custom.textContent).toContain('12 分钟')
    expect(blank.querySelector('h3')).toBeNull()
    setI18nLocale('zh-CN')
    await nextTick()
    expect(defaults.textContent).toContain('暂无数据')
    expect(custom.textContent).toContain('保存')
    setI18nLocale('en-US')
    await nextTick()
    expect(defaults.textContent).toContain('No data')
  })

  it('translates multiselect defaults without changing user option labels', async () => {
    const defaults = mount(MultiSelect, { modelValue: [], options: [] })
    const custom = mount(MultiSelect, { modelValue: ['save'], options: [{ value: 'save', label: '保存' }] })
    const placeholder = mount(MultiSelect, { modelValue: [], options: [], placeholder: '删除' })
    expect(defaults.querySelector('button')?.textContent).toContain('Select')
    expect(custom.querySelector('button')?.textContent).toContain('保存')
    expect(placeholder.querySelector('button')?.textContent).toContain('删除')
    setI18nLocale('zh-CN')
    await nextTick()
    expect(defaults.querySelector('button')?.textContent).toContain('请选择')
    expect(custom.querySelector('button')?.textContent).toContain('保存')
  })

  it('switches import defaults while preserving caller text', async () => {
    const defaults = mount(JsonImportInput, { modelValue: '' })
    const custom = mount(JsonImportInput, { modelValue: '', dropTitle: '保存', dropHint: '删除', pasteToggleText: '12 分钟' })
    expect(defaults.textContent).toContain('Drop import files or click to choose')
    expect(custom.textContent).toContain('保存')
    expect(custom.textContent).toContain('删除')
    expect(custom.textContent).toContain('12 分钟')
    setI18nLocale('zh-CN')
    await nextTick()
    expect(defaults.textContent).toContain('拖入导入文件或点击选择')
    expect(custom.textContent).toContain('保存')
  })

  it('switches alert button defaults without rewriting supplied descriptions', async () => {
    const defaults = mount(AlertDialog, { modelValue: true, title: '保存', description: '删除' })
    const custom = mount(AlertDialog, { modelValue: true, title: '保存', description: '12 分钟', confirmText: '删除', cancelText: '' })
    expect(defaults.textContent).toContain('Confirm')
    expect(defaults.textContent).toContain('Cancel')
    expect(defaults.textContent).toContain('保存')
    expect(defaults.textContent).toContain('删除')
    expect(custom.textContent).not.toContain('Cancel')
    expect(custom.textContent).toContain('12 分钟')
    setI18nLocale('zh-CN')
    await nextTick()
    expect(defaults.textContent).toContain('确认')
    expect(defaults.textContent).toContain('取消')
  })
})
