<template>
  <div class="space-y-6 pb-8">
    <!-- 统计卡片 -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
      <Card
        variant="default"
        class="p-4"
      >
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center">
            <FileUp class="w-5 h-5 text-primary" />
          </div>
          <div>
            <p class="text-2xl font-bold">
              {{ stats?.total_mappings ?? '-' }}
            </p>
            <p class="text-xs text-muted-foreground">
              {{ t('ui.totalFiles') }}
            </p>
          </div>
        </div>
      </Card>
      <Card
        variant="default"
        class="p-4"
      >
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg bg-green-500/10 flex items-center justify-center">
            <CheckCircle class="w-5 h-5 text-green-500" />
          </div>
          <div>
            <p class="text-2xl font-bold">
              {{ stats?.active_mappings ?? '-' }}
            </p>
            <p class="text-xs text-muted-foreground">
              {{ t('ui.activeFiles') }}
            </p>
          </div>
        </div>
      </Card>
      <Card
        variant="default"
        class="p-4"
      >
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg bg-amber-500/10 flex items-center justify-center">
            <Clock class="w-5 h-5 text-amber-500" />
          </div>
          <div>
            <p class="text-2xl font-bold">
              {{ stats?.expired_mappings ?? '-' }}
            </p>
            <p class="text-xs text-muted-foreground">
              {{ t('ui.expired') }}
            </p>
          </div>
        </div>
      </Card>
      <Card
        variant="default"
        class="p-4"
      >
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center">
            <Key class="w-5 h-5 text-blue-500" />
          </div>
          <div>
            <p class="text-2xl font-bold">
              {{ stats?.capable_keys_count ?? '-' }}
            </p>
            <p class="text-xs text-muted-foreground">
              {{ t('ui.supportedKeys') }}
            </p>
          </div>
        </div>
      </Card>
    </div>

    <!-- 上传区域 -->
    <Card
      variant="default"
      class="p-4"
    >
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-sm font-medium">
          {{ t('ui.uploadFiles') }}
        </h3>
        <Button
          v-if="capableKeys.length > 0"
          variant="ghost"
          size="sm"
          class="h-7 text-xs"
          @click="toggleSelectAll"
        >
          {{ selectedKeyIds.length === capableKeys.length ? t('ui.deselectAll') : t('ui.selectAll') }}
        </Button>
      </div>

      <!-- Key 选择器 -->
      <div
        v-if="capableKeys.length > 0"
        class="mb-4"
      >
        <p class="text-xs text-muted-foreground mb-2">
          {{ t('ui.selectKeysToUploadToMultipleAllowed') }}
        </p>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="key in capableKeys"
            :key="key.id"
            class="px-3 py-1.5 text-xs rounded-lg border transition-colors"
            :class="selectedKeyIds.includes(key.id)
              ? 'border-primary bg-primary/10 text-primary'
              : 'border-border hover:border-primary/50'"
            @click="toggleKeySelection(key.id)"
          >
            <span class="font-medium">{{ key.name }}</span>
            <span
              v-if="key.provider_name"
              class="text-muted-foreground ml-1"
            >({{ key.provider_name }})</span>
          </button>
        </div>
      </div>
      <div
        v-else
        class="mb-4 text-sm text-amber-600 bg-amber-50 dark:bg-amber-950/30 rounded-lg p-3"
      >
        {{ t('ui.noAvailableKeysConfigureAKeyWithGeminiFilesApi') }}
      </div>

      <!-- 拖拽上传区 -->
      <div
        class="border-2 border-dashed border-border/60 rounded-lg p-6 text-center transition-colors"
        :class="{
          'border-primary bg-primary/5': isDragging,
          'hover:border-primary/50': !isDragging && !uploading && selectedKeyIds.length > 0,
          'opacity-50 cursor-not-allowed': selectedKeyIds.length === 0
        }"
        @dragover.prevent="isDragging = true"
        @dragleave.prevent="isDragging = false"
        @drop.prevent="handleDrop"
      >
        <input
          ref="fileInputRef"
          type="file"
          class="hidden"
          @change="handleFileSelect"
        >
        <div
          v-if="uploading"
          class="flex flex-col items-center gap-2"
        >
          <Loader2 class="w-8 h-8 animate-spin text-primary" />
          <p class="text-sm text-muted-foreground">
            {{ t('ui.uploadingTo') }} {{ selectedKeyIds.length }} {{ t('ui.keys1490083') }}
          </p>
        </div>
        <div
          v-else
          class="flex flex-col items-center gap-2"
        >
          <Upload class="w-8 h-8 text-muted-foreground" />
          <p class="text-sm text-muted-foreground">
            <template v-if="selectedKeyIds.length > 0">
              {{ t('ui.dragFilesHereOr') }} <button
                class="text-primary hover:underline"
                @click="fileInputRef?.click()"
              >
                {{ t('ui.clickToSelect') }}
              </button>
            </template>
            <template v-else>
              {{ t('ui.selectAtLeastOneKeyFirst') }}
            </template>
          </p>
          <p class="text-xs text-muted-foreground">
            {{ t('ui.supportsVideoImagesAudioDocumentsAndMoreMax2Gb') }}
          </p>
        </div>
      </div>
    </Card>

    <!-- MIME 类型分布 -->
    <Card
      v-if="stats?.by_mime_type && Object.keys(stats.by_mime_type).length > 0"
      variant="default"
      class="p-4"
    >
      <h3 class="text-sm font-medium mb-3">
        {{ t('ui.fileTypeDistribution') }}
      </h3>
      <div class="flex flex-wrap gap-2">
        <Badge
          v-for="(count, mimeType) in stats.by_mime_type"
          :key="mimeType"
          variant="secondary"
          class="text-xs"
        >
          {{ mimeType }}: {{ count }}
        </Badge>
      </div>
    </Card>

    <!-- 文件映射表格 -->
    <Card
      variant="default"
      class="overflow-hidden"
    >
      <!-- 标题和筛选器 -->
      <div class="px-4 sm:px-6 py-3.5 border-b border-border/60">
        <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <h3 class="text-base font-semibold">
            {{ t('ui.fileMappings') }}
          </h3>
          <div class="flex items-center gap-2">
            <!-- 搜索 -->
            <Input
              v-model="searchQuery"
              type="text"
              :placeholder="t('ui.searchFileName')"
              class="w-40 h-8 text-xs"
            />
            <!-- 包含过期 -->
            <label class="flex items-center gap-1.5 text-xs text-muted-foreground cursor-pointer">
              <input
                v-model="includeExpired"
                type="checkbox"
                class="rounded border-border"
              > {{ t('ui.includeExpired') }} </label>
            <!-- 清理过期按钮 -->
            <Button
              variant="outline"
              size="sm"
              class="h-8 text-xs"
              :disabled="loading || (stats?.expired_mappings ?? 0) === 0"
              @click="cleanupExpired"
            >
              <Trash2 class="w-3 h-3 mr-1" /> {{ t('ui.cleanExpired') }}
            </Button>
            <!-- 刷新按钮 -->
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8"
              :disabled="loading"
              @click="fetchData"
            >
              <RefreshCw
                class="w-3.5 h-3.5"
                :class="{ 'animate-spin': loading }"
              />
            </Button>
          </div>
        </div>
      </div>

      <!-- 加载状态 -->
      <div
        v-if="loading && !mappings.length"
        class="p-8 text-center"
      >
        <Loader2 class="w-8 h-8 animate-spin mx-auto text-muted-foreground" />
        <p class="mt-2 text-sm text-muted-foreground">
          {{ t('common.loading') }}
        </p>
      </div>

      <!-- 空状态 -->
      <div
        v-else-if="!mappings.length"
        class="p-8 text-center"
      >
        <FileUp class="w-12 h-12 mx-auto text-muted-foreground/50" />
        <p class="mt-2 text-sm text-muted-foreground">
          {{ t('ui.noFileMappings') }}
        </p>
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.filesUploadedThroughTheGeminiFilesApiAppearHere') }}
        </p>
      </div>

      <!-- 文件列表 -->
      <div
        v-else
        class="divide-y divide-border/60"
      >
        <div
          v-for="mapping in mappings"
          :key="mapping.id"
          class="px-4 sm:px-6 py-4 hover:bg-muted/30 transition-colors"
          :class="{ 'opacity-50': mapping.is_expired }"
        >
          <div class="flex items-start justify-between gap-4">
            <div class="flex-1 min-w-0">
              <!-- 文件名和状态 -->
              <div class="flex items-center gap-2 mb-1">
                <component
                  :is="getFileIcon(mapping.mime_type)"
                  class="w-4 h-4 text-muted-foreground"
                />
                <span class="font-mono text-sm font-medium">{{ mapping.file_name }}</span>
                <Badge
                  v-if="mapping.is_expired"
                  variant="secondary"
                  class="text-xs"
                >
                  {{ t('ui.expired') }}
                </Badge>
                <Badge
                  v-else
                  variant="outline"
                  class="text-xs text-green-600"
                >
                  {{ t('ui.valid') }}
                </Badge>
              </div>
              <!-- 显示名 -->
              <p
                v-if="mapping.display_name"
                class="text-sm text-muted-foreground truncate"
              >
                {{ mapping.display_name }}
              </p>
              <!-- 元信息 -->
              <div class="flex items-center gap-4 mt-2 text-xs text-muted-foreground">
                <span
                  v-if="mapping.mime_type"
                  class="flex items-center gap-1"
                >
                  <File class="w-3 h-3" />
                  {{ mapping.mime_type }}
                </span>
                <span
                  v-if="mapping.username"
                  class="flex items-center gap-1"
                >
                  <User class="w-3 h-3" />
                  {{ mapping.username }}
                </span>
                <span
                  v-if="mapping.key_name"
                  class="flex items-center gap-1"
                >
                  <Key class="w-3 h-3" />
                  {{ mapping.key_name }}
                </span>
                <span class="flex items-center gap-1">
                  <Clock class="w-3 h-3" />
                  {{ formatDateTimeCompact(mapping.created_at) }}
                </span>
                <span
                  class="flex items-center gap-1"
                  :class="{ 'text-red-500': mapping.is_expired }"
                >
                  <Timer class="w-3 h-3" /> {{ t('ui.expires') }} {{ formatDateTimeCompact(mapping.expires_at) }}
                </span>
              </div>
            </div>
            <!-- 操作 -->
            <div class="flex items-center gap-2">
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 text-muted-foreground hover:text-red-500"
                :title="t('ui.deleteMapping')"
                @click.stop="deleteMapping(mapping)"
              >
                <Trash2 class="w-4 h-4" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      <!-- 分页 -->
      <div
        v-if="totalPages > 1"
        class="px-4 sm:px-6 py-3 border-t border-border/60 flex items-center justify-between"
      >
        <p class="text-xs text-muted-foreground">
          {{ t('ui.total76e547a') }} {{ total }} {{ t('ui.records6016bf6') }}
        </p>
        <div class="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            :disabled="currentPage <= 1"
            @click="currentPage--"
          >
            {{ t('userStats.pagination.previous') }}
          </Button>
          <span class="text-sm text-muted-foreground">
            {{ currentPage }} / {{ totalPages }}
          </span>
          <Button
            variant="outline"
            size="sm"
            :disabled="currentPage >= totalPages"
            @click="currentPage++"
          >
            {{ t('userStats.pagination.next') }}
          </Button>
        </div>
      </div>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { ref, computed, watch, onMounted } from 'vue'
import { useToast } from '@/composables/useToast'
import Card from '@/components/ui/card.vue'
import Badge from '@/components/ui/badge.vue'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import {
  FileUp,
  CheckCircle,
  Clock,
  Key,
  RefreshCw,
  Loader2,
  Trash2,
  File,
  User,
  Timer,
  Video,
  Image,
  FileText,
  Music,
  Upload
} from 'lucide-vue-next'
import { geminiFilesApi, type FileMappingStatsResponse, type FileMappingResponse, type CapableKeyResponse } from '@/api/gemini-files'
import { parseApiError } from '@/utils/errorParser'
import { log } from '@/utils/logger'
import { formatDateTimeCompact } from '@/utils/format'
import { useConfirm } from '@/composables/useConfirm'

const { toast } = useToast()
const { confirmDanger } = useConfirm()

// 状态
const loading = ref(false)
const stats = ref<FileMappingStatsResponse | null>(null)
const mappings = ref<FileMappingResponse[]>([])
const total = ref(0)
const currentPage = ref(1)
const pageSize = 20
const searchQuery = ref('')
const includeExpired = ref(false)

// 上传状态
const uploading = ref(false)
const isDragging = ref(false)
const fileInputRef = ref<HTMLInputElement | null>(null)
const capableKeys = ref<CapableKeyResponse[]>([])
const selectedKeyIds = ref<string[]>([])

// 计算属性
const totalPages = computed(() => Math.ceil(total.value / pageSize))

// 监听筛选条件变化
watch([searchQuery, includeExpired], () => {
  currentPage.value = 1
  fetchMappings()
})

watch(currentPage, () => {
  fetchMappings()
})

// 获取数据
async function fetchData() {
  await Promise.all([fetchStats(), fetchMappings(), fetchCapableKeys()])
}

async function fetchCapableKeys() {
  try {
    const keys = await geminiFilesApi.getCapableKeys()
    capableKeys.value = keys
    // 默认全选
    if (selectedKeyIds.value.length === 0 && keys.length > 0) {
      selectedKeyIds.value = keys.map(k => k.id)
    }
  } catch (error: unknown) {
    log.error('Failed to fetch capable keys', error)
  }
}

function toggleKeySelection(keyId: string) {
  const index = selectedKeyIds.value.indexOf(keyId)
  if (index === -1) {
    selectedKeyIds.value.push(keyId)
  } else {
    selectedKeyIds.value.splice(index, 1)
  }
}

function toggleSelectAll() {
  if (selectedKeyIds.value.length === capableKeys.value.length) {
    selectedKeyIds.value = []
  } else {
    selectedKeyIds.value = capableKeys.value.map(k => k.id)
  }
}

async function fetchStats() {
  try {
    const data = await geminiFilesApi.getStats()
    stats.value = data
  } catch (error: unknown) {
    toast({
      get title() { return t('ui.failedToLoadStatistics') },
      description: error instanceof Error ? error.message : String(error),
      variant: 'destructive'
    })
  }
}

async function fetchMappings() {
  loading.value = true
  try {
    const data = await geminiFilesApi.listMappings({
      page: currentPage.value,
      page_size: pageSize,
      include_expired: includeExpired.value,
      search: searchQuery.value || undefined
    })
    mappings.value = data.items
    total.value = data.total
  } catch (error: unknown) {
    toast({
      get title() { return t('ui.failedToLoadFileList') },
      description: error instanceof Error ? error.message : String(error),
      variant: 'destructive'
    })
  } finally {
    loading.value = false
  }
}

async function deleteMapping(mapping: FileMappingResponse) {
  if (!(await confirmDanger(t('ui.deleteMappingValueThisOnlyDeletesTheMappingRecordAnd', { name: mapping.file_name }), t('ui.deleteMapping')))) {
    return
  }

  try {
    await geminiFilesApi.deleteMapping(mapping.id)
    toast({
      get title() { return t('ui.deletedSuccessfully') },
      description: t('ui.deletedMappingValue', { name: mapping.file_name })
    })
    await fetchData()
  } catch (error: unknown) {
    toast({
      get title() { return t('ui.deletionFailed') },
      description: error instanceof Error ? error.message : String(error),
      variant: 'destructive'
    })
  }
}

async function cleanupExpired() {
  if (!(await confirmDanger(t('ui.cleanUpAllExpiredFileMappings'), '清理过期映射', '清理'))) {
    return
  }

  try {
    const result = await geminiFilesApi.cleanupExpired()
    toast({
      get title() { return t('ui.cleanupComplete') },
      description: t('ui.cleanedValueExpiredMappings', { count: result.deleted_count })
    })
    await fetchData()
  } catch (error: unknown) {
    toast({
      get title() { return t('ui.cleanupFailed') },
      description: error instanceof Error ? error.message : String(error),
      variant: 'destructive'
    })
  }
}

// 上传相关
async function uploadFile(file: globalThis.File) {
  if (selectedKeyIds.value.length === 0) {
    toast({
      get title() { return t('ui.selectAKey') },
      get description() { return t('ui.selectAtLeastOneKeyForTheFileUpload') },
      variant: 'destructive'
    })
    return
  }

  uploading.value = true
  let hasSuccess = false
  try {
    const result = await geminiFilesApi.uploadFile(file, selectedKeyIds.value)
    if (result.fail_count === 0) {
      toast({
        get title() { return t('ui.uploadSucceeded') },
        description: `文件 ${result.display_name} 已上传到 ${result.success_count} 个 Key`
      })
      hasSuccess = true
    } else if (result.success_count > 0) {
      toast({
        get title() { return t('ui.partiallySuccessful') },
        description: t('ui.succeededValueFailedValue6c5022e', { count: result.success_count, count2: result.fail_count })
      })
      hasSuccess = true
    } else {
      const errors = result.results.map(r => r.error).filter(Boolean).join('; ')
      toast({
        get title() { return t('ui.uploadFailed') },
        description: errors || '所有 Key 上传都失败了',
        variant: 'destructive'
      })
    }
  } catch (error: unknown) {
    toast({
      get title() { return t('ui.uploadFailed') },
      description: parseApiError(error, t('ui.uploadFailed')),
      variant: 'destructive'
    })
  } finally {
    uploading.value = false
    isDragging.value = false
    // 有成功上传时刷新列表，并重置到第一页
    if (hasSuccess) {
      currentPage.value = 1
      await fetchData()
    }
  }
}

function handleDrop(e: DragEvent) {
  isDragging.value = false
  const files = e.dataTransfer?.files
  if (files && files.length > 0) {
    uploadFile(files[0])
  }
}

function handleFileSelect(e: Event) {
  const input = e.target as HTMLInputElement
  if (input.files && input.files.length > 0) {
    uploadFile(input.files[0])
    input.value = '' // 清空以便重复选择同一文件
  }
}

function getFileIcon(mimeType: string | null) {
  if (!mimeType) return File
  if (mimeType.startsWith('video/')) return Video
  if (mimeType.startsWith('image/')) return Image
  if (mimeType.startsWith('audio/')) return Music
  if (mimeType.startsWith('text/') || mimeType.includes('pdf')) return FileText
  return File
}

// 初始化
onMounted(() => {
  fetchData()
})

</script>
