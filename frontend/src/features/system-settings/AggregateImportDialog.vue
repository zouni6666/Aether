<template>
  <!-- 完整备份导入对话框 -->
  <Dialog
    :open="aggregateImportDialogOpen"
    :title="t('ui.importFullBackup')"
    :description="t('ui.chooseHowToHandleConflictsAndConfirmTheImport')"
    @update:open="$emit('update:aggregateImportDialogOpen', $event)"
  >
    <div class="space-y-4">
      <div
        v-if="aggregateImportPreview"
        class="text-sm"
      >
        <p class="font-medium mb-2">
          {{ t('ui.fullBackupPreview') }}
        </p>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-muted-foreground">
          <div>
            <p class="font-medium text-foreground mb-1">
              {{ t('ui.configurationData') }}
            </p>
            <ul class="space-y-1">
              <li>{{ t('ui.globalModels') }} {{ aggregateImportPreview.config_data.global_models?.length || 0 }} {{ t('ui.items3a11681') }}</li>
              <li>{{ t('ui.providers') }} {{ aggregateImportPreview.config_data.providers?.length || 0 }} {{ t('ui.items3a11681') }}</li>
              <li>
                API Keys: {{ aggregateImportPreview.config_data.providers?.reduce((sum: number, p: { api_keys?: unknown[] }) => sum + (p.api_keys?.length || 0), 0) }} {{ t('ui.items3a11681') }}
              </li>
            </ul>
          </div>
          <div>
            <p class="font-medium text-foreground mb-1">
              {{ t('ui.userData') }}
            </p>
            <ul class="space-y-1">
              <li v-if="aggregateImportPreview.user_data.user_groups?.length">
                {{ t('ui.userGroups') }} {{ aggregateImportPreview.user_data.user_groups.length }} {{ t('ui.items3a11681') }}
              </li>
              <li>{{ t('ui.users') }} {{ aggregateImportPreview.user_data.users?.length || 0 }} {{ t('ui.items3a11681') }}</li>
              <li>
                API Keys: {{ aggregateImportPreview.user_data.users?.reduce((sum: number, u: { api_keys?: unknown[] }) => sum + (u.api_keys?.length || 0), 0) }} {{ t('ui.items3a11681') }}
              </li>
              <li v-if="aggregateImportPreview.user_data.standalone_keys?.length">
                {{ t('ui.standaloneBalanceKeys') }} {{ aggregateImportPreview.user_data.standalone_keys.length }} {{ t('ui.items3a11681') }}
              </li>
              <li v-if="usageAggregatePreviewCounts.total > 0">
                {{ t('ui.aggregatedStatistics') }} {{ usageAggregatePreviewCounts.total }} {{ t('ui.rows') }}
              </li>
            </ul>
          </div>
        </div>
      </div>

      <div>
        <Label class="block text-sm font-medium mb-2">{{ t('ui.conflictHandling') }}</Label>
        <Select
          :model-value="aggregateMergeMode"
          :open="aggregateMergeModeSelectOpen"
          @update:model-value="($event === 'skip' || $event === 'overwrite' || $event === 'error') && $emit('update:aggregateMergeMode', $event)"
          @update:open="$emit('update:aggregateMergeModeSelectOpen', $event)"
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="skip">
              {{ t('ui.skipAndKeepExistingData') }}
            </SelectItem>
            <SelectItem value="overwrite">
              {{ t('ui.overwriteWithImportedData') }}
            </SelectItem>
            <SelectItem value="error">
              {{ t('ui.abortIfAConflictIsFound') }}
            </SelectItem>
          </SelectContent>
        </Select>
        <p class="mt-1 text-xs text-muted-foreground">
          <template v-if="aggregateMergeMode === 'skip'">
            {{ t('ui.keepExistingDataAndImportOnlyNewEntries') }}
          </template>
          <template v-else-if="aggregateMergeMode === 'overwrite'">
            {{ t('ui.replaceExistingDataWithImportedData') }}
          </template>
          <template v-else>
            {{ t('ui.stopTheImportIfAConflictIsFound') }}
          </template>
        </p>
      </div>

      <p class="text-xs text-muted-foreground">
        {{ t('ui.configurationIsImportedBeforeUserDataPasswordsAndUserOr') }}
      </p>

      <div
        v-if="importAggregateProgress"
        class="space-y-2 rounded-md border border-border p-3"
      >
        <div class="flex items-center justify-between gap-3 text-xs text-muted-foreground">
          <span>{{ importAggregateProgress.message }}</span>
          <span>{{ importAggregateProgress.percent }}%</span>
        </div>
        <div class="h-1.5 overflow-hidden rounded-full bg-muted">
          <div
            class="h-full bg-primary transition-all"
            :style="{ width: `${importAggregateProgress.percent}%` }"
          />
        </div>
      </div>
    </div>

    <template #footer>
      <Button
        variant="outline"
        @click="$emit('update:aggregateImportDialogOpen', false); $emit('update:aggregateMergeModeSelectOpen', false)"
      >
        {{ t('common.cancel') }}
      </Button>
      <Button
        :disabled="importAggregateLoading"
        @click="$emit('confirm')"
      >
        {{ importAggregateLoading ? t('ui.importingc70b341') : t('ui.confirmImport') }}
      </Button>
    </template>
  </Dialog>

  <!-- 完整备份导入结果对话框 -->
  <Dialog
    :open="aggregateImportResultDialogOpen"
    :title="t('ui.fullBackupImported')"
    @update:open="$emit('update:aggregateImportResultDialogOpen', $event)"
  >
    <div
      v-if="aggregateImportResult"
      class="space-y-4"
    >
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
        <div>
          <p class="font-medium">
            {{ t('ui.configurationData') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.globalModelsCreated') }} {{ aggregateImportResult.config.stats.global_models.created }}{{ t('ui.providersCreated') }} {{ aggregateImportResult.config.stats.providers.created }}{{ t('ui.apiKeysCreated') }} {{ aggregateImportResult.config.stats.keys.created }}
          </p>
        </div>
        <div>
          <p class="font-medium">
            {{ t('ui.userData') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.usersCreated') }} {{ aggregateImportResult.users.stats.users.created }}{{ t('ui.apiKeysCreated') }} {{ aggregateImportResult.users.stats.api_keys.created }}{{ t('ui.skippedd5f7caa') }} {{ aggregateImportResult.users.stats.users.skipped }} {{ t('ui.users459fab8') }} <template v-if="usageAggregateResultText">
              {{ t('ui.aggregatedStatisticsf4c3024') }} {{ usageAggregateResultText }}
            </template>
          </p>
        </div>
      </div>

      <div
        v-if="warningMessages.length > 0"
        class="p-3 bg-destructive/10 rounded-lg"
      >
        <p class="font-medium text-destructive mb-2">
          {{ t('ui.warnings') }}
        </p>
        <ul class="text-sm text-destructive space-y-1">
          <li
            v-for="(message, index) in warningMessages"
            :key="index"
          >
            {{ message }}
          </li>
        </ul>
      </div>
    </div>

    <template #footer>
      <Button @click="$emit('update:aggregateImportResultDialogOpen', false)">
        {{ t('ui.confirmfac2a67') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const props = defineProps<{
  aggregateImportDialogOpen: boolean
  aggregateImportResultDialogOpen: boolean
  aggregateImportPreview: AggregateExportData | null
  aggregateImportResult: AggregateImportResponse | null
  aggregateMergeMode: 'skip' | 'overwrite' | 'error'
  aggregateMergeModeSelectOpen: boolean
  importAggregateLoading: boolean
  importAggregateProgress: ImportProgressState | null
}>()

defineEmits<{
  confirm: []
  'update:aggregateImportDialogOpen': [value: boolean]
  'update:aggregateImportResultDialogOpen': [value: boolean]
  'update:aggregateMergeMode': [value: 'skip' | 'overwrite' | 'error']
  'update:aggregateMergeModeSelectOpen': [value: boolean]
}>()

const { t } = useI18n()

import { computed } from 'vue'
import Button from '@/components/ui/button.vue'
import Label from '@/components/ui/label.vue'
import Select from '@/components/ui/select.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectValue from '@/components/ui/select-value.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import { Dialog } from '@/components/ui'
import type { AggregateExportData, AggregateImportResponse } from '@/api/admin'
import type { ImportProgressState } from './composables/useConfigExportImport'

const warningMessages = computed(() => {
  if (!props.aggregateImportResult) return []
  const configErrors = props.aggregateImportResult.config.stats.errors.map((message) => t('settings.import.configurationError', { message }))
  const userErrors = props.aggregateImportResult.users.stats.errors.map((message) => t('settings.import.userError', { message }))
  return [...configErrors, ...userErrors]
})

const usageAggregatePreviewCounts = computed(() => {
  const aggregates = props.aggregateImportPreview?.user_data.usage_aggregates
  const statsDaily = aggregates?.stats_daily?.length ?? 0
  const statsUserDaily = aggregates?.stats_user_daily?.length ?? 0
  const statsDailyApiKey = aggregates?.stats_daily_api_key?.length ?? 0

  return {
    statsDaily,
    statsUserDaily,
    statsDailyApiKey,
    total: statsDaily + statsUserDaily + statsDailyApiKey,
  }
})

const usageAggregateResultText = computed(() => {
  const aggregates = props.aggregateImportResult?.users.stats.usage_aggregates
  if (!aggregates) return ''

  const counters = [
    aggregates.stats_daily,
    aggregates.stats_user_daily,
    aggregates.stats_daily_api_key,
  ]
  const created = counters.reduce((sum, item) => sum + item.created, 0)
  const updated = counters.reduce((sum, item) => sum + item.updated, 0)
  const skipped = counters.reduce((sum, item) => sum + item.skipped, 0)
  const total = created + updated + skipped
  if (total === 0) return ''

  return t('settings.import.counts', { created, updated, skipped })
})

</script>
