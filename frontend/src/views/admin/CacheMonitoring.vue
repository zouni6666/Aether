<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { ref, computed, nextTick, onMounted, watch, onBeforeUnmount } from 'vue'
import Card from '@/components/ui/card.vue'
import Button from '@/components/ui/button.vue'
import Badge from '@/components/ui/badge.vue'
import Table from '@/components/ui/table.vue'
import TableBody from '@/components/ui/table-body.vue'
import TableCell from '@/components/ui/table-cell.vue'
import TableHead from '@/components/ui/table-head.vue'
import TableHeader from '@/components/ui/table-header.vue'
import TableRow from '@/components/ui/table-row.vue'
import Input from '@/components/ui/input.vue'
import Pagination from '@/components/ui/pagination.vue'
import RefreshButton from '@/components/ui/refresh-button.vue'
import Select from '@/components/ui/select.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import SelectValue from '@/components/ui/select-value.vue'
import ScatterChart from '@/components/charts/ScatterChart.vue'
import { Trash2, Eraser, Search, X, BarChart3, ChevronDown, ChevronRight, Database, ArrowRight, HardDrive } from 'lucide-vue-next'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { useClientPagination } from '@/composables/useClientPagination'
import { cacheApi, modelMappingCacheApi, redisCacheApi, type CacheStats, type CacheConfig, type UserAffinity, type ModelMappingCacheStats, type RedisCacheCategoriesResponse } from '@/api/cache'
import { formatNumber, formatTokens, formatCost, formatRemainingTime } from '@/utils/format'
import {
  useTTLAnalysis,
  ANALYSIS_HOURS_OPTIONS,
  getTTLBadgeVariant,
  getFrequencyLabel,
  getFrequencyClass
} from '@/composables/useTTLAnalysis'
import { log } from '@/utils/logger'
import { formatApiFormat } from '@/api/endpoints/types/api-format'
import { formatClientFamily } from '@/features/usage/utils/clientFamily'
import { observeNearViewportOnce } from '@/utils/nearViewport'
import {
  truncateMiddle,
  affinityUserLabel,
  affinityUserTitle,
  affinityUserApiKeyLabel,
  affinityModelLabel,
  affinityModelSubtitle,
  providerKeyLabel,
  providerKeyTitle,
  formatAffinityRequestCount,
  formatAffinityRequestCountUnit,
  formatIntervalDescription,
  formatTTL,
  getUnmappedStatusBadge,
} from '@/features/cache/utils/cacheDisplay'

// ==================== 缓存统计与亲和性列表 ====================

const stats = ref<CacheStats | null>(null)
const config = ref<CacheConfig | null>(null)
const loading = ref(false)
const affinityList = ref<UserAffinity[]>([])
const listLoading = ref(false)
const tableKeyword = ref('')
const matchedUserId = ref<string | null>(null)
const clearingRowAffinityKey = ref<string | null>(null)
const clearingAllAffinity = ref(false)
const currentTime = ref(Math.floor(Date.now() / 1000))
const isPageVisible = ref(typeof document === 'undefined' ? true : !document.hidden)
const nextExpireAt = ref<number | null>(null)

// ==================== 模型映射缓存 ====================

const modelMappingStats = ref<ModelMappingCacheStats | null>(null)
const modelMappingLoading = ref(false)
const clearingModelMapping = ref(false)
const clearingModelName = ref<string | null>(null)

// ==================== Redis 缓存分类管理 ====================

const redisCacheData = ref<RedisCacheCategoriesResponse | null>(null)
const redisCacheLoading = ref(false)
const clearingCategory = ref<string | null>(null)

const { success: showSuccess, error: showError, info: showInfo } = useToast()
const { confirm: showConfirm } = useConfirm()

let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null
let skipNextKeywordWatch = false
let countdownTimer: ReturnType<typeof setInterval> | null = null
let stopAnalysisObserver: (() => void) | null = null
let initialDataPromise: Promise<void> | null = null
let analysisLoadRequested = false
let isUnmounted = false

const analysisSectionRef = ref<HTMLElement | null>(null)

// ==================== TTL 分析 (使用 composable) ====================

const {
  ttlAnalysis,
  hitAnalysis,
  ttlAnalysisLoading,
  analysisHours,
  expandedUserId,
  userTimelineData,
  userTimelineLoading,
  userTimelineChartData,
  toggleUserExpand,
  refreshAnalysis
} = useTTLAnalysis()

// ==================== 计算属性 ====================

const { currentPage, pageSize, pageItems: paginatedAffinityList, resetPage } = useClientPagination(affinityList)

// ==================== 缓存统计方法 ====================

async function fetchCacheStats() {
  loading.value = true
  try {
    stats.value = await cacheApi.getStats()
  } catch (error) {
    showError(t('ui.failedToLoadCacheStatistics'))
    log.error(t('ui.failedToLoadCacheStatistics'), error)
  } finally {
    loading.value = false
  }
}

async function fetchCacheConfig() {
  try {
    config.value = await cacheApi.getConfig()
  } catch (error) {
    log.error('获取缓存配置失败', error)
  }
}

async function fetchAffinityList(keyword?: string) {
  listLoading.value = true
  try {
    const response = await cacheApi.listAffinities(keyword)
    affinityList.value = response.items
    matchedUserId.value = response.matched_user_id ?? null
    currentTime.value = Math.floor(Date.now() / 1000)
    pruneExpiredAffinities(currentTime.value, true)

    if (keyword && response.total === 0) {
      showInfo(t('ui.noMatchingCacheRecordsFound'))
    }
  } catch (error) {
    showError(t('ui.failedToLoadCacheList'))
    log.error(t('ui.failedToLoadCacheList'), error)
  } finally {
    listLoading.value = false
  }
}

async function resetAffinitySearch() {
  if (searchDebounceTimer) {
    clearTimeout(searchDebounceTimer)
    searchDebounceTimer = null
  }

  if (!tableKeyword.value) {
    resetPage()
    await fetchAffinityList()
    return
  }

  skipNextKeywordWatch = true
  tableKeyword.value = ''
  resetPage()
  await fetchAffinityList()
}

async function clearSingleAffinity(item: UserAffinity) {
  const affinityKey = item.affinity_key?.trim()
  const endpointId = item.endpoint_id?.trim()
  const modelId = item.global_model_id?.trim()
  const apiFormat = item.api_format?.trim()

  if (!affinityKey || !endpointId || !modelId || !apiFormat) {
    showError(t('ui.cacheRecordInformationIsIncompleteAndCannotBeDeleted'))
    return
  }

  const label = item.user_api_key_name || affinityKey
  const modelLabel = item.model_display_name || item.model_name || modelId
  const confirmed = await showConfirm({
    get title() { return t('ui.confirmClear') },
    message: t('ui.clearCacheAffinityForValueOnModelValue', { value: label, value2: modelLabel }),
    get confirmText() { return t('ui.confirmClear') },
    variant: 'destructive'
  })

  if (!confirmed) return

  clearingRowAffinityKey.value = affinityKey
  try {
    await cacheApi.clearSingleAffinity(
      affinityKey,
      endpointId,
      modelId,
      apiFormat,
      item.client_family,
      item.session_hash
    )
    showSuccess(t('ui.clearedSuccessfully'))
    await fetchCacheStats()
    await fetchAffinityList(tableKeyword.value.trim() || undefined)
  } catch (error) {
    showError(t('ui.failedToClear'))
    log.error('清除单条缓存失败', error)
  } finally {
    clearingRowAffinityKey.value = null
  }
}

async function clearAllCache() {
  const firstConfirm = await showConfirm({
    get title() { return t('ui.dangerousAction') },
    get message() { return t('ui.thisClearsCacheAffinityForAllUsersContinue') },
    get confirmText() { return t('ui.continue') },
    variant: 'destructive'
  })
  if (!firstConfirm) return

  const secondConfirm = await showConfirm({
    get title() { return t('ui.confirmAgain') },
    get message() { return t('ui.thisAffectsAllUsersPleaseConfirmAgain') },
    get confirmText() { return t('ui.confirmClear') },
    variant: 'destructive'
  })
  if (!secondConfirm) return

  clearingAllAffinity.value = true
  try {
    await cacheApi.clearAllCache()
    showSuccess('已清除所有缓存')
    await fetchCacheStats()
    await fetchAffinityList(tableKeyword.value.trim() || undefined)
  } catch (error) {
    showError(t('ui.failedToClear'))
    log.error('清除所有缓存失败', error)
  } finally {
    clearingAllAffinity.value = false
  }
}

// ==================== 工具方法 ====================

function getRemainingTime(expireAt?: number): string {
  return formatRemainingTime(expireAt, currentTime.value)
}

function handlePageChange() {
  window.scrollTo({ top: 0, behavior: 'smooth' })
}

function recalculateNextExpireAt(now: number = currentTime.value) {
  let nearestExpireAt: number | null = null

  for (const item of affinityList.value) {
    if (!item.expire_at || item.expire_at <= now) continue
    if (nearestExpireAt === null || item.expire_at < nearestExpireAt) {
      nearestExpireAt = item.expire_at
    }
  }

  nextExpireAt.value = nearestExpireAt
}

function pruneExpiredAffinities(now: number, silent = false) {
  const beforeCount = affinityList.value.length
  const activeItems = affinityList.value.filter(
    item => item.expire_at && item.expire_at > now
  )

  if (activeItems.length === beforeCount) {
    recalculateNextExpireAt(now)
    return
  }

  affinityList.value = activeItems
  recalculateNextExpireAt(now)

  if (!silent) {
    showInfo(`${beforeCount - activeItems.length} 个缓存已自动过期移除`)
  }
}

// ==================== 定时器管理 ====================

function startCountdown() {
  if (!isPageVisible.value) return
  if (countdownTimer) clearInterval(countdownTimer)

  countdownTimer = setInterval(() => {
    currentTime.value = Math.floor(Date.now() / 1000)

    if (nextExpireAt.value !== null && currentTime.value >= nextExpireAt.value) {
      pruneExpiredAffinities(currentTime.value)
    }
  }, 1000)
}

function stopCountdown() {
  if (countdownTimer) {
    clearInterval(countdownTimer)
    countdownTimer = null
  }
}

function handleVisibilityChange() {
  isPageVisible.value = !document.hidden
  if (!isPageVisible.value) {
    stopCountdown()
    return
  }
  currentTime.value = Math.floor(Date.now() / 1000)
  if (nextExpireAt.value !== null && currentTime.value >= nextExpireAt.value) {
    pruneExpiredAffinities(currentTime.value)
  }
  startCountdown()
}

// ==================== 模型映射缓存方法 ====================

async function fetchModelMappingStats() {
  modelMappingLoading.value = true
  try {
    modelMappingStats.value = await modelMappingCacheApi.getStats()
  } catch (error) {
    showError('获取模型映射缓存统计失败')
    log.error('获取模型映射缓存统计失败', error)
  } finally {
    modelMappingLoading.value = false
  }
}

async function clearAllModelMappingCache() {
  const confirmed = await showConfirm({
    get title() { return t('ui.confirmClear') },
    get message() { return t('ui.clearAllModelMappingCachesThisAffectsNameResolutionFor') },
    get confirmText() { return t('ui.confirmClear') },
    variant: 'destructive'
  })

  if (!confirmed) return

  clearingModelMapping.value = true
  try {
    const result = await modelMappingCacheApi.clearAll()
    showSuccess(`已清除 ${result.deleted_count} 个缓存键`)
    await fetchModelMappingStats()
  } catch (error) {
    showError('清除模型映射缓存失败')
    log.error('清除模型映射缓存失败', error)
  } finally {
    clearingModelMapping.value = false
  }
}

async function clearModelMappingByName(modelName: string) {
  clearingModelName.value = modelName
  try {
    await modelMappingCacheApi.clearByName(modelName)
    showSuccess(`已清除 ${modelName} 的映射缓存`)
    await fetchModelMappingStats()
  } catch (error) {
    showError('清除缓存失败')
    log.error('清除模型映射缓存失败', error)
  } finally {
    clearingModelName.value = null
  }
}

async function clearProviderModelMapping(providerId: string, globalModelId: string, displayName?: string) {
  const confirmed = await showConfirm({
    get title() { return t('ui.confirmClear') },
    message: `确定要清除 ${displayName || 'Provider 模型映射'} 的缓存吗？`,
    get confirmText() { return t('ui.confirmClear') },
    variant: 'destructive'
  })

  if (!confirmed) return

  try {
    await modelMappingCacheApi.clearProviderModel(providerId, globalModelId)
    showSuccess('已清除 Provider 模型映射缓存')
    await fetchModelMappingStats()
  } catch (error) {
    showError('清除缓存失败')
    log.error('清除 Provider 模型映射缓存失败', error)
  }
}

// ==================== Redis 缓存分类管理方法 ====================

async function fetchRedisCacheCategories() {
  redisCacheLoading.value = true
  try {
    redisCacheData.value = await redisCacheApi.getCategories()
  } catch (error) {
    showError('获取 Redis 缓存分类失败')
    log.error('获取 Redis 缓存分类失败', error)
  } finally {
    redisCacheLoading.value = false
  }
}

async function clearRedisCategory(categoryKey: string, categoryName: string, count: number) {
  if (count === 0) {
    showInfo(`${categoryName} 缓存为空，无需清理`)
    return
  }
  const confirmed = await showConfirm({
    title: `清除 ${categoryName} 缓存`,
    message: `确定要清除 ${categoryName} 的所有缓存吗？共 ${count} 个键。`,
  })
  if (!confirmed) return

  clearingCategory.value = categoryKey
  try {
    const result = await redisCacheApi.clearCategory(categoryKey)
    showSuccess(`已清除 ${categoryName} 缓存（${result.deleted_count} 个键）`)
    await fetchRedisCacheCategories()
  } catch (error) {
    showError(`清除 ${categoryName} 缓存失败`)
    log.error('清除 Redis 缓存分类失败', error)
  } finally {
    clearingCategory.value = null
  }
}

const redisCategoriesWithKeys = computed(() => {
  if (!redisCacheData.value?.categories) return []
  return redisCacheData.value.categories.filter(c => c.count > 0)
})

const redisCategoriesEmpty = computed(() => {
  if (!redisCacheData.value?.categories) return []
  return redisCacheData.value.categories.filter(c => c.count === 0)
})

// ==================== 刷新所有数据 ====================

async function refreshData() {
  await Promise.all([
    fetchCacheStats(),
    fetchCacheConfig(),
    fetchAffinityList(),
    fetchModelMappingStats(),
    fetchRedisCacheCategories()
  ])
}

function requestAnalysisLoad() {
  if (analysisLoadRequested) return
  analysisLoadRequested = true

  const waitForInitialData = initialDataPromise ?? Promise.resolve()
  void waitForInitialData.then(() => {
    if (!isUnmounted) {
      return refreshAnalysis()
    }
  })
}

function setupAnalysisObserver() {
  const target = analysisSectionRef.value
  if (!target) {
    requestAnalysisLoad()
    return
  }

  const scrollRoot = target.closest('.app-shell__content')
  stopAnalysisObserver = observeNearViewportOnce({
    target,
    root: scrollRoot,
    rootMargin: '600px 0px',
    onNearViewport: requestAnalysisLoad,
  })
}

// ==================== 生命周期 ====================

watch(tableKeyword, (value) => {
  if (skipNextKeywordWatch) {
    skipNextKeywordWatch = false
    return
  }

  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)

  const keyword = value.trim()
  searchDebounceTimer = setTimeout(() => {
    fetchAffinityList(keyword || undefined)
    searchDebounceTimer = null
  }, 600)
})

onMounted(() => {
  document.addEventListener('visibilitychange', handleVisibilityChange)
  const initialLoad = refreshData()
  initialDataPromise = initialLoad
  void initialLoad.finally(() => {
    if (initialDataPromise === initialLoad) {
      initialDataPromise = null
    }
  })
  startCountdown()
  void nextTick().then(() => {
    if (!isUnmounted) setupAnalysisObserver()
  })
})

onBeforeUnmount(() => {
  isUnmounted = true
  document.removeEventListener('visibilitychange', handleVisibilityChange)
  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
  stopAnalysisObserver?.()
  stopAnalysisObserver = null
  stopCountdown()
})

</script>

<template>
  <div class="space-y-6">
    <!-- 标题 -->
    <div>
      <h2 class="text-2xl font-bold">
        {{ t('nav.cacheMonitoring') }}
      </h2>
      <p class="text-sm text-muted-foreground mt-1">
        {{ t('ui.manageCacheAffinityToImprovePromptCachingHitRate') }}
      </p>
    </div>

    <!-- 亲和性系统状态 -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <Card class="p-4">
        <div class="text-xs text-muted-foreground">
          {{ t('ui.activeAffinities') }}
        </div>
        <div class="text-2xl font-bold mt-1">
          {{ stats?.affinity_stats?.active_affinities || stats?.affinity_stats?.total_affinities || 0 }}
        </div>
        <div class="text-xs text-muted-foreground mt-1">
          TTL {{ config?.cache_ttl_seconds || 300 }}s
        </div>
      </Card>

      <Card class="p-4">
        <div class="text-xs text-muted-foreground">
          {{ t('ui.providerSwitches') }}
        </div>
        <div
          class="text-2xl font-bold mt-1"
          :class="(stats?.affinity_stats?.provider_switches || 0) > 0 ? 'text-destructive' : ''"
        >
          {{ stats?.affinity_stats?.provider_switches || 0 }}
        </div>
        <div class="text-xs text-muted-foreground mt-1">
          {{ t('ui.keySwitches') }} {{ stats?.affinity_stats?.key_switches || 0 }}
        </div>
      </Card>

      <Card class="p-4">
        <div class="text-xs text-muted-foreground">
          {{ t('ui.cacheInvalidations') }}
        </div>
        <div
          class="text-2xl font-bold mt-1"
          :class="(stats?.affinity_stats?.cache_invalidations || 0) > 0 ? 'text-warning' : ''"
        >
          {{ stats?.affinity_stats?.cache_invalidations || 0 }}
        </div>
        <div class="text-xs text-muted-foreground mt-1">
          {{ t('ui.dueToProviderUnavailability') }}
        </div>
      </Card>

      <Card class="p-4">
        <div class="text-xs text-muted-foreground flex items-center gap-1">
          {{ t('ui.reservedRatio') }} <Badge
            v-if="config?.dynamic_reservation?.enabled"
            variant="outline"
            class="text-[10px] px-1"
          >
            {{ t('ui.dynamic') }}
          </Badge>
        </div>
        <div class="text-2xl font-bold mt-1">
          <template v-if="config?.dynamic_reservation?.enabled">
            {{ (config.dynamic_reservation.config.stable_min_reservation * 100).toFixed(0) }}-{{ (config.dynamic_reservation.config.stable_max_reservation * 100).toFixed(0) }}%
          </template>
          <template v-else>
            {{ config ? (config.cache_reservation_ratio * 100).toFixed(0) : '30' }}%
          </template>
        </div>
        <div class="text-xs text-muted-foreground mt-1">
          {{ t('ui.current') }} {{ stats ? (stats.cache_reservation_ratio * 100).toFixed(0) : '-' }}%
        </div>
      </Card>
    </div>

    <!-- 缓存亲和性列表 -->
    <Card class="overflow-hidden">
      <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <h3 class="text-sm sm:text-base font-semibold shrink-0">
            {{ t('ui.affinities') }}
          </h3>
          <div class="flex flex-wrap items-center gap-2">
            <div class="relative">
              <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground z-10 pointer-events-none" />
              <Input
                id="cache-affinity-search"
                v-model="tableKeyword"
                :placeholder="t('ui.searchUsersOrKeys')"
                class="w-32 sm:w-48 h-8 text-sm pl-8 pr-8"
              />
              <button
                v-if="tableKeyword"
                type="button"
                class="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground z-10"
                @click="resetAffinitySearch"
              >
                <X class="h-3.5 w-3.5" />
              </button>
            </div>
            <div class="hidden sm:block h-4 w-px bg-border" />
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8 text-muted-foreground/70 hover:text-destructive"
              :disabled="clearingAllAffinity"
              :title="t('ui.clearAllCaches')"
              @click="clearAllCache"
            >
              <Eraser class="h-4 w-4" />
            </Button>
            <RefreshButton
              :loading="loading || listLoading"
              @click="refreshData"
            />
          </div>
        </div>
      </div>

      <Table class="hidden xl:table">
        <TableHeader>
          <TableRow>
            <TableHead class="w-36">
              {{ t('auth.role.user') }}
            </TableHead>
            <TableHead class="w-28">
              Key
            </TableHead>
            <TableHead class="w-28">
              Provider
            </TableHead>
            <TableHead class="w-40">
              {{ t('ui.model') }}
            </TableHead>
            <TableHead class="w-24">
              {{ t('ui.client') }}
            </TableHead>
            <TableHead class="w-36">
              {{ t('ui.apiFormatKey') }}
            </TableHead>
            <TableHead class="w-20 text-center">
              {{ t('ui.remainingd6822b0') }}
            </TableHead>
            <TableHead class="w-14 text-center">
              {{ t('ui.count') }}
            </TableHead>
            <TableHead class="w-12 text-right">
              {{ t('ui.actionsed31fbb') }}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody v-if="!listLoading && affinityList.length">
          <TableRow
            v-for="item in paginatedAffinityList"
            :key="`${item.affinity_key}-${item.endpoint_id}-${item.key_id}-${item.global_model_id || item.model_name || 'unknown'}-${item.api_format || 'unknown'}-${item.client_family || 'unknown'}-${item.session_hash || 'legacy'}`"
          >
            <TableCell>
              <div class="flex items-center gap-1.5">
                <Badge
                  v-if="item.is_standalone"
                  variant="outline"
                  class="text-warning border-warning/30 text-[10px] px-1"
                >
                  {{ t('ui.independent') }}
                </Badge>
                <span
                  class="text-sm font-medium truncate max-w-[120px]"
                  :title="affinityUserTitle(item)"
                >{{ affinityUserLabel(item) }}</span>
              </div>
            </TableCell>
            <TableCell>
              <div class="flex items-center gap-1.5">
                <span
                  class="text-sm truncate max-w-[80px]"
                  :title="item.user_api_key_name || item.affinity_key"
                >{{ affinityUserApiKeyLabel(item) }}</span>
                <Badge
                  v-if="item.api_format && item.rate_multipliers?.[item.api_format] && item.rate_multipliers[item.api_format] !== 1.0"
                  variant="outline"
                  class="text-warning border-warning/30 text-[10px] px-2"
                >
                  {{ item.rate_multipliers[item.api_format] }}x
                </Badge>
              </div>
              <div class="text-xs text-muted-foreground font-mono">
                {{ item.user_api_key_prefix || '---' }}
              </div>
            </TableCell>
            <TableCell>
              <div
                class="text-sm truncate max-w-[100px]"
                :title="item.provider_name || undefined"
              >
                {{ item.provider_name || t('chart.unknown') }}
              </div>
            </TableCell>
            <TableCell>
              <div
                class="text-sm truncate max-w-[150px]"
                :title="affinityModelLabel(item)"
              >
                {{ affinityModelLabel(item) }}
              </div>
              <div
                v-if="affinityModelSubtitle(item)"
                class="text-xs text-muted-foreground"
                :title="item.model_name || undefined"
              >
                {{ affinityModelSubtitle(item) }}
              </div>
            </TableCell>
            <TableCell>
              <Badge
                variant="outline"
                class="text-[10px] px-2 font-normal"
              >
                {{ formatClientFamily(item.client_family) }}
              </Badge>
            </TableCell>
            <TableCell>
              <div class="text-sm">
                {{ formatApiFormat(item.api_format) }}
              </div>
              <div
                class="text-xs text-muted-foreground truncate max-w-[130px]"
                :title="providerKeyTitle(item)"
              >
                {{ providerKeyLabel(item) }}
              </div>
            </TableCell>
            <TableCell class="text-center">
              <span class="text-xs">{{ getRemainingTime(item.expire_at) }}</span>
            </TableCell>
            <TableCell class="text-center">
              <span
                class="text-sm"
                :title="item.request_count_known === false ? t('ui.exactCountsAreUnavailableForThisCacheSource') : undefined"
              >{{ formatAffinityRequestCount(item) }}</span>
            </TableCell>
            <TableCell class="text-right">
              <Button
                size="icon"
                variant="ghost"
                class="h-7 w-7 text-muted-foreground/70 hover:text-destructive"
                :disabled="clearingRowAffinityKey === item.affinity_key"
                :title="t('ui.clearCache')"
                @click="clearSingleAffinity(item)"
              >
                <Trash2 class="h-3.5 w-3.5" />
              </Button>
            </TableCell>
          </TableRow>
        </TableBody>
        <TableBody v-else>
          <TableRow>
            <TableCell
              colspan="9"
              class="text-center py-6 text-sm text-muted-foreground"
            >
              {{ listLoading ? t('common.loading') : t('ui.noCacheRecords') }}
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>

      <!-- 移动端卡片列表 -->
      <div
        v-if="!listLoading && affinityList.length > 0"
        class="xl:hidden divide-y divide-border/40"
      >
        <div
          v-for="item in paginatedAffinityList"
          :key="`m-${item.affinity_key}-${item.endpoint_id}-${item.key_id}-${item.global_model_id || item.model_name || 'unknown'}-${item.api_format || 'unknown'}-${item.client_family || 'unknown'}-${item.session_hash || 'legacy'}`"
          class="p-4 space-y-2"
        >
          <div class="flex items-start justify-between gap-3">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-1.5">
                <Badge
                  v-if="item.is_standalone"
                  variant="outline"
                  class="text-warning border-warning/30 text-[10px] px-1"
                >
                  {{ t('ui.independent') }}
                </Badge>
                <span class="text-sm font-medium truncate">{{ affinityUserLabel(item) }}</span>
              </div>
              <div class="text-xs text-muted-foreground mt-0.5">
                {{ affinityUserApiKeyLabel(item) }} · {{ item.user_api_key_prefix || truncateMiddle(item.affinity_key) }}
              </div>
            </div>
            <Button
              size="icon"
              variant="ghost"
              class="h-7 w-7 text-muted-foreground/70 hover:text-destructive shrink-0"
              :disabled="clearingRowAffinityKey === item.affinity_key"
              @click="clearSingleAffinity(item)"
            >
              <Trash2 class="h-3.5 w-3.5" />
            </Button>
          </div>
          <div class="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <span>{{ item.provider_name || t('chart.unknown') }}</span>
            <span>·</span>
            <span class="truncate max-w-[100px]">{{ affinityModelLabel(item) }}</span>
            <span>·</span>
            <Badge
              variant="outline"
              class="text-[10px] px-1.5 font-normal"
            >
              {{ formatClientFamily(item.client_family) }}
            </Badge>
          </div>
          <div class="flex items-center justify-between text-xs">
            <span class="text-muted-foreground">{{ formatApiFormat(item.api_format) }} · {{ providerKeyLabel(item) }}</span>
            <span>{{ getRemainingTime(item.expire_at) }} · {{ formatAffinityRequestCountUnit(item) }}</span>
          </div>
        </div>
      </div>
      <div
        v-else-if="!listLoading && affinityList.length === 0"
        class="xl:hidden text-center py-6 text-sm text-muted-foreground"
      >
        {{ t('ui.noCacheRecords') }}
      </div>

      <Pagination
        v-if="affinityList.length > 0"
        :current="currentPage"
        :total="affinityList.length"
        :page-size="pageSize"
        cache-key="cache-monitoring-page-size"
        @update:current="currentPage = $event; handlePageChange()"
        @update:page-size="pageSize = $event"
      />
    </Card>

    <!-- 模型映射缓存管理 -->
    <Card class="overflow-hidden">
      <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <div class="flex items-center gap-3 shrink-0">
            <Database class="h-5 w-5 text-muted-foreground hidden sm:block" />
            <h3 class="text-sm sm:text-base font-semibold">
              {{ t('ui.modelMappingCache') }}
            </h3>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8 text-muted-foreground/70 hover:text-destructive"
              :title="t('ui.clearAllMappingCaches')"
              :disabled="clearingModelMapping || !modelMappingStats?.available"
              @click="clearAllModelMappingCache"
            >
              <Eraser class="h-4 w-4" />
            </Button>
            <RefreshButton
              :loading="modelMappingLoading"
              @click="fetchModelMappingStats"
            />
          </div>
        </div>
      </div>

      <!-- 映射缓存表格 -->
      <Table
        v-if="modelMappingStats?.available && modelMappingStats.mappings && modelMappingStats.mappings.length > 0"
        class="hidden md:table"
      >
        <TableHeader>
          <TableRow>
            <TableHead class="w-[25%]">
              {{ t('ui.globalModel') }}
            </TableHead>
            <TableHead class="w-8 text-center" />
            <TableHead class="w-[30%]">
              {{ t('ui.mappedModel') }}
            </TableHead>
            <TableHead class="w-[25%]">
              {{ t('nav.providers') }}
            </TableHead>
            <TableHead class="w-[10%] text-center">
              {{ t('ui.remainingd6822b0') }}
            </TableHead>
            <TableHead class="w-[5%] text-right">
              {{ t('ui.actionsed31fbb') }}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow
            v-for="mapping in modelMappingStats.mappings"
            :key="mapping.mapping_name"
          >
            <TableCell>
              <div v-if="mapping.global_model_name">
                <div class="text-sm font-medium">
                  {{ mapping.global_model_display_name || mapping.global_model_name }}
                </div>
                <div
                  v-if="mapping.global_model_display_name && mapping.global_model_display_name !== mapping.global_model_name"
                  class="text-xs text-muted-foreground font-mono"
                >
                  {{ mapping.global_model_name }}
                </div>
              </div>
              <span
                v-else
                class="text-sm text-muted-foreground"
              >-</span>
            </TableCell>
            <TableCell class="text-center">
              <ArrowRight class="h-4 w-4 text-muted-foreground" />
            </TableCell>
            <TableCell>
              <span class="text-sm font-mono">{{ mapping.mapping_name }}</span>
            </TableCell>
            <TableCell>
              <div
                v-if="mapping.providers && mapping.providers.length > 0"
                class="flex flex-wrap gap-1"
              >
                <Badge
                  v-for="provider in mapping.providers.slice(0, 3)"
                  :key="provider"
                  variant="outline"
                  class="text-xs"
                >
                  {{ provider }}
                </Badge>
                <Badge
                  v-if="mapping.providers.length > 3"
                  variant="outline"
                  class="text-xs"
                >
                  +{{ mapping.providers.length - 3 }}
                </Badge>
              </div>
              <span
                v-else
                class="text-sm text-muted-foreground"
              >-</span>
            </TableCell>
            <TableCell class="text-center">
              <span class="text-xs text-muted-foreground">{{ formatTTL(mapping.ttl) }}</span>
            </TableCell>
            <TableCell class="text-right">
              <Button
                size="icon"
                variant="ghost"
                class="h-6 w-6 text-muted-foreground/50 hover:text-destructive"
                :disabled="clearingModelName === mapping.mapping_name"
                :title="t('ui.clearCache')"
                @click="clearModelMappingByName(mapping.mapping_name)"
              >
                <X class="h-3.5 w-3.5" />
              </Button>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>

      <!-- 移动端卡片列表 -->
      <div
        v-if="modelMappingStats?.available && modelMappingStats.mappings && modelMappingStats.mappings.length > 0"
        class="md:hidden divide-y divide-border/40"
      >
        <div
          v-for="mapping in modelMappingStats.mappings"
          :key="`m-${mapping.mapping_name}`"
          class="p-4 space-y-2"
        >
          <div class="flex items-center justify-between gap-2">
            <span class="text-sm font-medium truncate">{{ mapping.global_model_display_name || mapping.global_model_name || '-' }}</span>
            <Button
              size="icon"
              variant="ghost"
              class="h-6 w-6 text-muted-foreground/50 hover:text-destructive shrink-0"
              :disabled="clearingModelName === mapping.mapping_name"
              @click="clearModelMappingByName(mapping.mapping_name)"
            >
              <X class="h-3.5 w-3.5" />
            </Button>
          </div>
          <div class="flex items-center gap-2 text-xs text-muted-foreground">
            <ArrowRight class="h-3.5 w-3.5 shrink-0" />
            <span class="font-mono">{{ mapping.mapping_name }}</span>
          </div>
          <div
            v-if="mapping.providers && mapping.providers.length > 0"
            class="flex flex-wrap gap-1"
          >
            <Badge
              v-for="provider in mapping.providers"
              :key="provider"
              variant="outline"
              class="text-xs"
            >
              {{ provider }}
            </Badge>
          </div>
        </div>
      </div>

      <!-- 未映射条目（NOT_FOUND 等） -->
      <div
        v-if="modelMappingStats?.available && modelMappingStats.unmapped && modelMappingStats.unmapped.length > 0"
        class="px-6 py-4 border-t border-border/40"
      >
        <div class="text-xs text-muted-foreground mb-2">
          {{ t('ui.unmappedCacheEntries') }}
        </div>
        <div class="flex flex-wrap gap-1.5">
          <Badge
            v-for="entry in modelMappingStats.unmapped"
            :key="entry.mapping_name"
            :variant="getUnmappedStatusBadge(entry.status).variant"
            class="text-xs font-mono cursor-pointer"
            :title="t('ui.valueClickToClear', { value: getUnmappedStatusBadge(entry.status).text })"
            @click="clearModelMappingByName(entry.mapping_name)"
          >
            {{ entry.mapping_name }}
          </Badge>
        </div>
      </div>

      <!-- Provider 模型映射缓存 -->
      <div
        v-if="modelMappingStats?.available && modelMappingStats.provider_model_mappings && modelMappingStats.provider_model_mappings.length > 0"
        class="border-t border-border/40"
      >
        <div class="px-6 py-3 text-xs text-muted-foreground border-b border-border/30 bg-muted/20">
          {{ t('ui.providerModelMappingCache') }}
        </div>
        <!-- 桌面端表格 -->
        <Table class="hidden md:table">
          <TableHeader>
            <TableRow>
              <TableHead class="w-[15%]">
                {{ t('nav.providers') }}
              </TableHead>
              <TableHead class="w-[25%]">
                {{ t('ui.requestedName') }}
              </TableHead>
              <TableHead class="w-8 text-center" />
              <TableHead class="w-[25%]">
                {{ t('ui.mappedModel') }}
              </TableHead>
              <TableHead class="w-[10%] text-center">
                {{ t('ui.remainingd6822b0') }}
              </TableHead>
              <TableHead class="w-[10%] text-center">
                {{ t('ui.count') }}
              </TableHead>
              <TableHead class="w-[7%] text-right">
                {{ t('ui.actionsed31fbb') }}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <template
              v-for="(mapping, index) in modelMappingStats.provider_model_mappings"
              :key="index"
            >
              <TableRow
                v-for="(alias, aliasIndex) in (mapping.aliases || [])"
                :key="`${index}-${aliasIndex}`"
              >
                <TableCell>
                  <Badge
                    variant="outline"
                    class="text-xs"
                  >
                    {{ mapping.provider_name }}
                  </Badge>
                </TableCell>
                <TableCell>
                  <span class="text-sm font-mono">{{ alias }}</span>
                </TableCell>
                <TableCell class="text-center">
                  <ArrowRight class="h-4 w-4 text-muted-foreground" />
                </TableCell>
                <TableCell>
                  <span class="text-sm font-mono font-medium">{{ mapping.provider_model_name }}</span>
                </TableCell>
                <TableCell class="text-center">
                  <span class="text-xs text-muted-foreground">{{ formatTTL(mapping.ttl) }}</span>
                </TableCell>
                <TableCell class="text-center">
                  <span class="text-sm">{{ mapping.hit_count || 0 }}</span>
                </TableCell>
                <TableCell class="text-right">
                  <Button
                    size="icon"
                    variant="ghost"
                    class="h-7 w-7 text-muted-foreground/70 hover:text-destructive"
                    :title="t('ui.clearCache')"
                    @click="clearProviderModelMapping(mapping.provider_id, mapping.global_model_id, `${mapping.provider_name} - ${alias}`)"
                  >
                    <Trash2 class="h-3.5 w-3.5" />
                  </Button>
                </TableCell>
              </TableRow>
            </template>
          </TableBody>
        </Table>
        <!-- 移动端卡片 -->
        <div class="md:hidden divide-y divide-border/40">
          <template
            v-for="(mapping, index) in modelMappingStats.provider_model_mappings"
            :key="`m-pm-${index}`"
          >
            <div
              v-for="(alias, aliasIndex) in (mapping.aliases || [])"
              :key="`m-pm-${index}-${aliasIndex}`"
              class="p-4 space-y-2"
            >
              <div class="flex items-center justify-between">
                <Badge
                  variant="outline"
                  class="text-xs"
                >
                  {{ mapping.provider_name }}
                </Badge>
                <div class="flex items-center gap-2">
                  <span class="text-xs text-muted-foreground">{{ formatTTL(mapping.ttl) }}</span>
                  <span class="text-xs">{{ mapping.hit_count || 0 }}{{ t('ui.times') }}</span>
                  <Button
                    size="icon"
                    variant="ghost"
                    class="h-6 w-6 text-muted-foreground/70 hover:text-destructive"
                    :title="t('ui.clearCache')"
                    @click="clearProviderModelMapping(mapping.provider_id, mapping.global_model_id, `${mapping.provider_name} - ${alias}`)"
                  >
                    <Trash2 class="h-3 w-3" />
                  </Button>
                </div>
              </div>
              <div class="flex items-center gap-2 text-sm">
                <span class="font-mono">{{ alias }}</span>
                <ArrowRight class="h-3.5 w-3.5 shrink-0 text-muted-foreground/60" />
                <span class="font-mono font-medium">{{ mapping.provider_model_name }}</span>
              </div>
            </div>
          </template>
        </div>
      </div>

      <!-- 无缓存状态 -->
      <div
        v-else-if="modelMappingStats?.available && (!modelMappingStats.mappings || modelMappingStats.mappings.length === 0) && (!modelMappingStats.unmapped || modelMappingStats.unmapped.length === 0) && (!modelMappingStats.provider_model_mappings || modelMappingStats.provider_model_mappings.length === 0)"
        class="px-6 py-8 text-center text-sm text-muted-foreground"
      >
        {{ t('ui.noModelResolutionCache') }}
      </div>

      <!-- Redis 未启用 -->
      <div
        v-else-if="modelMappingStats && !modelMappingStats.available"
        class="px-6 py-8 text-center text-sm text-muted-foreground"
      >
        {{ modelMappingStats.message || t('ui.redisIsDisabled') }}
      </div>

      <!-- 加载中 -->
      <div
        v-else-if="modelMappingLoading"
        class="px-6 py-8 text-center text-sm text-muted-foreground"
      >
        {{ t('common.loading') }}
      </div>
    </Card>

    <!-- Redis 缓存分类管理 -->
    <Card class="overflow-hidden">
      <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <div class="flex items-center gap-3 shrink-0">
            <HardDrive class="h-5 w-5 text-muted-foreground hidden sm:block" />
            <h3 class="text-sm sm:text-base font-semibold">
              {{ t('ui.redisCacheManagement') }}
            </h3>
            <Badge
              v-if="redisCacheData?.total_keys !== undefined"
              variant="secondary"
            >
              {{ redisCacheData.total_keys }} {{ t('ui.keys') }}
            </Badge>
          </div>
          <div class="flex items-center gap-2">
            <RefreshButton
              :loading="redisCacheLoading"
              size="sm"
              :title="t('ui.refreshCacheCategories')"
              @click="fetchRedisCacheCategories"
            />
          </div>
        </div>
      </div>

      <!-- 有数据 -->
      <div v-if="redisCacheData?.available && redisCacheData.categories.length > 0">
        <!-- 有缓存的分类 -->
        <div
          v-if="redisCategoriesWithKeys.length > 0"
          class="divide-y divide-border/40"
        >
          <div
            v-for="cat in redisCategoriesWithKeys"
            :key="cat.key"
            class="flex items-center justify-between px-4 sm:px-6 py-2.5 hover:bg-muted/30 transition-colors"
          >
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium">{{ cat.name }}</span>
                <Badge variant="outline">
                  {{ cat.count }}
                </Badge>
              </div>
              <p class="text-xs text-muted-foreground mt-0.5 truncate">
                {{ cat.description }}
              </p>
            </div>
            <Button
              variant="ghost"
              size="sm"
              class="shrink-0 ml-3 text-destructive hover:text-destructive hover:bg-destructive/10"
              :disabled="clearingCategory === cat.key"
              :title="t('ui.clearThisCacheCategory')"
              @click="clearRedisCategory(cat.key, cat.name, cat.count)"
            >
              <Trash2 class="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>

        <!-- 空分类折叠 -->
        <div
          v-if="redisCategoriesEmpty.length > 0"
          class="px-4 sm:px-6 py-3 border-t border-border/40"
        >
          <p class="text-xs text-muted-foreground">
            {{ t('ui.another') }} {{ redisCategoriesEmpty.length }} {{ t('ui.categoriesAreEmpty') }}{{ redisCategoriesEmpty.map(c => c.name).join('、') }}
          </p>
        </div>

        <!-- 全部为空 -->
        <div
          v-if="redisCategoriesWithKeys.length === 0"
          class="px-6 py-8 text-center"
        >
          <HardDrive class="h-10 w-10 text-muted-foreground/30 mx-auto mb-2" />
          <p class="text-sm text-muted-foreground">
            {{ t('ui.allCacheCategoriesAreEmpty') }}
          </p>
        </div>
      </div>

      <!-- Redis 不可用 -->
      <div
        v-else-if="redisCacheData && !redisCacheData.available"
        class="px-6 py-8 text-center text-sm text-muted-foreground"
      >
        {{ redisCacheData.message || t('ui.redisIsDisabled') }}
      </div>

      <!-- 加载中 -->
      <div
        v-else-if="redisCacheLoading"
        class="px-6 py-8 text-center text-sm text-muted-foreground"
      >
        {{ t('ui.scanningRedisCache') }}
      </div>
    </Card>

    <!-- TTL 分析区域 -->
    <Card class="overflow-hidden">
      <div
        ref="analysisSectionRef"
        class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60"
      >
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <div class="flex items-center gap-3 shrink-0">
            <BarChart3 class="h-5 w-5 text-muted-foreground hidden sm:block" />
            <h3 class="text-sm sm:text-base font-semibold">
              {{ t('ui.ttlAnalysis') }}
            </h3>
            <span class="text-xs text-muted-foreground hidden sm:inline">{{ t('ui.analyzeUserRequestIntervalsAndRecommendAnAppropriateCacheTtl') }}</span>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <Select
              v-model="analysisHours"
            >
              <SelectTrigger class="w-24 sm:w-28 h-8">
                <SelectValue :placeholder="t('ui.timePeriod')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="option in ANALYSIS_HOURS_OPTIONS"
                  :key="option.value"
                  :value="option.value"
                >
                  {{ option.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </div>

      <!-- 缓存命中概览 -->
      <div
        v-if="hitAnalysis"
        class="px-6 py-4 border-b border-border/40 bg-muted/30"
      >
        <div class="grid grid-cols-2 md:grid-cols-5 gap-6">
          <div>
            <div class="text-xs text-muted-foreground">
              {{ t('ui.requestHitRate') }}
            </div>
            <div class="text-2xl font-bold text-success">
              {{ hitAnalysis.request_cache_hit_rate }}%
            </div>
            <div class="text-xs text-muted-foreground">
              {{ formatNumber(hitAnalysis.requests_with_cache_hit) }} / {{ formatNumber(hitAnalysis.total_requests) }} {{ t('ui.request') }}
            </div>
          </div>
          <div>
            <div class="text-xs text-muted-foreground">
              {{ t('ui.tokenHitRate') }}
            </div>
            <div class="text-2xl font-bold">
              {{ hitAnalysis.token_cache_hit_rate }}%
            </div>
            <div class="text-xs text-muted-foreground">
              {{ formatTokens(hitAnalysis.total_cache_read_tokens) }} {{ t('ui.tokensHit') }}
            </div>
          </div>
          <div>
            <div class="text-xs text-muted-foreground">
              {{ t('ui.cacheWriteCost2dd12ec') }}
            </div>
            <div class="text-2xl font-bold">
              {{ formatCost(hitAnalysis.total_cache_creation_cost_usd) }}
            </div>
            <div class="text-xs text-muted-foreground">
              {{ formatTokens(hitAnalysis.total_cache_creation_tokens) }} tokens
            </div>
          </div>
          <div>
            <div class="text-xs text-muted-foreground">
              {{ t('ui.cacheReadCost7694fad') }}
            </div>
            <div class="text-2xl font-bold">
              {{ formatCost(hitAnalysis.total_cache_read_cost_usd) }}
            </div>
            <div class="text-xs text-muted-foreground">
              {{ formatTokens(hitAnalysis.total_cache_read_tokens) }} tokens
            </div>
          </div>
          <div>
            <div class="text-xs text-muted-foreground">
              {{ t('ui.estimatedSavings') }}
            </div>
            <div class="text-2xl font-bold text-success">
              {{ formatCost(hitAnalysis.estimated_savings_usd) }}
            </div>
          </div>
        </div>
      </div>

      <!-- 用户 TTL 分析表格 -->
      <Table v-if="ttlAnalysis && ttlAnalysis.users.length > 0">
        <TableHeader>
          <TableRow>
            <TableHead class="w-10" />
            <TableHead class="w-[20%]">
              {{ t('auth.role.user') }}
            </TableHead>
            <TableHead class="w-[15%] text-center">
              {{ t('stats.metric.requests') }}
            </TableHead>
            <TableHead class="w-[15%] text-center">
              {{ t('ui.usageFrequency') }}
            </TableHead>
            <TableHead class="w-[15%] text-center">
              {{ t('ui.recommendedTtl') }}
            </TableHead>
            <TableHead>{{ t('ui.description') }}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <template
            v-for="user in ttlAnalysis.users"
            :key="user.group_id"
          >
            <TableRow
              class="cursor-pointer hover:bg-muted/50"
              @click="toggleUserExpand(user.group_id)"
            >
              <TableCell class="p-2">
                <button class="p-1 hover:bg-muted rounded">
                  <ChevronDown
                    v-if="expandedUserId === user.group_id"
                    class="h-4 w-4 text-muted-foreground"
                  />
                  <ChevronRight
                    v-else
                    class="h-4 w-4 text-muted-foreground"
                  />
                </button>
              </TableCell>
              <TableCell>
                <span class="text-sm font-medium">{{ user.username || t('ui.unknownUser') }}</span>
              </TableCell>
              <TableCell class="text-center">
                <span class="text-sm font-medium">{{ user.request_count }}</span>
              </TableCell>
              <TableCell class="text-center">
                <span
                  class="text-sm"
                  :class="getFrequencyClass(user.recommended_ttl_minutes)"
                >
                  {{ getFrequencyLabel(user.recommended_ttl_minutes) }}
                </span>
              </TableCell>
              <TableCell class="text-center">
                <Badge :variant="getTTLBadgeVariant(user.recommended_ttl_minutes)">
                  {{ user.recommended_ttl_minutes }} {{ t('ui.minutesbd957bc') }}
                </Badge>
              </TableCell>
              <TableCell>
                <span class="text-xs text-muted-foreground">
                  {{ formatIntervalDescription(user) }}
                </span>
              </TableCell>
            </TableRow>
            <!-- 展开行：显示用户散点图 -->
            <TableRow
              v-if="expandedUserId === user.group_id"
              class="bg-muted/30"
            >
              <TableCell
                colspan="6"
                class="p-0"
              >
                <div class="px-6 py-4">
                  <div class="flex items-center justify-between mb-3">
                    <h4 class="text-sm font-medium">
                      {{ t('ui.requestIntervalTimeline') }}
                    </h4>
                    <div class="flex items-center gap-3 text-xs text-muted-foreground">
                      <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-green-500" /> {{ t('ui.05Minutes') }}</span>
                      <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-blue-500" /> {{ t('ui.515Minutes') }}</span>
                      <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-purple-500" /> {{ t('ui.1530Minutes') }}</span>
                      <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-orange-500" /> {{ t('ui.3060Minutes') }}</span>
                      <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-red-500" /> {{ t('ui.60Minutes') }}</span>
                      <span
                        v-if="userTimelineData"
                        class="ml-2"
                      >{{ t('ui.total76e547a') }} {{ userTimelineData.total_points }} {{ t('ui.dataPoints') }}</span>
                    </div>
                  </div>
                  <div
                    v-if="userTimelineLoading"
                    class="h-64 flex items-center justify-center"
                  >
                    <span class="text-sm text-muted-foreground">{{ t('common.loading') }}</span>
                  </div>
                  <div
                    v-else-if="userTimelineData && userTimelineData.points.length > 0"
                    class="h-64"
                  >
                    <ScatterChart :data="userTimelineChartData" />
                  </div>
                  <div
                    v-else
                    class="h-64 flex items-center justify-center"
                  >
                    <span class="text-sm text-muted-foreground">{{ t('stats.empty.title') }}</span>
                  </div>
                </div>
              </TableCell>
            </TableRow>
          </template>
        </TableBody>
      </Table>

      <!-- 分析完成但无数据 -->
      <div
        v-else-if="ttlAnalysis && ttlAnalysis.users.length === 0"
        class="px-6 py-12 text-center"
      >
        <BarChart3 class="h-12 w-12 text-muted-foreground/50 mx-auto mb-3" />
        <p class="text-sm text-muted-foreground">
          {{ t('ui.noUserDataMatchesTheseCriteria') }}
        </p>
        <p class="text-xs text-muted-foreground mt-1">
          {{ t('ui.tryIncreasingTheAnalysisPeriodOrLoweringTheMinimumRequest') }}
        </p>
      </div>

      <!-- 加载中 -->
      <div
        v-else-if="ttlAnalysisLoading"
        class="px-6 py-12 text-center"
      >
        <p class="text-sm text-muted-foreground">
          {{ t('ui.analyzingUserRequestData') }}
        </p>
      </div>
    </Card>
  </div>
</template>
