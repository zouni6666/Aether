<template>
  <Dialog
    :model-value="modelValue"
    :title="dialogTitle"
    :description="dialogDescription"
    size="3xl"
    persistent
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="max-h-[calc(100dvh-13rem)] space-y-4 overflow-y-auto overscroll-contain pr-1 sm:max-h-[min(72vh,44rem)] sm:pr-2">
      <div class="space-y-2 rounded-lg border bg-muted/20 px-3 py-3">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <div class="text-xs font-medium text-foreground">
            {{ t('ui.actionScope') }}
          </div>
          <Badge
            variant="outline"
            class="text-[11px]"
          >
            {{ selectAllFiltered ? t('ui.selectAllFilteredResults') : t('ui.selectedTableRows') }}
          </Badge>
        </div>
        <div class="text-sm text-foreground">
          {{ t('ui.selected3f4ebc4') }} <span class="font-semibold tabular-nums">{{ selectedCount }}</span> {{ t('ui.accounts') }}
        </div>
        <div class="text-[11px] text-muted-foreground">
          {{ t('ui.theScopeUsesTheCurrentFiltersInThePoolManagement') }}
        </div>
      </div>

      <div class="space-y-2 rounded-lg border bg-background px-3 py-3">
        <div>
          <div class="text-xs font-medium text-muted-foreground">
            {{ t('ui.action') }}
          </div>
          <div class="mt-1 text-sm font-semibold text-foreground">
            {{ selectedActionOption.label }}
          </div>
          <p class="mt-1 text-[11px] text-muted-foreground">
            {{ selectedActionOption.hint }}
          </p>
        </div>

        <div
          v-if="selectedAction === 'set_proxy'"
          class="space-y-2 border-t border-border/60 pt-3"
        >
          <div class="text-[11px] text-muted-foreground">
            {{ t('ui.selectAProxyNodeToAssign') }}
          </div>
          <ProxyNodeSelect
            :model-value="proxyNodeIdForAction"
            trigger-class="h-9"
            @update:model-value="(v: string) => proxyNodeIdForAction = v"
          />
        </div>
      </div>

      <section
        v-if="selectedAction === 'update_settings'"
        class="space-y-3 rounded-lg border border-primary/25 bg-primary/5 p-3 sm:p-4"
      >
        <div class="flex flex-wrap items-start justify-between gap-2">
          <div>
            <h3 class="text-sm font-semibold">
              {{ t('ui.moreSettings') }}
            </h3>
            <p class="text-[11px] text-muted-foreground">
              {{ t('ui.updateOnlyCheckedFieldsOtherSettingsRemainUnchanged') }}
            </p>
          </div>
          <Badge
            variant="outline"
            class="tabular-nums"
          >
            {{ t('ui.selected') }} {{ selectedSettingsCount }} {{ t('ui.items') }}
          </Badge>
        </div>

        <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          <div class="space-y-2 rounded-md border bg-background p-3">
            <div class="flex min-h-6 items-center gap-2">
              <Checkbox
                :checked="settingsSelection.internal_priority"
                @update:checked="settingsSelection.internal_priority = $event === true"
              />
              <Label class="text-xs">{{ t('ui.priority565d646') }}</Label>
            </div>
            <Input
              v-model.number="settingsDraft.internal_priority"
              type="number"
              min="0"
              class="h-9"
              :disabled="!settingsSelection.internal_priority"
            />
          </div>

          <div class="space-y-2 rounded-md border bg-background p-3">
            <div class="flex min-h-6 items-center gap-2">
              <Checkbox
                :checked="settingsSelection.rpm_limit"
                @update:checked="settingsSelection.rpm_limit = $event === true"
              />
              <Label class="text-xs">{{ t('ui.rpmLimit50b7143') }}</Label>
            </div>
            <Input
              :model-value="settingsDraft.rpm_limit ?? ''"
              type="number"
              min="1"
              max="10000"
              class="h-9"
              :placeholder="t('ui.leaveEmptyForAdaptiveMode')"
              :disabled="!settingsSelection.rpm_limit"
              @update:model-value="settingsDraft.rpm_limit = parseNullableNumberInput($event, { min: 1, max: 10000 })"
            />
          </div>

          <div class="space-y-2 rounded-md border bg-background p-3">
            <div class="flex min-h-6 items-center gap-2">
              <Checkbox
                :checked="settingsSelection.concurrent_limit"
                @update:checked="settingsSelection.concurrent_limit = $event === true"
              />
              <Label class="text-xs">{{ t('ui.concurrentRequestLimit') }}</Label>
            </div>
            <Input
              :model-value="settingsDraft.concurrent_limit ?? ''"
              type="number"
              min="0"
              class="h-9"
              :placeholder="t('ui.leaveEmptyForNoLimit')"
              :disabled="!settingsSelection.concurrent_limit"
              @update:model-value="settingsDraft.concurrent_limit = parseNullableNumberInput($event, { min: 0 })"
            />
          </div>

          <div class="space-y-2 rounded-md border bg-background p-3">
            <div class="flex min-h-6 items-center gap-2">
              <Checkbox
                :checked="settingsSelection.cache_ttl_minutes"
                @update:checked="settingsSelection.cache_ttl_minutes = $event === true"
              />
              <Label class="text-xs">{{ t('ui.cacheTtlMinutes') }}</Label>
            </div>
            <Input
              :model-value="settingsDraft.cache_ttl_minutes"
              type="number"
              min="0"
              max="60"
              class="h-9"
              :disabled="!settingsSelection.cache_ttl_minutes"
              @update:model-value="settingsDraft.cache_ttl_minutes = parseNumberInput($event, { min: 0, max: 60 }) ?? 5"
            />
          </div>

          <div class="space-y-2 rounded-md border bg-background p-3">
            <div class="flex min-h-6 items-center gap-2">
              <Checkbox
                :checked="settingsSelection.max_probe_interval_minutes"
                @update:checked="settingsSelection.max_probe_interval_minutes = $event === true"
              />
              <Label class="text-xs">{{ t('ui.circuitBreakerProbeIntervalMinutes') }}</Label>
            </div>
            <Input
              :model-value="settingsDraft.max_probe_interval_minutes"
              type="number"
              min="0"
              max="32"
              class="h-9"
              :disabled="!settingsSelection.max_probe_interval_minutes"
              @update:model-value="settingsDraft.max_probe_interval_minutes = parseNumberInput($event, { min: 0, max: 32 }) ?? 32"
            />
          </div>

          <div class="space-y-2 rounded-md border bg-background p-3">
            <div class="flex min-h-6 items-center gap-2">
              <Checkbox
                :checked="settingsSelection.is_active"
                @update:checked="settingsSelection.is_active = $event === true"
              />
              <Label class="text-xs">{{ t('ui.enablementStatus') }}</Label>
            </div>
            <Select
              v-model="settingsStatus"
              :disabled="!settingsSelection.is_active"
            >
              <SelectTrigger class="h-9">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="enabled">
                  {{ t('ui.enablef4f0ead') }}
                </SelectItem>
                <SelectItem value="disabled">
                  {{ t('ui.disable4e6fd0e') }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-2 rounded-md border bg-background p-3 sm:col-span-2">
            <div class="flex min-h-6 items-center gap-2">
              <Checkbox
                :checked="settingsSelection.proxy_node_id"
                @update:checked="settingsSelection.proxy_node_id = $event === true"
              />
              <Label class="text-xs">{{ t('ui.accountProxy') }}</Label>
            </div>
            <div class="grid gap-2 sm:grid-cols-[9rem_minmax(0,1fr)]">
              <Select
                v-model="settingsDraft.proxy_mode"
                :disabled="!settingsSelection.proxy_node_id"
              >
                <SelectTrigger class="h-9">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="set">
                    {{ t('ui.setNode') }}
                  </SelectItem>
                  <SelectItem value="clear">
                    {{ t('ui.clearProxy') }}
                  </SelectItem>
                </SelectContent>
              </Select>
              <ProxyNodeSelect
                v-if="settingsDraft.proxy_mode === 'set'"
                :model-value="settingsDraft.proxy_node_id"
                trigger-class="h-9"
                :class="!settingsSelection.proxy_node_id ? 'pointer-events-none opacity-50' : ''"
                @update:model-value="(value: string) => settingsDraft.proxy_node_id = value"
              />
              <div
                v-else
                class="flex h-9 items-center rounded-md border bg-muted/30 px-3 text-xs text-muted-foreground"
              >
                {{ t('ui.fallBackToTheProviderDefaultProxy') }}
              </div>
            </div>
          </div>

          <div class="space-y-2 rounded-md border bg-background p-3">
            <div class="flex min-h-6 items-center gap-2">
              <Checkbox
                :checked="settingsSelection.note"
                @update:checked="settingsSelection.note = $event === true"
              />
              <Label class="text-xs">{{ t('ui.notes') }}</Label>
            </div>
            <Input
              v-model="settingsDraft.note"
              class="h-9"
              :placeholder="t('ui.leaveEmptyToClearNotes')"
              :disabled="!settingsSelection.note"
            />
          </div>
        </div>

        <p class="min-h-5 text-xs text-destructive">
          {{ settingsErrors[0] || '' }}
        </p>
      </section>

      <div
        v-if="executing && progressTotal > 0"
        class="space-y-1"
      >
        <div class="flex items-center justify-between text-xs text-muted-foreground">
          <span>{{ progressLabel }}</span>
          <span>{{ progressDone }} / {{ progressTotal }}</span>
        </div>
        <div class="h-1.5 w-full rounded-full bg-muted overflow-hidden">
          <div
            class="h-full rounded-full bg-primary transition-all duration-150"
            :style="{ width: `${Math.round((progressDone / progressTotal) * 100)}%` }"
          />
        </div>
      </div>
      <div
        v-else-if="lastResultMessage"
        class="rounded-md border bg-background px-3 py-2 text-xs text-muted-foreground"
      >
        {{ lastResultMessage }}
      </div>
    </div>

    <template #footer>
      <Button
        variant="outline"
        :disabled="executing"
        @click="emit('update:modelValue', false)"
      >
        {{ t('ui.close') }}
      </Button>
      <Button
        :variant="selectedActionOption.destructive ? 'destructive' : 'default'"
        :disabled="!canExecuteSpecifiedAction(selectedAction)"
        @click="confirmAndExecuteAction(selectedAction)"
      >
        {{ executeActionButtonLabel }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, nextTick, reactive, ref, watch } from 'vue'
import {
  Badge,
  Button,
  Checkbox,
  Dialog,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui'
import ProxyNodeSelect from '@/features/providers/components/ProxyNodeSelect.vue'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { parseApiError } from '@/utils/errorParser'
import { log } from '@/utils/logger'
import {
  batchActionPoolKeys,
  getPoolBatchDeleteTask,
  resolvePoolKeySelection,
  type PoolKeyDetail,
  type PoolKeySelectionRequest,
  type PoolKeySelectionItem,
} from '@/api/endpoints/pool'
import { exportKey, refreshProviderQuota } from '@/api/endpoints/keys'
import { refreshProviderOAuth } from '@/api/endpoints/provider_oauth'
import { useProxyNodesStore } from '@/stores/proxy-nodes'
import {
  canExportOAuthCredential,
  canRefreshOAuthCredential,
} from '@/features/providers/utils/providerKeyAuth'
import { runChunkedBatchAction } from '@/utils/batchAction'
import { parseNullableNumberInput, parseNumberInput } from '@/utils/form'
import {
  buildPoolKeySettingsPatch,
  createPoolKeyBatchSettingSelection,
  createPoolKeyBatchSettingsDraft,
  validatePoolKeyBatchSettings,
} from '@/features/pool/utils/poolKeyBatchSettings'
import {
  POOL_BATCH_ACTION_OPTIONS,
  type PoolBatchActionValue,
} from '@/features/pool/utils/poolBatchActions'

const props = defineProps<{
  modelValue: boolean
  providerId: string
  providerName?: string
  providerType?: string
  batchConcurrency?: number | null
  selectedKeys: PoolKeyDetail[]
  selectAllFiltered: boolean
  selectedCount: number
  selectionFilters: PoolKeySelectionRequest
  initialAction?: PoolBatchActionValue | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  changed: []
  'edit-config': [keyIds: string[]]
}>()

const { t } = useI18n()

const ACTION_OPTIONS = POOL_BATCH_ACTION_OPTIONS

const { success, warning, error: showError } = useToast()
const { confirm } = useConfirm()
const proxyNodesStore = useProxyNodesStore()

const executing = ref(false)
const selectedAction = ref<PoolBatchActionValue>('refresh_quota')
const proxyNodeIdForAction = ref('')
const settingsSelection = reactive(createPoolKeyBatchSettingSelection())
const settingsDraft = reactive(createPoolKeyBatchSettingsDraft())
const settingsStatus = computed({
  get: () => settingsDraft.is_active ? 'enabled' : 'disabled',
  set: (value: string) => { settingsDraft.is_active = value === 'enabled' },
})
const lastResultMessage = ref('')
const progressTotal = ref(0)
const progressDone = ref(0)
const progressLabel = ref('')

const dialogDescription = computed(() => {
  const name = (props.providerName || '').trim()
  return name ? t('feedback.pool.batchDescriptionNamed', { name }) : t('feedback.pool.batchDescription')
})

const selectedCount = computed(() => Math.max(0, Number(props.selectedCount || 0)))
const selectAllFiltered = computed(() => props.selectAllFiltered)
const settingsErrors = computed(() => validatePoolKeyBatchSettings(settingsSelection, settingsDraft))
const selectedSettingsCount = computed(() => Object.values(settingsSelection).filter(Boolean).length)
const selectedActionOption = computed(() => (
  ACTION_OPTIONS.find(option => option.value === selectedAction.value)
  || {
    value: 'refresh_quota' as const,
    destructive: false,
    get label() { return t('ui.refreshQuota') },
    get hint() { return t('ui.refreshQuotaInformationToCheckTheLatestStatus') },
  }
))
const dialogTitle = computed(() => t('feedback.pool.actionTitle', { action: selectedActionOption.value.label }))
const executeActionButtonLabel = computed(() => {
  if (executing.value) return t('ui.runningfeed25e')
  if (selectedAction.value === 'edit_config') return t('ui.editConfiguration')
  if (selectedAction.value === 'set_proxy') return t('ui.applyProxySettings')
  if (selectedAction.value === 'update_settings') return t('ui.applyAdditionalSettings')
  return t('feedback.pool.runAction', { action: selectedActionOption.value.label })
})

function sanitizeFileNamePart(value: unknown, fallback: string): string {
  const sanitized = String(value || '')
    .trim()
    .replace(/[^a-zA-Z0-9_\-@.]/g, '_')
    .replace(/_+/g, '_')
    .replace(/^_+|_+$/g, '')
  return sanitized || fallback
}

function formatExportTimestamp(date: Date = new Date()): string {
  const pad = (value: number) => String(value).padStart(2, '0')
  return `${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())}_${pad(date.getHours())}${pad(date.getMinutes())}${pad(date.getSeconds())}`
}

function getBatchExportFilename(): string {
  const providerType = sanitizeFileNamePart(props.providerType || 'pool', 'pool')
  const providerName = sanitizeFileNamePart(props.providerName || props.providerId.slice(0, 8), 'provider')
  return `aether_${providerType}_${providerName}_batch_export_${formatExportTimestamp()}.json`
}

function downloadJsonFile(data: unknown, filename: string): void {
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  URL.revokeObjectURL(url)
}

function canExecuteSpecifiedAction(action: PoolBatchActionValue): boolean {
  if (executing.value || selectedCount.value === 0) return false
  if (action === 'set_proxy') return Boolean(proxyNodeIdForAction.value)
  if (action === 'update_settings') return settingsErrors.value.length === 0
  return true
}

function handleActionButtonClick(action: PoolBatchActionValue): void {
  if (action === 'set_proxy' || action === 'update_settings') {
    selectedAction.value = action
    return
  }
  void confirmAndExecuteAction(action)
}

async function confirmAndExecuteAction(action: PoolBatchActionValue): Promise<void> {
  selectedAction.value = action
  if (selectedCount.value === 0) {
    warning(t('ui.selectAnAccountFirst'))
    return
  }
  if (action === 'set_proxy' && !proxyNodeIdForAction.value) {
    warning(t('ui.selectAProxyNodeFirst'))
    return
  }
  if (action === 'update_settings' && settingsErrors.value.length > 0) {
    warning(settingsErrors.value[0])
    return
  }
  if (!canExecuteSpecifiedAction(action)) return

  if (action === 'edit_config') {
    await openBatchEditor()
    return
  }

  const actionOption = ACTION_OPTIONS.find((item) => item.value === action)
  const actionLabel = actionOption?.label || t('ui.action')
  const scopeLabel = selectAllFiltered.value ? t('ui.filteredResults') : t('ui.selectedAccounts')
  const confirmed = await confirm({
    title: actionLabel,
    message: t('feedback.pool.confirmAction', { scope: scopeLabel, count: selectedCount.value, action: actionLabel }),
    confirmText: actionOption?.destructive ? t('ui.confirmDeletion') : t('ui.confirmAction'),
    ...(actionOption?.destructive ? { variant: 'destructive' as const } : {}),
  })
  if (!confirmed) return
  await executeAction(action)
}

async function openBatchEditor(): Promise<void> {
  if (executing.value || selectedCount.value === 0) return
  executing.value = true
  progressDone.value = 0
  progressTotal.value = 0
  progressLabel.value = selectAllFiltered.value ? t('ui.resolvingFilteredResults') : t('ui.preparingBulkEdit')
  try {
    const selectedKeys = await resolveSelectedItems()
    const keyIds = selectedKeys.map(key => key.key_id)
    if (keyIds.length === 0) {
      warning(t('ui.noEditableAccountsFoundRefreshTheListAndTryAgain'))
      return
    }
    emit('update:modelValue', false)
    emit('edit-config', keyIds)
  } catch (err) {
    showError(parseApiError(err, t('ui.failedToPrepareBulkEdit')))
  } finally {
    executing.value = false
    progressDone.value = 0
    progressTotal.value = 0
    progressLabel.value = ''
  }
}

const DELETE_POLL_INTERVAL_MS = 2000
const DELETE_POLL_MAX_MS = 10 * 60 * 1000
const DELETE_POLL_MAX_FAILURES = 3

async function pollDeleteTask(
  providerId: string,
  taskId: string,
  progressOffset: number,
): Promise<{ status: string; deleted: number }> {
  const deadline = Date.now() + DELETE_POLL_MAX_MS
  let consecutiveFailures = 0
  while (Date.now() < deadline) {
    try {
      const task = await getPoolBatchDeleteTask(providerId, taskId)
      consecutiveFailures = 0
      progressDone.value = progressOffset + task.deleted
      if (task.status === 'completed' || task.status === 'failed') {
        return { status: task.status, deleted: task.deleted }
      }
    } catch {
      consecutiveFailures++
      if (consecutiveFailures >= DELETE_POLL_MAX_FAILURES) {
        return { status: 'failed', deleted: 0 }
      }
    }
    await new Promise((resolve) => setTimeout(resolve, DELETE_POLL_INTERVAL_MS))
  }
  return { status: 'failed', deleted: 0 }
}

async function resolveSelectedItems(): Promise<PoolKeySelectionItem[]> {
  if (!props.providerId) return []

  if (selectAllFiltered.value) {
    progressLabel.value = t('ui.resolvingFilteredResults')
    const result = await resolvePoolKeySelection(props.providerId, { ...props.selectionFilters })
    return Array.isArray(result.items) ? result.items : []
  }

  const selectedKeys = [...new Map(
    props.selectedKeys
      .filter(key => Boolean(key.key_id))
      .map(key => [key.key_id, key] as const),
  ).values()]
  return selectedKeys.map((key) => {
    return {
      key_id: key.key_id,
      key_name: key.key_name || '',
      auth_type: key.auth_type || 'api_key',
      auth_type_by_format: key.auth_type_by_format,
      allow_auth_channel_mismatch_formats: key.allow_auth_channel_mismatch_formats,
      credential_kind: key.credential_kind,
      runtime_auth_kind: key.runtime_auth_kind,
      oauth_managed: key.oauth_managed,
      agent_identity: key.agent_identity,
      oauth_header_auth: key.oauth_header_auth,
      can_refresh_oauth: key.can_refresh_oauth,
      can_export_oauth: key.can_export_oauth,
      can_edit_oauth: key.can_edit_oauth,
    }
  })
}

async function executeAction(actionOverride?: PoolBatchActionValue): Promise<void> {
  if (executing.value) return
  if (actionOverride) {
    selectedAction.value = actionOverride
  }
  if (selectedCount.value === 0) {
    warning(t('ui.selectAnAccountFirst'))
    return
  }

  const requestedCount = selectedCount.value
  if (selectedAction.value === 'set_proxy' && !proxyNodeIdForAction.value) {
    warning(t('ui.selectAProxyNodeFirst'))
    return
  }
  if (selectedAction.value === 'update_settings' && settingsErrors.value.length > 0) {
    warning(settingsErrors.value[0])
    return
  }

  executing.value = true
  let successCount = 0
  let failedCount = 0
  let skippedCount = 0
  let resolvedCount = 0
  const actionStartedAt = performance.now()
  let actionPhaseMs = 0

  const actionLabel = ACTION_OPTIONS.find((item) => item.value === selectedAction.value)?.label || t('ui.run')
  progressDone.value = 0
  progressTotal.value = 0
  progressLabel.value = selectAllFiltered.value ? t('ui.resolvingFilteredResults') : t('feedback.pool.runningAction', { action: actionLabel })
  lastResultMessage.value = ''

  try {
    const selectedKeys = await resolveSelectedItems()
    resolvedCount = selectedKeys.length
    if (selectedKeys.length === 0) {
      warning(t('ui.noEligibleAccountsFoundRefreshTheListAndTryAgain'))
      return
    }

    progressDone.value = 0
    progressTotal.value = selectedKeys.length
    progressLabel.value = t('feedback.pool.runningAction', { action: actionLabel })

    if (selectedAction.value === 'refresh_quota') {
      const targetIds = selectedKeys.map((key) => key.key_id)
      const BATCH_SIZE = 20
      const counts = await runChunkedBatchAction({
        items: targetIds,
        chunkSize: BATCH_SIZE,
        runChunk: (batch) => refreshProviderQuota(props.providerId, batch),
        onChunkStart: ({ batchIndex, totalBatches }) => {
          progressLabel.value = t('feedback.pool.runningBatch', { action: actionLabel, index: batchIndex, total: totalBatches })
        },
        onChunkDone: ({ processed }) => {
          progressDone.value = processed
        },
      })
      successCount += counts.success
      failedCount += counts.failed
      skippedCount += counts.skipped
    } else if (selectedAction.value === 'export') {
      const exportableKeys = selectedKeys.filter((key) => canExportOAuthCredential(key))
      const exportedEntries: Array<Record<string, unknown> | null> = Array.from({ length: exportableKeys.length }, () => null)

      skippedCount += selectedKeys.length - exportableKeys.length
      progressDone.value = 0
      progressTotal.value = exportableKeys.length
      if (skippedCount > 0) {
        progressLabel.value = t('feedback.pool.runningSkipping', { action: actionLabel, count: skippedCount })
      }

      let cursor = 0
      const CONCURRENCY = props.batchConcurrency || 8
      const runNext = async (): Promise<void> => {
        while (cursor < exportableKeys.length) {
          const idx = cursor++
          const key = exportableKeys[idx]
          try {
            exportedEntries[idx] = await exportKey(key.key_id)
            successCount += 1
          } catch (err) {
            failedCount += 1
            log.error('Pool account credential export failed', err)
          } finally {
            progressDone.value += 1
          }
        }
      }

      const workers = Array.from(
        { length: Math.min(CONCURRENCY, exportableKeys.length) },
        () => runNext(),
      )
      await Promise.all(workers)

      const exportedData = exportedEntries.filter((item): item is Record<string, unknown> => item !== null)
      if (exportedData.length > 0) {
        downloadJsonFile(exportedData, getBatchExportFilename())
      }
    } else if (selectedAction.value === 'delete') {
      const targetIds = selectedKeys.map((key) => key.key_id)
      const BATCH_SIZE = 2000
      const totalBatches = Math.ceil(targetIds.length / BATCH_SIZE)

      for (let i = 0; i < targetIds.length; i += BATCH_SIZE) {
        const batchIndex = Math.floor(i / BATCH_SIZE) + 1
        const batch = targetIds.slice(i, i + BATCH_SIZE)
        if (totalBatches > 1) {
          progressLabel.value = t('feedback.pool.runningBatch', { action: actionLabel, index: batchIndex, total: totalBatches })
        }

        try {
          const result = await batchActionPoolKeys(props.providerId, {
            key_ids: batch,
            action: 'delete',
          })

          if (result.task_id) {
            progressLabel.value = t('feedback.pool.runningBackground', { action: actionLabel })
            const taskResult = await pollDeleteTask(props.providerId, result.task_id, i)
            successCount += taskResult.deleted
            if (taskResult.status === 'failed') {
              failedCount += batch.length - taskResult.deleted
            }
          } else {
            successCount += result.affected
          }
        } catch (err) {
          log.error('Pool account batch delete failed', err)
          failedCount += batch.length
        }

        progressDone.value = Math.min(i + BATCH_SIZE, targetIds.length)
      }
    } else if (['enable', 'disable', 'clear_proxy', 'set_proxy', 'update_settings'].includes(selectedAction.value)) {
      const targetIds = selectedKeys.map((key) => key.key_id)
      const BATCH_SIZE = 2000
      const totalBatches = Math.ceil(targetIds.length / BATCH_SIZE)

      for (let i = 0; i < targetIds.length; i += BATCH_SIZE) {
        const batchIndex = Math.floor(i / BATCH_SIZE) + 1
        const batch = targetIds.slice(i, i + BATCH_SIZE)
        if (totalBatches > 1) {
          progressLabel.value = t('feedback.pool.runningBatch', { action: actionLabel, index: batchIndex, total: totalBatches })
        }

        const payload = selectedAction.value === 'set_proxy'
          ? { node_id: proxyNodeIdForAction.value, enabled: true }
          : selectedAction.value === 'update_settings'
            ? buildPoolKeySettingsPatch(settingsSelection, settingsDraft)
            : undefined

        try {
          const result = await batchActionPoolKeys(props.providerId, {
            key_ids: batch,
            action: selectedAction.value as 'enable' | 'disable' | 'clear_proxy' | 'set_proxy' | 'update_settings',
            ...(payload ? { payload: { ...payload } } : {}),
          })
          successCount += result.affected
        } catch (err) {
          log.error('Pool account batch action failed', err)
          failedCount += batch.length
        }

        progressDone.value = Math.min(i + BATCH_SIZE, targetIds.length)
      }
    } else {
      const CONCURRENCY = props.batchConcurrency || 8
      const tasks: Array<() => Promise<'success' | 'skip'>> = []
      for (const key of selectedKeys) {
        if (selectedAction.value === 'refresh_oauth' && !canRefreshOAuthCredential(key)) {
          skippedCount += 1
          progressDone.value += 1
          continue
        }
        tasks.push(() => refreshProviderOAuth(key.key_id).then(() => 'success' as const))
      }
      progressTotal.value = selectedKeys.length

      let cursor = 0
      const runNext = async (): Promise<void> => {
        while (cursor < tasks.length) {
          const idx = cursor++
          try {
            await tasks[idx]()
            successCount += 1
          } catch {
            failedCount += 1
          }
          progressDone.value += 1
        }
      }
      const workers = Array.from({ length: Math.min(CONCURRENCY, tasks.length) }, () => runNext())
      await Promise.all(workers)
    }

    lastResultMessage.value = t('feedback.pool.actionCompleted', { success: successCount, failed: failedCount, skipped: skippedCount })
    if (failedCount > 0 || (selectedAction.value === 'export' && successCount === 0)) warning(lastResultMessage.value)
    else success(lastResultMessage.value)

    actionPhaseMs = performance.now() - actionStartedAt
    if (selectedAction.value !== 'export') {
      emit('changed')
    }
  } catch (err) {
    showError(parseApiError(err, t('ui.batchActionFailed')))
  } finally {
    log.info('Pool account batch action timing', {
      providerId: props.providerId,
      action: selectedAction.value,
      requestedCount,
      resolvedCount,
      successCount,
      failedCount,
      skippedCount,
      actionPhaseMs: Math.round(actionPhaseMs),
      totalMs: Math.round(performance.now() - actionStartedAt),
    })
    executing.value = false
    progressTotal.value = 0
    progressDone.value = 0
    progressLabel.value = ''
  }
}

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return
    const initialAction = props.initialAction || null
    lastResultMessage.value = ''
    selectedAction.value = initialAction || 'refresh_quota'
    proxyNodeIdForAction.value = ''
    Object.assign(settingsSelection, createPoolKeyBatchSettingSelection())
    Object.assign(settingsDraft, createPoolKeyBatchSettingsDraft())
    proxyNodesStore.ensureLoaded()
    if (initialAction) {
      void nextTick(() => {
        if (!props.modelValue || props.initialAction !== initialAction) return
        handleActionButtonClick(initialAction)
      })
    }
  },
  { immediate: true },
)

</script>
