<template>
  <Dialog
    :model-value="open"
    :title="isEditMode ? t('ui.editModel') : t('guide.nav.concepts.createModel')"
    :description="isEditMode ? t('ui.editModelConfigurationAndPricing') : ''"
    :icon="isEditMode ? SquarePen : Layers"
    :size="isEditMode ? '4xl' : '3xl'"
    :z-index="zIndex"
    @update:model-value="handleDialogUpdate"
  >
    <div
      class="flex gap-4"
      :class="isEditMode ? '' : 'h-[600px] flex-col'"
    >
      <!-- 上方：搜索和加载预设（仅创建模式） -->
      <ModelCatalogPresetPanel
        v-if="!isEditMode && !presetPanelCollapsed"
        v-model:search-query="searchQuery"
        v-model:expanded-provider="expandedProvider"
        :models="allModelsCache"
        :existing-models="existingModelsCache"
        :selected-model="selectedModel"
        :loading="loading"
        :catalog-load-error="catalogLoadError"
        @select="selectModel"
        @edit-existing="editExistingModel"
        @retry="retryModelsCatalog"
        @manual-entry="enterManualEntryMode"
      />

      <!-- 第二步：详细信息表单 -->
      <div
        v-if="isEditMode || presetPanelCollapsed"
        class="flex-1 min-h-0 overflow-y-auto scrollbar-thin"
        :class="isEditMode ? 'max-h-[70vh]' : ''"
      >
        <div
          v-if="!isEditMode"
          class="mb-4 flex items-center gap-3 rounded-lg border bg-muted/20 px-4 py-3"
        >
          <div
            v-if="selectedModel"
            class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border bg-background"
          >
            <img
              :src="getProviderLogoUrl(selectedModel.providerId)"
              :alt="selectedModel.providerName"
              class="h-7 w-7 rounded object-contain dark:invert dark:brightness-90"
              @error="hideBrokenImage"
            >
          </div>
          <div
            v-else
            class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border bg-background"
          >
            <SquarePen class="h-5 w-5 text-muted-foreground" />
          </div>
          <div class="min-w-0 flex-1">
            <div class="text-xs text-muted-foreground">
              {{ selectedModel ? t('ui.loadedTheValuePreset', { name: selectedModel.providerName }) : t('ui.manualEntry') }}
            </div>
            <div class="truncate text-sm font-medium">
              {{ selectedModel ? selectedModel.modelName : t('ui.enterModelDetails') }}
            </div>
            <div
              v-if="selectedModel"
              class="truncate text-xs text-muted-foreground"
            >
              {{ selectedModel.modelId }}
            </div>
          </div>
          <div class="flex shrink-0 items-center gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              class="shrink-0"
              @click="reopenPresetPanel"
            >
              {{ t('ui.backToModelSelection') }}
            </Button>
          </div>
        </div>
        <form
          class="space-y-5"
          @submit.prevent="handleSubmit"
        >
          <!-- 基本信息 -->
          <section
            ref="basicInfoSection"
            class="space-y-3 rounded-lg border bg-card p-4"
          >
            <h4 class="font-medium text-sm">
              {{ t('ui.basicInformation') }}
            </h4>
            <div class="grid grid-cols-2 gap-3">
              <div class="space-y-1.5">
                <Label
                  for="model-display-name"
                  class="text-xs"
                >{{ t('ui.name') }}</Label>
                <Input
                  id="model-display-name"
                  v-model="form.display_name"
                  placeholder="Claude 3.5 Sonnet"
                  required
                />
              </div>
              <div class="space-y-1.5">
                <Label
                  for="model-name"
                  class="text-xs"
                >{{ t('ui.modelId5f9dbab') }}</Label>
                <Input
                  id="model-name"
                  v-model="form.name"
                  placeholder="claude-3-5-sonnet-20241022"
                  :disabled="isEditMode"
                  required
                />
              </div>
            </div>
            <div class="space-y-1.5">
              <Label
                for="model-description"
                class="text-xs"
              >{{ t('ui.descriptiondc2ba46') }}</Label>
              <Input
                id="model-description"
                :model-value="getConfigInputValue('description')"
                :placeholder="t('ui.brieflyDescribeTheModel')"
                @update:model-value="(v) => setConfigField('description', v || undefined)"
              />
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div class="space-y-1.5">
                <Label
                  for="model-output-limit"
                  class="text-xs"
                >{{ t('ui.maximumOutputTokens') }}</Label>
                <Input
                  id="model-output-limit"
                  :model-value="getConfigInputValue('output_limit')"
                  type="number"
                  min="1"
                  :placeholder="t('ui.eG8192')"
                  @update:model-value="(v) => setConfigField('output_limit', parseNumberInput(v, { allowFloat: false }))"
                />
              </div>
              <div class="space-y-1.5">
                <Label
                  for="model-context-limit"
                  class="text-xs"
                >{{ t('ui.contextWindow') }}</Label>
                <Input
                  id="model-context-limit"
                  :model-value="getConfigInputValue('context_limit')"
                  type="number"
                  min="1"
                  :placeholder="t('ui.eG200000')"
                  @update:model-value="(v) => setConfigField('context_limit', parseNumberInput(v, { allowFloat: false }))"
                />
              </div>
            </div>
            <div class="rounded-lg border border-border/60 bg-muted/20 p-3 space-y-2">
              <div class="flex items-start gap-2">
                <Checkbox
                  :model-value="isEmbeddingEnabled"
                  class="mt-0.5"
                  @update:model-value="setEmbeddingEnabled"
                />
                <div class="space-y-1">
                  <div class="text-sm font-medium">
                    Embedding
                  </div>
                  <p class="text-xs text-muted-foreground">
                    {{ t('ui.markAsAnEmbeddingModelUsingADedicatedEmbeddingApi') }}
                  </p>
                  <div
                    v-if="isEmbeddingEnabled"
                    class="flex flex-wrap gap-1.5"
                  >
                    <span
                      v-for="format in embeddingApiFormats"
                      :key="format"
                      class="rounded-md border border-border/60 bg-background px-2 py-0.5 text-[11px] font-mono text-muted-foreground"
                    >{{ format }}</span>
                  </div>
                </div>
              </div>
            </div>
          </section>

          <!-- 价格配置 -->
          <section
            class="space-y-3 rounded-lg border bg-card p-4"
          >
            <div class="flex items-center justify-between gap-2">
              <h4 class="font-medium text-sm">
                {{ t('ui.selectBillingMode') }}
              </h4>
              <Popover
                v-if="isEditMode"
                :open="onlinePricingSourcePopoverOpen"
                :modal="false"
                @update:open="handleOnlinePricingSourcePopoverUpdate"
              >
                <PopoverTrigger as-child>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    class="h-8 min-w-0 max-w-56 shrink-0 gap-1.5 px-2.5"
                    :disabled="syncingOnlinePricing || submitting"
                    :title="syncingOnlinePricing
                      ? t('models.pricingSource.syncingTitle')
                      : currentOnlinePricingSource
                        ? t('models.pricingSource.editCurrentTitle', { provider: currentOnlinePricingSource.provider_name })
                        : t('models.pricingSource.editChooseTitle')"
                    :aria-label="t('ui.syncLatestOnlinePricing')"
                    data-testid="sync-online-pricing"
                    @click="syncOnlinePricing"
                  >
                    <RefreshCw
                      class="h-4 w-4"
                      :class="syncingOnlinePricing ? 'animate-spin' : ''"
                    />
                    <span class="truncate text-xs">
                      {{ currentOnlinePricingSource
                        ? t('models.pricingSource.buttonCurrent', { provider: currentOnlinePricingSource.provider_name })
                        : t('models.pricingSource.choose') }}
                    </span>
                  </Button>
                </PopoverTrigger>
                <PopoverContent
                  class="z-[130] w-80 max-w-[calc(100vw-2rem)] p-3"
                  side="bottom"
                  align="end"
                >
                  <div class="space-y-3">
                    <div class="flex items-center justify-between gap-3">
                      <div class="min-w-0">
                        <div class="truncate text-xs font-medium">
                          {{ t('models.management.batch.providerPlaceholder') }}
                        </div>
                        <div class="truncate text-[10px] text-muted-foreground">
                          {{ props.model?.display_name || props.model?.name || t('ui.currentModel') }}
                        </div>
                      </div>
                      <Loader2
                        v-if="syncingOnlinePricing"
                        class="h-3.5 w-3.5 shrink-0 animate-spin text-muted-foreground"
                      />
                    </div>

                    <div
                      v-if="syncingOnlinePricing && onlinePricingCandidates.length === 0"
                      class="flex items-center justify-center py-5 text-xs text-muted-foreground"
                    >
                      {{ t('ui.refreshingOnlinePricing') }}
                    </div>
                    <div
                      v-else
                      class="max-h-64 space-y-1.5 overflow-y-auto pr-0.5"
                      role="radiogroup"
                      :aria-label="t('ui.onlinePricingSource')"
                      @keydown="handleOnlinePricingSourcePopoverKeydown"
                    >
                      <button
                        v-for="candidate in onlinePricingCandidates"
                        :key="candidate.providerId"
                        type="button"
                        role="radio"
                        class="flex w-full items-center gap-2 rounded-md border px-2.5 py-2 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40 disabled:cursor-not-allowed disabled:opacity-55"
                        :class="selectedOnlinePricingProviderId === candidate.providerId
                          ? 'border-primary bg-primary/5 ring-1 ring-primary'
                          : 'border-border/70 hover:border-primary/35 hover:bg-muted/40'"
                        :aria-checked="selectedOnlinePricingProviderId === candidate.providerId"
                        :disabled="syncingOnlinePricing || !isOnlinePricingCandidateSyncable(candidate)"
                        :tabindex="selectedOnlinePricingProviderId === candidate.providerId
                          || (!selectedOnlinePricingProviderId
                            && firstSyncableOnlinePricingProviderId === candidate.providerId)
                          ? 0
                          : -1"
                        data-online-pricing-source-control
                        data-online-pricing-source-option
                        :data-provider-id="candidate.providerId"
                        :data-testid="`online-pricing-source-${candidate.providerId}`"
                        @click="selectedOnlinePricingProviderId = candidate.providerId"
                      >
                        <span class="flex h-7 w-7 shrink-0 items-center justify-center rounded border bg-background">
                          <img
                            :src="getProviderLogoUrl(candidate.providerId)"
                            :alt="candidate.providerName"
                            class="h-5 w-5 rounded object-contain dark:invert dark:brightness-90"
                            @error="hideBrokenImage"
                          >
                        </span>
                        <span class="min-w-0 flex-1">
                          <span class="block truncate text-xs font-medium">{{ candidate.providerName }}</span>
                          <span class="block truncate font-mono text-[9px] text-muted-foreground">{{ candidate.providerId }}</span>
                          <span
                            v-if="getOnlinePricingCandidateUnavailableReason(candidate)"
                            class="block truncate text-[10px] text-rose-600 dark:text-rose-400"
                          >{{ getOnlinePricingCandidateUnavailableReason(candidate) }}</span>
                        </span>
                        <span
                          v-if="isOnlinePricingCandidateSyncable(candidate)"
                          class="shrink-0 text-right text-[9px] tabular-nums text-muted-foreground"
                        >
                          <span class="block">{{ t('ui.input4b77c7d') }}{{ formatModelPrice(candidate.inputPrice) }}/M</span>
                          <span class="block">{{ t('ui.output') }}{{ formatModelPrice(candidate.outputPrice) }}/M</span>
                        </span>
                        <span
                          class="flex h-4 w-4 shrink-0 items-center justify-center rounded-full border"
                          :class="selectedOnlinePricingProviderId === candidate.providerId
                            ? 'border-primary bg-primary text-primary-foreground'
                            : 'border-border'"
                        >
                          <Check
                            v-if="selectedOnlinePricingProviderId === candidate.providerId"
                            class="h-2.5 w-2.5"
                          />
                        </span>
                      </button>
                      <div
                        v-if="onlinePricingCandidates.length === 0"
                        class="py-4 text-center text-xs text-muted-foreground"
                      >
                        {{ t('ui.noOnlinePricingSourcesAvailable') }}
                      </div>
                    </div>

                    <div class="flex justify-end gap-2 border-t border-border/60 pt-2">
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        :disabled="syncingOnlinePricing"
                        data-online-pricing-source-control
                        data-testid="online-pricing-source-cancel"
                        @keydown="handleOnlinePricingSourcePopoverKeydown"
                        @click="closeOnlinePricingSourcePopover"
                      >
                        {{ t('common.cancel') }}
                      </Button>
                      <Button
                        type="button"
                        size="sm"
                        :disabled="!selectedOnlinePricingCandidate
                          || !isOnlinePricingCandidateSyncable(selectedOnlinePricingCandidate)
                          || syncingOnlinePricing"
                        data-online-pricing-source-control
                        @keydown="handleOnlinePricingSourcePopoverKeydown"
                        @click="confirmOnlinePricingSource"
                      >
                        <Loader2
                          v-if="syncingOnlinePricing"
                          class="mr-1.5 h-3.5 w-3.5 animate-spin"
                        />
                        {{ syncingOnlinePricing ? t('models.management.batch.syncing') : t('ui.syncThisSource') }}
                      </Button>
                    </div>
                  </div>
                </PopoverContent>
              </Popover>
            </div>
            <Tabs
              v-model="billingMode"
              @update:model-value="handleBillingModeChange"
            >
              <TabsList class="grid w-full grid-cols-4">
                <TabsTrigger value="token">
                  Token
                </TabsTrigger>
                <TabsTrigger value="request">
                  {{ t('ui.perRequest') }}
                </TabsTrigger>
                <TabsTrigger value="image">
                  {{ t('ui.image') }}
                </TabsTrigger>
                <TabsTrigger value="video">
                  {{ t('ui.video') }}
                </TabsTrigger>
              </TabsList>

              <TieredPricingEditor
                v-show="billingMode === 'token' || billingMode === 'image'"
                ref="tieredPricingEditorRef"
                v-model="tieredPricing"
                class="mt-3"
                :auto-fill-missing-cache-prices="autoFillMissingCachePrices"
                :show-token-pricing="billingMode === 'token'"
                :show-image-pricing="isImageGenerationEnabled"
                :show-image-editor="billingMode === 'image'"
                :show-processing-tier-controls="false"
                :show-processing-tier-multiplier-controls="true"
              />

              <TabsContent
                value="request"
                class="pt-2"
              >
                <div class="rounded-lg border bg-muted/20 p-4 space-y-2">
                  <Label class="text-xs">{{ t('ui.pricePerRequestUsd') }}</Label>
                  <Input
                    :model-value="form.default_price_per_request ?? ''"
                    type="number"
                    step="0.001"
                    min="0"
                    class="max-w-48"
                    :placeholder="t('ui.eG001')"
                    @update:model-value="(v) => form.default_price_per_request = parseNumberInput(v, { allowFloat: true })"
                  />
                  <p class="text-xs text-muted-foreground">
                    {{ t('ui.chargeAFixedAmountPerApiRequestOptionallyCombinedWith') }}
                  </p>
                </div>
              </TabsContent>

              <TabsContent
                value="video"
                class="pt-2"
              >
                <div class="space-y-3 rounded-lg border bg-muted/20 p-4">
                  <div>
                    <div class="text-sm font-medium">
                      {{ t('ui.videoPricingResolutionDuration') }}
                    </div>
                    <p class="mt-1 text-xs text-muted-foreground">
                      {{ t('ui.setVideoPricingPerSecondByOutputResolution') }}
                    </p>
                  </div>

                  <div class="flex items-center gap-1.5 flex-wrap">
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      class="h-7 text-xs"
                      @click="fillVideoResolutionPricePreset('common')"
                    >
                      {{ t('ui.general') }}
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      class="h-7 text-xs"
                      @click="fillVideoResolutionPricePreset('sora')"
                    >
                      Sora
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      class="h-7 text-xs"
                      @click="fillVideoResolutionPricePreset('veo')"
                    >
                      Veo
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      class="h-7 text-xs"
                      @click="addVideoResolutionPriceRow"
                    >
                      <Plus class="w-3.5 h-3.5 mr-0.5" /> {{ t('ui.custom4eafa9e') }}
                    </Button>
                  </div>

                  <div
                    v-if="videoResolutionPrices.length > 0"
                    class="rounded-lg border border-border overflow-hidden"
                  >
                    <div class="grid grid-cols-[1fr_1fr_32px] gap-0 text-xs text-muted-foreground bg-muted/50 px-3 py-1.5 border-b border-border">
                      <span>{{ t('ui.resolution') }}</span>
                      <span>{{ t('ui.unitPriceSecond') }}</span>
                      <span />
                    </div>
                    <div class="divide-y divide-border">
                      <div
                        v-for="(row, idx) in videoResolutionPrices"
                        :key="idx"
                        class="grid grid-cols-[1fr_1fr_32px] gap-2 items-center px-3 py-1.5"
                      >
                        <Input
                          v-model="row.resolution"
                          class="h-7 text-sm"
                          :placeholder="t('ui.eG720p')"
                        />
                        <Input
                          :model-value="row.price_per_second ?? ''"
                          type="number"
                          step="0.0001"
                          min="0"
                          class="h-7 text-sm"
                          placeholder="0"
                          @update:model-value="(v) => row.price_per_second = parseNumberInput(v, { allowFloat: true })"
                        />
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          class="h-7 w-7"
                          :title="t('ui.delete')"
                          @click="removeVideoResolutionPriceRow(idx)"
                        >
                          <Trash2 class="w-3.5 h-3.5" />
                        </Button>
                      </div>
                    </div>
                  </div>
                  <div
                    v-else
                    class="rounded-lg border border-dashed py-8 text-center text-xs text-muted-foreground"
                  >
                    {{ t('ui.selectAPricingPresetOrAddACustomResolution') }}
                  </div>
                </div>
              </TabsContent>
            </Tabs>
          </section>
        </form>
      </div>
    </div>

    <template #footer>
      <Button
        type="button"
        variant="outline"
        @click="handleCancel"
      >
        {{ t('common.cancel') }}
      </Button>
      <Button
        v-if="!isEditMode && !presetPanelCollapsed"
        type="button"
        variant="outline"
        @click="enterManualEntryMode"
      >
        {{ t('ui.enterManually') }}
      </Button>
      <Button
        v-if="isEditMode || presetPanelCollapsed"
        :disabled="submitting || syncingOnlinePricing || !form.name || !form.display_name"
        @click="handleSubmit"
      >
        <Loader2
          v-if="submitting"
          class="w-4 h-4 mr-2 animate-spin"
        />
        {{ isEditMode ? t('ui.save') : t('ui.add') }}
      </Button>
      <Button
        v-if="selectedModel && !isEditMode && presetPanelCollapsed"
        type="button"
        variant="ghost"
        @click="clearSelection"
      >
        {{ t('ui.clear1ef3de0') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue'
import {
  Loader2,
  Layers,
  SquarePen,
  Plus,
  Trash2,
  Check,
  RefreshCw,
} from 'lucide-vue-next'
import {
  Dialog, Button, Input, Label, Checkbox,
  Tabs, TabsContent, TabsList, TabsTrigger,
  Popover, PopoverTrigger, PopoverContent,
} from '@/components/ui'
import { useToast } from '@/composables/useToast'
import { useFormDialog } from '@/composables/useFormDialog'
import { useI18n } from '@/i18n'
import { parseNumberInput, sortResolutionEntries } from '@/utils/form'
import { log } from '@/utils/logger'
import { parseApiError } from '@/utils/errorParser'
import TieredPricingEditor from './TieredPricingEditor.vue'
import ModelCatalogPresetPanel from './ModelCatalogPresetPanel.vue'
import { formatModelPrice, hideBrokenImage } from '../utils/modelsDevCatalog'
import {
  getModelsDevList,
  getProviderLogoUrl,
  refreshModelsDevList,
  type ModelsDevModelItem,
} from '@/api/models-dev'
import {
  createGlobalModel,
  listGlobalModels,
  updateGlobalModel,
  type GlobalModelResponse,
} from '@/api/global-models'
import type { TieredPricingConfig } from '@/api/endpoints/types'
import {
  EMBEDDING_API_FORMATS,
  buildGlobalModelCreatePayload,
  buildGlobalModelUpdatePayload,
  cloneTieredPricingConfig,
  findGlobalModelByName,
  tieredPricingConfigsEqual,
} from './global-model-form-helpers'
import { tieredPricingHasImageOutputPricing } from '../utils/tiered-pricing'
import {
  getModelsDevPricingSourceFromConfig,
  modelsDevPricingSourcesEqual,
  useModelsDevPricingSources,
  withModelsDevPricingSource,
} from '../composables/useModelsDevPricingSources'

const props = defineProps<{
  open: boolean
  model?: GlobalModelResponse | null
  // 嵌套在其他弹窗内部时使用更高的层级，避免被父弹窗遮挡
  zIndex?: number
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'success': []
  'editModel': [model: GlobalModelResponse]
  'pricingSynced': [model: GlobalModelResponse]
}>()

const { success, error: showError } = useToast()
const { t } = useI18n()
const { getSource, setSource } = useModelsDevPricingSources()
const submitting = ref(false)
const syncingOnlinePricing = ref(false)
const tieredPricingEditorRef = ref<InstanceType<typeof TieredPricingEditor> | null>(null)
const basicInfoSection = ref<HTMLElement | null>(null)

// 模型列表相关
const loading = ref(false)
const catalogLoadError = ref<string | null>(null)
const searchQuery = ref('')
const expandedProvider = ref<string | null>(null)
const allModelsCache = ref<ModelsDevModelItem[]>([]) // 全部模型（缓存）
const existingModelsCache = ref<GlobalModelResponse[]>([])
const selectedModel = ref<ModelsDevModelItem | null>(null)
const presetPanelCollapsed = ref(false)
const billingMode = ref('token')
const editingOnlinePricingSource = ref<{
  model_id: string
  provider_id: string
  provider_name: string
} | null>(null)
const onlinePricingSourcePopoverOpen = ref(false)
const onlinePricingCandidates = ref<ModelsDevModelItem[]>([])
const selectedOnlinePricingProviderId = ref('')

const selectedOnlinePricingCandidate = computed(() => (
  onlinePricingCandidates.value.find(candidate => (
    candidate.providerId === selectedOnlinePricingProviderId.value
  )) ?? null
))
const currentOnlinePricingSource = computed(() => (
  props.model ? getSource(props.model.id, props.model.config) : null
))
const firstSyncableOnlinePricingProviderId = computed(() => (
  onlinePricingCandidates.value.find(isOnlinePricingCandidateSyncable)?.providerId ?? ''
))

watch([onlinePricingSourcePopoverOpen, syncingOnlinePricing], ([open, syncing]) => {
  if (!open || syncing) return
  void nextTick().then(() => {
    if (onlinePricingSourcePopoverOpen.value && !syncingOnlinePricing.value) {
      focusOnlinePricingSourcePopover()
    }
  })
}, { flush: 'post' })

function getExistingModel(model: ModelsDevModelItem): GlobalModelResponse | undefined {
  return findGlobalModelByName(existingModelsCache.value, model.modelId)
}

function handleBillingModeChange(mode: string) {
  billingMode.value = mode
  if (mode === 'image' && !isImageGenerationEnabled.value) {
    setImageGenerationEnabled(true)
  }
}

function scrollToBasicInformation() {
  nextTick(() => {
    basicInfoSection.value?.scrollIntoView({
      behavior: 'smooth',
      block: 'nearest',
    })
  })
}

function enterManualEntryMode() {
  clearSelection()
  presetPanelCollapsed.value = true
  scrollToBasicInformation()
}

function reopenPresetPanel() {
  clearSelection()
  presetPanelCollapsed.value = false
}

// 阶梯计费配置
const tieredPricing = ref<TieredPricingConfig | null>(null)

type VideoResolutionPriceRow = { resolution: string; price_per_second: number | undefined }

const videoResolutionPrices = ref<VideoResolutionPriceRow[]>([])

const VIDEO_RESOLUTION_PRICE_PRESETS: Record<
  'common' | 'sora' | 'veo',
  VideoResolutionPriceRow[]
> = {
  common: [
    { resolution: '480p', price_per_second: 0 },
    { resolution: '720p', price_per_second: 0 },
    { resolution: '1080p', price_per_second: 0 },
    { resolution: '4k', price_per_second: 0 },
  ],
  sora: [
    { resolution: '720x1080', price_per_second: 0 },
    { resolution: '1024x1792', price_per_second: 0 },
  ],
  veo: [
    { resolution: '720p', price_per_second: 0 },
    { resolution: '1080p', price_per_second: 0 },
    { resolution: '4k', price_per_second: 0 },
  ],
}

const embeddingApiFormats = [...EMBEDDING_API_FORMATS]

interface FormData {
  name: string
  display_name: string
  default_price_per_request?: number
  supported_capabilities?: string[]
  config?: Record<string, unknown>
  is_active?: boolean
}

const defaultForm = (): FormData => ({
  name: '',
  display_name: '',
  default_price_per_request: undefined,
  supported_capabilities: [],
  config: { streaming: true },
  is_active: true,
})

const form = ref<FormData>(defaultForm())
const imageGenerationExplicitOverride = ref<boolean | null>(null)

const isEmbeddingEnabled = computed(() => {
  return form.value.supported_capabilities?.includes('embedding') === true
    || form.value.config?.embedding === true
    || form.value.config?.model_type === 'embedding'
})

const isImageGenerationEnabled = computed(() => {
  if (imageGenerationExplicitOverride.value !== null) {
    return imageGenerationExplicitOverride.value
  }
  return form.value.supported_capabilities?.includes('image_generation') === true
    || form.value.config?.image_generation === true
    || form.value.config?.model_type === 'image'
    || (Array.isArray(form.value.config?.api_formats)
      && form.value.config.api_formats.some((format) => String(format).endsWith(':image')))
    || tieredPricingHasImageOutputPricing(tieredPricing.value)
})

const KEEP_FALSE_CONFIG_KEYS = new Set(['streaming'])

// 设置 config 字段
function setConfigField(key: string, value: unknown) {
  if (!form.value.config) {
    form.value.config = {}
  }
  if (value === undefined || value === '' || (value === false && !KEEP_FALSE_CONFIG_KEYS.has(key))) {
    delete form.value.config[key]
  } else {
    form.value.config[key] = value
  }
}

function getConfigInputValue(key: string): string | number {
  const value = form.value.config?.[key]
  return typeof value === 'string' || typeof value === 'number' ? value : ''
}

function setEmbeddingEnabled(enabled: boolean) {
  const caps = new Set(form.value.supported_capabilities || [])
  if (enabled) {
    caps.add('embedding')
    setConfigField('embedding', true)
    setConfigField('model_type', 'embedding')
    setConfigField('streaming', false)
    form.value.config = {
      ...(form.value.config || {}),
      api_formats: [...embeddingApiFormats],
    }
  } else {
    caps.delete('embedding')
    setConfigField('embedding', undefined)
    if (form.value.config?.model_type === 'embedding') setConfigField('model_type', undefined)
    if (Array.isArray(form.value.config?.api_formats)
      && form.value.config.api_formats.every((format) => embeddingApiFormats.some((supportedFormat) => supportedFormat === String(format)))) {
      setConfigField('api_formats', undefined)
    }
  }
  form.value.supported_capabilities = [...caps]
}

function setImageGenerationEnabled(value: boolean | 'indeterminate') {
  const enabled = value === true
  imageGenerationExplicitOverride.value = enabled
  const caps = new Set(form.value.supported_capabilities || [])
  if (enabled) {
    caps.add('image_generation')
    setConfigField('image_generation', true)
  } else {
    caps.delete('image_generation')
    setConfigField('image_generation', undefined)
    if (form.value.config?.model_type === 'image') setConfigField('model_type', undefined)
  }
  form.value.supported_capabilities = [...caps]
}

function getNested(obj: unknown, path: string): unknown {
  if (!obj || typeof obj !== 'object') return undefined
  const parts = path.split('.').filter(Boolean)
  let cur = obj as Record<string, unknown>
  for (const p of parts) {
    if (!cur || typeof cur !== 'object') return undefined
    cur = cur[p] as Record<string, unknown>
  }
  return cur
}

function setNested(obj: unknown, path: string, value: unknown) {
  if (!obj || typeof obj !== 'object') return
  const parts = path.split('.').filter(Boolean)
  if (parts.length === 0) return
  let cur = obj as Record<string, unknown>
  for (let i = 0; i < parts.length - 1; i++) {
    const p = parts[i]
    if (!cur[p] || typeof cur[p] !== 'object') {
      cur[p] = {}
    }
    cur = cur[p] as Record<string, unknown>
  }
  cur[parts[parts.length - 1]] = value
}

function deleteNested(obj: unknown, path: string) {
  if (!obj || typeof obj !== 'object') return
  const parts = path.split('.').filter(Boolean)
  if (parts.length === 0) return
  let cur = obj as Record<string, unknown>
  for (let i = 0; i < parts.length - 1; i++) {
    const p = parts[i]
    if (!cur[p] || typeof cur[p] !== 'object') return
    cur = cur[p] as Record<string, unknown>
  }
  delete cur[parts[parts.length - 1]]
}

function pruneEmptyBillingConfig() {
  const cfg = form.value.config
  if (!cfg || typeof cfg !== 'object') return
  const billing = cfg.billing as Record<string, unknown> | undefined
  if (!billing || typeof billing !== 'object') return
  const video = billing.video as Record<string, unknown> | undefined
  if (video && typeof video === 'object' && Object.keys(video).length === 0) {
    delete billing.video
  }
  if (Object.keys(billing).length === 0) {
    delete cfg.billing
  }
}

/**
 * Normalize resolution key:
 * - lowercase, remove spaces, × → x
 * - For WxH format, sort dimensions so smaller comes first (720x1080 = 1080x720)
 */
function normalizeResolutionKey(raw: string): string {
  let k = (raw || '').trim().toLowerCase().replace(/\s+/g, '').replace(/×/g, 'x')
  // Check if it's WxH format (e.g., 1080x720)
  const match = k.match(/^(\d+)x(\d+)$/)
  if (match) {
    const a = parseInt(match[1], 10)
    const b = parseInt(match[2], 10)
    // Sort: smaller dimension first
    k = a <= b ? `${a}x${b}` : `${b}x${a}`
  }
  return k
}

function loadVideoPricingFromConfig() {
  const cfg = form.value.config || {}
  const raw = getNested(cfg, 'billing.video.price_per_second_by_resolution')
  if (raw && typeof raw === 'object' && !Array.isArray(raw)) {
    // 按分辨率从低到高排序
    const sortedEntries = sortResolutionEntries(Object.entries(raw as Record<string, unknown>))
    videoResolutionPrices.value = sortedEntries.map(([k, v]) => ({
      resolution: String(k),
      price_per_second: typeof v === 'number' ? v : undefined,
    }))
  } else {
    videoResolutionPrices.value = []
  }
}

function applyVideoPricingToConfig() {
  if (!form.value.config) {
    form.value.config = {}
  }
  const cfg = form.value.config

  // Clean legacy keys
  deleteNested(cfg, 'billing.video.price_per_second')
  deleteNested(cfg, 'billing.video.resolution_multipliers')

  // resolution/size prices (normalized: 1080x720 → 720x1080)
  const map: Record<string, number> = {}
  for (const row of videoResolutionPrices.value) {
    const k = normalizeResolutionKey(row.resolution || '')
    const v = row.price_per_second
    if (!k) continue
    if (typeof v !== 'number' || Number.isNaN(v)) continue
    map[k] = v
  }
  if (Object.keys(map).length > 0) {
    setNested(cfg, 'billing.video.price_per_second_by_resolution', map)
  } else {
    deleteNested(cfg, 'billing.video.price_per_second_by_resolution')
  }

  pruneEmptyBillingConfig()
}

function addVideoResolutionPriceRow() {
  videoResolutionPrices.value.push({ resolution: '', price_per_second: undefined })
}

function removeVideoResolutionPriceRow(idx: number) {
  videoResolutionPrices.value.splice(idx, 1)
}

function fillVideoResolutionPricePreset(preset: 'common' | 'sora' | 'veo') {
  videoResolutionPrices.value = VIDEO_RESOLUTION_PRICE_PRESETS[preset].map(r => ({
    resolution: r.resolution,
    price_per_second: r.price_per_second,
  }))
}

async function loadExistingModels() {
  const models: GlobalModelResponse[] = []
  let total = 0
  do {
    const response = await listGlobalModels({ skip: models.length, limit: 1000 })
    models.push(...response.models)
    total = response.total
    if (response.models.length === 0) break
  } while (models.length < total)
  existingModelsCache.value = models
}

async function loadModelsCatalog() {
  catalogLoadError.value = null
  try {
    allModelsCache.value = await getModelsDevList(false)
  } catch (err: unknown) {
    allModelsCache.value = []
    catalogLoadError.value = parseApiError(err, t('ui.theExternalModelCatalogIsTemporarilyUnavailable'))
    log.error('Failed to load online models:', err)
  }
}

async function retryModelsCatalog() {
  if (loading.value) return
  loading.value = true
  try {
    await loadModelsCatalog()
  } finally {
    loading.value = false
  }
}

// 加载在线目录和已有模型列表
async function loadModels() {
  loading.value = true
  const shouldLoadCatalog = allModelsCache.value.length === 0
  if (!shouldLoadCatalog) catalogLoadError.value = null
  try {
    await Promise.all([
      shouldLoadCatalog ? loadModelsCatalog() : Promise.resolve(),
      loadExistingModels()
        .catch(err => log.error('Failed to load existing models:', err)),
    ])
  } finally {
    loading.value = false
  }
}

// 打开对话框时加载数据
watch(() => props.open, async (isOpen) => {
  if (!isOpen) {
    editingOnlinePricingSource.value = null
    resetOnlinePricingSourceSelection()
    return
  }
  if (isOpen && !props.model) {
    await loadModels()
  }
})

// 选择模型并填充表单
function selectModel(model: ModelsDevModelItem) {
  if (getExistingModel(model)) return

  imageGenerationExplicitOverride.value = null
  selectedModel.value = model
  expandedProvider.value = model.providerId

  // 构建 config
  const config: Record<string, unknown> = {
    streaming: model.supportsEmbedding ? false : true,
  }
  if (model.supportsVision) config.vision = true
  if (model.supportsToolCall) config.function_calling = true
  if (model.supportsReasoning) config.extended_thinking = true
  if (model.supportsStructuredOutput) config.structured_output = true
  if (model.supportsTemperature !== false) config.temperature = model.supportsTemperature
  if (model.supportsAttachment) config.attachment = true
  if (model.openWeights) config.open_weights = true
  if (model.contextLimit) config.context_limit = model.contextLimit
  if (model.outputLimit) config.output_limit = model.outputLimit
  if (model.knowledgeCutoff) config.knowledge_cutoff = model.knowledgeCutoff
  if (model.family) config.family = model.family
  if (model.releaseDate) config.release_date = model.releaseDate
  if (model.inputModalities?.length) config.input_modalities = model.inputModalities
  if (model.outputModalities?.length) config.output_modalities = model.outputModalities
  const supportedCapabilities = new Set<string>()
  if (model.supportsEmbedding) supportedCapabilities.add('embedding')
  if (model.outputModalities?.includes('image')) supportedCapabilities.add('image_generation')
  form.value = {
    ...defaultForm(),
    name: model.modelId,
    display_name: model.modelName,
    config,
    supported_capabilities: [...supportedCapabilities],
  }
  if (model.supportsEmbedding) {
    setEmbeddingEnabled(true)
  }
  if (model.outputModalities?.includes('image')) {
    billingMode.value = 'image'
  } else if (model.outputModalities?.includes('video')) {
    billingMode.value = 'video'
  } else {
    billingMode.value = 'token'
  }
  loadVideoPricingFromConfig()

  tieredPricing.value = model.tieredPricing
    ? cloneTieredPricingConfig(model.tieredPricing)
    : null

  presetPanelCollapsed.value = true
  scrollToBasicInformation()
}

function editExistingModel(model: ModelsDevModelItem, existingModel: GlobalModelResponse) {
  editingOnlinePricingSource.value = {
    model_id: model.modelId,
    provider_id: model.providerId,
    provider_name: model.providerName,
  }
  emit('editModel', existingModel)
}

function normalizeModelId(value: string): string {
  return value.trim().toLowerCase()
}

function formatUnsupportedPricingFields(
  fields: ModelsDevModelItem['pricingUnsupportedFields'],
): string {
  const labels = {
    get reasoning() { return t('models.pricingField.reasoning') },
    get input_audio() { return t('models.pricingField.inputAudio') },
    get output_audio() { return t('models.pricingField.outputAudio') },
  }
  return (fields ?? []).map(field => labels[field]).join('、')
}

function getOnlinePricingCandidates(
  models: ModelsDevModelItem[],
  model: GlobalModelResponse,
): ModelsDevModelItem[] {
  const modelId = normalizeModelId(model.name)
  return models.filter(item => normalizeModelId(item.modelId) === modelId)
}

function resolveOnlinePricingModel(
  candidates: ModelsDevModelItem[],
  model: GlobalModelResponse,
): ModelsDevModelItem | null {
  const modelId = normalizeModelId(model.name)
  const transientSource = editingOnlinePricingSource.value
  const storedSource = getSource(model.id, model.config)
  const preferredProviderId = transientSource?.model_id && normalizeModelId(transientSource.model_id) === modelId
    ? transientSource.provider_id
    : storedSource?.provider_id

  if (preferredProviderId) {
    const preferred = candidates.find(item => (
      item.providerId.trim().toLowerCase() === preferredProviderId.trim().toLowerCase()
    ))
    if (preferred && isOnlinePricingCandidateSyncable(preferred)) return preferred
  }
  if (candidates.length === 1) return candidates[0]
  if (candidates.length === 0) {
    throw new Error(t('ui.noOnlinePricingFoundForThisModelOnModelsDev'))
  }
  return null
}

function isOnlinePricingCandidateSyncable(model: ModelsDevModelItem): boolean {
  return !model.pricingUnsupportedFields?.length && !!model.tieredPricing?.tiers?.length
}

function getOnlinePricingCandidateUnavailableReason(model: ModelsDevModelItem): string {
  if (model.pricingUnsupportedFields?.length) {
    return `不支持：${formatUnsupportedPricingFields(model.pricingUnsupportedFields)}`
  }
  if (!model.tieredPricing?.tiers?.length) return t('ui.theOnlineCatalogDoesNotProvideTokenPricing')
  return ''
}

function resetOnlinePricingSourceSelection() {
  onlinePricingSourcePopoverOpen.value = false
  onlinePricingCandidates.value = []
  selectedOnlinePricingProviderId.value = ''
}

function getOnlinePricingSourceControls(): HTMLElement[] {
  return [...document.querySelectorAll<HTMLElement>(
    '[data-online-pricing-source-control]:not([disabled]):not([tabindex="-1"])',
  )]
}

function focusOnlinePricingSourcePopover() {
  getOnlinePricingSourceControls()[0]?.focus()
}

function handleOnlinePricingSourcePopoverKeydown(event: KeyboardEvent) {
  const target = event.target instanceof HTMLElement
    ? event.target.closest<HTMLElement>('[data-online-pricing-source-control]')
    : null
  if (!target) return

  if (event.key === 'Tab') {
    const controls = getOnlinePricingSourceControls()
    const currentIndex = controls.indexOf(target)
    if (currentIndex === -1 || controls.length === 0) return
    if (event.shiftKey && currentIndex === 0) {
      event.preventDefault()
      controls[controls.length - 1]?.focus()
    } else if (!event.shiftKey && currentIndex === controls.length - 1) {
      event.preventDefault()
      controls[0]?.focus()
    }
    return
  }

  if (!['ArrowDown', 'ArrowRight', 'ArrowUp', 'ArrowLeft', 'Home', 'End'].includes(event.key)) {
    return
  }
  const options = [...document.querySelectorAll<HTMLButtonElement>(
    '[data-online-pricing-source-option]:not([disabled])',
  )]
  const currentIndex = options.indexOf(target as HTMLButtonElement)
  if (currentIndex === -1 || options.length === 0) return

  event.preventDefault()
  const nextIndex = event.key === 'Home'
    ? 0
    : event.key === 'End'
      ? options.length - 1
      : ['ArrowDown', 'ArrowRight'].includes(event.key)
        ? (currentIndex + 1) % options.length
        : (currentIndex - 1 + options.length) % options.length
  const nextOption = options[nextIndex]
  const providerId = nextOption?.dataset.providerId
  if (!nextOption || !providerId) return
  selectedOnlinePricingProviderId.value = providerId
  nextTick(() => nextOption.focus())
}

function handleOnlinePricingSourcePopoverUpdate(open: boolean) {
  if (open) {
    // The trigger starts the async refresh. Only show the popover once the
    // refresh is in progress or source candidates are available.
    if (syncingOnlinePricing.value || onlinePricingCandidates.value.length > 0) {
      onlinePricingSourcePopoverOpen.value = true
    }
    return
  }
  if (!syncingOnlinePricing.value) resetOnlinePricingSourceSelection()
}

function closeOnlinePricingSourcePopover() {
  if (syncingOnlinePricing.value) return
  resetOnlinePricingSourceSelection()
}

async function applyOnlinePricingModel(onlineModel: ModelsDevModelItem) {
  if (!props.model) return
  if (onlineModel.pricingUnsupportedFields?.length) {
    throw new Error(
      `在线价格包含当前计费引擎无法独立结算的${formatUnsupportedPricingFields(onlineModel.pricingUnsupportedFields)}`,
    )
  }
  if (!onlineModel.tieredPricing?.tiers?.length) {
    throw new Error(t('ui.theOnlineCatalogDoesNotProvidePricingForThisModel'))
  }

  const pricing = cloneTieredPricingConfig(onlineModel.tieredPricing)
  const pricingChanged = !tieredPricingConfigsEqual(
    props.model.default_tiered_pricing,
    pricing,
  )
  const pricingSource = {
    provider_id: onlineModel.providerId,
    provider_name: onlineModel.providerName,
  }
  const sourceChanged = !modelsDevPricingSourcesEqual(
    getModelsDevPricingSourceFromConfig(props.model.config),
    pricingSource,
  )
  const nextConfig = withModelsDevPricingSource(props.model.config, pricingSource)
  let syncedModel: GlobalModelResponse
  if (pricingChanged || sourceChanged) {
    const updatedModel = await updateGlobalModel(props.model.id, {
      default_tiered_pricing: pricing,
      config: nextConfig,
    })
    syncedModel = {
      ...updatedModel,
      default_tiered_pricing: pricing,
      config: nextConfig,
    }
  } else {
    syncedModel = {
      ...props.model,
      default_tiered_pricing: pricing,
      config: nextConfig,
    }
  }
  tieredPricing.value = cloneTieredPricingConfig(pricing)
  form.value.config = { ...nextConfig }
  billingMode.value = 'token'
  setSource(props.model.id, pricingSource)
  emit('pricingSynced', syncedModel)
  success(
    pricingChanged
      ? `已同步 ${onlineModel.providerName} 的最新价格`
      : sourceChanged
        ? t('models.pricingSource.savedNoPriceChange', { provider: onlineModel.providerName })
        : `当前价格已是 ${onlineModel.providerName} 的最新价格`,
  )
}

async function confirmOnlinePricingSource() {
  const onlineModel = selectedOnlinePricingCandidate.value
  if (!onlineModel || syncingOnlinePricing.value || submitting.value) return

  syncingOnlinePricing.value = true
  let synced = false
  try {
    await applyOnlinePricingModel(onlineModel)
    synced = true
  } catch (err: unknown) {
    log.error('同步在线模型价格失败:', err)
    showError(parseApiError(err, t('ui.failedToSyncOnlinePricing')), t('models.pricingSource.syncFailed'))
  } finally {
    syncingOnlinePricing.value = false
    if (synced) resetOnlinePricingSourceSelection()
  }
}

async function syncOnlinePricing() {
  if (!isEditMode.value || !props.model || syncingOnlinePricing.value || submitting.value) return

  syncingOnlinePricing.value = true
  try {
    const onlineModels = await refreshModelsDevList(false)
    allModelsCache.value = onlineModels
    const candidates = getOnlinePricingCandidates(onlineModels, props.model)
    const onlineModel = resolveOnlinePricingModel(candidates, props.model)
    if (onlineModel) {
      await applyOnlinePricingModel(onlineModel)
      resetOnlinePricingSourceSelection()
      return
    }

    onlinePricingCandidates.value = candidates
    selectedOnlinePricingProviderId.value = ''
    onlinePricingSourcePopoverOpen.value = true
  } catch (err: unknown) {
    resetOnlinePricingSourceSelection()
    log.error('同步在线模型价格失败:', err)
    showError(parseApiError(err, t('ui.failedToSyncOnlinePricing')), t('models.pricingSource.syncFailed'))
  } finally {
    syncingOnlinePricing.value = false
  }
}

// 清除选择（手动填写）
function clearSelection() {
  imageGenerationExplicitOverride.value = null
  selectedModel.value = null
  form.value = defaultForm()
  tieredPricing.value = null
  videoResolutionPrices.value = []
  billingMode.value = 'token'
}

// 重置表单
function resetForm() {
  resetOnlinePricingSourceSelection()
  imageGenerationExplicitOverride.value = null
  editingOnlinePricingSource.value = null
  form.value = defaultForm()
  tieredPricing.value = null
  videoResolutionPrices.value = []
  searchQuery.value = ''
  selectedModel.value = null
  expandedProvider.value = null
  presetPanelCollapsed.value = false
  billingMode.value = 'token'
}

function populateFormFromGlobalModel(model: GlobalModelResponse) {
  imageGenerationExplicitOverride.value = null
  const modelTieredPricing = model.default_tiered_pricing
    ? cloneTieredPricingConfig(model.default_tiered_pricing)
    : null
  const supportedCapabilities = new Set(model.supported_capabilities || [])
  if (tieredPricingHasImageOutputPricing(modelTieredPricing)) {
    supportedCapabilities.add('image_generation')
  }

  form.value = {
    name: model.name,
    display_name: model.display_name,
    default_price_per_request: model.default_price_per_request,
    supported_capabilities: [...supportedCapabilities],
    config: model.config ? { ...model.config } : { streaming: true },
    is_active: model.is_active,
  }
  tieredPricing.value = modelTieredPricing
  loadVideoPricingFromConfig()
  billingMode.value = 'token'
}

// 加载模型数据（编辑模式）
function loadModelData() {
  if (!props.model) return
  resetOnlinePricingSourceSelection()
  if (
    editingOnlinePricingSource.value
    && normalizeModelId(editingOnlinePricingSource.value.model_id) !== normalizeModelId(props.model.name)
  ) {
    editingOnlinePricingSource.value = null
  }
  // 先重置创建模式的残留状态
  selectedModel.value = null
  searchQuery.value = ''
  expandedProvider.value = null
  presetPanelCollapsed.value = false
  populateFormFromGlobalModel(props.model)
}

// 使用 useFormDialog 统一处理对话框逻辑
const { isEditMode, handleDialogUpdate, handleCancel } = useFormDialog({
  isOpen: () => props.open,
  entity: () => props.model,
  isLoading: submitting,
  onClose: () => emit('update:open', false),
  loadData: loadModelData,
  resetForm,
  extraLoadingStates: [syncingOnlinePricing],
})

const autoFillMissingCachePrices = computed(() => (
  !isEditMode.value && selectedModel.value === null
))

async function handleSubmit() {
  if (syncingOnlinePricing.value) return
  if (!form.value.name || !form.value.display_name) {
    showError(t('ui.enterTheModelIdAndName'))
    return
  }

  const pricingValidationError = tieredPricingEditorRef.value?.getValidationError()
  if (pricingValidationError) {
    showError(pricingValidationError, t('ui.invalidPricingConfiguration'))
    return
  }

  const finalTieredPricing = tieredPricingEditorRef.value?.getFinalPricing() ?? tieredPricing.value

  if (!finalTieredPricing?.tiers?.length) {
    showError(t('ui.configureAtLeastOnePricingTier'))
    return
  }

  // Apply billing (video) pricing into config before cleaning/submitting.
  applyVideoPricingToConfig()

  // Auto-infer supported_capabilities from tiered pricing config
  const caps = new Set(form.value.supported_capabilities || [])
  if (tieredPricingHasImageOutputPricing(finalTieredPricing)) {
    caps.add('image_generation')
  }
  form.value.supported_capabilities = caps.size > 0 ? [...caps] : []

  submitting.value = true
  try {
    if (isEditMode.value && props.model) {
      const updateData = buildGlobalModelUpdatePayload(form.value, finalTieredPricing)
      await updateGlobalModel(props.model.id, updateData)
      success(t('ui.modelUpdated'))
    } else {
      const createData = buildGlobalModelCreatePayload(form.value, finalTieredPricing)
      if (selectedModel.value) {
        createData.config = withModelsDevPricingSource(createData.config, {
          provider_id: selectedModel.value.providerId,
          provider_name: selectedModel.value.providerName,
        })
      }
      const createdModel = await createGlobalModel(createData)
      existingModelsCache.value.unshift(createdModel)
      if (selectedModel.value) {
        setSource(createdModel.id, {
          provider_id: selectedModel.value.providerId,
          provider_name: selectedModel.value.providerName,
        })
      }
      success(t('ui.modelCreated'))
      clearSelection()
      emit('success')
      return
    }
    emit('update:open', false)
    emit('success')
  } catch (err: unknown) {
    showError(parseApiError(err, isEditMode.value ? t('update.error.updateFailed') : t('ui.createFailed')), isEditMode.value ? t('update.error.updateFailed') : t('ui.createFailed'))
  } finally {
    submitting.value = false
  }
}

</script>
