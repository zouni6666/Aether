<template>
  <Dialog
    :model-value="open"
    :title="editingGroup ? t('ui.editModelMapping') : t('ui.addModelMapping')"
    :description="editingGroup ? t('ui.editMappingSettings') : t('ui.mapProviderModelsToClientModels')"
    :icon="Tag"
    size="lg"
    @update:model-value="$emit('update:open', $event)"
  >
    <div class="space-y-4">
      <!-- 目标模型选择 -->
      <div class="space-y-1.5">
        <Label class="text-xs">{{ t('ui.clientModel') }}</Label>
        <Select
          :model-value="formData.modelId"
          :disabled="!!editingGroup"
          @update:model-value="handleModelChange"
        >
          <SelectTrigger class="h-9">
            <SelectValue :placeholder="t('ui.selectAClientModel')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem
              v-for="model in models"
              :key="model.id"
              :value="model.id"
            >
              <div class="flex items-center gap-2">
                <span class="font-medium">{{ model.global_model_display_name || model.provider_model_name }}</span>
                <span class="text-xs text-muted-foreground font-mono">{{ model.provider_model_name }}</span>
              </div>
            </SelectItem>
          </SelectContent>
        </Select>
        <p class="text-xs text-muted-foreground">
          {{ t('ui.clientRequestsForThisModelAreRoutedToTheSelected') }}
        </p>
      </div>

      <!-- 映射名称选择面板 -->
      <div class="space-y-1.5">
        <Label class="text-xs">{{ t('ui.providerModel') }}</Label>
        <!-- 搜索栏 -->
        <div class="flex items-center gap-2">
          <div class="flex-1 relative">
            <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              v-model="searchQuery"
              :placeholder="t('ui.searchOrAddACustomProviderModel')"
              class="pl-8 h-9"
            />
          </div>
          <!-- 已选数量徽章 -->
          <span
            v-if="selectedNames.length === 0"
            class="h-7 px-2.5 text-xs rounded-md flex items-center bg-muted text-muted-foreground shrink-0"
          > {{ t('ui.notSelected') }} </span>
          <span
            v-else
            class="h-7 px-2.5 text-xs rounded-md flex items-center bg-primary/10 text-primary shrink-0"
          > {{ t('ui.selected') }} {{ selectedNames.length }} {{ t('ui.items3a11681') }} </span>
          <!-- 刷新上游模型按钮 -->
          <button
            v-if="upstreamModelsLoaded"
            type="button"
            class="p-2 hover:bg-muted rounded-md transition-colors shrink-0"
            :disabled="fetchingUpstreamModels"
            :title="t('ui.refreshUpstreamModels')"
            @click="fetchUpstreamModels(true)"
          >
            <RefreshCw
              class="w-4 h-4"
              :class="{ 'animate-spin': fetchingUpstreamModels }"
            />
          </button>
          <button
            v-else-if="!fetchingUpstreamModels"
            type="button"
            class="p-2 hover:bg-muted rounded-md transition-colors shrink-0"
            :title="t('ui.fetchModelsFromProvider')"
            @click="fetchUpstreamModels(true)"
          >
            <Zap class="w-4 h-4" />
          </button>
          <Loader2
            v-else
            class="w-4 h-4 animate-spin text-muted-foreground shrink-0"
          />
        </div>

        <!-- 模型列表 -->
        <div class="border rounded-lg overflow-hidden">
          <div class="min-h-60 max-h-80 overflow-y-auto">
            <!-- 加载中 -->
            <div
              v-if="loadingModels"
              class="flex items-center justify-center py-12"
            >
              <Loader2 class="w-6 h-6 animate-spin text-primary" />
            </div>

            <template v-else>
              <!-- 添加自定义映射名称（搜索内容不在列表中时显示） -->
              <div
                v-if="searchQuery && canAddAsCustom"
                class="px-3 py-2 border-b bg-background sticky top-0 z-30"
              >
                <div
                  class="flex items-center justify-between px-3 py-2 rounded-lg border border-dashed hover:border-primary hover:bg-primary/5 cursor-pointer transition-colors"
                  @click="addCustomName"
                >
                  <div class="flex items-center gap-2">
                    <Plus class="w-4 h-4 text-muted-foreground" />
                    <span class="text-sm font-mono">{{ searchQuery }}</span>
                  </div>
                  <span class="text-xs text-muted-foreground">{{ t('ui.addCustomProviderModel') }}</span>
                </div>
              </div>

              <!-- 自定义映射名称 -->
              <div v-if="customNames.length > 0">
                <div
                  class="flex items-center justify-between px-3 py-2 bg-muted sticky top-0 z-20 cursor-pointer hover:bg-muted/80 transition-colors"
                  @click="toggleGroupCollapse('custom')"
                >
                  <div class="flex items-center gap-2">
                    <ChevronDown
                      class="w-4 h-4 transition-transform shrink-0"
                      :class="collapsedGroups.has('custom') ? '-rotate-90' : ''"
                    />
                    <span class="text-xs font-medium">{{ t('ui.customModel') }}</span>
                    <span class="text-xs text-muted-foreground">({{ customNames.length }})</span>
                  </div>
                </div>
                <div
                  v-show="!collapsedGroups.has('custom')"
                  class="space-y-1 p-2"
                >
                  <div
                    v-for="name in sortedCustomNames"
                    :key="name"
                    class="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-muted cursor-pointer"
                    @click="toggleName(name)"
                  >
                    <div
                      class="w-4 h-4 border rounded flex items-center justify-center shrink-0"
                      :class="selectedNames.includes(name) ? 'bg-primary border-primary' : ''"
                    >
                      <Check
                        v-if="selectedNames.includes(name)"
                        class="w-3 h-3 text-primary-foreground"
                      />
                    </div>
                    <span class="text-sm font-mono truncate flex-1">{{ name }}</span>
                  </div>
                </div>
              </div>

              <!-- 上游模型 -->
              <template v-if="filteredUpstreamModels.length > 0">
                <div
                  class="flex items-center justify-between px-3 py-2 bg-muted sticky top-0 z-20 cursor-pointer hover:bg-muted/80 transition-colors"
                  @click="toggleGroupCollapse('upstream')"
                >
                  <div class="flex items-center gap-2">
                    <ChevronDown
                      class="w-4 h-4 transition-transform shrink-0"
                      :class="collapsedGroups.has('upstream') ? '-rotate-90' : ''"
                    />
                    <span class="text-xs font-medium">{{ t('ui.upstreamModels') }}</span>
                    <span class="text-xs text-muted-foreground">({{ upstreamModelNames.length }})</span>
                  </div>
                  <button
                    type="button"
                    class="text-xs text-primary hover:underline"
                    @click.stop="toggleAllUpstreamModels"
                  >
                    {{ isAllUpstreamModelsSelected ? t('ui.deselectAll') : t('ui.selectAll') }}
                  </button>
                </div>
                <div
                  v-show="!collapsedGroups.has('upstream')"
                  class="space-y-1 p-2"
                >
                  <div
                    v-for="name in filteredUpstreamModels"
                    :key="name"
                    class="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-muted cursor-pointer"
                    @click="toggleName(name)"
                  >
                    <div
                      class="w-4 h-4 border rounded flex items-center justify-center shrink-0"
                      :class="selectedNames.includes(name) ? 'bg-primary border-primary' : ''"
                    >
                      <Check
                        v-if="selectedNames.includes(name)"
                        class="w-3 h-3 text-primary-foreground"
                      />
                    </div>
                    <span class="text-sm font-mono truncate flex-1">{{ name }}</span>
                  </div>
                </div>
              </template>

              <!-- 空状态 -->
              <div
                v-if="showEmptyState"
                class="flex flex-col items-center justify-center py-12 text-muted-foreground"
              >
                <Tag class="w-10 h-10 mb-2 opacity-30" />
                <p class="text-sm">
                  {{ searchQuery ? t('ui.noMatchingResults') : t('ui.noModelsAvailableToSelect') }}
                </p>
                <p class="text-xs mt-1">
                  {{ t('ui.enterAModelNameToAddACustomProviderModel') }}
                </p>
              </div>
            </template>
          </div>
        </div>
      </div>

      <div class="space-y-3 border-t border-border/60 pt-4">
        <div class="flex flex-col gap-1.5 sm:flex-row sm:items-start sm:justify-between sm:gap-4">
          <div class="min-w-0 space-y-0.5">
            <h3 class="text-sm font-medium text-foreground">
              {{ t('ui.appliesTo') }}
            </h3>
            <p class="text-xs text-muted-foreground">
              {{ t('providers.modelMapping.scope.matchHelp') }}
            </p>
          </div>
          <span class="max-w-full break-words text-left text-xs text-muted-foreground sm:max-w-[55%] sm:text-right">
            {{ mappingScopeSummary }}
          </span>
        </div>

        <div class="grid gap-4 sm:grid-cols-2">
          <div class="min-w-0 space-y-1.5">
            <Label class="text-xs">{{ t('ui.applicableEndpoints') }}</Label>
            <MultiSelect
              v-model="selectedEndpointIds"
              :options="endpointOptions"
              :placeholder="t('providers.modelMapping.scope.allEndpoints')"
              :empty-text="t('ui.noEndpoints')"
              :no-results-text="t('ui.noEndpointsFound')"
              trigger-class="h-9 rounded-md"
              :search-threshold="4"
            />
            <p class="text-xs leading-5 text-muted-foreground">
              {{ t('providers.modelMapping.scope.endpointHelp') }}
            </p>
          </div>

          <div class="min-w-0 space-y-1.5">
            <Label class="text-xs">{{ t('ui.applicableRequests') }}</Label>
            <div
              class="flex min-h-9 w-full flex-wrap gap-1 rounded-md bg-muted/50 p-1"
              role="radiogroup"
              :aria-label="t('ui.applicableRequests')"
            >
              <Button
                v-for="option in requestScopeOptions"
                :key="option.value"
                type="button"
                size="sm"
                :variant="requestScopeValue === option.value ? 'secondary' : 'ghost'"
                class="h-7 min-w-0 flex-1 basis-[9rem] px-2.5"
                role="radio"
                :aria-checked="requestScopeValue === option.value"
                :title="option.label"
                @click="handleRequestScopeChange(option.value)"
              >
                <span class="truncate">{{ option.label }}</span>
              </Button>
            </div>
            <p class="text-xs leading-5 text-muted-foreground">
              {{ requestScopeDescription }}
            </p>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <Button
        variant="outline"
        @click="$emit('update:open', false)"
      >
        {{ t('common.cancel') }}
      </Button>
      <Button
        :disabled="submitting || !formData.modelId || selectedNames.length === 0"
        @click="handleSubmit"
      >
        <Loader2
          v-if="submitting"
          class="w-4 h-4 mr-2 animate-spin"
        />
        {{ editingGroup ? t('ui.saveMapping') : t('ui.addMapping') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Tag, Loader2, Plus, Search, Check, ChevronDown, RefreshCw, Zap } from 'lucide-vue-next'
import {
  Button,
  Input,
  Label,
  Dialog,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui'
import MultiSelect from '@/components/common/MultiSelect.vue'
import { useToast } from '@/composables/useToast'
import { useI18n } from '@/i18n'
import { parseApiError } from '@/utils/errorParser'
import {
  type Model,
  type ProviderEndpoint,
  type ProviderModelAlias,
  type UpstreamModel,
} from '@/api/endpoints'
import { updateModel } from '@/api/endpoints/models'
import { useUpstreamModelsCache } from '../composables/useUpstreamModelsCache'
import {
  ALL_REQUESTS_SCOPE_VALUE,
  COMPACT_REQUEST_SCOPE_VALUE,
  formatModelMappingEndpointLabel,
  formatModelMappingRequestScope,
  modelMappingEndpointScopeSupportsSessionCompaction,
  modelMappingOperationsKey,
  modelMappingOperationsFromScopeValue,
  modelMappingRequestScopeOptions,
  modelMappingRequestScopeValue,
  normalizeModelMappingOperations,
} from '../utils/modelMappingScope'

export interface AliasGroup {
  model: Model
  /** @deprecated */
  apiFormatsKey: string
  /** @deprecated */
  apiFormats: string[]
  endpointIdsKey: string
  endpointIds: string[]
  operationsKey: string
  operations: string[]
  aliases: ProviderModelAlias[]
}

const props = defineProps<{
  open: boolean
  providerId: string
  /** @deprecated */
  providerApiFormats?: string[]
  endpoints?: ProviderEndpoint[]
  models: Model[]
  editingGroup?: AliasGroup | null
  preselectedModelId?: string | null
  hasAutoFetchKey?: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': []
}>()

const { t } = useI18n()

const { error: showError, success: showSuccess, warning: showWarning } = useToast()

const { fetchModels: fetchCachedModels } = useUpstreamModelsCache()

type EndpointOption = {
  value: string
  label: string
}

// 状态
const submitting = ref(false)
const loadingModels = ref(false)
const fetchingUpstreamModels = ref(false)
const upstreamModelsLoaded = ref(false)

// 搜索
const searchQuery = ref('')

// 折叠状态
const collapsedGroups = ref<Set<string>>(new Set())

// 上游模型
const upstreamModels = ref<UpstreamModel[]>([])

// 表单数据
const formData = ref<{
  modelId: string
}>({
  modelId: ''
})

// 选中的映射名称
const selectedNames = ref<string[]>([])

// 选中的端点 ID；空数组表示全部端点
const selectedEndpointIds = ref<string[]>([])

const selectedOperations = ref<string[]>([])

// 自定义名称列表（手动添加的）
const allCustomNames = ref<string[]>([])

const endpointOptions = computed<EndpointOption[]>(() => {
  const endpoints = props.endpoints ?? []
  return endpoints.map(endpoint => ({
    value: endpoint.id,
    label: formatModelMappingEndpointLabel(endpoint, endpoints),
  }))
})

const normalizedSelectedEndpointIds = computed(() => {
  const selected = normalizeStringList(selectedEndpointIds.value)
  return selected.length > 0 ? selected : undefined
})

const sessionCompactionScopeAvailable = computed(() => {
  return modelMappingEndpointScopeSupportsSessionCompaction(
    normalizedSelectedEndpointIds.value,
    props.endpoints ?? [],
  )
})

const endpointScopeSummary = computed(() => {
  const selected = normalizedSelectedEndpointIds.value
  if (!selected || selected.length === 0) {
    return t('providers.modelMapping.scope.allEndpoints')
  }
  if (selected.length === 1) {
    return endpointOptions.value.find(option => option.value === selected[0])?.label
      ?? t('providers.modelMapping.scope.endpointCount', { count: 1 })
  }
  return t('providers.modelMapping.scope.endpointCount', { count: selected.length })
})

const requestScopeLabels = computed(() => ({
  allRequests: t('providers.modelMapping.scope.allRequests'),
  sessionCompactionOnly: t('providers.modelMapping.scope.sessionCompactionOnly'),
  customOperations: (operations: string[]) => t(
    'providers.modelMapping.scope.customOperations',
    { operations: operations.join(', ') },
  ),
}))

const normalizedSelectedOperations = computed(() => {
  const selected = normalizeModelMappingOperations(selectedOperations.value)
  return selected.length > 0 ? selected : undefined
})

const operationScopeSummary = computed(() => {
  return formatModelMappingRequestScope(
    normalizedSelectedOperations.value,
    requestScopeLabels.value,
  )
})

const mappingScopeSummary = computed(() => {
  return `${endpointScopeSummary.value} · ${operationScopeSummary.value}`
})

const requestScopeValue = computed(() => {
  return modelMappingRequestScopeValue(selectedOperations.value)
})

const requestScopeOptions = computed(() => {
  return modelMappingRequestScopeOptions(
    selectedOperations.value,
    { sessionCompaction: sessionCompactionScopeAvailable.value },
    requestScopeLabels.value,
  )
})

const requestScopeDescription = computed(() => {
  if (requestScopeValue.value === ALL_REQUESTS_SCOPE_VALUE) {
    return sessionCompactionScopeAvailable.value
      ? t('providers.modelMapping.scope.allRequestsDescription')
      : t('providers.modelMapping.scope.allRequestsDefaultDescription')
  }
  if (requestScopeValue.value === COMPACT_REQUEST_SCOPE_VALUE) {
    return t('providers.modelMapping.scope.sessionCompactionDescription')
  }
  return t('providers.modelMapping.scope.customOperationsDescription', {
    operations: normalizeModelMappingOperations(selectedOperations.value).join(', '),
  })
})

watch(
  [() => props.endpoints, () => selectedEndpointIds.value],
  () => normalizeUnavailableSessionCompactionScope(),
  { deep: true },
)

// 所有已知名称集合
const allKnownNames = computed(() => {
  const set = new Set<string>()
  upstreamModels.value.forEach(m => set.add(m.id))
  return set
})

// 上游模型名称列表（去重后）
const upstreamModelNames = computed(() => {
  const names = new Set<string>()
  upstreamModels.value.forEach(m => {
    names.add(m.id)
  })
  return Array.from(names).sort()
})

// 自定义名称列表（排除上游模型中已有的）
const customNames = computed(() => {
  const upstreamSet = new Set(upstreamModelNames.value)
  return allCustomNames.value.filter(name => !upstreamSet.has(name))
})

// 排序后的自定义名称
const sortedCustomNames = computed(() => {
  const search = searchQuery.value.toLowerCase().trim()
  if (!search) return customNames.value

  const matched: string[] = []
  const unmatched: string[] = []
  for (const name of customNames.value) {
    if (name.toLowerCase().includes(search)) {
      matched.push(name)
    } else {
      unmatched.push(name)
    }
  }
  return [...matched, ...unmatched]
})

// 判断搜索内容是否可以作为自定义名称添加
const canAddAsCustom = computed(() => {
  const search = searchQuery.value.trim()
  if (!search) return false
  if (selectedNames.value.includes(search)) return false
  if (allCustomNames.value.includes(search)) return false
  if (allKnownNames.value.has(search)) return false
  return true
})

// 过滤后的上游模型
const filteredUpstreamModels = computed(() => {
  if (!searchQuery.value.trim()) return upstreamModelNames.value
  const query = searchQuery.value.toLowerCase()
  return upstreamModelNames.value.filter(name => name.toLowerCase().includes(query))
})

// 空状态判断
const showEmptyState = computed(() => {
  return filteredUpstreamModels.value.length === 0 && customNames.value.length === 0
})

// 上游模型是否全选
const isAllUpstreamModelsSelected = computed(() => {
  if (filteredUpstreamModels.value.length === 0) return false
  return filteredUpstreamModels.value.every(name => selectedNames.value.includes(name))
})

// 切换名称选中状态
function toggleName(name: string) {
  const idx = selectedNames.value.indexOf(name)
  if (idx === -1) {
    selectedNames.value.push(name)
  } else {
    selectedNames.value.splice(idx, 1)
  }
}

// 添加自定义名称
function addCustomName() {
  const name = searchQuery.value.trim()
  if (name && !selectedNames.value.includes(name)) {
    selectedNames.value.push(name)
    if (!allKnownNames.value.has(name) && !allCustomNames.value.includes(name)) {
      allCustomNames.value.push(name)
    }
    searchQuery.value = ''
  }
}

// 全选/取消全选上游模型
function toggleAllUpstreamModels() {
  const allNames = filteredUpstreamModels.value
  if (isAllUpstreamModelsSelected.value) {
    selectedNames.value = selectedNames.value.filter(name => !allNames.includes(name))
  } else {
    allNames.forEach(name => {
      if (!selectedNames.value.includes(name)) {
        selectedNames.value.push(name)
      }
    })
  }
}

function normalizeStringList(values: string[] | undefined): string[] {
  const seen = new Set<string>()
  const result: string[] = []
  for (const value of values ?? []) {
    const normalized = value.trim()
    if (!normalized || seen.has(normalized)) continue
    seen.add(normalized)
    result.push(normalized)
  }
  return result
}

function getScopeKey(values: string[] | undefined): string {
  return normalizeStringList(values).sort().join(',')
}

function scopesOverlap(left: string[] | undefined, right: string[] | undefined): boolean {
  const leftValues = normalizeStringList(left)
  const rightValues = normalizeStringList(right)
  if (leftValues.length === 0 || rightValues.length === 0) return true
  const rightSet = new Set(rightValues)
  return leftValues.some(value => rightSet.has(value))
}

function operationScopesOverlap(
  left: string[] | undefined,
  right: string[] | undefined,
): boolean {
  const leftValues = normalizeModelMappingOperations(left)
  const rightValues = normalizeModelMappingOperations(right)
  if (leftValues.length === 0 || rightValues.length === 0) return true
  const rightSet = new Set(rightValues)
  return leftValues.some(value => rightSet.has(value))
}

function findDuplicateNames(
  existingAliases: ProviderModelAlias[],
  names: string[],
  endpointIds: string[] | undefined,
  apiFormats: string[] | undefined = undefined,
  operations: string[] | undefined = undefined,
): string[] {
  const duplicates = new Set<string>()
  for (const rawName of names) {
    const name = rawName.trim()
    if (!name) continue
    const duplicate = existingAliases.some((alias) => {
      return alias.name === name
        && scopesOverlap(alias.endpoint_ids, endpointIds)
        && scopesOverlap(alias.api_formats, apiFormats)
        && operationScopesOverlap(alias.operations, operations)
    })
    if (duplicate) duplicates.add(name)
  }
  return Array.from(duplicates)
}

// 切换折叠状态
function toggleGroupCollapse(group: string) {
  if (collapsedGroups.value.has(group)) {
    collapsedGroups.value.delete(group)
  } else {
    collapsedGroups.value.add(group)
  }
  collapsedGroups.value = new Set(collapsedGroups.value)
}

// 从提供商获取模型（使用缓存）
async function fetchUpstreamModels(forceRefresh = false) {
  if (!props.providerId) return
  try {
    loadingModels.value = true
    fetchingUpstreamModels.value = true
    const result = await fetchCachedModels(props.providerId, undefined, forceRefresh)
    if (result.models.length > 0) {
      upstreamModels.value = result.models
      upstreamModelsLoaded.value = true
      // 获取上游模型后，将不在上游列表中的已选名称添加到自定义列表
      const upstreamIds = new Set(result.models.map(m => m.id))
      const customFromSelected = selectedNames.value.filter(name => !upstreamIds.has(name))
      const mergedCustom = new Set([...allCustomNames.value, ...customFromSelected])
      allCustomNames.value = Array.from(mergedCustom).filter(name => !upstreamIds.has(name))
    }
    if (result.warning) {
      showWarning(result.warning, t('ui.failedToFetchSomeFormats'))
    }
    if (result.error) {
      showError(result.error, t('ui.failedToFetchUpstreamModels'))
    }
  } catch (err: unknown) {
    showError(parseApiError(err, t('ui.failedToFetchUpstreamModelList')), t('ui.error'))
  } finally {
    loadingModels.value = false
    fetchingUpstreamModels.value = false
  }
}

// 监听打开状态
watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      initForm()
      if (props.hasAutoFetchKey) {
        await fetchUpstreamModels()
      }
    }
  },
  { immediate: true },
)

// 初始化表单
function initForm() {
  if (props.editingGroup) {
    formData.value = {
      modelId: props.editingGroup.model.id
    }
    const existingNames = props.editingGroup.aliases.map(a => a.name)
    selectedNames.value = [...existingNames]
    selectedEndpointIds.value = normalizeStringList(props.editingGroup.endpointIds)
    selectedOperations.value = normalizeModelMappingOperations(props.editingGroup.operations)
    allCustomNames.value = [...existingNames]
  } else {
    formData.value = {
      modelId: props.preselectedModelId || ''
    }
    selectedNames.value = []
    selectedEndpointIds.value = []
    selectedOperations.value = []
    allCustomNames.value = []
  }
  searchQuery.value = ''
  upstreamModels.value = []
  upstreamModelsLoaded.value = false
  collapsedGroups.value = new Set()
  normalizeUnavailableSessionCompactionScope()
}

// 处理模型选择变更
function handleModelChange(value: string) {
  formData.value.modelId = value
}

function handleRequestScopeChange(value: string) {
  if (
    value === COMPACT_REQUEST_SCOPE_VALUE
    && !sessionCompactionScopeAvailable.value
  ) return
  selectedOperations.value = modelMappingOperationsFromScopeValue(value) ?? []
}

function normalizeUnavailableSessionCompactionScope() {
  // An empty list is also used while endpoints are loading or after loading fails,
  // so it cannot prove that the existing scope is unsupported.
  if (!props.endpoints || props.endpoints.length === 0) return
  if (
    requestScopeValue.value === COMPACT_REQUEST_SCOPE_VALUE
    && !sessionCompactionScopeAvailable.value
  ) {
    selectedOperations.value = []
  }
}

// 生成作用域唯一键
function getApiFormatsKey(formats: string[] | undefined): string {
  return getScopeKey(formats)
}

function getEndpointIdsKey(endpointIds: string[] | undefined): string {
  return getScopeKey(endpointIds)
}

function getOperationsKey(operations: string[] | undefined): string {
  return modelMappingOperationsKey(operations)
}

// 提交表单
async function handleSubmit() {
  if (submitting.value) return
  if (!formData.value.modelId || selectedNames.value.length === 0) return

  submitting.value = true
  try {
    const targetModel = props.models.find(m => m.id === formData.value.modelId)
    if (!targetModel) {
      showError(t('ui.modelNotFound'), t('ui.error'))
      return
    }

    const currentAliases = targetModel.provider_model_mappings || []
    let newAliases: ProviderModelAlias[]
    const nextEndpointIds = normalizedSelectedEndpointIds.value
    const nextOperations = normalizedSelectedOperations.value

    const buildAliases = (names: string[]): ProviderModelAlias[] => {
      return names.map((name) => {
        const alias: ProviderModelAlias = {
          name: name.trim(),
          priority: 1
        }
        if (nextEndpointIds && nextEndpointIds.length > 0) {
          alias.endpoint_ids = nextEndpointIds
        }
        if (nextOperations && nextOperations.length > 0) {
          alias.operations = nextOperations
        }
        return alias
      })
    }

    if (props.editingGroup) {
      const oldApiFormatsKey = props.editingGroup.apiFormatsKey
      const oldEndpointIdsKey = props.editingGroup.endpointIdsKey
      const oldOperationsKey = modelMappingOperationsKey(props.editingGroup.operations)
      const oldAliasNames = new Set(props.editingGroup.aliases.map(a => a.name))

      const filteredAliases = currentAliases.filter((a: ProviderModelAlias) => {
        const currentKey = getApiFormatsKey(a.api_formats)
        const currentEndpointIdsKey = getEndpointIdsKey(a.endpoint_ids)
        const currentOperationsKey = getOperationsKey(a.operations)
        return !(currentKey === oldApiFormatsKey
          && currentEndpointIdsKey === oldEndpointIdsKey
          && currentOperationsKey === oldOperationsKey
          && oldAliasNames.has(a.name))
      })

      const duplicates = findDuplicateNames(
        filteredAliases,
        selectedNames.value,
        nextEndpointIds,
        undefined,
        nextOperations,
      )
      if (duplicates.length > 0) {
        showError(t('providers.migration.theseMappingNamesAlreadyExistdcbf0a', { value: String(duplicates.join(', ')) }), t('ui.error'))
        return
      }

      newAliases = [
        ...filteredAliases,
        ...buildAliases(selectedNames.value)
      ]
    } else {
      const duplicates = findDuplicateNames(
        currentAliases,
        selectedNames.value,
        nextEndpointIds,
        undefined,
        nextOperations,
      )
      if (duplicates.length > 0) {
        showError(t('providers.migration.theseMappingNamesAlreadyExistdcbf0a', { value: String(duplicates.join(', ')) }), t('ui.error'))
        return
      }
      newAliases = [
        ...currentAliases,
        ...buildAliases(selectedNames.value)
      ]
    }

    await updateModel(props.providerId, targetModel.id, {
      provider_model_mappings: newAliases
    })

    showSuccess(props.editingGroup ? t('ui.mappingGroupUpdated') : t('ui.mappingAdded'))
    emit('update:open', false)
    emit('saved')
  } catch (err: unknown) {
    showError(parseApiError(err, t('ui.operationFailed')), t('ui.error'))
  } finally {
    submitting.value = false
  }
}
</script>
