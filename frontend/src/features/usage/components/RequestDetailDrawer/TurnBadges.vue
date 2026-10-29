<template>
  <div class="flex items-center gap-1">
    <Brain
      v-if="showBadge('thinking') && stats.hasThinking"
      class="w-3.5 h-3.5 text-muted-foreground"
      :title="t('ui.includesReasoning')"
    />
    <Wrench
      v-if="showBadge('tool') && stats.hasToolUse"
      class="w-3.5 h-3.5 text-muted-foreground"
      :title="t('ui.includesToolCalls')"
    />
    <span
      v-if="showBadge('tool') && stats.toolCount > 1"
      class="text-xs text-muted-foreground"
    >×{{ stats.toolCount }}</span>
    <ImageIcon
      v-if="showBadge('image') && stats.hasImage"
      class="w-3.5 h-3.5 text-muted-foreground"
      :title="t('ui.includesImages')"
    />
    <AlertCircle
      v-if="showBadge('error') && stats.hasError"
      class="w-3.5 h-3.5 text-muted-foreground"
      :title="t('ui.includesErrors')"
    />
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { Brain, Wrench, Image as ImageIcon, AlertCircle } from 'lucide-vue-next'
import type { TurnStats } from '../../conversation/types'

type BadgeType = 'thinking' | 'tool' | 'image' | 'error'

const props = defineProps<{
  stats: TurnStats
  showOnly?: BadgeType[]
}>()

const { t } = useI18n()

function showBadge(type: BadgeType): boolean {
  if (!props.showOnly || props.showOnly.length === 0) {
    return true
  }
  return props.showOnly.includes(type)
}

</script>
