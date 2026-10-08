<template>
  <div
    class="min-w-0"
    :role="inline ? 'group' : undefined"
    :aria-label="inline ? '选择适用模型' : undefined"
    :class="compact ? 'space-y-1' : 'space-y-2'"
  >
    <div class="min-w-0 space-y-2">
      <div
        class="overflow-hidden bg-background"
        :class="inline ? '' : 'rounded-lg border border-border/60'"
      >
        <button
          v-if="!inline"
          ref="trigger"
          type="button"
          class="flex w-full items-center justify-between gap-2 px-3 text-left text-sm font-normal text-foreground transition-colors hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
          :class="compact ? 'min-h-8 py-1' : 'min-h-10 py-2'"
          :disabled="disabled"
          aria-label="选择适用模型"
          :aria-expanded="open"
          :aria-controls="listId"
          @click="open ? closeModels() : openModels()"
        >
          <span
            class="min-w-0 flex-1 truncate"
            :class="!modelValue.length ? 'text-muted-foreground' : ''"
          >
            {{ selectionLabel }}
          </span>
          <ChevronDown
            class="h-4 w-4 shrink-0 text-muted-foreground transition-transform"
            :class="open ? 'rotate-180' : ''"
          />
        </button>
        <div
          v-if="inline || open"
          :id="listId"
          class="flex min-w-0 flex-col"
          :class="inline ? '' : 'border-t border-border/60'"
          role="region"
          aria-label="全局模型选择列表"
          @keydown.esc.stop.prevent="closeModels"
        >
          <div
            class="relative shrink-0 px-2"
            :class="compact ? 'py-1.5' : 'py-2'"
          >
            <Search class="pointer-events-none absolute left-5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input
              ref="searchInput"
              v-model="search"
              size="sm"
              class="rounded-md border-border/60 bg-background pl-9 pr-3 text-sm"
              :class="compact ? 'h-8' : 'h-9'"
              placeholder="搜索模型名称"
              aria-label="搜索全局模型"
              :disabled="disabled"
            />
          </div>

          <p
            v-if="loading"
            class="p-6 text-center text-xs text-muted-foreground"
          >
            正在加载全局模型
          </p>
          <div
            v-else-if="error"
            role="alert"
            class="flex items-center justify-between gap-3 p-4 text-xs text-destructive"
          >
            <span class="min-w-0 break-words">{{ error }}</span>
            <Button
              type="button"
              variant="outline"
              size="sm"
              class="shrink-0"
              :disabled="disabled"
              @click="emit('reload')"
            >
              重试
            </Button>
          </div>
          <template v-else>
            <div class="flex min-h-8 shrink-0 items-center justify-between gap-2 px-3 py-1 text-xs text-muted-foreground">
              <span>指定全局模型</span>
              <Button
                v-if="selectableRows.length"
                type="button"
                variant="ghost"
                size="sm"
                class="h-6 px-1.5 text-xs font-normal"
                :disabled="disabled"
                :aria-label="search.trim() ? '全选搜索结果' : '选择当前列表'"
                :aria-pressed="allResultsSelected"
                @click="selectResults(!allResultsSelected)"
              >
                {{ allResultsSelected ? '取消当前选择' : search.trim() ? '全选结果' : '全选当前' }}
              </Button>
            </div>
            <div
              class="grid min-h-0 grid-cols-1 gap-1 overflow-y-auto overscroll-contain px-2"
              :class="[
                compact ? 'max-h-48 py-1' : 'max-h-64 py-2',
                narrow ? '' : compact ? 'sm:grid-cols-2 xl:grid-cols-3' : 'sm:grid-cols-2',
              ]"
            >
              <label
                v-for="model in filteredModels"
                :key="model.name"
                class="flex min-w-0 items-center gap-3 rounded-md px-2 text-sm"
                :class="[
                  compact ? 'py-1.5' : 'py-2',
                  model.owner ? 'cursor-not-allowed opacity-50' : 'cursor-pointer hover:bg-muted/50',
                  selectedModels.includes(model.name) ? 'bg-accent/60' : '',
                ]"
              >
                <Checkbox
                  :checked="selectedModels.includes(model.name)"
                  :disabled="disabled || Boolean(model.owner)"
                  :aria-label="`选择模型 ${model.name}`"
                  @update:checked="selected => toggleModel(model.name, selected)"
                />
                <span class="min-w-0 flex-1">
                  <span
                    class="block truncate"
                    :title="model.displayName"
                  >{{ model.displayName }}</span>
                  <span
                    v-if="model.displayName !== model.name"
                    class="block truncate text-xs text-muted-foreground"
                    :title="model.name"
                  >
                    {{ model.name }}
                  </span>
                  <span
                    v-if="model.owner"
                    class="block text-xs text-muted-foreground"
                  >
                    已用于配置 {{ model.owner }}
                  </span>
                </span>
              </label>
              <p
                v-if="filteredModels.length === 0"
                class="col-span-full px-3 py-6 text-center text-xs text-muted-foreground"
              >
                {{ search.trim() ? '未匹配到全局模型' : '暂无可选模型' }}
              </p>
            </div>
          </template>

          <div
            v-if="!inline"
            class="flex shrink-0 flex-wrap items-center gap-1 border-t border-border/60 px-2"
            :class="compact ? 'py-1' : 'py-2'"
          >
            <span class="flex-1 whitespace-nowrap text-xs text-muted-foreground">
              已选 {{ selectedModels.length }} 个
            </span>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              class="h-7 px-2 text-xs font-normal text-muted-foreground"
              :disabled="disabled || selectedModels.length === 0"
              aria-label="清空已选"
              @click="updateModels([])"
            >
              {{ narrow ? '清空' : '清空已选' }}
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              class="h-7 gap-1 px-2 text-xs font-medium"
              :disabled="disabled"
              aria-label="完成选择"
              @click="closeModels"
            >
              <Check class="h-3.5 w-3.5" />
              {{ narrow ? '完成' : '完成选择' }}
            </Button>
          </div>
        </div>
      </div>
    </div>
    <p
      v-if="modelValue.length === 0 && !inline"
      class="text-xs text-muted-foreground"
    >
      支持多选，选中的模型共用一套调度设置。
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, useId, watch } from 'vue'
import { Check, ChevronDown, Search } from 'lucide-vue-next'
import { Button, Checkbox, Input } from '@/components/ui'
import type { GlobalModelResponse } from '@/api/global-models'

const props = defineProps<{
  modelValue: string[]
  models: GlobalModelResponse[]
  assignedModels: Record<string, number>
  loading?: boolean
  error?: string | null
  disabled?: boolean
  compact?: boolean
  narrow?: boolean
  /** Keep the search and model checklist mounted inside the parent card. */
  inline?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [models: string[]]
  reload: []
  close: []
}>()

const trigger = ref<HTMLButtonElement | null>(null)
const searchInput = ref<InstanceType<typeof Input> | null>(null)
const listId = useId()
const open = ref(props.modelValue.length === 0 && !props.disabled)
const search = ref('')
const selectedModels = computed(() => props.modelValue)
const selectionLabel = computed(() => {
  if (!props.modelValue.length) return '请选择全局模型'
  if (props.modelValue.length <= 2) return props.modelValue.map(modelLabel).join('、')
  return `已选择 ${props.modelValue.length} 个模型`
})
const filteredModels = computed(() => {
  const models = new Map(props.models.map(model => [model.name, {
    name: model.name,
    displayName: model.display_name || model.name,
    owner: props.assignedModels[model.name],
  }]))
  for (const name of props.modelValue) {
    if (!models.has(name)) models.set(name, { name, displayName: name, owner: props.assignedModels[name] })
  }
  const query = search.value.trim().toLowerCase()
  return [...models.values()].filter(model => query
    ? model.name.toLowerCase().includes(query) || model.displayName.toLowerCase().includes(query)
    : !model.owner)
})
const selectableRows = computed(() => filteredModels.value.filter(model => !model.owner))
const allResultsSelected = computed(() => selectableRows.value.length > 0
  && selectableRows.value.every(model => selectedModels.value.includes(model.name)))

watch(open, value => { if (!value) search.value = '' })
watch(() => props.disabled, disabled => { if (disabled) open.value = false })

async function openModels(): Promise<void> {
  if (props.disabled) return
  open.value = true
  await nextTick()
  focusSearch()
}

function closeModels(): void {
  if (props.inline) {
    emit('close')
    return
  }
  open.value = false
  trigger.value?.focus({ preventScroll: true })
}

function focusSearch(): void {
  searchInput.value?.inputRef?.focus({ preventScroll: true })
}

defineExpose({ focusSearch })

function modelLabel(name: string): string {
  return props.models.find(model => model.name === name)?.display_name || name
}

function updateModels(models: string[]): void {
  if (props.disabled) return
  emit('update:modelValue', models)
}

function toggleModel(model: string, selected: boolean): void {
  if (props.assignedModels[model]) return
  updateModels(selected ? [...new Set([...selectedModels.value, model])] : selectedModels.value.filter(name => name !== model))
}

function selectResults(selected: boolean): void {
  const names = new Set(selectableRows.value.map(model => model.name))
  updateModels(selected ? [...new Set([...selectedModels.value, ...names])] : selectedModels.value.filter(name => !names.has(name)))
}
</script>
