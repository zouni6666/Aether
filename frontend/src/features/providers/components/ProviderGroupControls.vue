<template>
  <div
    class="flex items-center text-xs"
    :class="showPriority ? 'flex-wrap gap-x-4 gap-y-2 rounded-lg bg-muted/30 px-2.5 py-2' : 'justify-center'"
    @click.stop
    @mousedown.stop
    @pointerdown.stop
  >
    <label
      v-if="showPriority !== false"
      class="flex items-center gap-2"
    >
      <span class="text-muted-foreground">优先级</span>
      <ProviderPriorityInput
        :provider-name="providerName"
        :priority="priority"
        :edit-context="editContext"
        :disabled="disabled || priorityDisabled"
        @update:priority="emit('update:priority', $event)"
      />
    </label>
    <Badge
      :variant="enabled ? 'success' : 'secondary'"
      class="whitespace-nowrap text-xs"
    >
      {{ legacyT(enabled ? '本组启用' : '本组禁用') }}
    </Badge>
  </div>
</template>

<script setup lang="ts">
import Badge from '@/components/ui/badge.vue'
import { useI18n } from '@/i18n'
import ProviderPriorityInput from './ProviderPriorityInput.vue'

withDefaults(defineProps<{ providerName: string; priority: number; enabled: boolean; disabled?: boolean; priorityDisabled?: boolean; showPriority?: boolean; editContext?: string }>(), { showPriority: true, editContext: undefined })
const emit = defineEmits<{ 'update:priority': [value: number] }>()
const { legacyT } = useI18n()
</script>
