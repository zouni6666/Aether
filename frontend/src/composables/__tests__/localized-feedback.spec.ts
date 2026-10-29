import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { setI18nLocale, translate } from '@/i18n'
import { useToast } from '../useToast'
import { useConfirm } from '../useConfirm'

beforeEach(() => setI18nLocale('zh-CN'))
afterEach(() => {
  useToast().clearAll()
  useConfirm().handleCancel()
  setI18nLocale('zh-CN')
})

describe('localized feedback', () => {
  it('preserves toast content even when it matches a translated UI phrase', () => {
    const { showToast, toasts, removeToast } = useToast()
    setI18nLocale('en-US')
    const id = showToast({ title: '保存', description: '  删除\n12 分钟  ', duration: 0 })
    expect(toasts.value[0]).toMatchObject({ id, title: '保存', message: '  删除\n12 分钟  ' })
    setI18nLocale('zh-CN')
    expect(toasts.value[0]).toMatchObject({ id, title: '保存', message: '  删除\n12 分钟  ' })
    removeToast(id)
    expect(toasts.value).toEqual([])
  })

  it('accepts text explicitly translated by the caller without recognizing it again', () => {
    setI18nLocale('en-US')
    const { success, toasts } = useToast()
    success(translate('common.copy'))
    expect(toasts.value[0].message).toBe('Copy')
    setI18nLocale('zh-CN')
    expect(toasts.value[0].message).toBe('Copy')
  })

  it('keeps pending confirmations while their default labels change language', async () => {
    const { confirm, state, handleConfirm } = useConfirm()
    setI18nLocale('en-US')
    const result = confirm({ message: '保存' })
    expect(state.value).toMatchObject({ title: 'Confirm action', message: '保存', confirmText: 'Confirm', cancelText: 'Cancel' })
    setI18nLocale('zh-CN')
    expect(state.value).toMatchObject({ isOpen: true, title: '确认操作', message: '保存', confirmText: '确认', cancelText: '取消' })
    handleConfirm()
    expect(await result).toBe(true)
    expect(state.value.isOpen).toBe(false)
  })

  it('preserves explicitly supplied confirmation text and empty strings', async () => {
    const { confirm, state, handleCancel } = useConfirm()
    setI18nLocale('en-US')
    const result = confirm({ title: '', message: '  删除\n12 分钟  ', confirmText: '保存', cancelText: '' })
    expect(state.value).toMatchObject({ title: '', message: '  删除\n12 分钟  ', confirmText: '保存', cancelText: '' })
    setI18nLocale('zh-CN')
    expect(state.value).toMatchObject({ title: '', message: '  删除\n12 分钟  ', confirmText: '保存', cancelText: '' })
    handleCancel()
    expect(await result).toBe(false)
  })

  it('translates convenience confirmation defaults while preserving the message', async () => {
    const { confirmDanger, confirmWarning, confirmInfo, state, handleCancel } = useConfirm()
    setI18nLocale('en-US')
    const danger = confirmDanger('删除')
    expect(state.value).toMatchObject({ title: 'Dangerous action', message: '删除', confirmText: 'Delete' })
    handleCancel()
    expect(await danger).toBe(false)
    const warning = confirmWarning('保存')
    expect(state.value).toMatchObject({ title: 'Warning', message: '保存', confirmText: 'Continue' })
    handleCancel()
    expect(await warning).toBe(false)
    const info = confirmInfo('12 分钟')
    expect(state.value).toMatchObject({ title: 'Information', message: '12 分钟', confirmText: 'Confirm' })
    handleCancel()
    expect(await info).toBe(false)
  })
})
