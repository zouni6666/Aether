<template>
  <section class="space-y-4">
    <div
      v-if="showSchedulingMode"
      class="space-y-1 text-sm"
    >
      <span class="text-muted-foreground">调度策略</span>
      <div class="grid grid-cols-3 gap-1 rounded-lg bg-muted/40 p-1">
        <button
          v-for="mode in schedulingModes"
          :key="mode.value"
          type="button"
          class="h-9 rounded-md px-3 text-sm font-medium transition-colors"
          :class="effectiveSchedulingMode === mode.value
            ? 'bg-background text-foreground shadow-sm'
            : 'text-muted-foreground hover:bg-background/60 hover:text-foreground'"
          @click="updateSchedulingMode(mode.value)"
        >
          {{ mode.label }}
        </button>
      </div>
    </div>

    <div class="rounded-lg border border-border/60">
      <div class="flex flex-col gap-3 border-b border-border/60 px-4 py-3 md:flex-row md:items-center md:justify-between">
        <div>
          <h3 class="text-sm font-medium">
            提供商排序
          </h3>
          <p class="mt-1 text-xs text-muted-foreground">
            {{ subtitle }}
          </p>
        </div>
        <div class="flex flex-wrap items-center gap-2">
          <button
            type="button"
            class="inline-flex h-8 items-center gap-2 rounded-md px-3 text-xs font-medium transition-colors"
            :class="providerMultiSelectEnabled
              ? 'bg-primary/10 text-primary hover:bg-primary/10 hover:text-primary'
              : 'text-muted-foreground hover:bg-muted hover:text-foreground'"
            @click="toggleProviderMultiSelect"
          >
            <ListChecks class="h-3.5 w-3.5" />
            {{ providerMultiSelectEnabled ? '退出多选' : '多选' }}
            <span v-if="providerMultiSelectEnabled && selectedProviderIds.size">{{ selectedProviderIds.size }}</span>
          </button>
        </div>
      </div>

      <div class="space-y-3 border-b border-border/60 bg-muted/15 px-4 py-3">
        <p class="text-xs leading-relaxed text-muted-foreground">
          {{ schedulingDescription }}
          优先级数字越小越先，可填写相同数字。移动或拖动会重新编号，未拆开的同级组保留。
        </p>
        <div class="flex flex-wrap items-center gap-3">
          <label class="relative min-w-0 flex-1 sm:max-w-sm">
            <Search class="pointer-events-none absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
            <input
              v-model="searchQuery"
              type="search"
              aria-label="搜索调度提供商"
              placeholder="搜索提供商名称"
              class="h-9 w-full rounded-md border border-border bg-background pl-9 pr-3 text-sm"
            >
          </label>
          <span class="text-xs text-muted-foreground">
            {{ visibleRowCount }} / {{ totalRowCount }} 项
          </span>
        </div>
        <p
          v-if="searchQuery.trim()"
          class="text-xs text-muted-foreground"
        >
          搜索仅用于定位；移动按完整列表执行，置顶和置底会越过隐藏项。
        </p>
      </div>

      <div class="min-h-[180px] p-3">
        <div
          v-if="loading"
          class="py-10 text-center text-sm text-muted-foreground"
        >
          正在加载
        </div>
        <div
          v-else-if="loadError"
          class="rounded-lg border border-destructive/30 bg-destructive/5 px-4 py-3 text-sm text-destructive"
        >
          {{ loadError }}
        </div>
        <div
          v-else
          class="space-y-2"
        >
          <div
            v-if="filteredProviderRows.length === 0"
            class="rounded-lg border border-dashed border-border/70 px-4 py-8 text-center text-sm text-muted-foreground"
          >
            {{ providerRows.length ? '没有匹配的提供商' : '暂无 Provider' }}
          </div>
          <div
            v-for="row in filteredProviderRows"
            v-else
            :key="row.id"
            class="group grid min-h-[56px] items-center gap-3 rounded-lg border px-3 py-2 transition-colors"
            :class="[providerGridClass, providerRowClass(row.id)]"
            draggable="true"
            @dragstart="handleProviderDragStart(row.id, $event)"
            @dragend="handleProviderDragEnd"
            @dragover.prevent="handleProviderDragOver(row.id)"
            @dragleave="handleProviderDragLeave"
            @drop="handleProviderDrop(row.id)"
          >
            <Checkbox
              v-if="providerMultiSelectEnabled"
              class="shrink-0"
              :checked="isProviderSelected(row.id)"
              :aria-label="`选择 ${row.name}`"
              @click.stop
              @change.stop
              @update:checked="checked => setProviderSelected(row.id, checked)"
            />
            <div class="cursor-grab rounded p-1 text-muted-foreground/40 transition-colors group-hover:text-muted-foreground active:cursor-grabbing">
              <GripVertical class="h-4 w-4" />
            </div>
            <div class="order-last col-span-full flex items-center justify-end gap-1 sm:order-none sm:col-span-1">
              <button
                v-for="action in providerMoveActions"
                :key="action.label"
                type="button"
                class="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground disabled:opacity-30"
                :aria-label="`${action.label} ${row.name}`"
                :title="action.label"
                :disabled="providerMoveDisabled(row.id, action.direction)"
                @click="action.edge ? moveProviderToEdge(row.id, action.edge) : moveProvider(row.id, action.direction)"
              >
                <component
                  :is="action.icon"
                  class="h-4 w-4"
                />
              </button>
            </div>
            <input
              :value="row.priority"
              :aria-label="`${row.name} 优先级`"
              type="number"
              min="0"
              class="priority-input h-8 w-14 rounded-md border border-border bg-background px-2 text-center text-sm"
              @change="event => setProviderPriority(row.id, event)"
            >
            <div class="min-w-0">
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  class="truncate text-left text-sm font-medium hover:text-primary hover:underline"
                  :aria-label="`查看提供商 ${row.name}`"
                  @click="emit('inspect-provider', row.id)"
                >
                  {{ row.name }}
                </button>
                <span
                  v-if="poolProviderIds.has(row.id)"
                  class="rounded bg-primary/10 px-1.5 py-0.5 text-[10px] text-primary"
                >Pool</span>
                <span
                  v-if="!row.is_active"
                  class="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground"
                >停用</span>
                <span
                  v-if="!isRoutingProviderEnabled(config, row.id, targetModelPolicy)"
                  class="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground"
                >本组禁用</span>
              </div>
              <div class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
                <span v-if="row.active_keys != null">可用 Key {{ row.active_keys }} / {{ row.total_keys }}</span>
                <span :class="row.health_score != null && row.health_score < 0.8 ? 'text-amber-600 dark:text-amber-400' : ''">
                  {{ row.health_score == null ? '健康度暂无数据' : `健康度 ${Math.round(row.health_score * 100)}%` }}
                </span>
                <slot
                  name="provider-status"
                  :provider="providerById.get(row.id)"
                />
              </div>
            </div>
            <div class="hidden max-w-[240px] flex-wrap justify-end gap-1 sm:flex">
              <span
                v-for="format in row.api_formats.slice(0, 3)"
                :key="format"
                :title="formatLabel(format)"
                class="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground"
              >{{ formatShortLabel(format) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { ArrowDown, ArrowUp, ChevronsDown, ChevronsUp, GripVertical, ListChecks, Search } from 'lucide-vue-next'

import {
  Checkbox,
} from '@/components/ui'
import {
  getProvidersSummary,
  type ProviderWithEndpointsSummary,
} from '@/api/endpoints'
import { formatApiFormat, formatApiFormatShort } from '@/api/endpoints/types/api-format'
import { parseApiError } from '@/utils/errorParser'
import {
  DEFAULT_ROUTING_POLICY_MODEL,
  getDefaultModelPolicy,
  getModelPolicy,
  isRoutingProviderEnabled,
  setModelProviderPriorityOverrides,
  type RoutingDefaultPolicy,
  type RoutingGroupConfig,
  type RoutingSchedulingMode,
} from '../utils/routingPolicy'
import { buildRoutingProviderSummaryQuery } from '../utils/providerQuery'
import { normalizeProviderSchedulingConfig } from '../utils/schedulingPolicies'

interface ProviderPriorityRow {
  id: string
  name: string
  is_active: boolean
  api_formats: string[]
  priority: number
  active_keys: number
  total_keys: number
  health_score: number | null
}

const props = defineProps<{
  config: RoutingGroupConfig
  model?: string
  modelId?: string
  providerModelIds?: string[]
  schedulingMode?: RoutingSchedulingMode
  showSchedulingMode?: boolean
  subtitle?: string
  refreshRevision?: number
}>()

const emit = defineEmits<{
  'update:config': [value: RoutingGroupConfig]
  'update:scheduling-mode': [value: RoutingSchedulingMode]
  'inspect-provider': [providerId: string]
}>()

const schedulingModes: Array<{ value: RoutingDefaultPolicy['scheduling_mode']; label: string }> = [
  { value: 'cache_affinity', label: '缓存亲和' },
  { value: 'load_balance', label: '负载均衡' },
  { value: 'fixed_order', label: '固定顺序' },
]
const providerMoveActions = [
  { label: '置顶', direction: -1, edge: 'start', icon: ChevronsUp },
  { label: '上移', direction: -1, edge: null, icon: ArrowUp },
  { label: '下移', direction: 1, edge: null, icon: ArrowDown },
  { label: '置底', direction: 1, edge: 'end', icon: ChevronsDown },
] as const

const providers = ref<ProviderWithEndpointsSummary[]>([])
const searchQuery = ref('')
const loadingProviders = ref(false)
const loadError = ref<string | null>(null)
const draggedProviderId = ref<string | null>(null)
const dragOverProviderId = ref<string | null>(null)
const providerMultiSelectEnabled = ref(false)
const selectedProviderIds = ref<Set<string>>(new Set())
let providerLoadRequestId = 0

const config = computed(() => normalizeProviderSchedulingConfig(props.config))
const targetModel = computed(() => props.model?.trim() || DEFAULT_ROUTING_POLICY_MODEL)
const targetModelPolicy = computed(() => targetModel.value === DEFAULT_ROUTING_POLICY_MODEL
  ? getDefaultModelPolicy(config.value)
  : getModelPolicy(config.value, targetModel.value))
const showSchedulingMode = computed(() => props.showSchedulingMode !== false)
const effectiveSchedulingMode = computed(() => props.schedulingMode ?? config.value.default_policy.scheduling_mode)
const subtitle = computed(() => props.subtitle ?? '默认作用于全部模型')
const schedulingDescription = computed(() => {
  if (effectiveSchedulingMode.value === 'fixed_order') return '固定顺序：优先级决定候选的尝试次序，实际可用候选还受模型能力和 API 格式影响。'
  if (effectiveSchedulingMode.value === 'load_balance') return '负载均衡：请求会分散到可用候选，列表优先级不代表实际尝试顺序。'
  return '缓存亲和：会优先复用缓存命中的候选，实际顺序可能与列表不同。'
})
const loading = computed(() => loadingProviders.value)
const providerGridClass = computed(() => providerMultiSelectEnabled.value
  ? 'grid-cols-[auto_auto_56px_minmax(0,1fr)] sm:grid-cols-[auto_auto_auto_56px_minmax(0,1fr)_auto]'
  : 'grid-cols-[auto_56px_minmax(0,1fr)] sm:grid-cols-[auto_auto_56px_minmax(0,1fr)_auto]')
const providerById = computed(() => {
  const map = new Map<string, ProviderWithEndpointsSummary>()
  for (const provider of providers.value) {
    map.set(provider.id, provider)
  }
  return map
})
const poolProviderIds = computed(() => {
  const set = new Set<string>()
  for (const provider of providers.value) {
    if (provider.pool_advanced) {
      set.add(provider.id)
    }
  }
  return set
})

const scopedProviders = computed(() => {
  // 多选模型取提供商并集；空数组表示模型尚未解析，不能回退到全部提供商。
  const modelIds = props.providerModelIds === undefined ? null : new Set(props.providerModelIds)
  return providers.value
    .filter(provider => !modelIds || provider.global_model_ids?.some(id => modelIds.has(id)))
})

const providerRows = computed<ProviderPriorityRow[]>(() => {
  const overrides = targetModelPolicy.value.provider_priority_overrides
  return scopedProviders.value
    .map(provider => ({
      id: provider.id,
      name: provider.name,
      is_active: provider.is_active,
      api_formats: provider.api_formats ?? [],
      priority: priorityValue(overrides[provider.id], provider.provider_priority),
      active_keys: provider.active_keys,
      total_keys: provider.total_keys,
      health_score: provider.avg_health_score ?? null,
    }))
    .sort(comparePriorityRows)
})

// Search only affects presentation. All move operations use the complete candidate list.
const filteredProviderRows = computed(() => providerRows.value.filter(row => matchesSearch(row.name)))
const totalRowCount = computed(() => providerRows.value.length)
const visibleRowCount = computed(() => filteredProviderRows.value.length)

function matchesSearch(value: string): boolean {
  return value.toLocaleLowerCase().includes(searchQuery.value.trim().toLocaleLowerCase())
}

// 父组件异步解析全局模型 ID 后，重新加载对应模型的提供商列表。
watch([targetModel, () => props.modelId], () => {
  void loadProviders()
})

watch(() => props.refreshRevision, () => { void loadProviders() })

watch(providerRows, rows => {
  const visibleIds = new Set(rows.map(row => row.id))
  const next = new Set([...selectedProviderIds.value].filter(id => visibleIds.has(id)))
  if (next.size !== selectedProviderIds.value.size) {
    selectedProviderIds.value = next
  }
})

onMounted(() => { void loadProviders() })

function updateConfig(value: RoutingGroupConfig): void {
  emit('update:config', normalizeProviderSchedulingConfig(value))
}

function updateDefaultPolicy(patch: Partial<RoutingDefaultPolicy>): void {
  updateConfig({
    ...config.value,
    default_policy: {
      ...config.value.default_policy,
      ...patch,
    },
  })
}

function updateSchedulingMode(mode: RoutingSchedulingMode): void {
  if (props.schedulingMode != null) {
    emit('update:scheduling-mode', mode)
    return
  }
  updateDefaultPolicy({ scheduling_mode: mode })
}

async function loadProviders(): Promise<void> {
  const requestId = ++providerLoadRequestId
  loadingProviders.value = true
  loadError.value = null
  try {
    const query = buildRoutingProviderSummaryQuery(
      targetModel.value,
      props.modelId,
      'provider',
    )
    if (!query) {
      providers.value = []
      return
    }

    const response = await getProvidersSummary(query)
    if (requestId !== providerLoadRequestId) return
    providers.value = response.items
  } catch (err) {
    if (requestId !== providerLoadRequestId) return
    loadError.value = parseApiError(err, '加载 Provider 失败')
    providers.value = []
  } finally {
    if (requestId === providerLoadRequestId) {
      loadingProviders.value = false
    }
  }
}

function setProviderPriority(providerId: string, event: Event): void {
  const priority = readPriorityInput(event)
  if (priority == null) return
  updateProviderOverrides({
    ...targetModelPolicy.value.provider_priority_overrides,
    [providerId]: priority,
  })
}

function moveProvider(providerId: string, direction: -1 | 1): void {
  const movingIds = providerMoveIds(providerId)
  const rows = movingIds.length > 1
    ? moveRowsByGroup(providerRows.value, movingIds, direction)
    : moveRow(providerRows.value, providerId, direction)
  updateReorderedProviders(rows, movingIds)
}

function moveProviderToEdge(providerId: string, edge: 'start' | 'end'): void {
  const movingIds = providerMoveIds(providerId)
  updateReorderedProviders(moveRowsToEdge(providerRows.value, movingIds, edge), movingIds)
}

function updateReorderedProviders(rows: ProviderPriorityRow[], movingIds: string[]): void {
  updateProviderOverrides({
    ...targetModelPolicy.value.provider_priority_overrides,
    ...reorderedPriorities(rows, movingIds),
  })
}

function updateProviderOverrides(overrides: Record<string, number>): void {
  updateConfig(setModelProviderPriorityOverrides(config.value, targetModel.value, overrides))
}

function isProviderSelected(providerId: string): boolean {
  return providerMultiSelectEnabled.value && selectedProviderIds.value.has(providerId)
}

function setProviderSelected(providerId: string, selected: boolean): void {
  if (!providerMultiSelectEnabled.value) return
  const next = new Set(selectedProviderIds.value)
  if (selected) {
    next.add(providerId)
  } else {
    next.delete(providerId)
  }
  selectedProviderIds.value = next
}

function toggleProviderMultiSelect(): void {
  providerMultiSelectEnabled.value = !providerMultiSelectEnabled.value
  if (!providerMultiSelectEnabled.value) {
    selectedProviderIds.value = new Set()
  }
}

function providerMoveIds(providerId: string): string[] {
  if (!providerMultiSelectEnabled.value || !selectedProviderIds.value.has(providerId)) {
    return [providerId]
  }
  return providerRows.value
    .map(row => row.id)
    .filter(id => selectedProviderIds.value.has(id))
}

function providerMoveDisabled(providerId: string, direction: -1 | 1): boolean {
  const movingIds = providerMoveIds(providerId)
  if (movingIds.length <= 1) {
    const index = providerRows.value.findIndex(row => row.id === providerId)
    return direction === -1 ? index === 0 : index === providerRows.value.length - 1
  }
  const movingSet = new Set(movingIds)
  const movingIndexes = providerRows.value
    .map((row, rowIndex) => movingSet.has(row.id) ? rowIndex : -1)
    .filter(rowIndex => rowIndex >= 0)
  if (movingIndexes.length === 0) return true
  return direction === -1
    ? Math.min(...movingIndexes) === 0
    : Math.max(...movingIndexes) === providerRows.value.length - 1
}

function providerRowClass(providerId: string): string {
  if (isProviderDragged(providerId)) {
    return 'border-primary/50 bg-primary/5 shadow-sm'
  }
  if (dragOverProviderId.value === providerId) {
    return 'border-primary/30 bg-primary/5'
  }
  if (isProviderSelected(providerId)) {
    return 'border-primary/40 bg-primary/5'
  }
  return 'border-border/50 bg-background hover:bg-muted/30'
}

function isProviderDragged(providerId: string): boolean {
  const draggedId = draggedProviderId.value
  return Boolean(draggedId && providerMoveIds(draggedId).includes(providerId))
}

function handleProviderDragStart(providerId: string, event: DragEvent): void {
  draggedProviderId.value = providerId
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', providerId)
  }
}

function handleProviderDragEnd(): void {
  draggedProviderId.value = null
  dragOverProviderId.value = null
}

function handleProviderDragOver(providerId: string): void {
  dragOverProviderId.value = providerId
}

function handleProviderDragLeave(): void {
  dragOverProviderId.value = null
}

function handleProviderDrop(providerId: string): void {
  const draggedId = draggedProviderId.value
  if (!draggedId || draggedId === providerId) {
    handleProviderDragEnd()
    return
  }
  const movingIds = providerMoveIds(draggedId)
  if (movingIds.includes(providerId)) {
    handleProviderDragEnd()
    return
  }
  const rows = movingIds.length > 1
    ? reorderRowsByGroup(providerRows.value, movingIds, providerId)
    : reorderRows(providerRows.value, draggedId, providerId)
  updateReorderedProviders(rows, movingIds)
  handleProviderDragEnd()
}

function moveRow<T extends { id: string }>(rows: T[], id: string, direction: -1 | 1): T[] {
  const next = [...rows]
  const index = next.findIndex(row => row.id === id)
  const targetIndex = index + direction
  if (index < 0 || targetIndex < 0 || targetIndex >= next.length) {
    return next
  }
  const [item] = next.splice(index, 1)
  next.splice(targetIndex, 0, item)
  return next
}

function moveRowsToEdge<T extends { id: string }>(rows: T[], movingIds: string[], edge: 'start' | 'end'): T[] {
  const movingSet = new Set(movingIds)
  const movingRows = rows.filter(row => movingSet.has(row.id))
  const remainingRows = rows.filter(row => !movingSet.has(row.id))
  return edge === 'start' ? [...movingRows, ...remainingRows] : [...remainingRows, ...movingRows]
}

function reorderedPriorities<T extends { id: string, priority: number }>(rows: T[], movingIds: string[]): Record<string, number> {
  const movingSet = new Set(movingIds)
  let priority = 0
  return Object.fromEntries(rows.map((row, index) => {
    const previous = rows[index - 1]
    // Keep untouched, contiguous priority groups together. The moved rows form their own position.
    if (previous && (previous.priority !== row.priority || movingSet.has(previous.id) !== movingSet.has(row.id))) {
      priority += 1
    }
    return [row.id, priority]
  }))
}

function moveRowsByGroup<T extends { id: string }>(rows: T[], movingIds: string[], direction: -1 | 1): T[] {
  const movingSet = new Set(movingIds)
  const movingRows = rows.filter(row => movingSet.has(row.id))
  if (movingRows.length === 0) return [...rows]

  const firstMovingIndex = rows.findIndex(row => movingSet.has(row.id))
  const remainingRows = rows.filter(row => !movingSet.has(row.id))
  const baseInsertIndex = rows
    .slice(0, firstMovingIndex)
    .filter(row => !movingSet.has(row.id))
    .length
  const insertIndex = direction === -1
    ? Math.max(0, baseInsertIndex - 1)
    : Math.min(remainingRows.length, baseInsertIndex + 1)

  const next = [...remainingRows]
  next.splice(insertIndex, 0, ...movingRows)
  return next
}

function reorderRows<T extends { id: string }>(rows: T[], draggedId: string, targetId: string): T[] {
  const next = [...rows]
  const fromIndex = next.findIndex(row => row.id === draggedId)
  const toIndex = next.findIndex(row => row.id === targetId)
  if (fromIndex < 0 || toIndex < 0) return next
  const [item] = next.splice(fromIndex, 1)
  next.splice(toIndex, 0, item)
  return next
}

function reorderRowsByGroup<T extends { id: string }>(rows: T[], movingIds: string[], targetId: string): T[] {
  const movingSet = new Set(movingIds)
  if (movingSet.has(targetId)) return [...rows]

  const movingRows = rows.filter(row => movingSet.has(row.id))
  if (movingRows.length === 0) return [...rows]

  const targetIndex = rows.findIndex(row => row.id === targetId)
  const firstMovingIndex = rows.findIndex(row => movingSet.has(row.id))
  if (targetIndex < 0 || firstMovingIndex < 0) return [...rows]

  const remainingRows = rows.filter(row => !movingSet.has(row.id))
  const remainingTargetIndex = remainingRows.findIndex(row => row.id === targetId)
  if (remainingTargetIndex < 0) return [...rows]

  const insertIndex = firstMovingIndex < targetIndex
    ? remainingTargetIndex + 1
    : remainingTargetIndex
  const next = [...remainingRows]
  next.splice(insertIndex, 0, ...movingRows)
  return next
}

function readPriorityInput(event: Event): number | null {
  const value = Number((event.target as HTMLInputElement).value)
  if (!Number.isFinite(value) || value < 0) {
    return null
  }
  return Math.trunc(value)
}

function priorityValue(override: number | undefined, fallback: number | null | undefined): number {
  if (typeof override === 'number' && Number.isFinite(override)) return override
  if (typeof fallback === 'number' && Number.isFinite(fallback)) return fallback
  return 0
}

function formatLabel(format: string): string {
  return formatApiFormat(format)
}

function formatShortLabel(format: string): string {
  return formatApiFormatShort(format)
}

function comparePriorityRows(left: ProviderPriorityRow, right: ProviderPriorityRow): number {
  return left.priority - right.priority
    || Number(right.is_active) - Number(left.is_active)
    || left.name.localeCompare(right.name)
    || left.id.localeCompare(right.id)
}
</script>

<style scoped>
.priority-input::-webkit-outer-spin-button,
.priority-input::-webkit-inner-spin-button {
  margin: 0;
  appearance: none;
}

.priority-input[type='number'] {
  appearance: textfield;
}
</style>
