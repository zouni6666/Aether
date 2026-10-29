<template>
  <Dialog
    :model-value="open"
    :title="model ? t('ui.manageProvidersInBulkValue', { name: model.display_name }) : t('ui.manageProvidersInBulk')"
    :description="t('ui.selectedProvidersAreLinkedToTheModelDeselectingAProvider')"
    :icon="Server"
    size="2xl"
    @update:model-value="handleBatchAddProvidersDialogUpdate"
  >
    <template #default>
      <div class="space-y-4">
        <!-- 搜索栏 -->
        <div class="flex items-center gap-2">
          <div class="flex-1 relative">
            <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              v-model="batchProviderSearchQuery"
              :placeholder="t('ui.searchProviders')"
              class="pl-8 h-9"
            />
          </div>
        </div>

        <!-- 单列提供商列表 -->
        <div class="border rounded-lg overflow-hidden">
          <div class="max-h-96 overflow-y-auto">
            <div
              v-if="loadingProviderOptions"
              class="flex items-center justify-center py-12"
            >
              <Loader2 class="w-6 h-6 animate-spin text-primary" />
            </div>

            <template v-else>
              <!-- 提供商组 -->
              <div v-if="filteredBatchProviders.length > 0">
                <div
                  class="flex items-center justify-between px-3 py-2 bg-muted sticky top-0 z-10"
                >
                  <div class="flex items-center gap-2">
                    <span class="text-xs font-medium">{{ t('nav.providers') }}</span>
                    <span class="text-xs text-muted-foreground">({{ filteredBatchProviders.length }})</span>
                  </div>
                  <button
                    type="button"
                    class="text-xs text-primary hover:underline shrink-0"
                    @click="toggleAllBatchProviders"
                  >
                    {{ isAllBatchProvidersSelected ? t('ui.deselectAll') : t('ui.selectAll') }}
                  </button>
                </div>
                <div class="space-y-1 p-2">
                  <div
                    v-for="provider in filteredBatchProviders"
                    :key="provider.id"
                    class="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-muted cursor-pointer"
                    @click="toggleBatchProviderSelection(provider.id)"
                  >
                    <div
                      class="w-4 h-4 border rounded flex items-center justify-center shrink-0"
                      :class="isBatchProviderSelected(provider.id) ? 'bg-primary border-primary' : ''"
                    >
                      <Check
                        v-if="isBatchProviderSelected(provider.id)"
                        class="w-3 h-3 text-primary-foreground"
                      />
                    </div>
                    <div class="flex-1 min-w-0">
                      <p class="text-sm font-medium truncate">
                        {{ provider.name }}
                      </p>
                    </div>
                    <Badge
                      :variant="provider.is_active ? 'outline' : 'secondary'"
                      :class="provider.is_active ? 'text-green-600 border-green-500/60' : ''"
                      class="text-xs shrink-0"
                    >
                      {{ provider.is_active ? t('ui.activee85e590') : t('ui.disable4e6fd0e') }}
                    </Badge>
                  </div>
                </div>
              </div>

              <!-- 空状态 -->
              <div
                v-if="filteredBatchProviders.length === 0"
                class="flex flex-col items-center justify-center py-12 text-muted-foreground"
              >
                <Building2 class="w-10 h-10 mb-2 opacity-30" />
                <p class="text-sm">
                  {{ batchProviderSearchQuery ? t('ui.noMatchingResults') : t('ui.noProvidersAvailable') }}
                </p>
              </div>
            </template>
          </div>
        </div>
      </div>
    </template>
    <template #footer>
      <div class="flex items-center justify-between w-full">
        <p class="text-xs text-muted-foreground">
          {{ hasBatchProviderChanges ? t('ui.valueChangesToSave', { count: batchProviderPendingChangesCount }) : '' }}
        </p>
        <div class="flex items-center gap-2">
          <Button
            :disabled="!hasBatchProviderChanges || submittingBatchProviders"
            @click="saveBatchProviderChanges"
          >
            <Loader2
              v-if="submittingBatchProviders"
              class="w-4 h-4 mr-1 animate-spin"
            />
            {{ submittingBatchProviders ? t('ui.saving') : t('ui.save') }}
          </Button>
          <Button
            variant="outline"
            @click="closeBatchAddProvidersDialog"
          >
            {{ t('ui.close') }}
          </Button>
        </div>
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, ref, watch } from 'vue'
import { Building2, Check, Loader2, Search, Server } from 'lucide-vue-next'
import { Badge, Button, Dialog, Input } from '@/components/ui'
import { batchAssignToProviders, type GlobalModelResponse } from '@/api/global-models'
import { getProvidersSummary, type ProviderWithEndpointsSummary } from '@/api/endpoints/providers'
import { useToast } from '@/composables/useToast'
import { parseApiError } from '@/utils/errorParser'

/**
 * 批量管理某个全局模型关联的提供商：勾选即关联、取消勾选即移除。
 * 打开时按 linkedProviders 初始化勾选，保存完成后通知父组件刷新。
 */
const props = defineProps<{
  open: boolean
  model: GlobalModelResponse | null
  linkedProviders: Array<{ id: string; model_id?: string | null }>
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  saved: []
}>()

const { t } = useI18n()

const { success, error: showError } = useToast()

let providerOptionsRequest: Promise<void> | null = null

const submittingBatchProviders = ref(false)

const providerOptions = ref<ProviderWithEndpointsSummary[]>([])

const loadingProviderOptions = ref(false)

// 单列勾选模式所需状态
const batchProviderSearchQuery = ref('')

const selectedBatchProviderIds = ref<Set<string>>(new Set())

const initialBatchProviderIds = ref<Set<string>>(new Set())

// 过滤后的提供商列表
const filteredBatchProviders = computed(() => {
  const query = batchProviderSearchQuery.value.toLowerCase().trim()
  return providerOptions.value.filter(p => {
    if (query && !p.name.toLowerCase().includes(query)) {
      return false
    }
    return true
  })
})

// 检查提供商是否已选中
function isBatchProviderSelected(providerId: string): boolean {
  return selectedBatchProviderIds.value.has(providerId)
}

// 是否全选
const isAllBatchProvidersSelected = computed(() => {
  if (filteredBatchProviders.value.length === 0) return false
  return filteredBatchProviders.value.every(p => isBatchProviderSelected(p.id))
})

// 计算待添加的提供商
const batchProvidersToAdd = computed(() => {
  const toAdd: string[] = []
  for (const id of selectedBatchProviderIds.value) {
    if (!initialBatchProviderIds.value.has(id)) {
      toAdd.push(id)
    }
  }
  return toAdd
})

// 计算待移除的提供商
const batchProvidersToRemove = computed(() => {
  const toRemove: string[] = []
  for (const id of initialBatchProviderIds.value) {
    if (!selectedBatchProviderIds.value.has(id)) {
      toRemove.push(id)
    }
  }
  return toRemove
})

// 是否有变更
const hasBatchProviderChanges = computed(() => {
  return batchProvidersToAdd.value.length > 0 || batchProvidersToRemove.value.length > 0
})

// 待变更数量
const batchProviderPendingChangesCount = computed(() => {
  return batchProvidersToAdd.value.length + batchProvidersToRemove.value.length
})

// 切换提供商选择
function toggleBatchProviderSelection(providerId: string) {
  if (selectedBatchProviderIds.value.has(providerId)) {
    selectedBatchProviderIds.value.delete(providerId)
  } else {
    selectedBatchProviderIds.value.add(providerId)
  }
  selectedBatchProviderIds.value = new Set(selectedBatchProviderIds.value)
}

// 全选/取消全选
function toggleAllBatchProviders() {
  const allIds = filteredBatchProviders.value.map(p => p.id)
  if (isAllBatchProvidersSelected.value) {
    for (const id of allIds) {
      selectedBatchProviderIds.value.delete(id)
    }
  } else {
    for (const id of allIds) {
      selectedBatchProviderIds.value.add(id)
    }
  }
  selectedBatchProviderIds.value = new Set(selectedBatchProviderIds.value)
}

// 同步初始选择状态
function syncBatchProviderSelection() {
  const existingIds = new Set(props.linkedProviders.map((p) => p.id))
  selectedBatchProviderIds.value = new Set(existingIds)
  initialBatchProviderIds.value = new Set(existingIds)
}

// 保存变更
async function saveBatchProviderChanges() {
  if (!hasBatchProviderChanges.value || submittingBatchProviders.value || !props.model) return

  submittingBatchProviders.value = true
  try {
    let totalSuccess = 0
    const allErrors: string[] = []

    // 并行移除提供商
    if (batchProvidersToRemove.value.length > 0) {
      const { deleteModel } = await import('@/api/endpoints')
      const removePromises = batchProvidersToRemove.value.map(async (providerId) => {
        const existingProvider = props.linkedProviders.find((p) => p.id === providerId)
        if (existingProvider && existingProvider.model_id) {
          return deleteModel(providerId, existingProvider.model_id)
        }
        return null
      })

      const results = await Promise.allSettled(removePromises)
      for (const result of results) {
        if (result.status === 'fulfilled' && result.value !== null) {
          totalSuccess++
        } else if (result.status === 'rejected') {
          allErrors.push(parseApiError(result.reason, t('ui.failedToRemove')))
        }
      }
    }

    // 添加提供商
    if (batchProvidersToAdd.value.length > 0) {
      const result = await batchAssignToProviders(props.model.id, {
        provider_ids: batchProvidersToAdd.value,
        create_models: true
      })
      totalSuccess += result.success.length
      if (result.errors.length > 0) {
        allErrors.push(...result.errors.map(e => e.error))
      }
    }

    if (totalSuccess > 0) {
      success(t('ui.successfullyProcessedValueProviders', { count: totalSuccess }))
    }

    if (allErrors.length > 0) {
      showError(t('ui.someOperationsFailedValueValue', { value: allErrors.slice(0, 3).join(', '), count: allErrors.length > 3 ? '...' : '' }), t('ui.warning'))
    }

    emit('saved')
    closeBatchAddProvidersDialog()
  } catch (err: unknown) {
    showError(parseApiError(err, t('ui.failedToSave')), t('ui.error'))
  } finally {
    submittingBatchProviders.value = false
  }
}

// 确保 Provider 选项已加载
async function ensureProviderOptions() {
  if (providerOptions.value.length > 0 || loadingProviderOptions.value) {
    return
  }
  if (providerOptionsRequest) {
    await providerOptionsRequest
    return
  }
  providerOptionsRequest = (async () => {
    try {
      loadingProviderOptions.value = true
      providerOptions.value = (await getProvidersSummary({ page_size: 9999 })).items
    } catch (err: unknown) {
      const message = parseApiError(err, t('ui.failedToLoadProviders'))
      showError(message, t('ui.error'))
    } finally {
      loadingProviderOptions.value = false
    }
  })()
  try {
    await providerOptionsRequest
  } finally {
    providerOptionsRequest = null
  }
}

// 处理批量添加 Provider 对话框关闭事件
function handleBatchAddProvidersDialogUpdate(value: boolean) {
  // 只有在不处于提交状态时才允许关闭
  if (!value && submittingBatchProviders.value) {
    return
  }
  emit('update:open', value)
}

// 关闭批量添加对话框
function closeBatchAddProvidersDialog() {
  emit('update:open', false)
  batchProviderSearchQuery.value = ''
  selectedBatchProviderIds.value = new Set()
  initialBatchProviderIds.value = new Set()
  submittingBatchProviders.value = false
}

watch(() => props.open, (open) => {
  if (!open) return
  batchProviderSearchQuery.value = ''
  void ensureProviderOptions().then(() => {
    syncBatchProviderSelection()
  })
})

</script>
