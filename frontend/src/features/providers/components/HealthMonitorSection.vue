<template>
  <Card
    variant="default"
    class="overflow-hidden"
  >
    <HealthMonitorHeader
      v-model:lookback-hours="lookbackHours"
      :title="title"
      :description="description"
      :loading="loading"
      @refresh="refreshData"
    />

    <div class="p-6">
      <div
        v-if="loadingMonitors"
        class="flex items-center justify-center py-12"
      >
        <Loader2 class="w-6 h-6 animate-spin text-muted-foreground" />
        <span class="ml-2 text-muted-foreground">{{ t('common.loading') }}</span>
      </div>

      <div
        v-else-if="items.length === 0"
        class="flex flex-col items-center justify-center py-12 text-muted-foreground"
      >
        <component
          :is="icon"
          class="w-12 h-12 mb-3 opacity-30"
        />
        <p>{{ emptyTitle }}</p>
        <p class="text-xs mt-1">
          {{ emptyDescription }}
        </p>
      </div>

      <div
        v-else
        class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4"
      >
        <HealthMonitorItemCard
          v-for="item in items"
          :key="item.key"
          :icon="icon"
          :title="item.title"
          :meta-text="item.metaText"
          :monitor="item.monitor"
          :lookback-hours="lookbackHoursNumber"
          :generated-at="generatedAt"
          :entity-label="entityLabel"
          :entity-name="item.entityName"
          @view-details="openDetails(item)"
        >
          <template
            v-if="$slots.timeline"
            #timeline
          >
            <slot
              name="timeline"
              :item="item"
              :lookback-hours="lookbackHoursNumber"
            />
          </template>
        </HealthMonitorItemCard>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts" generic="T">
import { useI18n } from '@/i18n'

import { computed, onMounted, ref, shallowRef, watch, type Component } from 'vue'
import { Loader2 } from 'lucide-vue-next'
import Card from '@/components/ui/card.vue'
import HealthMonitorHeader from './HealthMonitorHeader.vue'
import HealthMonitorItemCard from './HealthMonitorItemCard.vue'
import { useToast } from '@/composables/useToast'
import { parseApiError } from '@/utils/errorParser'
import {
  summarizeHealthMonitorItems,
  type HealthMonitorDetailTarget,
  type HealthMonitorSectionItem,
  type HealthMonitorSectionData,
  type HealthMonitorSectionSummary,
} from './health-monitor-utils'

/**
 * 健康监控分区：负责回溯时间选择、加载态、空态与卡片网格。
 * 各维度（端点 / 模型 / 提供商）只需提供 load 函数把接口数据整理成 items。
 */
const props = defineProps<{
  title: string
  description: string
  icon: Component
  emptyTitle: string
  emptyDescription: string
  entityLabel: string
  errorMessage: string
  load: (lookbackHours: number) => Promise<HealthMonitorSectionData<T>>
}>()

const emit = defineEmits<{
  viewDetails: [target: HealthMonitorDetailTarget]
  summaryUpdated: [summary: HealthMonitorSectionSummary]
}>()

const { t } = useI18n()

const { error: showError } = useToast()

const loading = ref(false)
const loadingMonitors = ref(false)
const items = shallowRef<HealthMonitorSectionItem<T>[]>([])
const generatedAt = ref<string | null>(null)
const lookbackHours = ref('6')
const lookbackHoursNumber = computed(() => parseInt(lookbackHours.value))

async function loadMonitors() {
  loadingMonitors.value = true
  try {
    const data = await props.load(lookbackHoursNumber.value)
    items.value = data.items
    generatedAt.value = data.generatedAt ?? null
    emit('summaryUpdated', summarizeHealthMonitorItems(items.value.map(item => item.monitor)))
  } catch (err: unknown) {
    showError(parseApiError(err, props.errorMessage), t('ui.error'))
  } finally {
    loadingMonitors.value = false
  }
}

async function refreshData() {
  loading.value = true
  try {
    await loadMonitors()
  } finally {
    loading.value = false
  }
}

function openDetails(item: HealthMonitorSectionItem<T>) {
  emit('viewDetails', {
    lookbackHours: lookbackHoursNumber.value,
    source: item.detail,
  })
}

watch(lookbackHours, () => {
  loadMonitors()
})

onMounted(() => {
  refreshData()
})

</script>
