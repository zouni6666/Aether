<template>
  <PageContainer>
    <PageHeader
      :title="t('ui.serverChanNotifications')"
      :description="t('ui.thirdPartyPushServiceForTheServerChanNotificationChannel')"
    />

    <div class="mt-6 space-y-6">
      <CardSection
        :title="t('ui.serviceSettings')"
        :description="t('ui.configureServerChanTurboSendkeyAndServiceEnabledState')"
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
              <Label class="text-sm font-medium"> {{ t('ui.enableServerChanNotifications') }} </Label>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ t('ui.serverChanMustBeEnabledBeforeTheNotificationServiceCan') }}
              </p>
            </div>
            <Switch
              v-model="enabled"
              :disabled="!canEnable"
            />
          </div>

          <div>
            <Label
              for="server-chan-send-key"
              class="block text-sm font-medium"
            >
              SendKey
            </Label>
            <Input
              id="server-chan-send-key"
              v-model="sendKeyInput"
              masked
              :placeholder="sendKeyIsSet ? t('ui.setLeaveEmptyToKeepUnchanged') : 'SCTxxxxxxxxxxxxxxxxxxxxxxxx'"
              class="mt-1"
            />
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('ui.availableInThe') }} <span class="font-mono">sct.ftqq.com</span> {{ t('ui.console') }}
            </p>
          </div>
        </div>
      </CardSection>

      <CardSection
        :title="t('ui.notificationTemplates')"
        :description="t('ui.markdownTemplatesSupportValueAndValueVariables')"
      >
        <div>
          <Label
            for="server-chan-template"
            class="block text-sm font-medium"
          > {{ t('ui.templateContent') }} </Label>
          <Textarea
            id="server-chan-template"
            v-model="templateInput"
            rows="10"
            class="mt-1 font-mono text-sm"
            placeholder="**{title}**&#10;&#10;{body}"
            spellcheck="false"
          />
        </div>
      </CardSection>

      <CardSection
        :title="t('ui.testService')"
        :description="t('ui.sendAServerChanTestNotificationUsingTheSavedSettings')"
      >
        <div class="flex flex-wrap gap-2">
          <Button
            variant="outline"
            :disabled="testing || !sendKeyIsSet"
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
  enabled: 'module.server_chan_push.enabled',
  send_key: 'module.server_chan_push.send_key',
  template: 'module.server_chan_push.template',
} as const

const { success, error } = useToast()

const saving = ref(false)
const testing = ref(false)
const enabled = ref(false)
const sendKeyIsSet = ref(false)
const sendKeyInput = ref('')
const templateInput = ref('')
const lastTestResult = ref<Array<{ channel: string; success: boolean; message: string }>>([])

const canEnable = computed(() => sendKeyIsSet.value || sendKeyInput.value.trim() !== '')

onMounted(() => {
  loadConfig()
})

async function loadConfig() {
  try {
    const [moduleStatus, configs] = await Promise.all([
      modulesApi.getStatus('server_chan_push'),
      adminApi.getAllSystemConfigs({ cacheTtlMs: 30_000 }),
    ])
    const configsByKey = new Map(configs.map(config => [config.key, config]))
    const sendKey = configsByKey.get(CONFIG_KEYS.send_key)
    const template = configsByKey.get(CONFIG_KEYS.template)

    enabled.value = moduleStatus.enabled === true
    sendKeyIsSet.value = sendKey?.is_set === true
    sendKeyInput.value = ''
    templateInput.value = typeof template?.value === 'string' ? template.value : ''
  } catch (err) {
    error(parseApiError(err, t('ui.failedToLoadServerChanSettings')))
    log.error('加载 Server 酱推送配置失败:', err)
  }
}

async function saveConfig() {
  saving.value = true
  try {
    const updates: Array<Promise<unknown>> = [
      adminApi.updateSystemConfig(CONFIG_KEYS.template, templateInput.value, 'Server 酱推送模板'),
    ]
    const trimmedKey = sendKeyInput.value.trim()
    if (trimmedKey) {
      updates.push(adminApi.updateSystemConfig(CONFIG_KEYS.send_key, trimmedKey, 'Server 酱 SendKey'))
    }
    await Promise.all(updates)
    if (trimmedKey) {
      sendKeyIsSet.value = true
      sendKeyInput.value = ''
    }
    if (!canEnable.value) {
      enabled.value = false
    }
    await modulesApi.setEnabled('server_chan_push', enabled.value)
    success(t('ui.serverChanSettingsSaved'))
  } catch (err) {
    error(parseApiError(err, t('ui.failedToSaveServerChanSettings')))
    log.error('保存 Server 酱推送配置失败:', err)
  } finally {
    saving.value = false
  }
}

async function handleTest() {
  testing.value = true
  try {
    const result = await adminApi.testImportantNotification({ channel: 'server_chan' })
    lastTestResult.value = result.channels || []
    if (result.success) {
      success(result.message || t('ui.testNotificationSent'))
    } else {
      error(result.message || t('ui.failedToSendTestNotification'))
    }
  } catch (err) {
    error(parseApiError(err, t('ui.failedToSendTestNotification')))
    log.error('测试 Server 酱推送失败:', err)
  } finally {
    testing.value = false
  }
}

</script>
