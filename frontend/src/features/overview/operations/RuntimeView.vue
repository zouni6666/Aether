<template>
  <section
    data-operations-core
    class="space-y-3"
  >
    <OverviewStatus
      :loading="loading && !data"
      :error="error"
      @retry="refresh"
    />
    <div
      v-if="data"
      class="grid grid-cols-1 items-stretch gap-6 lg:grid-cols-2"
    >
      <section
        data-operations-hardware
        class="flex min-w-0 flex-col gap-3"
      >
        <div class="flex items-center justify-between gap-3">
          <h2 class="text-sm font-semibold">
            {{ t('硬件性能', 'Hardware performance') }}
          </h2>
          <Badge
            variant="outline"
            class="text-[10px]"
          >
            {{ t('当前节点', 'Current node') }}
          </Badge>
        </div>
        <div class="grid flex-1 grid-cols-2 gap-3 sm:gap-4">
          <OperationsMetricCard
            v-for="item in hardwareMetrics"
            :key="item.label"
            v-bind="item"
          />
        </div>
      </section>
      <section
        data-operations-gateway
        class="flex min-w-0 flex-col gap-3"
      >
        <div class="flex items-center justify-between gap-3">
          <h2 class="text-sm font-semibold">
            {{ t('网关指标', 'Gateway metrics') }}
          </h2>
          <Badge
            variant="outline"
            class="text-[10px]"
          >
            {{ t('实时', 'Live') }}
          </Badge>
        </div>
        <div class="grid flex-1 grid-cols-2 gap-3 sm:grid-cols-3 sm:gap-4">
          <OperationsMetricCard
            v-for="item in coreMetrics"
            :key="item.label"
            v-bind="item"
          />
        </div>
      </section>
    </div>
  </section>
  <slot :snapshot="snapshot" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Activity, CheckCircle2, Cpu, Gauge, Hash, MemoryStick, Radio, TriangleAlert } from 'lucide-vue-next'
import { overviewApi } from '@/api/overview'
import { buildGatewayMetricsSummary } from '@/api/monitoring'
import { Badge } from '@/components/ui'
import OverviewStatus from '../components/OverviewStatus.vue'
import OperationsMetricCard from './OperationsMetricCard.vue'
import { useOverviewRequest } from '../useOverviewRequest'
import { useOverviewI18n } from '../i18n'
import { count, percent } from '../format'

const props = defineProps<{ revision: number }>()
const { t } = useOverviewI18n()
const { data, loading, error, refresh } = useOverviewRequest(() => props.revision, signal => overviewApi.live(signal), { scopeKey: () => 'live' })
const runtime = computed(() => {
  const snapshot = data.value?.data
  return snapshot?.metrics || (snapshot?.metrics_text ? buildGatewayMetricsSummary(snapshot.metrics_text) : null)
})
const snapshot = computed(() => data.value ? { ...data.value.data, metrics: runtime.value } : undefined)
function memory(value: number | null | undefined) {
  if (value == null) return '-'
  return value >= 1024 ** 3 ? `${count(value / 1024 ** 3)} GiB` : `${count(value / 1024 ** 2)} MiB`
}
function basis(value: number | null | undefined) {
  return value == null ? '-' : `${count(value / 100)}%`
}
const hardwareMetrics = computed(() => {
  const process = runtime.value?.process
  const memoryNote = process?.systemMemoryUsedBytes != null && process.systemMemoryTotalBytes != null && process.systemMemoryTotalBytes > 0
    ? `${memory(process.systemMemoryUsedBytes)} / ${memory(process.systemMemoryTotalBytes)}`
    : t('内存使用率', 'Memory usage')
  return [
    { label: t('主机 CPU', 'Host CPU'), value: basis(process?.systemCpuUsageBasisPoints), icon: Cpu, note: t('CPU 使用率', 'CPU usage') },
    { label: t('主机内存', 'Host memory'), value: process?.systemMemoryTotalBytes === 0 ? '-' : basis(process?.systemMemoryUsageBasisPoints), icon: MemoryStick, note: memoryNote },
    { label: t('进程 CPU', 'Process CPU'), value: basis(process?.processCpuUsageBasisPoints), icon: Cpu, note: t('100% = 1 核', '100% = 1 core') },
    { label: t('进程内存', 'Process memory'), value: memory(process?.processMemoryBytes), icon: MemoryStick, note: t('网关进程', 'Gateway process') },
  ]
})
const coreMetrics = computed(() => {
  const metrics = data.value?.data.recent_activity?.data
  const recentNote = t('最近 60 秒', 'Last 60 seconds')
  const nodeNote = t('当前节点', 'Current node')
  return [
    { label: 'RPM', value: count(metrics?.requests_per_minute), icon: Activity, note: recentNote },
    { label: 'TPM', value: count(metrics?.tokens_per_minute), icon: Hash, note: recentNote },
    { label: t('请求成功率', 'Success rate'), value: percent(metrics?.success_rate?.value), icon: CheckCircle2, note: recentNote },
    { label: t('失败请求', 'Failed requests'), value: count(metrics?.failed_request_count), icon: TriangleAlert, note: recentNote },
    { label: t('当前并发', 'Concurrent requests'), value: count(runtime.value?.local.inFlight), icon: Gauge, note: nodeNote },
    { label: t('活跃流', 'Active streams'), value: count(runtime.value?.tunnel.activeStreams), icon: Radio, note: nodeNote },
  ]
})
</script>
