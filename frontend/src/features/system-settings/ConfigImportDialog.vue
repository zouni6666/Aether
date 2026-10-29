<template>
  <!-- 导入配置对话框 -->
  <Dialog
    :open="importDialogOpen"
    :title="t('ui.importConfiguration')"
    :description="t('ui.chooseHowToHandleConflictsAndConfirmTheImport')"
    @update:open="$emit('update:importDialogOpen', $event)"
  >
    <div class="space-y-4">
      <div
        v-if="importPreview"
        class="text-sm"
      >
        <p class="font-medium mb-2">
          {{ t('ui.configurationPreview') }}
        </p>
        <ul class="space-y-1 text-muted-foreground">
          <li>{{ t('ui.globalModels') }} {{ importPreview.global_models?.length || 0 }} {{ t('ui.items3a11681') }}</li>
          <li>{{ t('ui.providers') }} {{ importPreview.providers?.length || 0 }} {{ t('ui.items3a11681') }}</li>
          <li> {{ t('ui.endpoints') }} {{ importPreview.providers?.reduce((sum: number, p: { endpoints?: unknown[] }) => sum + (p.endpoints?.length || 0), 0) }} {{ t('ui.items3a11681') }} </li>
          <li>
            API Keys: {{ importPreview.providers?.reduce((sum: number, p: { api_keys?: unknown[] }) => sum + (p.api_keys?.length || 0), 0) }} {{ t('ui.items3a11681') }}
          </li>
          <li v-if="importPreview.proxy_nodes?.length">
            {{ t('ui.proxyNodes') }} {{ importPreview.proxy_nodes.length }} {{ t('ui.items3a11681') }}
          </li>
          <li v-if="importPreview.ldap_config">
            {{ t('ui.ldapConfigurations1') }}
          </li>
          <li v-if="importPreview.oauth_providers?.length">
            OAuth Providers: {{ importPreview.oauth_providers.length }} {{ t('ui.items3a11681') }}
          </li>
        </ul>
      </div>

      <div>
        <Label class="block text-sm font-medium mb-2">{{ t('ui.conflictHandling') }}</Label>
        <Select
          :model-value="mergeMode"
          :open="mergeModeSelectOpen"
          @update:model-value="($event === 'skip' || $event === 'overwrite' || $event === 'error') && $emit('update:mergeMode', $event)"
          @update:open="$emit('update:mergeModeSelectOpen', $event)"
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="skip">
              {{ t('ui.skipAndKeepExistingConfiguration') }}
            </SelectItem>
            <SelectItem value="overwrite">
              {{ t('ui.overwriteWithImportedConfiguration') }}
            </SelectItem>
            <SelectItem value="error">
              {{ t('ui.abortIfAConflictIsFound') }}
            </SelectItem>
          </SelectContent>
        </Select>
        <p class="mt-1 text-xs text-muted-foreground">
          <template v-if="mergeMode === 'skip'">
            {{ t('ui.keepExistingConfigurationAndImportOnlyNewEntries') }}
          </template>
          <template v-else-if="mergeMode === 'overwrite'">
            {{ t('ui.replaceExistingConfigurationWithImportedConfiguration') }}
          </template>
          <template v-else>
            {{ t('ui.conflictsAreCheckedBeforeAnyDataIsWrittenTheImport') }}
          </template>
        </p>
      </div>

      <p class="text-xs text-muted-foreground">
        {{ t('ui.duplicateApiKeysAreSkippedAutomaticallyNoDuplicateRecordsAre') }}
      </p>

      <div
        v-if="importProgress"
        class="space-y-2 rounded-md border border-border p-3"
      >
        <div class="flex items-center justify-between gap-3 text-xs text-muted-foreground">
          <span>{{ importProgress.message }}</span>
          <span>{{ importProgress.percent }}%</span>
        </div>
        <div class="h-1.5 overflow-hidden rounded-full bg-muted">
          <div
            class="h-full bg-primary transition-all"
            :style="{ width: `${importProgress.percent}%` }"
          />
        </div>
      </div>
    </div>

    <template #footer>
      <Button
        variant="outline"
        @click="$emit('update:importDialogOpen', false); $emit('update:mergeModeSelectOpen', false)"
      >
        {{ t('common.cancel') }}
      </Button>
      <Button
        :disabled="importLoading"
        @click="$emit('confirm')"
      >
        {{ importLoading ? t('ui.importingc70b341') : t('ui.confirmImport') }}
      </Button>
    </template>
  </Dialog>

  <!-- 导入结果对话框 -->
  <Dialog
    :open="importResultDialogOpen"
    :title="t('ui.importCompleted613fef')"
    @update:open="$emit('update:importResultDialogOpen', $event)"
  >
    <div
      v-if="importResult"
      class="space-y-4"
    >
      <div class="grid grid-cols-2 gap-4 text-sm">
        <div>
          <p class="font-medium">
            {{ t('ui.globalModel') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.created64aefde') }} {{ importResult.stats.global_models.created }}{{ t('ui.updated') }} {{ importResult.stats.global_models.updated }}{{ t('ui.skipped') }} {{ importResult.stats.global_models.skipped }}
          </p>
        </div>
        <div>
          <p class="font-medium">
            {{ t('nav.providers') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.created64aefde') }} {{ importResult.stats.providers.created }}{{ t('ui.updated') }} {{ importResult.stats.providers.updated }}{{ t('ui.skipped') }} {{ importResult.stats.providers.skipped }}
          </p>
        </div>
        <div>
          <p class="font-medium">
            {{ t('ui.endpoint52fff16') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.created64aefde') }} {{ importResult.stats.endpoints.created }}{{ t('ui.updated') }} {{ importResult.stats.endpoints.updated }}{{ t('ui.skipped') }} {{ importResult.stats.endpoints.skipped }}
          </p>
        </div>
        <div>
          <p class="font-medium">
            API Keys
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.created64aefde') }} {{ importResult.stats.keys.created }}{{ t('ui.skipped') }} {{ importResult.stats.keys.skipped }}
          </p>
        </div>
        <div class="col-span-2">
          <p class="font-medium">
            {{ t('ui.modelConfiguration') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.created64aefde') }} {{ importResult.stats.models.created }}{{ t('ui.updated') }} {{ importResult.stats.models.updated }}{{ t('ui.skipped') }} {{ importResult.stats.models.skipped }}
          </p>
        </div>
        <div v-if="importResult.stats.ldap">
          <p class="font-medium">
            {{ t('ui.ldapConfiguration') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.created64aefde') }} {{ importResult.stats.ldap.created }}{{ t('ui.updated') }} {{ importResult.stats.ldap.updated }}{{ t('ui.skipped') }} {{ importResult.stats.ldap.skipped }}
          </p>
        </div>
        <div v-if="importResult.stats.oauth">
          <p class="font-medium">
            OAuth Providers
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.created64aefde') }} {{ importResult.stats.oauth.created }}{{ t('ui.updated') }} {{ importResult.stats.oauth.updated }}{{ t('ui.skipped') }} {{ importResult.stats.oauth.skipped }}
          </p>
        </div>
        <div v-if="importResult.stats.proxy_nodes">
          <p class="font-medium">
            {{ t('ui.proxyNodes23f1b6a') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.created64aefde') }} {{ importResult.stats.proxy_nodes.created }}{{ t('ui.updated') }} {{ importResult.stats.proxy_nodes.updated }}{{ t('ui.skipped') }} {{ importResult.stats.proxy_nodes.skipped }}
          </p>
        </div>
      </div>

      <div
        v-if="importResult.stats.errors.length > 0"
        class="p-3 bg-destructive/10 rounded-lg"
      >
        <p class="font-medium text-destructive mb-2">
          {{ t('ui.warnings') }}
        </p>
        <ul class="text-sm text-destructive space-y-1">
          <li
            v-for="(err, index) in importResult.stats.errors"
            :key="index"
          >
            {{ err }}
          </li>
        </ul>
      </div>
    </div>

    <template #footer>
      <Button @click="$emit('update:importResultDialogOpen', false)">
        {{ t('ui.confirmfac2a67') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import Button from '@/components/ui/button.vue'
import Label from '@/components/ui/label.vue'
import Select from '@/components/ui/select.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectValue from '@/components/ui/select-value.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import { Dialog } from '@/components/ui'
import type { ConfigExportData, ConfigImportResponse } from '@/api/admin'
import type { ImportProgressState } from './composables/useConfigExportImport'

defineProps<{
  importDialogOpen: boolean
  importResultDialogOpen: boolean
  importPreview: ConfigExportData | null
  importResult: ConfigImportResponse | null
  mergeMode: 'skip' | 'overwrite' | 'error'
  mergeModeSelectOpen: boolean
  importLoading: boolean
  importProgress: ImportProgressState | null
}>()

defineEmits<{
  confirm: []
  'update:importDialogOpen': [value: boolean]
  'update:importResultDialogOpen': [value: boolean]
  'update:mergeMode': [value: 'skip' | 'overwrite' | 'error']
  'update:mergeModeSelectOpen': [value: boolean]
}>()

const { t } = useI18n()

</script>
