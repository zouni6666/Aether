<template>
  <PageContainer>
    <PageHeader
      :title="t('ui.s3Backups')"
      :description="t('ui.backUpSelectedExportScopesToS3CompatibleStorage')"
    />

    <CardSection
      :title="t('ui.backupConfiguration')"
      :description="t('ui.configureAutomaticBackupScheduleObjectStorageConnectionAndRetentionPolicy')"
      class="mt-6"
    >
      <template #actions>
        <div class="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            :disabled="backupRunDisabled"
            @click="backup.runS3BackupNow"
          >
            <Play class="w-3.5 h-3.5 mr-1.5" />
            {{ backup.running.value ? t('ui.submitting') : t('ui.backUpNow') }}
          </Button>
          <Button
            size="sm"
            :disabled="backup.saving.value || !backup.hasChanges.value"
            @click="backup.saveS3BackupConfig"
          >
            <Save class="w-3.5 h-3.5 mr-1.5" />
            {{ backup.saving.value ? t('ui.saving') : t('ui.save') }}
          </Button>
        </div>
      </template>

      <div class="space-y-5">
        <div class="flex items-center justify-between rounded-lg border border-border p-4">
          <div class="flex items-center gap-3 min-w-0">
            <div class="w-9 h-9 rounded-lg bg-primary/10 text-primary flex items-center justify-center shrink-0">
              <CloudUpload class="w-4.5 h-4.5" />
            </div>
            <div class="min-w-0">
              <h4 class="text-sm font-medium">
                {{ t('ui.automaticBackups') }}
              </h4>
              <p class="text-xs text-muted-foreground mt-0.5">
                {{ backup.config.value.enabled ? t('ui.enableddfb8022') : t('ui.disabled') }}
              </p>
            </div>
          </div>
          <Switch
            :model-value="backup.config.value.enabled"
            @update:model-value="backup.config.value.enabled = $event"
          />
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <Label
              for="backup-scope"
              class="block text-sm font-medium"
            > {{ t('ui.backupScope') }} </Label>
            <Select
              :model-value="backup.config.value.scope"
              @update:model-value="setScope"
            >
              <SelectTrigger
                id="backup-scope"
                class="mt-1"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="config">
                  {{ t('ui.configurationData') }}
                </SelectItem>
                <SelectItem value="users">
                  {{ t('ui.userData') }}
                </SelectItem>
                <SelectItem value="data">
                  {{ t('ui.fullBackup') }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div>
            <Label
              for="backup-bucket"
              class="block text-sm font-medium"
            >
              Bucket
            </Label>
            <Input
              id="backup-bucket"
              :model-value="backup.config.value.bucket"
              class="mt-1"
              placeholder="aether-backups"
              @update:model-value="backup.config.value.bucket = String($event)"
            />
          </div>

          <div>
            <Label
              for="backup-endpoint"
              class="block text-sm font-medium"
            >
              Endpoint
            </Label>
            <Input
              id="backup-endpoint"
              :model-value="backup.config.value.endpoint"
              class="mt-1"
              placeholder="https://s3.example.com"
              @update:model-value="backup.config.value.endpoint = String($event)"
            />
          </div>

          <div>
            <Label
              for="backup-access-key"
              class="block text-sm font-medium"
            >
              Access Key ID
            </Label>
            <Input
              id="backup-access-key"
              :model-value="backup.config.value.accessKeyId"
              class="mt-1"
              autocomplete="off"
              @update:model-value="backup.config.value.accessKeyId = String($event)"
            />
          </div>

          <div>
            <div class="flex items-center justify-between gap-2">
              <Label
                for="backup-secret-key"
                class="block text-sm font-medium"
              >
                Secret Access Key
              </Label>
              <button
                v-if="backup.config.value.secretAccessKeyIsSet"
                type="button"
                class="text-xs text-muted-foreground hover:text-foreground"
                :disabled="backup.saving.value"
                @click="backup.clearS3SecretAccessKey"
              >
                {{ t('ui.clear') }}
              </button>
            </div>
            <div class="relative mt-1">
              <Input
                id="backup-secret-key"
                :model-value="backup.config.value.secretAccessKey"
                masked
                disable-autofill
                autocomplete="new-password"
                :placeholder="backup.config.value.secretAccessKeyIsSet ? t('ui.configuredLeaveEmptyToKeepUnchanged30fda99') : ''"
                @update:model-value="backup.config.value.secretAccessKey = String($event)"
              />
              <KeyRound
                v-if="backup.config.value.secretAccessKeyIsSet && !backup.config.value.secretAccessKey"
                class="absolute right-10 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none"
              />
            </div>
          </div>

          <div>
            <Label
              for="backup-retention"
              class="block text-sm font-medium"
            > {{ t('ui.maximumBackupsToRetain') }} </Label>
            <Input
              id="backup-retention"
              :model-value="backup.config.value.retentionCount"
              type="number"
              min="1"
              class="mt-1"
              @update:model-value="setNumber('retentionCount', $event, 1)"
            />
          </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-[1fr_1fr] gap-4">
          <div>
            <Label
              for="backup-interval"
              class="block text-sm font-medium"
            > {{ t('ui.interval') }} </Label>
            <Input
              id="backup-interval"
              :model-value="backup.config.value.scheduleInterval"
              type="number"
              min="1"
              class="mt-1"
              @update:model-value="setNumber('scheduleInterval', $event, 1)"
            />
          </div>
          <div>
            <Label
              for="backup-unit"
              class="block text-sm font-medium"
            > {{ t('ui.intervalUnit') }} </Label>
            <Select
              :model-value="backup.config.value.scheduleUnit"
              @update:model-value="setScheduleUnit"
            >
              <SelectTrigger
                id="backup-unit"
                class="mt-1"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="hours">
                  {{ t('ui.hours') }}
                </SelectItem>
                <SelectItem value="days">
                  {{ t('ui.days') }}
                </SelectItem>
                <SelectItem value="weeks">
                  {{ t('providers.antigravity.quota.period.weekly') }}
                </SelectItem>
                <SelectItem value="months">
                  {{ t('ui.month') }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <div>
          <Button
            variant="ghost"
            size="sm"
            class="px-0"
            @click="backup.advancedOpen.value = !backup.advancedOpen.value"
          >
            <ChevronDown
              v-if="backup.advancedOpen.value"
              class="w-3.5 h-3.5 mr-1.5"
            />
            <ChevronRight
              v-else
              class="w-3.5 h-3.5 mr-1.5"
            /> {{ t('ui.advancedOptions') }}
          </Button>

          <div
            v-if="backup.advancedOpen.value"
            class="mt-3 grid grid-cols-1 md:grid-cols-2 gap-4"
          >
            <div>
              <Label
                for="backup-region"
                class="block text-sm font-medium"
              >
                Region
              </Label>
              <Input
                id="backup-region"
                :model-value="backup.config.value.region"
                class="mt-1"
                placeholder="auto"
                @update:model-value="backup.config.value.region = String($event)"
              />
            </div>
            <div>
              <Label
                for="backup-user-agent"
                class="block text-sm font-medium"
              >
                User-Agent
              </Label>
              <Input
                id="backup-user-agent"
                :model-value="backup.config.value.userAgent"
                class="mt-1"
                placeholder="rclone/v1.68.0"
                @update:model-value="backup.config.value.userAgent = String($event)"
              />
            </div>
            <div>
              <Label
                for="backup-prefix"
                class="block text-sm font-medium"
              >
                Prefix
              </Label>
              <Input
                id="backup-prefix"
                :model-value="backup.config.value.prefix"
                class="mt-1"
                placeholder="aether/backups/"
                @update:model-value="backup.config.value.prefix = String($event)"
              />
            </div>
            <div>
              <Label
                for="backup-hour"
                class="block text-sm font-medium"
              > {{ t('ui.hour') }} </Label>
              <Input
                id="backup-hour"
                :model-value="backup.config.value.scheduleHour"
                type="number"
                min="0"
                max="23"
                class="mt-1"
                @update:model-value="setNumber('scheduleHour', $event, 0)"
              />
            </div>
            <div>
              <Label
                for="backup-minute"
                class="block text-sm font-medium"
              > {{ t('ui.minute') }} </Label>
              <Input
                id="backup-minute"
                :model-value="backup.config.value.scheduleMinute"
                type="number"
                min="0"
                max="59"
                class="mt-1"
                @update:model-value="setNumber('scheduleMinute', $event, 0)"
              />
            </div>
            <div v-if="backup.config.value.scheduleUnit === 'weeks'">
              <Label
                for="backup-weekday"
                class="block text-sm font-medium"
              > {{ t('ui.weekday') }} </Label>
              <Select
                :model-value="String(backup.config.value.scheduleWeekday)"
                @update:model-value="setNumber('scheduleWeekday', $event, 1)"
              >
                <SelectTrigger
                  id="backup-weekday"
                  class="mt-1"
                >
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem
                    v-for="item in weekdays"
                    :key="item.value"
                    :value="String(item.value)"
                  >
                    {{ item.label }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div v-if="backup.config.value.scheduleUnit === 'months'">
              <Label
                for="backup-month-day"
                class="block text-sm font-medium"
              > {{ t('ui.dayOfMonth') }} </Label>
              <Input
                id="backup-month-day"
                :model-value="backup.config.value.scheduleMonthDay"
                type="number"
                min="1"
                max="31"
                class="mt-1"
                @update:model-value="setNumber('scheduleMonthDay', $event, 1)"
              />
            </div>
            <div class="flex items-center justify-between rounded-lg border border-border p-4">
              <span class="text-sm font-medium">Path Style</span>
              <Switch
                :model-value="backup.config.value.pathStyle"
                @update:model-value="backup.config.value.pathStyle = $event"
              />
            </div>
            <div>
              <Label
                for="backup-compression"
                class="block text-sm font-medium"
              > {{ t('ui.compressionFormat') }} </Label>
              <Select
                :model-value="backup.config.value.compression"
                @update:model-value="backup.config.value.compression = $event"
              >
                <SelectTrigger
                  id="backup-compression"
                  class="mt-1"
                >
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="zstd">
                    zstd
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>
      </div>
    </CardSection>
  </PageContainer>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { computed, onMounted } from 'vue'
import {
  ChevronDown,
  ChevronRight,
  CloudUpload,
  KeyRound,
  Play,
  Save,
} from 'lucide-vue-next'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import Select from '@/components/ui/select.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectValue from '@/components/ui/select-value.vue'
import Switch from '@/components/ui/switch.vue'
import { CardSection, PageContainer, PageHeader } from '@/components/layout'
import {
  useS3BackupConfig,
  type S3BackupConfig,
  type S3BackupScheduleUnit,
} from './composables/useS3BackupConfig'
import type { S3BackupScope } from '@/api/admin'

const backup = useS3BackupConfig()

const backupRunDisabled = computed(() =>
  backup.running.value ||
  backup.saving.value ||
  backup.loading.value ||
  backup.hasChanges.value
)

const weekdays = [
  { value: 1, get label() { return t('ui.mon') } },
  { value: 2, get label() { return t('ui.tue') } },
  { value: 3, get label() { return t('ui.wed') } },
  { value: 4, get label() { return t('ui.thu') } },
  { value: 5, get label() { return t('ui.fri') } },
  { value: 6, get label() { return t('ui.sat') } },
  { value: 7, get label() { return t('ui.sun') } },
]

function setScope(value: string) {
  if (value === 'config' || value === 'users' || value === 'data') {
    backup.config.value.scope = value as S3BackupScope
  }
}

function setScheduleUnit(value: string) {
  if (value === 'hours' || value === 'days' || value === 'weeks' || value === 'months') {
    backup.config.value.scheduleUnit = value as S3BackupScheduleUnit
  }
}

function setNumber(field: keyof Pick<
  S3BackupConfig,
  | 'retentionCount'
  | 'scheduleInterval'
  | 'scheduleMinute'
  | 'scheduleHour'
  | 'scheduleWeekday'
  | 'scheduleMonthDay'
>, value: string | number, fallback: number) {
  const next = Number(value)
  backup.config.value[field] = Number.isFinite(next) ? next : fallback
}

onMounted(() => {
  backup.loadS3BackupConfig()
})

</script>
