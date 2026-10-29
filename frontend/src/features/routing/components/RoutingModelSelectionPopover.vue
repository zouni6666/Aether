<template>
  <Popover v-model:open="open">
    <PopoverTrigger as-child>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        class="h-7 w-7 shrink-0 text-muted-foreground"
        :disabled="disabled"
        :aria-label="t('ui.editModel')"
        :title="t('ui.editModel')"
      >
        <Pencil class="h-3.5 w-3.5" />
      </Button>
    </PopoverTrigger>
    <PopoverContent
      align="end"
      :side-offset="6"
      :collision-padding="12"
      class="w-[min(22rem,calc(100vw-1.5rem))] overflow-hidden p-0"
      :aria-label="t('ui.editApplicableModels')"
      @open-auto-focus.prevent="focusSearch"
    >
      <div class="flex items-center justify-between gap-2 px-3 pt-2 text-xs font-medium">
        <span>{{ t('ui.applicableModels') }}</span>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          class="h-7 w-7 text-muted-foreground"
          :aria-label="t('ui.closeModelSelection')"
          @click="open = false"
        >
          <X class="h-3.5 w-3.5" />
        </Button>
      </div>
      <RoutingModelSelector
        ref="selector"
        inline
        compact
        narrow
        :model-value="modelValue"
        :models="models"
        :assigned-models="assignedModels"
        :loading="loading"
        :error="error"
        :disabled="disabled"
        @update:model-value="emit('update:modelValue', $event)"
        @reload="emit('reload')"
        @close="open = false"
      />
      <div class="flex justify-end border-t border-border/50 px-2 py-1.5">
        <Button
          type="button"
          variant="ghost"
          size="sm"
          class="h-7 px-2 text-xs"
          :aria-label="t('ui.finishSelection')"
          @click="open = false"
        >
          {{ t('ui.completed') }}
        </Button>
      </div>
    </PopoverContent>
  </Popover>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, nextTick, ref, watch } from 'vue'
import { Pencil, X } from 'lucide-vue-next'
import { Button, Popover, PopoverContent, PopoverTrigger } from '@/components/ui'
import type { GlobalModelResponse } from '@/api/global-models'
import RoutingModelSelector from './RoutingModelSelector.vue'

const props = defineProps<{
  modelValue: string[]
  models: GlobalModelResponse[]
  assignedModels: Record<string, number>
  loading?: boolean
  error?: string | null
  disabled?: boolean
  open: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [models: string[]]
  reload: []
  'update:open': [open: boolean]
}>()

const { t } = useI18n()

const open = computed({
  get: () => props.open,
  set: value => emit('update:open', value),
})
const selector = ref<InstanceType<typeof RoutingModelSelector> | null>(null)

watch(() => props.disabled, disabled => { if (disabled) open.value = false })

async function focusSearch(): Promise<void> {
  await nextTick()
  selector.value?.focusSearch()
}

</script>
