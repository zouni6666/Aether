import { computed, ref } from 'vue'
import { translate, type MessageKey } from '@/i18n'

export type ConfirmVariant = 'danger' | 'destructive' | 'warning' | 'info' | 'question'

export interface ConfirmOptions {
  title?: string
  message: string
  confirmText?: string
  cancelText?: string
  variant?: ConfirmVariant
}

interface ConfirmState extends ConfirmOptions {
  isOpen: boolean
  resolve?: (value: boolean) => void
  defaultTitleKey?: MessageKey
  defaultConfirmTextKey?: MessageKey
}

const state = ref<ConfirmState>({
  isOpen: false,
  message: '',
  variant: 'question'
})

export function useConfirm() {
  /**
   * 显示确认对话框
   * @param options 对话框选项
   * @returns Promise<boolean> - true表示确认，false表示取消
   */
  const openConfirm = (
    options: ConfirmOptions,
    defaultTitleKey: MessageKey = 'feedback.confirm.title',
    defaultConfirmTextKey: MessageKey = 'feedback.confirm.confirm',
  ): Promise<boolean> => {
    return new Promise((resolve) => {
      state.value = {
        isOpen: true,
        title: options.title,
        message: options.message,
        confirmText: options.confirmText,
        cancelText: options.cancelText,
        defaultTitleKey,
        defaultConfirmTextKey,
        variant: options.variant || 'question',
        resolve
      }
    })
  }

  const confirm = (options: ConfirmOptions): Promise<boolean> => openConfirm(options)

  /**
   * 便捷方法：危险操作确认（红色主题）
   */
  const confirmDanger = (message: string, title?: string, confirmText?: string): Promise<boolean> => {
    return openConfirm({
      message,
      title,
      confirmText,
      variant: 'danger'
    }, 'feedback.confirm.danger', 'feedback.confirm.delete')
  }

  /**
   * 便捷方法：警告确认（黄色主题）
   */
  const confirmWarning = (message: string, title?: string): Promise<boolean> => {
    return openConfirm({
      message,
      title,
      variant: 'warning'
    }, 'feedback.confirm.warning', 'feedback.confirm.continue')
  }

  /**
   * 便捷方法：信息确认（蓝色主题）
   */
  const confirmInfo = (message: string, title?: string): Promise<boolean> => {
    return openConfirm({
      message,
      title,
      variant: 'info'
    }, 'feedback.confirm.info', 'feedback.confirm.ok')
  }

  /**
   * 处理确认
   */
  const handleConfirm = () => {
    if (state.value.resolve) {
      state.value.resolve(true)
    }
    state.value.isOpen = false
  }

  /**
   * 处理取消
   */
  const handleCancel = () => {
    if (state.value.resolve) {
      state.value.resolve(false)
    }
    state.value.isOpen = false
  }

  return {
    state: computed(() => ({
      ...state.value,
      title: state.value.title ?? translate(state.value.defaultTitleKey ?? 'feedback.confirm.title'),
      confirmText: state.value.confirmText ?? translate(state.value.defaultConfirmTextKey ?? 'feedback.confirm.confirm'),
      cancelText: state.value.cancelText ?? translate('feedback.confirm.cancel'),
    })),
    confirm,
    confirmDanger,
    confirmWarning,
    confirmInfo,
    handleConfirm,
    handleCancel
  }
}
