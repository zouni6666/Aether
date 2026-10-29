import { translate } from '@/i18n'
export type PoolHealthToggleKey =
  | 'probing_enabled'
  | 'account_self_check_enabled'
  | 'auto_remove_banned_keys'
  | 'auto_remove_quota_exhausted_keys'
  | 'ignore_exhausted_accounts'

export interface PoolHealthToggleCard {
  key: PoolHealthToggleKey
  label: string
  description: string
}

export interface PoolCooldownFieldLayout {
  fields: string[]
  desktopColumnsClass: string
}

export function buildPoolHealthToggleCards(): PoolHealthToggleCard[] {
  return [
    {
      key: 'probing_enabled',
      get label() { return translate('ui.adaptiveWarmPool') },
      get description() { return translate('ui.maintainAWarmPoolAndReplenishMissingCapacityAsynchronously') },
    },
    {
      key: 'account_self_check_enabled',
      get label() { return translate('ui.accountSelfCheck') },
      get description() { return translate('ui.checkAccountStatusPeriodicallyUsingTheProviderAdapterPolicy') },
    },
    {
      key: 'auto_remove_banned_keys',
      get label() { return translate('ui.removeInvalidAccountsAutomatically') },
      get description() { return translate('ui.removeAccountsFromThePoolWhenErrorsAreUnrecoverableOr') },
    },
    {
      key: 'auto_remove_quota_exhausted_keys',
      get label() { return translate('ui.removeExhaustedAccountsAutomatically') },
      get description() { return translate('ui.removeAccountsAutomaticallyWhenProbesDetectTheExhaustedQuotaStatus') },
    },
    {
      key: 'ignore_exhausted_accounts',
      get label() { return translate('feedback.pool.ignoreExhausted') },
      get description() { return translate('feedback.pool.ignoreExhaustedHint') },
    },
  ]
}

export function buildPoolCooldownFieldLayout(): PoolCooldownFieldLayout {
  return {
    fields: [
      'rate_limit_cooldown_seconds',
      'overload_cooldown_seconds',
    ],
    desktopColumnsClass: 'xl:grid-cols-2',
  }
}
