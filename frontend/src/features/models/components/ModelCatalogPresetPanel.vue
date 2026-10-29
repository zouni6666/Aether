<template>
  <section class="h-full flex flex-col space-y-3">
    <!-- 搜索框 -->
    <div class="relative">
      <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
      <Input
        v-model="searchQuery"
        type="text"
        :placeholder="t('ui.searchModelsOrProviders')"
        class="pl-8 h-8 text-sm"
      />
    </div>

    <!-- 横向提供商 Logo 与模型列表 -->
    <div class="flex-1 min-h-0 overflow-hidden border rounded-lg flex flex-col">
      <div
        v-if="loading"
        class="flex flex-col items-center justify-center gap-3 flex-1"
      >
        <Loader2 class="w-5 h-5 animate-spin text-muted-foreground" />
        <p class="text-xs text-muted-foreground">
          {{ t('ui.loadingTheOnlineCatalogYouCanAlsoEnterModelDetails') }}
        </p>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          class="h-8"
          data-testid="models-catalog-manual-entry-loading"
          @click="$emit('manualEntry')"
        >
          {{ t('ui.enterManually') }}
        </Button>
      </div>
      <div
        v-else-if="catalogLoadError"
        class="flex flex-1 flex-col items-center justify-center gap-3 px-6 text-center"
        role="alert"
        data-testid="models-catalog-load-error"
      >
        <div class="flex h-9 w-9 items-center justify-center rounded-md border bg-muted/40 text-destructive">
          <TriangleAlert class="h-4 w-4" />
        </div>
        <div class="max-w-md space-y-1">
          <p class="text-sm font-medium">
            {{ t('ui.failedToLoadExternalModelCatalog') }}
          </p>
          <p class="break-words text-xs text-muted-foreground">
            {{ catalogLoadError }}
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          class="h-8 gap-1.5"
          data-testid="models-catalog-retry"
          @click="$emit('retry')"
        >
          <RefreshCw class="h-3.5 w-3.5" /> {{ t('vscodex.connection.retry') }}
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          class="h-8"
          data-testid="models-catalog-manual-entry"
          @click="$emit('manualEntry')"
        >
          {{ t('ui.enterDetailsManually') }}
        </Button>
      </div>
      <template v-else>
        <!-- 提供商 Logo 横向选择 -->
        <div
          v-if="groupedModels.length > 0"
          class="relative shrink-0 border-b"
        >
          <Button
            type="button"
            variant="outline"
            size="icon"
            class="absolute left-1 top-1/2 z-10 h-8 w-8 -translate-y-1/2 bg-background/95 shadow-sm"
            :title="t('ui.scrollLeft')"
            :aria-label="t('ui.scrollProvidersLeft')"
            @click="scrollProviderLogos(-1)"
          >
            <ChevronLeft class="h-4 w-4" />
          </Button>
          <div
            ref="providerLogoScroller"
            class="mx-11 flex gap-2 overflow-x-auto p-2 scrollbar-hide"
          >
            <button
              v-for="group in groupedModels"
              :key="group.providerId"
              type="button"
              class="w-[76px] shrink-0 rounded-md border px-2 py-1.5 flex flex-col items-center gap-1 transition-colors"
              :class="expandedProvider === group.providerId
                ? 'border-primary bg-primary/10 text-primary'
                : 'border-transparent hover:border-border hover:bg-muted'"
              :title="`${group.providerName}（${group.models.length}）`"
              @click="toggleProvider(group.providerId)"
            >
              <img
                :src="getProviderLogoUrl(group.providerId)"
                :alt="group.providerName"
                class="w-7 h-7 rounded object-contain dark:invert dark:brightness-90"
                @error="hideBrokenImage"
              >
              <span class="w-full truncate text-[10px] font-medium text-center">{{ group.providerName }}</span>
            </button>
          </div>
          <Button
            type="button"
            variant="outline"
            size="icon"
            class="absolute right-1 top-1/2 z-10 h-8 w-8 -translate-y-1/2 bg-background/95 shadow-sm"
            :title="t('ui.scrollRight')"
            :aria-label="t('ui.scrollProvidersRight')"
            @click="scrollProviderLogos(1)"
          >
            <ChevronRight class="h-4 w-4" />
          </Button>
        </div>

        <!-- 当前提供商模型 -->
        <div
          v-if="expandedProviderGroup"
          class="flex-1 min-h-0 overflow-y-auto p-2 scrollbar-thin"
        >
          <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
            <div
              v-for="item in expandedProviderGroup.models"
              :key="item.modelId"
              class="relative min-w-0"
            >
              <button
                type="button"
                class="group relative flex h-full min-h-[152px] w-full min-w-0 flex-col rounded-lg border bg-card p-4 text-left shadow-sm transition-[border-color,box-shadow,transform,background-color] duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40"
                :class="getExistingModel(item)
                  ? 'cursor-default border-border/70'
                  : isSelected(item)
                    ? 'cursor-pointer border-primary bg-primary/5 ring-1 ring-primary hover:-translate-y-0.5 hover:border-primary/35 hover:shadow-md active:scale-[0.96]'
                    : 'cursor-pointer border-border/70 hover:-translate-y-0.5 hover:border-primary/35 hover:shadow-md active:scale-[0.96]'"
                :disabled="!!getExistingModel(item)"
                @click="$emit('select', item)"
              >
                <span
                  v-if="isSelected(item)"
                  class="absolute right-2.5 top-2.5 flex h-5 w-5 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-sm"
                >
                  <Check class="h-3 w-3" />
                </span>

                <span
                  class="flex w-full items-start gap-2"
                  :class="getExistingModel(item)
                    ? 'pr-24'
                    : isSelected(item)
                      ? 'pr-7'
                      : ''"
                >
                  <span class="min-w-0 flex-1">
                    <span class="block truncate text-sm font-semibold leading-5">{{ item.modelName }}</span>
                    <span class="block truncate font-mono text-[10px] text-muted-foreground">{{ item.modelId }}</span>
                  </span>
                </span>

                <span class="mt-2 flex min-h-5 flex-wrap gap-1">
                  <span
                    v-if="item.supportsReasoning"
                    class="inline-flex items-center gap-1 rounded-md border border-violet-500/20 bg-violet-500/10 px-1.5 py-0.5 text-[9px] font-medium text-violet-700 dark:text-violet-300"
                  >
                    <BrainCircuit class="h-2.5 w-2.5" />{{ t('ui.reasoning132411b') }} </span>
                  <span
                    v-if="item.supportsVision"
                    class="inline-flex items-center gap-1 rounded-md border border-sky-500/20 bg-sky-500/10 px-1.5 py-0.5 text-[9px] font-medium text-sky-700 dark:text-sky-300"
                  >
                    <Eye class="h-2.5 w-2.5" />{{ t('ui.vision209c47b') }} </span>
                  <span
                    v-if="item.supportsToolCall"
                    class="inline-flex items-center gap-1 rounded-md border border-amber-500/20 bg-amber-500/10 px-1.5 py-0.5 text-[9px] font-medium text-amber-700 dark:text-amber-300"
                  >
                    <Wrench class="h-2.5 w-2.5" />{{ t('ui.tools') }} </span>
                  <span
                    v-if="item.supportsStructuredOutput"
                    class="inline-flex items-center gap-1 rounded-md border border-emerald-500/20 bg-emerald-500/10 px-1.5 py-0.5 text-[9px] font-medium text-emerald-700 dark:text-emerald-300"
                  >
                    <Braces class="h-2.5 w-2.5" />{{ t('ui.structuredOutput') }} </span>
                  <span
                    v-if="item.supportsEmbedding"
                    class="inline-flex items-center gap-1 rounded-md border border-fuchsia-500/20 bg-fuchsia-500/10 px-1.5 py-0.5 text-[9px] font-medium text-fuchsia-700 dark:text-fuchsia-300"
                  >
                    <Database class="h-2.5 w-2.5" />Embedding
                  </span>
                  <span
                    v-if="item.openWeights"
                    class="inline-flex items-center gap-1 rounded-md border border-border bg-muted/70 px-1.5 py-0.5 text-[9px] font-medium text-muted-foreground"
                  >
                    <PackageOpen class="h-2.5 w-2.5" />{{ t('ui.openWeights') }} </span>
                </span>

                <span class="mt-auto flex w-full items-end justify-between gap-2 border-t border-border/60 pt-2 text-[9px] text-muted-foreground">
                  <span class="flex min-w-0 flex-col">
                    <span v-if="item.contextLimit">{{ t('ui.context') }} {{ formatTokenLimit(item.contextLimit) }}</span>
                    <span v-else>{{ t('ui.unknownContextSize') }}</span>
                    <span v-if="item.outputLimit">{{ t('ui.outputfb04add') }} {{ formatTokenLimit(item.outputLimit) }}</span>
                  </span>
                  <span
                    v-if="item.inputPrice !== undefined || item.outputPrice !== undefined"
                    class="shrink-0 text-right font-medium tabular-nums text-foreground/70"
                  >
                    <span class="block">{{ t('ui.input4b77c7d') }}{{ formatModelPrice(item.inputPrice) }}/M</span>
                    <span class="block">{{ t('ui.output') }}{{ formatModelPrice(item.outputPrice) }}/M</span>
                  </span>
                  <span
                    v-else-if="item.releaseDate"
                    class="shrink-0"
                  >{{ item.releaseDate }}</span>
                </span>
              </button>
              <div
                v-if="getExistingModel(item)"
                class="absolute right-2.5 top-2.5 z-10 flex items-center gap-1.5"
              >
                <span class="inline-flex h-6 items-center rounded-md bg-muted px-2 text-[10px] font-medium text-muted-foreground">{{ t('ui.added') }}</span>
                <Button
                  type="button"
                  variant="outline"
                  size="icon"
                  class="h-7 w-7 bg-background/90 text-muted-foreground shadow-sm hover:text-foreground"
                  :title="t('ui.edit')"
                  :aria-label="t('ui.editValue', { name: item.modelName })"
                  :data-testid="`edit-existing-model-${item.modelId}`"
                  @click="editExisting(item)"
                >
                  <SquarePen class="h-3.5 w-3.5" />
                </Button>
              </div>
            </div>
          </div>
        </div>
        <div
          v-else-if="groupedModels.length > 0"
          class="flex flex-1 items-center justify-center text-xs text-muted-foreground"
        >
          {{ t('ui.expandAProviderToViewItsModels') }}
        </div>
        <div
          v-else
          class="flex flex-1 items-center justify-center text-sm text-muted-foreground"
        >
          {{ searchQuery ? t('ui.noModelsFound') : t('ui.noModelsAvailable') }}
        </div>
      </template>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, ref, watch } from 'vue'
import {
  BrainCircuit, Braces, Check, ChevronLeft, ChevronRight, Database, Eye,
  Loader2, PackageOpen, RefreshCw, Search, SquarePen, TriangleAlert, Wrench,
} from 'lucide-vue-next'
import { Button, Input } from '@/components/ui'
import { getProviderLogoUrl, type ModelsDevModelItem } from '@/api/models-dev'
import type { GlobalModelResponse } from '@/api/global-models'
import { findGlobalModelByName } from './global-model-form-helpers'
import {
  formatModelPrice,
  formatTokenLimit,
  getDefaultProviderId,
  groupModelsDevCatalog,
  hideBrokenImage,
} from '../utils/modelsDevCatalog'

/**
 * 创建模型时的在线目录预设面板：按提供商 Logo 切换、搜索并选择一个预设。
 * 目录、已有模型、搜索词与展开的提供商都由父组件持有，面板只负责展示与交互。
 */
const props = defineProps<{
  models: ModelsDevModelItem[]
  existingModels: GlobalModelResponse[]
  selectedModel: ModelsDevModelItem | null
  loading: boolean
  catalogLoadError: string | null
  searchQuery: string
  expandedProvider: string | null
}>()

const emit = defineEmits<{
  'update:searchQuery': [value: string]
  'update:expandedProvider': [value: string | null]
  select: [model: ModelsDevModelItem]
  editExisting: [model: ModelsDevModelItem, existing: GlobalModelResponse]
  retry: []
  manualEntry: []
}>()

const { t } = useI18n()

// 搜索词与展开的提供商由父组件持有，折叠后再打开时保持原状
const searchQuery = computed({
  get: () => props.searchQuery,
  set: (value: string) => emit('update:searchQuery', value),
})

const expandedProvider = computed({
  get: () => props.expandedProvider,
  set: (value: string | null) => emit('update:expandedProvider', value),
})

const providerLogoScroller = ref<HTMLElement | null>(null)

const groupedModels = computed(() => groupModelsDevCatalog(props.models, searchQuery.value))

const expandedProviderGroup = computed(() => (
  groupedModels.value.find(group => group.providerId === expandedProvider.value) ?? null
))

// 搜索时只剩一个提供商则自动展开；首次进入默认展开 Claude / 第一个提供商
watch(groupedModels, (groups) => {
  if (expandedProvider.value && !groups.some(group => group.providerId === expandedProvider.value)) {
    expandedProvider.value = null
  }
  if (searchQuery.value && groups.length === 1) {
    expandedProvider.value = groups[0].providerId
  } else if (!searchQuery.value && !expandedProvider.value) {
    expandedProvider.value = getDefaultProviderId(groups)
  }
}, { immediate: true })

// 切换提供商展开状态
function toggleProvider(providerId: string) {
  expandedProvider.value = expandedProvider.value === providerId ? null : providerId
}

function scrollProviderLogos(direction: -1 | 1) {
  providerLogoScroller.value?.scrollBy({
    left: direction * 280,
    behavior: 'smooth',
  })
}

function getExistingModel(model: ModelsDevModelItem): GlobalModelResponse | undefined {
  return findGlobalModelByName(props.existingModels, model.modelId)
}

function isSelected(item: ModelsDevModelItem): boolean {
  return props.selectedModel?.modelId === item.modelId && props.selectedModel?.providerId === item.providerId
}

function editExisting(item: ModelsDevModelItem) {
  const existing = getExistingModel(item)
  if (existing) emit('editExisting', item, existing)
}

</script>
