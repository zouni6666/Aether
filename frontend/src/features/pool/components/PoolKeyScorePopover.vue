<template>
  <Popover
    :open="open"
    @update:open="(value: boolean) => $emit('update:open', value)"
  >
    <PopoverTrigger as-child>
      <Button
        variant="ghost"
        size="icon"
        class="h-5 w-5 rounded-full border border-transparent text-muted-foreground/80 hover:border-border/60 hover:bg-muted/60 hover:text-foreground"
        :title="t('ui.viewScoreCalculation')"
        :aria-label="t('ui.viewScoreCalculation')"
        @click.stop
      >
        <CircleHelp class="h-3.5 w-3.5" />
      </Button>
    </PopoverTrigger>
    <PopoverContent
      v-if="open"
      class="w-[22rem] max-w-[calc(100vw-1rem)] overflow-hidden rounded-xl border-border/60 bg-card/95 p-0 text-card-foreground shadow-xl shadow-black/5 backdrop-blur supports-[backdrop-filter]:bg-card/90"
      side="bottom"
      align="end"
      :side-offset="8"
    >
      <div class="text-left">
        <div class="flex items-center justify-between gap-3 border-b border-border/60 bg-muted/30 px-3 py-2.5">
          <span class="text-xs font-semibold text-foreground">{{ t('ui.scoreCalculation') }}</span>
          <span class="font-mono text-xs tabular-nums text-foreground/90">
            {{ formatPoolScore(score.score) }}
          </span>
        </div>
        <div class="space-y-2 px-3 py-2.5">
          <div class="flex flex-wrap items-center gap-1.5">
            <Badge
              variant="outline"
              class="h-5 rounded-md border-border/60 bg-background/60 px-2 text-[10px] font-normal"
            >
              {{ getPoolScoreHardStateLabel(score.hard_state) }}
            </Badge>
            <Badge
              variant="secondary"
              class="h-5 rounded-md px-2 text-[10px] font-normal"
            >
              {{ getPoolScoreProbeStatusLabel(score.probe_status) }}
            </Badge>
            <span class="text-[10px] text-muted-foreground"> {{ t('ui.update') }} {{ formatUnixSeconds(score.updated_at) }}
            </span>
          </div>
          <pre class="max-h-56 overflow-auto rounded-md border border-border/50 bg-muted/30 px-3 py-2 font-mono text-[11px] leading-5 text-muted-foreground whitespace-pre-wrap break-words">{{ formatPoolScoreReason(score.score_reason) }}</pre>
        </div>
      </div>
    </PopoverContent>
  </Popover>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { CircleHelp } from 'lucide-vue-next'
import { Badge, Button, Popover, PopoverContent, PopoverTrigger } from '@/components/ui'
import type { PoolKeyScoreDetail } from '@/api/endpoints/pool'
import {
  formatPoolScore,
  formatPoolScoreReason,
  formatUnixSeconds,
  getPoolScoreHardStateLabel,
  getPoolScoreProbeStatusLabel,
} from '../utils/poolKeyStatus'

/** 号池密钥评分的解释弹层：硬状态、探测状态、更新时间与评分依据 */
defineProps<{
  score: PoolKeyScoreDetail
  open: boolean
}>()

defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()

</script>
