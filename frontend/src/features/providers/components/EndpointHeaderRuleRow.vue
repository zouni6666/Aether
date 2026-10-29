<template>
  <div
    class="flex items-center gap-1.5 px-2 py-1.5 rounded-md border-l-4 bg-muted/30"
    :class="rowClass"
    @dragover.prevent="$emit('dragover')"
    @dragleave="$emit('dragleave')"
    @drop.prevent="$emit('drop')"
  >
    <button
      type="button"
      class="h-7 w-6 shrink-0 inline-flex items-center justify-center rounded-sm text-muted-foreground/60 hover:text-muted-foreground hover:bg-muted cursor-grab active:cursor-grabbing"
      :title="t('ui.dragToReorder')"
      draggable="true"
      @dragstart="(e: DragEvent) => $emit('dragstart', e)"
      @dragend="$emit('dragend')"
    >
      <GripVertical class="w-3.5 h-3.5" />
    </button>
    <span
      class="text-[10px] font-semibold shrink-0"
      :class="palette.marker"
      :title="labels.marker"
    >{{ labels.markerText }}</span>
    <Switch
      :model-value="rule.enabled"
      class="shrink-0 scale-75 origin-center"
      :title="rule.enabled ? labels.disableHint : labels.enableHint"
      @update:model-value="(v: boolean) => $emit('update:enabled', v)"
    />
    <Select
      :model-value="rule.action"
      :open="selectOpen"
      @update:model-value="(v) => $emit('update:action', v as HeaderRuleAction)"
      @update:open="(v) => $emit('update:selectOpen', v)"
    >
      <SelectTrigger class="w-[88px] h-7 text-xs shrink-0">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="set">
          {{ t('ui.overwrite') }}
        </SelectItem>
        <SelectItem value="drop">
          {{ t('ui.delete') }}
        </SelectItem>
        <SelectItem value="rename">
          {{ t('ui.rename') }}
        </SelectItem>
      </SelectContent>
    </Select>
    <Button
      variant="ghost"
      size="icon"
      class="h-7 w-7 shrink-0"
      :class="rule.condition ? 'text-primary' : ''"
      :title="t('ui.conditional')"
      @click="$emit('toggleCondition')"
    >
      <Filter class="w-3 h-3" />
    </Button>
    <template v-if="rule.action === 'set'">
      <Input
        :model-value="rule.key"
        :placeholder="labels.namePlaceholder"
        size="sm"
        class="flex-1 min-w-0 h-7 text-xs"
        @update:model-value="(v) => $emit('update:field', 'key', v)"
      />
      <span class="text-muted-foreground text-xs">=</span>
      <Input
        :model-value="rule.value"
        :placeholder="t('ui.value')"
        size="sm"
        class="flex-1 min-w-0 h-7 text-xs"
        @update:model-value="(v) => $emit('update:field', 'value', v)"
      />
    </template>
    <template v-else-if="rule.action === 'drop'">
      <Input
        :model-value="rule.key"
        :placeholder="labels.dropPlaceholder"
        size="sm"
        class="flex-1 min-w-0 h-7 text-xs"
        @update:model-value="(v) => $emit('update:field', 'key', v)"
      />
    </template>
    <template v-else-if="rule.action === 'rename'">
      <Input
        :model-value="rule.from"
        :placeholder="t('ui.originalName')"
        size="sm"
        class="flex-1 min-w-0 h-7 text-xs"
        @update:model-value="(v) => $emit('update:field', 'from', v)"
      />
      <span class="text-muted-foreground text-xs">→</span>
      <Input
        :model-value="rule.to"
        :placeholder="t('ui.newName')"
        size="sm"
        class="flex-1 min-w-0 h-7 text-xs"
        @update:model-value="(v) => $emit('update:field', 'to', v)"
      />
    </template>
    <Button
      variant="ghost"
      size="icon"
      class="h-7 w-7 shrink-0"
      @click="$emit('remove')"
    >
      <X class="w-3 h-3" />
    </Button>
  </div>
  <EndpointConditionEditor
    v-if="rule.condition"
    :model-value="rule.condition"
    :path-hint="labels.conditionPathHint"
    removable
    @update:model-value="(condition) => $emit('update:condition', condition)"
    @remove="$emit('clearCondition')"
  />
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed } from 'vue'
import { Filter, GripVertical, X } from 'lucide-vue-next'
import {
  Button,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
} from '@/components/ui'
import EndpointConditionEditor from './EndpointConditionEditor.vue'
import type { EditableConditionNode } from './endpoint-rule-condition'
import type { EditableRule } from './endpoint-rule-editing'

export type HeaderRuleAction = EditableRule['action']
export type HeaderRuleField = 'key' | 'value' | 'from' | 'to'

/**
 * 单条请求头 / 响应头规则的编辑行：拖拽把手、启用开关、动作选择、
 * 条件触发、字段输入与删除。请求头与响应头只在配色和文案上不同。
 */
const props = withDefaults(defineProps<{
  rule: EditableRule
  kind?: 'request' | 'response'
  selectOpen?: boolean
  dragging?: boolean
  dragOver?: boolean
}>(), {
  kind: 'request',
  selectOpen: false,
  dragging: false,
  dragOver: false,
})

defineEmits<{
  dragover: []
  dragleave: []
  drop: []
  dragstart: [event: DragEvent]
  dragend: []
  'update:enabled': [value: boolean]
  'update:action': [value: HeaderRuleAction]
  'update:selectOpen': [value: boolean]
  'update:field': [field: HeaderRuleField, value: string]
  'update:condition': [condition: EditableConditionNode]
  toggleCondition: []
  clearCondition: []
  remove: []
}>()

const { t } = useI18n()

const palette = computed(() => (props.kind === 'request'
  ? {
      border: 'border-primary/60',
      disabled: 'border-primary/25',
      dragging: 'border-primary bg-primary/5',
      dragOver: 'ring-primary/40 bg-primary/10',
      marker: 'text-primary',
    }
  : {
      border: 'border-sky-500/60',
      disabled: 'border-sky-500/25',
      dragging: 'border-sky-500 bg-sky-500/5',
      dragOver: 'ring-sky-500/40 bg-sky-500/10',
      marker: 'text-sky-600 dark:text-sky-400',
    }))

const labels = computed(() => (props.kind === 'request'
  ? {
      marker: t('ui.requestHeaders'),
      markerText: 'H',
      enableHint: t('ui.enableThisRequestHeaderRule'),
      disableHint: t('ui.disableThisRequestHeaderRule'),
      namePlaceholder: t('stats.column.name'),
      dropPlaceholder: t('ui.nameToDelete'),
      conditionPathHint: t('ui.requestBodyFieldPath'),
    }
  : {
      marker: t('ui.responseHeaders'),
      markerText: 'R',
      enableHint: t('ui.enableThisResponseHeaderRule'),
      disableHint: t('ui.disableThisResponseHeaderRule'),
      namePlaceholder: t('ui.responseHeaderName'),
      dropPlaceholder: t('ui.responseHeaderToDelete'),
      conditionPathHint: t('ui.responseBodyFieldPath'),
    }))

const rowClass = computed(() => [
  palette.value.border,
  !props.rule.enabled ? `opacity-60 ${palette.value.disabled} bg-muted/20` : '',
  props.dragging ? `opacity-60 ${palette.value.dragging}` : '',
  props.dragOver ? `ring-1 ${palette.value.dragOver}` : '',
])

</script>
