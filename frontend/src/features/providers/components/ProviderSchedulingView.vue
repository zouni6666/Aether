<template>
  <section
    class="min-w-0"
    aria-label="提供商调度工作区"
  >
    <div class="grid min-w-0 items-stretch gap-4 xl:grid-cols-[400px_minmax(0,1fr)]">
      <aside
        class="flex min-w-0 flex-col gap-3 xl:min-h-0 xl:[contain:size]"
        aria-label="策略分组设置"
      >
        <Card
          class="flex min-w-0 flex-1 flex-col xl:h-0 xl:min-h-full"
          aria-label="策略分组与调度配置"
        >
          <div class="flex shrink-0 items-center justify-between gap-2 border-b border-border/50 px-3 py-3 sm:py-3.5">
            <h3 class="sr-only">
              策略分组
            </h3>
            <Select
              :model-value="selectedValue"
              :disabled="busy"
              @update:model-value="changeGroup"
            >
              <SelectTrigger
                class="h-8 min-w-0 flex-1 px-3 text-sm"
                aria-label="当前调度策略"
              >
                <SelectValue :placeholder="loading ? '正在加载分组' : '选择策略分组'" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-if="isNewDraft"
                  value="new"
                >
                  新建策略
                </SelectItem>
                <SelectItem
                  v-for="group in groups"
                  :key="group.id"
                  :value="group.id"
                >
                  {{ group.name }}{{ group.is_system_default ? ' · 默认' : '' }}{{ !group.enabled ? ' · 停用' : '' }}
                </SelectItem>
              </SelectContent>
            </Select>
            <div class="flex shrink-0 items-center gap-0.5">
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8"
                :disabled="busy || isNewDraft"
                title="新建分组"
                aria-label="新建策略"
                @click="openCreate"
              >
                <Plus class="h-3.5 w-3.5" />
              </Button>
              <Button
                v-if="draft?.id"
                variant="ghost"
                size="icon"
                class="h-8 w-8 text-muted-foreground hover:text-destructive"
                :disabled="busy"
                title="删除策略分组"
                aria-label="删除策略"
                @click="deleteDialogOpen = true"
              >
                <Trash2 class="h-3.5 w-3.5" />
              </Button>
              <Button
                v-if="draft"
                variant="ghost"
                size="icon"
                class="h-8 w-8"
                :class="{ 'text-primary': draft.is_system_default }"
                :disabled="busy"
                :title="draft.is_system_default ? '取消系统默认（保存后生效）' : '设为系统默认（保存后生效）'"
                aria-label="设为系统默认"
                :aria-pressed="draft.is_system_default"
                @click="draft.is_system_default = !draft.is_system_default"
              >
                <Star
                  class="h-3.5 w-3.5"
                  :class="{ 'fill-current': draft.is_system_default }"
                />
              </Button>
              <Button
                v-if="draft"
                variant="ghost"
                size="icon"
                class="h-8 w-8"
                :class="{ 'text-primary': draftDirty }"
                :disabled="!canSaveDraft"
                :title="saving ? '正在保存…' : billingMultiplierError ?? (!routingSchedulingValid ? '请先选择适用模型' : draftDirty ? '保存修改' : '已保存')"
                aria-label="保存调度"
                :aria-busy="saving"
                @click="saveDraft"
              >
                <Save
                  class="h-3.5 w-3.5"
                  :class="{ 'animate-pulse': saving }"
                />
              </Button>
            </div>
          </div>
          <div class="min-h-0 flex-1 xl:overflow-y-auto">
            <div
              v-if="draft"
              ref="groupMetadata"
              class="min-w-0 space-y-2 border-b border-border/50 p-3"
              aria-label="分组信息"
              :inert="busy"
            >
              <div class="flex min-w-0 items-center gap-2">
                <label class="block min-w-0 flex-1">
                  <span class="sr-only">策略名称</span>
                  <Input
                    v-model="draft.name"
                    size="sm"
                    class="min-w-0"
                    aria-label="策略名称"
                    placeholder="分组名称"
                    :disabled="busy"
                  />
                </label>
                <label class="flex shrink-0 items-center gap-1 text-xs">
                  <span>启用</span>
                  <Switch
                    v-model="draft.enabled"
                    :disabled="busy"
                    aria-label="启用策略"
                  />
                </label>
              </div>
              <div class="flex min-w-0 items-center justify-between gap-3">
                <label class="flex min-w-0 items-center gap-2 text-xs">
                  <span class="shrink-0">分组倍率</span>
                  <Input
                    :model-value="billingMultiplierInput"
                    type="number"
                    min="0"
                    step="any"
                    size="sm"
                    class="w-24 min-w-0"
                    aria-label="分组倍率"
                    :aria-invalid="Boolean(billingMultiplierError)"
                    :aria-describedby="billingMultiplierError ? 'group-billing-multiplier-error' : undefined"
                    :disabled="busy"
                    @update:model-value="updateBillingMultiplier"
                  />
                  <span class="shrink-0 text-muted-foreground">倍</span>
                </label>
                <label class="flex shrink-0 items-center gap-1 text-xs">
                  <span>用户可见</span>
                  <Switch
                    v-model="draft.config_json.user_visible"
                    :disabled="busy"
                    aria-label="用户可见"
                  />
                </label>
              </div>
              <p
                v-if="billingMultiplierError"
                id="group-billing-multiplier-error"
                class="text-xs text-destructive"
              >
                {{ billingMultiplierError }}
              </p>
            </div>
            <RoutingSchedulingPolicyEditor
              v-if="draft"
              :key="draftGeneration"
              ref="routingSchedulingPolicyEditor"
              :config="draft.config_json"
              :initial-selection="initialSchedulingSelection ?? undefined"
              :refresh-revision="providerRevision"
              :global-models="globalModels"
              :loading-models="loadingGlobalModels"
              :models-error="globalModelsError"
              :disabled="busy"
              :aria-busy="saving"
              class="p-3"
              aria-label="策略调度配置"
              layout="config-only"
              sidebar
              @update:config="updateDraftConfig"
              @validity-change="routingSchedulingValid = $event"
              @selection-change="activePolicy = $event"
              @reload-models="loadGlobalModels()"
              @inspect-provider="emit('inspect-provider', $event)"
            />

            <div
              v-if="draft"
              class="min-w-0"
              :inert="busy"
            >
              <section
                class="min-w-0 border-t border-border/50"
                :inert="busy"
              >
                <button
                  type="button"
                  class="flex w-full items-center justify-between gap-3 px-3 py-3 text-left"
                  :aria-expanded="advancedOpen"
                  aria-label="高级设置"
                  @click="advancedOpen = !advancedOpen"
                >
                  <span class="text-sm font-medium">高级设置</span>
                  <ChevronRight
                    class="h-4 w-4 shrink-0 text-muted-foreground transition-transform"
                    :class="{ 'rotate-90': advancedOpen }"
                  />
                </button>
                <div
                  v-show="advancedOpen"
                  class="min-w-0 space-y-4 px-3 pb-3"
                >
                  <section class="min-w-0 space-y-2">
                    <div>
                      <h3 class="text-sm font-medium">
                        系统配置
                      </h3>
                    </div>
                    <div class="grid min-w-0 grid-cols-1 gap-2">
                      <div
                        class="order-1 flex min-w-0 min-h-10 items-center justify-between gap-2 rounded-md border border-border/60 px-2 py-2 text-xs"
                        data-testid="keep-priority-on-conversion"
                      >
                        <div class="flex min-w-0 items-center gap-1.5">
                          <span class="font-medium">格式转换保持优先级</span>
                          <HelpHint
                            portal
                            label="格式转换保持优先级"
                            text="开启后，跨 API 格式转换的候选不会被降级到同格式候选之后；Provider 自身的同名开关仍单独生效。"
                          />
                        </div>
                        <Switch
                          :model-value="keepPriorityOnConversion"
                          :disabled="saving"
                          aria-label="格式转换保持优先级"
                          @update:model-value="updateKeepPriorityOnConversion"
                        />
                      </div>
                      <div
                        class="order-3 flex min-w-0 min-h-10 items-center justify-between gap-2 rounded-md border border-border/60 px-2 py-2 text-xs"
                        data-testid="cf-heartbeat"
                      >
                        <div class="flex min-w-0 items-center gap-1.5">
                          <span class="font-medium">CF保持心跳</span>
                          <HelpHint
                            portal
                            label="CF保持心跳"
                            text="同步生图和标准文本非流式失败时保持外层 HTTP 状态为 200，并在响应体中返回错误。"
                          />
                        </div>
                        <Switch
                          :model-value="cfHeartbeat"
                          :disabled="saving"
                          aria-label="CF保持心跳"
                          @update:model-value="updateExecutionPolicy('enable_cf_heartbeat', $event)"
                        />
                      </div>
                      <div
                        class="order-2 flex min-w-0 min-h-10 items-center justify-between gap-2 rounded-md border border-border/60 px-2 py-2 text-xs"
                        data-testid="cyber-continue-failover"
                      >
                        <div class="flex min-w-0 items-center gap-1.5">
                          <span class="font-medium">Cyber继续转移</span>
                          <HelpHint
                            portal
                            label="Cyber继续转移"
                            text="响应开始前遇到 Cyber Policy 错误时继续故障转移。"
                          />
                        </div>
                        <Switch
                          :model-value="cyberContinueFailover"
                          :disabled="saving"
                          aria-label="Cyber继续转移"
                          @update:model-value="updateExecutionPolicy('cyber_continue_failover', $event)"
                        />
                      </div>
                      <div
                        class="order-5 flex min-w-0 min-h-10 items-center justify-between gap-2 rounded-md border border-border/60 px-2 py-2 text-xs"
                        data-testid="cancel-on-client-disconnect"
                      >
                        <div class="flex min-w-0 items-center gap-1.5">
                          <span class="font-medium">取消请求立即打断</span>
                          <HelpHint
                            portal
                            label="取消请求立即打断"
                            text="默认关闭：客户端取消或断开后，服务端继续等待请求完成并正常计费。开启后立即打断且不计费，按次计费的请求仍收取单次请求费用。仅作用于当前调度策略。"
                          />
                        </div>
                        <Switch
                          :model-value="cancelOnClientDisconnect"
                          :disabled="saving"
                          aria-label="取消请求立即打断"
                          @update:model-value="updateExecutionPolicy('cancel_on_client_disconnect', $event)"
                        />
                      </div>
                    </div>
                  </section>
                </div>
              </section>
              <section class="min-w-0 border-t border-border/50">
                <button
                  type="button"
                  class="flex w-full items-center justify-between gap-3 px-3 py-3 text-left"
                  :aria-expanded="failoverOpen"
                  aria-label="故障转移"
                  @click="failoverOpen = !failoverOpen"
                >
                  <span class="text-sm font-medium">故障转移</span>
                  <ChevronRight
                    class="h-4 w-4 shrink-0 text-muted-foreground transition-transform"
                    :class="{ 'rotate-90': failoverOpen }"
                  />
                </button>
                <div
                  v-show="failoverOpen"
                  class="min-w-0 px-3 pb-3"
                >
                  <RoutingFailoverPolicyEditor
                    :key="draftGeneration"
                    ref="routingFailoverPolicyEditor"
                    :model-value="draft.config_json.default_policy"
                    :disabled="saving"
                    sidebar
                    @update:model-value="updateRoutingFailoverPolicy"
                    @pending-change="routingFailoverPending = $event"
                  >
                    <template #limits-extra>
                      <label
                        class="min-w-0 space-y-1.5 text-xs"
                        data-testid="sticky-key-attempts"
                      >
                        <span>错误重试次数</span>
                        <Input
                          :model-value="stickyKeyAttempts"
                          type="number"
                          min="0"
                          max="99"
                          :disabled="saving"
                          aria-label="错误重试次数"
                          @update:model-value="updateStickyKeyAttempts"
                        />
                      </label>
                    </template>
                  </RoutingFailoverPolicyEditor>
                </div>
              </section>
            </div>
            <div
              v-if="saveConflict"
              role="alert"
              class="m-3 flex flex-wrap items-center justify-between gap-3 rounded-lg border border-amber-500/30 bg-amber-500/5 p-3 text-sm"
            >
              <span>分组已被其他操作更新，当前修改已保留。重新加载后可基于最新配置继续编辑。</span>
              <Button
                variant="outline"
                size="sm"
                :disabled="busy"
                aria-label="重新加载分组"
                @click="reloadCurrentGroup"
              >
                重新加载分组
              </Button>
            </div>
            <div
              v-if="!draft"
              class="p-8 text-center"
            >
              <p class="text-sm text-muted-foreground">
                {{ emptyMessage }}
              </p>
              <Button
                v-if="!loading"
                variant="outline"
                class="mt-4"
                @click="loadingError ? refreshGroups() : groups.length ? openGroup(defaultGroupId) : openCreate()"
              >
                {{ loadingError ? '重新加载' : groups.length ? '打开默认分组' : '新建策略分组' }}
              </Button>
            </div>
          </div>
        </Card>
      </aside>
      <div
        class="min-w-0"
        aria-label="提供商目录"
      >
        <slot />
      </div>
    </div>
    <AlertDialog
      v-model="deleteDialogOpen"
      type="destructive"
      title="删除调度策略"
      :description="`确认删除调度策略「${draft?.name ?? ''}」？此操作无法撤销。`"
      confirm-text="删除"
      :loading="deleting"
      @confirm="confirmDeleteDraft"
    />
  </section>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { onBeforeRouteLeave, onBeforeRouteUpdate, useRoute, useRouter, type RouteLocationNormalized } from 'vue-router'
import { ChevronRight, Plus, Save, Star, Trash2 } from 'lucide-vue-next'
import { Button, Card, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Switch } from '@/components/ui'
import { AlertDialog } from '@/components/common'
import HelpHint from '@/components/common/HelpHint.vue'
import {
  DEFAULT_STICKY_KEY_ATTEMPTS,
  createEmptyRoutingGroupConfig,
  normalizeStickyKeyAttempts,
  parseBillingMultiplier,
  type RoutingModelPolicy,
  type RoutingPriorityMode,
  type RoutingSchedulingMode,
  type RoutingGroupConfig,
} from '@/features/routing/utils/routingPolicy'
import { RoutingFailoverPolicyEditor, RoutingSchedulingPolicyEditor } from '@/features/routing/components'
import { normalizeProviderSchedulingConfig } from '@/features/routing/utils/schedulingPolicies'
import { normalizeRoutingFailoverPolicy, validateRoutingFailoverPolicy, type RoutingFailoverPolicy } from '@/features/routing/utils/routingFailover'
import { createRoutingGroup, deleteRoutingGroup, listRoutingGroups, updateRoutingGroup, type RoutingGroupRecord } from '@/api/routing-profiles'
import { getGlobalModels, type GlobalModelResponse } from '@/api/global-models'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { parseApiError } from '@/utils/errorParser'
import { log } from '@/utils/logger'

withDefaults(defineProps<{ providerRevision?: number }>(), { providerRevision: 0 })

const emit = defineEmits<{
  'inspect-provider': [providerId: string]
  saved: []
  'context-change': [context: { groupId: string | null; groupName: string; config: RoutingGroupConfig | null; busy: boolean; activePolicy: SchedulingSelection | null; providerModelIds: string[] | undefined; priorityMode: RoutingPriorityMode; schedulingMode: RoutingSchedulingMode }]
}>()

interface SchedulingSelection {
  id?: string | null
  policy: RoutingModelPolicy | null
  priorityMode: RoutingPriorityMode
  schedulingMode: RoutingSchedulingMode
  scope: 'all' | 'selected'
  modelNames: string[]
}

interface RoutingGroupDraft {
  id?: string
  version: number
  name: string
  enabled: boolean
  is_system_default: boolean
  config_json: RoutingGroupConfig
}

const { success, error: showError } = useToast()
const { confirm } = useConfirm()
const route = useRoute()
const router = useRouter()
const groups = ref<RoutingGroupRecord[]>([])
const draft = ref<RoutingGroupDraft | null>(null)
const activePolicy = ref<SchedulingSelection | null>(null)
const initialSchedulingSelection = ref<Pick<SchedulingSelection, 'id' | 'scope' | 'modelNames'> | null>(null)
const routingSchedulingPolicyEditor = ref<{ updateSelectedPolicy: (policy: RoutingModelPolicy) => void } | null>(null)
const providerModelIds = computed(() => activePolicy.value?.scope === 'selected'
  ? globalModels.value.filter(model => activePolicy.value?.modelNames.includes(model.name)).map(model => model.id)
  : undefined)
const routingFailoverPolicyEditor = ref<{ commitJsonDrafts: () => boolean } | null>(null)
const routingFailoverPending = ref(false)
const routingSchedulingValid = ref(true)
const savedDraftSnapshot = ref<string | null>(null)
const globalModels = ref<GlobalModelResponse[]>([])
const loadingGlobalModels = ref(false)
const globalModelsError = ref<string | null>(null)
const loading = ref(true)
const loadingError = ref<string | null>(null)
const saving = ref(false)
const saveConflict = ref(false)
const deleting = ref(false)
const busy = computed(() => loading.value || saving.value || deleting.value)
const billingMultiplierInput = ref('1')
const draftGeneration = ref(0)
const groupMetadata = ref<HTMLElement | null>(null)
const advancedOpen = ref(false)
const failoverOpen = ref(false)
const deleteDialogOpen = ref(false)
let internalNavigation = false
let discardConfirmation: Promise<boolean> | null = null

const routeGroupId = computed(() => queryToString(route.query.group))
const defaultGroupId = computed(() => groups.value.find(group => group.is_system_default)?.id ?? groups.value[0]?.id ?? null)
const isNewDraft = computed(() => draft.value != null && !draft.value.id)
const selectedValue = computed(() => isNewDraft.value ? 'new' : draft.value?.id ?? '')
const priorityMode = 'provider' as const
const schedulingMode = computed(() => activePolicy.value?.schedulingMode ?? draft.value?.config_json.default_policy.scheduling_mode ?? 'cache_affinity')
const emptyMessage = computed(() => loading.value ? '正在加载调度策略' : loadingError.value ?? (groups.value.length ? '未找到调度策略' : '还没有调度策略'))
const keepPriorityOnConversion = computed(() => draft.value?.config_json.default_policy.keep_priority_on_conversion ?? false)
const stickyKeyAttempts = computed(() => draft.value?.config_json.default_policy.sticky_key_attempts ?? DEFAULT_STICKY_KEY_ATTEMPTS)
const cfHeartbeat = computed(() => draft.value?.config_json.default_policy.enable_cf_heartbeat ?? false)
const cyberContinueFailover = computed(() => draft.value?.config_json.default_policy.cyber_continue_failover ?? false)
const cancelOnClientDisconnect = computed(() => draft.value?.config_json.default_policy.cancel_on_client_disconnect ?? false)
const billingMultiplierError = computed(() => parseBillingMultiplier(billingMultiplierInput.value) == null ? '分组倍率必须是大于或等于 0 的有效数字' : null)
const draftDirty = computed(() => draft.value != null && (Boolean(billingMultiplierError.value) || routingFailoverPending.value || savedDraftSnapshot.value !== draftSnapshotValue(draft.value)))
const canSaveDraft = computed(() => Boolean(draft.value) && !busy.value && draftDirty.value && routingSchedulingValid.value && !billingMultiplierError.value)

function queryToString(value: unknown): string | null {
  if (Array.isArray(value)) return typeof value[0] === 'string' ? value[0] : null
  return typeof value === 'string' ? value : null
}

function normalizeRecord(group: RoutingGroupRecord): RoutingGroupRecord {
  return { ...group, sort_order: Number.isFinite(group.sort_order) ? group.sort_order : 0, config_json: normalizeProviderSchedulingConfig(group.config_json) }
}

function sortGroups(items: RoutingGroupRecord[]): RoutingGroupRecord[] {
  return [...items].sort((left, right) => {
    if (left.enabled !== right.enabled) return left.enabled ? -1 : 1
    return left.sort_order - right.sort_order || left.name.localeCompare(right.name) || left.id.localeCompare(right.id)
  })
}

function cloneConfig(config: RoutingGroupConfig): RoutingGroupConfig {
  return normalizeProviderSchedulingConfig(JSON.parse(JSON.stringify(config)) as Partial<RoutingGroupConfig>)
}

function draftSnapshotValue(value: RoutingGroupDraft): string {
  return JSON.stringify({ name: value.name.trim(), enabled: value.enabled, is_system_default: value.is_system_default, config_json: cloneConfig(value.config_json) })
}

function resetEditors(): void {
  activePolicy.value = null
  initialSchedulingSelection.value = null
  saveConflict.value = false
  draftGeneration.value += 1
  routingFailoverPending.value = false
  routingSchedulingValid.value = true
  deleteDialogOpen.value = false
}

function selectGroup(group: RoutingGroupRecord, preserveSelection = false): void {
  const selection = preserveSelection ? activePolicy.value : null
  resetEditors()
  initialSchedulingSelection.value = selection
    ? { id: selection.id, scope: selection.scope, modelNames: [...selection.modelNames] }
    : null
  draft.value = { id: group.id, version: group.version, name: group.name, enabled: group.enabled, is_system_default: group.is_system_default, config_json: cloneConfig(group.config_json) }
  billingMultiplierInput.value = String(draft.value.config_json.billing_multiplier)
  savedDraftSnapshot.value = draftSnapshotValue(draft.value)
}

function openCreate(): void {
  if (busy.value || isNewDraft.value) return
  openGroup('new')
}

function startNewDraft(): void {
  resetEditors()
  draft.value = { version: 0, name: '', enabled: true, is_system_default: groups.value.length === 0, config_json: createEmptyRoutingGroupConfig() }
  billingMultiplierInput.value = '1'
  savedDraftSnapshot.value = null
  void nextTick(() => {
    groupMetadata.value?.scrollIntoView?.({ block: 'nearest' })
    groupMetadata.value?.querySelector<HTMLInputElement>('[aria-label="策略名称"]')?.focus()
  })
}

function clearDraft(): void {
  resetEditors()
  draft.value = null
  savedDraftSnapshot.value = null
}

function syncRouteState(): void {
  if (loading.value) return
  if (routeGroupId.value === 'new') {
    if (!isNewDraft.value) startNewDraft()
    return
  }
  const group = groups.value.find(item => item.id === (routeGroupId.value ?? defaultGroupId.value))
  if (!group) { clearDraft(); return }
  if (draft.value?.id !== group.id) selectGroup(group)
}
function openGroup(groupId: string | null): void {
  if (busy.value) return
  void router.push({ name: 'ProviderManagement', query: { ...route.query, view: undefined, group: groupId ?? undefined } })
}

function changeGroup(value: string): void {
  // The route guard owns the switch; keep the controlled value until navigation succeeds.
  openGroup(value)
}

async function confirmDiscard(): Promise<boolean> {
  if (!draftDirty.value) return true
  if (!discardConfirmation) {
    discardConfirmation = confirm({ title: '有未保存的调度修改', message: '离开后将丢弃当前策略的未保存修改。是否继续？', confirmText: '放弃修改并离开', cancelText: '继续编辑', variant: 'warning' })
      .finally(() => { discardConfirmation = null })
  }
  return discardConfirmation
}

async function guardNavigation(to: RouteLocationNormalized): Promise<boolean> {
  if (internalNavigation) return true
  const targetGroup = queryToString(to.query.group) ?? defaultGroupId.value
  const staysOnDraft = to.name === 'ProviderManagement' && targetGroup === selectedValue.value
  if (staysOnDraft) return true
  if (busy.value) {
    showError('正在保存调度设置，请稍候再切换')
    return false
  }
  return confirmDiscard()
}

onBeforeRouteUpdate(guardNavigation)
onBeforeRouteLeave(guardNavigation)

function preventUnload(event: BeforeUnloadEvent): void {
  if (!draftDirty.value && !busy.value) return
  event.preventDefault()
  event.returnValue = ''
}

function updateDraftConfig(value: RoutingGroupConfig): void {
  if (draft.value) draft.value.config_json = normalizeProviderSchedulingConfig(value)
}

function updateBillingMultiplier(value: string | number): void {
  if (!draft.value || busy.value) return
  billingMultiplierInput.value = String(value)
  const parsed = parseBillingMultiplier(value)
  if (parsed != null) draft.value.config_json.billing_multiplier = parsed
}

function updatePriorityPolicy(policy: RoutingModelPolicy): void {
  if (busy.value) return
  routingSchedulingPolicyEditor.value?.updateSelectedPolicy(policy)
}

function updateStickyKeyAttempts(value: string | number): void {
  if (!draft.value) return
  updateDraftConfig({ ...draft.value.config_json, default_policy: { ...draft.value.config_json.default_policy, sticky_key_attempts: normalizeStickyKeyAttempts(value) } })
}

function updateKeepPriorityOnConversion(value: boolean): void {
  if (!draft.value) return
  updateDraftConfig({ ...draft.value.config_json, default_policy: { ...draft.value.config_json.default_policy, keep_priority_on_conversion: value } })
}

function updateExecutionPolicy(field: 'enable_cf_heartbeat' | 'cyber_continue_failover' | 'cancel_on_client_disconnect', value: boolean): void {
  if (!draft.value) return
  updateDraftConfig({ ...draft.value.config_json, default_policy: { ...draft.value.config_json.default_policy, [field]: value } })
}

function updateRoutingFailoverPolicy(value: RoutingFailoverPolicy): void {
  if (draft.value) Object.assign(draft.value.config_json.default_policy, normalizeRoutingFailoverPolicy(value))
}

function replaceGroup(group: RoutingGroupRecord, select: boolean, preserveSelection = false): void {
  const normalized = normalizeRecord(group)
  const otherGroups = groups.value.filter(item => item.id !== normalized.id).map(item => normalized.is_system_default ? { ...item, is_system_default: false } : item)
  groups.value = sortGroups([...otherGroups, normalized])
  if (select) selectGroup(normalized, preserveSelection)
}

async function refreshGroups(): Promise<void> {
  const preservedDraft = draft.value
  loading.value = true
  loadingError.value = null
  try {
    const response = await listRoutingGroups()
    groups.value = sortGroups(response.items.map(normalizeRecord))
  } catch (err) {
    loadingError.value = parseApiError(err, '加载调度策略失败')
    showError(loadingError.value)
    log.error('加载调度策略失败:', err)
  } finally {
    loading.value = false
    if (!preservedDraft) syncRouteState()
  }
}

async function reloadCurrentGroup(): Promise<void> {
  if (busy.value || !draft.value?.id) return
  if (draftDirty.value && !await confirm({ title: '重新加载最新分组', message: '重新加载会放弃当前未保存的修改。是否继续？', confirmText: '放弃修改并重新加载', cancelText: '保留修改', variant: 'warning' })) return
  const groupId = draft.value.id
  await refreshGroups()
  if (loadingError.value) return
  const group = groups.value.find(item => item.id === groupId)
  if (group) selectGroup(group)
  else { clearDraft(); syncRouteState() }
}

async function loadGlobalModels(options: { cacheTtlMs?: number } = {}): Promise<void> {
  loadingGlobalModels.value = true
  globalModelsError.value = null
  try {
    const response = await getGlobalModels({ limit: 1000, is_active: true }, { cacheTtlMs: options.cacheTtlMs ?? 0 })
    globalModels.value = response.models ?? []
  } catch (err) {
    globalModels.value = []
    globalModelsError.value = parseApiError(err, '加载全局模型失败')
    log.error('加载全局模型失败:', err)
  } finally {
    loadingGlobalModels.value = false
  }
}

async function saveDraft(): Promise<boolean> {
  if (!draft.value || busy.value) return false
  const name = draft.value.name.trim()
  if (!name) {
    groupMetadata.value?.scrollIntoView?.({ block: 'nearest' })
    groupMetadata.value?.querySelector<HTMLInputElement>('[aria-label="策略名称"]')?.focus()
    showError('策略名称不能为空')
    return false
  }
  if (billingMultiplierError.value) {
    groupMetadata.value?.scrollIntoView?.({ block: 'nearest' })
    groupMetadata.value?.querySelector<HTMLInputElement>('[aria-label="分组倍率"]')?.focus()
    showError(billingMultiplierError.value)
    return false
  }
  if (routingFailoverPolicyEditor.value && !routingFailoverPolicyEditor.value.commitJsonDrafts()) { failoverOpen.value = true; return false }
  const failoverError = validateRoutingFailoverPolicy(draft.value.config_json.default_policy)
  if (failoverError) { failoverOpen.value = true; showError(failoverError); return false }
  if (!routingSchedulingValid.value) { showError('请为每条调度配置选择适用模型'); return false }
  const targetGroupId = draft.value.id
  const submittedGeneration = draftGeneration.value
  const submittedSnapshot = draftSnapshotValue(draft.value)
  const payload = { name, enabled: draft.value.enabled, is_system_default: draft.value.is_system_default, config_json: cloneConfig(draft.value.config_json) }
  const expectedVersion = draft.value.version
  saving.value = true
  try {
    const saved = targetGroupId
      ? await updateRoutingGroup(targetGroupId, { ...payload, expected_version: expectedVersion })
      : await createRoutingGroup({ ...payload, sort_order: groups.value.length })
    const unchanged = draftGeneration.value === submittedGeneration && draft.value?.id === targetGroupId && draftSnapshotValue(draft.value) === submittedSnapshot
    replaceGroup(saved, unchanged, true)
    if (!targetGroupId) {
      // Retain any newer edits while attaching the server ID, so retries update the created group.
      if (!unchanged && draft.value && draftGeneration.value === submittedGeneration && !draft.value.id) {
        draft.value.id = saved.id
        draft.value.version = saved.version
        savedDraftSnapshot.value = draftSnapshotValue({ ...saved, config_json: cloneConfig(saved.config_json) })
      }
      internalNavigation = true
      try { await router.replace({ name: 'ProviderManagement', query: { ...route.query, view: undefined, group: saved.id } }) }
      finally { internalNavigation = false }
    }
    success(targetGroupId ? '调度策略已保存' : '策略分组已创建')
    emit('saved')
    return unchanged
  } catch (err) {
    const status = (err as { response?: { status?: number } })?.response?.status
    saveConflict.value = Boolean(targetGroupId) && status === 409
    showError(saveConflict.value ? '此分组已在其他操作中更新。当前修改已保留，请重新加载最新分组后再编辑。' : parseApiError(err, targetGroupId ? '保存调度策略失败' : '创建策略分组失败'))
    log.error('保存调度策略失败:', err)
    return false
  } finally { saving.value = false }
}
async function ensureSaved(): Promise<boolean> {
  if (busy.value || !draft.value) return false
  if (!draftDirty.value) return true
  const approved = await confirm({ title: '先保存当前分组', message: '当前分组有未保存的修改。保存后继续？', confirmText: '保存并继续', cancelText: '继续编辑', variant: 'question' })
  return approved && await saveDraft()
}

async function confirmDeleteDraft(): Promise<void> {
  if (!draft.value?.id || busy.value) return
  const targetId = draft.value.id
  deleting.value = true
  try {
    await deleteRoutingGroup(targetId)
    groups.value = groups.value.filter(group => group.id !== targetId)
    clearDraft()
    internalNavigation = true
    try {
      await router.replace({ name: 'ProviderManagement', query: { ...route.query, view: undefined, group: defaultGroupId.value ?? undefined } })
      syncRouteState()
    } finally {
      internalNavigation = false
    }
    success('调度策略已删除')
    emit('saved')
  } catch (err) {
    showError(parseApiError(err, '删除调度策略失败'))
    log.error('删除调度策略失败:', err)
  } finally {
    deleting.value = false
  }
}

onMounted(() => {
  void refreshGroups()
  void loadGlobalModels({ cacheTtlMs: 60_000 })
  window.addEventListener('beforeunload', preventUnload)
})
onBeforeUnmount(() => window.removeEventListener('beforeunload', preventUnload))
watch(() => [route.name, route.query.group], syncRouteState)
watch(() => [draft.value, busy.value, activePolicy.value, providerModelIds.value], () => emit('context-change', { groupId: draft.value?.id ?? null, groupName: draft.value?.name ?? '', config: draft.value?.config_json ?? null, busy: busy.value, activePolicy: activePolicy.value, providerModelIds: providerModelIds.value, priorityMode, schedulingMode: schedulingMode.value }), { deep: true, immediate: true })
defineExpose({ updateDraftConfig, updatePriorityPolicy, refreshGroups, ensureSaved })
</script>
