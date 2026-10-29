import { translate } from '@/i18n'
import { useConfirm } from './useConfirm'

/**
 * 关闭表单前的放弃确认
 *
 * 当表单存在未保存更改时弹出警告确认框，否则直接放行。
 */
export function useDiscardConfirm(isDirty: () => boolean) {
  const { confirmWarning } = useConfirm()

  async function confirmDiscard(): Promise<boolean> {
    if (!isDirty()) return true
    return confirmWarning(translate('ui.youHaveUnsavedChangesCloseWithoutSaving'), translate('ui.discardChanges'))
  }

  return { confirmDiscard }
}
