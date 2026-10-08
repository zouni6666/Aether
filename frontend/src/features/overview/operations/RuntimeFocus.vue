<template>
  <section
    data-runtime-focus
    class="space-y-3"
  >
    <div class="flex items-center justify-between gap-3">
      <h2 class="text-sm font-semibold">
        {{ t('处理链路', 'Processing pipeline') }}
      </h2>
      <Badge
        variant="outline"
        class="text-[10px]"
      >
        {{ t('实时', 'Live') }}
      </Badge>
    </div>
    <div class="grid grid-cols-1 items-stretch gap-4 lg:grid-cols-3">
      <Card
        v-for="group in groups"
        :key="group.key"
        :data-runtime-focus-group="group.key"
        class="min-w-0 p-4"
      >
        <div class="mb-3 flex min-h-5 flex-wrap items-center justify-between gap-2">
          <h3 class="flex items-center gap-2 text-sm font-semibold">
            <component
              :is="group.icon"
              class="h-4 w-4 text-muted-foreground"
              aria-hidden="true"
            />
            {{ group.title }}
          </h3>
          <RouterLink
            v-if="group.key === 'upstream'"
            :to="link('/admin/health-monitor')"
            class="text-xs text-primary hover:underline"
          >
            {{ t('健康监控', 'Health monitor') }}
          </RouterLink>
          <span
            v-if="group.key === 'usage' && queueStatus"
            data-runtime-focus-queue-state
            class="text-xs text-muted-foreground"
          >
            {{ queueStatus }}
          </span>
        </div>
        <dl class="divide-y divide-border/50">
          <div
            v-for="item in group.items"
            :key="item.key"
            :data-runtime-focus-item="item.key"
            class="flex items-center justify-between gap-3 py-2"
          >
            <dt
              class="text-xs text-muted-foreground"
              :title="item.description"
            >
              {{ item.label }}
            </dt>
            <dd class="shrink-0 text-sm font-semibold tabular-nums">
              {{ count(item.value) }}
            </dd>
          </div>
        </dl>
      </Card>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { GitBranch, ListTodo, ShieldCheck } from 'lucide-vue-next'
import { Badge, Card } from '@/components/ui'
import { buildGatewayMetricsSummary } from '@/api/monitoring'
import type { OverviewLive } from '@/api/overview'
import { count } from '../format'
import { useOverviewI18n } from '../i18n'
import { useOverviewQuery } from '../query'

const props = defineProps<{ snapshot: OverviewLive }>()
const { t } = useOverviewI18n()
const { link } = useOverviewQuery()
const metrics = computed(() => props.snapshot.metrics || (props.snapshot.metrics_text ? buildGatewayMetricsSummary(props.snapshot.metrics_text) : null))
const queueStatus = computed(() => {
  const queue = metrics.value?.usageQueue
  if (queue?.enabled === false) return t('未启用', 'Disabled')
  if (queue?.configured === false) return t('未配置', 'Not configured')
  if (queue?.unavailable === true || !queue || [queue.groupLag, queue.groupPending, queue.dlqLength].every(value => value == null)) return t('未采集', 'Unavailable')
  return ''
})
const groups = computed(() => {
  const m = metrics.value
  const upstream = props.snapshot.resilience?.error_statistics
  const postgres = m?.postgres?.unavailable === true || m?.postgres?.available === false ? undefined : m?.postgres
  const queue = queueStatus.value ? undefined : m?.usageQueue
  return [
    {
      key: 'upstream', title: t('上游可用性', 'Upstream availability'), icon: ShieldCheck,
      items: [
        { key: 'open-circuits', label: t('打开熔断器', 'Open circuits'), value: upstream?.open_circuit_breakers },
        { key: 'degraded-credentials', label: t('降级凭据', 'Degraded credentials'), value: upstream?.degraded_keys },
      ],
    },
    {
      key: 'requests', title: t('请求处理', 'Request processing'), icon: GitBranch,
      items: [
        { key: 'candidate-queue', label: t('候选队列', 'Candidate queue'), value: m?.requestCandidateQueue?.depth },
        { key: 'database-waiting', label: t('数据库等待', 'Database waits'), value: postgres?.waitingConnections, description: t('存在等待事件的活跃数据库会话', 'Active database sessions with a wait event') },
        { key: 'lock-waiting', label: t('锁等待', 'Lock waits'), value: postgres?.lockWaitingConnections, description: t('数据库等待中正在等待锁的会话', 'Database sessions waiting for a lock') },
      ],
    },
    {
      key: 'usage', title: t('用量落盘', 'Usage persistence'), icon: ListTodo,
      items: [
        { key: 'usage-lag', label: t('待消费', 'Awaiting delivery'), value: queue?.groupLag, description: t('尚未投递给消费组的条目', 'Entries not yet delivered to the consumer group') },
        { key: 'usage-pending', label: t('待确认', 'Awaiting acknowledgement'), value: queue?.groupPending, description: t('已投递但尚未确认完成的条目', 'Delivered entries not yet acknowledged') },
        { key: 'usage-dead-letters', label: t('死信', 'Dead letters'), value: queue?.dlqLength, description: t('当前死信队列中的条目', 'Entries currently retained in the dead-letter queue') },
      ],
    },
  ]
})
</script>
