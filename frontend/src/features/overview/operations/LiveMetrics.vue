<template>
  <section class="space-y-4">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <h2 class="text-sm font-semibold">
        {{ resources ? t('资源诊断', 'Resource diagnostics') : t('实时运行', 'Live runtime') }}
      </h2><p class="min-w-0 break-words text-xs text-muted-foreground">
        {{ snapshot.scope.kind === 'cluster' ? t('已知节点集合', 'Known nodes') : t('当前节点', 'Current node') }} · {{ snapshot.node_id || t('节点标识未知', 'Unknown node ID') }} · {{ timestamp(snapshot.observed_at) }}
      </p>
    </div>
    <p
      v-if="snapshot.unavailable_sections.length"
      class="break-words rounded-xl border border-amber-500/20 bg-amber-500/5 px-3 py-2 text-xs text-amber-700 dark:text-amber-400"
    >
      {{ t('未采集', 'Unavailable') }}: {{ snapshot.unavailable_sections.join(', ') }}
    </p>
    <p
      v-if="!metrics"
      class="py-8 text-center text-sm text-muted-foreground"
    >
      {{ t('暂无资源采样', 'No resource samples') }}
    </p>
    <template v-else>
      <div class="grid auto-rows-fr grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
        <Card
          v-for="group in groups"
          :key="group.title"
          class="min-w-0 p-4 sm:p-5"
        >
          <div class="mb-5 flex items-center justify-between gap-3">
            <h3 class="text-sm font-semibold">
              {{ group.title }}
            </h3>
            <span class="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border border-border/60 bg-muted/30 text-muted-foreground">
              <component
                :is="group.icon"
                class="h-4 w-4"
                aria-hidden="true"
              />
            </span>
          </div><dl class="grid grid-cols-2 gap-x-5 gap-y-5">
            <div
              v-for="item in group.items"
              :key="item.label"
              class="min-w-0"
            >
              <dt class="min-h-8 break-words text-xs leading-4 text-muted-foreground">
                {{ item.label }}
              </dt><dd class="mt-1 min-w-0 font-semibold leading-8 tabular-nums">
                <MetricValue
                  :value="item.value"
                  :max-font-size="24"
                />
              </dd>
            </div>
          </dl>
        </Card>
      </div>
      <Card
        v-if="!resources && snapshot.resilience"
        class="min-w-0 space-y-4 p-4 sm:p-5"
      >
        <div class="flex flex-wrap items-center justify-between gap-3">
          <h3 class="flex items-center gap-2 text-sm font-semibold">
            <ShieldCheck
              class="h-4 w-4 text-muted-foreground"
              aria-hidden="true"
            />
            {{ t('服务韧性', 'Resilience') }}
          </h3><RouterLink
            :to="link('/admin/health-monitor')"
            class="text-xs text-primary hover:underline"
          >
            {{ t('健康监控', 'Health monitor') }}
          </RouterLink>
        </div><dl class="grid grid-cols-2 gap-5">
          <div class="min-w-0">
            <dt class="text-xs text-muted-foreground">
              {{ t('已打开熔断器', 'Open circuits') }}
            </dt><dd class="mt-2 font-semibold tabular-nums">
              <MetricValue
                :value="count(snapshot.resilience.error_statistics.open_circuit_breakers)"
                :max-font-size="24"
              />
            </dd>
          </div><div class="min-w-0">
            <dt class="text-xs text-muted-foreground">
              {{ t('降级凭据', 'Degraded credentials') }}
            </dt><dd class="mt-2 font-semibold tabular-nums">
              <MetricValue
                :value="count(snapshot.resilience.error_statistics.degraded_keys)"
                :max-font-size="24"
              />
            </dd>
          </div>
        </dl><ul
          v-if="snapshot.resilience.recent_errors.length"
          class="divide-y border-t text-xs"
        >
          <li
            v-for="item in snapshot.resilience.recent_errors.slice(0, 8)"
            :key="item.error_id"
            class="flex flex-wrap items-start justify-between gap-2 py-3"
          >
            <span class="min-w-0 break-all">{{ item.error_type }} · {{ item.context.provider_name || item.context.model || item.operation }}</span><RouterLink
              v-if="item.context.request_id"
              :to="link('/admin/usage', { request_id: item.context.request_id })"
              class="min-w-0 break-all font-mono text-primary hover:underline"
            >
              {{ item.context.request_id }}
            </RouterLink><span class="text-muted-foreground">{{ timestamp(item.timestamp) }}</span>
          </li>
        </ul>
      </Card>
      <Card
        v-if="resources && metrics.upstreamTargets.rows.length"
        class="min-w-0 space-y-4 p-4 sm:p-5"
      >
        <h3 class="flex items-center gap-2 text-sm font-semibold">
          <Network
            class="h-4 w-4 text-muted-foreground"
            aria-hidden="true"
          />
          {{ t('上游准入', 'Upstream admission') }}
        </h3><div class="overflow-x-auto">
          <table class="w-full min-w-[420px] text-sm">
            <thead>
              <tr class="border-b text-xs text-muted-foreground">
                <th class="py-2 text-left">
                  {{ t('目标', 'Target') }}
                </th><th class="px-3 text-right">
                  {{ t('处理中', 'In flight') }}
                </th><th class="px-3 text-right">
                  {{ t('可用许可', 'Available permits') }}
                </th><th class="px-3 text-right">
                  {{ t('拒绝累计', 'Rejected total') }}
                </th>
              </tr>
            </thead><tbody>
              <tr
                v-for="row in metrics.upstreamTargets.rows"
                :key="row.target"
                class="border-b"
              >
                <td class="max-w-64 break-words py-2">
                  {{ row.target }}
                </td><td class="px-3 text-right">
                  {{ count(row.inFlight) }}
                </td><td class="px-3 text-right">
                  {{ count(row.availablePermits) }}
                </td><td class="px-3 text-right">
                  {{ count(row.rejectedTotal) }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </Card>
    </template>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { Activity, Cpu, Database, Layers, ListChecks, ListTodo, Network, Radio, ShieldCheck } from 'lucide-vue-next'
import { Card } from '@/components/ui'
import { buildGatewayMetricsSummary } from '@/api/monitoring'
import type { OverviewLive } from '@/api/overview'
import MetricValue from '../components/MetricValue.vue'
import { useOverviewQuery } from '../query'
import { useOverviewI18n } from '../i18n'
import { count, timestamp } from '../format'
const props = defineProps<{ snapshot: OverviewLive; resources?: boolean }>()
const { t } = useOverviewI18n()
const { link } = useOverviewQuery()
const metrics = computed(() => props.snapshot.metrics || (props.snapshot.metrics_text ? buildGatewayMetricsSummary(props.snapshot.metrics_text) : null))
function bytes(value: number | null | undefined) { return value == null ? '-' : `${count(value / 1024 / 1024)} MiB` }
function basis(value: number | null | undefined) { return value == null ? '-' : `${count(value / 100)}%` }
const groups = computed(() => {
  const m = metrics.value
  if (!m) return []
  const runtime = [
    { title: t('请求准入', 'Request admission'), icon: Activity, items: [{ label: t('本机处理中', 'Node in flight'), value: count(m.local.inFlight) }, { label: t('本机可用许可', 'Node available permits'), value: count(m.local.availablePermits) }, { label: t('全局处理中', 'Global in flight'), value: count(m.distributed.inFlight) }, { label: t('全局可用许可', 'Global available permits'), value: count(m.distributed.availablePermits) }] },
    { title: t('代理通道', 'Proxy tunnels'), icon: Network, items: [{ label: t('连接数', 'Connections'), value: count(m.tunnel.proxyConnections) }, { label: t('活跃流', 'Active streams'), value: count(m.tunnel.activeStreams) }, { label: t('队列深度', 'Queue depth'), value: count(m.tunnel.outboundQueueDepthTotal) }, { label: t('队列容量', 'Queue capacity'), value: count(m.tunnel.outboundQueueCapacityTotal) }] },
    { title: t('用量写入', 'Usage writes'), icon: ListTodo, items: [{ label: t('待确认', 'Awaiting acknowledgement'), value: count(m.usageQueue.groupPending) }, { label: t('待消费', 'Awaiting delivery'), value: count(m.usageQueue.groupLag) }, { label: t('死信', 'Dead letters'), value: count(m.usageQueue.dlqLength) }, { label: t('最旧待处理 (ms)', 'Oldest pending (ms)'), value: count(m.usageQueue.oldestPendingIdleMs) }] },
  ]
  if (!props.resources) return runtime
  return [
    { title: t('进程与主机', 'Process and host'), icon: Cpu, items: [{ label: t('进程 CPU', 'Process CPU'), value: basis(m.process.processCpuUsageBasisPoints) }, { label: t('主机 CPU', 'Host CPU'), value: basis(m.process.systemCpuUsageBasisPoints) }, { label: t('进程内存', 'Process memory'), value: bytes(m.process.processMemoryBytes) }, { label: t('主机内存', 'Host memory'), value: basis(m.process.systemMemoryUsageBasisPoints) }, { label: t('打开文件', 'Open file descriptors'), value: count(m.process.openFds) }] },
    { title: 'PostgreSQL', icon: Database, items: [{ label: t('连接池占用', 'Pool checked out'), value: `${count(m.databasePool.checkedOut)} / ${count(m.databasePool.max)}` }, { label: t('数据库等待', 'Database waits'), value: count(m.postgres.waitingConnections) }, { label: t('等待锁', 'Lock waits'), value: count(m.postgres.lockWaitingConnections) }, { label: t('最长查询 (ms)', 'Oldest query (ms)'), value: count(m.postgres.oldestActiveQueryAgeMs) }, { label: t('块缓存命中', 'Block cache hit'), value: basis(m.postgres.blockCacheHitRateBasisPoints) }] },
    { title: 'Redis', icon: Radio, items: [{ label: t('已用内存', 'Used memory'), value: bytes(m.redisRuntime.usedMemoryBytes) }, { label: t('阻塞客户端', 'Blocked clients'), value: count(m.redisRuntime.blockedClients) }, { label: t('命中率', 'Hit rate'), value: basis(m.redisRuntime.keyspaceHitRateBasisPoints) }, { label: t('操作 / 秒', 'Operations / second'), value: count(m.redisRuntime.instantaneousOpsPerSec) }, { label: t('淘汰累计', 'Evictions total'), value: count(m.redisRuntime.evictedKeysTotal) }] },
    { title: t('后台任务与调度', 'Tasks and scheduling'), icon: Layers, items: [{ label: t('活跃任务', 'Active tasks'), value: count(m.backgroundTasks.active) }, { label: t('异常退出累计', 'Unexpected exits total'), value: count(m.backgroundTasks.unexpectedExitsTotal) }, { label: t('异步任务', 'Async tasks'), value: count(m.tokioRuntime.aliveTasks) }, { label: t('调度队列', 'Scheduling queue'), value: count(m.tokioRuntime.globalQueueDepth) }] },
    { title: t('审计计数', 'Audit counters'), icon: ListChecks, items: [{ label: t('待刷新', 'Pending flush'), value: count(m.usageCounter.pendingRows) }, { label: t('最旧待处理 (s)', 'Oldest pending (s)'), value: count(m.usageCounter.oldestPendingAgeSeconds) }, { label: t('失败批次累计', 'Failed batches total'), value: count(m.usageCounter.flushFailedBatchesTotal) }, { label: t('候选队列', 'Candidate queue'), value: count(m.requestCandidateQueue.depth) }] },
  ]
})
</script>
