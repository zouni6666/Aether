<template>
  <span
    class="inline-flex min-w-6 justify-center"
    @click.stop
    @mousedown.stop
    @pointerdown.stop
    @keydown.stop
  >
    <input
      v-if="editing"
      ref="inputRef"
      v-model="draft"
      type="number"
      min="0"
      max="2147483647"
      step="1"
      :disabled="disabled"
      :aria-label="`${providerName} 的组内优先级`"
      class="h-7 w-14 rounded border border-input bg-background px-1 text-center text-xs tabular-nums focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      @blur="finishEdit(false)"
      @keydown.enter.prevent="finishEdit(true)"
      @keydown.esc.prevent="cancelEdit(true)"
    >
    <button
      v-else
      ref="buttonRef"
      type="button"
      :disabled="disabled"
      :aria-label="`${providerName} 的组内优先级`"
      title="点击修改优先级"
      class="min-h-7 min-w-6 rounded px-1 text-xs font-medium tabular-nums text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-default disabled:opacity-50"
      @click="startEdit"
    >
      {{ priority }}
    </button>
  </span>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'

const props = defineProps<{ providerName: string; priority: number; disabled?: boolean; editContext?: string }>()
const emit = defineEmits<{ 'update:priority': [value: number] }>()
const editing = ref(false)
const draft = ref<string | number>('')
const inputRef = ref<HTMLInputElement | null>(null)
const buttonRef = ref<HTMLButtonElement | null>(null)

async function startEdit() {
  if (props.disabled) return
  draft.value = String(props.priority)
  editing.value = true
  await nextTick()
  inputRef.value?.focus()
  inputRef.value?.select()
}

async function cancelEdit(restoreFocus = false) {
  editing.value = false
  draft.value = String(props.priority)
  if (restoreFocus) {
    await nextTick()
    buttonRef.value?.focus({ preventScroll: true })
  }
}

function finishEdit(restoreFocus: boolean) {
  if (!editing.value) return
  const value = String(draft.value).trim() === '' ? NaN : Number(draft.value)
  const valid = !props.disabled && Number.isInteger(value) && value >= 0 && value <= 2147483647
  void cancelEdit(restoreFocus)
  if (valid && value !== props.priority) {
    emit('update:priority', value)
  }
}

watch(() => [props.priority, props.disabled, props.editContext], () => { void cancelEdit() }, { flush: 'sync' })
</script>
