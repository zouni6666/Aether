<template>
  <div class="space-y-6 pb-8">
    <!-- 用户表格 -->
    <Card
      variant="default"
      class="overflow-hidden"
    >
      <UserManagementHeader
        :search-query="searchQuery"
        :filter-role="filterRole"
        :filter-group="filterGroup"
        :filter-status="filterStatus"
        :sort-option="sortOption"
        :user-groups="userGroups"
        :role-options="userRoleFilterOptions"
        :status-options="userStatusFilterOptions"
        :sort-options="userSortOptions"
        :loading="usersStore.loading"
        :can-operate-admin="authStore.canOperateAdmin"
        @update:search-query="searchQuery = $event"
        @update:filter-role="filterRole = $event"
        @update:filter-group="filterGroup = $event"
        @update:filter-status="filterStatus = $event"
        @update:sort-option="sortOption = $event"
        @open-groups="showUserGroupsDialog = true"
        @create-user="openCreateDialog"
        @refresh="handleManualRefresh"
      />

      <UserSelectionToolbar
        :is-all-filtered-selected="isAllFilteredSelected"
        :is-partially-filtered-selected="isPartiallyFilteredSelected"
        :filtered-user-count="filteredUserCount"
        :current-page-count="paginatedUsers.length"
        :selected-count="selectedCount"
        :is-current-page-fully-selected="isCurrentPageFullySelected"
        :can-clear-selection="canClearSelection"
        :select-all-filtered="selectAllFiltered"
        :loading="usersStore.loading"
        :can-operate-admin="authStore.canOperateAdmin"
        :group-count="userGroups.length"
        @toggle-select-filtered="toggleSelectFiltered"
        @toggle-select-current-page="toggleSelectCurrentPage"
        @clear-selection="clearSelection"
        @open-batch-dialog="openUserBatchDialog"
      />

      <UserManagementList
        :rows="userRows"
        :selected-id-set="selectedIdSet"
        :select-all-filtered="selectAllFiltered"
        :is-all-filtered-selected="isAllFilteredSelected"
        :is-partially-filtered-selected="isPartiallyFilteredSelected"
        :is-current-page-fully-selected="isCurrentPageFullySelected"
        :selection-disabled="selectAllFiltered || usersStore.loading"
        :loading="usersStore.loading"
        :can-operate-admin="authStore.canOperateAdmin"
        :has-filters="hasUserFilters"
        :sort-by="sortBy"
        :sort-order="sortOrder"
        @toggle-selected="toggleOne"
        @toggle-select-current-page="toggleSelectCurrentPage"
        @edit="editUser"
        @wallet="openWalletActionDialog"
        @plans="manageUserPlans"
        @api-keys="manageApiKeys"
        @sessions="manageUserSessions"
        @toggle-status="toggleUserStatus"
        @delete="deleteUser"
        @sort="handleTableSort"
      />

      <!-- 分页控件 -->
      <Pagination
        :current="currentPage"
        :total="filteredUserCount"
        :page-size="pageSize"
        cache-key="users-page-size"
        @update:current="handlePageChange"
        @update:page-size="handlePageSizeChange"
      />
    </Card>

    <!-- 用户表单对话框（创建/编辑共用） -->
    <UserFormDialog
      ref="userFormDialogRef"
      :open="showUserFormDialog"
      :user="editingUser"
      :groups="userGroups"
      @close="closeUserFormDialog"
      @submit="handleUserFormSubmit"
    />

    <UserBatchActionDialog
      :open="showUserBatchDialog"
      :selected-ids="selectedIds"
      :select-all-filtered="selectAllFiltered"
      :selected-count="selectedCount"
      :filters="batchSelectionFilters"
      :groups="userGroups"
      @close="showUserBatchDialog = false"
      @completed="handleUserBatchCompleted"
    />

    <UserGroupsDialog
      :open="showUserGroupsDialog"
      :users-version="userOptionsVersion"
      @close="showUserGroupsDialog = false"
      @changed="handleUserGroupsChanged"
    />

    <UserPlanDialog
      :open="showUserPlansDialog"
      :user-id="selectedUser?.id || null"
      :user-name="selectedUser?.username || ''"
      :entitlements="userPlanEntitlements"
      :plans="grantableBillingPlans"
      :selected-plan-id="selectedGrantPlanId"
      :grant-reason="grantReason"
      :loading-entitlements="loadingUserPlans"
      :loading-plans="loadingBillingPlans"
      :granting="grantingUserPlan"
      :revoking-entitlement-id="revokingUserPlanEntitlementId"
      :format-date-time="formatDateTime"
      :format-plan-price="formatPlanPrice"
      :format-plan-duration="formatPlanDuration"
      :entitlement-labels="entitlementLabels"
      @close="showUserPlansDialog = false"
      @update:selected-plan-id="selectedGrantPlanId = $event"
      @update:grant-reason="grantReason = $event"
      @refresh-entitlements="loadUserPlanEntitlements"
      @grant="grantPlanToSelectedUser"
      @revoke="revokePlanFromSelectedUser"
    />

    <UserApiKeysDialog
      :open="showApiKeysDialog"
      :api-keys="userApiKeys"
      :creating="creatingApiKey"
      :format-rate-limit="formatRateLimitSimple"
      :format-concurrent-limit="formatConcurrentLimitSimple"
      :format-ip-rules="formatIpRules"
      @close="closeApiKeysDialog"
      @create-key="openCreateUserApiKeyDialog"
      @edit-key="openEditUserApiKeyDialog"
      @toggle-lock="toggleLockApiKey"
      @delete-key="deleteApiKey"
      @copy-full-key="copyFullKey"
    />

    <UserApiKeyFormDialog
      :open="showUserApiKeyFormDialog"
      :form="userApiKeyForm"
      :is-editing="Boolean(editingUserApiKey)"
      :creating="creatingApiKey"
      @close="closeUserApiKeyFormDialog"
      @update:form="userApiKeyForm = $event"
      @submit="submitUserApiKeyForm"
    />

    <UserSessionsDialog
      :open="showUserSessionsDialog"
      :sessions="userSessions"
      :loading="loadingUserSessions"
      :action-loading="sessionDialogActionLoading"
      :format-date="formatDateShort"
      :format-session-meta="formatSessionMeta"
      @close="showUserSessionsDialog = false"
      @revoke-session="revokeSelectedUserSession"
      @revoke-all="revokeAllSelectedUserSessions"
    />

    <WalletOpsDrawer
      :open="showWalletActionDialogState"
      :wallet="walletActionTarget?.wallet || null"
      :owner-name="walletActionTarget?.user.username || ''"
      :owner-subtitle="walletActionTarget?.user.email || t('ui.noEmailSet')"
      :context-label="t('ui.userWallet')"
      accent="emerald"
      @close="closeWalletActionDrawer"
      @changed="handleWalletDrawerChanged"
    />

    <NewApiKeyDialog
      :open="showNewApiKeyDialog"
      :api-key="newApiKey"
      @close="closeNewApiKeyDialog"
      @copy="copyApiKey"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useUsersStore } from '@/stores/users'
import { useAuthStore } from '@/stores/auth'
import {
  usersApi,
  type User,
  type ApiKey,
  type UserSession,
  type UserBatchActionResponse,
  type UserBatchSelectionFilters,
  type UserGroup,
  type AdminUserPlanEntitlement,
  type AdminUserSortBy,
  type AdminUserSortOrder,
} from '@/api/users'
import { formatSessionMeta } from '@/types/session'
import { adminWalletApi, type AdminWallet } from '@/api/admin-wallets'
import { adminBillingPlansApi, type BillingEntitlement, type BillingPlan } from '@/api/billing'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { useClipboard } from '@/composables/useClipboard'
import { adminApi } from '@/api/admin'
import { walletStatusBadge, walletStatusLabel } from '@/features/wallet/utils/walletDisplay'

// UI 组件
import {
  Card,
  Pagination,
} from '@/components/ui'

// 功能组件
import NewApiKeyDialog from '@/features/users/components/NewApiKeyDialog.vue'
import UserApiKeyFormDialog, { type UserApiKeyFormState } from '@/features/users/components/UserApiKeyFormDialog.vue'
import UserApiKeysDialog from '@/features/users/components/UserApiKeysDialog.vue'
import UserFormDialog, { type UserFormData } from '@/features/users/components/UserFormDialog.vue'
import UserBatchActionDialog from '@/features/users/components/UserBatchActionDialog.vue'
import UserGroupsDialog from '@/features/users/components/UserGroupsDialog.vue'
import UserManagementHeader from '@/features/users/components/UserManagementHeader.vue'
import UserManagementList from '@/features/users/components/UserManagementList.vue'
import UserPlanDialog from '@/features/users/components/UserPlanDialog.vue'
import UserSelectionToolbar from '@/features/users/components/UserSelectionToolbar.vue'
import UserSessionsDialog from '@/features/users/components/UserSessionsDialog.vue'
import {
  buildApiKeyRedactionFeatureSettingsPatch,
  resolveApiKeyRedactionFormState,
} from '@/features/users/apiKeyFeatureSettings'
import type { UserManagementRow } from '@/features/users/components/user-management-types'
import {
  USER_ROLE_FILTER_OPTIONS,
  USER_SORT_OPTIONS,
  USER_STATUS_FILTER_OPTIONS,
  formatUserRoleLabel,
  userRoleBadgeVariant,
} from '@/features/users/components/user-management-config'
import WalletOpsDrawer from '@/features/wallet/components/WalletOpsDrawer.vue'
import { parseApiError } from '@/utils/errorParser'
import {
  entitlementsWillReplaceExisting,
  isPlanEntitlementReplacementCandidate,
  billingEntitlementLabels,
  formatBillingPlanDuration,
} from '@/utils/billingEntitlements'
import {
  formatTokens,
  formatRateLimitInheritable,
  formatRateLimitSimple,
  isRateLimitInherited,
  isRateLimitUnlimited,
  formatDateShort,
  formatDateTime,
  formatNumber,
} from '@/utils/format'
import { log } from '@/utils/logger'
import { useBatchSelection } from '@/composables/useBatchSelection'
import { useI18n } from '@/i18n'

const { success, error } = useToast()
const { confirmDanger } = useConfirm()
const { copyToClipboard } = useClipboard()
const { t, locale } = useI18n()
const usersStore = useUsersStore()
const authStore = useAuthStore()

function localizedApiError(err: unknown, fallback: string): string {
  return parseApiError(err, fallback)
}

// 用户表单对话框状态
const showUserFormDialog = ref(false)
const editingUser = ref<UserFormData | null>(null)
const userFormDialogRef = ref<InstanceType<typeof UserFormDialog>>()

// API Keys 对话框状态
const showApiKeysDialog = ref(false)
const showUserSessionsDialog = ref(false)
const showUserPlansDialog = ref(false)
const showNewApiKeyDialog = ref(false)
const showUserApiKeyFormDialog = ref(false)
const selectedUser = ref<User | null>(null)
const userApiKeys = ref<ApiKey[]>([])
const userSessions = ref<UserSession[]>([])
const userPlanEntitlements = ref<AdminUserPlanEntitlement[]>([])
const availableBillingPlans = ref<BillingPlan[]>([])
const selectedGrantPlanId = ref('')
const grantReason = ref('')
const newApiKey = ref('')
const creatingApiKey = ref(false)
const loadingUserSessions = ref(false)
const loadingUserPlans = ref(false)
const loadingBillingPlans = ref(false)
const grantingUserPlan = ref(false)
const revokingUserPlanEntitlementId = ref<string | null>(null)
const sessionDialogActionLoading = ref<string | null>(null)
const editingUserApiKey = ref<ApiKey | null>(null)
const userApiKeyForm = ref<UserApiKeyFormState>({
  name: '',
  rate_limit: undefined,
  concurrent_limit: undefined,
  ip_rules_text: '',
  chat_pii_redaction_mode: 'inherit',
  chat_pii_redaction_enabled: false,
  chat_pii_redaction_placeholder_notice: true,
})

// 用户统计
const userWalletMap = ref<Record<string, AdminWallet>>({})

const showWalletActionDialogState = ref(false)
const walletActionTarget = ref<{ user: User; wallet: AdminWallet } | null>(null)
const showUserBatchDialog = ref(false)
const showUserGroupsDialog = ref(false)
const userOptionsVersion = ref(0)

const searchQuery = ref('')
const filterRole = ref<'all' | User['role']>('all')
const filterStatus = ref<'all' | 'active' | 'inactive'>('all')
const filterGroup = ref('all')
const sortOption = ref<'default' | 'created_at_desc' | 'created_at_asc'>('default')
const userGroups = ref<UserGroup[]>([])
const userRoleFilterOptions = USER_ROLE_FILTER_OPTIONS
const userStatusFilterOptions = USER_STATUS_FILTER_OPTIONS
const userSortOptions = USER_SORT_OPTIONS
const sortBy = computed<AdminUserSortBy | null>(() =>
  sortOption.value === 'default' ? null : 'created_at'
)
const sortOrder = computed<AdminUserSortOrder>(() =>
  sortOption.value === 'created_at_asc' ? 'asc' : 'desc'
)

const currentPage = ref(1)
const pageSize = ref(20)
const USERS_PAGE_CACHE_TTL_MS = 10 * 1000
const USER_WALLETS_CACHE_TTL_MS = 10 * 1000
const USERS_SEARCH_DEBOUNCE_MS = 300
let userWalletsRequestId = 0
let userApiKeysRequestId = 0
let userApiKeyMutationRequestId = 0
let usersSearchDebounceTimer: ReturnType<typeof setTimeout> | null = null

const filteredUsers = computed(() => usersStore.users)

const paginatedUsers = computed(() => filteredUsers.value)

const filteredUserCount = computed(() => usersStore.total)
const {
  selectedIds,
  selectAllFiltered,
  selectedIdSet,
  selectedCount,
  isAllFilteredSelected,
  isPartiallyFilteredSelected,
  isCurrentPageFullySelected,
  canClearSelection,
  rememberItems: rememberBatchPageUsers,
  resetSelection: resetBatchSelection,
  toggleOne,
  toggleSelectFiltered,
  toggleSelectCurrentPage,
  clearSelection,
} = useBatchSelection<User>({
  pageItems: paginatedUsers,
  filteredTotal: filteredUserCount,
  getItemId: (user) => user.id,
})

const batchSelectionFilters = computed<UserBatchSelectionFilters>(() => {
  const filters: UserBatchSelectionFilters = {}
  const search = searchQuery.value.trim()
  if (search) filters.search = search
  if (filterRole.value === 'admin' || filterRole.value === 'audit_admin' || filterRole.value === 'user') filters.role = filterRole.value
  if (filterStatus.value === 'active') filters.is_active = true
  if (filterStatus.value === 'inactive') filters.is_active = false
  if (filterGroup.value !== 'all') filters.group_id = filterGroup.value
  return filters
})

const grantableBillingPlans = computed(() =>
  availableBillingPlans.value.filter((plan) => hasPackageEntitlement(plan.entitlements))
)

const hasUserFilters = computed(() =>
  Boolean(searchQuery.value.trim())
  || filterRole.value !== 'all'
  || filterStatus.value !== 'all'
  || filterGroup.value !== 'all'
)

const userRows = computed<UserManagementRow[]>(() =>
  paginatedUsers.value.map((user) => {
    const totalBalance = getUserWalletTotalBalance(user)
    const walletStatus = getUserWalletStatus(user.id)
    return {
      user,
      roleLabel: formatUserRoleLabel(user.role),
      roleBadgeVariant: userRoleBadgeVariant(user.role),
      isUnlimited: isUserUnlimited(user),
      hasWallet: Boolean(getUserWallet(user.id)),
      totalBalanceLabel: formatCurrencyValue(totalBalance, '-'),
      packageBalanceLabel: formatCurrencyValue(getUserPackageBalance(user), '$0.00'),
      walletBalanceLabel: formatCurrencyValue(getUserWalletBalance(user), '$0.00'),
      consumedLabel: `$${getUserWalletConsumed(user).toFixed(2)}`,
      isNegativeBalance: isNegativeWalletValue(totalBalance),
      walletStatusLabel: walletStatusLabel(walletStatus),
      walletStatusVariant: walletStatusBadge(walletStatus),
      requestCountLabel: formatNumber(user.request_count),
      tokensLabel: formatTokens(user.total_tokens ?? 0),
      rateLimitLabel: formatRateLimitInheritable(user.rate_limit),
      rateLimitSource: formatUserEffectiveRateLimitSource(user),
      rateLimitAsBadge: isRateLimitInherited(user.rate_limit) || isRateLimitUnlimited(user.rate_limit),
      createdAtLabel: formatDateShort(user.created_at),
      statusLabel: user.is_active ? t('ui.activee85e590') : t('ui.disable'),
      statusVariant: user.is_active ? 'success' : 'destructive',
    }
  })
)

function resetUserListForFilterChange() {
  currentPage.value = 1
  resetBatchSelection()
}

function clearUsersSearchDebounce() {
  if (usersSearchDebounceTimer) {
    clearTimeout(usersSearchDebounceTimer)
    usersSearchDebounceTimer = null
  }
}

watch(searchQuery, () => {
  resetUserListForFilterChange()
  clearUsersSearchDebounce()
  usersSearchDebounceTimer = setTimeout(() => {
    usersSearchDebounceTimer = null
    void refreshUsers()
  }, USERS_SEARCH_DEBOUNCE_MS)
})

watch([filterRole, filterStatus, filterGroup, sortOption], () => {
  resetUserListForFilterChange()
  clearUsersSearchDebounce()
  void refreshUsers()
})

watch(paginatedUsers, (users) => rememberBatchPageUsers(users), { immediate: true })

onMounted(() => {
  void refreshUsers({ preferCache: true }).then(() =>
    loadUserWallets({ cacheTtlMs: USER_WALLETS_CACHE_TTL_MS })
  )
  void loadUserGroups()
})

onBeforeUnmount(() => {
  clearUsersSearchDebounce()
  userWalletsRequestId += 1
  userApiKeysRequestId += 1
  userApiKeyMutationRequestId += 1
})

async function refreshUsers(options: { preferCache?: boolean } = {}) {
  const cacheTtlMs = options.preferCache ? USERS_PAGE_CACHE_TTL_MS : 0
  const search = searchQuery.value.trim()
  await usersStore.fetchUsers({
    cacheTtlMs,
    search: search || undefined,
    role: filterRole.value === 'all' ? undefined : filterRole.value,
    is_active: filterStatus.value === 'all' ? undefined : filterStatus.value === 'active',
    group_id: filterGroup.value === 'all' ? undefined : filterGroup.value,
    sort_by: sortBy.value ?? undefined,
    sort_order: sortBy.value ? sortOrder.value : undefined,
    skip: (currentPage.value - 1) * pageSize.value,
    limit: pageSize.value,
  })
}

async function handleManualRefresh() {
  clearUsersSearchDebounce()
  await Promise.all([
    refreshUsers(),
    loadUserGroups(),
    loadUserWallets(),
  ])
}

function handleTableSort(payload: { key: string, direction: AdminUserSortOrder }): void {
  if (payload.key !== 'created_at') return
  sortOption.value = payload.direction === 'asc' ? 'created_at_asc' : 'created_at_desc'
}

function handlePageChange(page: number): void {
  currentPage.value = page
  void refreshUsers({ preferCache: true })
}

function handlePageSizeChange(size: number): void {
  pageSize.value = size
  currentPage.value = 1
  resetBatchSelection()
  void refreshUsers()
}

async function loadUserGroups(): Promise<void> {
  try {
    const response = await usersStore.listUserGroups()
    userGroups.value = response.items
    if (filterGroup.value !== 'all' && !userGroups.value.some((group) => group.id === filterGroup.value)) {
      filterGroup.value = 'all'
    }
  } catch (err) {
    log.error('加载用户分组失败:', err)
  }
}

async function handleUserGroupsChanged(): Promise<void> {
  await Promise.all([refreshUsers(), loadUserGroups()])
}

function openUserBatchDialog(): void {
  if (selectedCount.value === 0 && userGroups.value.length === 0) return
  showUserBatchDialog.value = true
}

async function handleUserBatchCompleted(_result: UserBatchActionResponse): Promise<void> {
  await Promise.all([refreshUsers(), loadUserWallets()])
  resetBatchSelection(true)
}

function invalidateUserOptions(): void {
  userOptionsVersion.value += 1
}

function formatPlanPrice(plan: BillingPlan): string {
  return `${Number(plan.price_amount || 0).toFixed(2)} ${plan.price_currency || 'CNY'}`
}

function formatPlanDuration(plan: BillingPlan): string {
  return formatBillingPlanDuration(plan.duration_unit, Number(plan.duration_value || 1))
}

function entitlementLabels(items: BillingEntitlement[] | undefined): string[] {
  return billingEntitlementLabels(items, t)
}

function hasPackageEntitlement(items: BillingEntitlement[] | undefined): boolean {
  return (items || []).some((item) =>
    item.type === 'daily_quota'
    || item.type === 'membership_group'
    || item.type === 'usage_policy'
  )
}

async function loadUserWallets(options: { cacheTtlMs?: number } = {}) {
  const requestId = ++userWalletsRequestId
  try {
    const wallets = await adminWalletApi.listAllWallets(
      { owner_type: 'user' },
      { cacheTtlMs: options.cacheTtlMs ?? 0 },
    )
    if (requestId !== userWalletsRequestId) return
    userWalletMap.value = wallets
      .filter((wallet) => !!wallet.user_id)
      .reduce<Record<string, AdminWallet>>((acc, wallet) => {
        acc[wallet.user_id as string] = wallet
        return acc
      }, {})
  } catch (err) {
    if (requestId !== userWalletsRequestId) return
    log.error('加载用户钱包失败:', err)
  }
}

function getUserWallet(userId: string): AdminWallet | null {
  return userWalletMap.value[userId] || null
}

function isUserUnlimited(user: User): boolean {
  const wallet = getUserWallet(user.id)
  if (wallet?.limit_mode === 'unlimited' || wallet?.unlimited === true) {
    return true
  }
  return Boolean(user.unlimited)
}

function getUserWalletTotalBalance(user: User): number | null {
  if (isUserUnlimited(user)) {
    return null
  }
  const wallet = getUserWallet(user.id)
  if (!wallet) {
    return null
  }
  if (typeof wallet.total_available_balance === 'number' && Number.isFinite(wallet.total_available_balance)) {
    return wallet.total_available_balance
  }
  return getUserWalletBalance(user) + getUserPackageBalance(user)
}

function getUserWalletBalance(user: User): number {
  const wallet = getUserWallet(user.id)
  const value = wallet?.wallet_balance ?? wallet?.balance ?? 0
  return Number.isFinite(value) ? value : 0
}

function getUserPackageBalance(user: User): number {
  const value = getUserWallet(user.id)?.package_balance ?? 0
  return Number.isFinite(value) ? value : 0
}

function getUserWalletConsumed(user: User): number {
  return getUserWallet(user.id)?.total_consumed ?? 0
}

function getUserWalletStatus(userId: string): string | null {
  return getUserWallet(userId)?.status ?? null
}

function formatCurrencyValue(value: number | null, nullLabel = '-'): string {
  if (value == null) {
    return nullLabel
  }
  return `$${value.toFixed(2)}`
}

function formatConcurrentLimitSimple(concurrentLimit?: number | null): string {
  if (concurrentLimit == null || concurrentLimit === 0) {
    return t('ui.unlimitedConcurrency')
  }
  return t('users.concurrentLimit', { limit: concurrentLimit })
}

function formatIpRules(ipRules?: string[] | null): string {
  return ipRules && ipRules.length > 0 ? ipRules.join(', ') : t('ui.noRestriction')
}

function parseIpRulesInput(value: string): string[] | null {
  const items = value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean)
  return items.length > 0 ? items : null
}

function formatUserEffectiveRateLimitSource(user: User): string {
  const source = user.effective_policy?.rate_limit
  if (!source) return ''
  if (source.source === 'group' && source.group_name) {
    return `${t('ui.inheritedFromGroup')}${source.group_name}`
  }
  if (source.source === 'combined') {
    const groupNames = Array.isArray(source.group_names) ? source.group_names.join(locale.value === 'en-US' ? ', ' : '、') : ''
    return groupNames ? `${t('ui.userExtraLimitCombinedWithGroups')}${groupNames}` : t('ui.userExtraLimitCombinedWithGroupsccb5591')
  }
  if (source.source === 'user') {
    return t('ui.userSpecificConfiguration')
  }
  return t('ui.systemDefault')
}

function isNegativeWalletValue(value: number | null): boolean {
  return typeof value === 'number' && value < 0
}

async function toggleUserStatus(user: User) {
  const disabling = user.is_active
  const confirmed = await confirmDanger(
    disabling
      ? t('users.disable.confirmMessage', { username: user.username })
      : t('users.enable.confirmMessage', { username: user.username }),
    disabling ? t('users.disable.confirmTitle') : t('users.enable.confirmTitle'),
    disabling ? t('users.disable.confirmButton') : t('users.enable.confirmButton'),
  )

  if (!confirmed) return

  try {
    await usersStore.updateUser(user.id, { is_active: !disabling })
    invalidateUserOptions()
    await refreshUsers()
    success(disabling ? t('users.disable.success') : t('users.enable.success'))
  } catch (err: unknown) {
    const fallback = disabling ? t('users.disable.failure') : t('users.enable.failure')
    error(localizedApiError(err, fallback), fallback)
  }
}

// ========== 用户表单对话框方法 ==========

function openCreateDialog() {
  editingUser.value = null
  showUserFormDialog.value = true
}

function editUser(user: User) {
  // 创建数组副本，避免与 store 数据共享引用
  editingUser.value = {
    id: user.id,
    username: user.username,
    email: user.email ?? '',
    unlimited: user.unlimited,
    role: user.role,
    is_active: user.is_active,
    group_ids: (user.groups || []).map(group => group.id),
    feature_settings: user.feature_settings ?? null,
  }
  showUserFormDialog.value = true
}

function closeUserFormDialog() {
  showUserFormDialog.value = false
  editingUser.value = null
}

async function handleUserFormSubmit(data: UserFormData & { password?: string; unlimited?: boolean }) {
  userFormDialogRef.value?.setSaving(true)
  try {
    if (data.id) {
      // 更新用户
      const updateData: Record<string, unknown> = {
        username: data.username,
        email: data.email || undefined,
        unlimited: data.unlimited,
        role: data.role,
        group_ids: data.group_ids ?? [],
        feature_settings: data.feature_settings ?? null,
      }
      if (data.password) {
        updateData.password = data.password
      }
      await usersStore.updateUser(data.id, updateData)
      invalidateUserOptions()
      success(t('ui.userInformationUpdated'))
    } else {
      // 创建用户
      const newUser = await usersStore.createUser({
        username: data.username,
        password: data.password ?? '',
        email: data.email || undefined,
        initial_gift_usd: data.initial_gift_usd,
        unlimited: data.unlimited,
        role: data.role,
        group_ids: data.group_ids ?? [],
        feature_settings: data.feature_settings ?? null,
      })
      // 如果创建时指定为禁用，则更新状态
      if (data.is_active === false && newUser) {
        await usersStore.updateUser(newUser.id, { is_active: false })
      }
      invalidateUserOptions()
      success(t('ui.userCreated'))
    }
    closeUserFormDialog()
    await Promise.all([refreshUsers(), loadUserWallets()])
  } catch (err: unknown) {
    const title = data.id ? t('ui.failedToUpdateUser') : t('ui.failedToCreateUser')
    error(localizedApiError(err, title), title)
  } finally {
    userFormDialogRef.value?.setSaving(false)
  }
}

async function manageApiKeys(user: User) {
  userApiKeyMutationRequestId += 1
  creatingApiKey.value = false
  selectedUser.value = user
  userApiKeys.value = []
  showApiKeysDialog.value = true
  await loadUserApiKeys(user.id)
}

function closeApiKeysDialog() {
  userApiKeyMutationRequestId += 1
  creatingApiKey.value = false
  showApiKeysDialog.value = false
  userApiKeys.value = []
  userApiKeysRequestId += 1
}

async function manageUserSessions(user: User) {
  selectedUser.value = user
  showUserSessionsDialog.value = true
  loadingUserSessions.value = true
  try {
    userSessions.value = await usersStore.getUserSessions(user.id)
  } catch (err) {
    error(localizedApiError(err, t('ui.failedToLoadUserDeviceSessions')), t('ui.failedToLoadUserDeviceSessions'))
  } finally {
    loadingUserSessions.value = false
  }
}

async function manageUserPlans(user: User) {
  selectedUser.value = user
  showUserPlansDialog.value = true
  revokingUserPlanEntitlementId.value = null
  selectedGrantPlanId.value = ''
  grantReason.value = ''
  await Promise.all([
    loadUserPlanEntitlements(user.id),
    loadAvailableBillingPlans(),
  ])
  if (!selectedGrantPlanId.value && grantableBillingPlans.value.length > 0) {
    selectedGrantPlanId.value = grantableBillingPlans.value[0].id
  }
}

async function loadUserPlanEntitlements(userId: string) {
  loadingUserPlans.value = true
  try {
    const response = await usersApi.listUserPlanEntitlements(userId)
    userPlanEntitlements.value = response.items
  } catch (err) {
    error(localizedApiError(err, t('ui.failedToLoadUserPlans')), t('ui.failedToLoadUserPlans'))
    userPlanEntitlements.value = []
  } finally {
    loadingUserPlans.value = false
  }
}

async function loadAvailableBillingPlans() {
  loadingBillingPlans.value = true
  try {
    const response = await adminBillingPlansApi.list()
    availableBillingPlans.value = response.items
    if (
      selectedGrantPlanId.value
      && !response.items.some((plan) => plan.id === selectedGrantPlanId.value)
    ) {
      selectedGrantPlanId.value = ''
    }
  } catch (err) {
    error(localizedApiError(err, t('ui.failedToLoadPlanList')), t('ui.failedToLoadPlanList'))
    availableBillingPlans.value = []
  } finally {
    loadingBillingPlans.value = false
  }
}

async function grantPlanToSelectedUser() {
  if (!selectedUser.value || !selectedGrantPlanId.value) return
  const selectedPlan = availableBillingPlans.value.find(
    plan => plan.id === selectedGrantPlanId.value,
  )
  const replacesExisting = selectedPlan && userPlanEntitlements.value.some(item =>
    isPlanEntitlementReplacementCandidate(item)
    && entitlementsWillReplaceExisting(selectedPlan.entitlements, item.entitlements)
  )
  if (replacesExisting) {
    const confirmed = await confirmDanger(
      t('ui.grantingThisPlanWillReplaceConflictingPlansAndAllTheir'),
      t('ui.confirmPlanReplacement'),
      t('ui.grantPlan'),
    )
    if (!confirmed) return
  }
  grantingUserPlan.value = true
  try {
    const response = await usersApi.grantUserPlan(selectedUser.value.id, {
      plan_id: selectedGrantPlanId.value,
      reason: grantReason.value.trim() || null,
    })
    userPlanEntitlements.value = response.items
    grantReason.value = ''
    success(t('ui.planGranted'))
  } catch (err) {
    error(localizedApiError(err, t('ui.failedToGrantPlan')), t('ui.failedToGrantPlan'))
  } finally {
    grantingUserPlan.value = false
  }
}

async function revokePlanFromSelectedUser(entitlement: AdminUserPlanEntitlement) {
  if (!selectedUser.value || revokingUserPlanEntitlementId.value) return
  const userId = selectedUser.value.id
  const planTitle = entitlement.plan_title || entitlement.plan?.title || entitlement.plan_id
  const confirmed = await confirmDanger(
    `${planTitle}\n\n${t('ui.theUserWillImmediatelyLoseThePlanSRemainingQuota')}`,
    t('ui.revokeUserPlan'),
    t('ui.confirmRevoke'),
  )
  if (!confirmed) return
  revokingUserPlanEntitlementId.value = entitlement.id
  try {
    const response = await usersApi.revokeUserPlanEntitlement(
      userId,
      entitlement.id,
    )
    if (selectedUser.value?.id === userId) {
      userPlanEntitlements.value = response.items
    }
    success(t('ui.planRevoked'))
  } catch (err) {
    error(localizedApiError(err, t('ui.failedToRevokePlan')), t('ui.failedToRevokePlan'))
  } finally {
    revokingUserPlanEntitlementId.value = null
  }
}

async function loadUserApiKeys(userId: string) {
  const requestId = ++userApiKeysRequestId
  try {
    const apiKeys = await usersStore.getUserApiKeys(userId)
    if (
      requestId !== userApiKeysRequestId
      || selectedUser.value?.id !== userId
      || !showApiKeysDialog.value
    ) return
    userApiKeys.value = apiKeys
  } catch (err) {
    if (
      requestId !== userApiKeysRequestId
      || selectedUser.value?.id !== userId
      || !showApiKeysDialog.value
    ) return
    log.error('加载API Keys失败:', err)
    userApiKeys.value = []
  }
}

function openCreateUserApiKeyDialog() {
  const redactionFeature = resolveApiKeyRedactionFormState(
    null,
    selectedUser.value?.feature_settings,
  )
  userApiKeyForm.value = {
    name: `Key-${new Date().toISOString().split('T')[0]}`,
    rate_limit: undefined,
    concurrent_limit: undefined,
    ip_rules_text: '',
    chat_pii_redaction_mode: redactionFeature.mode,
    chat_pii_redaction_enabled: redactionFeature.enabled,
    chat_pii_redaction_placeholder_notice: redactionFeature.inject_model_instruction,
  }
  editingUserApiKey.value = null
  showUserApiKeyFormDialog.value = true
}

function openEditUserApiKeyDialog(apiKey: ApiKey) {
  const redactionFeature = resolveApiKeyRedactionFormState(
    apiKey.feature_settings,
    selectedUser.value?.feature_settings,
  )
  editingUserApiKey.value = apiKey
  userApiKeyForm.value = {
    name: apiKey.name || '',
    rate_limit: apiKey.rate_limit ?? undefined,
    concurrent_limit: apiKey.concurrent_limit ?? undefined,
    ip_rules_text: apiKey.ip_rules?.join(', ') ?? '',
    chat_pii_redaction_mode: redactionFeature.mode,
    chat_pii_redaction_enabled: redactionFeature.enabled,
    chat_pii_redaction_placeholder_notice: redactionFeature.inject_model_instruction,
  }
  showUserApiKeyFormDialog.value = true
}

function closeUserApiKeyFormDialog() {
  if (creatingApiKey.value) {
    userApiKeyMutationRequestId += 1
    creatingApiKey.value = false
  }
  showUserApiKeyFormDialog.value = false
  editingUserApiKey.value = null
  userApiKeyForm.value = {
    name: '',
    rate_limit: undefined,
    concurrent_limit: undefined,
    ip_rules_text: '',
    chat_pii_redaction_mode: 'inherit',
    chat_pii_redaction_enabled: false,
    chat_pii_redaction_placeholder_notice: true,
  }
}

async function submitUserApiKeyForm() {
  if (!selectedUser.value) return
  if (!userApiKeyForm.value.name.trim()) {
    error(t('ui.enterAKeyName'), editingUserApiKey.value ? t('ui.failedToUpdateApiKeydce5430') : t('ui.failedToCreateApiKey438ebef'))
    return
  }

  const targetUserId = selectedUser.value.id
  const editingApiKey = editingUserApiKey.value
  const form = { ...userApiKeyForm.value }
  const mutationRequestId = ++userApiKeyMutationRequestId
  const mutationIsCurrent = () => (
    mutationRequestId === userApiKeyMutationRequestId
    && selectedUser.value?.id === targetUserId
    && showApiKeysDialog.value
  )

  creatingApiKey.value = true
  try {
    const ipRules = parseIpRulesInput(form.ip_rules_text)
    const featureSettingsPatch = buildApiKeyRedactionFeatureSettingsPatch({
      isEditing: Boolean(editingApiKey),
      currentFeatureSettings: editingApiKey?.feature_settings,
      mode: form.chat_pii_redaction_mode,
      value: {
        enabled: form.chat_pii_redaction_enabled,
        inject_model_instruction: form.chat_pii_redaction_placeholder_notice,
      },
    })
    if (editingApiKey) {
      await usersStore.updateApiKey(targetUserId, editingApiKey.id, {
        name: form.name,
        rate_limit: form.rate_limit ?? 0,
        concurrent_limit: form.concurrent_limit,
        ip_rules: ipRules,
        ...featureSettingsPatch,
      })
      if (!mutationIsCurrent()) return
      success(t('ui.apiKeyUpdated3b4d973'))
    } else {
      const response = await usersStore.createApiKey(targetUserId, {
        name: form.name,
        rate_limit: form.rate_limit ?? 0,
        concurrent_limit: form.concurrent_limit,
        ip_rules: ipRules,
        ...featureSettingsPatch,
      })
      if (!mutationIsCurrent()) return
      newApiKey.value = response.key || ''
      showNewApiKeyDialog.value = true
      success(t('ui.apiKeyCreated16574ff'))
    }
    await loadUserApiKeys(targetUserId)
    if (!mutationIsCurrent()) return
    closeUserApiKeyFormDialog()
  } catch (err: unknown) {
    if (mutationIsCurrent()) {
      error(localizedApiError(err, t('ui.unknownError')), editingApiKey ? t('ui.failedToUpdateApiKeydce5430') : t('ui.failedToCreateApiKey438ebef'))
    }
  } finally {
    if (mutationRequestId === userApiKeyMutationRequestId) {
      creatingApiKey.value = false
    }
  }
}

async function revokeSelectedUserSession(sessionId: string) {
  if (!selectedUser.value) return
  sessionDialogActionLoading.value = sessionId
  try {
    await usersStore.revokeUserSession(selectedUser.value.id, sessionId)
    userSessions.value = userSessions.value.filter((session) => session.id !== sessionId)
    success(t('ui.deviceSignedOuteb86c77'))
  } catch (err) {
    error(localizedApiError(err, t('ui.failedToSignOutDevicedc79407')), t('ui.failedToSignOutDevicedc79407'))
  } finally {
    sessionDialogActionLoading.value = null
  }
}

async function revokeAllSelectedUserSessions() {
  if (!selectedUser.value) return
  sessionDialogActionLoading.value = 'all'
  try {
    const result = await usersStore.revokeAllUserSessions(selectedUser.value.id)
    userSessions.value = []
    success(result.revoked_count > 0
      ? t('ui.signedOutValueDevices', { count: result.revoked_count })
      : t('ui.noDevicesToSignOut'))
  } catch (err) {
    error(localizedApiError(err, t('ui.failedToSignOutAllDevices')), t('ui.failedToSignOutAllDevices'))
  } finally {
    sessionDialogActionLoading.value = null
  }
}

async function copyApiKey() {
  await copyToClipboard(newApiKey.value)
}

async function closeNewApiKeyDialog() {
  showNewApiKeyDialog.value = false
  newApiKey.value = ''
}

async function deleteApiKey(apiKey: ApiKey) {
  const user = selectedUser.value
  if (!user) return
  const confirmed = await confirmDanger(
    t('users.apiKey.deleteConfirm', { key: apiKey.key_display || '****' }),
    t('ui.deleteApiKey')
  )

  if (!confirmed) return

  try {
    await usersStore.deleteApiKey(user.id, apiKey.id)
    if (selectedUser.value?.id === user.id) await loadUserApiKeys(user.id)
    success(t('ui.apiKeyDeleted66e44ad'))
  } catch (err: unknown) {
    error(localizedApiError(err, t('ui.unknownError')), t('ui.failedToDeleteApiKey0130c20'))
  }
}

async function toggleLockApiKey(apiKey: ApiKey) {
  if (!selectedUser.value) return
  try {
    const response = await adminApi.toggleUserApiKeyLock(selectedUser.value.id, apiKey.id)
    // 更新本地状态
    const index = userApiKeys.value.findIndex(k => k.id === apiKey.id)
    if (index !== -1) {
      userApiKeys.value[index].is_locked = response.is_locked
    }
    success(response.message)
  } catch (err: unknown) {
    log.error('切换密钥锁定状态失败:', err)
    error(localizedApiError(err, t('ui.operationFailed')), t('ui.failedToLockUnlock'))
  }
}

async function copyFullKey(apiKey: ApiKey) {
  if (!selectedUser.value) return
  try {
    const response = await usersStore.getFullApiKey(selectedUser.value.id, apiKey.id)
    await copyToClipboard(response.key)
  } catch (err: unknown) {
    log.error('复制密钥失败:', err)
    error(localizedApiError(err, t('ui.unknownError')), t('ui.failedToCopyKey'))
  }
}

function openWalletActionDialog(user: User) {
  const wallet = getUserWallet(user.id)
  if (!wallet) {
    error(t('ui.thisUserWalletIsNotInitializedYetSoWalletOperations'))
    return
  }

  walletActionTarget.value = {
    user,
    wallet,
  }
  showWalletActionDialogState.value = true
}

function closeWalletActionDrawer() {
  showWalletActionDialogState.value = false
}

async function handleWalletDrawerChanged() {
  await loadUserWallets()
  if (!walletActionTarget.value) return
  const latestWallet = getUserWallet(walletActionTarget.value.user.id)
  if (latestWallet) {
    walletActionTarget.value.wallet = latestWallet
  }
}

async function deleteUser(user: User) {
  const confirmed = await confirmDanger(
    t('users.delete.confirmMessage', { username: user.username }),
    t('ui.deleteUser')
  )

  if (!confirmed) return

  try {
    await usersStore.deleteUser(user.id)
    invalidateUserOptions()
    if (usersStore.users.length === 0 && currentPage.value > 1) {
      currentPage.value -= 1
    }
    await refreshUsers()
    success(t('ui.userDeleted'))
  } catch (err: unknown) {
    error(localizedApiError(err, t('ui.unknownError')), t('ui.failedToDeleteUser'))
  }
}
</script>
