<template>
  <Dialog
    :model-value="modelValue"
    :title="t('ui.providerBulkActions')"
    :description="dialogDescription"
    :icon="Users"
    size="2xl"
    persistent
    @update:model-value="handleDialogUpdate"
  >
    <div class="space-y-3.5">
      <!-- 步骤 1：选择动作（顶部分段控件） -->
      <div>
        <div class="grid grid-cols-3 gap-1.5 rounded-xl border bg-muted/30 p-1.5">
          <button
            v-for="action in actionOptions"
            :key="action.value"
            type="button"
            class="group flex items-center justify-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-all disabled:opacity-60"
            :class="getSegmentClass(action)"
            :disabled="executing"
            @click="selectedAction = action.value"
          >
            <component
              :is="action.icon"
              class="h-4 w-4 shrink-0"
            />
            <span>{{ action.label }}</span>
          </button>
        </div>
        <p
          class="mt-2 flex items-center gap-1.5 px-1 text-xs leading-relaxed"
          :class="selectedActionMeta?.destructive ? 'text-destructive' : 'text-muted-foreground'"
        >
          <AlertTriangle
            v-if="selectedActionMeta?.destructive"
            class="h-3.5 w-3.5 shrink-0"
          />
          <span>{{ selectedActionMeta?.hint }}</span>
        </p>
      </div>

      <!-- 步骤 2：筛选 + 选择 -->
      <div class="flex flex-wrap items-center gap-2">
        <div class="relative min-w-0 flex-1">
          <Search class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            v-model="searchText"
            class="h-9 w-full pl-9"
            :placeholder="t('ui.searchProviderNameTypeNotes')"
            :disabled="executing"
          />
        </div>
        <label class="flex h-9 cursor-pointer items-center gap-2 rounded-md border bg-background px-3 text-xs text-muted-foreground transition-colors hover:bg-muted/40">
          <Checkbox
            :checked="allFilteredSelected"
            :disabled="filteredProviders.length === 0 || executing"
            @update:checked="toggleFilteredSelection"
          />
          <span>{{ t('ui.selectAll') }}{{ searchText.trim() ? t('ui.filteredResults') : t('ui.thisPage') }}</span>
        </label>
        <Button
          variant="ghost"
          size="sm"
          class="h-9 px-3 text-xs"
          :disabled="selectedCount === 0 || executing"
          @click="clearSelection"
        >
          {{ t('ui.clearSelection') }}
        </Button>
      </div>

      <!-- 提供商列表（占满整宽） -->
      <div class="flex min-w-0 flex-col overflow-hidden rounded-lg border">
        <div class="grid grid-cols-[1.75rem_minmax(0,1fr)_8.5rem] items-center gap-2 border-b bg-muted/30 px-3 py-2 text-[11px] font-medium uppercase tracking-wide text-muted-foreground">
          <Checkbox
            :checked="allFilteredSelected"
            :disabled="filteredProviders.length === 0 || executing"
            @update:checked="toggleFilteredSelection"
          />
          <span class="normal-case tracking-normal">{{ t('nav.providers') }}</span>
          <span class="text-right normal-case tracking-normal">{{ t('ui.resourcesActiveTotal') }}</span>
        </div>
        <div class="max-h-[min(52vh,440px)] overflow-y-auto">
          <div
            v-if="filteredProviders.length === 0"
            class="flex flex-col items-center justify-center gap-1.5 py-14 text-center"
          >
            <Search class="h-6 w-6 text-muted-foreground/50" />
            <span class="text-sm text-muted-foreground">{{ t('ui.noMatchingProviders0a5b7ee') }}</span>
          </div>
          <label
            v-for="provider in filteredProviders"
            :key="provider.id"
            class="grid cursor-pointer grid-cols-[1.75rem_minmax(0,1fr)_8.5rem] items-center gap-2 border-b px-3 py-2.5 transition-colors last:border-b-0"
            :class="rowClass(provider.id)"
          >
            <Checkbox
              :checked="selectedIdSet.has(provider.id)"
              :disabled="executing"
              @update:checked="(checked) => toggleProvider(provider.id, checked)"
            />
            <div class="min-w-0">
              <div class="flex items-center gap-1.5">
                <span class="truncate text-sm font-medium">{{ provider.name }}</span>
                <Badge
                  :variant="provider.is_active ? 'success' : 'secondary'"
                  class="shrink-0 text-[10px]"
                >
                  {{ provider.is_active ? t('ui.activee85e590') : t('ui.disable4e6fd0e') }}
                </Badge>
              </div>
              <div class="mt-0.5 flex min-w-0 items-center gap-1.5 text-[11px] text-muted-foreground">
                <span class="shrink-0 rounded bg-muted px-1.5 py-px font-mono text-[10px]">{{ provider.provider_type || 'custom' }}</span>
                <span
                  v-if="provider.description"
                  class="truncate"
                >{{ provider.description }}</span>
              </div>
            </div>
            <div class="flex flex-col items-end gap-0.5 text-[11px] leading-tight text-muted-foreground">
              <span><span class="text-muted-foreground/70">{{ t('ui.endpoint52fff16') }}</span> <span class="font-medium tabular-nums text-foreground/80">{{ provider.active_endpoints }}/{{ provider.total_endpoints }}</span></span>
              <span><span class="text-muted-foreground/70">{{ t('ui.account') }}</span> <span class="font-medium tabular-nums text-foreground/80">{{ provider.active_keys }}/{{ provider.total_keys }}</span></span>
              <span><span class="text-muted-foreground/70">{{ t('ui.model') }}</span> <span class="font-medium tabular-nums text-foreground/80">{{ provider.active_models }}/{{ provider.total_models }}</span></span>
            </div>
          </label>
        </div>
      </div>

      <!-- 执行进度 / 结果 -->
      <div
        v-if="executing"
        class="space-y-1.5 rounded-lg border bg-muted/15 px-3 py-2.5"
      >
        <div class="flex items-center justify-between text-xs">
          <span class="truncate text-foreground">{{ progressLabel }}</span>
          <span class="shrink-0 font-medium tabular-nums text-muted-foreground">{{ progressDone }} / {{ progressTotal }}</span>
        </div>
        <div class="h-1.5 overflow-hidden rounded-full bg-muted">
          <div
            class="h-full rounded-full transition-all duration-150"
            :class="selectedActionMeta?.destructive ? 'bg-destructive' : 'bg-primary'"
            :style="{ width: `${progressPercent}%` }"
          />
        </div>
      </div>

      <div
        v-else-if="lastResultMessage"
        class="rounded-lg border bg-background px-3 py-2.5 text-xs text-muted-foreground"
      >
        {{ lastResultMessage }}
      </div>
    </div>

    <template #footer>
      <div class="flex w-full flex-wrap items-center justify-between gap-3">
        <div class="flex items-center gap-2 text-xs text-muted-foreground">
          <span>{{ t('ui.selected') }} <span class="font-semibold tabular-nums text-foreground">{{ selectedCount }}</span></span>
          <span class="text-border">·</span>
          <span>{{ t('ui.thisPage') }} <span class="tabular-nums">{{ providers.length }}</span></span>
          <span class="text-emerald-600 dark:text-emerald-400">{{ t('ui.activee85e590') }} <span class="tabular-nums">{{ activeProviderCount }}</span></span>
          <span>{{ t('ui.disable4e6fd0e') }} <span class="tabular-nums">{{ inactiveProviderCount }}</span></span>
        </div>
        <div class="flex items-center gap-2">
          <Button
            variant="outline"
            :disabled="executing"
            @click="emit('update:modelValue', false)"
          >
            {{ t('ui.close') }}
          </Button>
          <Button
            :variant="selectedAction === 'delete' ? 'destructive' : 'default'"
            :disabled="!canExecute"
            @click="confirmAndExecute"
          >
            {{ executeButtonLabel }}
          </Button>
        </div>
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, ref, watch, type Component } from 'vue'
import { AlertTriangle, Power, PowerOff, Search, Trash2, Users } from 'lucide-vue-next'
import { Badge, Button, Checkbox, Dialog, Input } from '@/components/ui'
import {
  deleteProvider,
  getProviderDeleteTask,
  updateProvider,
  type ProviderWithEndpointsSummary,
} from '@/api/endpoints'
import { useConfirm } from '@/composables/useConfirm'
import { useToast } from '@/composables/useToast'
import { parseApiError } from '@/utils/errorParser'
import { log } from '@/utils/logger'

type ProviderBatchAction = 'enable' | 'disable' | 'delete'

interface ProviderBatchActionOption {
  value: ProviderBatchAction
  label: string
  hint: string
  icon: Component
  destructive?: boolean
}

const props = defineProps<{
  modelValue: boolean
  providers: ProviderWithEndpointsSummary[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  changed: []
}>()

const { t } = useI18n()

const { confirm } = useConfirm()
const { success, warning, error: showError } = useToast()

const actionOptions: ProviderBatchActionOption[] = [
  { value: 'enable', get label() { return t('ui.enablef4f0ead') }, get hint() { return t('providers.migration.enableSelectedProvidersGloballyEach9d0f23') }, icon: Power },
  { value: 'disable', get label() { return t('ui.disable4e6fd0e') }, get hint() { return t('providers.migration.stopSelectedProvidersFromScheduling7244c5') }, icon: PowerOff },
  { value: 'delete', get label() { return t('ui.delete') }, get hint() { return t('ui.permanentlyDeleteTheSelectedProvidersEndpointsAccountsAndConfigurationThis') }, icon: Trash2, destructive: true },
]

const SEGMENT_ACCENT: Record<ProviderBatchAction, string> = {
  enable: 'bg-background text-emerald-600 shadow-sm ring-1 ring-emerald-500/25 dark:text-emerald-400',
  disable: 'bg-background text-primary shadow-sm ring-1 ring-primary/25',
  delete: 'bg-background text-destructive shadow-sm ring-1 ring-destructive/30',
}

const searchText = ref('')
const selectedProviderIds = ref<string[]>([])
const selectedAction = ref<ProviderBatchAction>('disable')
const executing = ref(false)
const progressDone = ref(0)
const progressTotal = ref(0)
const progressLabel = ref('')
const lastResultMessage = ref('')

const dialogDescription = computed(() => t('providers.migration.batchUpdateTheGlobalStatusd11f2c'))
const selectedIdSet = computed(() => new Set(selectedProviderIds.value))
const selectedCount = computed(() => selectedProviderIds.value.length)
const selectedActionLabel = computed(() => actionOptions.find(action => action.value === selectedAction.value)?.label || '')
const selectedActionMeta = computed(() => actionOptions.find(action => action.value === selectedAction.value))
const executeButtonLabel = computed(() => {
  if (executing.value) return t('ui.runningfeed25e')
  if (selectedCount.value > 0) return t('providers.migration.items41e151', { value: String(selectedActionLabel.value), value_2: String(selectedCount.value) })
  return t('providers.migration.execute2e1e8c', { value: String(selectedActionLabel.value) })
})
const activeProviderCount = computed(() => props.providers.filter(provider => provider.is_active).length)
const inactiveProviderCount = computed(() => props.providers.length - activeProviderCount.value)
const providerById = computed(() => new Map(props.providers.map(provider => [provider.id, provider])))
const filteredProviders = computed(() => {
  const keyword = searchText.value.trim().toLowerCase()
  if (!keyword) return props.providers
  return props.providers.filter((provider) => {
    return [
      provider.name,
      provider.provider_type,
      provider.description,
      provider.website,
    ].some(value => String(value || '').toLowerCase().includes(keyword))
  })
})
const allFilteredSelected = computed(() => {
  return filteredProviders.value.length > 0
    && filteredProviders.value.every(provider => selectedIdSet.value.has(provider.id))
})
const progressPercent = computed(() => {
  if (progressTotal.value <= 0) return 0
  return Math.min(100, Math.round((progressDone.value / progressTotal.value) * 100))
})
const canExecute = computed(() => selectedCount.value > 0 && !executing.value)

function toggleProvider(providerId: string, checked: boolean): void {
  const next = new Set(selectedProviderIds.value)
  if (checked) next.add(providerId)
  else next.delete(providerId)
  selectedProviderIds.value = [...next]
}

function toggleFilteredSelection(checked: boolean): void {
  const next = new Set(selectedProviderIds.value)
  for (const provider of filteredProviders.value) {
    if (checked) next.add(provider.id)
    else next.delete(provider.id)
  }
  selectedProviderIds.value = [...next]
}

function clearSelection(): void {
  selectedProviderIds.value = []
}

function handleDialogUpdate(open: boolean): void {
  if (executing.value && !open) return
  emit('update:modelValue', open)
}

function getSegmentClass(action: ProviderBatchActionOption): string {
  if (selectedAction.value !== action.value) {
    return 'text-muted-foreground hover:bg-background/60 hover:text-foreground'
  }
  return SEGMENT_ACCENT[action.value]
}

function rowClass(providerId: string): string {
  if (!selectedIdSet.value.has(providerId)) {
    return 'hover:bg-muted/40'
  }
  return selectedActionMeta.value?.destructive
    ? 'bg-destructive/5 hover:bg-destructive/10'
    : 'bg-primary/5 hover:bg-primary/10'
}

async function confirmAndExecute(): Promise<void> {
  if (!canExecute.value) return
  const action = actionOptions.find(item => item.value === selectedAction.value)
  const actionLabel = action?.label || t('ui.batchActions')
  const confirmed = await confirm({
    title: t('providers.migration.batchProviders9d3da1', { value: String(actionLabel) }),
    message: selectedAction.value === 'delete'
      ? t('providers.migration.deleteProvidersAndAllTheira6a45b', { value: String(selectedCount.value) })
      : t('providers.migration.applyToProviders7d8b7d', { value: String(selectedCount.value), value_2: String(actionLabel) }),
    confirmText: selectedAction.value === 'delete' ? t('ui.confirmDeletion') : t('ui.confirmAction'),
    ...(selectedAction.value === 'delete' ? { variant: 'destructive' as const } : {}),
  })
  if (!confirmed) return
  await executeBatchAction()
}

const DELETE_POLL_INTERVAL_MS = 2000
const DELETE_POLL_MAX_MS = 30 * 60 * 1000
const DELETE_POLL_MAX_FAILURES = 3

async function pollProviderDeleteTask(providerId: string, taskId: string): Promise<void> {
  const deadline = Date.now() + DELETE_POLL_MAX_MS
  let consecutiveFailures = 0
  while (Date.now() < deadline) {
    try {
      const task = await getProviderDeleteTask(providerId, taskId)
      consecutiveFailures = 0
      if (task.status === 'completed') return
      if (task.status === 'failed') {
        throw new Error(task.message || 'provider delete task failed')
      }
    } catch (err) {
      consecutiveFailures += 1
      if (consecutiveFailures >= DELETE_POLL_MAX_FAILURES) {
        throw err
      }
    }
    await new Promise(resolve => setTimeout(resolve, DELETE_POLL_INTERVAL_MS))
  }
  throw new Error('provider delete task timeout')
}

async function executeBatchAction(): Promise<void> {
  if (executing.value) return

  const targets = selectedProviderIds.value
    .map(id => providerById.value.get(id))
    .filter((provider): provider is ProviderWithEndpointsSummary => Boolean(provider))
  if (targets.length === 0) {
    warning(t('ui.selectProvidersFirst'))
    return
  }

  executing.value = true
  progressDone.value = 0
  progressTotal.value = targets.length
  lastResultMessage.value = ''
  let successCount = 0
  let failedCount = 0

  try {
    for (const provider of targets) {
      progressLabel.value = t('providers.migration.message9bfe53', { value: String(selectedActionLabel.value), value_2: String(provider.name) })
      try {
        if (selectedAction.value === 'delete') {
          const result = await deleteProvider(provider.id)
          await pollProviderDeleteTask(provider.id, result.task_id)
        } else {
          await updateProvider(provider.id, { is_active: selectedAction.value === 'enable' })
        }
        successCount += 1
      } catch (err) {
        failedCount += 1
        log.error('Provider batch action failed', err)
      } finally {
        progressDone.value += 1
      }
    }

    lastResultMessage.value = t('providers.migration.completedSucceededFaileded7158', { value: String(successCount), value_2: String(failedCount) })
    if (failedCount > 0) warning(lastResultMessage.value)
    else success(lastResultMessage.value)
    if (successCount > 0) {
      clearSelection()
      emit('changed')
    }
  } catch (err) {
    showError(parseApiError(err, t('ui.providerBulkActionFailed')), t('ui.error'))
  } finally {
    executing.value = false
    progressDone.value = 0
    progressTotal.value = 0
    progressLabel.value = ''
  }
}

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return
    searchText.value = ''
    selectedProviderIds.value = []
    selectedAction.value = 'disable'
    lastResultMessage.value = ''
  },
)

watch(
  () => props.providers.map(provider => provider.id),
  () => {
    const availableIds = new Set(props.providers.map(provider => provider.id))
    selectedProviderIds.value = selectedProviderIds.value.filter(id => availableIds.has(id))
  },
)

</script>
