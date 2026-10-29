<template>
  <Dialog
    :model-value="modelValue"
    :title="t('ui.poolProxyDistribution')"
    :description="t('ui.selectAPoolAndProxyNodesGenerateAnAssignmentPreview')"
    size="3xl"
    persistent
    @update:model-value="handleOpenChange"
  >
    <div class="space-y-4">
      <div class="grid gap-3 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)]">
        <div class="space-y-1.5">
          <Label>{{ t('ui.pool') }}</Label>
          <Select
            :model-value="selectedProviderId"
            :disabled="loadingPools || executing || poolOptions.length === 0"
            @update:model-value="(value: string) => selectedProviderId = value"
          >
            <SelectTrigger class="h-9 text-xs">
              <SelectValue
                :placeholder="loadingPools
                  ? t('ui.loadingPools')
                  : poolOptions.length === 0
                    ? t('ui.noAvailablePools')
                    : t('ui.selectPool')"
              />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="pool in poolOptions"
                :key="pool.value"
                :value="pool.value"
              >
                {{ pool.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div class="space-y-1.5">
          <div class="flex items-center justify-between gap-2">
            <Label>{{ t('ui.proxyNodes23f1b6a') }}</Label>
            <Button
              variant="ghost"
              size="sm"
              class="h-7 px-2 text-[11px]"
              :disabled="executing || proxyNodeOptions.length === 0"
              @click="selectAllProxyNodes"
            >
              {{ t('ui.all5c55a67') }}
            </Button>
          </div>
          <MultiSelect
            v-model="selectedProxyNodeIds"
            :options="proxyNodeOptions"
            :placeholder="t('ui.selectProxyNode')"
            :empty-text="t('ui.noAvailableProxyNodes')"
            trigger-class="h-9 text-xs"
            dropdown-min-width="22rem"
            :disabled="executing || proxyNodeOptions.length === 0"
          />
        </div>
      </div>

      <div class="grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-end">
        <div class="space-y-1.5">
          <Label>{{ t('ui.mode') }}</Label>
          <div class="grid grid-cols-2 gap-2 rounded-lg border border-border/60 bg-muted/30 p-1">
            <Button
              type="button"
              class="h-8 text-xs"
              :variant="mode === 'fill' ? 'default' : 'ghost'"
              :disabled="executing"
              @click="mode = 'fill'"
            >
              <Shuffle class="mr-1.5 h-3.5 w-3.5" /> {{ t('ui.balanceFill') }}
            </Button>
            <Button
              type="button"
              class="h-8 text-xs"
              :variant="mode === 'rewrite' ? 'default' : 'ghost'"
              :disabled="executing"
              @click="mode = 'rewrite'"
            >
              <RefreshCw class="mr-1.5 h-3.5 w-3.5" /> {{ t('ui.forceRebalance') }}
            </Button>
          </div>
        </div>
        <Button
          variant="outline"
          class="h-9 px-3 text-xs"
          :disabled="loadingKeys || executing || !canBuildPlan"
          @click="loadKeysAndBuildPlan"
        >
          <RefreshCw
            class="mr-1.5 h-3.5 w-3.5"
            :class="{ 'animate-spin': loadingKeys }"
          />
          {{ plan ? t('ui.refreshPreview') : t('ui.generatePreview') }}
        </Button>
      </div>

      <div
        v-if="loadingPools || loadingKeys"
        class="rounded-lg border border-border/60 bg-muted/20 px-3 py-6 text-center text-sm text-muted-foreground"
      >
        {{ loadingText }}
      </div>

      <div
        v-else-if="!plan"
        class="rounded-lg border border-border/60 bg-muted/20 px-3 py-6 text-center text-sm text-muted-foreground"
      >
        {{ t('ui.selectAPoolAndProxyNodesToGenerateAnAssignment') }}
      </div>

      <div
        v-else
        class="space-y-3"
      >
        <div class="grid gap-2 text-xs sm:grid-cols-4">
          <div class="rounded-lg border bg-background px-3 py-2">
            <div class="text-muted-foreground">
              {{ t('ui.poolAccounts') }}
            </div>
            <div class="mt-1 text-base font-semibold tabular-nums">
              {{ plan.totalKeys }}
            </div>
          </div>
          <div class="rounded-lg border bg-background px-3 py-2">
            <div class="text-muted-foreground">
              {{ t('ui.proxyNodes23f1b6a') }}
            </div>
            <div class="mt-1 text-base font-semibold tabular-nums">
              {{ plan.nodeCount }}
            </div>
          </div>
          <div class="rounded-lg border bg-background px-3 py-2">
            <div class="text-muted-foreground">
              {{ t('ui.perNodeLimit') }}
            </div>
            <div class="mt-1 text-base font-semibold tabular-nums">
              {{ plan.maxPerNode }}
            </div>
          </div>
          <div class="rounded-lg border bg-background px-3 py-2">
            <div class="text-muted-foreground">
              {{ t('ui.pendingWrite') }}
            </div>
            <div class="mt-1 text-base font-semibold tabular-nums">
              {{ plan.changedCount }}
            </div>
          </div>
        </div>

        <div class="rounded-lg border bg-muted/20 px-3 py-2 text-xs text-muted-foreground">
          <span v-if="mode === 'fill'"> {{ t('ui.keep') }} {{ plan.retainedCount }} {{ t('ui.validExistingAssignmentsProcess') }} {{ plan.overflowCount }} {{ t('ui.excessAssignmentsAnd') }} {{ plan.outsideSelectedProxyCount }} {{ t('ui.assignmentsToUnselectedNodes') }} </span>
          <span v-else> {{ t('ui.reassignAll') }} {{ plan.totalKeys }} {{ t('ui.accountsRandomlyAcrossTheSelectedProxyNodes') }} </span>
        </div>

        <div class="max-h-[360px] overflow-y-auto rounded-lg border">
          <div
            v-for="item in assignmentRows"
            :key="item.nodeId"
            class="grid gap-2 border-b px-3 py-2 last:border-b-0 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center"
          >
            <div class="min-w-0">
              <div class="truncate text-sm font-medium">
                {{ item.nodeName }}
              </div>
              <div class="mt-0.5 truncate text-xs text-muted-foreground">
                {{ item.nodeMeta }}
              </div>
            </div>
            <div class="flex flex-wrap items-center gap-2 text-xs">
              <Badge variant="outline">
                {{ t('ui.target') }} {{ item.targetCount }}
              </Badge>
              <Badge variant="secondary">
                {{ t('ui.keep') }} {{ item.retainedCount }}
              </Badge>
              <Badge :variant="item.changedCount > 0 ? 'default' : 'outline'">
                {{ t('ui.write') }} {{ item.changedCount }}
              </Badge>
            </div>
          </div>
        </div>

        <div
          v-if="executing && progressTotal > 0"
          class="space-y-1"
        >
          <div class="flex items-center justify-between text-xs text-muted-foreground">
            <span>{{ t('ui.writingProxyAssignments') }}</span>
            <span>{{ progressDone }} / {{ progressTotal }}</span>
          </div>
          <div class="h-1.5 w-full overflow-hidden rounded-full bg-muted">
            <div
              class="h-full rounded-full bg-primary transition-all duration-150"
              :style="{ width: `${Math.round((progressDone / progressTotal) * 100)}%` }"
            />
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <Button
        variant="outline"
        :disabled="executing"
        @click="handleOpenChange(false)"
      >
        {{ t('ui.close') }}
      </Button>
      <Button
        :disabled="executing || !plan || plan.changedCount === 0"
        @click="executePlan"
      >
        <Loader2
          v-if="executing"
          class="mr-1.5 h-3.5 w-3.5 animate-spin"
        />
        {{ executing ? t('ui.runningfeed25e') : t('ui.assign') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, ref, watch } from 'vue'
import { Loader2, RefreshCw, Shuffle } from 'lucide-vue-next'
import {
  Badge,
  Button,
  Dialog,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui'
import { MultiSelect } from '@/components/common'
import type { MultiSelectOption } from '@/components/common/MultiSelect.vue'
import { useConfirm } from '@/composables/useConfirm'
import { useToast } from '@/composables/useToast'
import { parseApiError } from '@/utils/errorParser'
import { useProxyNodesStore } from '@/stores/proxy-nodes'
import { formatRegion } from '@/utils/region'
import {
  batchActionPoolKeys,
  getPoolOverview,
  listPoolKeys,
  type PoolKeyDetail,
  type PoolOverviewItem,
} from '@/api/endpoints/pool'
import {
  buildPoolProxyDistributionPlan,
  type PoolProxyDistributionMode,
  type PoolProxyDistributionPlan,
} from '@/features/pool/utils/poolProxyDistribution'
import type { ProxyNode } from '@/api/proxy-nodes'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  changed: []
}>()

const { t } = useI18n()

const { success, error: showError, warning } = useToast()
const { confirm } = useConfirm()
const proxyNodesStore = useProxyNodesStore()

const loadingPools = ref(false)
const loadingKeys = ref(false)
const executing = ref(false)
const pools = ref<PoolOverviewItem[]>([])
const selectedProviderId = ref('')
const selectedProxyNodeIds = ref<string[]>([])
const mode = ref<PoolProxyDistributionMode>('fill')
const poolKeys = ref<PoolKeyDetail[]>([])
const plan = ref<PoolProxyDistributionPlan | null>(null)
const loadedKeysProviderId = ref('')
const progressDone = ref(0)
const progressTotal = ref(0)
const loadingKeyPage = ref(0)

let poolLoadRequestId = 0
let keyLoadRequestId = 0

const poolOptions = computed<MultiSelectOption[]>(() =>
  pools.value
    .filter(pool => pool.pool_enabled)
    .map(pool => ({
      value: pool.provider_id,
      label: `${pool.provider_name} (${pool.total_keys})`,
    })),
)

const selectableProxyNodes = computed<ProxyNode[]>(() => {
  const online = proxyNodesStore.onlineNodes
  return online.length > 0 ? online : []
})

const proxyNodeOptions = computed<MultiSelectOption[]>(() =>
  selectableProxyNodes.value.map(node => ({
    value: node.id,
    label: `${node.name}${node.region ? ` · ${formatRegion(node.region, '')}` : ''} (${node.ip}:${node.port})`,
  })),
)

const selectedNodes = computed(() => {
  const selectedSet = new Set(selectedProxyNodeIds.value)
  return selectableProxyNodes.value.filter(node => selectedSet.has(node.id))
})

const canBuildPlan = computed(() =>
  Boolean(selectedProviderId.value) && selectedNodes.value.length > 0,
)

const loadingText = computed(() => {
  if (loadingKeys.value) {
    return loadingKeyPage.value > 0
      ? t('feedback.pool.loadingAccountsPage', { page: loadingKeyPage.value })
      : t('ui.loadingAccounts')
  }
  return t('ui.loadingPoolsAndProxyNodes')
})

const assignmentRows = computed(() => {
  const nodeById = new Map(selectableProxyNodes.value.map(node => [node.id, node]))
  return (plan.value?.assignments ?? []).map((assignment) => {
    const node = nodeById.get(assignment.nodeId)
    return {
      nodeId: assignment.nodeId,
      nodeName: node?.name || assignment.nodeId,
      nodeMeta: node ? `${node.ip}:${node.port}${node.region ? ` | ${formatRegion(node.region, '')}` : ''}` : assignment.nodeId,
      targetCount: assignment.targetCount,
      retainedCount: assignment.retainedKeys.length,
      changedCount: assignment.changedKeys.length,
    }
  })
})

function handleOpenChange(open: boolean): void {
  emit('update:modelValue', open)
}

function selectAllProxyNodes(): void {
  selectedProxyNodeIds.value = proxyNodeOptions.value.map(option => option.value)
}

function resetPreview(): void {
  plan.value = null
  poolKeys.value = []
  loadedKeysProviderId.value = ''
  progressDone.value = 0
  progressTotal.value = 0
  loadingKeyPage.value = 0
}

async function loadInitialData(): Promise<void> {
  const requestId = ++poolLoadRequestId
  loadingPools.value = true
  resetPreview()
  try {
    await proxyNodesStore.ensureLoaded()
    const overview = await getPoolOverview({ cacheTtlMs: 0 })
    if (requestId !== poolLoadRequestId) return
    pools.value = Array.isArray(overview.items) ? overview.items : []
    if (!selectedProviderId.value || !poolOptions.value.some(option => option.value === selectedProviderId.value)) {
      selectedProviderId.value = poolOptions.value[0]?.value ?? ''
    }
    selectAllProxyNodes()
  } catch (err) {
    if (requestId !== poolLoadRequestId) return
    showError(parseApiError(err, t('ui.failedToLoadPoolInformation')))
  } finally {
    if (requestId === poolLoadRequestId) {
      loadingPools.value = false
    }
  }
}

async function loadAllPoolKeys(providerId: string): Promise<PoolKeyDetail[]> {
  const pageSize = 200
  let page = 1
  const keys: PoolKeyDetail[] = []

  while (true) {
    loadingKeyPage.value = page
    const result = await listPoolKeys(providerId, {
      page,
      page_size: pageSize,
      status: 'all',
    }, {
      cacheTtlMs: 0,
    })
    const pageKeys = Array.isArray(result.keys) ? result.keys : []
    keys.push(...pageKeys)
    if (keys.length >= result.total || pageKeys.length === 0) {
      return keys
    }
    page += 1
  }
}

async function loadKeysAndBuildPlan(): Promise<void> {
  if (!canBuildPlan.value) {
    warning(t('ui.selectAPoolAndProxyNodesFirst'))
    return
  }

  const providerId = selectedProviderId.value
  const requestId = ++keyLoadRequestId
  loadingKeys.value = true
  plan.value = null
  try {
    const keys = await loadAllPoolKeys(providerId)
    if (requestId !== keyLoadRequestId || selectedProviderId.value !== providerId) return
    poolKeys.value = keys
    loadedKeysProviderId.value = providerId
    buildPreviewPlan()
  } catch (err) {
    if (requestId !== keyLoadRequestId) return
    showError(parseApiError(err, t('ui.failedToLoadPoolAccounts')))
  } finally {
    if (requestId === keyLoadRequestId) {
      loadingKeys.value = false
      loadingKeyPage.value = 0
    }
  }
}

function buildPreviewPlan(): void {
  if (!canBuildPlan.value || loadedKeysProviderId.value !== selectedProviderId.value) {
    plan.value = null
    return
  }
  plan.value = buildPoolProxyDistributionPlan({
    mode: mode.value,
    keys: poolKeys.value,
    nodes: selectedNodes.value.map(node => ({ id: node.id, name: node.name })),
  })
}

async function executePlan(): Promise<void> {
  if (!plan.value || !selectedProviderId.value) return
  if (plan.value.changedCount === 0) {
    success(t('ui.currentAssignmentsAlreadyMeetTheTargetNoChangesAreNeeded'))
    emit('changed')
    emit('update:modelValue', false)
    return
  }

  const confirmed = await confirm({
    get title() { return t('ui.distributePoolAccountsAcrossProxies') },
    message: t('feedback.pool.distributeConfirm', { count: plan.value.changedCount }),
    get confirmText() { return t('ui.startAssignment') },
    variant: 'warning',
  })
  if (!confirmed || !plan.value) return

  executing.value = true
  progressDone.value = 0
  progressTotal.value = plan.value.changedCount
  const providerId = selectedProviderId.value
  let affected = 0

  try {
    for (const assignment of plan.value.assignments) {
      const keyIds = assignment.changedKeys.map(key => key.key_id)
      for (let index = 0; index < keyIds.length; index += 2000) {
        const batch = keyIds.slice(index, index + 2000)
        if (batch.length === 0) continue
        const result = await batchActionPoolKeys(providerId, {
          key_ids: batch,
          action: 'set_proxy',
          payload: { node_id: assignment.nodeId, enabled: true },
        })
        affected += Number(result.affected || 0)
        progressDone.value += batch.length
      }
    }

    success(t('feedback.pool.distributeCompleted', { count: affected }))
    emit('changed')
    emit('update:modelValue', false)
  } catch (err) {
    showError(parseApiError(err, t('ui.failedToDistributePoolAccountsAcrossProxies')))
  } finally {
    executing.value = false
    progressDone.value = 0
    progressTotal.value = 0
  }
}

watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      void loadInitialData()
    } else {
      keyLoadRequestId += 1
      resetPreview()
    }
  },
)

watch([selectedProviderId, selectedProxyNodeIds, mode], () => {
  if (!props.modelValue || loadingKeys.value || executing.value) return
  if (loadedKeysProviderId.value === selectedProviderId.value && poolKeys.value.length > 0) {
    buildPreviewPlan()
  } else {
    plan.value = null
  }
})

</script>
