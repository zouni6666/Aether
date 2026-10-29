import { translate } from '@/i18n'
import {
  Ban,
  CheckCircle2,
  ShieldCheck,
  UserCog,
  Wallet,
} from 'lucide-vue-next'
import type { Component } from 'vue'
import type { UserBatchAction, UserRole } from '@/api/users'
import type {
  UserFilterOption,
  UserFilterRole,
  UserFilterStatus,
  UserSortOption,
} from './user-management-types'

export interface UserBatchActionOption {
  value: UserBatchAction
  label: string
  description: string
  icon: Component
}

export const USER_ROLE_FILTER_OPTIONS: UserFilterOption<UserFilterRole>[] = [
  { value: 'all', get label() { return translate('ui.allRoles') } },
  { value: 'admin', get label() { return translate('auth.role.admin') } },
  { value: 'audit_admin', get label() { return translate('auth.role.auditAdmin') } },
  { value: 'user', get label() { return translate('ui.user') } },
]

export const USER_STATUS_FILTER_OPTIONS: UserFilterOption<UserFilterStatus>[] = [
  { value: 'all', get label() { return translate('ui.allStatuses') } },
  { value: 'active', get label() { return translate('ui.activee85e590') } },
  { value: 'inactive', get label() { return translate('ui.disable') } },
]

export const USER_SORT_OPTIONS: UserFilterOption<UserSortOption>[] = [
  { value: 'default', get label() { return translate('ui.defaultSort') } },
  { value: 'created_at_desc', get label() { return translate('ui.createdTimeNewestFirst') } },
  { value: 'created_at_asc', get label() { return translate('ui.createdTimeOldestFirst') } },
]

export const USER_BATCH_ACTION_OPTIONS: UserBatchActionOption[] = [
  {
    value: 'enable',
    get label() { return translate('ui.enablef4f0ead') },
    get description() { return translate('ui.restoreUserSignInAndApiAccess') },
    icon: CheckCircle2,
  },
  {
    value: 'disable',
    get label() { return translate('ui.disable') },
    get description() { return translate('ui.pauseUserAccess') },
    icon: Ban,
  },
  {
    value: 'update_access_control',
    get label() { return translate('poolQuota.claudeCode.unknownWindow') },
    get description() { return translate('ui.batchUpdateUserQuotaMode') },
    icon: ShieldCheck,
  },
  {
    value: 'update_role',
    get label() { return translate('ui.changeRole') },
    get description() { return translate('ui.batchSetUsersToUserOrAdminRoles') },
    icon: UserCog,
  },
  {
    value: 'adjust_wallet_balance',
    get label() { return translate('ui.adjustBalance') },
    get description() { return translate('ui.addOrDeductWalletBalancesInBulk') },
    icon: Wallet,
  },
]

export function formatUserRoleLabel(role: UserRole | string): string {
  if (role === 'admin') return translate('auth.role.admin')
  if (role === 'audit_admin') return translate('auth.role.auditAdmin')
  return translate('ui.user')
}

export function userRoleBadgeVariant(role: UserRole | string): 'default' | 'secondary' {
  return role === 'admin' ? 'default' : 'secondary'
}
