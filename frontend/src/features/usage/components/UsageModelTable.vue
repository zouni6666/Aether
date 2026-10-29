<template>
  <UsageStatsTable
    :title="t('ui.byModelAnalysis')"
    :name-header="t('ui.model')"
    :empty-text="t('ui.noModelStatistics')"
    :rows="rows"
    :extra-columns="MODEL_STATS_COLUMNS"
    :is-admin="isAdmin"
  />
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const props = defineProps<{
  data: EnhancedModelStatsItem[]
  isAdmin: boolean
}>()

const { t } = useI18n()

import { computed } from 'vue'
import UsageStatsTable from './UsageStatsTable.vue'
import { MODEL_STATS_COLUMNS, modelStatsRows } from '../utils/usageStatsRows'
import type { EnhancedModelStatsItem } from '../types'

const rows = computed(() => modelStatsRows(props.data))

</script>
