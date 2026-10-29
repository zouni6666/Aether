<template>
  <PageContainer>
    <PageHeader
      :title="t('ui.notificationService')"
      :description="t('ui.manageNotificationsTemplatesAndPushServicesInOnePlace')"
    />

    <div class="mt-6 space-y-6">
      <CardSection
        :title="t('ui.notificationServiceSettings')"
        :description="t('ui.selectTheDefaultPushServiceAndConfigureEmailAndThird')"
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

        <div class="space-y-6">
          <div class="grid gap-4 lg:grid-cols-[minmax(0,1fr)_320px]">
            <div>
              <Label class="block text-sm font-medium"> {{ t('ui.defaultPushService7bb64ce') }} </Label>
              <Select v-model="config.default_channel">
                <SelectTrigger class="mt-1">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">
                    {{ t('ui.allAvailableServices') }}
                  </SelectItem>
                  <SelectItem value="email">
                    {{ t('ui.email') }}
                  </SelectItem>
                  <SelectItem value="server_chan">
                    {{ t('ui.serverChan') }}
                  </SelectItem>
                  <SelectItem value="bark">
                    Bark
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div class="flex items-center justify-between gap-4">
              <div>
                <Label class="text-sm font-medium"> {{ t('ui.enableNotificationServicea224643') }} </Label>
                <p class="mt-1 text-xs text-muted-foreground">
                  {{ canEnableService ? t('ui.pushServicesAreAvailableForTheCurrentPolicy') : t('ui.noPushServicesAreAvailableForTheCurrentPolicy') }}
                </p>
              </div>
              <Switch
                v-model="config.enabled"
                :disabled="!canEnableService"
              />
            </div>
          </div>

          <div class="grid gap-6 border-t border-border/60 pt-5 lg:grid-cols-3">
            <section class="space-y-4">
              <div class="flex items-center justify-between gap-3">
                <div>
                  <div class="flex items-center gap-2">
                    <Label class="text-sm font-medium"> {{ t('guide.nav.modules.emailConfig') }} </Label>
                    <Badge :variant="emailReady ? 'success' : 'outline'">
                      {{ emailReady ? t('ui.available4d99c97') : t('ui.notReady') }}
                    </Badge>
                  </div>
                  <p class="mt-1 text-xs text-muted-foreground">
                    {{ t('ui.smtpSettingsAreManagedIn') }} <RouterLink
                      to="/admin/email"
                      class="text-primary hover:underline"
                    >
                      {{ t('guide.nav.modules.emailConfig') }}
                    </RouterLink> {{ t('ui.settings0437a8c') }}
                  </p>
                </div>
                <Switch
                  v-model="config.email_enabled"
                  :disabled="!smtpConfigured"
                />
              </div>

              <div>
                <Label
                  for="notification-service-recipients"
                  class="block text-sm font-medium"
                > {{ t('ui.administratorRecipients02c2945') }} </Label>
                <Textarea
                  id="notification-service-recipients"
                  v-model="config.email_recipients"
                  rows="4"
                  placeholder="ops@example.com&#10;admin@example.com"
                  class="mt-1"
                />
              </div>
            </section>

            <section class="space-y-4">
              <div class="flex items-center justify-between gap-3">
                <div>
                  <div class="flex items-center gap-2">
                    <Label class="text-sm font-medium"> {{ t('ui.serverChan') }} </Label>
                    <Badge :variant="serverChanReady ? 'success' : 'outline'">
                      {{ serverChanReady ? t('ui.available4d99c97') : t('ui.notReady') }}
                    </Badge>
                  </div>
                  <p class="mt-1 text-xs text-muted-foreground">
                    {{ t('ui.enableThirdPartyPushServicesSeparatelyInExtensionModules') }}
                  </p>
                </div>
              </div>

              <RouterLink
                to="/admin/modules/server-chan"
                class="inline-flex h-11 items-center rounded-xl border border-border/60 bg-card/60 px-4 text-sm font-semibold text-foreground hover:border-primary/60 hover:bg-primary/10 hover:text-primary"
              >
                {{ t('ui.configureServerChan') }}
              </RouterLink>
            </section>

            <section class="space-y-4">
              <div class="flex items-center justify-between gap-3">
                <div>
                  <div class="flex items-center gap-2">
                    <Label class="text-sm font-medium">
                      Bark
                    </Label>
                    <Badge :variant="barkReady ? 'success' : 'outline'">
                      {{ barkReady ? t('ui.available4d99c97') : t('ui.notReady') }}
                    </Badge>
                  </div>
                  <p class="mt-1 text-xs text-muted-foreground">
                    {{ t('ui.sendNotificationsToIosDevicesThroughBark') }}
                  </p>
                </div>
              </div>

              <RouterLink
                to="/admin/modules/bark"
                class="inline-flex h-11 items-center rounded-xl border border-border/60 bg-card/60 px-4 text-sm font-semibold text-foreground hover:border-primary/60 hover:bg-primary/10 hover:text-primary"
              >
                {{ t('ui.configureBark') }}
              </RouterLink>
            </section>
          </div>
        </div>
      </CardSection>

      <CardSection
        :title="t('ui.notificationItems')"
        :description="t('ui.eachNotificationCanUseTheDefaultServiceOrASpecific')"
      >
        <template #actions>
          <Button
            size="sm"
            variant="outline"
            @click="addItem"
          >
            <Plus class="mr-1.5 h-4 w-4" /> {{ t('ui.addNotification') }}
          </Button>
        </template>

        <div class="space-y-4">
          <div
            v-for="(item, index) in config.items"
            :key="item.local_id"
            class="rounded-lg border border-border/70 p-4"
          >
            <div class="flex flex-wrap items-start justify-between gap-3">
              <div class="min-w-0">
                <div class="flex flex-wrap items-center gap-2">
                  <Label class="text-sm font-semibold">
                    {{ item.name || item.key || t('ui.untitledNotification') }}
                  </Label>
                  <Badge
                    v-if="item.system"
                    variant="outline"
                  >
                    {{ t('ui.builtIn95e35aa') }}
                  </Badge>
                  <Badge :variant="isItemReady(item) ? 'success' : 'outline'">
                    {{ isItemReady(item) ? t('ui.deliverable') : t('ui.notReady') }}
                  </Badge>
                </div>
                <p class="mt-1 truncate text-xs text-muted-foreground">
                  {{ item.key }}
                </p>
              </div>
              <div class="flex items-center gap-2">
                <Switch v-model="item.enabled" />
                <Button
                  v-if="!item.system"
                  size="icon"
                  variant="ghost"
                  @click="removeItem(index)"
                >
                  <Trash2 class="h-4 w-4" />
                </Button>
              </div>
            </div>

            <div class="mt-4 grid gap-4 lg:grid-cols-2">
              <div>
                <Label class="block text-xs font-medium"> {{ t('ui.notificationKey') }} </Label>
                <Input
                  v-model="item.key"
                  class="mt-1"
                  :disabled="item.system"
                  placeholder="custom_event"
                />
              </div>
              <div>
                <Label class="block text-xs font-medium"> {{ t('stats.column.name') }} </Label>
                <Input
                  v-model="item.name"
                  class="mt-1"
                  :placeholder="t('ui.customNotification')"
                />
              </div>
              <div>
                <Label class="block text-xs font-medium"> {{ t('ui.pushService') }} </Label>
                <Select v-model="item.channel">
                  <SelectTrigger class="mt-1">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="global">
                      {{ t('ui.useDefault') }}
                    </SelectItem>
                    <SelectItem value="all">
                      {{ t('ui.allAvailableServices') }}
                    </SelectItem>
                    <SelectItem value="email">
                      {{ t('ui.email') }}
                    </SelectItem>
                    <SelectItem value="server_chan">
                      {{ t('ui.serverChan') }}
                    </SelectItem>
                    <SelectItem value="bark">
                      Bark
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div class="flex items-center justify-between rounded-lg border border-border/70 px-4 py-3">
                <div>
                  <Label class="text-xs font-medium"> {{ t('ui.userEmail') }} </Label>
                  <p class="mt-1 text-xs text-muted-foreground">
                    {{ t('ui.allowDeliveryToTheUserEmailAddress') }}
                  </p>
                </div>
                <Switch
                  v-model="item.user_email_enabled"
                  :disabled="!smtpConfigured"
                />
              </div>
            </div>

            <div class="mt-4 grid gap-4">
              <div>
                <Label class="block text-xs font-medium"> {{ t('ui.titleTemplate') }} </Label>
                <Input
                  v-model="item.title_template"
                  class="mt-1"
                  placeholder="{title}"
                />
              </div>
              <div class="grid gap-4 lg:grid-cols-2">
                <div>
                  <Label class="block text-xs font-medium"> {{ t('ui.markdownTemplate') }} </Label>
                  <Textarea
                    v-model="item.markdown_template"
                    rows="5"
                    class="mt-1 font-mono text-sm"
                    placeholder="{body}"
                  />
                </div>
                <div>
                  <Label class="block text-xs font-medium"> {{ t('ui.textTemplate') }} </Label>
                  <Textarea
                    v-model="item.text_template"
                    rows="5"
                    class="mt-1 font-mono text-sm"
                    placeholder="{text_body}"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </CardSection>

      <CardSection
        :title="t('ui.testNotification')"
        :description="t('ui.sendATestNotificationUsingTheSavedSettings')"
      >
        <div class="grid gap-3 sm:grid-cols-[minmax(0,1fr)_180px_auto]">
          <Select v-model="testItemKey">
            <SelectTrigger>
              <SelectValue :placeholder="t('ui.selectNotification')" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="item in config.items"
                :key="item.local_id"
                :value="item.key"
              >
                {{ item.name || item.key }}
              </SelectItem>
            </SelectContent>
          </Select>
          <Select v-model="testChannel">
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="global">
                {{ t('ui.byNotification') }}
              </SelectItem>
              <SelectItem value="all">
                {{ t('ui.allAvailableServices') }}
              </SelectItem>
              <SelectItem value="email">
                {{ t('ui.email') }}
              </SelectItem>
              <SelectItem value="server_chan">
                {{ t('ui.serverChan') }}
              </SelectItem>
              <SelectItem value="bark">
                Bark
              </SelectItem>
            </SelectContent>
          </Select>
          <Button
            variant="outline"
            :disabled="testing || !testItemKey"
            @click="handleTest"
          >
            <Send class="mr-1.5 h-4 w-4" />
            {{ testing ? t('auth.register.sendingCode') : t('ui.sendTest') }}
          </Button>
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
import { Plus, Send, Trash2 } from 'lucide-vue-next'
import {
  Badge,
  Button,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Textarea,
} from '@/components/ui'
import { PageHeader, PageContainer, CardSection } from '@/components/layout'
import { adminApi } from '@/api/admin'
import { modulesApi, type ModuleStatus } from '@/api/modules'
import { useToast } from '@/composables/useToast'
import { parseApiError } from '@/utils/errorParser'
import { log } from '@/utils/logger'
import { formatNotificationChannel } from './notificationChannel'

type DeliveryChannel = 'global' | 'all' | 'email' | 'server_chan' | 'bark'

interface NotificationItem {
  local_id: string
  key: string
  name: string
  enabled: boolean
  channel: DeliveryChannel
  title_template: string
  markdown_template: string
  text_template: string
  user_email_enabled: boolean
  system: boolean
}

interface NotificationConfig {
  enabled: boolean
  email_enabled: boolean
  email_recipients: string
  default_channel: Exclude<DeliveryChannel, 'global'>
  items: NotificationItem[]
}

const CONFIG_KEYS = {
  enabled: 'module.important_notification.enabled',
  email_enabled: 'module.important_notification.email_enabled',
  email_recipients: 'module.important_notification.email_recipients',
  default_channel: 'module.important_notification.default_channel',
  items: 'module.important_notification.items',
  server_chan_send_key: 'module.server_chan_push.send_key',
  bark_device_key: 'module.bark_push.device_key',
} as const

const DEFAULT_ITEMS: NotificationItem[] = [
  {
    local_id: 'provider_quota_alert',
    key: 'provider_quota_alert',
    name: '号池额度不足',
    enabled: true,
    channel: 'global',
    title_template: '',
    markdown_template: '',
    text_template: '',
    user_email_enabled: false,
    system: true,
  },
  {
    local_id: 'provider_pool_abnormal',
    key: 'provider_pool_abnormal',
    name: '号池异常',
    enabled: true,
    channel: 'global',
    title_template: '号池异常：{provider_name}',
    markdown_template: '号池 `{provider_name}` 出现异常，请检查服务状态。',
    text_template: '号池 {provider_name} 出现异常，请检查服务状态。',
    user_email_enabled: false,
    system: true,
  },
  {
    local_id: 'user_balance_low',
    key: 'user_balance_low',
    name: '用户余额不足',
    enabled: true,
    channel: 'email',
    title_template: '余额不足提醒',
    markdown_template: '你的账户余额已低于提醒阈值，请及时处理。',
    text_template: '你的账户余额已低于提醒阈值，请及时处理。',
    user_email_enabled: true,
    system: true,
  },
]

const { success, error } = useToast()

const saving = ref(false)
const testing = ref(false)
const smtpConfigured = ref(false)
const serverChanKeyIsSet = ref(false)
const serverChanStatus = ref<ModuleStatus | null>(null)
const barkKeyIsSet = ref(false)
const barkStatus = ref<ModuleStatus | null>(null)
const testItemKey = ref('provider_quota_alert')
const testChannel = ref<DeliveryChannel>('global')
const lastTestResult = ref<Array<{ channel: string; success: boolean; message: string }>>([])

const config = ref<NotificationConfig>({
  enabled: false,
  email_enabled: false,
  email_recipients: '',
  default_channel: 'all',
  items: cloneDefaultItems(),
})

const emailReady = computed(() => {
  return config.value.email_enabled && smtpConfigured.value && config.value.email_recipients.trim() !== ''
})

const serverChanReady = computed(() => {
  return serverChanStatus.value?.enabled === true && serverChanKeyIsSet.value
})

const barkReady = computed(() => {
  return barkStatus.value?.enabled === true && barkKeyIsSet.value
})

const canEnableService = computed(() => {
  if (deliveryReady(config.value.default_channel)) return true
  return config.value.items.some(item => item.enabled && isItemReady(item))
})

onMounted(() => {
  loadConfig()
})

async function loadConfig() {
  try {
    const [moduleStatuses, configs] = await Promise.all([
      modulesApi.getAllStatus(),
      adminApi.getAllSystemConfigs({ cacheTtlMs: 30_000 }),
    ])
    const moduleStatus = moduleStatuses.important_notification
    const serverChanModuleStatus = moduleStatuses.server_chan_push
    const barkModuleStatus = moduleStatuses.bark_push
    const configsByKey = new Map(configs.map(config => [config.key, config]))
    const emailEnabled = configsByKey.get(CONFIG_KEYS.email_enabled)
    const recipients = configsByKey.get(CONFIG_KEYS.email_recipients)
    const defaultChannel = configsByKey.get(CONFIG_KEYS.default_channel)
    const items = configsByKey.get(CONFIG_KEYS.items)
    const serverChanKey = configsByKey.get(CONFIG_KEYS.server_chan_send_key)
    const barkDeviceKey = configsByKey.get(CONFIG_KEYS.bark_device_key)
    const smtpHost = configsByKey.get('smtp_host')
    const smtpFromEmail = configsByKey.get('smtp_from_email')

    config.value.enabled = moduleStatus.enabled === true
    config.value.email_enabled = emailEnabled?.value === true
    config.value.email_recipients = normalizeRecipients(recipients?.value)
    config.value.default_channel = normalizeDefaultChannel(defaultChannel?.value)
    config.value.items = normalizeItems(items?.value)
    serverChanStatus.value = serverChanModuleStatus
    serverChanKeyIsSet.value = serverChanKey?.is_set === true
    barkStatus.value = barkModuleStatus
    barkKeyIsSet.value = barkDeviceKey?.is_set === true
    smtpConfigured.value = isNonEmptyString(smtpHost?.value) && isNonEmptyString(smtpFromEmail?.value)
    if (!config.value.items.some(item => item.key === testItemKey.value)) {
      testItemKey.value = config.value.items[0]?.key || ''
    }
  } catch (err) {
    error(parseApiError(err, t('ui.failedToLoadNotificationServiceSettings')))
    log.error('加载通知服务配置失败:', err)
  }
}

async function saveConfig() {
  saving.value = true
  try {
    if (!canEnableService.value) {
      config.value.enabled = false
    }
    await Promise.all([
      adminApi.updateSystemConfig(CONFIG_KEYS.email_enabled, config.value.email_enabled, '通知服务邮件推送开关'),
      adminApi.updateSystemConfig(CONFIG_KEYS.email_recipients, config.value.email_recipients, '通知服务管理员收件人'),
      adminApi.updateSystemConfig(CONFIG_KEYS.default_channel, config.value.default_channel, '通知服务全局推送服务'),
      adminApi.updateSystemConfig(CONFIG_KEYS.items, serializeItems(), '通知服务通知项和模板'),
    ])
    await adminApi.updateSystemConfig(CONFIG_KEYS.enabled, config.value.enabled, '通知服务总开关')
    success(t('ui.notificationServiceSettingsSaved'))
  } catch (err) {
    error(parseApiError(err, t('ui.failedToSaveNotificationServiceSettings')))
    log.error('保存通知服务配置失败:', err)
  } finally {
    saving.value = false
  }
}

async function handleTest() {
  testing.value = true
  try {
    const result = await adminApi.testImportantNotification({
      item_key: testItemKey.value,
      channel: testChannel.value === 'global' ? undefined : testChannel.value,
    })
    lastTestResult.value = result.channels || []
    if (result.success) {
      success(result.message || t('ui.testNotificationSent'))
    } else {
      error(result.message || t('ui.failedToSendTestNotification'))
    }
  } catch (err) {
    error(parseApiError(err, t('ui.failedToSendTestNotification')))
    log.error('测试通知服务失败:', err)
  } finally {
    testing.value = false
  }
}

function addItem() {
  const suffix = Date.now().toString(36)
  const key = `custom_${suffix}`
  config.value.items.push({
    local_id: key,
    key,
    name: '自定义通知',
    enabled: true,
    channel: 'global',
    title_template: '',
    markdown_template: '',
    text_template: '',
    user_email_enabled: false,
    system: false,
  })
  testItemKey.value = key
}

function removeItem(index: number) {
  const [removed] = config.value.items.splice(index, 1)
  if (removed?.key === testItemKey.value) {
    testItemKey.value = config.value.items[0]?.key || ''
  }
}

function isItemReady(item: NotificationItem): boolean {
  if (!item.enabled) return false
  return deliveryReady(resolveItemChannel(item))
}

function deliveryReady(channel: Exclude<DeliveryChannel, 'global'>): boolean {
  if (channel === 'all') return emailReady.value || serverChanReady.value || barkReady.value
  if (channel === 'email') return emailReady.value
  if (channel === 'server_chan') return serverChanReady.value
  if (channel === 'bark') return barkReady.value
  return false
}

function resolveItemChannel(item: NotificationItem): Exclude<DeliveryChannel, 'global'> {
  return item.channel === 'global' ? config.value.default_channel : item.channel
}

function serializeItems() {
  return config.value.items.map(item => ({
    key: normalizeItemKey(item.key),
    name: item.name.trim() || normalizeItemKey(item.key),
    enabled: item.enabled,
    channel: item.channel,
    title_template: item.title_template.trim(),
    markdown_template: item.markdown_template.trim(),
    text_template: item.text_template.trim(),
    user_email_enabled: item.user_email_enabled,
    system: item.system,
  }))
}

function normalizeItems(value: unknown): NotificationItem[] {
  if (!Array.isArray(value)) return cloneDefaultItems()
  const items = value
    .map((item, index) => normalizeItem(item, index))
    .filter((item): item is NotificationItem => item !== null)
  return items.length > 0 ? items : cloneDefaultItems()
}

function normalizeItem(value: unknown, index: number): NotificationItem | null {
  if (!value || typeof value !== 'object') return null
  const raw = value as Record<string, unknown>
  const key = normalizeItemKey(raw.key)
  if (!key) return null
  return {
    local_id: `${key}_${index}`,
    key,
    name: typeof raw.name === 'string' && raw.name.trim() ? raw.name.trim() : key,
    enabled: raw.enabled !== false,
    channel: normalizeItemChannel(raw.channel),
    title_template: typeof raw.title_template === 'string' ? raw.title_template : '',
    markdown_template: typeof raw.markdown_template === 'string' ? raw.markdown_template : '',
    text_template: typeof raw.text_template === 'string' ? raw.text_template : '',
    user_email_enabled: raw.user_email_enabled === true,
    system: raw.system === true,
  }
}

function normalizeItemKey(value: unknown): string {
  if (typeof value !== 'string') return ''
  return value.trim().replace(/[^A-Za-z0-9_.:-]/g, '_').slice(0, 64)
}

function normalizeItemChannel(value: unknown): DeliveryChannel {
  if (value === 'all' || value === 'email' || value === 'server_chan' || value === 'bark') return value
  return 'global'
}

function normalizeDefaultChannel(value: unknown): Exclude<DeliveryChannel, 'global'> {
  if (value === 'email' || value === 'server_chan' || value === 'bark') return value
  return 'all'
}

function cloneDefaultItems(): NotificationItem[] {
  return DEFAULT_ITEMS.map(item => ({ ...item }))
}

function isNonEmptyString(value: unknown): boolean {
  return typeof value === 'string' && value.trim() !== ''
}

function normalizeRecipients(value: unknown): string {
  if (Array.isArray(value)) {
    return value
      .map(item => String(item).trim())
      .filter(Boolean)
      .join('\n')
  }
  return typeof value === 'string' ? value : ''
}

</script>
