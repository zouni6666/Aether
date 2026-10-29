<template>
  <!-- 用户数据导入对话框 -->
  <Dialog
    :open="importUsersDialogOpen"
    :title="t('ui.importUserData')"
    :description="t('ui.chooseHowToHandleConflictsAndConfirmTheImport')"
    @update:open="$emit('update:importUsersDialogOpen', $event)"
  >
    <div class="space-y-4">
      <div
        v-if="importUsersPreview"
        class="text-sm"
      >
        <p class="font-medium mb-2">
          {{ t('ui.dataPreview') }}
        </p>
        <ul class="space-y-1 text-muted-foreground">
          <li v-if="importUsersPreview.user_groups?.length">
            {{ t('ui.userGroups') }} {{ importUsersPreview.user_groups.length }} {{ t('ui.items3a11681') }}
          </li>
          <li>{{ t('ui.users') }} {{ importUsersPreview.users?.length || 0 }} {{ t('ui.items3a11681') }}</li>
          <li>
            API Keys: {{ importUsersPreview.users?.reduce((sum: number, u: { api_keys?: unknown[] }) => sum + (u.api_keys?.length || 0), 0) }} {{ t('ui.items3a11681') }}
          </li>
          <li v-if="importUsersPreview.standalone_keys?.length">
            {{ t('ui.standaloneBalanceKeys') }} {{ importUsersPreview.standalone_keys.length }} {{ t('ui.items3a11681') }}
          </li>
        </ul>
      </div>

      <div>
        <Label class="block text-sm font-medium mb-2">{{ t('ui.conflictHandling') }}</Label>
        <Select
          :model-value="usersMergeMode"
          :open="usersMergeModeSelectOpen"
          @update:model-value="($event === 'skip' || $event === 'overwrite' || $event === 'error') && $emit('update:usersMergeMode', $event)"
          @update:open="$emit('update:usersMergeModeSelectOpen', $event)"
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="skip">
              {{ t('ui.skipAndKeepExistingUsers') }}
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
          <template v-if="usersMergeMode === 'skip'">
            {{ t('ui.keepExistingUsersAndImportOnlyNewUsers') }}
          </template>
          <template v-else-if="usersMergeMode === 'overwrite'">
            {{ t('ui.replaceExistingUserDataWithImportedData') }}
          </template>
          <template v-else>
            {{ t('ui.conflictsAreCheckedBeforeAnyDataIsWrittenTheImport') }}
          </template>
        </p>
      </div>

      <p class="text-xs text-muted-foreground">
        {{ t('ui.backupsDoNotExportOrRestorePasswordsOrApiKey') }}
      </p>

      <div
        v-if="importUsersProgress"
        class="space-y-2 rounded-md border border-border p-3"
      >
        <div class="flex items-center justify-between gap-3 text-xs text-muted-foreground">
          <span>{{ importUsersProgress.message }}</span>
          <span>{{ importUsersProgress.percent }}%</span>
        </div>
        <div class="h-1.5 overflow-hidden rounded-full bg-muted">
          <div
            class="h-full bg-primary transition-all"
            :style="{ width: `${importUsersProgress.percent}%` }"
          />
        </div>
      </div>
    </div>

    <template #footer>
      <Button
        variant="outline"
        @click="$emit('update:importUsersDialogOpen', false); $emit('update:usersMergeModeSelectOpen', false)"
      >
        {{ t('common.cancel') }}
      </Button>
      <Button
        :disabled="importUsersLoading"
        @click="$emit('confirm')"
      >
        {{ importUsersLoading ? t('ui.importingc70b341') : t('ui.confirmImport') }}
      </Button>
    </template>
  </Dialog>

  <!-- 用户数据导入结果对话框 -->
  <Dialog
    :open="importUsersResultDialogOpen"
    :title="t('ui.userDataImported')"
    @update:open="$emit('update:importUsersResultDialogOpen', $event)"
  >
    <div
      v-if="importUsersResult"
      class="space-y-4"
    >
      <div class="grid grid-cols-2 gap-4 text-sm">
        <div v-if="importUsersResult.stats.user_groups">
          <p class="font-medium">
            {{ t('ui.userGroups963ead0') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.created64aefde') }} {{ importUsersResult.stats.user_groups.created }}{{ t('ui.updated') }} {{ importUsersResult.stats.user_groups.updated }}{{ t('ui.skipped') }} {{ importUsersResult.stats.user_groups.skipped }}
          </p>
        </div>
        <div>
          <p class="font-medium">
            {{ t('auth.role.user') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.created64aefde') }} {{ importUsersResult.stats.users.created }}{{ t('ui.updated') }} {{ importUsersResult.stats.users.updated }}{{ t('ui.skipped') }} {{ importUsersResult.stats.users.skipped }}
          </p>
        </div>
        <div>
          <p class="font-medium">
            API Keys
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.created64aefde') }} {{ importUsersResult.stats.api_keys.created }}{{ t('ui.skipped') }} {{ importUsersResult.stats.api_keys.skipped }}
          </p>
        </div>
        <div
          v-if="importUsersResult.stats.standalone_keys"
          class="col-span-2"
        >
          <p class="font-medium">
            {{ t('ui.standaloneBalanceKeysa08a36d') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('ui.created64aefde') }} {{ importUsersResult.stats.standalone_keys.created }}{{ t('ui.skipped') }} {{ importUsersResult.stats.standalone_keys.skipped }}
          </p>
        </div>
      </div>

      <div
        v-if="importUsersResult.stats.errors.length > 0"
        class="p-3 bg-destructive/10 rounded-lg"
      >
        <p class="font-medium text-destructive mb-2">
          {{ t('ui.warnings') }}
        </p>
        <ul class="text-sm text-destructive space-y-1">
          <li
            v-for="(err, index) in importUsersResult.stats.errors"
            :key="index"
          >
            {{ err }}
          </li>
        </ul>
      </div>
    </div>

    <template #footer>
      <Button @click="$emit('update:importUsersResultDialogOpen', false)">
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
import type { UsersExportData, UsersImportResponse } from '@/api/admin'
import type { ImportProgressState } from './composables/useConfigExportImport'

defineProps<{
  importUsersDialogOpen: boolean
  importUsersResultDialogOpen: boolean
  importUsersPreview: UsersExportData | null
  importUsersResult: UsersImportResponse | null
  usersMergeMode: 'skip' | 'overwrite' | 'error'
  usersMergeModeSelectOpen: boolean
  importUsersLoading: boolean
  importUsersProgress: ImportProgressState | null
}>()

defineEmits<{
  confirm: []
  'update:importUsersDialogOpen': [value: boolean]
  'update:importUsersResultDialogOpen': [value: boolean]
  'update:usersMergeMode': [value: 'skip' | 'overwrite' | 'error']
  'update:usersMergeModeSelectOpen': [value: boolean]
}>()

const { t } = useI18n()

</script>
