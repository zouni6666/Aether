<template>
  <UsageStatsTable
    :title="t('ui.byProviderAnalysis')"
    :name-header="t('nav.providers')"
    :empty-text="t('ui.noProviderStatistics')"
    :rows="rows"
    :extra-columns="PROVIDER_STATS_COLUMNS"
    :is-admin="isAdmin"
  />
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const props = defineProps<{
  data: ProviderStatsItem[]
  isAdmin: boolean
}>()

const { t } = useI18n()

import { computed } from 'vue'
import UsageStatsTable from './UsageStatsTable.vue'
import { PROVIDER_STATS_COLUMNS, providerStatsRows } from '../utils/usageStatsRows'
import type { ProviderStatsItem } from '../types'

const rows = computed(() => providerStatsRows(props.data))

</script>
