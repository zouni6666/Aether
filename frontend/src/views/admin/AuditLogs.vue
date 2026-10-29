<template>
  <div class="space-y-6 pb-8">
    <!-- 审计日志列表 -->
    <Card
      variant="default"
      class="overflow-hidden"
    >
      <!-- 标题和操作栏 -->
      <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <div class="shrink-0">
            <h3 class="text-sm sm:text-base font-semibold">
              {{ t('ui.auditLogs') }}
            </h3>
            <p class="text-xs text-muted-foreground mt-0.5">
              {{ t('ui.viewAllSystemOperationRecords') }}
            </p>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <!-- 搜索框 -->
            <div class="relative">
              <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground z-10 pointer-events-none" />
              <Input
                id="audit-logs-search"
                v-model="searchQuery"
                :placeholder="t('ui.searchUsernames')"
                class="w-32 sm:w-64 h-8 text-sm pl-8"
                @input="handleSearchChange"
              />
            </div>
            <!-- 分隔线 -->
            <div class="hidden sm:block h-4 w-px bg-border" />
            <!-- 事件类型筛选 -->
            <div class="xl:hidden">
              <Select
                v-model="filters.eventType"
                @update:model-value="handleEventTypeChange"
              >
                <SelectTrigger class="w-24 sm:w-40 h-8 border-border/60">
                  <SelectValue :placeholder="t('ui.allTypes')" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem
                    v-for="option in auditEventTypeFilterOptions"
                    :key="option.value"
                    :value="option.value"
                  >
                    {{ option.label }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            <!-- 时间范围筛选 -->
            <div class="xl:hidden">
              <Select
                v-model="filtersDaysString"
                @update:model-value="handleDaysChange"
              >
                <SelectTrigger class="w-20 sm:w-28 h-8 border-border/60">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem
                    v-for="option in auditDaysFilterOptions"
                    :key="option.value"
                    :value="option.value"
                  >
                    {{ option.label }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            <!-- 重置筛选 -->
            <Button
              v-if="hasActiveFilters"
              variant="ghost"
              size="icon"
              class="h-8 w-8"
              :title="t('ui.resetFilters')"
              @click="handleResetFilters"
            >
              <FilterX class="w-3.5 h-3.5" />
            </Button>
            <div class="hidden sm:block h-4 w-px bg-border" />
            <!-- 导出按钮 -->
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8"
              :title="t('ui.export')"
              @click="exportLogs"
            >
              <Download class="w-3.5 h-3.5" />
            </Button>
            <!-- 刷新按钮 -->
            <RefreshButton
              :loading="loading"
              @click="refreshLogs"
            />
          </div>
        </div>
      </div>

      <div
        v-if="loading"
        class="flex items-center justify-center py-12"
      >
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
      </div>

      <div
        v-else-if="logs.length === 0"
        class="text-center py-12 text-muted-foreground"
      >
        {{ t('ui.noAuditRecords') }}
      </div>

      <div v-else>
        <Table class="hidden xl:table">
          <TableHeader>
            <TableRow class="border-b border-border/60 hover:bg-transparent">
              <SortableTableHead
                class="h-12 font-semibold"
                column-key="created_at"
                :sortable="false"
                :filter-active="filters.days !== 7"
                :filter-title="t('ui.filterByTimeRange')"
                filter-content-class="w-32 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
              >
                {{ t('ui.time') }} <template #filter="{ close }">
                  <TableFilterMenu
                    :model-value="filtersDaysString"
                    :options="auditDaysFilterOptions"
                    @update:model-value="handleDaysChange"
                    @select="close"
                  />
                </template>
              </SortableTableHead>
              <TableHead class="h-12 font-semibold">
                {{ t('auth.role.user') }}
              </TableHead>
              <SortableTableHead
                class="h-12 font-semibold"
                column-key="event_type"
                :sortable="false"
                :filter-active="filters.eventType !== '__all__'"
                :filter-title="t('ui.filterByEventType')"
                filter-content-class="w-48 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
              >
                {{ t('ui.eventType') }} <template #filter="{ close }">
                  <TableFilterMenu
                    :model-value="filters.eventType"
                    :options="auditEventTypeFilterOptions"
                    @update:model-value="handleEventTypeChange"
                    @select="close"
                  />
                </template>
              </SortableTableHead>
              <TableHead class="h-12 font-semibold">
                {{ t('ui.descriptiondc2ba46') }}
              </TableHead>
              <TableHead class="h-12 font-semibold">
                {{ t('ui.ipAddressffcb085') }}
              </TableHead>
              <TableHead class="h-12 font-semibold">
                {{ t('ui.status6320b4a') }}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow
              v-for="entry in logs"
              :key="entry.id"
              class="cursor-pointer border-b border-border/40 hover:bg-muted/30 transition-colors"
              @mousedown="handleMouseDown"
              @click="handleRowClick($event, entry)"
            >
              <TableCell class="text-xs py-4">
                {{ formatDateTime(entry.created_at) }}
              </TableCell>

              <TableCell class="py-4">
                <div
                  v-if="entry.user_id"
                  class="flex flex-col"
                >
                  <span class="text-sm font-medium">
                    {{ entry.user_email || t('ui.userValue', { value: entry.user_id }) }}
                  </span>
                  <span
                    v-if="entry.user_username"
                    class="text-xs text-muted-foreground"
                  >
                    {{ entry.user_username }}
                  </span>
                </div>
                <span
                  v-else
                  class="text-muted-foreground italic"
                >{{ t('nav.group.system') }}</span>
              </TableCell>

              <TableCell class="py-4">
                <Badge :variant="getEventTypeBadgeVariant(entry.event_type)">
                  <component
                    :is="getEventTypeIcon(entry.event_type)"
                    class="h-3 w-3 mr-1"
                  />
                  {{ getEventTypeLabel(entry.event_type) }}
                </Badge>
              </TableCell>

              <TableCell
                class="max-w-xs truncate py-4"
                :title="entry.description"
              >
                {{ entry.description || t('ui.noDescription8ae96d7') }}
              </TableCell>

              <TableCell class="py-4">
                <span
                  v-if="entry.ip_address"
                  class="flex items-center text-sm"
                >
                  <Globe class="h-3 w-3 mr-1 text-muted-foreground" />
                  {{ entry.ip_address }}
                </span>
                <span v-else>-</span>
              </TableCell>

              <TableCell class="py-4">
                <Badge
                  v-if="entry.status_code"
                  :variant="getStatusCodeVariant(entry.status_code)"
                >
                  {{ entry.status_code }}
                </Badge>
                <span v-else>-</span>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>

        <!-- 移动端卡片列表 -->
        <div
          v-if="logs.length > 0"
          class="xl:hidden divide-y divide-border/40"
        >
          <div
            v-for="logItem in logs"
            :key="logItem.id"
            class="p-4 space-y-2 hover:bg-muted/30 cursor-pointer transition-colors"
            @click="showLogDetail(logItem)"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="flex-1 min-w-0">
                <Badge :variant="getEventTypeBadgeVariant(logItem.event_type)">
                  <component
                    :is="getEventTypeIcon(logItem.event_type)"
                    class="h-3 w-3 mr-1"
                  />
                  {{ getEventTypeLabel(logItem.event_type) }}
                </Badge>
                <div class="text-xs text-muted-foreground mt-1.5">
                  {{ formatDateTime(logItem.created_at) }}
                </div>
              </div>
              <Badge
                v-if="logItem.status_code"
                :variant="getStatusCodeVariant(logItem.status_code)"
                class="shrink-0"
              >
                {{ logItem.status_code }}
              </Badge>
            </div>
            <div
              v-if="logItem.user_id"
              class="text-sm"
            >
              {{ logItem.user_email || t('ui.userValue', { value: logItem.user_id }) }}
            </div>
            <div
              class="text-xs text-muted-foreground truncate"
              :title="logItem.description"
            >
              {{ logItem.description || t('ui.noDescription8ae96d7') }}
            </div>
            <div
              v-if="logItem.ip_address"
              class="flex items-center text-xs text-muted-foreground"
            >
              <Globe class="h-3 w-3 mr-1" />
              {{ logItem.ip_address }}
            </div>
          </div>
        </div>

        <!-- 分页控件 -->
        <Pagination
          :current="currentPage"
          :total="totalRecords"
          :page-size="pageSize"
          :page-size-options="[10, 20, 50, 100]"
          cache-key="audit-logs-page-size"
          @update:current="handlePageChange"
          @update:page-size="pageSize = $event; currentPage = 1; loadLogs()"
        />
      </div>
    </Card>

    <!-- 详情对话框 (使用shadcn Dialog组件) -->
    <div
      v-if="selectedLog"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click="closeLogDetail"
    >
      <Card
        class="max-w-2xl w-full mx-4 max-h-[80vh] overflow-y-auto"
        @click.stop
      >
        <div class="p-6">
          <div class="flex justify-between items-center mb-4">
            <h3 class="text-lg font-medium">
              {{ t('ui.auditLogDetails') }}
            </h3>
            <Button
              variant="ghost"
              size="sm"
              @click="closeLogDetail"
            >
              <X class="h-4 w-4" />
            </Button>
          </div>

          <div class="space-y-4">
            <div>
              <Label>{{ t('ui.eventType') }}</Label>
              <p class="mt-1 text-sm">
                {{ getEventTypeLabel(selectedLog.event_type) }}
              </p>
            </div>

            <Separator />

            <div>
              <Label>{{ t('ui.descriptiondc2ba46') }}</Label>
              <p class="mt-1 text-sm">
                {{ selectedLog.description || t('ui.noDescription8ae96d7') }}
              </p>
            </div>

            <div>
              <Label>{{ t('ui.time') }}</Label>
              <p class="mt-1 text-sm">
                {{ formatDateTime(selectedLog.created_at) }}
              </p>
            </div>

            <div v-if="selectedLog.user_id">
              <Label>{{ t('ui.userInformation') }}</Label>
              <div class="mt-1 text-sm">
                <p class="font-medium">
                  {{ selectedLog.user_email || t('ui.userValue', { value: selectedLog.user_id }) }}
                </p>
                <p
                  v-if="selectedLog.user_username"
                  class="text-muted-foreground"
                >
                  {{ selectedLog.user_username }}
                </p>
                <p class="text-xs text-muted-foreground">
                  ID: {{ selectedLog.user_id }}
                </p>
              </div>
            </div>

            <div v-if="selectedLog.ip_address">
              <Label>{{ t('ui.ipAddressffcb085') }}</Label>
              <p class="mt-1 text-sm">
                {{ selectedLog.ip_address }}
              </p>
            </div>

            <div v-if="selectedLog.status_code">
              <Label>{{ t('ui.statusCode') }}</Label>
              <p class="mt-1 text-sm">
                {{ selectedLog.status_code }}
              </p>
            </div>

            <div v-if="selectedLog.error_message">
              <Label>{{ t('ui.errorMessagea033ad5') }}</Label>
              <p class="mt-1 text-sm text-destructive">
                {{ selectedLog.error_message }}
              </p>
            </div>

            <div v-if="selectedLog.metadata">
              <Label>{{ t('ui.metadata') }}</Label>
              <pre class="mt-1 text-sm bg-muted p-3 rounded-md overflow-x-auto">{{ JSON.stringify(selectedLog.metadata, null, 2) }}</pre>
            </div>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { getI18nLocale } from '@/i18n'
import { ref, onMounted, onBeforeUnmount, computed } from 'vue'
import {
  Card,
  Button,
  Badge,
  Separator,
  Label,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  SortableTableHead,
  TableFilterMenu,
  TableCell,
  Input,
  Pagination,
  RefreshButton
} from '@/components/ui'
import { auditApi } from '@/api/audit'
import {
  Download,
  Shield,
  Key,
  Activity,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Globe,
  X,
  User,
  Settings,
  Search,
  FilterX
} from 'lucide-vue-next'

interface AuditLog {
  id: string
  event_type: string
  user_id?: number
  user_email?: string
  user_username?: string
  description: string
  ip_address?: string
  status_code?: number
  error_message?: string
  metadata?: Record<string, unknown>
  created_at: string
}

const loading = ref(false)
const logs = ref<AuditLog[]>([])
const selectedLog = ref<AuditLog | null>(null)
let logsRequestId = 0

// 搜索查询
const searchQuery = ref('')

const filters = ref({
  username: '',
  eventType: '__all__',
  days: 7,
  limit: 50
})

const filtersDaysString = ref('7')
const auditEventTypeFilterOptions = [
  { value: '__all__', get label() { return t('ui.allTypes') } },
  { value: 'login_success', get label() { return t('site.auth.success') } },
  { value: 'login_failed', get label() { return t('ui.signInFailed') } },
  { value: 'logout', get label() { return t('common.logout') } },
  { value: 'api_key_created', get label() { return t('ui.apiKeyCreated3aeb758') } },
  { value: 'api_key_deleted', get label() { return t('ui.apiKeyDeleted71d5c8a') } },
  { value: 'request_success', get label() { return t('ui.requestSucceeded') } },
  { value: 'request_failed', get label() { return t('ui.requestFailed') } },
  { value: 'user_created', get label() { return t('ui.usersCreated') } },
  { value: 'user_updated', get label() { return t('ui.userUpdated') } },
  { value: 'user_deleted', get label() { return t('ui.userDeleted7d21d97') } },
]
const auditDaysFilterOptions = [
  { value: '1', get label() { return t('ui.1Day') } },
  { value: '7', get label() { return t('ui.7Days') } },
  { value: '30', get label() { return t('ui.30Days') } },
  { value: '90', get label() { return t('ui.90Days') } },
]

const currentPage = ref(1)
const pageSize = ref(20)
const totalRecords = ref(0)

let loadTimeout: number | null = null
const debouncedLoadLogs = () => {
  if (loadTimeout !== null) clearTimeout(loadTimeout)
  loadTimeout = window.setTimeout(resetAndLoad, 500)
}

const hasActiveFilters = computed(() => {
  return searchQuery.value !== '' ||
    filters.value.eventType !== '__all__' ||
    filters.value.days !== 7
})

async function loadLogs() {
  const requestId = ++logsRequestId
  loading.value = true
  try {
    const offset = (currentPage.value - 1) * pageSize.value

    const filterParams = {
      username: filters.value.username || undefined,
      event_type: (filters.value.eventType !== '__all__' ? filters.value.eventType : undefined),
      days: filters.value.days,
      limit: pageSize.value,
      offset
    }

    const data = await auditApi.getAuditLogs(filterParams)
    if (requestId !== logsRequestId) return
    logs.value = data.items || []
    totalRecords.value = data.meta?.total ?? logs.value.length
  } catch (error) {
    if (requestId !== logsRequestId) return
    log.error('获取审计日志失败:', error)
    logs.value = []
    totalRecords.value = 0
  } finally {
    if (requestId === logsRequestId) {
      loading.value = false
    }
  }
}

function refreshLogs() {
  loadLogs()
}

// 搜索变化处理
function handleSearchChange() {
  filters.value.username = searchQuery.value
  debouncedLoadLogs()
}

// 重置筛选条件
function handleResetFilters() {
  searchQuery.value = ''
  filters.value.username = ''
  filters.value.eventType = '__all__'
  filters.value.days = 7
  filtersDaysString.value = '7'
  currentPage.value = 1
  loadLogs()
}

// 页码变化处理
function handlePageChange(page: number) {
  currentPage.value = page
  loadLogs()
}

function handleEventTypeChange(value: string) {
  filters.value.eventType = value
  resetAndLoad()
}

function handleDaysChange(value: string) {
  filtersDaysString.value = value
  filters.value.days = parseInt(value)
  resetAndLoad()
}

function resetAndLoad() {
  currentPage.value = 1
  loadLogs()
}

async function exportLogs() {
  try {
    let allLogs: AuditLog[] = []
    let offset = 0
    const batchSize = 500
    let hasMore = true

    while (hasMore) {
      const data = await auditApi.getAuditLogs({
        username: filters.value.username || undefined,
        event_type: filters.value.eventType !== '__all__' ? filters.value.eventType : undefined,
        days: filters.value.days,
        limit: batchSize,
        offset
      })

      const batch = data.items || []
      allLogs = allLogs.concat(batch)

      if (batch.length < batchSize) {
        hasMore = false
      } else {
        offset += batch.length
        hasMore = offset < (data.meta?.total ?? offset)
      }
    }

    const csvContent = [
      ['时间', '用户邮箱', '用户名', '用户ID', '事件类型', '描述', 'IP地址', '状态码', '错误消息'].join(','),
      ...allLogs.map((log: AuditLog) => [
        log.created_at,
        `"${log.user_email || ''}"`,
        `"${log.user_username || ''}"`,
        log.user_id || '',
        log.event_type,
        `"${log.description || ''}"`,
        log.ip_address || '',
        log.status_code || '',
        `"${log.error_message || ''}"`
      ].join(','))
    ].join('\n')

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `audit-logs-${new Date().toISOString().split('T')[0]}.csv`
    link.click()
  } catch (error) {
    log.error('导出失败:', error)
  }
}

// 使用复用的行点击逻辑
import { useRowClick } from '@/composables/useRowClick'
import { log } from '@/utils/logger'
const { handleMouseDown, shouldTriggerRowClick } = useRowClick()

function handleRowClick(event: MouseEvent, log: AuditLog) {
  if (!shouldTriggerRowClick(event)) return
  showLogDetail(log)
}

function showLogDetail(log: AuditLog) {
  selectedLog.value = log
}

function closeLogDetail() {
  selectedLog.value = null
}

function getEventTypeLabel(eventType: string): string {
  const labels: Record<string, string> = {
    get "login_success"() { return t('site.auth.success') },
    get "login_failed"() { return t('ui.signInFailed') },
    get "logout"() { return t('common.logout') },
    get "api_key_created"() { return t('ui.apiKeyCreated3aeb758') },
    get "api_key_deleted"() { return t('ui.apiKeyDeleted71d5c8a') },
    'api_key_used': 'API密钥使用',
    get "request_success"() { return t('ui.requestSucceeded') },
    get "request_failed"() { return t('ui.requestFailed') },
    'request_rate_limited': '请求限流',
    'request_quota_exceeded': '配额超出',
    get "user_created"() { return t('ui.usersCreated') },
    get "user_updated"() { return t('ui.userUpdated') },
    get "user_deleted"() { return t('ui.userDeleted7d21d97') },
    'provider_added': '提供商添加',
    'provider_updated': '提供商更新',
    'provider_removed': '提供商删除',
    'suspicious_activity': '可疑活动',
    'unauthorized_access': '未授权访问',
    'data_export': '数据导出',
    'config_changed': '配置变更'
  }
  return labels[eventType] || eventType
}

function getEventTypeIcon(eventType: string) {
  const icons: Record<string, unknown> = {
    'login_success': CheckCircle,
    'login_failed': XCircle,
    'logout': User,
    'api_key_created': Key,
    'api_key_deleted': Key,
    'api_key_used': Key,
    'request_success': CheckCircle,
    'request_failed': XCircle,
    'request_rate_limited': AlertTriangle,
    'request_quota_exceeded': AlertTriangle,
    'user_created': User,
    'user_updated': User,
    'user_deleted': User,
    'provider_added': Settings,
    'provider_updated': Settings,
    'provider_removed': Settings,
    'suspicious_activity': Shield,
    'unauthorized_access': Shield,
    'data_export': Activity,
    'config_changed': Settings
  }
  return icons[eventType] || Activity
}

function getEventTypeBadgeVariant(eventType: string): 'default' | 'success' | 'destructive' | 'warning' | 'secondary' {
  if (eventType.includes('success') || eventType.includes('created')) return 'success'
  if (eventType.includes('failed') || eventType.includes('deleted') || eventType.includes('unauthorized')) return 'destructive'
  if (eventType.includes('limited') || eventType.includes('exceeded') || eventType.includes('suspicious')) return 'warning'
  return 'secondary'
}

function getStatusCodeVariant(statusCode: number): 'default' | 'success' | 'destructive' | 'warning' {
  if (statusCode < 300) return 'success'
  if (statusCode < 400) return 'default'
  if (statusCode < 500) return 'warning'
  return 'destructive'
}

function formatDateTime(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleString(getI18nLocale(), {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

onMounted(() => {
  loadLogs()
})

onBeforeUnmount(() => {
  if (loadTimeout !== null) {
    clearTimeout(loadTimeout)
    loadTimeout = null
  }
  logsRequestId += 1
})

</script>
