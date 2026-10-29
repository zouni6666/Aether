import { translate } from '@/i18n'
export type PoolBatchActionValue =
  | 'edit_config'
  | 'export'
  | 'delete'
  | 'refresh_oauth'
  | 'refresh_quota'
  | 'clear_proxy'
  | 'set_proxy'
  | 'update_settings'
  | 'enable'
  | 'disable'

export interface PoolBatchActionOption {
  value: PoolBatchActionValue
  label: string
  hint: string
  destructive?: boolean
}

export const POOL_BATCH_ACTION_OPTIONS: readonly PoolBatchActionOption[] = [
  { value: 'edit_config', get label() { return translate('ui.editConfiguration') }, get hint() { return translate('ui.updateSupportedApisSchedulingParametersAndAutomaticModelFetchSettings') } },
  { value: 'refresh_quota', get label() { return translate('ui.refreshQuota') }, get hint() { return translate('ui.refreshQuotaInformationToCheckTheLatestStatus') } },
  { value: 'refresh_oauth', get label() { return translate('ui.refreshOauth') }, get hint() { return translate('ui.appliesOnlyToOauthAccountsOtherAccountTypesAreSkipped') } },
  { value: 'set_proxy', get label() { return translate('ui.configureProxy') }, get hint() { return translate('ui.assignDedicatedProxyNodesToTheSelectedAccounts') } },
  { value: 'update_settings', get label() { return translate('ui.moreSettings') }, get hint() { return translate('ui.selectivelyUpdateRpmConcurrencyCircuitBreakerNotesAndProxySettings') } },
  { value: 'clear_proxy', get label() { return translate('ui.clearProxy') }, get hint() { return translate('ui.removeAccountProxyOverridesAndUseTheProviderDefaultProxy') } },
  { value: 'enable', get label() { return translate('ui.enablef4f0ead') }, get hint() { return translate('ui.enableAccountsInBulkAndRestoreScheduling') } },
  { value: 'disable', get label() { return translate('ui.disable') }, get hint() { return translate('ui.disableAccountsInBulkRetainingDataButStoppingScheduling') } },
  { value: 'export', get label() { return translate('ui.exportCredentials') }, get hint() { return translate('ui.exportOauthCredentialsOnlyOtherAccountTypesAreSkipped') } },
  { value: 'delete', get label() { return translate('ui.deleteAccount') }, get hint() { return translate('ui.permanentlyDeleteAccountDataThisCannotBeUndone') }, destructive: true },
]
