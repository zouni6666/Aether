<template>
  <main class="space-y-5 px-4 pb-8 sm:px-6 lg:px-0">
    <OverviewToolbar
      :title="t('运维总览', 'Operations')"
      :range="range"
      :preset="relativePreset"
      :refresh-active="autoRefresh"
      :refresh-title="refreshTitle"
      :show-range="false"
      presets-only
      @update:range="setRange"
      @refresh="toggleAutoRefresh"
    />
    <RuntimeView :revision="revision">
      <template #default="{ snapshot }">
        <PerformanceView
          :revision="revision"
          :activity="snapshot?.execution_activity"
        >
          <template #diagnostics>
            <LiveMetrics
              v-if="snapshot"
              :snapshot="snapshot"
            />
            <LiveMetrics
              v-if="snapshot"
              :snapshot="snapshot"
              resources
            />
          </template>
        </PerformanceView>
      </template>
    </RuntimeView>
  </main>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref } from 'vue'
import OverviewToolbar from '@/features/overview/components/OverviewToolbar.vue'
import RuntimeView from '@/features/overview/operations/RuntimeView.vue'
import PerformanceView from '@/features/overview/operations/PerformanceView.vue'
import LiveMetrics from '@/features/overview/operations/LiveMetrics.vue'
import { useOverviewQuery } from '@/features/overview/query'
import { useOverviewI18n } from '@/features/overview/i18n'
const { t } = useOverviewI18n()
const { range, relativePreset, refreshRange, setRange } = useOverviewQuery('today', { rolling: true })
const revision = ref(0)
const autoRefresh = ref(false)
const refreshTitle = computed(() => autoRefresh.value
  ? t('关闭自动刷新（每 10 秒）', 'Stop auto refresh (every 10 seconds)')
  : t('开启自动刷新（每 10 秒）', 'Start auto refresh (every 10 seconds)'))
let timer: ReturnType<typeof setInterval> | undefined
function refreshAll() { refreshRange(); revision.value += 1 }
function stopAutoRefresh() {
  autoRefresh.value = false
  if (timer !== undefined) clearInterval(timer)
  timer = undefined
}
function toggleAutoRefresh() {
  if (autoRefresh.value) {
    stopAutoRefresh()
    return
  }
  autoRefresh.value = true
  refreshAll()
  timer = setInterval(() => {
    if (document.visibilityState === 'visible') refreshAll()
  }, 10_000)
}
onUnmounted(stopAutoRefresh)
</script>
