<template>
  <PageContainer>
    <PageHeader
      :title="t('ui.barkNotifications')"
      :description="t('ui.thirdPartyPushServiceForTheBarkNotificationChannel')"
    />

    <div class="mt-6 space-y-6">
      <CardSection
        :title="t('ui.serviceSettings')"
        :description="t('ui.configureBarkDeviceKeyServerUrlAndServiceEnabledState')"
      >
        <template #actions>
          <Button
            size="sm"
            :disabled="saving"
            @click="saveConfig"
          >
            {{ saving ? t('ui.saving') : t('ui.save') }}
          </Button>
        </template>

        <div class="space-y-5">
          <div class="flex items-center justify-between gap-4 rounded-lg border border-border/70 px-4 py-3">
            <div>
              <Label class="text-sm font-medium"> {{ t('ui.enableBarkNotifications') }} </Label>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ t('ui.barkMustBeEnabledBeforeTheNotificationServiceCanUse') }}
              </p>
            </div>
            <Switch
              v-model="enabled"
              :disabled="!canEnable"
            />
          </div>

          <div class="grid gap-4 lg:grid-cols-2">
            <div>
              <Label
                for="bark-device-key"
                class="block text-sm font-medium"
              >
                Device Key
              </Label>
              <Input
                id="bark-device-key"
                v-model="deviceKeyInput"
                masked
                :placeholder="deviceKeyIsSet ? t('ui.setLeaveEmptyToKeepUnchanged') : t('ui.findThisInYourBarkAppPushUrl')"
                class="mt-1"
              />
              <p class="mt-1 text-xs text-muted-foreground">
                {{ $t('modules.bark.deviceKeyHint') }}
              </p>
            </div>

            <div>
              <Label
                for="bark-server-url"
                class="block text-sm font-medium"
              > {{ t('ui.serverAddress') }} </Label>
              <Input
                id="bark-server-url"
                v-model="serverUrlInput"
                placeholder="https://api.day.app"
                class="mt-1"
              />
              <p class="mt-1 text-xs text-muted-foreground">
                {{ t('ui.supportsTheOfficialServiceOrASelfHostedBarkServer') }}
              </p>
            </div>
          </div>
        </div>
      </CardSection>

      <CardSection
        :title="t('ui.notificationTemplates')"
        :description="t('ui.templatesSupportValueAndValueVariables')"
      >
        <div>
          <Label
            for="bark-template"
            class="block text-sm font-medium"
          > {{ t('ui.templateContent') }} </Label>
          <Textarea
            id="bark-template"
            v-model="templateInput"
            rows="10"
            class="mt-1 font-mono text-sm"
            placeholder="{body}"
            spellcheck="false"
          />
        </div>
      </CardSection>

      <CardSection
        :title="t('ui.testService')"
        :description="t('ui.sendABarkTestNotificationUsingTheSavedSettings')"
      >
        <div class="flex flex-wrap gap-2">
          <Button
            variant="outline"
            :disabled="testing || !deviceKeyIsSet"
            @click="handleTest"
          >
            {{ testing ? t('auth.register.sendingCode') : t('ui.sendTest') }}
          </Button>
          <RouterLink
            to="/admin/notification-service"
            class="inline-flex h-11 items-center rounded-xl px-3 text-sm text-primary hover:underline"
          >
            {{ t('ui.openNotificationService') }}
          </RouterLink>
        </div>

        <div
          v-if="lastTestResult.length > 0"
          class="mt-4 space-y-2"
        >
          <div
            v-for="item in lastTestResult"
            :key="item.channel"
            class="flex items-center justify-between gap-4 rounded-md border border-border px-3 py-2 text-sm"
          >
            <span>{{ formatNotificationChannel(item.channel) }}</span>
            <span :class="item.success ? 'text-green-600 dark:text-green-400' : 'text-destructive'">
              {{ item.message }}
            </span>
          </div>
        </div>
      </CardSection>
    </div>
  </PageContainer>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { computed, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { Button, Input, Label, Switch, Textarea } from '@/components/ui'
import { PageHeader, PageContainer, CardSection } from '@/components/layout'
import { adminApi } from '@/api/admin'
import { modulesApi } from '@/api/modules'
import { useToast } from '@/composables/useToast'
import { parseApiError } from '@/utils/errorParser'
import { log } from '@/utils/logger'
import { formatNotificationChannel } from './notificationChannel'

const CONFIG_KEYS = {
  enabled: 'module.bark_push.enabled',
  device_key: 'module.bark_push.device_key',
  server_url: 'module.bark_push.server_url',
  template: 'module.bark_push.template',
} as const

const DEFAULT_SERVER_URL = 'https://api.day.app'

const { success, error } = useToast()

const saving = ref(false)
const testing = ref(false)
const enabled = ref(false)
const deviceKeyIsSet = ref(false)
const deviceKeyInput = ref('')
const serverUrlInput = ref(DEFAULT_SERVER_URL)
const templateInput = ref('')
const lastTestResult = ref<Array<{ channel: string; success: boolean; message: string }>>([])

const canEnable = computed(() => deviceKeyIsSet.value || deviceKeyInput.value.trim() !== '')

onMounted(() => {
  loadConfig()
})

async function loadConfig() {
  try {
    const [moduleStatus, configs] = await Promise.all([
      modulesApi.getStatus('bark_push'),
      adminApi.getAllSystemConfigs({ cacheTtlMs: 30_000 }),
    ])
    const configsByKey = new Map(configs.map(config => [config.key, config]))
    const deviceKey = configsByKey.get(CONFIG_KEYS.device_key)
    const serverUrl = configsByKey.get(CONFIG_KEYS.server_url)
    const template = configsByKey.get(CONFIG_KEYS.template)

    enabled.value = moduleStatus.enabled === true
    deviceKeyIsSet.value = deviceKey?.is_set === true
    deviceKeyInput.value = ''
    serverUrlInput.value = typeof serverUrl?.value === 'string' && serverUrl.value.trim()
      ? serverUrl.value
      : DEFAULT_SERVER_URL
    templateInput.value = typeof template?.value === 'string' ? template.value : ''
  } catch (err) {
    error(parseApiError(err, t('ui.failedToLoadBarkSettings')))
    log.error('加载 Bark 推送配置失败:', err)
  }
}

async function saveConfig() {
  saving.value = true
  try {
    const updates: Array<Promise<unknown>> = [
      adminApi.updateSystemConfig(
        CONFIG_KEYS.server_url,
        normalizeServerUrl(serverUrlInput.value),
        'Bark 服务器地址'
      ),
      adminApi.updateSystemConfig(CONFIG_KEYS.template, templateInput.value, 'Bark 推送模板'),
    ]
    const trimmedKey = deviceKeyInput.value.trim()
    if (trimmedKey) {
      updates.push(adminApi.updateSystemConfig(
        CONFIG_KEYS.device_key,
        trimmedKey,
        'Bark Device Key'
      ))
    }
    await Promise.all(updates)
    if (trimmedKey) {
      deviceKeyIsSet.value = true
      deviceKeyInput.value = ''
    }
    if (!canEnable.value) {
      enabled.value = false
    }
    await modulesApi.setEnabled('bark_push', enabled.value)
    success(t('ui.barkSettingsSaved'))
  } catch (err) {
    error(parseApiError(err, t('ui.failedToSaveBarkSettings')))
    log.error('保存 Bark 推送配置失败:', err)
  } finally {
    saving.value = false
  }
}

async function handleTest() {
  testing.value = true
  try {
    const result = await adminApi.testImportantNotification({ channel: 'bark' })
    lastTestResult.value = result.channels || []
    if (result.success) {
      success(result.message || t('ui.testNotificationSent'))
    } else {
      error(result.message || t('ui.failedToSendTestNotification'))
    }
  } catch (err) {
    error(parseApiError(err, t('ui.failedToSendTestNotification')))
    log.error('测试 Bark 推送失败:', err)
  } finally {
    testing.value = false
  }
}

function normalizeServerUrl(value: string): string {
  const trimmed = value.trim().replace(/\/+$/, '')
  return trimmed || DEFAULT_SERVER_URL
}

</script>
