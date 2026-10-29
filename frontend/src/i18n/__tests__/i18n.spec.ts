import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, nextTick, ref } from 'vue'

import { createI18n, getI18nLocale, normalizeLocale, setI18nLocale, useI18n, useLocaleOptions } from '@/i18n'
import { formatDate, formatRelativeTime } from '@/utils/format'
import { BUILTIN_TOOLS, BUILTIN_TOOL_BREADCRUMBS } from '@/config/builtin-tools'

describe('i18n infrastructure', () => {
  beforeEach(() => {
    localStorage.clear()
    document.documentElement.lang = ''
  })

  it('installs a single app-level translator and updates document language', () => {
    const app = createApp(defineComponent({ setup: () => () => h('div') }))

    app.use(createI18n())

    expect(['zh-CN', 'en-US']).toContain(document.documentElement.lang)
    expect(app.config.globalProperties.$t('site.home.login')).toBeTruthy()
    expect(localStorage.getItem('aether_locale')).toBe(document.documentElement.lang)
  })

  it('switches locale, persists it, and interpolates params', async () => {
    const Probe = defineComponent({
      setup() {
        const { t, setLocale } = useI18n()
        const { currentLocaleLabel } = useLocaleOptions()

        return { t, setLocale, currentLocaleLabel }
      },
      render() {
        return h('div', [
          h('span', { id: 'label' }, this.currentLocaleLabel),
          h('span', { id: 'message' }, this.t('site.privacy.currentVersion', { version: '2' })),
        ])
      },
    })

    const root = document.createElement('div')
    document.body.appendChild(root)
    const app = createApp(Probe)
    app.use(createI18n())
    const vm = app.mount(root) as InstanceType<typeof Probe> & {
      setLocale: (locale: 'zh-CN' | 'en-US') => void
    }

    vm.setLocale('en-US')
    await nextTick()

    expect(document.documentElement.lang).toBe('en-US')
    expect(localStorage.getItem('aether_locale')).toBe('en-US')
    expect(root.textContent).toContain('English')
    expect(root.textContent).toContain('Current version: 2')

    app.unmount()
    root.remove()
  })

  it('renders keyed UI while preserving application content across language switches', async () => {
    const Probe = defineComponent({
      setup() {
        const { t, setLocale } = useI18n()
        const source = ref('保存')
        return { t, setLocale, source }
      },
      render() {
        return h('div', [
          h('button', { title: this.t('common.cancel') }, this.t('common.copy')),
          h('span', { title: this.source }, this.source),
        ])
      },
    })
    const root = document.createElement('div')
    document.body.appendChild(root)
    const app = createApp(Probe)
    app.use(createI18n())
    const vm = app.mount(root) as InstanceType<typeof Probe>
    vm.setLocale('en-US')
    await nextTick()
    expect(root.querySelector('button')?.textContent).toBe('Copy')
    expect(root.querySelector('button')?.title).toBe('Cancel')
    expect(root.querySelector('span')?.textContent).toBe('保存')
    expect(root.querySelector('span')?.title).toBe('保存')
    vm.source = '删除'
    await nextTick()
    expect(root.querySelector('span')?.textContent).toBe('删除')
    vm.setLocale('zh-CN')
    await nextTick()
    expect(root.querySelector('button')?.textContent).toBe('复制')
    expect(root.querySelector('span')?.textContent).toBe('删除')
    app.unmount()
    root.remove()
  })

  it('normalizes saved language aliases without accepting unrelated language names', () => {
    expect(normalizeLocale('en')).toBe('en-US')
    expect(normalizeLocale('en_GB')).toBe('en-US')
    expect(normalizeLocale(' ZH-cn ')).toBe('zh-CN')
    expect(normalizeLocale('english')).toBeUndefined()
    expect(normalizeLocale('fr-FR')).toBeUndefined()
  })

  it('updates built-in tool names and breadcrumbs after their module has loaded', () => {
    const previousLocale = getI18nLocale()
    try {
      setI18nLocale('en-US')
      expect(BUILTIN_TOOLS.find(tool => tool.href === '/admin/ip-security')?.name).toBe('IP security')
      expect(BUILTIN_TOOL_BREADCRUMBS['/admin/audit-logs']).toBe('Audit logs')
      setI18nLocale('zh-CN')
      expect(BUILTIN_TOOLS.find(tool => tool.href === '/admin/ip-security')?.name).toBe('IP 安全')
      expect(BUILTIN_TOOL_BREADCRUMBS['/admin/audit-logs']).toBe('审计日志')
    } finally {
      setI18nLocale(previousLocale)
    }
  })

  it('continues switching language when browser storage is unavailable', () => {
    const write = vi.spyOn(localStorage, 'setItem').mockImplementation(() => {
      throw new DOMException('Storage blocked', 'SecurityError')
    })
    try {
      expect(() => setI18nLocale('en-US')).not.toThrow()
      expect(getI18nLocale()).toBe('en-US')
      expect(document.documentElement.lang).toBe('en-US')
    } finally {
      write.mockRestore()
    }
  })

  it('updates date and relative-time formatting with the selected language', () => {
    const date = '2026-09-07T12:30:00'
    setI18nLocale('zh-CN')
    const chineseDate = formatDate(date)
    expect(formatRelativeTime(-1, 'day')).toBe('昨天')
    setI18nLocale('en-US')
    expect(formatRelativeTime(-1, 'day')).toBe('yesterday')
    expect(formatRelativeTime(-1, 'minute')).toBe('1 minute ago')
    expect(formatRelativeTime(-2, 'minute')).toBe('2 minutes ago')
    expect(formatDate(date)).not.toBe(chineseDate)
    setI18nLocale('zh-CN')
    expect(formatDate(date)).toBe(chineseDate)
  })
})
