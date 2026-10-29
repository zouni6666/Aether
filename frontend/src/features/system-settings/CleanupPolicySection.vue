<template>
  <CardSection
    :title="t('ui.requestLogCleanupPolicy')"
    :description="t('ui.configureTieredRetentionAndAutomaticCleanupForRequestRecords')"
  >
    <template #actions>
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <Switch
            id="enable-auto-cleanup"
            :model-value="enableAutoCleanup"
            @update:model-value="$emit('toggleAutoCleanup', $event)"
          />
          <div>
            <Label
              for="enable-auto-cleanup"
              class="text-sm cursor-pointer"
            > {{ t('ui.enableAutomaticCleanup') }} </Label>
            <p class="text-xs text-muted-foreground">
              {{ t('ui.runsEveryDayBeforeDawn') }}
            </p>
          </div>
        </div>
        <Button
          variant="destructive"
          size="sm"
          :disabled="manualCleanupRunning"
          @click="openManualCleanupDialog"
        >
          <Trash2 class="w-3.5 h-3.5 mr-1.5" />
          {{ manualCleanupRunning ? t('ui.cleaning') : t('ui.cleanNow') }}
        </Button>
        <Button
          size="sm"
          :disabled="loading || !hasChanges"
          @click="$emit('save')"
        >
          {{ loading ? t('ui.saving') : t('ui.save') }}
        </Button>
      </div>
    </template>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div>
        <Label
          for="detail-log-retention-days"
          class="block text-sm font-medium"
        > {{ t('ui.detailedRecordRetentionDays') }} </Label>
        <Input
          id="detail-log-retention-days"
          :model-value="detailLogRetentionDays"
          type="number"
          placeholder="7"
          class="mt-1"
          @update:model-value="$emit('update:detailLogRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.compressBodyFieldsAfterThisPeriod') }}
        </p>
      </div>

      <div>
        <Label
          for="compressed-log-retention-days"
          class="block text-sm font-medium"
        > {{ t('ui.compressedRecordRetentionDays') }} </Label>
        <Input
          id="compressed-log-retention-days"
          :model-value="compressedLogRetentionDays"
          type="number"
          placeholder="30"
          class="mt-1"
          @update:model-value="$emit('update:compressedLogRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.deleteBodyFieldsAfterThisPeriod') }}
        </p>
      </div>

      <div>
        <Label
          for="header-retention-days"
          class="block text-sm font-medium"
        > {{ t('ui.requestHeaderRetentionDays') }} </Label>
        <Input
          id="header-retention-days"
          :model-value="headerRetentionDays"
          type="number"
          placeholder="90"
          class="mt-1"
          @update:model-value="$emit('update:headerRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.clearHeadersFieldsAfterThisPeriod') }}
        </p>
      </div>

      <div>
        <Label
          for="log-retention-days"
          class="block text-sm font-medium"
        > {{ t('ui.requestRecordRetentionDays') }} </Label>
        <Input
          id="log-retention-days"
          :model-value="logRetentionDays"
          type="number"
          placeholder="365"
          class="mt-1"
          @update:model-value="$emit('update:logRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.deleteFullRecordsAfterThisPeriod') }}
        </p>
      </div>

      <div>
        <Label
          for="cleanup-batch-size"
          class="block text-sm font-medium"
        > {{ t('ui.cleanupBatchSize') }} </Label>
        <Input
          id="cleanup-batch-size"
          :model-value="cleanupBatchSize"
          type="number"
          placeholder="1000"
          class="mt-1"
          @update:model-value="$emit('update:cleanupBatchSize', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.avoidOversizedSingleOperationsThatAffectPerformance') }}
        </p>
      </div>

      <div>
        <Label
          for="audit-log-retention-days"
          class="block text-sm font-medium"
        > {{ t('ui.auditLogRetentionDays') }} </Label>
        <Input
          id="audit-log-retention-days"
          :model-value="auditLogRetentionDays"
          type="number"
          placeholder="30"
          class="mt-1"
          @update:model-value="$emit('update:auditLogRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.deleteAuditLogRecordsAfterThisPeriod') }}
        </p>
      </div>

      <div>
        <Label
          for="request-candidates-retention-days"
          class="block text-sm font-medium"
        > {{ t('ui.candidateRecordRetentionDays') }} </Label>
        <Input
          id="request-candidates-retention-days"
          :model-value="requestCandidatesRetentionDays"
          type="number"
          placeholder="30"
          class="mt-1"
          @update:model-value="$emit('update:requestCandidatesRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.afterExpirationCleanRequestCandidatesAuditRecordsInTimeWindow') }}
        </p>
      </div>

      <div>
        <Label
          for="request-candidates-cleanup-batch-size"
          class="block text-sm font-medium"
        > {{ t('ui.candidateCleanupBatch') }} </Label>
        <Input
          id="request-candidates-cleanup-batch-size"
          :model-value="requestCandidatesCleanupBatchSize"
          type="number"
          placeholder="5000"
          class="mt-1"
          @update:model-value="$emit('update:requestCandidatesCleanupBatchSize', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.controlCleanupPacingForTheCandidateRecordTableIndependentlyNo') }}
        </p>
      </div>

      <div>
        <Label
          for="proxy-node-metrics-1m-retention-days"
          class="block text-sm font-medium"
        > {{ t('ui.proxy1mMetricRetentionDays') }} </Label>
        <Input
          id="proxy-node-metrics-1m-retention-days"
          :model-value="proxyNodeMetrics1mRetentionDays"
          type="number"
          min="1"
          max="365"
          placeholder="30"
          class="mt-1"
          @update:model-value="$emit('update:proxyNodeMetrics1mRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.usedForMinuteLevelProxyNodeStabilityChartsBackendKeeps') }}
        </p>
      </div>

      <div>
        <Label
          for="proxy-node-metrics-1h-retention-days"
          class="block text-sm font-medium"
        > {{ t('ui.proxy1hMetricRetentionDays') }} </Label>
        <Input
          id="proxy-node-metrics-1h-retention-days"
          :model-value="proxyNodeMetrics1hRetentionDays"
          type="number"
          min="1"
          max="1095"
          placeholder="180"
          class="mt-1"
          @update:model-value="$emit('update:proxyNodeMetrics1hRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.hourlyAggregatesAreUsedForLongTermTrendsAndCannot') }}
        </p>
      </div>

      <div>
        <Label
          for="proxy-node-metrics-cleanup-batch-size"
          class="block text-sm font-medium"
        > {{ t('ui.proxyMetricCleanupBatch') }} </Label>
        <Input
          id="proxy-node-metrics-cleanup-batch-size"
          :model-value="proxyNodeMetricsCleanupBatchSize"
          type="number"
          min="1"
          max="50000"
          placeholder="5000"
          class="mt-1"
          @update:model-value="$emit('update:proxyNodeMetricsCleanupBatchSize', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.separatelyLimitsPerBatchDeletesForTheProxyNodeMetrics') }}
        </p>
      </div>
    </div>

    <!-- 清理策略说明 -->
    <div class="mt-4 p-4 bg-muted/50 rounded-lg">
      <h4 class="text-sm font-medium mb-2">
        {{ t('ui.cleanupPolicyNotes') }}
      </h4>
      <div class="text-xs text-muted-foreground space-y-1">
        <p>1. <strong>{{ t('ui.detailedLogStage') }}</strong>{{ t('ui.retainCompleteRequestBodyAndResponseBodyData') }}</p>
        <p>2. <strong>{{ t('ui.compressedLogStage') }}</strong>{{ t('ui.compressBodyFieldsToSaveStorageSpace') }}</p>
        <p>3. <strong>{{ t('ui.statisticsStage') }}</strong>{{ t('ui.keepOnlyStatisticsSuchAsTokensAndCost') }}</p>
        <p>4. <strong>{{ t('ui.archiveDeletion') }}</strong>{{ t('ui.deleteRecordsEntirelyAfterTheRetentionPeriod') }}</p>
        <p>5. <strong>{{ t('ui.candidateRecords') }}</strong>{{ t('ui.cleanUpRequestCandidatesAuditRecordsByRetentionPeriodIndependently') }}</p>
        <p>6. <strong>{{ t('ui.auditLogs') }}</strong>{{ t('ui.cleanUpSecurityEventsSuchAsSignInsAndUser') }}</p>
        <p>7. <strong>{{ t('ui.proxyMetrics') }}</strong>{{ t('ui.keep1MinuteAnd1HourAggregatesCleanupDeletesExpired') }}</p>
      </div>
    </div>

    <ManualCleanupConfirmDialog
      :open="manualCleanupDialogOpen"
      @update:open="manualCleanupDialogOpen = $event"
      @running-change="manualCleanupRunning = $event"
      @completed="handleManualCleanupCompleted"
    />

    <div
      v-if="manualCleanupResult"
      class="mt-4 rounded-md border border-border bg-muted/30 px-4 py-3 text-sm"
    >
      <div class="font-medium">
        {{ manualCleanupResult.title }}
      </div>
      <div
        v-if="manualCleanupResult.description"
        class="mt-1 text-xs text-muted-foreground"
      >
        {{ manualCleanupResult.description }}
      </div>
    </div>

    <div class="mt-4 border border-border rounded-lg overflow-hidden">
      <div class="flex flex-wrap items-start justify-between gap-3 px-4 py-3 border-b border-border">
        <div class="min-w-0 flex-1 basis-48">
          <h4 class="text-sm font-medium">
            {{ t('ui.recentCleanupRuns') }}
          </h4>
          <p class="text-xs text-muted-foreground">
            {{ t('ui.resultsFromAutomaticCleanupManualSystemCleanupAndRequestBody') }}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          class="shrink-0"
          :disabled="cleanupRunsLoading"
          @click="loadCleanupRuns"
        >
          <RefreshCw
            class="w-3.5 h-3.5 mr-1.5"
            :class="{ 'animate-spin': cleanupRunsLoading }"
          /> {{ t('ui.refresh') }}
        </Button>
      </div>
      <div
        v-if="cleanupRuns.length === 0 && !cleanupRunsLoading"
        class="px-4 py-6 text-sm text-muted-foreground"
      >
        {{ t('ui.noCleanupHistory') }}
      </div>
      <div
        v-else
        class="overflow-x-auto"
      >
        <table class="w-full text-sm">
          <thead class="bg-muted/30 text-xs text-muted-foreground">
            <tr>
              <th class="px-4 py-2 text-left font-medium">
                {{ t('ui.time') }}
              </th>
              <th class="px-4 py-2 text-left font-medium">
                {{ t('ui.typeba40014') }}
              </th>
              <th class="px-4 py-2 text-left font-medium">
                {{ t('ui.source') }}
              </th>
              <th class="px-4 py-2 text-left font-medium">
                {{ t('ui.status6320b4a') }}
              </th>
              <th class="px-4 py-2 text-left font-medium">
                {{ t('ui.result') }}
              </th>
              <th class="px-4 py-2 text-right font-medium">
                {{ t('ui.duration') }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="run in cleanupRuns"
              :key="run.id"
              class="border-t border-border"
            >
              <td class="px-4 py-2 whitespace-nowrap">
                {{ formatRunTime(run.started_at_unix_secs) }}
              </td>
              <td class="px-4 py-2 whitespace-nowrap">
                {{ cleanupKindLabel(run.kind) }}
              </td>
              <td class="px-4 py-2 whitespace-nowrap text-muted-foreground">
                {{ run.trigger === 'manual' ? t('ui.manual') : t('ui.automatic') }}
              </td>
              <td class="px-4 py-2 whitespace-nowrap">
                <span :class="cleanupStatusClass(run.status)">
                  {{ cleanupStatusLabel(run.status) }}
                </span>
              </td>
              <td class="px-4 py-2 min-w-[18rem]">
                <div>{{ run.error || run.message }}</div>
                <div class="text-xs text-muted-foreground">
                  {{ cleanupSummaryText(run.summary) }}
                </div>
              </td>
              <td class="px-4 py-2 text-right whitespace-nowrap text-muted-foreground">
                {{ formatDuration(run.duration_ms) }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </CardSection>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
defineProps<{
  enableAutoCleanup: boolean
  detailLogRetentionDays: number
  compressedLogRetentionDays: number
  headerRetentionDays: number
  logRetentionDays: number
  cleanupBatchSize: number
  auditLogRetentionDays: number
  requestCandidatesRetentionDays: number
  requestCandidatesCleanupBatchSize: number
  proxyNodeMetrics1mRetentionDays: number
  proxyNodeMetrics1hRetentionDays: number
  proxyNodeMetricsCleanupBatchSize: number
  loading: boolean
  hasChanges: boolean
}>()

defineEmits<{
  save: []
  toggleAutoCleanup: [enabled: boolean]
  'update:detailLogRetentionDays': [value: number]
  'update:compressedLogRetentionDays': [value: number]
  'update:headerRetentionDays': [value: number]
  'update:logRetentionDays': [value: number]
  'update:cleanupBatchSize': [value: number]
  'update:auditLogRetentionDays': [value: number]
  'update:requestCandidatesRetentionDays': [value: number]
  'update:requestCandidatesCleanupBatchSize': [value: number]
  'update:proxyNodeMetrics1mRetentionDays': [value: number]
  'update:proxyNodeMetrics1hRetentionDays': [value: number]
  'update:proxyNodeMetricsCleanupBatchSize': [value: number]
}>()

const { t } = useI18n()

import { onBeforeUnmount, onMounted, ref } from 'vue'
import { RefreshCw, Trash2 } from 'lucide-vue-next'
import { adminApi, type CleanupRunRecord } from '@/api/admin'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import Switch from '@/components/ui/switch.vue'
import { CardSection } from '@/components/layout'
import ManualCleanupConfirmDialog from './ManualCleanupConfirmDialog.vue'
import { useToast } from '@/composables/useToast'

const cleanupRuns = ref<CleanupRunRecord[]>([])
const cleanupRunsLoading = ref(false)
let cleanupRunsTimer: number | null = null

const manualCleanupDialogOpen = ref(false)
const manualCleanupRunning = ref(false)
const manualCleanupResult = ref<{ title: string; description?: string } | null>(null)
const toast = useToast()

function openManualCleanupDialog() {
  if (manualCleanupRunning.value) return
  manualCleanupDialogOpen.value = true
}

function handleManualCleanupCompleted(task: CleanupRunRecord) {
  manualCleanupRunning.value = false
  manualCleanupResult.value = {
    title: task.message,
    description: cleanupSummaryText(task.summary),
  }
  if (task.status === 'failed') {
    toast.error(task.error || task.message)
  } else {
    toast.success(task.message)
  }
  void loadCleanupRuns()
}

async function loadCleanupRuns() {
  cleanupRunsLoading.value = true
  try {
    const response = await adminApi.getCleanupRuns()
    cleanupRuns.value = response.items.slice(0, 10)
  } finally {
    cleanupRunsLoading.value = false
  }
}

function cleanupKindLabel(kind: string): string {
  const labels: Record<string, string> = {
    get usage_cleanup() { return t('ui.requestRecords') },
    get audit_cleanup() { return t('ui.auditLogs') },
    get request_candidate_cleanup() { return t('ui.candidateRecords') },
    get request_bodies() { return t('ui.requestBody') },
    get config_purge() { return t('ui.configurationCleared') },
    get users_purge() { return t('ui.usersCleared') },
    get usage_purge() { return t('ui.usageRecordsCleared') },
    get audit_logs_purge() { return t('ui.auditLogsCleared') },
    get stats_purge() { return t('ui.aggregatedStatisticsCleared') },
    get system_cleanup() { return t('ui.systemCleanup') },
  }
  return labels[kind] || kind
}

function cleanupStatusLabel(status: string): string {
  if (status === 'processing') return t('ui.running5026a63')
  if (status === 'failed') return t('ui.failed28384d7')
  return t('ui.completed')
}

function cleanupStatusClass(status: string): string {
  if (status === 'processing') return 'text-amber-500'
  if (status === 'failed') return 'text-destructive'
  return 'text-emerald-500'
}

function formatRunTime(value: number): string {
  if (!value) return '-'
  return new Date(value * 1000).toLocaleString()
}

function formatDuration(value: number | null): string {
  if (value === null || value === undefined) return '-'
  if (value < 1000) return `${value}ms`
  return `${(value / 1000).toFixed(1)}s`
}

function cleanupSummaryText(summary: Record<string, unknown>): string {
  const total = typeof summary.total === 'number' ? summary.total : null
  if (total !== null) return t('settings.cleanup.affectedRows', { count: total })

  const entries = Object.entries(summary)
    .filter(([, value]) => typeof value === 'number' && value > 0)
    .map(([key, value]) => `${summaryLabel(key)} ${value}`)
  return entries.length > 0 ? entries.join(' / ') : t('settings.cleanup.noChanges')
}

function summaryLabel(key: string): string {
  const labels: Record<string, string> = {
    get body_externalized() { return t('ui.detailedBody') },
    get legacy_body_refs_migrated() { return t('ui.migration') },
    get body_cleaned() { return t('ui.bodyCleanup') },
    get header_cleaned() { return t('ui.headerCleanup') },
    keys_cleaned: 'Key',
    get records_deleted() { return t('ui.deleteRecords') },
    get audit_logs_deleted() { return t('ui.deleteLogs') },
    get request_candidates_deleted() { return t('ui.deleteCandidates') },
  }
  return labels[key] || key
}

onMounted(() => {
  void loadCleanupRuns()
  cleanupRunsTimer = window.setInterval(() => {
    void loadCleanupRuns()
  }, 15_000)
})

onBeforeUnmount(() => {
  if (cleanupRunsTimer) {
    window.clearInterval(cleanupRunsTimer)
    cleanupRunsTimer = null
  }
})

</script>
