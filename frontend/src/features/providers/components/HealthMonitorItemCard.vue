<template>
  <div class="relative overflow-hidden rounded-xl border border-border/60 bg-card/60 p-4 transition-colors hover:border-primary/50">
    <div class="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-primary/40 to-transparent" />

    <div class="flex items-start justify-between gap-3">
      <div class="flex min-w-0 items-center gap-3">
        <div class="flex h-11 w-11 flex-shrink-0 items-center justify-center rounded-xl border border-border/60 bg-muted/50">
          <component
            :is="icon"
            class="h-5 w-5 text-muted-foreground"
          />
        </div>
        <div class="min-w-0">
          <h4 class="truncate text-sm font-semibold">
            {{ title }}
          </h4>
          <p
            v-if="subtitle"
            class="mt-1 truncate text-xs text-muted-foreground"
          >
            {{ subtitle }}
          </p>
        </div>
      </div>
      <Badge
        :variant="getHealthBadgeVariant(monitor)"
        class="shrink-0"
      >
        {{ getHealthLabel(monitor) }}
      </Badge>
    </div>

    <HealthMetricGrid
      class="mt-4"
      :avg-latency-ms="monitor.avg_latency_ms"
      :avg-first-byte-ms="monitor.avg_first_byte_ms"
      :avg-tps="monitor.avg_tps"
      :total-attempts="monitor.total_attempts"
      :success-rate="monitor.success_rate"
    />

    <div class="mt-4 flex items-center justify-between gap-3 text-[11px] uppercase tracking-wide text-muted-foreground">
      <span>History (60pts)</span>
      <span class="truncate normal-case tracking-normal">
        {{ metaText }}
      </span>
    </div>

    <slot name="timeline">
      <HealthStatusTimeline
        class="mt-2"
        :timeline="monitor.timeline"
        :timeline-details="monitor.timeline_details"
        :time-range-start="monitor.time_range_start"
        :time-range-end="monitor.time_range_end"
        :generated-at="generatedAt"
        :lookback-hours="lookbackHours"
        :entity-label="entityLabel"
        :entity-name="entityName"
      />
    </slot>

    <div
      v-if="showDetailButton"
      class="mt-4 flex justify-end"
    >
      <Button
        variant="outline"
        size="sm"
        @click="$emit('viewDetails')"
      >
        {{ t('ui.viewDetails') }}
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import type { Component } from 'vue'
import Badge from '@/components/ui/badge.vue'
import Button from '@/components/ui/button.vue'
import HealthMetricGrid from './HealthMetricGrid.vue'
import HealthStatusTimeline from './HealthStatusTimeline.vue'
import { getHealthBadgeVariant, getHealthLabel, type HealthMonitorItemMetrics } from './health-monitor-utils'

/**
 * 单个健康监控对象的卡片：标题、健康徽章、指标网格、历史时间线与详情按钮。
 * 时间线可通过 timeline 插槽替换（端点维度使用事件回退渲染）。
 */
withDefaults(defineProps<{
  icon: Component
  title: string
  subtitle?: string | null
  metaText: string
  monitor: HealthMonitorItemMetrics
  lookbackHours: number
  generatedAt?: string | null
  entityLabel: string
  entityName?: string | null
  showDetailButton?: boolean
}>(), {
  subtitle: null,
  generatedAt: null,
  entityName: null,
  showDetailButton: true,
})

defineEmits<{
  viewDetails: []
}>()

const { t } = useI18n()

</script>
