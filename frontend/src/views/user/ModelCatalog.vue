<template>
  <div class="space-y-6 pb-8">
    <!-- 模型列表 -->
    <Card class="overflow-hidden">
      <!-- 标题和操作栏 -->
      <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <!-- 左侧：标题 -->
          <h3 class="text-sm sm:text-base font-semibold shrink-0">
            {{ t('ui.availableModels') }}
          </h3>

          <!-- 右侧：操作区 -->
          <div class="flex flex-wrap items-center gap-2">
            <!-- 搜索框 -->
            <div class="relative">
              <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground" />
              <Input
                id="model-search"
                v-model="searchQuery"
                type="text"
                :placeholder="t('ui.searchModels')"
                class="w-32 sm:w-44 pl-8 pr-3 h-8 text-sm bg-background/50 border-border/60 focus:border-primary/40 transition-colors"
              />
            </div>

            <!-- 刷新按钮 -->
            <RefreshButton
              :loading="loading"
              @click="refreshData"
            />
          </div>
        </div>
      </div>

      <div class="overflow-x-auto">
        <Table class="hidden xl:table table-fixed w-full">
          <TableHeader>
            <TableRow class="border-b border-border/60 hover:bg-transparent">
              <TableHead class="w-[140px] h-12 font-semibold">
                {{ t('ui.modelName') }}
              </TableHead>
              <TableHead class="w-[140px] h-12 font-semibold text-center">
                {{ t('ui.priceM') }}
              </TableHead>
              <TableHead class="w-[80px] h-12 font-semibold text-center">
                {{ t('ui.callCount') }}
              </TableHead>
              <TableHead class="w-[70px] h-12 font-semibold text-center">
                {{ t('ui.status6320b4a') }}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-if="loading">
              <TableCell
                colspan="4"
                class="text-center py-12"
              >
                <Loader2 class="w-6 h-6 animate-spin mx-auto" />
              </TableCell>
            </TableRow>
            <TableRow v-else-if="filteredModels.length === 0">
              <TableCell
                colspan="4"
                class="text-center py-12 text-muted-foreground"
              >
                {{ t('ui.noMatchingModelsFound') }}
              </TableCell>
            </TableRow>
            <template v-else>
              <TableRow
                v-for="model in paginatedModels"
                :key="model.id"
                class="border-b border-border/40 hover:bg-muted/30 transition-colors cursor-pointer"
                @mousedown="handleMouseDown"
                @click="openModelDetail(model, $event)"
              >
                <TableCell class="py-4">
                  <div>
                    <div class="flex items-center gap-2">
                      <span class="font-medium hover:text-primary transition-colors">{{ model.display_name || model.name }}</span>
                      <Badge
                        v-for="capability in getModelCapabilityLabels(model)"
                        :key="capability"
                        variant="secondary"
                        class="text-[10px] px-1.5 py-0"
                      >
                        {{ capability }}
                      </Badge>
                    </div>
                    <div class="text-xs text-muted-foreground flex items-center gap-1 mt-0.5">
                      <span>{{ model.name }}</span>
                      <button
                        class="p-0.5 rounded hover:bg-muted transition-colors"
                        :title="t('ui.copyModelId')"
                        @click.stop="copyToClipboard(model.name)"
                      >
                        <Copy class="w-3 h-3" />
                      </button>
                    </div>
                  </div>
                </TableCell>
                <TableCell class="py-4 text-center">
                  <div class="text-xs space-y-0.5">
                    <!-- 按 Token 计费 -->
                    <div v-if="getFirstTierPrice(model, 'input') || getFirstTierPrice(model, 'output')">
                      <span class="text-muted-foreground">In:</span>
                      <span class="font-mono ml-1">{{ getFirstTierPrice(model, 'input')?.toFixed(2) || '-' }}</span>
                      <span class="text-muted-foreground mx-1">/</span>
                      <span class="text-muted-foreground">Out:</span>
                      <span class="font-mono ml-1">{{ getFirstTierPrice(model, 'output')?.toFixed(2) || '-' }}</span>
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
                    <!-- 无计费配置 -->
                    <div
                      v-if="!getFirstTierPrice(model, 'input') && !getFirstTierPrice(model, 'output') && !model.default_price_per_request"
                      class="text-muted-foreground"
                    >
                      -
                    </div>
                  </div>
                </TableCell>
                <TableCell class="py-4 text-center">
                  <span class="text-sm font-mono">{{ formatUsageCount(model.usage_count || 0) }}</span>
                </TableCell>
                <TableCell class="py-4 text-center">
                  <Badge :variant="model.is_active ? 'success' : 'secondary'">
                    {{ model.is_active ? t('ui.available4d99c97') : t('ui.disable4e6fd0e') }}
                  </Badge>
                </TableCell>
              </TableRow>
            </template>
          </TableBody>
        </Table>

        <!-- 移动端卡片列表 -->
        <div
          v-if="!loading && filteredModels.length > 0"
          class="xl:hidden divide-y divide-border/40"
        >
          <div
            v-for="model in paginatedModels"
            :key="model.id"
            class="p-4 space-y-3 hover:bg-muted/30 cursor-pointer transition-colors"
            @click="selectedModel = model; drawerOpen = true"
          >
            <!-- 第一行：名称 + 状态 -->
            <div class="flex items-start justify-between gap-3">
              <div class="flex-1 min-w-0">
                <span class="font-medium truncate block">{{ model.display_name || model.name }}</span>
                <div class="flex flex-wrap gap-1 mt-1">
                  <Badge
                    v-for="capability in getModelCapabilityLabels(model)"
                    :key="capability"
                    variant="secondary"
                    class="text-[10px] px-1.5 py-0"
                  >
                    {{ capability }}
                  </Badge>
                </div>
                <div class="text-xs text-muted-foreground flex items-center gap-1 mt-0.5">
                  <span class="truncate">{{ model.name }}</span>
                  <button
                    class="p-0.5 rounded hover:bg-muted transition-colors shrink-0"
                    @click.stop="copyToClipboard(model.name)"
                  >
                    <Copy class="w-3 h-3" />
                  </button>
                </div>
              </div>
              <Badge :variant="model.is_active ? 'success' : 'secondary'">
                {{ model.is_active ? t('ui.available4d99c97') : t('ui.disable4e6fd0e') }}
              </Badge>
            </div>

            <!-- 第二行：价格 + 调用次数 -->
            <div class="flex items-center gap-3 text-xs text-muted-foreground">
              <span
                v-if="getFirstTierPrice(model, 'input') || getFirstTierPrice(model, 'output')"
                class="font-mono"
              >
                In: ${{ getFirstTierPrice(model, 'input')?.toFixed(2) || '-' }} / Out: ${{ getFirstTierPrice(model, 'output')?.toFixed(2) || '-' }}
              </span>
              <span class="font-mono">{{ formatUsageCount(model.usage_count || 0) }} {{ t('ui.times') }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 分页 -->
      <Pagination
        v-if="!loading && filteredModels.length > 0"
        :current="currentPage"
        :total="filteredModels.length"
        :page-size="pageSize"
        cache-key="model-catalog-page-size"
        @update:current="currentPage = $event"
        @update:page-size="pageSize = $event"
      />
    </Card>

    <!-- 模型详情抽屉 -->
    <UserModelDetailDrawer
      v-model:open="drawerOpen"
      :model="selectedModel"
    />
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { ref, computed, onMounted, watch } from 'vue'
import {
  Loader2,
  Search,
  Copy,
} from 'lucide-vue-next'
import { useToast } from '@/composables/useToast'
import { useClipboard } from '@/composables/useClipboard'
import { useClientPagination } from '@/composables/useClientPagination'
import {
  Card,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
  Badge,
  Input,
  Pagination,
  RefreshButton,
} from '@/components/ui'
import { type PublicGlobalModel } from '@/api/public-models'
import { formatUsageCount } from '@/utils/format'
import { meApi } from '@/api/me'
import UserModelDetailDrawer from '@/features/models/components/UserModelDetailDrawer.vue'
import { useRowClick } from '@/composables/useRowClick'
import { log } from '@/utils/logger'
import { parseApiError } from '@/utils/errorParser'
import { getModelCapabilityLabels } from '@/features/models/utils/model-catalog-helpers'

const { error: showError } = useToast()
const { copyToClipboard } = useClipboard()

// 状态
const loading = ref(false)
const searchQuery = ref('')
const models = ref<PublicGlobalModel[]>([])

// 抽屉状态
const drawerOpen = ref(false)
const selectedModel = ref<PublicGlobalModel | null>(null)

// 使用复用的行点击逻辑
const { handleMouseDown, shouldTriggerRowClick } = useRowClick()

function openModelDetail(model: PublicGlobalModel, event: MouseEvent) {
  if (!shouldTriggerRowClick(event)) return
  selectedModel.value = model
  drawerOpen.value = true
}

// 筛选后的模型列表
const filteredModels = computed(() => {
  let result = models.value

  // 搜索（支持空格分隔的多关键词 AND 搜索）
  if (searchQuery.value) {
    const keywords = searchQuery.value.toLowerCase().split(/\s+/).filter(k => k.length > 0)
    result = result.filter(m => {
      const searchableText = `${m.name} ${m.display_name || ''}`.toLowerCase()
      return keywords.every(keyword => searchableText.includes(keyword))
    })
  }

  return result
})

// 分页
const { currentPage, pageSize, pageItems: paginatedModels, resetPage } = useClientPagination(filteredModels)

// 搜索变化时重置到第一页
watch(searchQuery, resetPage)

async function loadModels() {
  loading.value = true
  try {
    // 使用用户认证端点，只获取用户有权限使用的模型
    const response = await meApi.getAvailableModels({ limit: 1000 })
    models.value = (response.models || []) as PublicGlobalModel[]
  } catch (err: unknown) {
    log.error('加载模型失败:', err)
    showError(parseApiError(err, ''), t('ui.failedToLoadModels'))
  } finally {
    loading.value = false
  }
}

async function refreshData() {
  await loadModels()
}

// 从 PublicGlobalModel 的 default_tiered_pricing 获取第一阶梯价格
function getFirstTierPrice(model: PublicGlobalModel, type: 'input' | 'output'): number | null {
  const tiered = model.default_tiered_pricing
  if (!tiered?.tiers?.length) return null
  const firstTier = tiered.tiers[0]
  if (type === 'input') {
    return firstTier.input_price_per_1m || null
  }
  return firstTier.output_price_per_1m || null
}

// 检测是否有阶梯计费（多于一个阶梯）
function hasTieredPricing(model: PublicGlobalModel): boolean {
  const tiered = model.default_tiered_pricing
  return (tiered?.tiers?.length || 0) > 1
}

onMounted(() => {
  refreshData()
})

</script>
