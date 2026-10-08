<template>
  <ProviderSchedulingView
    ref="schedulingWorkspace"
    :provider-revision="providerRevision"
    @inspect-provider="openProviderDrawer"
    @context-change="updateSchedulingContext"
  >
    <div
      ref="providerListRef"
      class="flex h-full min-w-0 flex-col gap-4"
      :class="{ 'select-none [&_*]:!cursor-grabbing': draggingProvider }"
      @click.capture="handleSortClick"
    >
      <ProviderDeleteProgressCard
        :progress="providerDeleteProgress"
        :stage-label="providerDeleteStageLabel"
        :total-units="providerDeleteTotalUnits"
        :completed-units="providerDeleteCompletedUnits"
        :overall-percent="providerDeleteOverallPercent"
        :keys-percent="providerDeleteKeysPercent"
        :endpoints-percent="providerDeleteEndpointsPercent"
      />

      <!-- 提供商表格 -->
      <Card
        variant="default"
        class="flex-1"
      >
        <!-- 标题和操作栏 -->
        <ProviderTableHeader
          :search-query="searchQuery"
          :filter-api-format="filterApiFormat"
          filter-model="all"
          :show-model-filter="false"
          :api-format-filters="apiFormatFilters"
          :model-filters="[]"
          :has-active-filters="hasActiveFilters"
          :loading="loading"
          @update:search-query="searchQuery = $event"
          @update:filter-api-format="filterApiFormat = $event"
          @reset-filters="resetFilters"
          @batch-process="openProviderBatchDialog"
          @add-provider="openAddProviderDialog"
          @refresh="loadProviders"
        />

        <!-- 加载状态 -->
        <div
          v-if="loading"
          class="flex items-center justify-center py-12"
        >
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
        </div>

        <!-- 空状态 -->
        <div
          v-else-if="displayedProviders.length === 0"
          class="contents"
        >
          <ProviderEmptyState
            :has-active-filters="hasActiveFilters"
            @reset-filters="resetFilters"
          />
        </div>

        <!-- 桌面端表格 -->
        <div
          v-else
          class="hidden xl:block overflow-x-auto"
        >
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead class="w-px px-2 text-center">
                  <span class="sr-only">{{ legacyT('优先级') }}</span>
                </TableHead>
                <TableHead class="w-[18%] min-w-[200px]">
                  {{ legacyT('提供商') }}
                </TableHead>
                <TableHead class="w-[20%] min-w-[180px]">
                  {{ legacyT('余额监控') }}
                </TableHead>
                <TableHead class="w-[12%] min-w-[100px] text-center">
                  {{ legacyT('资源统计') }}
                </TableHead>
                <SortableTableHead
                  class="w-[22%] min-w-[240px]"
                  column-key="api_format"
                  :sortable="false"
                  :filter-active="filterApiFormat !== 'all'"
                  :filter-title="legacyT('筛选 API 格式')"
                  filter-content-class="w-72 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
                >
                  {{ legacyT('端点健康') }}
                  <template #filter="{ close }">
                    <TableFilterMenu
                      v-model="filterApiFormat"
                      :options="apiFormatFilters"
                      @select="close"
                    />
                  </template>
                </SortableTableHead>
                <TableHead class="w-[10%] min-w-[96px] text-center">
                  {{ legacyT('状态') }}
                </TableHead>
                <TableHead class="w-[18%] min-w-[192px] text-center">
                  {{ legacyT('操作') }}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <ProviderTableRow
                v-for="provider in displayedProviders"
                :key="provider.id"
                :provider="provider"
                :data-provider-sort-id="provider.id"
                :class="sortItemClass(provider.id)"
                :editing-description-id="editingDescriptionId"
                :is-balance-loading="isBalanceLoading"
                :get-provider-balance="getProviderBalance"
                :get-provider-balance-breakdown="getProviderBalanceBreakdown"
                :get-provider-balance-error="getProviderBalanceError"
                :get-provider-checkin="getProviderCheckin"
                :get-provider-cookie-expired="getProviderCookieExpired"
                :get-provider-balance-extra="getProviderBalanceExtra"
                :format-balance-display="formatBalanceDisplay"
                :format-reset-countdown="formatResetCountdown"
                :get-quota-used-color-class="getQuotaUsedColorClass"
                @mousedown="handleMouseDown"
                @row-click="handleRowClick"
                @view-detail="openProviderDrawer"
                @edit-provider="openEditProviderDialog"
                @open-ops-config="openOpsConfigDialog"
                @toggle-status="toggleProviderStatus"
                @delete-provider="handleDeleteProvider"
                @start-edit-description="startEditDescription"
                @save-description="saveDescription"
                @cancel-edit-description="cancelEditDescription"
              >
                <template #priority>
                  <div class="flex items-center gap-0.5">
                    <ProviderDragHandle
                      :provider-name="provider.name"
                      :disabled="loading || priorityEditingDisabled || displayedProviders.length < 2"
                      @pointerdown="startDrag(provider.id, $event)"
                      @keydown="handleSortKeydown(provider.id, $event)"
                    />
                    <ProviderPriorityInput
                      :provider-name="provider.name"
                      :priority="getGroupPriority(provider)"
                      :edit-context="priorityEditContext"
                      :disabled="priorityEditingDisabled"
                      @update:priority="setGroupPriority(provider.id, $event)"
                    />
                  </div>
                </template>
                <template #scheduling>
                  <ProviderGroupControls
                    :provider-name="provider.name"
                    :priority="getGroupPriority(provider)"
                    :edit-context="priorityEditContext"
                    :enabled="isGroupEnabled(provider.id)"
                    :disabled="priorityEditingDisabled"
                    :priority-disabled="priorityEditingDisabled"
                    :show-priority="false"
                    @update:priority="setGroupPriority(provider.id, $event)"
                  />
                </template>
                <template #group-action>
                  <ProviderGroupToggleButton
                    :provider-name="provider.name"
                    :enabled="isGroupEnabled(provider.id)"
                    :disabled="priorityEditingDisabled"
                    @update:enabled="setGroupEnabled(provider.id, $event)"
                  />
                </template>
              </ProviderTableRow>
            </TableBody>
          </Table>
        </div>

        <!-- 移动端卡片列表 -->
        <div
          v-if="!loading && displayedProviders.length > 0"
          class="xl:hidden divide-y divide-border/40"
        >
          <ProviderMobileCard
            v-for="provider in displayedProviders"
            :key="provider.id"
            :provider="provider"
            :data-provider-sort-id="provider.id"
            :class="sortItemClass(provider.id)"
            :editing-description-id="editingDescriptionId"
            :is-balance-loading="isBalanceLoading"
            :get-provider-balance="getProviderBalance"
            :get-provider-balance-error="getProviderBalanceError"
            :get-provider-checkin="getProviderCheckin"
            :get-provider-cookie-expired="getProviderCookieExpired"
            :format-balance-display="formatBalanceDisplay"
            :get-quota-used-color-class="getQuotaUsedColorClass"
            @view-detail="openProviderDrawer"
            @edit-provider="openEditProviderDialog"
            @open-ops-config="openOpsConfigDialog"
            @toggle-status="toggleProviderStatus"
            @delete-provider="handleDeleteProvider"
            @start-edit-description="startEditDescription"
            @save-description="saveDescription"
            @cancel-edit-description="cancelEditDescription"
          >
            <template #scheduling>
              <ProviderGroupControls
                :provider-name="provider.name"
                :priority="getGroupPriority(provider)"
                :edit-context="priorityEditContext"
                :enabled="isGroupEnabled(provider.id)"
                :disabled="priorityEditingDisabled"
                :priority-disabled="priorityEditingDisabled"
                @update:priority="setGroupPriority(provider.id, $event)"
              />
            </template>
            <template #group-action>
              <ProviderGroupToggleButton
                :provider-name="provider.name"
                :enabled="isGroupEnabled(provider.id)"
                :disabled="priorityEditingDisabled"
                @update:enabled="setGroupEnabled(provider.id, $event)"
              />
            </template>
            <template #drag-handle>
              <ProviderDragHandle
                class="-ml-2 w-4"
                :provider-name="provider.name"
                :disabled="loading || priorityEditingDisabled || displayedProviders.length < 2"
                @pointerdown="startDrag(provider.id, $event)"
                @keydown="handleSortKeydown(provider.id, $event)"
              />
            </template>
          </ProviderMobileCard>
        </div>

        <!-- 分页 -->
        <Pagination
          v-if="!loading && total > 0"
          :current="currentPage"
          :total="total"
          :page-size="pageSize"
          cache-key="provider-management-page-size"
          @update:current="currentPage = $event"
          @update:page-size="pageSize = $event"
        />
      </Card>
      <span
        class="sr-only"
        role="status"
        aria-live="polite"
        aria-atomic="true"
      >{{ announcement }}</span>
    </div>
  </ProviderSchedulingView>

  <Teleport to="body">
    <div
      v-if="draggingProvider"
      class="pointer-events-none fixed z-[100] w-[200px] truncate rounded-xl border border-primary/40 bg-card px-3 py-2 text-sm font-medium text-foreground shadow-lg"
      :style="dragPreviewStyle"
      aria-hidden="true"
    >
      {{ draggingProvider.name }}
    </div>
  </Teleport>

  <!-- 对话框 -->
  <ProviderFormDialog
    v-model="providerDialogOpen"
    :provider="providerToEdit"
    :max-priority="maxProviderPriority"
    :routing-group-id="providerCreationGroup?.id"
    :routing-group-name="providerCreationGroup?.name"
    @provider-created="handleProviderAdded"
    @provider-updated="handleProviderUpdated"
  />

  <ProviderBatchActionDialog
    v-model="providerBatchDialogOpen"
    :providers="displayedProviders"
    @changed="handleProviderBatchChanged"
  />

  <ProviderDetailDrawer
    v-if="providerDrawerMounted"
    :open="providerDrawerOpen"
    :provider-id="selectedProviderId"
    :initial-provider="selectedProvider"
    @update:open="providerDrawerOpen = $event"
    @edit="openEditProviderDialog"
    @toggle-status="toggleProviderStatus"
    @refresh="handleDrawerRefresh"
  />

  <ProviderAuthDialog
    v-model:open="opsConfigDialogOpen"
    :provider-id="opsConfigProviderId"
    :provider-website="opsConfigProviderWebsite"
    @saved="handleOpsConfigSaved"
  />
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, defineAsyncComponent } from 'vue'
import Card from '@/components/ui/card.vue'
import Table from '@/components/ui/table.vue'
import TableHeader from '@/components/ui/table-header.vue'
import TableBody from '@/components/ui/table-body.vue'
import TableRow from '@/components/ui/table-row.vue'
import TableHead from '@/components/ui/table-head.vue'
import SortableTableHead from '@/components/ui/sortable-table-head.vue'
import TableFilterMenu from '@/components/ui/table-filter-menu.vue'
import Pagination from '@/components/ui/pagination.vue'
import { ProviderFormDialog, ProviderAuthDialog } from '@/features/providers/components'
import ProviderBatchActionDialog from '@/features/providers/components/ProviderBatchActionDialog.vue'
import ProviderTableHeader from '@/features/providers/components/ProviderTableHeader.vue'
import ProviderTableRow from '@/features/providers/components/ProviderTableRow.vue'
import ProviderMobileCard from '@/features/providers/components/ProviderMobileCard.vue'
import ProviderGroupControls from '@/features/providers/components/ProviderGroupControls.vue'
import ProviderGroupToggleButton from '@/features/providers/components/ProviderGroupToggleButton.vue'
import ProviderPriorityInput from '@/features/providers/components/ProviderPriorityInput.vue'
import { providerGroupPriority, sortGroupProviders, moveGroupProvider } from '@/features/providers/utils/groupPriority'
import { getDefaultModelPolicy, isRoutingProviderEnabled, normalizeRoutingGroupConfig, type RoutingModelPolicy, type RoutingPriorityMode, type RoutingSchedulingMode, type RoutingGroupConfig } from '@/features/routing/utils/routingPolicy'
import ProviderDragHandle from '@/features/providers/components/ProviderDragHandle.vue'
import ProviderDeleteProgressCard from '@/features/providers/components/ProviderDeleteProgressCard.vue'
import ProviderEmptyState from '@/features/providers/components/ProviderEmptyState.vue'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { useRowClick } from '@/composables/useRowClick'
import { useProviderFilters } from '@/features/providers/composables/useProviderFilters'
import { useProviderBalance } from '@/features/providers/composables/useProviderBalance'
import { useProviderPriorityOrder } from '@/features/providers/composables/useProviderPriorityOrder'
import {
  getProvidersSummary,
  getProvider,
  deleteProvider,
  getProviderDeleteTask,
  updateProvider,
  type ProviderWithEndpointsSummary,
} from '@/api/endpoints'
import { parseApiError } from '@/utils/errorParser'
import { useI18n } from '@/i18n'

const ProviderDetailDrawer = defineAsyncComponent(
  () => import('@/features/providers/components/ProviderDetailDrawer.vue'),
)
const ProviderSchedulingView = defineAsyncComponent(
  () => import('@/features/providers/components/ProviderSchedulingView.vue'),
)

interface SchedulingSelection {
  policy: RoutingModelPolicy | null
  priorityMode: RoutingPriorityMode
  schedulingMode: RoutingSchedulingMode
  scope: 'all' | 'selected' | null
  modelNames: string[]
}
interface SchedulingContext {
  groupId: string | null
  groupName: string
  config: RoutingGroupConfig | null
  busy: boolean
  activePolicy: SchedulingSelection | null
}
const schedulingContext = ref<SchedulingContext>({ groupId: null, groupName: '', config: null, busy: true, activePolicy: null })
const schedulingWorkspace = ref<{
  updateDraftConfig: (config: RoutingGroupConfig) => void
  updatePriorityPolicy: (policy: RoutingModelPolicy) => void
  refreshGroups: () => Promise<void>
  ensureSaved: () => Promise<boolean>
} | null>(null)
const schedulingBusy = computed(() => schedulingContext.value.busy || !schedulingContext.value.config)
const providerCreationGroup = ref<{ id: string; name: string } | null>(null)
const resourcesLoaded = ref(false)
const providerRevision = ref(0)
function updateSchedulingContext(context: SchedulingContext) {
  if (context.groupId !== schedulingContext.value.groupId) {
    currentPage.value = 1
    cancelDrag()
  }
  schedulingContext.value = context
}

interface ProviderDeleteProgressState {
  providerId: string
  providerName: string
  taskId: string
  status: string
  stage: string
  totalKeys: number
  deletedKeys: number
  totalEndpoints: number
  deletedEndpoints: number
  message: string
}

const { error: showError, success: showSuccess, info: showInfo } = useToast()
const { confirmDanger } = useConfirm()
const { legacyT } = useI18n()

function showLegacyError(err: unknown, fallback: string, title = '错误') {
  showError(legacyT(parseApiError(err, fallback)), legacyT(title))
}

// 状态
const loading = ref(false)
const providers = ref<ProviderWithEndpointsSummary[]>([])
let providersRequestId = 0
const providerDialogOpen = ref(false)
const providerBatchDialogOpen = ref(false)
const providerToEdit = ref<ProviderWithEndpointsSummary | null>(null)
const providerDrawerOpen = ref(false)
const providerDrawerMounted = ref(false)
const selectedProviderId = ref<string | null>(null)
const selectedProviderSnapshot = ref<ProviderWithEndpointsSummary | null>(null)
const selectedProvider = computed<ProviderWithEndpointsSummary | null>(() => {
  if (!selectedProviderId.value) return null
  return providers.value.find(provider => provider.id === selectedProviderId.value)
    ?? (selectedProviderSnapshot.value?.id === selectedProviderId.value ? selectedProviderSnapshot.value : null)
})
const providerDeleteProgress = ref<ProviderDeleteProgressState | null>(null)
let deletePollAbort: AbortController | null = null

const DELETE_POLL_INTERVAL_MS = 2000
const DELETE_POLL_MAX_MS = 30 * 60 * 1000
const DELETE_POLL_MAX_FAILURES = 3
const PROVIDER_SUMMARY_CACHE_TTL_MS = 10 * 1000

async function pollProviderDeleteTask(providerId: string, taskId: string) {
  deletePollAbort?.abort()
  const abort = new AbortController()
  deletePollAbort = abort

  const deadline = Date.now() + DELETE_POLL_MAX_MS
  let consecutiveFailures = 0

  while (Date.now() < deadline) {
    if (abort.signal.aborted) return null
    try {
      const task = await getProviderDeleteTask(providerId, taskId)
      consecutiveFailures = 0
      if (providerDeleteProgress.value?.taskId === taskId) {
        providerDeleteProgress.value = {
          ...providerDeleteProgress.value,
          status: task.status,
          stage: task.stage,
          totalKeys: task.total_keys,
          deletedKeys: task.deleted_keys,
          totalEndpoints: task.total_endpoints,
          deletedEndpoints: task.deleted_endpoints,
          message: task.message,
        }
      }
      if (task.status === 'completed' || task.status === 'failed') {
        return task
      }
    } catch {
      consecutiveFailures += 1
      if (consecutiveFailures >= DELETE_POLL_MAX_FAILURES) {
        throw new Error('provider delete task polling failed')
      }
    }
    await new Promise((resolve) => {
      const timer = setTimeout(resolve, DELETE_POLL_INTERVAL_MS)
      abort.signal.addEventListener('abort', () => { clearTimeout(timer); resolve(undefined) }, { once: true })
    })
  }

  throw new Error('provider delete task timeout')
}

const providerDeleteStageLabel = computed(() => {
  switch (providerDeleteProgress.value?.stage) {
    case 'preparing':
      return legacyT('准备删除')
    case 'disabling':
      return legacyT('停用提供商')
    case 'cleaning_restrictions':
      return legacyT('清理访问限制')
    case 'cleaning_provider_refs':
      return legacyT('清理历史引用')
    case 'deleting_keys':
      return legacyT('删除号池账号')
    case 'deleting_endpoints':
      return legacyT('删除端点')
    case 'completed':
      return legacyT('删除完成')
    case 'failed':
      return legacyT('删除失败')
    default:
      return legacyT('等待执行')
  }
})

const providerDeleteTotalUnits = computed(() => {
  const progress = providerDeleteProgress.value
  if (!progress) return 0
  return progress.totalKeys + progress.totalEndpoints
})

const providerDeleteCompletedUnits = computed(() => {
  const progress = providerDeleteProgress.value
  if (!progress) return 0
  return Math.min(progress.deletedKeys + progress.deletedEndpoints, providerDeleteTotalUnits.value)
})

const providerDeleteOverallPercent = computed(() => {
  const progress = providerDeleteProgress.value
  if (!progress) return 0
  if (progress.status === 'completed') return 100
  if (providerDeleteTotalUnits.value <= 0) return 0
  return Math.min(
    100,
    Math.round((providerDeleteCompletedUnits.value / providerDeleteTotalUnits.value) * 100),
  )
})

const providerDeleteKeysPercent = computed(() => {
  const progress = providerDeleteProgress.value
  if (!progress?.totalKeys) return 0
  return Math.min(100, Math.round((progress.deletedKeys / progress.totalKeys) * 100))
})

const providerDeleteEndpointsPercent = computed(() => {
  const progress = providerDeleteProgress.value
  if (!progress?.totalEndpoints) return 0
  return Math.min(100, Math.round((progress.deletedEndpoints / progress.totalEndpoints) * 100))
})

// Composables
const {
  searchQuery,
  filterApiFormat,
  apiFormatFilters,
  hasActiveFilters,
  currentPage,
  pageSize,
  total,
  queryParams,
  resetFilters,
} = useProviderFilters(
  () => [],
)

const {
  loadArchitectureSchemas,
  loadBalances,
  getProviderBalance,
  getProviderBalanceBreakdown,
  getProviderBalanceError,
  isBalanceLoading,
  getProviderCheckin,
  getProviderCookieExpired,
  formatBalanceDisplay,
  formatResetCountdown,
  getProviderBalanceExtra,
  getQuotaUsedColorClass,
  startTick,
  stopTick,
} = useProviderBalance()

// 扩展操作配置对话框
const opsConfigDialogOpen = ref(false)
const opsConfigProviderId = ref('')
const opsConfigProviderWebsite = ref('')

// 内联编辑备注
const editingDescriptionId = ref<string | null>(null)

const selectedPolicy = computed(() => schedulingContext.value.activePolicy)
const priorityEditContext = computed(() => JSON.stringify([
  schedulingContext.value.groupId,
  selectedPolicy.value?.scope,
  selectedPolicy.value?.modelNames,
]))
const priorityEditingDisabled = computed(() => schedulingBusy.value || !selectedPolicy.value?.policy
  || (selectedPolicy.value.scope === 'selected' && selectedPolicy.value.modelNames.length === 0))
// The active configuration owns this shared order. Directory filters only narrow its display.
const priorityConfig = computed(() => {
  const config = schedulingContext.value.config
  const selection = selectedPolicy.value
  if (!config || !selection?.policy) return config
  const defaults = getDefaultModelPolicy(config).provider_priority_overrides
  return normalizeRoutingGroupConfig({
    ...config,
    model_policies: [{ ...selection.policy, model: '*', provider_priority_overrides: { ...defaults, ...selection.policy.provider_priority_overrides } }],
  })
})
function getGroupPriority(provider: ProviderWithEndpointsSummary) {
  return providerGroupPriority(priorityConfig.value, provider)
}
function isGroupEnabled(providerId: string) {
  const config = schedulingContext.value.config
  return config ? isRoutingProviderEnabled(config, providerId, selectedPolicy.value?.policy) : true
}
function setGroupEnabled(providerId: string, enabled: boolean) {
  const policy = selectedPolicy.value?.policy
  if (!policy || priorityEditingDisabled.value) return
  schedulingWorkspace.value?.updatePriorityPolicy({
    ...policy,
    provider_enabled_overrides: { ...policy.provider_enabled_overrides, [providerId]: enabled },
  })
}
function setGroupPriority(providerId: string, priority: number) {
  const policy = selectedPolicy.value?.policy
  if (!policy || priorityEditingDisabled.value) return
  schedulingWorkspace.value?.updatePriorityPolicy({
    ...policy,
    provider_priority_overrides: { ...policy.provider_priority_overrides, [providerId]: priority },
  })
}
const filteredProviders = computed(() => {
  const search = searchQuery.value.trim().toLocaleLowerCase()
  return providers.value.filter(provider => (
    (!search || `${provider.name} ${provider.description ?? ''}`.toLocaleLowerCase().includes(search))
    && (filterApiFormat.value === 'all' || provider.api_formats?.includes(filterApiFormat.value))
  ))
})
const providerListRef = ref<HTMLElement | null>(null)
const {
  orderedProviders,
  draggingProvider,
  dragPreviewStyle,
  announcement,
  startDrag,
  cancelDrag,
  handleSortKeydown,
  handleSortClick,
  sortItemClass,
} = useProviderPriorityOrder(
  () => sortGroupProviders(priorityConfig.value, filteredProviders.value),
  providerListRef,
  {
    disabled: () => priorityEditingDisabled.value || loading.value,
    move(providerId, targetId) {
      const config = priorityConfig.value
      const policy = selectedPolicy.value?.policy
      if (!config || !policy) return
      const updated = getDefaultModelPolicy(moveGroupProvider(config, providers.value, providerId, targetId))
      schedulingWorkspace.value?.updatePriorityPolicy({ ...policy, provider_priority_overrides: updated.provider_priority_overrides })
    },
  },
)
const displayedProviders = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return orderedProviders.value.slice(start, start + pageSize.value)
})
watch(() => filteredProviders.value.length, length => {
  total.value = length
  currentPage.value = Math.min(currentPage.value, Math.max(1, Math.ceil(length / pageSize.value)))
}, { immediate: true })
watch(() => [schedulingContext.value.busy, selectedPolicy.value?.scope, selectedPolicy.value?.modelNames.join('|')], () => {
  cancelDrag()
  currentPage.value = 1
})

watch([loading, queryParams], cancelDrag)

function startEditDescription(_event: Event, provider: ProviderWithEndpointsSummary) {
  editingDescriptionId.value = provider.id
}

function cancelEditDescription(_event?: Event) {
  editingDescriptionId.value = null
}

async function saveDescription(_event: Event, provider: ProviderWithEndpointsSummary, newValue: string) {
  const trimmed = newValue.trim()
  const oldValue = provider.description || ''
  if (trimmed === oldValue) {
    cancelEditDescription()
    return
  }
  try {
    await updateProvider(provider.id, { description: trimmed || null })
    provider.description = trimmed || undefined
    // 同步更新 providers 数组
    const target = providers.value.find(p => p.id === provider.id)
    if (target) {
      target.description = trimmed || undefined
    }
    cancelEditDescription()
  } catch (err: unknown) {
    showLegacyError(err, '更新备注失败')
  }
}

// 当前已有提供商的最大优先级
const maxProviderPriority = computed(() => {
  if (providers.value.length === 0) return undefined
  const priorities = providers.value
    .map(p => p.provider_priority)
    .filter(v => typeof v === 'number' && Number.isFinite(v))
  return priorities.length > 0 ? Math.max(...priorities) : undefined
})

// 先取得完整目录，再按当前分组的优先级排序、筛选和分页。
async function loadProviders(options: { cacheTtlMs?: number } = {}) {
  const requestId = ++providersRequestId
  loading.value = true
  try {
    const response = await getProvidersSummary({ page: 1, page_size: 10_000 }, {
      cacheTtlMs: options.cacheTtlMs ?? 0,
    })
    const items = new Map(response.items.map(item => [item.id, item]))
    let nextPage = 2
    while (items.size < response.total) {
      if (requestId !== providersRequestId) return
      const page = await getProvidersSummary({ page: nextPage++, page_size: 10_000 }, {
        cacheTtlMs: options.cacheTtlMs ?? 0,
      })
      const previousSize = items.size
      page.items.forEach(item => items.set(item.id, item))
      if (items.size === previousSize) break
    }
    if (requestId !== providersRequestId) return
    const existingProviders = new Map(providers.value.map(provider => [provider.id, provider]))
    providers.value = [...items.values()].map((item) => {
      const existing = existingProviders.get(item.id)
      if (!existing) return item
      Object.assign(existing, item)
      return existing
    })
    // 异步加载配置了 ops 的 provider 的余额数据
    loadBalances(providers.value)
  } catch (err: unknown) {
    if (requestId !== providersRequestId) return
    showLegacyError(err, '加载提供商列表失败')
  } finally {
    if (requestId === providersRequestId) {
      loading.value = false
    }
  }
}

// 使用复用的行点击逻辑
const { handleMouseDown, shouldTriggerRowClick } = useRowClick()

// 处理行点击 - 只在非选中文本时打开抽屉
function handleRowClick(event: MouseEvent, providerId: string) {
  if (!shouldTriggerRowClick(event)) return
  openProviderDrawer(providerId)
}

// 打开添加提供商对话框
function openAddProviderDialog() {
  const { groupId, groupName, config } = schedulingContext.value
  if (!groupId || schedulingBusy.value) {
    showInfo(legacyT(!groupId && config ? '请先保存新分组，再添加提供商' : '请先创建或选择策略分组'))
    return
  }
  providerCreationGroup.value = { id: groupId, name: groupName }
  providerToEdit.value = null
  providerDialogOpen.value = true
}

function openProviderBatchDialog() {
  providerBatchDialogOpen.value = true
}

async function handleProviderBatchChanged() {
  providerRevision.value += 1
  await loadProviders()
}

// 打开提供商详情抽屉
function openProviderDrawer(providerId: string) {
  selectedProviderId.value = providerId
  providerDrawerMounted.value = true
  providerDrawerOpen.value = true
}

function mergeUpdatedProvider(updated: ProviderWithEndpointsSummary) {
  if (selectedProviderId.value === updated.id) selectedProviderSnapshot.value = updated
  const index = providers.value.findIndex(p => p.id === updated.id)
  if (index !== -1) {
    Object.assign(providers.value[index], updated)
    loadBalances([providers.value[index]], false)
  }
}

async function refreshProviderSnapshot(
  providerId: string,
  fallbackErrorMessage = '刷新提供商数据失败',
): Promise<ProviderWithEndpointsSummary | null> {
  try {
    const updated = await getProvider(providerId)
    mergeUpdatedProvider(updated)
    return updated
  } catch (err) {
    showLegacyError(err, fallbackErrorMessage)
    return null
  }
}

// 打开编辑提供商对话框
async function openEditProviderDialog(provider: ProviderWithEndpointsSummary) {
  const latest = await refreshProviderSnapshot(provider.id, '刷新提供商状态失败')
  providerToEdit.value = latest ?? provider
  providerDialogOpen.value = true
}

// 打开扩展操作配置对话框
function openOpsConfigDialog(provider: ProviderWithEndpointsSummary) {
  opsConfigProviderId.value = provider.id
  opsConfigProviderWebsite.value = provider.website || ''
  opsConfigDialogOpen.value = true
}

// 扩展操作配置保存回调
function handleOpsConfigSaved() {
  opsConfigDialogOpen.value = false
  void loadProviders()
}

// 处理提供商编辑完成
function handleProviderUpdated(updated: ProviderWithEndpointsSummary) {
  mergeUpdatedProvider(updated)
  providerRevision.value += 1
}

// 处理详情抽屉内的刷新：只刷新当前查看的那一条提供商
async function handleDrawerRefresh() {
  if (!selectedProviderId.value) return
  await refreshProviderSnapshot(selectedProviderId.value)
  providerRevision.value += 1
}

// 处理提供商添加
function handleProviderAdded() {
  providerRevision.value += 1
  void loadProviders()
  void schedulingWorkspace.value?.refreshGroups()
}

// 删除提供商
async function handleDeleteProvider(provider: ProviderWithEndpointsSummary) {
  const confirmed = await confirmDanger(
    legacyT('删除提供商'),
    legacyT(`确定要删除提供商 "${provider.name}" 吗？\n\n这将同时删除其所有端点、密钥和配置。此操作不可恢复！`),
  )

  if (!confirmed) return

  try {
    const result = await deleteProvider(provider.id)
    providerDeleteProgress.value = {
      providerId: provider.id,
      providerName: provider.name,
      taskId: result.task_id,
      status: result.status,
      stage: 'queued',
      totalKeys: provider.total_keys || 0,
      deletedKeys: 0,
      totalEndpoints: provider.total_endpoints || 0,
      deletedEndpoints: 0,
      message: result.message || '删除任务已提交，后台处理中',
    }
    showInfo(legacyT(result.message || '删除任务已提交，后台处理中'))

    const task = await pollProviderDeleteTask(provider.id, result.task_id)
    if (!task) return // aborted
    if (task.status === 'failed') {
      throw new Error(task.message || 'provider delete task failed')
    }

    showSuccess(legacyT('提供商已删除'))
    providerDeleteProgress.value = null
    void loadProviders()
  } catch (err: unknown) {
    providerDeleteProgress.value = null
    showLegacyError(err, '删除提供商失败')
  }
}

// 切换提供商状态
async function toggleProviderStatus(provider: ProviderWithEndpointsSummary) {
  try {
    const newStatus = !provider.is_active
    await updateProvider(provider.id, { is_active: newStatus })

    // 更新抽屉内部的 provider 对象
    provider.is_active = newStatus

    // 同时更新主页面 providers 数组中的对象，实现无感更新
    const targetProvider = providers.value.find(p => p.id === provider.id)
    if (targetProvider) {
      targetProvider.is_active = newStatus
    }
    providerRevision.value += 1

    showSuccess(legacyT(newStatus ? '提供商已启用' : '提供商已停用'))
  } catch (err: unknown) {
    showLegacyError(err, '操作失败')
  }
}

// 点击外部自动取消编辑备注
function handleGlobalClick(event: MouseEvent) {
  if (!editingDescriptionId.value) return
  const target = event.target as HTMLElement
  if (target.closest('[data-desc-editor]')) return
  cancelEditDescription()
}

function ensureResourcesLoaded() {
  if (resourcesLoaded.value) return
  resourcesLoaded.value = true
  void loadProviders({ cacheTtlMs: PROVIDER_SUMMARY_CACHE_TTL_MS })
  void loadArchitectureSchemas()
  startTick()
}

onMounted(() => {
  ensureResourcesLoaded()
  document.addEventListener('click', handleGlobalClick, true)
})

onUnmounted(() => {
  deletePollAbort?.abort()
  document.removeEventListener('click', handleGlobalClick, true)
  stopTick()
})
</script>
