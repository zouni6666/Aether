<template>
  <UsageStatsTable
    :title="t('ui.byApiFormatAnalysis')"
    :name-header="t('ui.apiFormat55eba68')"
    :empty-text="t('ui.noApiFormatStatistics')"
    :rows="rows"
    :extra-columns="API_FORMAT_STATS_COLUMNS"
    :is-admin="isAdmin"
  />
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const props = defineProps<{
  data: ApiFormatStatsItem[]
  isAdmin: boolean
}>()

const { t } = useI18n()

import { computed } from 'vue'
import UsageStatsTable from './UsageStatsTable.vue'
import { API_FORMAT_STATS_COLUMNS, apiFormatStatsRows } from '../utils/usageStatsRows'
import type { ApiFormatStatsItem } from '../types'

const rows = computed(() => apiFormatStatsRows(props.data))

</script>
