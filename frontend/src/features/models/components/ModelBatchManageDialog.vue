<template>
  <Dialog
    :model-value="open"
    :title="t('models.management.batch.title')"
    :description="t('models.management.batch.description')"
    :icon="ListChecks"
    size="2xl"
    @update:model-value="(value: boolean) => $emit('update:open', value)"
  >
    <template #default>
      <div class="space-y-4">
        <!-- 搜索与在线价格来源 -->
        <div class="grid grid-cols-1 gap-2 sm:grid-cols-[minmax(0,1fr)_220px]">
          <div class="flex-1 relative">
            <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              v-model="batchManageSearchQuery"
              :placeholder="t('ui.searchModelscd01488')"
              class="pl-8 h-9"
            />
          </div>
          <Select
            v-model="batchPricingProviderId"
            :disabled="batchManageOnlineLoading"
          >
            <SelectTrigger class="h-9 text-xs">
              <SelectValue :placeholder="t('models.management.batch.providerPlaceholder')" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem :value="REMEMBERED_PRICING_PROVIDER_ID">
                {{ t('models.management.batch.rememberedProvider') }}
              </SelectItem>
              <SelectItem
                v-for="provider in batchPricingProviderOptions"
                :key="provider.providerId"
                :value="provider.providerId"
              >
                {{ provider.providerName }}
                <span class="ml-1 text-muted-foreground">({{ provider.matchCount }})</span>
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- 快捷选中 -->
        <div class="flex flex-wrap items-center gap-1.5">
          <span class="text-xs text-muted-foreground mr-0.5">{{ t('ui.quickSelect') }}</span>
          <button
            v-for="shortcut in batchManageShortcuts"
            :key="shortcut.label"
            type="button"
            class="rounded-md border border-border/60 px-2 py-1 text-xs transition-colors hover:bg-muted active:scale-[0.96]"
            :class="shortcut.emphasis ? 'border-primary/30 bg-primary/5 text-primary' : ''"
            :title="shortcut.description"
            @click="applyBatchManageShortcut(shortcut.filter)"
          >
            {{ shortcut.label }} ({{ shortcut.count }})
          </button>
        </div>

        <!-- 模型列表 -->
        <div class="border rounded-lg overflow-hidden">
          <div class="max-h-96 overflow-y-auto">
            <div
              v-if="batchManageLoading || batchManageOnlineLoading"
              class="flex items-center justify-center py-12"
            >
              <Loader2 class="w-6 h-6 animate-spin text-primary" />
            </div>

            <template v-else-if="filteredBatchManageModels.length > 0">
              <div
                class="flex items-center justify-between px-3 py-2 bg-muted sticky top-0 z-10"
              >
                <div class="flex items-center gap-2">
                  <span class="text-xs font-medium">{{ t('ui.model') }}</span>
                  <span class="text-xs text-muted-foreground">({{ filteredBatchManageModels.length }})</span>
                </div>
                <button
                  type="button"
                  class="text-xs text-primary hover:underline shrink-0"
                  @click="toggleAllBatchManageModels"
                >
                  {{ isAllBatchManageModelsSelected ? t('ui.deselectAll') : t('ui.selectAll') }}
                </button>
              </div>
              <div class="space-y-1 p-2">
                <div
                  v-for="model in filteredBatchManageModels"
                  :key="model.id"
                  class="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-muted cursor-pointer"
                  @click="toggleBatchManageModelSelection(model.id)"
                >
                  <div
                    class="w-4 h-4 border rounded flex items-center justify-center shrink-0"
                    :class="selectedBatchManageModelIds.has(model.id) ? 'bg-primary border-primary' : ''"
                  >
                    <Check
                      v-if="selectedBatchManageModelIds.has(model.id)"
                      class="w-3 h-3 text-primary-foreground"
                    />
                  </div>
                  <div class="flex-1 min-w-0">
                    <p class="text-sm font-medium truncate">
                      {{ model.display_name }}
                    </p>
                    <p class="text-xs text-muted-foreground truncate">
                      {{ model.name }}
                    </p>
                  </div>
                  <div class="flex items-center gap-2 shrink-0">
                    <span
                      class="max-w-28 truncate text-[10px] text-muted-foreground"
                      :title="getBatchPricingSourceLabel(model)"
                    >{{ getBatchPricingSourceLabel(model) }}</span>
                    <span
                      class="inline-flex items-center gap-1.5 text-[11px] font-medium"
                      :class="getBatchPricingStateClass(model)"
                      :title="getBatchPricingStateDescription(model)"
                    >
                      <span
                        class="h-1.5 w-1.5 rounded-full"
                        :class="getBatchPricingStateDotClass(model)"
                      />
                      {{ getBatchPricingStateLabel(model) }}
                    </span>
                    <Badge
                      variant="secondary"
                      class="text-xs"
                    >
                      {{ model.active_provider_count || 0 }}/{{ model.provider_count || 0 }}
                    </Badge>
                    <Badge
                      :variant="model.is_active ? 'outline' : 'secondary'"
                      :class="model.is_active ? 'text-green-600 border-green-500/60' : ''"
                      class="text-xs"
                    >
                      {{ model.is_active ? t('ui.activee85e590') : t('ui.disable4e6fd0e') }}
                    </Badge>
                  </div>
                </div>
              </div>
            </template>

            <!-- 空状态 -->
            <div
              v-else
              class="flex flex-col items-center justify-center py-12 text-muted-foreground"
            >
              <p class="text-sm">
                {{ batchManageSearchQuery ? t('ui.noMatchingResults') : t('ui.noModels') }}
              </p>
            </div>
          </div>
        </div>
      </div>
    </template>
    <template #footer>
      <div class="flex w-full min-w-0 flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <p class="min-w-0 break-keep text-pretty text-xs leading-5 text-muted-foreground lg:flex-1">
          {{ batchManageSelectionSummary }}
        </p>
        <div class="flex w-full min-w-0 flex-col gap-2 sm:flex-row lg:w-auto lg:shrink-0">
          <Button
            class="w-full whitespace-nowrap sm:flex-1 lg:w-auto lg:flex-none"
            :disabled="selectedBatchPriceSyncEntries.length === 0 || submittingBatchManage"
            @click="confirmBatchSyncPrices"
          >
            <Loader2
              v-if="batchManageAction === 'sync-prices'"
              class="w-4 h-4 mr-1 animate-spin"
            />
            <RefreshCw
              v-else
              class="w-4 h-4 mr-1"
            />
            {{ batchManageAction === 'sync-prices'
              ? t('models.management.batch.syncing')
              : t('models.management.batch.syncButton', { count: selectedBatchPriceSyncEntries.length }) }}
          </Button>
          <Button
            class="w-full whitespace-nowrap sm:flex-1 lg:w-auto lg:flex-none"
            variant="destructive"
            :disabled="selectedBatchManageModelIds.size === 0 || submittingBatchManage"
            @click="confirmBatchDeleteModels"
          >
            <Loader2
              v-if="batchManageAction === 'delete'"
              class="w-4 h-4 mr-1 animate-spin"
            />
            {{ batchManageAction === 'delete' ? t('ui.deleting') : t('ui.deleteSelected') }}
          </Button>
          <Button
            class="w-full whitespace-nowrap sm:flex-1 lg:w-auto lg:flex-none"
            variant="outline"
            @click="$emit('update:open', false)"
          >
            {{ t('ui.close') }}
          </Button>
        </div>
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { Check, ListChecks, Loader2, RefreshCw, Search } from 'lucide-vue-next'
import {
  Badge,
  Button,
  Dialog,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui'
import {
  batchDeleteGlobalModels,
  listGlobalModels,
  updateGlobalModel,
  type GlobalModelResponse,
} from '@/api/global-models'
import { getModelsDevList, type ModelsDevModelItem } from '@/api/models-dev'
import { useConfirm } from '@/composables/useConfirm'
import { useToast } from '@/composables/useToast'
import { useI18n } from '@/i18n'
import { runTasksWithConcurrency } from '@/utils/concurrency'
import { parseApiError } from '@/utils/errorParser'
import { log } from '@/utils/logger'
import {
  buildGlobalModelPriceSyncPlan,
  cloneTieredPricingConfig,
  type GlobalModelPriceSyncEntry,
} from './global-model-form-helpers'
import {
  getModelsDevPricingSourceFromConfig,
  modelsDevPricingSourcesEqual,
  useModelsDevPricingSources,
  withModelsDevPricingSource,
} from '../composables/useModelsDevPricingSources'
import { useModelsDevPricingSourceMigration } from '../composables/useModelsDevPricingSourceMigration'
import { getFirstTierPrice } from '../utils/globalModelPricing'

/**
 * 批量管理全局模型：按在线目录同步价格与来源，或批量删除。
 * 选中集合由父组件持有（列表勾选与对话框共用），这里通过 update:selectedIds 回写。
 */
const props = defineProps<{
  open: boolean
  selectedIds: Set<string>
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'update:selectedIds': [value: Set<string>]
  modelUpdated: [model: GlobalModelResponse]
  modelsDeleted: [ids: string[]]
  changed: []
}>()

const { success, error: showError } = useToast()
const { t, locale } = useI18n()
const { confirm, confirmDanger } = useConfirm()
const {
  getSource: getModelsDevPricingSource,
  setSource: setModelsDevPricingSource,
} = useModelsDevPricingSources()

const REMEMBERED_PRICING_PROVIDER_ID = '__remembered__'

const batchManageModels = ref<GlobalModelResponse[]>([])

const batchManageLoading = ref(false)
const { migrateLegacyModelsDevPricingSources } = useModelsDevPricingSourceMigration(applyBatchModelUpdate)

function applyBatchModelUpdate(updatedModel: GlobalModelResponse) {
  const current = batchManageModels.value.find(model => model.id === updatedModel.id)
  if (current) Object.assign(current, updatedModel)
  emit('modelUpdated', updatedModel)
}

const batchManageOnlineModels = ref<ModelsDevModelItem[]>([])

const batchManageOnlineLoading = ref(false)

const GLOBAL_MODELS_BATCH_FETCH_PAGE_SIZE = 1000

let batchManageModelsRequestId = 0

let batchManageOnlineModelsRequest: Promise<void> | null = null

const batchManageSearchQuery = ref('')
const selectedBatchManageModelIds = computed({
  get: () => props.selectedIds,
  set: (value: Set<string>) => emit('update:selectedIds', value),
})

const batchPricingProviderId = ref(REMEMBERED_PRICING_PROVIDER_ID)

const submittingBatchManage = ref(false)

const batchManageAction = ref<'sync-prices' | 'delete' | null>(null)

async function loadBatchManageModels() {
  const requestId = ++batchManageModelsRequestId
  batchManageLoading.value = true
  try {
    const allModels: GlobalModelResponse[] = []
    let skip = 0
    let expectedTotal: number | null = null

    while (true) {
      const response = await listGlobalModels({
        skip,
        limit: GLOBAL_MODELS_BATCH_FETCH_PAGE_SIZE,
      })
      if (expectedTotal === null && typeof response.total === 'number') {
        expectedTotal = response.total
      }
      const pageModels = response.models || []
      allModels.push(...pageModels)

      if (expectedTotal !== null && allModels.length >= expectedTotal) {
        break
      }
      if (pageModels.length < GLOBAL_MODELS_BATCH_FETCH_PAGE_SIZE) {
        break
      }

      skip += pageModels.length
    }

    if (requestId !== batchManageModelsRequestId) return
    batchManageModels.value = allModels
    const validModelIds = new Set(allModels.map(model => model.id))
    selectedBatchManageModelIds.value = new Set(
      [...selectedBatchManageModelIds.value].filter(modelId => validModelIds.has(modelId)),
    )
    migrateLegacyModelsDevPricingSources(allModels)
  } catch (err: unknown) {
    if (requestId !== batchManageModelsRequestId) return
    log.error('加载批量管理模型失败:', err)
    showError(parseApiError(err, t('ui.failedToLoadModels')), t('ui.failedToLoadModels'))
  } finally {
    if (requestId === batchManageModelsRequestId) {
      batchManageLoading.value = false
    }
  }
}

// 批量管理全局模型 - 过滤
const filteredBatchManageModels = computed(() => {
  const query = batchManageSearchQuery.value.toLowerCase().trim()
  if (!query) return batchManageModels.value
  return batchManageModels.value.filter(m => {
    const searchableText = `${m.name} ${m.display_name || ''}`.toLowerCase()
    return searchableText.includes(query)
  })
})

const batchPricingProviderOptions = computed(() => {
  const existingModelNames = new Set(batchManageModels.value.map(model => model.name.trim().toLowerCase()))
  const providers = new Map<string, {
    providerId: string
    providerName: string
    matchCount: number
    official: boolean
  }>()
  for (const onlineModel of batchManageOnlineModels.value) {
    const provider = providers.get(onlineModel.providerId) ?? {
      providerId: onlineModel.providerId,
      providerName: onlineModel.providerName,
      matchCount: 0,
      official: onlineModel.official === true,
    }
    if (
      (onlineModel.tieredPricing || onlineModel.pricingUnsupportedFields?.length)
      && existingModelNames.has(onlineModel.modelId.trim().toLowerCase())
    ) {
      provider.matchCount += 1
    }
    providers.set(onlineModel.providerId, provider)
  }
  return [...providers.values()]
    .filter(provider => provider.matchCount > 0)
    .sort((left, right) => (
      Number(right.official) - Number(left.official)
      || right.matchCount - left.matchCount
      || left.providerName.localeCompare(right.providerName)
    ))
})

const selectedBatchPricingProvider = computed(() => (
  batchPricingProviderOptions.value.find(provider => provider.providerId === batchPricingProviderId.value)
))

const batchPricingProviderModels = computed(() => (
  batchPricingProviderId.value === REMEMBERED_PRICING_PROVIDER_ID
    ? batchManageOnlineModels.value
    : batchManageOnlineModels.value.filter(model => model.providerId === batchPricingProviderId.value)
))

const rememberedBatchPricingProviderIds = computed(() => {
  if (batchPricingProviderId.value !== REMEMBERED_PRICING_PROVIDER_ID) return undefined
  const providerIds = new Map<string, string>()
  for (const model of batchManageModels.value) {
    const source = getModelsDevPricingSource(model.id, model.config)
    if (source) providerIds.set(model.id, source.provider_id)
  }
  return providerIds
})

const batchPriceSyncPlan = computed(() => (
  buildGlobalModelPriceSyncPlan(
    batchManageModels.value,
    batchPricingProviderModels.value,
    rememberedBatchPricingProviderIds.value,
  )
))

function doesBatchEntryNeedSourcePersistence(entry: GlobalModelPriceSyncEntry): boolean {
  return !modelsDevPricingSourcesEqual(
    getModelsDevPricingSourceFromConfig(entry.model.config),
    {
      provider_id: entry.onlineModel.providerId,
      provider_name: entry.onlineModel.providerName,
    },
  )
}

const batchPricingStateByModelId = computed(() => {
  const states = new Map<string, 'syncable' | 'source-pending' | 'unchanged' | 'unsupported' | 'unavailable'>()
  for (const entry of batchPriceSyncPlan.value.syncable) states.set(entry.model.id, 'syncable')
  for (const entry of batchPriceSyncPlan.value.unchanged) {
    states.set(
      entry.model.id,
      doesBatchEntryNeedSourcePersistence(entry) ? 'source-pending' : 'unchanged',
    )
  }
  for (const entry of batchPriceSyncPlan.value.unsupported) states.set(entry.model.id, 'unsupported')
  for (const model of batchPriceSyncPlan.value.unavailable) states.set(model.id, 'unavailable')
  return states
})

const selectedBatchManageModels = computed(() => (
  batchManageModels.value.filter(model => selectedBatchManageModelIds.value.has(model.id))
))

const selectedBatchPriceSyncPlan = computed(() => (
  buildGlobalModelPriceSyncPlan(
    selectedBatchManageModels.value,
    batchPricingProviderModels.value,
    rememberedBatchPricingProviderIds.value,
  )
))

const selectedBatchPriceSyncEntries = computed(() => [
  ...selectedBatchPriceSyncPlan.value.syncable,
  ...selectedBatchPriceSyncPlan.value.unchanged.filter(doesBatchEntryNeedSourcePersistence),
])

const batchManageSelectionSummary = computed(() => {
  const selectedCount = selectedBatchManageModelIds.value.size
  if (selectedCount === 0) return t('models.management.batch.selectionEmpty')
  const plan = selectedBatchPriceSyncPlan.value
  const sourcePendingCount = plan.unchanged.filter(doesBatchEntryNeedSourcePersistence).length
  const unchangedCount = plan.unchanged.length - sourcePendingCount
  return t('models.management.batch.selectionSummary', {
    selected: selectedCount,
    syncable: plan.syncable.length,
    sourcePending: sourcePendingCount,
    unchanged: unchangedCount,
    unsupported: plan.unsupported.length,
    unavailable: plan.unavailable.length,
  })
})

function getBatchPricingState(model: GlobalModelResponse) {
  return batchPricingStateByModelId.value.get(model.id) ?? 'unavailable'
}

function getBatchPricingStateLabel(model: GlobalModelResponse): string {
  const state = getBatchPricingState(model)
  if (state === 'syncable') return t('models.management.batch.state.syncable')
  if (state === 'source-pending') return t('models.management.batch.state.sourcePending')
  if (state === 'unchanged') return t('models.management.batch.state.unchanged')
  if (state === 'unsupported') return t('models.management.batch.state.unsupported')
  return t('models.management.batch.state.unavailable')
}

function getBatchPricingStateDescription(model: GlobalModelResponse): string {
  const providerName = getBatchPricingSourceLabel(model)
  const unsupportedEntry = batchPriceSyncPlan.value.unsupported.find(entry => entry.model.id === model.id)
  if (unsupportedEntry) {
    return `${providerName || unsupportedEntry.onlineModel.providerName} · 不支持独立结算 ${formatBatchUnsupportedPricingFields(unsupportedEntry.onlineModel)}`
  }
  return `${providerName} · ${getBatchPricingStateLabel(model)}`
}

function getBatchPricingSourceLabel(model: GlobalModelResponse): string {
  if (batchPricingProviderId.value !== REMEMBERED_PRICING_PROVIDER_ID) {
    return selectedBatchPricingProvider.value?.providerName
      ?? t('models.management.batch.source.noneSelected')
  }
  return getModelsDevPricingSource(model.id, model.config)?.provider_name
    ?? t('models.management.batch.source.noneRecorded')
}

function getBatchPricingStateClass(model: GlobalModelResponse): string {
  const state = getBatchPricingState(model)
  if (state === 'syncable') return 'text-amber-700 dark:text-amber-300'
  if (state === 'source-pending') return 'text-sky-700 dark:text-sky-300'
  if (state === 'unchanged') return 'text-emerald-700 dark:text-emerald-300'
  if (state === 'unsupported') return 'text-rose-700 dark:text-rose-300'
  return 'text-muted-foreground'
}

function getBatchPricingStateDotClass(model: GlobalModelResponse): string {
  const state = getBatchPricingState(model)
  if (state === 'syncable') return 'bg-amber-500'
  if (state === 'source-pending') return 'bg-sky-500'
  if (state === 'unchanged') return 'bg-emerald-500'
  if (state === 'unsupported') return 'bg-rose-500'
  return 'bg-muted-foreground/45'
}

function formatBatchUnsupportedPricingFields(model: ModelsDevModelItem): string {
  const labels = {
    reasoning: t('models.pricingField.reasoning'),
    input_audio: t('models.pricingField.inputAudio'),
    output_audio: t('models.pricingField.outputAudio'),
  }
  return (model.pricingUnsupportedFields ?? [])
    .map(field => labels[field])
    .join(locale.value === 'zh-CN' ? '、' : ', ')
}

// 批量管理 - 快捷筛选定义
function hasNoPrice(m: GlobalModelResponse): boolean {
  return !getFirstTierPrice(m, 'input') && !getFirstTierPrice(m, 'output')
    && !m.default_price_per_request
}

const batchManageShortcuts = computed(() => {
  const models = batchManageModels.value
  const defs: {
    label: string
    description: string
    filter: (m: GlobalModelResponse) => boolean
    emphasis?: boolean
  }[] = [
    { label: t('models.management.batch.state.syncable'), description: t('models.management.batch.shortcut.syncable'), filter: m => getBatchPricingState(m) === 'syncable', emphasis: true },
    { label: t('models.management.batch.state.sourcePending'), description: t('models.management.batch.shortcut.sourcePending'), filter: m => getBatchPricingState(m) === 'source-pending', emphasis: true },
    { label: t('models.management.batch.state.unchanged'), description: t('models.management.batch.shortcut.unchanged'), filter: m => getBatchPricingState(m) === 'unchanged' },
    { label: t('models.management.batch.state.unsupported'), description: t('models.management.batch.shortcut.unsupported'), filter: m => getBatchPricingState(m) === 'unsupported' },
    { label: t('models.management.batch.state.unavailable'), description: t('models.management.batch.shortcut.unavailable'), filter: m => getBatchPricingState(m) === 'unavailable' },
    { get label() { return t('ui.noProviders') }, get description() { return t('ui.modelsWithoutAnyLinkedProviders') }, filter: m => (m.provider_count || 0) === 0 },
    { get label() { return t('ui.noActiveProviders') }, get description() { return t('ui.modelsWithProvidersButNoneActive') }, filter: m => (m.active_provider_count || 0) === 0 && (m.provider_count || 0) > 0 },
    { get label() { return t('ui.disable') }, get description() { return t('ui.disabledModels') }, filter: m => !m.is_active },
    { get label() { return t('ui.unused') }, get description() { return t('ui.modelsWithNoRecordedRequests') }, filter: m => (m.usage_count || 0) === 0 },
    { get label() { return t('ui.noPricing') }, get description() { return t('ui.modelsWithoutConfiguredPricing') }, filter: m => hasNoPrice(m) },
  ]
  return defs.map(d => ({ ...d, count: models.filter(d.filter).length }))
})

// 批量管理 - 应用快捷选中
function applyBatchManageShortcut(filter: (m: GlobalModelResponse) => boolean) {
  const matchedIds = batchManageModels.value.filter(filter).map(m => m.id)
  selectedBatchManageModelIds.value = new Set(matchedIds)
}

// 批量管理 - 是否全选
const isAllBatchManageModelsSelected = computed(() => {
  if (filteredBatchManageModels.value.length === 0) return false
  return filteredBatchManageModels.value.every(m => selectedBatchManageModelIds.value.has(m.id))
})

// 批量管理 - 切换单个选择
function toggleBatchManageModelSelection(modelId: string) {
  const next = new Set(selectedBatchManageModelIds.value)
  if (next.has(modelId)) {
    next.delete(modelId)
  } else {
    next.add(modelId)
  }
  selectedBatchManageModelIds.value = next
}

// 批量管理 - 全选/取消全选
function toggleAllBatchManageModels() {
  const allIds = filteredBatchManageModels.value.map(m => m.id)
  const next = new Set(selectedBatchManageModelIds.value)
  if (isAllBatchManageModelsSelected.value) {
    for (const id of allIds) next.delete(id)
  } else {
    for (const id of allIds) next.add(id)
  }
  selectedBatchManageModelIds.value = next
}

async function loadBatchManageOnlineModels() {
  if (batchManageOnlineModels.value.length > 0) return
  if (batchManageOnlineModelsRequest) return batchManageOnlineModelsRequest

  batchManageOnlineModelsRequest = (async () => {
    batchManageOnlineLoading.value = true
    try {
      batchManageOnlineModels.value = await getModelsDevList(false)
    } catch (err: unknown) {
      log.error('加载在线模型价格失败:', err)
      showError(parseApiError(err, t('ui.failedToLoadOnlineModelPricing')), t('ui.failedToLoad'))
    } finally {
      batchManageOnlineLoading.value = false
      batchManageOnlineModelsRequest = null
    }
  })()
  return batchManageOnlineModelsRequest
}

async function confirmBatchSyncPrices() {
  const plan = selectedBatchPriceSyncPlan.value
  const entries = selectedBatchPriceSyncEntries.value
  if (entries.length === 0) return
  const providerName = batchPricingProviderId.value === REMEMBERED_PRICING_PROVIDER_ID
    ? t('models.management.batch.rememberedProviderLabel')
    : selectedBatchPricingProvider.value?.providerName
      || t('models.management.batch.selectedProviderLabel')
  const sourceOnlyCount = plan.unchanged.filter(doesBatchEntryNeedSourcePersistence).length
  const skippedCount = plan.unchanged.length - sourceOnlyCount
    + plan.unsupported.length
    + plan.unavailable.length
  const sourceOnlyMessage = sourceOnlyCount > 0
    ? ` ${t('models.management.batch.confirmSourceOnly', { count: sourceOnlyCount })}`
    : ''
  const skippedMessage = skippedCount > 0
    ? ` ${t('models.management.batch.confirmSkipped', { count: skippedCount })}`
    : ''
  const confirmed = await confirm({
    title: t('models.management.batch.confirmTitle'),
    message: `${t('models.management.batch.confirmMain', {
      provider: providerName,
      count: entries.length,
    })}${sourceOnlyMessage}${skippedMessage}\n\n${t('models.management.batch.confirmUnchanged')}`,
    confirmText: t('models.management.batch.confirmButton'),
    variant: 'info',
  })
  if (!confirmed) return

  submittingBatchManage.value = true
  batchManageAction.value = 'sync-prices'
  const failedIds = new Set<string>()
  const failureMessages: string[] = []
  let successCount = 0
  try {
    const tasks = entries.map(entry => async () => {
      try {
        const onlinePricing = entry.onlineModel.tieredPricing
        if (!onlinePricing) {
          throw new Error(t('models.management.batch.catalogMissingPrice'))
        }
        const pricing = cloneTieredPricingConfig(onlinePricing)
        const source = {
          provider_id: entry.onlineModel.providerId,
          provider_name: entry.onlineModel.providerName,
        }
        const nextConfig = withModelsDevPricingSource(entry.model.config, source)
        const updatedModel = await updateGlobalModel(entry.model.id, {
          default_tiered_pricing: pricing,
          config: nextConfig,
        })
        setModelsDevPricingSource(entry.model.id, source)
        applyBatchModelUpdate({
          ...updatedModel,
          default_tiered_pricing: pricing,
          config: nextConfig,
        })
        successCount += 1
      } catch (err: unknown) {
        failedIds.add(entry.model.id)
        failureMessages.push(`${entry.model.display_name}: ${parseApiError(err, t('update.error.updateFailed'))}`)
      }
    })
    await runTasksWithConcurrency(tasks)

    if (successCount > 0) {
      success(t('models.management.batch.success', { count: successCount }))
    }
    if (failureMessages.length > 0) {
      showError(
        t('models.management.batch.partialFailure', {
          count: failureMessages.length,
          details: failureMessages.slice(0, 2).join(locale.value === 'zh-CN' ? '；' : '; '),
        }),
        t('ui.someOperationsFailed'),
      )
    }
    emit('changed')
    await loadBatchManageModels()
    selectedBatchManageModelIds.value = failedIds
  } finally {
    batchManageAction.value = null
    submittingBatchManage.value = false
  }
}

// 确认批量删除模型
async function confirmBatchDeleteModels() {
  const count = selectedBatchManageModelIds.value.size
  if (count === 0) return

  const confirmed = await confirmDanger(
    t('ui.deleteTheSelectedValueModelsThisCannotBeUndone', { count }),
    t('ui.deleteModelsInBulk')
  )
  if (!confirmed) return

  submittingBatchManage.value = true
  batchManageAction.value = 'delete'
  try {
    const ids = Array.from(selectedBatchManageModelIds.value)
    const result = await batchDeleteGlobalModels(ids)

    if (result.success_count > 0) {
      success(t('ui.deletedValueModels', { count: result.success_count }))
    }
    if (result.failed.length > 0) {
      showError(t('ui.failedToDeleteValueModels', { count: result.failed.length }), t('ui.someOperationsFailed'))
    }

    emit('modelsDeleted', ids)
    selectedBatchManageModelIds.value = new Set()
    emit('changed')
    await loadBatchManageModels()
  } catch (err: unknown) {
    showError(parseApiError(err, t('ui.bulkDeletionFailed')), t('ui.error'))
  } finally {
    batchManageAction.value = null
    submittingBatchManage.value = false
  }
}

watch([batchPricingProviderOptions, () => props.open], ([options, dialogOpen]) => {
  if (!dialogOpen) return
  if (
    batchPricingProviderId.value !== REMEMBERED_PRICING_PROVIDER_ID
    && !options.some(provider => provider.providerId === batchPricingProviderId.value)
  ) {
    batchPricingProviderId.value = REMEMBERED_PRICING_PROVIDER_ID
  }
}, { immediate: true })

watch(() => props.open, (open) => {
  if (!open) return
  batchManageSearchQuery.value = ''
  batchPricingProviderId.value = REMEMBERED_PRICING_PROVIDER_ID
  void Promise.all([loadBatchManageModels(), loadBatchManageOnlineModels()])
})

onBeforeUnmount(() => {
  batchManageModelsRequestId += 1
})
</script>
