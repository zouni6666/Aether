<template>
  <section
    :class="[
      layout === 'config-only' ? 'config-only-editor space-y-2' : 'space-y-4',
      { 'config-only-editor--sidebar': layout === 'config-only' && sidebar },
    ]"
  >
    <div
      v-if="layout !== 'config-only'"
      class="flex flex-wrap items-start justify-between gap-3"
    >
      <div>
        <h3 class="text-sm font-medium">
          调度配置
        </h3>
      </div>
      <Button
        v-if="scopeMode === 'selected'"
        type="button"
        variant="outline"
        size="sm"
        class="shrink-0 gap-1.5"
        :disabled="!canAddEntry"
        :title="addEntryHint"
        aria-label="添加调度配置"
        @click="addEntry"
      >
        <Plus class="h-3.5 w-3.5" />
        添加配置
      </Button>
    </div>

    <div
      v-if="layout === 'config-only'"
      class="scheduling-toolbar"
      :class="{
        'scheduling-toolbar--composed': !sidebar && ($slots['toolbar-leading'] || $slots['toolbar-actions']),
        'scheduling-toolbar--sidebar': sidebar,
      }"
    >
      <div
        v-if="$slots['toolbar-leading']"
        class="scheduling-toolbar__leading min-w-0"
      >
        <slot name="toolbar-leading" />
      </div>
      <div class="scheduling-toolbar__scope min-w-0 space-y-1.5">
        <span class="flex h-6 items-center text-xs font-medium text-muted-foreground">模型配置</span>
        <div
          role="group"
          aria-label="调度范围"
          class="scheduling-switch w-full grid-cols-2"
        >
          <button
            v-for="mode in scopeModes"
            :key="mode.value"
            type="button"
            class="scheduling-switch__option"
            :aria-label="mode.label"
            :aria-pressed="scopeMode === mode.value"
            :disabled="disabled"
            @click="setScopeMode(mode.value)"
          >
            {{ mode.value === 'all' ? '全局配置' : mode.label }}
          </button>
        </div>
      </div>
      <div
        v-if="!sidebar && $slots['toolbar-actions']"
        class="scheduling-toolbar__actions min-w-0"
      >
        <slot name="toolbar-actions" />
      </div>
    </div>
    <div
      v-else
      role="group"
      aria-label="调度范围"
      class="scheduling-switch w-full grid-cols-2"
    >
      <button
        v-for="mode in scopeModes"
        :key="mode.value"
        type="button"
        class="scheduling-switch__option"
        :aria-label="mode.label"
        :aria-pressed="scopeMode === mode.value"
        :disabled="disabled"
        @click="setScopeMode(mode.value)"
      >
        {{ mode.label }}
      </button>
    </div>

    <div
      v-if="layout === 'config-only' && scopeMode === 'all' && selectedEntry"
      class="min-w-0 space-y-1.5 border-t border-border/50 pt-2"
    >
      <div class="flex h-6 items-center gap-1 text-xs font-medium text-muted-foreground">
        <span>调度策略</span>
        <HelpHint
          label="调度策略"
          :text="schedulingDescription"
          :portal="sidebar"
        />
      </div>
      <div
        role="group"
        aria-label="调度策略"
        class="scheduling-switch w-full grid-cols-3"
      >
        <button
          v-for="mode in schedulingModes"
          :key="mode.value"
          type="button"
          class="scheduling-switch__option"
          :aria-pressed="selectedEntry.schedulingMode === mode.value"
          :disabled="disabled"
          @click="updateEntry(selectedEntry.id, { schedulingMode: mode.value })"
        >
          {{ mode.label }}
        </button>
      </div>
    </div>

    <fieldset
      v-if="layout === 'config-only' && scopeMode === 'selected'"
      :disabled="disabled"
      :inert="disabled"
      class="min-w-0 space-y-2 border-t border-border/50 pt-2"
    >
      <div
        role="group"
        aria-label="模型调度配置"
        class="divide-y divide-border/60 border-b border-border/60"
      >
        <section
          v-for="(entry, index) in entries"
          :key="entry.id"
          class="min-w-0"
          :aria-label="`调度配置 ${index + 1}`"
        >
          <div class="flex min-w-0 items-center">
            <button
              type="button"
              class="flex min-h-10 min-w-0 flex-1 items-center gap-2 rounded-md px-1 text-left text-xs hover:bg-muted/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring disabled:opacity-50"
              :class="selectedEntryId === entry.id ? 'text-primary' : 'text-foreground'"
              :aria-label="`选择调度配置 ${index + 1}`"
              :aria-pressed="selectedEntryId === entry.id"
              :aria-expanded="expandedId === entry.id"
              :disabled="disabled"
              @click="toggleEntry(entry.id)"
            >
              <ChevronRight
                class="h-3.5 w-3.5 shrink-0 transition-transform"
                :class="expandedId === entry.id ? 'rotate-90' : ''"
              />
              <span
                class="min-w-0 flex-1 truncate font-medium"
                :title="entry.scope === 'selected' ? entry.models.join('、') : '默认配置'"
              >{{ scopeSummary(entry) }}</span>
              <span class="shrink-0 text-muted-foreground">{{ schedulingModeLabel(entry.schedulingMode) }}</span>
            </button>
            <RoutingModelSelectionPopover
              v-if="entry.scope === 'selected'"
              :open="editingModelsId === entry.id"
              :model-value="entry.models"
              :models="globalModels"
              :assigned-models="otherModelOwners(entry.id)"
              :loading="loadingModels"
              :error="modelsError"
              :disabled="disabled"
              @update:open="open => setModelEditorOpen(entry.id, open)"
              @update:model-value="models => updateEntry(entry.id, { models })"
              @reload="emit('reload-models')"
            />
            <Button
              v-if="entries.length > 1"
              type="button"
              variant="ghost"
              size="icon"
              class="mr-0.5 h-7 w-7 shrink-0 text-muted-foreground hover:text-destructive"
              :aria-label="`删除调度配置 ${index + 1}`"
              :disabled="disabled"
              @click="removeEntry(entry.id)"
            >
              <Trash2 class="h-3.5 w-3.5" />
            </Button>
          </div>
          <div
            v-if="selectedEntryId === entry.id && expandedId === entry.id"
            role="region"
            aria-label="当前配置的适用模型"
            class="min-w-0 px-1 pb-2"
          >
            <span
              v-if="entry.scope === 'selected'"
              class="mb-1.5 block text-xs font-medium text-muted-foreground"
            >适用模型</span>
            <div
              v-if="entry.models.length"
              class="flex min-w-0 flex-wrap gap-1.5 py-1"
              aria-label="已配置模型"
            >
              <span
                v-for="name in entry.models"
                :key="name"
                class="max-w-full break-words rounded-md bg-muted/70 px-2 py-1 text-xs text-foreground [overflow-wrap:anywhere]"
                :title="name"
              >{{ modelDisplayName(name) }}</span>
            </div>
            <p
              v-else-if="entry.scope === 'selected'"
              class="py-2 text-xs leading-5 text-muted-foreground"
            >
              请选择适用模型
            </p>
            <p
              v-else
              class="py-2 text-xs leading-5 text-muted-foreground"
            >
              此默认配置适用于未单独指定的模型，新增模型也会自动使用。
            </p>
          </div>
          <div
            v-if="selectedEntryId === entry.id && expandedId === entry.id"
            class="min-w-0 space-y-1.5 px-1 pb-3 pt-1"
          >
            <div class="flex h-6 items-center gap-1 text-xs font-medium text-muted-foreground">
              <span>调度策略</span>
              <HelpHint
                label="调度策略"
                :text="schedulingDescription"
                :portal="sidebar"
              />
            </div>
            <div
              role="group"
              aria-label="调度策略"
              class="scheduling-switch w-full grid-cols-3"
            >
              <button
                v-for="mode in schedulingModes"
                :key="mode.value"
                type="button"
                class="scheduling-switch__option"
                :aria-pressed="entry.schedulingMode === mode.value"
                :disabled="disabled"
                @click="updateEntry(entry.id, { schedulingMode: mode.value })"
              >
                {{ mode.label }}
              </button>
            </div>
          </div>
        </section>
      </div>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        class="h-8 w-full gap-1 px-2 text-xs"
        :disabled="!canAddEntry"
        :title="addEntryHint"
        aria-label="添加调度配置"
        @click="addEntry"
      >
        <Plus class="h-3.5 w-3.5" />
        添加配置
      </Button>
    </fieldset>
    <fieldset
      v-if="layout !== 'config-only'"
      :disabled="disabled"
      :inert="disabled"
      class="min-w-0 space-y-3"
    >
      <section
        v-for="(entry, index) in entries"
        :key="entry.id"
        :class="scopeMode === 'selected' ? 'rounded-lg border border-border/60' : ''"
        :aria-label="`调度配置 ${index + 1}`"
      >
        <div
          v-if="scopeMode === 'selected'"
          class="flex items-center gap-3 px-4 py-3"
        >
          <button
            type="button"
            class="flex min-w-0 flex-1 items-center gap-3 text-left"
            :aria-label="`${expandedId === entry.id ? '收起' : '展开'}调度配置 ${index + 1}`"
            :aria-expanded="expandedId === entry.id"
            @click="toggleEntry(entry.id)"
          >
            <ChevronRight
              class="h-4 w-4 shrink-0 text-muted-foreground transition-transform"
              :class="expandedId === entry.id ? 'rotate-90' : ''"
            />
            <span class="min-w-0">
              <span
                class="block truncate text-sm font-medium"
                :title="entry.scope === 'selected' ? entry.models.join('、') : '默认配置'"
              >
                {{ scopeSummary(entry) }}
              </span>
              <span class="mt-0.5 block text-xs text-muted-foreground">
                配置 {{ index + 1 }} · {{ schedulingModeLabel(entry.schedulingMode) }}
              </span>
            </span>
          </button>
          <Button
            v-if="entries.length > 1"
            type="button"
            variant="ghost"
            size="icon"
            class="h-8 w-8 shrink-0 text-muted-foreground hover:text-destructive"
            :aria-label="`删除调度配置 ${index + 1}`"
            @click="removeEntry(entry.id)"
          >
            <Trash2 class="h-4 w-4" />
          </Button>
        </div>

        <div
          v-if="scopeMode === 'all' || expandedId === entry.id"
          class="space-y-5"
          :class="scopeMode === 'selected' ? 'border-t border-border/60 p-4' : ''"
        >
          <div
            v-if="entry.scope === 'selected'"
            class="min-w-0"
          >
            <div class="min-w-0 space-y-2">
              <h4 class="text-sm font-medium">
                适用模型
              </h4>
              <RoutingModelSelector
                :model-value="entry.models"
                :models="globalModels"
                :assigned-models="otherModelOwners(entry.id)"
                :loading="loadingModels"
                :error="modelsError"
                :disabled="disabled"
                @update:model-value="models => updateEntry(entry.id, { models })"
                @reload="emit('reload-models')"
              />
            </div>
          </div>

          <div
            v-if="entry.scope === 'all' || entry.models.length > 0"
            class="space-y-4"
            :class="entry.scope === 'selected' ? 'border-t border-border/60 pt-4' : ''"
          >
            <h4 class="text-sm font-medium">
              调度设置
            </h4>
            <div class="rounded-md bg-primary/5 px-3 py-2 text-xs leading-relaxed text-muted-foreground">
              <template v-if="entry.scope === 'all'">
                {{ scopeMode === 'all' ? '此配置适用于全部模型，新增模型也会自动使用。' : '此默认配置适用于未单独指定的模型。' }}
              </template>
              <template v-else>
                正在编辑 {{ entry.models.length }} 个模型共用的配置：
                <span class="text-foreground">{{ entry.models.join('、') }}</span>。
                此处修改调度方式或排序，会同时应用于这些模型。
              </template>
            </div>
            <div class="max-w-xl">
              <div class="space-y-1.5 text-sm">
                <span class="text-muted-foreground">调度策略</span>
                <div
                  role="group"
                  aria-label="调度策略"
                  class="scheduling-switch grid-cols-3"
                >
                  <button
                    v-for="mode in schedulingModes"
                    :key="mode.value"
                    type="button"
                    class="scheduling-switch__option"
                    :aria-pressed="entry.schedulingMode === mode.value"
                    :disabled="disabled"
                    @click="updateEntry(entry.id, { schedulingMode: mode.value })"
                  >
                    {{ mode.label }}
                  </button>
                </div>
              </div>
            </div>
            <RoutingPriorityPolicyEditor
              :config="schedulingPolicyEditorConfig(config, entry)"
              :provider-model-ids="entry.scope === 'selected' ? providerModelIds(entry) : undefined"
              :show-scheduling-mode="false"
              :refresh-revision="refreshRevision"
              :subtitle="entry.scope === 'all' ? '此配置范围内的模型共用排序，仅对实际可用的候选生效' : `${entry.models.length} 个模型共用此排序，仅对各模型可用的候选生效`"
              @update:config="value => updateEntry(entry.id, { policy: getDefaultModelPolicy(value) })"
              @inspect-provider="providerId => emit('inspect-provider', providerId)"
            >
              <template #provider-status="{ provider }">
                <slot
                  name="provider-status"
                  :provider="provider"
                />
              </template>
            </RoutingPriorityPolicyEditor>
          </div>
          <p
            v-else
            class="border-t border-border/60 pt-4 text-xs text-muted-foreground"
          >
            选好模型后，即可设置调度方式和排序。
          </p>
        </div>
      </section>
    </fieldset>
    <p
      v-if="validationError && entries.every(entry => entry.scope === 'all' || entry.models.length > 0)"
      role="alert"
      class="text-xs text-destructive"
    >
      {{ validationError }}
    </p>
    <p
      v-if="layout !== 'config-only' && scopeMode === 'selected' && !hasAllModels"
      class="text-xs text-muted-foreground"
    >
      {{ availableModels.length ? `还有 ${availableModels.length} 个模型可配置；` : '' }}未指定的模型继续使用默认调度。
    </p>
    <div
      v-if="layout === 'config-only' && sidebar && $slots['toolbar-actions']"
      class="min-w-0 border-t border-border/50 pt-3"
    >
      <slot name="toolbar-actions" />
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ChevronRight, Plus, Trash2 } from 'lucide-vue-next'
import { Button } from '@/components/ui'
import HelpHint from '@/components/common/HelpHint.vue'
import type { GlobalModelResponse } from '@/api/global-models'
import RoutingPriorityPolicyEditor from './RoutingPriorityPolicyEditor.vue'
import RoutingModelSelector from './RoutingModelSelector.vue'
import RoutingModelSelectionPopover from './RoutingModelSelectionPopover.vue'
import { getDefaultModelPolicy, normalizeRoutingGroupConfig, type RoutingGroupConfig, type RoutingModelPolicy, type RoutingPriorityMode, type RoutingSchedulingMode } from '../utils/routingPolicy'
import {
  createSchedulingPolicy,
  readSchedulingPolicies,
  schedulingPolicyEditorConfig,
  validateSchedulingPolicies,
  writeSchedulingPolicies,
  type SchedulingPolicy,
} from '../utils/schedulingPolicies'

const props = defineProps<{
  config: RoutingGroupConfig
  refreshRevision?: number
  globalModels: GlobalModelResponse[]
  loadingModels?: boolean
  modelsError?: string | null
  disabled?: boolean
  layout?: 'embedded' | 'config-only'
  sidebar?: boolean
  initialSelection?: {
    id?: string | null
    scope: SchedulingPolicy['scope']
    modelNames: string[]
  }
}>()

const emit = defineEmits<{
  'update:config': [value: RoutingGroupConfig]
  'validity-change': [valid: boolean]
  'reload-models': []
  'inspect-provider': [providerId: string]
  'selection-change': [value: {
    id: string | null
    policy: RoutingModelPolicy | null
    priorityMode: RoutingPriorityMode
    schedulingMode: RoutingSchedulingMode
    scope: SchedulingPolicy['scope']
    modelNames: string[]
  }]
}>()

const scopeModes = [
  { value: 'all' as const, label: '全部模型' },
  { value: 'selected' as const, label: '区分模型' },
]
const schedulingModes: Array<{ value: RoutingSchedulingMode; label: string }> = [
  { value: 'cache_affinity', label: '缓存亲和' },
  { value: 'load_balance', label: '负载均衡' },
  { value: 'fixed_order', label: '固定顺序' },
]
const entries = ref(readSchedulingPolicies(props.config))
const initialSelection = props.initialSelection
const initialSelectedEntry = entries.value.find(entry => entry.id === initialSelection?.id)
  ?? entries.value.find(entry => initialSelection
    && entry.scope === initialSelection.scope
    && entry.models.length === initialSelection.modelNames.length
    && entry.models.every(model => initialSelection.modelNames.includes(model)))
  ?? entries.value[0]
const config = computed(() => normalizeRoutingGroupConfig(props.config))
const layout = computed(() => props.layout ?? 'embedded')
const selectedEntryId = ref<string | null>(initialSelectedEntry?.id ?? null)
const scopeMode = ref<SchedulingPolicy['scope']>(entries.value.some(entry => entry.scope === 'selected') ? 'selected' : 'all')
let allModelsDraft: SchedulingPolicy[] | null = null
let selectedModelsDraft: SchedulingPolicy[] | null = null
const fallbackScheduling = {
  priority_mode: 'provider' as const,
  scheduling_mode: props.config.default_policy.scheduling_mode,
}
const expandedId = ref<string | null>(initialSelectedEntry?.id ?? null)
const editingModelsId = ref<string | null>(null)
const validationError = computed(() => validateSchedulingPolicies(entries.value))
const hasAllModels = computed(() => entries.value.some(entry => entry.scope === 'all'))
const assignedModels = computed(() => new Set(entries.value.filter(entry => entry.scope === 'selected').flatMap(entry => entry.models)))
const availableModels = computed(() => props.globalModels.filter(model => !assignedModels.value.has(model.name)))
const canAddEntry = computed(() => scopeMode.value === 'selected' && !props.disabled && !props.loadingModels && !props.modelsError
  && !validationError.value && availableModels.value.length > 0)
const addEntryHint = computed(() => {
  if (validationError.value) return validationError.value
  if (props.loadingModels) return '正在加载全局模型'
  if (props.modelsError) return '请先重新加载全局模型'
  return availableModels.value.length ? '为其他模型添加一套调度配置' : '所有全局模型都已有配置'
})

const selectedEntry = computed(() => entries.value.find(entry => entry.id === selectedEntryId.value) ?? entries.value[0] ?? null)
const schedulingDescription = computed(() => {
  if (selectedEntry.value?.schedulingMode === 'fixed_order') return '固定顺序：优先级决定候选的尝试次序，实际可用候选还受模型能力和 API 格式影响。'
  if (selectedEntry.value?.schedulingMode === 'load_balance') return '负载均衡：请求会分散到可用候选，列表优先级不代表实际尝试顺序。'
  return '缓存亲和：会优先复用缓存命中的候选，实际顺序可能与列表不同。'
})

function emitSelection(): void {
  const entry = selectedEntry.value
  emit('selection-change', entry
    ? {
        id: entry.id,
        policy: entry.scope === 'selected' && !entry.models.length
          ? null
          : getDefaultModelPolicy({ ...config.value, model_policies: [entry.policy] }),
        priorityMode: 'provider',
        schedulingMode: entry.schedulingMode,
        scope: entry.scope,
        modelNames: [...entry.models],
      }
    : {
        id: null,
        policy: null,
        priorityMode: 'provider',
        schedulingMode: config.value.default_policy.scheduling_mode,
        scope: scopeMode.value,
        modelNames: [],
      })
}

function toggleEntry(id: string): void {
  if (props.disabled) return
  editingModelsId.value = null
  if (layout.value === 'config-only') {
    if (selectedEntryId.value === id) expandedId.value = expandedId.value === id ? null : id
    else selectEntry(id)
    return
  }
  if (expandedId.value === id) expandedId.value = null
  else {
    expandedId.value = id
  }
}

function selectEntry(id: string): void {
  if (props.disabled) return
  editingModelsId.value = null
  selectedEntryId.value = id
  expandedId.value = id
  emitSelection()
}

function setModelEditorOpen(id: string, open: boolean): void {
  if (!open) {
    if (editingModelsId.value === id) editingModelsId.value = null
    return
  }
  if (props.disabled) return
  selectEntry(id)
  editingModelsId.value = id
}

function updateSelectedPolicy(policy: RoutingModelPolicy): void {
  const entry = selectedEntry.value
  if (!entry || props.disabled || entry.scope === 'selected' && !entry.models.length) return
  updateEntry(entry.id, { policy: { ...policy, model: '*' } })
}

defineExpose({ updateSelectedPolicy, selectEntry })

watch(validationError, error => emit('validity-change', !error), { immediate: true })
watch([selectedEntry, entries, scopeMode], emitSelection, { immediate: true, deep: true })

function schedulingModeLabel(mode: RoutingSchedulingMode): string {
  return schedulingModes.find(item => item.value === mode)?.label ?? mode
}

function scopeSummary(entry: SchedulingPolicy): string {
  if (entry.scope === 'all') return '默认配置'
  if (entry.models.length === 0) return '请选择适用模型'
  if (layout.value === 'config-only') {
    const first = entry.models[0] ?? ''
    const label = props.globalModels.find(model => model.name === first)?.display_name || first
    return label + (entry.models.length > 1 ? ` +${entry.models.length - 1}` : '')
  }
  const labels = entry.models.slice(0, 2).map(name => props.globalModels.find(model => model.name === name)?.display_name || name)
  return labels.join('、') + (entry.models.length > 2 ? ` 等 ${entry.models.length} 个模型` : '')
}

function modelDisplayName(name: string): string {
  return props.globalModels.find(model => model.name === name)?.display_name || name
}

function otherModelOwners(entryId: string): Record<string, number> {
  return Object.fromEntries(entries.value.flatMap((entry, index) => entry.id !== entryId && entry.scope === 'selected'
    ? entry.models.map(model => [model, index + 1])
    : []))
}

function providerModelIds(entry: SchedulingPolicy): string[] {
  const names = new Set(entry.models)
  return props.globalModels.filter(model => names.has(model.name)).map(model => model.id)
}

function publish(): void {
  emit('update:config', writeSchedulingPolicies({
    ...props.config,
    default_policy: { ...props.config.default_policy, ...fallbackScheduling },
  }, entries.value))
}

function setScopeMode(scope: SchedulingPolicy['scope']): void {
  if (props.disabled || scopeMode.value === scope) return
  editingModelsId.value = null
  if (scopeMode.value === 'all') allModelsDraft = entries.value
  else selectedModelsDraft = entries.value

  const saved = scope === 'all' ? allModelsDraft : selectedModelsDraft
  if (saved) {
    entries.value = saved
  } else {
    const source = entries.value.find(entry => entry.scope === 'all') ?? entries.value[0]
      ?? createSchedulingPolicy(props.config, scope)
    entries.value = [{
      ...createSchedulingPolicy(props.config, scope),
      priorityMode: 'provider',
      schedulingMode: source.schedulingMode,
      policy: source.policy,
    }]
  }
  scopeMode.value = scope
  expandedId.value = entries.value[0]?.id ?? null
  selectedEntryId.value = entries.value[0]?.id ?? null
  publish()
}

function updateEntry(id: string, patch: Partial<SchedulingPolicy>): void {
  if (props.disabled) return
  const current = entries.value.find(entry => entry.id === id)
  if (!current) return
  if (layout.value === 'config-only') selectedEntryId.value = id
  if (Object.entries(patch).every(([field, value]) => current[field as keyof SchedulingPolicy] === value)) return
  entries.value = entries.value.map(entry => {
    if (entry.id !== id) return entry
    const updated = { ...entry, ...patch }
    if (updated.scope === 'selected') {
      updated.models = updated.models.filter(model => !otherModelOwners(id)[model])
    }
    return updated
  })
  publish()
}

function addEntry(): void {
  if (!canAddEntry.value) return
  editingModelsId.value = null
  const entry = createSchedulingPolicy(props.config)
  entries.value.push(entry)
  selectedEntryId.value = entry.id
  expandedId.value = entry.id
  publish()
}

function removeEntry(id: string): void {
  if (props.disabled || entries.value.length === 1) return
  editingModelsId.value = null
  entries.value = entries.value.filter(entry => entry.id !== id)
  if (entries.value.every(entry => entry.scope === 'all')) {
    scopeMode.value = 'all'
    selectedModelsDraft = null
  }
  if (expandedId.value === id) expandedId.value = entries.value[0]?.id ?? null
  if (selectedEntryId.value === id) selectedEntryId.value = entries.value[0]?.id ?? null
  publish()
}
</script>

<style scoped>
.scheduling-toolbar {
  display: grid;
  min-width: 0;
  align-items: end;
  gap: 12px;
}

@media (min-width: 640px) {
  .scheduling-toolbar:not(.scheduling-toolbar--sidebar) {
    grid-template-columns: minmax(0, 300px) minmax(0, 220px);
    column-gap: 20px;
  }

  .scheduling-toolbar--composed {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  }

  .scheduling-toolbar--composed .scheduling-toolbar__leading {
    grid-area: 1 / 1;
  }

  .scheduling-toolbar--composed .scheduling-toolbar__actions {
    grid-area: 1 / 2;
    justify-self: end;
  }

  .scheduling-toolbar--composed .scheduling-toolbar__scope {
    grid-area: 2 / 1 / 3 / -1;
  }
}

@media (min-width: 1280px) {
  .scheduling-toolbar--composed {
    grid-template-columns: minmax(200px, 280px) minmax(190px, 220px) minmax(max-content, 1fr);
  }

  .scheduling-toolbar--composed .scheduling-toolbar__scope {
    grid-area: 1 / 2;
  }

  .scheduling-toolbar--composed .scheduling-toolbar__actions {
    grid-area: 1 / 3;
  }
}

.scheduling-switch {
  display: grid;
  gap: 4px;
  padding: 4px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: color-mix(in oklab, var(--muted) 40%, var(--background));
}

.scheduling-switch__option {
  position: relative;
  display: flex;
  min-width: 0;
  min-height: 38px;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 7px 4px;
  border: 1px solid transparent;
  border-radius: 5px;
  color: color-mix(in oklab, var(--foreground) 75%, var(--muted-foreground));
  font-size: 14px;
  font-weight: 600;
  line-height: 20px;
  letter-spacing: 0;
  cursor: pointer;
  transition: background-color 150ms, border-color 150ms, color 150ms, box-shadow 150ms;
}

.config-only-editor .scheduling-switch {
  gap: 2px;
  padding: 2px;
}

.config-only-editor .scheduling-switch__option {
  min-height: 30px;
  padding: 4px;
  font-size: 13px;
}

.config-only-editor--sidebar .scheduling-switch__option {
  gap: 4px;
  font-size: 12px;
}

.scheduling-switch__option:hover:not(:disabled) {
  background: color-mix(in oklab, var(--primary) 8%, var(--background));
  color: var(--foreground);
}

.scheduling-switch__option[aria-pressed='true'] {
  background: var(--primary);
  color: var(--primary-foreground);
  box-shadow: 0 1px 2px color-mix(in oklab, var(--primary) 20%, transparent);
}

.scheduling-switch__option[aria-pressed='true']:hover:not(:disabled) {
  background: color-mix(in oklab, var(--primary) 92%, black);
  color: var(--primary-foreground);
}

.scheduling-switch__option:focus-visible {
  z-index: 1;
  outline: 2px solid var(--ring);
  outline-offset: 2px;
}

.scheduling-switch__option:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

@media (prefers-reduced-motion: reduce) {
  .scheduling-switch__option {
    transition: none;
  }
}
</style>
