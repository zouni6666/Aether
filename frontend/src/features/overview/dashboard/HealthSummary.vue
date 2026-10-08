<template>
  <section class="space-y-3 border-y py-4">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <RouterLink
        to="/admin/health-monitor?window=1h"
        class="inline-flex items-center gap-1 text-sm font-semibold hover:text-primary"
      >
        {{ t('服务健康', 'Service health') }}<ArrowUpRight class="h-3.5 w-3.5" />
      </RouterLink><span class="text-xs text-muted-foreground">{{ t('全站 API 格式 · 近 1 小时', 'All API formats · Last hour') }}</span>
    </div>
    <p
      v-if="loading && !data"
      class="text-xs text-muted-foreground"
    >
      {{ t('加载中', 'Loading') }}
    </p>
    <div
      v-else-if="error"
      role="alert"
      class="flex items-center gap-2 text-xs text-destructive"
    >
      <span>{{ t('健康数据暂不可用', 'Health data unavailable') }}</span><button
        class="underline"
        @click="refresh"
      >
        {{ t('重试', 'Retry') }}
      </button>
    </div>
    <dl
      v-if="data"
      class="grid grid-cols-2 gap-4 text-sm sm:grid-cols-4"
    >
      <div>
        <dt class="text-xs text-muted-foreground">
          {{ t('总体状态', 'Overall status') }}
        </dt><dd
          class="mt-1 font-semibold"
          :class="data.data.status === 'healthy' ? 'text-emerald-700 dark:text-emerald-400' : data.data.status === 'unknown' ? 'text-muted-foreground' : 'text-destructive'"
        >
          {{ status }}
        </dd>
      </div><div>
        <dt class="text-xs text-muted-foreground">
          {{ t('服务可用率', 'Service availability') }}
        </dt><dd class="mt-1 font-semibold tabular-nums">
          {{ percent(data.data.requests.service_availability.value) }}
        </dd>
      </div><div>
        <dt class="text-xs text-muted-foreground">
          {{ t('异常对象', 'Affected objects') }}
        </dt><dd class="mt-1 font-semibold tabular-nums">
          {{ count(data.data.degraded_count + data.data.unavailable_count) }} / {{ count(data.data.object_count) }}
        </dd>
      </div><div>
        <dt class="text-xs text-muted-foreground">
          {{ t('状态未知', 'Unknown status') }}
        </dt><dd class="mt-1 font-semibold tabular-nums">
          {{ count(data.data.unknown_count) }}
        </dd>
      </div>
    </dl>
    <p
      v-if="data"
      class="text-xs text-muted-foreground"
    >
      {{ t('更新', 'Updated') }} {{ timestamp(data.meta.generated_at) }} · {{ data.meta.freshness === 'current' ? t('采集正常', 'Current') : data.meta.freshness === 'stale' ? t('采集滞后', 'Delayed') : t('采集进度未知', 'Collection progress unknown') }}
    </p>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { ArrowUpRight } from 'lucide-vue-next'
import { getAdminHealthSummary } from '@/api/endpoints/health-v2'
import { useOverviewRequest } from '../useOverviewRequest'
import { useOverviewI18n } from '../i18n'
import { count, percent, timestamp } from '../format'
const props = defineProps<{ revision: number }>()
const { t } = useOverviewI18n()
const { data, loading, error, refresh } = useOverviewRequest(() => props.revision, signal => getAdminHealthSummary({ kind: 'api_format', window: '1h' }, signal), { scopeKey: () => 'api_format:1h' })
const status = computed(() => ({ healthy: t('正常', 'Healthy'), degraded: t('降级', 'Degraded'), unavailable: t('不可用', 'Unavailable'), unknown: t('未知', 'Unknown') })[data.value?.data.status || 'unknown'])
</script>
