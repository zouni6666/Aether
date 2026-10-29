import { translate } from '@/i18n'
export function proxyNodeDeleteSuccessMessage(
  clearedSystemProxy: boolean,
  clearedExternalModelsProxy: boolean,
): string {
  if (clearedSystemProxy && clearedExternalModelsProxy) {
    return translate('ui.proxyNodeDeletedTheSystemDefaultProxyAndExternalModel')
  }
  if (clearedSystemProxy) {
    return translate('ui.proxyNodeDeletedTheSystemDefaultProxyWasClearedAutomatically')
  }
  if (clearedExternalModelsProxy) {
    return translate('ui.proxyNodeDeletedTheExternalModelCatalogProxyWasCleared')
  }
  return translate('ui.proxyNodeDeleted')
}
