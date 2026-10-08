<template>
  <Button
    variant="ghost"
    size="icon"
    class="h-7 w-7"
    :class="enabled ? 'text-primary hover:text-primary' : 'text-muted-foreground/70 hover:text-foreground'"
    :disabled="disabled"
    :title="legacyT(enabled ? '本组禁用提供商' : '本组启用提供商')"
    :aria-label="`${providerName} ${legacyT('本组启用')}`"
    :aria-pressed="enabled"
    @click.stop="emit('update:enabled', !enabled)"
    @mousedown.stop
    @pointerdown.stop
  >
    <component
      :is="enabled ? ToggleRight : ToggleLeft"
      class="h-4 w-4"
      aria-hidden="true"
    />
  </Button>
</template>

<script setup lang="ts">
import { ToggleLeft, ToggleRight } from 'lucide-vue-next'
import Button from '@/components/ui/button.vue'
import { useI18n } from '@/i18n'

defineProps<{ providerName: string; enabled: boolean; disabled?: boolean }>()
const emit = defineEmits<{ 'update:enabled': [value: boolean] }>()
const { legacyT } = useI18n()
</script>
