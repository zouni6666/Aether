<template>
  <div class="flex flex-col h-[calc(100vh-12rem)]">
    <!-- 主内容区 -->
    <div class="flex-1 flex flex-col min-w-0">
      <!-- 模型列表 -->
      <Card class="overflow-hidden">
        <!-- 标题和操作栏 -->
        <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60">
          <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
            <!-- 左侧：标题 -->
            <div class="flex min-w-0 items-baseline gap-2">
              <h3 class="text-sm sm:text-base font-semibold shrink-0">
                {{ t('nav.modelManagement') }}
              </h3>
              <span
                v-if="selectedBatchManageModelIds.size > 0"
                class="text-xs font-medium text-primary"
                aria-live="polite"
              >
                {{ t('models.management.selectedCount', { count: selectedBatchManageModelIds.size }) }}
              </span>
            </div>

            <!-- 右侧：操作区 -->
            <div class="flex flex-wrap items-center gap-2">
              <!-- 搜索框 -->
              <div class="relative">
                <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground/70 z-10 pointer-events-none" />
                <Input
                  id="model-search"
                  v-model="searchQuery"
                  type="text"
                  :placeholder="t('ui.searchModels')"
                  class="w-32 sm:w-44 pl-8 pr-3 h-8 text-sm bg-muted/30 border-border/50 focus:border-primary/50 transition-colors"
                />
              </div>

              <!-- 操作按钮 -->
              <Button
                variant="outline"
                size="sm"
                class="h-8 gap-1.5 px-2.5"
                :title="selectedBatchManageModelIds.size > 0
                  ? t('models.management.manageSelectedTitle', { count: selectedBatchManageModelIds.size })
                  : t('models.management.batch.title')"
                @click="openBatchManageDialog"
              >
                <ListChecks class="w-3.5 h-3.5" />
                <span>{{ selectedBatchManageModelIds.size > 0
                  ? t('models.management.batchButtonSelected', { count: selectedBatchManageModelIds.size })
                  : t('models.management.batchButton') }}</span>
              </Button>
              <ExternalModelsAccessControl />
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8"
                :title="t('ui.createModel')"
                @click="openCreateModelDialog"
              >
                <Plus class="w-3.5 h-3.5" />
              </Button>
              <RefreshButton
                :loading="loading"
                @click="refreshData"
              />
            </div>
          </div>
        </div>

        <Table class="hidden xl:table">
          <TableHeader>
            <TableRow>
              <TableHead class="w-[250px]">
                <div class="flex items-center gap-2">
                  <Checkbox
                    class="h-3.5 w-3.5 shrink-0"
                    :checked="isCurrentModelPageFullySelected"
                    :indeterminate="isCurrentModelPagePartiallySelected"
                    :disabled="paginatedGlobalModels.length === 0 || loading"
                    :aria-label="t('models.management.selectCurrentPage')"
                    data-testid="model-select-page-desktop"
                    @update:checked="toggleCurrentModelPageSelection($event === true)"
                  />
                  <span>{{ t('ui.modelName') }}</span>
                </div>
              </TableHead>
              <TableHead class="w-[160px] text-center">
                {{ t('ui.priceM') }}
              </TableHead>
              <TableHead class="w-[80px] text-center">
                {{ t('nav.providers') }}
              </TableHead>
              <TableHead class="w-[80px] text-center">
                {{ t('ui.callCount') }}
              </TableHead>
              <TableHead class="w-[70px]">
                {{ t('ui.status6320b4a') }}
              </TableHead>
              <TableHead class="w-[140px] text-center">
                {{ t('ui.actionsed31fbb') }}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-if="loading">
              <TableCell
                colspan="6"
                class="text-center py-8"
              >
                <Loader2 class="w-6 h-6 animate-spin mx-auto" />
              </TableCell>
            </TableRow>
            <TableRow v-else-if="filteredGlobalModels.length === 0">
              <TableCell
                colspan="6"
                class="text-center py-8 text-muted-foreground"
              >
                {{ t('ui.noMatchingModelsFound') }}
              </TableCell>
            </TableRow>
            <template v-else>
              <TableRow
                v-for="model in paginatedGlobalModels"
                :key="model.id"
                class="cursor-pointer hover:bg-muted/50 group"
                :class="selectedBatchManageModelIds.has(model.id) ? 'bg-primary/5' : ''"
                @mousedown="handleMouseDown"
                @click="handleRowClick($event, model)"
              >
                <TableCell>
                  <div class="flex min-w-0 items-start gap-2">
                    <Checkbox
                      class="mt-0.5 h-3.5 w-3.5 shrink-0"
                      :checked="selectedBatchManageModelIds.has(model.id)"
                      :aria-label="t('models.management.selectModel', { name: model.display_name || model.name })"
                      :data-testid="`model-select-desktop-${model.id}`"
                      @click.stop
                      @update:checked="setBatchManageModelSelection(model.id, $event === true)"
                    />
                    <div class="min-w-0 flex-1">
                      <div class="font-medium truncate">
                        {{ model.display_name }}
                      </div>
                      <div class="text-xs text-muted-foreground flex items-center gap-1">
                        <span class="truncate">{{ model.name }}</span>
                        <button
                          class="p-0.5 rounded hover:bg-muted transition-colors shrink-0"
                          :title="t('ui.copyModelId')"
                          @click.stop="copyToClipboard(model.name)"
                        >
                          <Copy class="w-3 h-3" />
                        </button>
                      </div>
                    </div>
                  </div>
                </TableCell>
                <TableCell class="text-center">
                  <div class="text-xs space-y-0.5">
                    <!-- 按 Token 计费 -->
                    <div v-if="getFirstTierPrice(model, 'input') || getFirstTierPrice(model, 'output')">
                      <span class="text-muted-foreground">In:</span>
                      <span class="font-mono ml-1">{{ getFirstTierPrice(model, 'input')?.toFixed(2) || '-' }}</span>
                      <span class="text-muted-foreground mx-1">/</span>
                      <span class="text-muted-foreground">Out:</span>
                      <span class="font-mono ml-1">{{ getFirstTierPrice(model, 'output')?.toFixed(2) || '-' }}</span>
                      <!-- 阶梯计费标记 -->
                      <span
                        v-if="hasTieredPricing(model)"
                        class="ml-1 text-muted-foreground"
                        :title="t('ui.tieredPricing')"
                      >{{ t('ui.tiered') }}</span>
                    </div>
                    <!-- 按次计费 -->
                    <div v-if="model.default_price_per_request && model.default_price_per_request > 0">
                      <span class="text-muted-foreground">{{ t('ui.perRequest5b310fa') }}</span>
                      <span class="font-mono ml-1">${{ model.default_price_per_request.toFixed(3) }}{{ t('ui.requestad9c0d7') }}</span>
                    </div>
                    <!-- 视频费用计费 -->
                    <div v-if="hasVideoPricing(model)">
                      <span class="text-muted-foreground">{{ t('ui.video61a56bb') }}</span>
                      <span
                        class="font-mono ml-1"
                        :title="getVideoPricingTooltip(model)"
                      >{{ getVideoPricingDisplay(model) }}</span>
                    </div>
                    <!-- 无计费配置 -->
                    <div
                      v-if="!getFirstTierPrice(model, 'input') && !getFirstTierPrice(model, 'output') && !model.default_price_per_request && !hasVideoPricing(model)"
                      class="text-muted-foreground"
                    >
                      -
                    </div>
                  </div>
                </TableCell>
                <TableCell class="text-center">
                  <Badge variant="secondary">
                    {{ model.active_provider_count || 0 }}/{{ model.provider_count || 0 }}
                  </Badge>
                </TableCell>
                <TableCell class="text-center">
                  <span class="text-sm font-mono">{{ formatUsageCount(model.usage_count || 0) }}</span>
                </TableCell>
                <TableCell>
                  <Badge :variant="model.is_active ? 'default' : 'secondary'">
                    {{ model.is_active ? t('ui.activee85e590') : t('ui.disable4e6fd0e') }}
                  </Badge>
                </TableCell>
                <TableCell>
                  <div class="flex items-center justify-center gap-1">
                    <Button
                      variant="ghost"
                      size="icon"
                      class="h-8 w-8"
                      :title="t('ui.viewDetails')"
                      @click.stop="selectModel(model)"
                    >
                      <Eye class="w-4 h-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      class="h-8 w-8"
                      :title="t('ui.editModel')"
                      @click.stop="editModel(model)"
                    >
                      <Edit class="w-4 h-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      class="h-8 w-8"
                      :title="model.is_active ? t('ui.disableModel') : t('ui.enableModel')"
                      @click.stop="toggleModelStatus(model)"
                    >
                      <Power class="w-4 h-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      class="h-8 w-8"
                      :title="t('ui.deleteModel')"
                      @click.stop="deleteModel(model)"
                    >
                      <Trash2 class="w-4 h-4" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            </template>
          </TableBody>
        </Table>

        <!-- 移动端卡片列表 -->
        <div
          v-if="!loading && paginatedGlobalModels.length > 0"
          class="xl:hidden divide-y divide-border/40"
        >
          <div
            v-for="model in paginatedGlobalModels"
            :key="model.id"
            class="p-4 space-y-3 hover:bg-muted/50 cursor-pointer transition-colors"
            :class="selectedBatchManageModelIds.has(model.id) ? 'bg-primary/5' : ''"
            @click="selectModel(model)"
          >
            <!-- 第一行：名称 + 状态 + 操作 -->
            <div class="flex items-start justify-between gap-3">
              <div class="flex min-w-0 flex-1 items-start gap-2">
                <Checkbox
                  class="mt-0.5 h-4 w-4 shrink-0"
                  :checked="selectedBatchManageModelIds.has(model.id)"
                  :aria-label="t('models.management.selectModel', { name: model.display_name || model.name })"
                  :data-testid="`model-select-mobile-${model.id}`"
                  @click.stop
                  @update:checked="setBatchManageModelSelection(model.id, $event === true)"
                />
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-2">
                    <span class="font-medium truncate">{{ model.display_name }}</span>
                    <Badge
                      :variant="model.is_active ? 'default' : 'secondary'"
                      class="text-xs shrink-0"
                    >
                      {{ model.is_active ? t('ui.activee85e590') : t('ui.disable4e6fd0e') }}
                    </Badge>
                  </div>
                  <div class="text-xs text-muted-foreground flex items-center gap-1 mt-0.5">
                    <span class="font-mono truncate">{{ model.name }}</span>
                    <button
                      class="p-0.5 rounded hover:bg-muted transition-colors shrink-0"
                      @click.stop="copyToClipboard(model.name)"
                    >
                      <Copy class="w-3 h-3" />
                    </button>
                  </div>
                </div>
              </div>
              <div
                class="flex items-center gap-0.5 shrink-0"
                @click.stop
              >
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7"
                  @click="editModel(model)"
                >
                  <Edit class="w-3.5 h-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7"
                  @click="toggleModelStatus(model)"
                >
                  <Power class="w-3.5 h-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7"
                  @click="deleteModel(model)"
                >
                  <Trash2 class="w-3.5 h-3.5" />
                </Button>
              </div>
            </div>

            <!-- 第二行：统计信息 -->
            <div class="flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
              <span>{{ t('nav.providers') }} {{ model.active_provider_count || 0 }}/{{ model.provider_count || 0 }}</span>
              <span>{{ t('ui.calls') }} {{ formatUsageCount(model.usage_count || 0) }}</span>
              <span
                v-if="getFirstTierPrice(model, 'input') || getFirstTierPrice(model, 'output')"
                class="font-mono"
              >
                ${{ getFirstTierPrice(model, 'input')?.toFixed(2) || '-' }}/${{ getFirstTierPrice(model, 'output')?.toFixed(2) || '-' }}
              </span>
            </div>
          </div>
        </div>

        <!-- 分页 -->
        <Pagination
          v-if="!loading && totalGlobalModels > 0"
          :current="catalogCurrentPage"
          :total="totalGlobalModels"
          :page-size="catalogPageSize"
          cache-key="model-management-page-size"
          @update:current="catalogCurrentPage = $event"
          @update:page-size="catalogPageSize = $event"
        />
      </Card>
    </div>

    <!-- 创建/编辑模型对话框 -->
    <GlobalModelFormDialog
      :open="createModelDialogOpen"
      :model="editingModel"
      @update:open="handleModelDialogUpdate"
      @success="handleModelFormSuccess"
      @edit-model="editModel"
      @pricing-synced="handleModelPricingSynced"
    />

    <!-- 模型详情抽屉 -->
    <ModelDetailDrawer
      ref="modelDetailDrawerRef"
      :model="selectedModel"
      :open="!!selectedModel"
      :has-blocking-dialog-open="hasBlockingDialogOpen"
      @update:open="handleDrawerOpenChange"
      @edit-model="editModel"
      @toggle-model-status="toggleModelStatus"
      @add-provider="openAddProviderDialog"
      @edit-provider="openEditProviderImplementation"
      @delete-provider="confirmDeleteProviderImplementation"
      @toggle-provider-status="toggleProviderStatus"
      @refresh-model="refreshSelectedModel"
      @link-provider="linkProviderToModel"
      @link-providers="linkProvidersToModel"
    />

    <!-- 批量添加关联提供商对话框 -->
    <ModelBatchProvidersDialog
      :open="batchAddProvidersDialogOpen"
      :model="selectedModel"
      :linked-providers="selectedModelProviders"
      @update:open="batchAddProvidersDialogOpen = $event"
      @saved="handleBatchProvidersSaved"
    />

    <!-- 编辑提供商模型对话框 -->
    <ProviderModelFormDialog
      :open="editProviderDialogOpen"
      :provider-id="editingProvider?.id || ''"
      :provider-name="editingProvider?.name || ''"
      :editing-model="editingProviderModel"
      @update:open="handleEditProviderDialogUpdate"
      @saved="handleEditProviderSaved"
    />

    <!-- 批量管理全局模型对话框 -->
    <ModelBatchManageDialog
      :open="batchManageDialogOpen"
      :selected-ids="selectedBatchManageModelIds"
      @update:open="batchManageDialogOpen = $event"
      @update:selected-ids="selectedBatchManageModelIds = $event"
      @model-updated="applyGlobalModelUpdate"
      @models-deleted="handleBatchModelsDeleted"
      @changed="loadGlobalModels"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onBeforeUnmount, watch } from 'vue'
import {
  Plus,
  Edit,
  Trash2,
  Loader2,
  Eye,
  Search,
  Power,
  Copy,
  ListChecks,
} from 'lucide-vue-next'
import ModelDetailDrawer from '@/features/models/components/ModelDetailDrawer.vue'
import GlobalModelFormDialog from '@/features/models/components/GlobalModelFormDialog.vue'
import ExternalModelsAccessControl from '@/features/models/components/ExternalModelsAccessControl.vue'
import ModelBatchProvidersDialog from '@/features/models/components/ModelBatchProvidersDialog.vue'
import ModelBatchManageDialog from '@/features/models/components/ModelBatchManageDialog.vue'
import { useModelsDevPricingSourceMigration } from '@/features/models/composables/useModelsDevPricingSourceMigration'
import {
  getFirstTierPrice,
  getVideoPricingDisplay,
  getVideoPricingTooltip,
  hasTieredPricing,
  hasVideoPricing,
} from '@/features/models/utils/globalModelPricing'
import ProviderModelFormDialog from '@/features/providers/components/ProviderModelFormDialog.vue'
import type { Model } from '@/api/endpoints'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { useClipboard } from '@/composables/useClipboard'
import { useRowClick } from '@/composables/useRowClick'
import { useI18n } from '@/i18n'
import { parseApiError } from '@/utils/errorParser'
import {
  Button,
  Card,
  Checkbox,
  Input,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
  Badge,
  Pagination,
  RefreshButton,
} from '@/components/ui'
import {
  listGlobalModels,
  getGlobalModel,
  updateGlobalModel,
  deleteGlobalModel,
  batchAssignToProviders,
  getGlobalModelProviders,
  type GlobalModelResponse,
} from '@/api/global-models'
import { log } from '@/utils/logger'
import { formatUsageCount } from '@/utils/format'
import {
  cloneTieredPricingConfig,
} from '@/features/models/components/global-model-form-helpers'

interface ModelProviderDisplay {
  id: string
  model_id?: string | null
  name: string
  provider_type?: string
  target_model?: string
  is_active: boolean
  input_price_per_1m?: number | null
  output_price_per_1m?: number | null
  cache_creation_price_per_1m?: number | null
  cache_read_price_per_1m?: number | null
  cache_1h_creation_price_per_1m?: number | null
  price_per_request?: number | null
  effective_tiered_pricing?: unknown
  tier_count?: number
  supports_vision?: boolean | null
  supports_function_calling?: boolean | null
  supports_streaming?: boolean | null
  supports_extended_thinking?: boolean | null
  supports_embedding?: boolean | null
}

const { success, error: showError } = useToast()
const { t } = useI18n()
const { copyToClipboard } = useClipboard()

// 状态
const loading = ref(false)
const detailTab = ref('basic')
const searchQuery = ref('')
const selectedModel = ref<GlobalModelResponse | null>(null)
const modelDetailDrawerRef = ref<InstanceType<typeof ModelDetailDrawer> | null>(null)
const createModelDialogOpen = ref(false)
const editingModel = ref<GlobalModelResponse | null>(null)

// 数据
const globalModels = ref<GlobalModelResponse[]>([])
const totalGlobalModels = ref(0)
let globalModelsRequestId = 0
let modelSelectionRequestId = 0
let modelProvidersRequestId = 0
const GLOBAL_MODELS_LIST_CACHE_TTL_MS = 10 * 1000

// 模型目录分页
const catalogCurrentPage = ref(1)
const catalogPageSize = ref(20)

// 选中模型的详细数据
const selectedModelProviders = ref<ModelProviderDisplay[]>([])
const loadingModelProviders = ref(false)

// 批量添加关联提供商
const batchAddProvidersDialogOpen = ref(false)

// 编辑提供商模型
const editProviderDialogOpen = ref(false)
const editingProvider = ref<ModelProviderDisplay | null>(null)

// 批量管理全局模型
const batchManageDialogOpen = ref(false)
const { migrateLegacyModelsDevPricingSources } = useModelsDevPricingSourceMigration(applyGlobalModelUpdate)
const selectedBatchManageModelIds = ref<Set<string>>(new Set())

// 将 provider 数据转换为 Model 类型供 ProviderModelFormDialog 使用
const editingProviderModel = computed<Model | null>(() => {
  if (!editingProvider.value) return null

  const p = editingProvider.value

  return {
    id: p.model_id,
    provider_id: p.id,
    provider_model_name: p.target_model || '',
    // 使用 API 返回的完整阶梯配置
    tiered_pricing: null,  // 原始配置为空（继承模式）
    effective_tiered_pricing: p.effective_tiered_pricing,  // 有效配置（含继承）
    price_per_request: p.price_per_request,
    supports_streaming: p.supports_streaming,
    supports_vision: p.supports_vision,
    supports_function_calling: p.supports_function_calling,
    supports_extended_thinking: p.supports_extended_thinking,
    supports_embedding: p.supports_embedding,
    is_active: p.is_active,
    global_model_display_name: selectedModel.value?.display_name,
  } as Model
})

// 使用全局确认对话框
const { confirmDanger } = useConfirm()

// 检测是否有对话框打开（防止误关闭抽屉）
const hasBlockingDialogOpen = computed(() =>
  createModelDialogOpen.value ||
  batchAddProvidersDialogOpen.value ||
  editProviderDialogOpen.value ||
  batchManageDialogOpen.value
)

// 能力筛选
const capabilityFilters = ref({
  streaming: false,
  imageGeneration: false,
  vision: false,
  toolUse: false,
  extendedThinking: false,
})

// 筛选后的模型列表
const filteredGlobalModels = computed(() => {
  let result = globalModels.value

  // 能力筛选
  if (capabilityFilters.value.streaming) {
    result = result.filter(m => m.config?.streaming !== false)
  }
  if (capabilityFilters.value.imageGeneration) {
    result = result.filter(m => m.config?.image_generation === true)
  }
  if (capabilityFilters.value.vision) {
    result = result.filter(m => m.config?.vision === true)
  }
  if (capabilityFilters.value.toolUse) {
    result = result.filter(m => m.config?.function_calling === true)
  }
  if (capabilityFilters.value.extendedThinking) {
    result = result.filter(m => m.config?.extended_thinking === true)
  }

  return result
})

// 模型目录分页计算
const paginatedGlobalModels = computed(() => filteredGlobalModels.value)
const selectedOnCurrentModelPageCount = computed(() => (
  paginatedGlobalModels.value.filter(model => selectedBatchManageModelIds.value.has(model.id)).length
))
const isCurrentModelPageFullySelected = computed(() => (
  paginatedGlobalModels.value.length > 0
  && selectedOnCurrentModelPageCount.value === paginatedGlobalModels.value.length
))
const isCurrentModelPagePartiallySelected = computed(() => (
  selectedOnCurrentModelPageCount.value > 0 && !isCurrentModelPageFullySelected.value
))

function setBatchManageModelSelection(modelId: string, selected: boolean) {
  const nextSelection = new Set(selectedBatchManageModelIds.value)
  if (selected) nextSelection.add(modelId)
  else nextSelection.delete(modelId)
  selectedBatchManageModelIds.value = nextSelection
}

function toggleCurrentModelPageSelection(selected: boolean) {
  const nextSelection = new Set(selectedBatchManageModelIds.value)
  for (const model of paginatedGlobalModels.value) {
    if (selected) nextSelection.add(model.id)
    else nextSelection.delete(model.id)
  }
  selectedBatchManageModelIds.value = nextSelection
}

watch(searchQuery, () => {
  catalogCurrentPage.value = 1
})

watch(catalogPageSize, () => {
  catalogCurrentPage.value = 1
})

const globalModelsQueryParams = computed(() => ({
  skip: Math.max(0, (catalogCurrentPage.value - 1) * catalogPageSize.value),
  limit: catalogPageSize.value,
  search: searchQuery.value.trim() || undefined,
}))

let modelSearchDebounceTimer: ReturnType<typeof setTimeout> | null = null

async function loadGlobalModels(options: { cacheTtlMs?: number } = {}) {
  const requestId = ++globalModelsRequestId
  loading.value = true
  try {
    const response = await listGlobalModels(globalModelsQueryParams.value, {
      cacheTtlMs: options.cacheTtlMs ?? 0,
    })
    if (requestId !== globalModelsRequestId) return

    const pageModels = response.models || []
    const total = typeof response.total === 'number' ? response.total : pageModels.length
    const totalPages = Math.max(1, Math.ceil(total / Math.max(catalogPageSize.value, 1)))
    if (total > 0 && catalogCurrentPage.value > totalPages) {
      catalogCurrentPage.value = totalPages
      return
    }

    globalModels.value = pageModels
    totalGlobalModels.value = total
    migrateLegacyModelsDevPricingSources(pageModels)
  } catch (err: unknown) {
    if (requestId !== globalModelsRequestId) return
    log.error('加载模型失败:', err)
    showError(parseApiError(err, t('ui.failedToLoadModels')), t('ui.failedToLoadModels'))
  } finally {
    if (requestId === globalModelsRequestId) {
      loading.value = false
    }
  }
}

// 使用复用的行点击逻辑
const { handleMouseDown, shouldTriggerRowClick } = useRowClick()

// 处理行点击，如果用户选择了文字则不触发抽屉
function handleRowClick(event: MouseEvent, model: GlobalModelResponse) {
  if (!shouldTriggerRowClick(event)) return
  selectModel(model)
}

async function selectModel(model: GlobalModelResponse) {
  const requestId = ++modelSelectionRequestId
  // 先显示缓存数据，提升响应速度
  selectedModel.value = model
  detailTab.value = 'basic'

  // 并行加载最新模型数据和关联提供商
  const [latestModel] = await Promise.all([
    getGlobalModel(model.id).catch(err => {
      log.error('获取最新模型数据失败:', err)
      return null
    }),
    loadModelProviders(model.id)
  ])

  // 更新为最新数据（如果获取成功）
  if (requestId !== modelSelectionRequestId) return
  if (latestModel) {
    selectedModel.value = latestModel
  }
}

// 刷新当前选中的模型数据
async function refreshSelectedModel() {
  if (!selectedModel.value) return
  try {
    const updated = await getGlobalModel(selectedModel.value.id)
    selectedModel.value = updated
  } catch (err) {
    log.error('刷新模型数据失败:', err)
  }
}

// 加载指定模型的关联提供商
async function loadModelProviders(_globalModelId: string) {
  const requestId = ++modelProvidersRequestId
  loadingModelProviders.value = true
  try {
    // 使用新的 API 获取所有关联提供商（包括非活跃的）
    const response = await getGlobalModelProviders(_globalModelId)
    if (requestId !== modelProvidersRequestId) return

    // 转换为展示格式
    selectedModelProviders.value = response.providers.map(p => ({
      id: p.provider_id,
      model_id: p.model_id,
      name: p.provider_name,
      provider_type: 'API',
      target_model: p.target_model,
      is_active: p.is_active,
      // 价格信息
      input_price_per_1m: p.input_price_per_1m,
      output_price_per_1m: p.output_price_per_1m,
      cache_creation_price_per_1m: p.cache_creation_price_per_1m,
      cache_read_price_per_1m: p.cache_read_price_per_1m,
      cache_1h_creation_price_per_1m: p.cache_1h_creation_price_per_1m,
      price_per_request: p.price_per_request,
      effective_tiered_pricing: p.effective_tiered_pricing,
      tier_count: p.tier_count,
      // 能力信息
      supports_vision: p.supports_vision,
      supports_function_calling: p.supports_function_calling,
      supports_streaming: p.supports_streaming,
      supports_embedding: p.supports_embedding
    }))
  } catch (err: unknown) {
    if (requestId !== modelProvidersRequestId) return
    log.error('加载关联提供商失败:', err)
    showError(parseApiError(err, t('ui.failedToLoadLinkedProviders')), t('ui.error'))
    selectedModelProviders.value = []
  } finally {
    if (requestId === modelProvidersRequestId) {
      loadingModelProviders.value = false
    }
  }
}

// 打开添加关联提供商对话框
function openAddProviderDialog() {
  if (!selectedModel.value) return
  batchAddProvidersDialogOpen.value = true
}

async function handleBatchProvidersSaved() {
  if (!selectedModel.value) return
  await loadModelProviders(selectedModel.value.id)
  await loadGlobalModels()
  modelDetailDrawerRef.value?.refreshRoutingData?.()
}

function openBatchManageDialog() {
  batchManageDialogOpen.value = true
}

function handleBatchModelsDeleted(ids: string[]) {
  if (selectedModel.value && ids.includes(selectedModel.value.id)) {
    selectedModel.value = null
  }
}

// 关联指定提供商到当前模型
async function linkProviderToModel(providerId: string) {
  await linkProvidersToModel([providerId])
}

// 批量关联提供商到当前模型
async function linkProvidersToModel(providerIds: string[]) {
  if (!selectedModel.value || providerIds.length === 0) return

  try {
    const result = await batchAssignToProviders(selectedModel.value.id, {
      provider_ids: providerIds,
      create_models: true
    })

    // 显示关联结果
    if (result.errors.length > 0) {
      showError(`${result.errors.length} 个提供商关联失败`, t('ui.someOperationsFailed'))
    } else {
      success(`${providerIds.length} 个提供商已关联`)
    }

    // 刷新数据
    await loadModelProviders(selectedModel.value.id)
    await loadGlobalModels()
    modelDetailDrawerRef.value?.refreshRoutingData?.()
  } catch (err: unknown) {
    showError(parseApiError(err, t('ui.failedToLink')), t('ui.error'))
  }
}

function applyGlobalModelUpdate(updatedModel: GlobalModelResponse) {
  const current = globalModels.value.find(model => model.id === updatedModel.id)
  if (current) Object.assign(current, updatedModel)
  if (editingModel.value?.id === updatedModel.id) {
    editingModel.value = { ...editingModel.value, ...updatedModel }
  }
  if (selectedModel.value?.id === updatedModel.id) {
    selectedModel.value = { ...selectedModel.value, ...updatedModel }
  }
}

// 抽屉控制函数
function handleDrawerOpenChange(value: boolean) {
  if (!value && !hasBlockingDialogOpen.value) {
    modelSelectionRequestId += 1
    modelProvidersRequestId += 1
    selectedModel.value = null
  }
}

// 编辑提供商模型
function openEditProviderImplementation(provider: ModelProviderDisplay) {
  editingProvider.value = selectedModelProviders.value.find((item) => item.id === provider.id && item.model_id === provider.model_id) ?? provider
  editProviderDialogOpen.value = true
}

// 处理编辑 Provider 对话框关闭事件
function handleEditProviderDialogUpdate(value: boolean) {
  editProviderDialogOpen.value = value
  if (!value) {
    editingProvider.value = null
  }
}

// 编辑提供商模型保存成功后刷新列表
async function handleEditProviderSaved() {
  if (selectedModel.value) {
    await loadModelProviders(selectedModel.value.id)
  }
}

// 切换关联提供商状态
async function toggleProviderStatus(provider: ModelProviderDisplay) {
  if (!provider.model_id) {
    showError(t('ui.modelIdIsMissing'))
    return
  }

  try {
    const { updateModel } = await import('@/api/endpoints')
    const newStatus = !provider.is_active
    await updateModel(provider.id, provider.model_id, { is_active: newStatus })
    provider.is_active = newStatus
    success(newStatus ? t('ui.linkedProviderEnabled') : t('ui.linkedProviderDisabled'))
    // 刷新路由数据
    modelDetailDrawerRef.value?.refreshRoutingData?.()
  } catch (err: unknown) {
    showError(parseApiError(err, t('ui.failedToUpdateStatus')))
  }
}

// 删除关联提供商
async function confirmDeleteProviderImplementation(provider: ModelProviderDisplay) {
  if (!provider.model_id) {
    showError(t('ui.modelIdIsMissing'))
    return
  }

  const confirmed = await confirmDanger(
    `确定要删除 ${provider.name} 的模型关联吗？\n\n此操作不可恢复！`,
    t('ui.removeLinkedProvider')
  )
  if (!confirmed) return

  try {
    const { deleteModel } = await import('@/api/endpoints')
    await deleteModel(provider.id, provider.model_id)
    success(`已删除 ${provider.name} 的模型实现`)
    // 同步更新 selectedModelProviders 确保状态一致
    if (selectedModel.value) {
      await loadModelProviders(selectedModel.value.id)
    }
    // 刷新路由数据
    modelDetailDrawerRef.value?.refreshRoutingData?.()
  } catch (err: unknown) {
    showError(parseApiError(err, t('ui.failedToDeleteModel')))
  }
}

function openCreateModelDialog() {
  editingModel.value = null
  createModelDialogOpen.value = true
}

// 处理模型对话框关闭事件
function handleModelDialogUpdate(value: boolean) {
  createModelDialogOpen.value = value
  if (!value) {
    editingModel.value = null
  }
}

// 处理模型表单提交成功
async function handleModelFormSuccess() {
  // 编辑模式关闭对话框，创建模式保持打开以便连续添加
  if (editingModel.value) {
    createModelDialogOpen.value = false
    editingModel.value = null
  }
  await loadGlobalModels()
}

async function editModel(model: GlobalModelResponse) {
  editingModel.value = model
  createModelDialogOpen.value = true
}

function handleModelPricingSynced(model: GlobalModelResponse) {
  applyGlobalModelUpdate({
    ...model,
    default_tiered_pricing: cloneTieredPricingConfig(model.default_tiered_pricing),
    config: model.config ? { ...model.config } : model.config,
  })
}

async function deleteModel(model: GlobalModelResponse) {
  const confirmed = await confirmDanger(
    `确定删除模型 "${model.name}" 吗？\n\n此操作不可撤销。`,
    t('ui.deleteModel')
  )
  if (!confirmed) return

  try {
    await deleteGlobalModel(model.id)
    success(t('ui.modelDeleted'))
    if (selectedModel.value?.id === model.id) {
      selectedModel.value = null
    }
    setBatchManageModelSelection(model.id, false)
    await loadGlobalModels()
  } catch (err: unknown) {
    showError(parseApiError(err, t('ui.deletionFailed')), t('ui.deletionFailed'))
  }
}

async function toggleModelStatus(model: GlobalModelResponse) {
  try {
    await updateGlobalModel(model.id, { is_active: !model.is_active })
    model.is_active = !model.is_active
    success(model.is_active ? t('ui.modelEnabled') : t('ui.modelDisabledda3c032'))
  } catch (err: unknown) {
    showError(parseApiError(err, t('ui.operationFailed')), t('ui.operationFailed'))
  }
}

async function refreshData() {
  await loadGlobalModels()
}

watch(globalModelsQueryParams, (newParams, oldParams) => {
  if (modelSearchDebounceTimer) clearTimeout(modelSearchDebounceTimer)
  const isSearchOnly = newParams.search !== oldParams?.search
    && newParams.skip === oldParams?.skip
    && newParams.limit === oldParams?.limit
  if (isSearchOnly) {
    modelSearchDebounceTimer = setTimeout(() => {
      void loadGlobalModels({ cacheTtlMs: GLOBAL_MODELS_LIST_CACHE_TTL_MS })
    }, 300)
  } else {
    void loadGlobalModels({ cacheTtlMs: GLOBAL_MODELS_LIST_CACHE_TTL_MS })
  }
}, { immediate: true })

onBeforeUnmount(() => {
  if (modelSearchDebounceTimer) clearTimeout(modelSearchDebounceTimer)
  globalModelsRequestId += 1
  modelSelectionRequestId += 1
  modelProvidersRequestId += 1
})
</script>

