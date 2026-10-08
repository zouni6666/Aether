<template>
  <div
    v-if="error"
    role="alert"
    class="flex flex-wrap items-center justify-between gap-2 border-l-2 border-destructive bg-destructive/5 px-3 py-2 text-sm"
  >
    <span class="min-w-0 break-words">{{ t('数据加载失败', 'Unable to load data') }}: {{ error }}</span>
    <Button
      variant="outline"
      size="sm"
      :disabled="loading"
      @click="$emit('retry')"
    >
      {{ t('重试', 'Retry') }}
    </Button>
  </div>
  <div
    v-if="loading && !meta"
    class="flex min-h-40 items-center justify-center gap-2 text-sm text-muted-foreground"
    role="status"
  >
    <Loader2 class="h-4 w-4 animate-spin" />{{ t('加载中', 'Loading') }}
  </div>
  <div
    v-if="meta"
    class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground"
  >
    <span :class="partial ? 'text-amber-700 dark:text-amber-400' : ''">{{ partial ? t('部分数据', 'Partial data') : t('数据完整', 'Complete') }}</span>
    <span>{{ t('计量覆盖', 'Usage coverage') }} {{ count(meta.coverage.usage_available_count) }}/{{ count(meta.coverage.request_count) }}</span>
    <span>{{ t('定价覆盖', 'Pricing coverage') }} {{ count(meta.coverage.pricing_available_count) }}/{{ count(meta.coverage.request_count) }}</span>
    <span>{{ t('已结算', 'Settled') }} {{ count(meta.coverage.settled_count) }}/{{ count(meta.coverage.request_count) }}</span>
    <span
      v-if="meta.coverage.unrecoverable_bucket_count"
      class="text-amber-700 dark:text-amber-400"
    >{{ t('原始数据已清理，无法恢复的时间桶', 'Source data removed; unrecoverable buckets') }} {{ count(meta.coverage.unrecoverable_bucket_count) }}</span>
    <span
      v-if="stale"
      class="text-amber-700 dark:text-amber-400"
    >{{ t('数据水位滞后', 'Data delayed') }}</span>
    <span>{{ t('更新', 'Updated') }} {{ timestamp(meta.generated_at, meta.range.timezone) }}</span>
    <span v-if="meta.data_through">{{ t('数据截至', 'Data through') }} {{ timestamp(meta.data_through, meta.range.timezone) }}</span>
  </div>
  <div
    v-if="meta?.projection"
    class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground"
  >
    <span v-if="!meta.projection.read_enabled">{{ t('原始事实读取', 'Reading source facts') }}</span>
    <span>{{ t('聚合覆盖至', 'Aggregate coverage through') }} {{ timestamp(meta.projection.projection_through, meta.range.timezone) }}</span>
    <span>{{ t('待重建', 'Awaiting rebuild') }} {{ count(meta.projection.dirty_bucket_count) }}</span>
    <span>{{ t('未建', 'Not built') }} {{ count(meta.projection.missing_bucket_count) }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Loader2 } from 'lucide-vue-next'
import { Button } from '@/components/ui'
import type { OverviewMeta } from '@/api/overview'
import { count, timestamp } from '../format'
import { useOverviewI18n } from '../i18n'
const props = defineProps<{ meta?: OverviewMeta | null; loading?: boolean; error?: string | null }>()
defineEmits<{ retry: [] }>()
const { t } = useOverviewI18n()
const partial = computed(() => props.meta?.coverage.status !== 'complete')
const stale = computed(() => !!props.meta?.data_through && Date.parse(props.meta.data_through) < Math.min(Date.parse(props.meta.range.to), Date.now()) - 300_000)
</script>
