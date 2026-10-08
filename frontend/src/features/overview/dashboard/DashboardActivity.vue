<template>
  <div class="grid grid-cols-1 items-stretch gap-4 xl:grid-cols-2">
    <Card
      class="min-w-0 overflow-hidden p-4 sm:p-5"
      :title="scopeHint"
      :aria-busy="loading"
      data-dashboard-activity
    >
      <div class="flex items-start justify-between gap-3 sm:items-center">
        <div class="flex min-w-0 flex-wrap items-baseline gap-x-3 gap-y-1">
          <h3 class="text-sm font-medium text-foreground">
            {{ t('连续活跃', 'Activity streak') }}
          </h3>
          <Skeleton
            v-if="loading"
            class="h-8 w-28"
          />
          <p
            v-else
            class="flex shrink-0 items-baseline gap-1.5"
            :title="t('连续活跃天数 / 总活跃天数。今天尚无请求时，从昨天向前计算；有请求的日期计为一个活跃日。', 'Consecutive active days / total active days. If today has no requests yet, the streak ends yesterday. Each date with requests counts as one active day.')"
          >
            <span class="text-[28px] font-semibold leading-8 tabular-nums text-foreground">{{ count(consecutiveActiveDays) }} / {{ count(activeDays) }}</span>
            <span class="text-xs text-muted-foreground">{{ t('天', 'days') }}</span>
          </p>
        </div>
        <div class="flex shrink-0 items-center gap-1 pt-1 text-[10px] text-muted-foreground sm:pt-0">
          <span class="mr-0.5">{{ t('少', 'Less') }}</span>
          <span
            v-for="level in legendLevels"
            :key="level"
            class="h-3 w-3 rounded-[4px]"
            :style="{ backgroundColor: `rgba(var(--color-primary-rgb), ${level})` }"
          />
          <span class="ml-0.5">{{ t('多', 'More') }}</span>
        </div>
      </div>
      <Skeleton
        v-if="loading"
        class="mt-4 h-24 w-full"
      />
      <ActivityHeatmap
        v-else-if="data?.days.length"
        class="mt-4"
        :data="data"
        :show-header="false"
        :compact="true"
      />
      <div
        v-else
        class="mt-4 flex h-24 items-center justify-center text-xs text-muted-foreground"
      >
        {{ t('暂无活跃数据', 'No activity yet') }}
      </div>
    </Card>
    <IntervalTimelineCard
      :title="t('全站请求间隔（最近24小时）', 'Site-wide request intervals (last 24 hours)')"
      :is-admin="true"
      :hours="24"
      :refresh-interval-ms="0"
      :data="timelineData"
    />
  </div>
</template>

<script setup lang="ts">
import { Card, Skeleton } from '@/components/ui'
import ActivityHeatmap from '@/components/stats/ActivityHeatmap.vue'
import IntervalTimelineCard from '@/features/usage/components/IntervalTimelineCard.vue'
import type { ActivityHeatmap as ActivityHeatmapData } from '@/types/activity'
import type { IntervalTimelineResponse } from '@/api/cache'
import { useOverviewI18n } from '../i18n'
import { count } from '../format'

defineProps<{ data: ActivityHeatmapData | null; consecutiveActiveDays: number | null; activeDays: number | null; scopeHint: string; loading: boolean; error: boolean; timelineData?: IntervalTimelineResponse | null }>()
const { t } = useOverviewI18n()
const legendLevels = [0.08, 0.25, 0.45, 0.65, 0.85]
</script>
