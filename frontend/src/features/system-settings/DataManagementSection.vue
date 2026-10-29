<template>
  <CardSection
    :title="t('ui.dataManagement')"
    :description="t('ui.exportImportOrClearSystemData')"
  >
    <div class="space-y-6">
      <div>
        <div class="flex items-center gap-2 mb-3">
          <Database class="w-4 h-4 text-muted-foreground" />
          <h4 class="text-sm font-medium">
            {{ t('ui.exportImport') }}
          </h4>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
          <div
            v-for="item in dataItems"
            :key="item.key"
            class="flex flex-col gap-3 p-4 rounded-lg border border-border"
          >
            <div class="flex items-center gap-2">
              <component
                :is="item.icon"
                class="w-4 h-4 text-muted-foreground"
              />
              <span class="text-sm font-medium">{{ item.title }}</span>
            </div>
            <p class="text-xs text-muted-foreground flex-1">
              {{ item.description }}
            </p>
            <div class="grid grid-cols-2 gap-2">
              <Button
                variant="outline"
                size="sm"
                class="w-full"
                :title="item.exportLabel"
                :disabled="item.exportLoading"
                @click="$emit('export', item.key)"
              >
                <Download class="w-3.5 h-3.5 mr-1.5" />
                {{ item.exportLoading ? t('ui.exporting') : t('ui.export') }}
              </Button>
              <Button
                variant="outline"
                size="sm"
                class="w-full"
                :title="item.importLabel"
                :disabled="item.importLoading"
                @click="triggerDataFileSelect(item.key)"
              >
                <Upload class="w-3.5 h-3.5 mr-1.5" />
                {{ item.importLoading ? t('ui.importingc70b341') : t('ui.import') }}
              </Button>
            </div>
          </div>
        </div>

        <input
          ref="configFileInput"
          type="file"
          accept=".json"
          class="hidden"
          @change="$emit('fileSelect', 'config', $event)"
        >
        <input
          ref="usersFileInput"
          type="file"
          accept=".json"
          class="hidden"
          @change="$emit('fileSelect', 'users', $event)"
        >
        <input
          ref="aggregateFileInput"
          type="file"
          accept=".json"
          class="hidden"
          @change="$emit('fileSelect', 'aggregate', $event)"
        >
      </div>

      <Separator />

      <div>
        <div class="flex items-center gap-2 mb-3">
          <Trash2 class="w-4 h-4 text-muted-foreground" />
          <h4 class="text-sm font-medium">
            {{ t('ui.clearData') }}
          </h4>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          <div
            v-for="item in purgeItems"
            :key="item.key"
            class="flex flex-col gap-2 p-4 rounded-lg border border-border"
          >
            <div class="flex items-center gap-2">
              <component
                :is="item.icon"
                class="w-4 h-4 text-muted-foreground"
              />
              <span class="text-sm font-medium">{{ item.title }}</span>
            </div>
            <p class="text-xs text-muted-foreground flex-1">
              {{ item.description }}
            </p>
            <Button
              variant="destructive"
              size="sm"
              class="w-full mt-1"
              :disabled="loadingKey === item.key"
              @click="handlePurge(item)"
            >
              <Trash2 class="w-3.5 h-3.5 mr-1.5" />
              {{ loadingKey === item.key ? t('ui.clearing') : item.buttonText }}
            </Button>
          </div>
        </div>
      </div>
    </div>
  </CardSection>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const props = defineProps<{
  configExportLoading: boolean
  configImportLoading: boolean
  usersExportLoading: boolean
  usersImportLoading: boolean
  aggregateExportLoading: boolean
  aggregateImportLoading: boolean
}>()

defineEmits<{
  export: [key: DataItemKey]
  fileSelect: [key: DataItemKey, event: Event]
}>()

const { t } = useI18n()

import { computed, ref, markRaw, type Component } from 'vue'
import {
  Download,
  Upload,
  Settings,
  Users,
  Database,
  Layers3,
  Trash2,
  BarChart3,
  Shield,
  FileText,
  PieChart,
} from 'lucide-vue-next'
import Button from '@/components/ui/button.vue'
import { Separator } from '@/components/ui'
import { CardSection } from '@/components/layout'
import { adminApi } from '@/api/admin'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { parseApiError } from '@/utils/errorParser'

type DataItemKey = 'config' | 'users' | 'aggregate'

interface DataItem {
  key: DataItemKey
  title: string
  description: string
  exportLabel: string
  importLabel: string
  icon: Component
  exportLoading: boolean
  importLoading: boolean
}

interface PurgeItem {
  key: string
  title: string
  description: string
  buttonText: string
  icon: Component
  confirmMessage: string
  action: () => Promise<{ message: string }>
}

const { success, error } = useToast()
const { confirmDanger } = useConfirm()
const loadingKey = ref<string | null>(null)
const configFileInput = ref<HTMLInputElement | null>(null)
const usersFileInput = ref<HTMLInputElement | null>(null)
const aggregateFileInput = ref<HTMLInputElement | null>(null)

const dataItems = computed<DataItem[]>(() => [
  {
    key: 'config',
    get title() { return t('ui.configurationData') },
    get description() { return t('ui.providersEndpointsApiKeysModelsAndSystemConfiguration') },
    get exportLabel() { return t('ui.exportConfiguration') },
    get importLabel() { return t('ui.importConfiguration') },
    icon: markRaw(Settings),
    exportLoading: props.configExportLoading,
    importLoading: props.configImportLoading,
  },
  {
    key: 'users',
    get title() { return t('ui.userData') },
    get description() { return t('ui.regularUsersUserGroupsApiKeysAndWalletSnapshotsExcluding') },
    get exportLabel() { return t('ui.exportUsers') },
    get importLabel() { return t('ui.importUsers') },
    icon: markRaw(Users),
    exportLoading: props.usersExportLoading,
    importLoading: props.usersImportLoading,
  },
  {
    key: 'aggregate',
    get title() { return t('ui.fullBackup') },
    get description() { return t('ui.integratedBackupOfConfigurationAndUserDataIncludingUsersUser') },
    get exportLabel() { return t('ui.exportBackup') },
    get importLabel() { return t('ui.importBackup') },
    icon: markRaw(Layers3),
    exportLoading: props.aggregateExportLoading,
    importLoading: props.aggregateImportLoading,
  },
])

const purgeItems: PurgeItem[] = [
  {
    key: 'config',
    get title() { return t('ui.clearConfiguration') },
    get description() { return t('ui.deleteAllProvidersEndpointsApiKeysAndModelConfigurationIn') },
    get buttonText() { return t('ui.clearConfiguration') },
    icon: markRaw(Settings),
    get confirmMessage() { return t('ui.clearAllProviderConfigurationInTheBackgroundAllProvidersEndpoints') },
    action: () => adminApi.purgeConfig(),
  },
  {
    key: 'users',
    get title() { return t('ui.clearUsers') },
    get description() { return t('ui.deleteAllNonAdminUsersAndTheirApiKeysIn') },
    get buttonText() { return t('ui.clearUsers') },
    icon: markRaw(Users),
    get confirmMessage() { return t('ui.permanentlyDeleteAllNonAdministratorUsersInTheBackgroundAdministrator') },
    action: () => adminApi.purgeUsers(),
  },
  {
    key: 'usage',
    get title() { return t('ui.clearUsageRecords') },
    get description() { return t('ui.clearAllUsageRecordsAndRequestCandidateRecordsInThe') },
    get buttonText() { return t('ui.clearRecords') },
    icon: markRaw(BarChart3),
    get confirmMessage() { return t('ui.clearAllUsageRecordsInTheBackgroundAllRequestStatistics') },
    action: () => adminApi.purgeUsage(),
  },
  {
    key: 'audit-logs',
    get title() { return t('ui.clearAuditLogs') },
    get description() { return t('ui.deleteAllAuditLogRecordsInTheBackground') },
    get buttonText() { return t('ui.clearLogs') },
    icon: markRaw(Shield),
    get confirmMessage() { return t('ui.clearAllAuditLogsInTheBackgroundAllSecurityEvent') },
    action: () => adminApi.purgeAuditLogs(),
  },
  {
    key: 'request-bodies',
    get title() { return t('ui.clearRequestBodies') },
    get description() { return t('ui.clearAllRequestResponseBodyDataInBatchesWhileKeeping') },
    get buttonText() { return t('ui.clearRequestBodies') },
    icon: markRaw(FileText),
    get confirmMessage() { return t('ui.permanentlyClearAllRequestAndResponseBodiesInTheBackground') },
    action: () => adminApi.purgeRequestBodiesAsync(),
  },
  {
    key: 'stats',
    get title() { return t('ui.clearAggregatedStatistics') },
    get description() { return t('ui.deleteAndRebuildStatisticsAggregatesWhileKeepingRawUsageRecords') },
    get buttonText() { return t('ui.clearAggregatedStatistics') },
    icon: markRaw(PieChart),
    get confirmMessage() { return t('ui.clearAllAggregatedStatisticsInTheBackgroundOriginalUsageRecords') },
    action: () => adminApi.purgeStats(),
  },
]

function triggerDataFileSelect(key: DataItemKey) {
  if (key === 'config') {
    configFileInput.value?.click()
  } else if (key === 'users') {
    usersFileInput.value?.click()
  } else {
    aggregateFileInput.value?.click()
  }
}

async function handlePurge(item: PurgeItem) {
  const confirmed = await confirmDanger(item.confirmMessage, item.title)
  if (!confirmed) return

  loadingKey.value = item.key
  try {
    const result = await item.action()
    success(result.message || t('ui.operationCompleted'))
  } catch (e) {
    error(parseApiError(e, t('ui.failedToClearData')))
  } finally {
    loadingKey.value = null
  }
}

</script>
