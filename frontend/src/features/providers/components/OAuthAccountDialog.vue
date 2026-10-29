<template>
  <Dialog
    :model-value="isOpen"
    :title="t('ui.addAccount')"
    :icon="UserPlus"
    size="md"
    @update:model-value="handleDialogUpdate"
  >
    <!-- 右上角代理按钮 -->
    <template #header-actions>
      <Popover
        :open="proxyPopoverOpen"
        @update:open="(v: boolean) => { proxyPopoverOpen = v; if (v) proxyNodesStore.ensureLoaded() }"
      >
        <PopoverTrigger as-child>
          <button
            class="flex items-center justify-center w-8 h-8 rounded-md transition-colors shrink-0"
            :class="selectedProxyNodeId
              ? 'text-blue-500 bg-blue-500/10 hover:bg-blue-500/20'
              : 'text-muted-foreground hover:text-foreground hover:bg-muted'"
            :title="selectedProxyNodeId ? `${t('ui.proxy')}: ${getSelectedNodeLabel()}` : t('ui.setProxyNode')"
          >
            <Globe class="w-4 h-4" />
          </button>
        </PopoverTrigger>
        <PopoverContent
          class="w-72 p-3 z-[80]"
          side="bottom"
          align="end"
        >
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-1.5">
                <span class="text-xs font-medium">{{ t('ui.proxyNodes23f1b6a') }}</span>
                <span
                  v-if="!proxyNodesStore.loading && proxyNodesStore.onlineNodes.length === 0"
                  class="text-[10px] text-muted-foreground"
                >· {{ t('ui.goToModulesProxyNodesToAddOne') }}</span>
              </div>
              <button
                v-if="selectedProxyNodeId"
                class="text-[10px] text-muted-foreground hover:text-foreground transition-colors"
                @click="selectedProxyNodeId = ''; proxyPopoverOpen = false"
              >
                {{ t('ui.clear') }}
              </button>
            </div>
            <ProxyNodeSelect
              :model-value="selectedProxyNodeId"
              trigger-class="h-8"
              @update:model-value="(v: string) => { selectedProxyNodeId = v; proxyPopoverOpen = false }"
            />
            <p class="text-[10px] text-muted-foreground">
              {{ selectedProxyNodeId ? proxyUsageDescription : t('ui.notSetFallsBackToProviderProxyThenSystemProxy') }}
            </p>
          </div>
        </PopoverContent>
      </Popover>
    </template>

    <div class="space-y-4">
      <!-- Tab 切换 -->
      <div
        v-if="showAuthorizationMode"
        class="grid rounded-lg border border-border p-0.5 bg-muted/30"
        :class="isCodexProvider || isClaudeCodeProvider ? 'grid-cols-3' : 'grid-cols-2'"
      >
        <button
          class="min-w-0 min-h-8 px-2 py-1.5 text-xs font-medium leading-4 rounded-md transition-all disabled:cursor-not-allowed disabled:opacity-60"
          :class="[
            mode === 'oauth'
              ? 'bg-background text-foreground shadow-sm'
              : 'text-muted-foreground hover:text-foreground',
          ]"
          :disabled="importing || creatingAgentIdentity || cookieAuthorizing"
          @click="switchMode('oauth')"
        >
          {{ authorizationModeLabel }}
        </button>
        <button
          v-if="isClaudeCodeProvider"
          class="min-w-0 min-h-8 px-2 py-1.5 text-xs font-medium leading-4 rounded-md transition-all disabled:cursor-not-allowed disabled:opacity-60"
          :class="mode === 'cookie'
            ? 'bg-background text-foreground shadow-sm'
            : 'text-muted-foreground hover:text-foreground'"
          :disabled="importing || creatingAgentIdentity || cookieAuthorizing"
          @click="switchMode('cookie')"
        >
          {{ t('ui.cookieAuthorization') }}
        </button>
        <button
          class="min-w-0 min-h-8 px-2 py-1.5 text-xs font-medium leading-4 rounded-md transition-all disabled:cursor-not-allowed disabled:opacity-60"
          :class="mode === 'import'
            ? 'bg-background text-foreground shadow-sm'
            : 'text-muted-foreground hover:text-foreground'"
          :disabled="importing || creatingAgentIdentity || cookieAuthorizing"
          @click="switchMode('import')"
        >
          {{ importModeLabel }}
        </button>
        <button
          v-if="isCodexProvider"
          class="min-w-0 min-h-8 px-2 py-1.5 text-xs font-medium leading-4 rounded-md transition-all disabled:cursor-not-allowed disabled:opacity-60"
          :class="mode === 'agent_identity'
            ? 'bg-background text-foreground shadow-sm'
            : 'text-muted-foreground hover:text-foreground'"
          :disabled="importing || creatingAgentIdentity || cookieAuthorizing"
          @click="switchMode('agent_identity')"
        >
          {{ "Agent Identity" }}
        </button>
      </div>

      <!-- Tab 内容：grid 叠放，高度取较高者 -->
      <div class="grid [&>*]:col-start-1 [&>*]:row-start-1">
        <!-- ===== 获取授权 / 设备授权 ===== -->
        <div
          class="space-y-4 transition-opacity duration-150"
          :class="mode === 'oauth' ? 'opacity-100' : 'opacity-0 pointer-events-none'"
          :inert="mode !== 'oauth' ? true : undefined"
          :aria-hidden="mode !== 'oauth'"
        >
          <OAuthDeviceAuthPanel
            v-if="deviceAuthVariant"
            :variant="deviceAuthVariant"
            :device="device"
            :countdown-seconds="deviceCountdown"
            :totp-code="totp.code.value"
            :totp-remaining="totp.remaining.value"
            :kiro-option-disabled="isKiroDeviceAuthOptionDisabled(device.auth_type)"
            @update:field="(field, value) => { device[field] = value }"
            @select-windsurf-login-option="selectWindsurfLoginOption"
            @select-auth-type="selectDeviceAuthType"
            @reset="resetDevice"
            @start="startDeviceAuth"
            @open-verification-url="openDeviceVerificationUrl"
          />

          <!-- 非 Kiro: 原有 OAuth 流程 -->
          <template v-else>
            <div
              v-if="oauth.starting && !oauth.authorization_url"
              class="flex items-center justify-center py-12"
            >
              <div class="text-center">
                <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary mx-auto mb-3" />
                <p class="text-xs text-muted-foreground">
                  {{ t('ui.preparingAuthorization') }}
                </p>
              </div>
            </div>

            <div
              v-else-if="oauth.authorization_url"
              class="flex h-full min-h-0 flex-col gap-4"
            >
              <div class="shrink-0 space-y-2">
                <div class="flex items-center gap-2">
                  <span class="flex items-center justify-center w-4 h-4 rounded-full bg-primary/10 text-primary text-[10px] font-semibold shrink-0">1</span>
                  <span class="text-xs font-medium">{{ t('ui.goToAuthorization') }}</span>
                </div>
                <div class="flex gap-2 pl-6">
                  <Button
                    size="sm"
                    :disabled="oauthBusy"
                    @click="openAuthorizationUrl"
                  >
                    <ExternalLink class="w-3 h-3 mr-1" />
                    {{ t('ui.openc771248') }}
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    :disabled="oauthBusy"
                    @click="copyToClipboard(oauth.authorization_url)"
                  >
                    <Copy class="w-3 h-3 mr-1" />
                    {{ t('common.copy') }}
                  </Button>
                </div>
              </div>

              <div class="flex min-h-0 flex-1 flex-col gap-2">
                <div class="flex shrink-0 items-center gap-2">
                  <span class="flex items-center justify-center w-4 h-4 rounded-full bg-primary/10 text-primary text-[10px] font-semibold shrink-0">2</span>
                  <span class="text-xs font-medium">{{ oauthCallbackLabel }}</span>
                </div>
                <div class="min-h-0 flex-1 pl-6">
                  <Textarea
                    v-model="oauth.callback_url"
                    :disabled="oauthBusy"
                    :placeholder="oauthCallbackPlaceholder"
                    class="h-full min-h-[120px] overflow-y-auto text-xs font-mono break-all !rounded-xl"
                    data-testid="oauth-callback-textarea"
                    spellcheck="false"
                  />
                </div>
              </div>
            </div>
          </template>
        </div>

        <!-- ===== Cookie 授权 ===== -->
        <div
          v-if="isClaudeCodeProvider"
          class="flex flex-col gap-3 justify-center transition-opacity duration-150"
          :class="mode === 'cookie' ? 'opacity-100' : 'opacity-0 pointer-events-none'"
          :inert="mode !== 'cookie' ? true : undefined"
          :aria-hidden="mode !== 'cookie'"
        >
          <label
            class="sr-only"
            for="claude-session-cookie"
          >
            {{ "Claude sessionKey Cookie" }}
          </label>
          <div class="relative">
            <Textarea
              id="claude-session-cookie"
              v-model="cookieInput"
              :disabled="cookieAuthorizing"
              :placeholder="t('ui.pasteOneSessionkeyCookieValueOrCompleteCookieHeaderPer')"
              aria-describedby="claude-session-cookie-status"
              class="h-[200px] min-h-[200px] overflow-y-auto pb-7 text-xs font-mono break-words !rounded-xl"
              data-testid="claude-cookie-input"
              autocomplete="off"
              spellcheck="false"
            />
            <p
              id="claude-session-cookie-status"
              class="pointer-events-none absolute bottom-2 right-3 text-[10px]"
              :class="cookieInputOverLimit ? 'text-destructive' : 'text-muted-foreground'"
              aria-live="polite"
            >
              {{ cookieInputStatusText }}
            </p>
          </div>
        </div>

        <!-- ===== 导入授权 ===== -->
        <div
          class="flex flex-col gap-3 justify-center transition-opacity duration-150"
          :class="mode === 'import' ? 'opacity-100' : 'opacity-0 pointer-events-none'"
          :inert="mode !== 'import' ? true : undefined"
          :aria-hidden="mode !== 'import'"
        >
          <div
            v-if="isWindsurfProvider"
            class="grid grid-cols-2 gap-1.5 rounded-lg border border-border p-0.5 bg-muted/30"
          >
            <button
              v-for="method in ([
                { key: 'email_password', label: '邮箱密码' },
                { key: 'token_json', label: 'Token / JSON' },
              ] as const)"
              :key="method.key"
              class="h-8 text-xs font-medium rounded-md transition-colors"
              :class="windsurfImportMethod === method.key
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground'"
              :disabled="importing"
              @click="setWindsurfImportMethod(method.key)"
            >
              {{ method.label }}
            </button>
          </div>

          <div
            v-if="isWindsurfEmailPasswordImport"
            class="space-y-3"
          >
            <div class="space-y-1.5">
              <label class="text-xs font-medium">{{ t('auth.register.email') }}</label>
              <input
                v-model="windsurfEmail"
                type="email"
                autocomplete="username"
                :disabled="importing"
                placeholder="you@example.com"
                class="w-full h-9 px-2.5 text-xs rounded-md border border-border bg-background focus:outline-none focus:ring-1 focus:ring-ring focus:relative focus:z-10"
                spellcheck="false"
              >
            </div>
            <div class="space-y-1.5">
              <label class="text-xs font-medium">{{ t('auth.register.password') }}</label>
              <input
                v-model="windsurfPassword"
                type="password"
                autocomplete="current-password"
                :disabled="importing"
                :placeholder="t('ui.windsurfPassword')"
                class="w-full h-9 px-2.5 text-xs rounded-md border border-border bg-background focus:outline-none focus:ring-1 focus:ring-ring focus:relative focus:z-10"
              >
            </div>
            <div class="space-y-1.5">
              <label class="text-xs font-medium text-muted-foreground">{{ t('ui.nameOptional') }}</label>
              <input
                v-model="windsurfAccountName"
                type="text"
                autocomplete="off"
                :disabled="importing"
                :placeholder="t('ui.usesEmailIfLeftEmpty')"
                class="w-full h-9 px-2.5 text-xs rounded-md border border-border bg-background focus:outline-none focus:ring-1 focus:ring-ring focus:relative focus:z-10"
                spellcheck="false"
              >
            </div>
          </div>

          <template v-else>
            <JsonImportInput
              v-model="importText"
              :disabled="importing"
              :reset-key="importInputResetKey"
              :drop-title="importDropTitle"
              :drop-hint="importDropHint"
              :manual-placeholder="importManualPlaceholder"
              :manual-description="importManualDescription"
              :paste-toggle-text="importPasteToggleText"
              :file-toggle-text="importFileToggleText"
              textarea-class="min-h-[200px] text-xs font-mono break-all !rounded-xl"
              @error="handleImportInputError"
            />
          </template>

          <div
            v-if="importTask && !isWindsurfEmailPasswordImport"
            class="rounded-xl border border-border bg-muted/20 p-3 space-y-2"
          >
            <div class="flex items-center justify-between text-xs">
              <span class="font-medium">
                {{ getImportTaskStatusText(importTask.status) }}
              </span>
              <span class="font-mono tabular-nums">
                {{ importTask.progress_percent }}%
              </span>
            </div>
            <div class="h-1.5 rounded-full bg-muted overflow-hidden">
              <div
                class="h-full rounded-full bg-primary transition-all duration-300"
                :style="{ width: `${Math.max(0, Math.min(importTask.progress_percent, 100))}%` }"
              />
            </div>
            <div class="flex items-center justify-between text-[11px] text-muted-foreground">
              <span>{{ importProgressText(importTask) }}</span>
              <span>{{ importResultSummaryText(importTask) }}</span>
            </div>
            <p
              v-if="importTaskMessageText"
              class="text-[11px] text-muted-foreground"
            >
              {{ importTaskMessageText }}
            </p>
            <div
              v-if="importTask.error_samples.length > 0"
              class="space-y-1"
            >
              <p class="text-[11px] text-destructive">
                {{ t('ui.recentErrors') }}
              </p>
              <p
                v-for="item in importTask.error_samples.slice(0, 3)"
                :key="`${item.index}-${item.error || item.status}`"
                class="text-[11px] text-destructive/90"
              >
                {{ importErrorSampleText(item) }}
              </p>
            </div>
          </div>
        </div>

        <!-- ===== 创建 Agent Identity ===== -->
        <div
          v-if="isCodexProvider"
          class="flex flex-col gap-3 justify-center transition-opacity duration-150"
          :class="mode === 'agent_identity' ? 'opacity-100' : 'opacity-0 pointer-events-none'"
          :inert="mode !== 'agent_identity' ? true : undefined"
          :aria-hidden="mode !== 'agent_identity'"
        >
          <Textarea
            v-model="agentIdentityInput"
            :disabled="creatingAgentIdentity"
            :placeholder="t('ui.pasteAnAccessTokenOrChatgptAuthSessionJson')"
            class="min-h-[200px] text-xs font-mono break-all !rounded-xl"
            autocomplete="off"
            spellcheck="false"
          />
        </div>
      </div>
    </div>

    <template #footer>
      <Button
        variant="outline"
        @click="handleClose"
      >
        {{ t('common.cancel') }}
      </Button>
      <Button
        v-if="mode === 'oauth' && showAuthorizationMode && !isDeviceBrowserProvider"
        :disabled="!canCompleteOAuth"
        @click="handleCompleteOAuth"
      >
        {{ oauth.completing ? t('auth.register.verifying') : t('ui.verify') }}
      </Button>
      <Button
        v-if="mode === 'oauth' && isManualDeviceCallbackMode"
        :disabled="!canCompleteDeviceAuth"
        @click="completeDeviceAuth"
      >
        {{ device.completing ? t('auth.register.verifying') : t('ui.verify') }}
      </Button>
      <Button
        v-if="mode === 'cookie' && isClaudeCodeProvider"
        :disabled="!canAuthorizeWithCookie"
        @click="handleCookieAuthorize"
      >
        {{ cookieAuthorizeButtonText }}
      </Button>
      <Button
        v-if="mode === 'import'"
        :disabled="!canImport"
        @click="handleImport"
      >
        {{ importButtonText }}
      </Button>
      <Button
        v-if="mode === 'agent_identity'"
        :disabled="!canCreateAgentIdentity"
        @click="handleCreateAgentIdentity"
      >
        {{ creatingAgentIdentity ? t('ui.creating') : t('ui.create') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import { Dialog, Button, Textarea, Popover, PopoverTrigger, PopoverContent } from '@/components/ui'
import {
  UserPlus,
  Copy,
  ExternalLink,
  Globe,
} from 'lucide-vue-next'
import { useToast } from '@/composables/useToast'
import { useClipboard } from '@/composables/useClipboard'
import { useTotp } from '@/composables/useTotp'
import { parseApiError } from '@/utils/errorParser'
import { safeExternalHttpsUrl } from '@/utils/navigationSecurity'
import { useI18n } from '@/i18n'
import {
  startProviderLevelOAuth,
  completeProviderLevelOAuth,
  authorizeProviderWithCookie,
  startProviderCookieAuthorizeTask,
  getProviderCookieAuthorizeTaskStatus,
  importProviderRefreshToken,
  startBatchImportOAuthTask,
  getBatchImportOAuthTaskStatus,
  startDeviceAuthorize,
  pollDeviceAuthorize,
  normalizeBatchImportCredentials,
} from '@/api/endpoints'
import type {
  OAuthBatchImportTaskStatus,
  OAuthBatchImportTaskStartResponse,
  OAuthBatchImportTaskStatusResponse,
} from '@/api/endpoints/provider_oauth'
import ProxyNodeSelect from './ProxyNodeSelect.vue'
import OAuthDeviceAuthPanel from './OAuthDeviceAuthPanel.vue'
import { useProxyNodesStore } from '@/stores/proxy-nodes'
import JsonImportInput from '@/components/common/JsonImportInput.vue'
import {
  parseImportText,
  resolveAgentIdentityAccessToken,
  requiresCodexBatchImport,
} from '@/features/providers/utils/oauthCredentialImport'
import { createInitialDeviceState } from '@/features/providers/utils/oauthDeviceAuth'
import type {
  DeviceAuthState,
  DeviceAuthType,
  WindsurfLoginOption,
} from '@/features/providers/utils/oauthDeviceAuth'

const props = defineProps<{
  open: boolean
  providerId: string | null
  providerType: string | null
}>()

const emit = defineEmits<{
  close: []
  saved: []
}>()

const { t, locale } = useI18n()

const { success, warning, error: showError } = useToast()
const { copyToClipboard } = useClipboard()

const proxyNodesStore = useProxyNodesStore()
const totp = useTotp()

// 代理节点选择
const proxyPopoverOpen = ref(false)
const selectedProxyNodeId = ref('')

/** 获取已选代理节点的显示名称 */
function getSelectedNodeLabel(): string {
  if (!selectedProxyNodeId.value) return ''
  const node = proxyNodesStore.nodes.find(n => n.id === selectedProxyNodeId.value)
  return node ? node.name : `${selectedProxyNodeId.value.slice(0, 8)  }...`
}

function isEnglishLocale(): boolean {
  return locale.value === 'en-US'
}

function localizedApiError(error: unknown, fallback: string): string {
  return parseApiError(error, fallback)
}

// 模式
type DialogMode = 'oauth' | 'cookie' | 'import' | 'agent_identity'
const mode = ref<DialogMode>((props.providerType || '').toLowerCase() === 'grok' ? 'import' : 'oauth')
type WindsurfImportMethod = 'email_password' | 'token_json'

// OAuth 状态
interface OAuthState {
  authorization_url: string
  redirect_uri: string
  instructions: string
  provider_type: string
  callback_url: string
  starting: boolean
  completing: boolean
}

function createInitialOAuthState(): OAuthState {
  return {
    authorization_url: '',
    redirect_uri: '',
    instructions: '',
    provider_type: '',
    callback_url: '',
    starting: false,
    completing: false,
  }
}

const oauth = ref<OAuthState>(createInitialOAuthState())
let oauthInitRequestId = 0
let oauthCompleteRequestId = 0

const BUILDER_ID_START_URL = 'https://view.awsapps.com/start'
const BUILDER_ID_REGION = 'us-east-1'

const device = ref<DeviceAuthState>(createInitialDeviceState())
let deviceAuthRequestId = 0
let devicePollTimer: ReturnType<typeof setTimeout> | null = null
const deviceCountdown = ref(0)
let countdownTimer: ReturnType<typeof setInterval> | null = null

// 导入状态
const importText = ref('')
const importing = ref(false)
const importInputResetKey = ref(0)
const importTask = ref<OAuthBatchImportTaskStatusResponse | null>(null)
let importPollTimer: ReturnType<typeof setTimeout> | null = null
const importPolling = ref(false)
const redundantImportTaskMessagePattern = /^处理中(?:\s+\d+\s*\/\s*\d+)?$/
const windsurfImportMethod = ref<WindsurfImportMethod>('email_password')
const windsurfEmail = ref('')
const windsurfPassword = ref('')
const windsurfAccountName = ref('')
const agentIdentityInput = ref('')
const creatingAgentIdentity = ref(false)
let agentIdentityRequestId = 0
const cookieInput = ref('')
const cookieAuthorizing = ref(false)
let cookieAuthorizeRequestId = 0
const cookieAuthorizeTask = ref<OAuthBatchImportTaskStartResponse | OAuthBatchImportTaskStatusResponse | null>(null)
const cookieAuthorizeSubmittedEntries = ref<string[]>([])
let cookieAuthorizePollTimer: ReturnType<typeof setTimeout> | null = null
const cookieAuthorizePolling = ref(false)
const CLAUDE_COOKIE_BATCH_LIMIT = 20

const isOpen = computed(() => props.open)

const isKiroProvider = computed(() => (props.providerType || '').toLowerCase() === 'kiro')
const isGrokProvider = computed(() => (props.providerType || '').toLowerCase() === 'grok')
const isWindsurfProvider = computed(() => (props.providerType || '').toLowerCase() === 'windsurf')
const isXaiProvider = computed(() => (props.providerType || '').toLowerCase() === 'xai')
const isCodexProvider = computed(() => (props.providerType || '').toLowerCase() === 'codex')
const isClaudeCodeProvider = computed(() => (props.providerType || '').toLowerCase() === 'claude_code')
const isDeviceBrowserProvider = computed(() => isKiroProvider.value || isWindsurfProvider.value || isXaiProvider.value)
const deviceAuthVariant = computed<'windsurf' | 'xai' | 'kiro' | null>(() => {
  if (isWindsurfProvider.value) return 'windsurf'
  if (isXaiProvider.value) return 'xai'
  if (isKiroProvider.value) return 'kiro'
  return null
})
const showAuthorizationMode = computed(() => !isGrokProvider.value)
const defaultMode = computed<DialogMode>(() => (isGrokProvider.value ? 'import' : 'oauth'))

const isSocialDeviceAuth = computed(() =>
  device.value.auth_type === 'google' || device.value.auth_type === 'github'
)

const isKiroSocialManualCallbackMode = computed(() =>
  isKiroProvider.value && isSocialDeviceAuth.value
)

const isManualDeviceCallbackMode = computed(() =>
  isKiroSocialManualCallbackMode.value || isWindsurfProvider.value
)

const isManualDeviceCallbackPending = computed(() =>
  isManualDeviceCallbackMode.value
  && device.value.session_id.length > 0
  && device.value.status === 'pending'
)

const authorizationModeLabel = computed(() => {
  if (isWindsurfProvider.value) return t('ui.browserSignIn')
  if (isXaiProvider.value || isDeviceBrowserProvider.value) return t('ui.deviceAuthorization')
  return t('ui.authorize530a149')
})

const oauthCallbackLabel = computed(() =>
  isClaudeCodeProvider.value ? t('ui.pasteCallbackUrlOrAuthorizationCode') : t('ui.pasteCallbackUrl')
)

const oauthCallbackPlaceholder = computed(() =>
  isClaudeCodeProvider.value
    ? t('ui.pasteTheFullCallbackUrlOrAuthorizationCodeCodeState')
    : 'http://localhost:xxx/callback?code=...'
)

const oauthBusy = computed(() =>
  oauth.value.starting || oauth.value.completing
)

const canCompleteOAuth = computed(() => {
  if (!oauth.value.authorization_url) return false
  if (!oauth.value.callback_url.trim()) return false
  return !oauthBusy.value
})

const canCompleteDeviceAuth = computed(() => {
  if (!isManualDeviceCallbackPending.value) return false
  if (!device.value.callback_url.trim()) return false
  return !device.value.starting && !device.value.completing
})

const canImport = computed(() => {
  if (isWindsurfEmailPasswordImport.value) {
    return windsurfEmail.value.trim().length > 0
      && windsurfPassword.value.trim().length > 0
      && !importing.value
  }
  return importText.value.trim().length > 0 && !importing.value
})

const cookieEntries = computed(() => cookieInput.value
  .split(/\r?\n/)
  .map(value => value.trim())
  .filter(Boolean)
)
const cookieInputOverLimit = computed(() => cookieEntries.value.length > CLAUDE_COOKIE_BATCH_LIMIT)
const canAuthorizeWithCookie = computed(() =>
  isClaudeCodeProvider.value
  && cookieEntries.value.length > 0
  && !cookieInputOverLimit.value
  && !cookieAuthorizing.value
)
const cookieAuthorizeButtonText = computed(() => {
  if (cookieAuthorizing.value) return t('ui.authorizing')
  return cookieEntries.value.length > 1 ? t('ui.batchAuthorize') : t('ui.authorization')
})
const cookieInputStatusText = computed(() => {
  const task = cookieAuthorizeTask.value
  if (cookieAuthorizing.value && task) {
    return t('providers.migration.progressSucceededFailed5fe992', { value: String(task.processed), value_2: String(task.total), value_3: String(task.success), value_4: String(task.failed) })
  }
  const count = cookieEntries.value.length
  if (count === 0) return t('ui.onePerLineUpTo20')
  return t('providers.migration.enteredMaximum6e8e7a', { value: String(count), value_2: String(CLAUDE_COOKIE_BATCH_LIMIT) })
})

const canCreateAgentIdentity = computed(() =>
  isCodexProvider.value
  && agentIdentityInput.value.trim().length > 0
  && !creatingAgentIdentity.value
)

const importModeLabel = computed(() => isGrokProvider.value ? t('ui.importAccounts') : t('ui.importAuthorization'))
const importButtonLabel = computed(() => isGrokProvider.value ? t('ui.importAccounts') : t('ui.import'))
const importDropTitle = computed(() => (
  isGrokProvider.value ? t('ui.dropAGrokAccountFileOrClickToChoose') : t('ui.dropAnAuthorizationFileOrClickToChoose')
))
const importDropHint = computed(() => (
  isGrokProvider.value ? t('ui.supportsJsonTxtMultipleSelectionAndBatchImport') : t('ui.supportsJsonTxtAndMultipleSelection')
))
const importManualPlaceholder = computed(() => {
  if (isGrokProvider.value) {
    return t('ui.pasteGrokSsoSessionTokensOnePerLineOrPaste')
  }
  if (isClaudeCodeProvider.value) {
    return t('ui.pasteAClaudeRefreshTokenOrClaudeCodeCredentialsJson')
  }
  if (isXaiProvider.value) {
    return t('ui.pasteAnXaiApiKeyAccessTokenOrJsonContaining')
  }
  if (isWindsurfProvider.value) {
    return t('ui.pasteAShowAuthTokenTokenApiKeyOrJson')
  }
  return t('ui.pasteARefreshTokenAccessTokenOrAgentIdentityJson')
})
const importManualDescription = computed(() => (
  isGrokProvider.value
    ? t('ui.planTypePoolTierAreSavedAsAccountPlanAnd')
    : ''
))
const importPasteToggleText = computed(() => (
  isGrokProvider.value ? t('ui.orManuallyPasteAGrokToken') : t('ui.orManuallyPasteAToken')
))
const importFileToggleText = computed(() => (
  isGrokProvider.value ? t('ui.orChooseAGrokTokenFileToImport') : t('ui.orChooseAJsonFileToImport')
))
const proxyUsageDescription = computed(() => {
  return isGrokProvider.value
    ? t('ui.importRefreshAndQuotaQueriesUseThisProxy')
    : t('ui.authorizationRefreshAndQuotaQueriesUseThisProxy')
})
const isWindsurfEmailPasswordImport = computed(() =>
  isWindsurfProvider.value && windsurfImportMethod.value === 'email_password'
)
const importButtonText = computed(() => {
  if (importing.value) {
    return importTask.value && !isWindsurfEmailPasswordImport.value
      ? (t('providers.migration.importing91b33f', { value: String(importTask.value.progress_percent) }))
      : t('ui.importingc70b341')
  }
  return isWindsurfEmailPasswordImport.value ? t('ui.signInAndImport') : importButtonLabel.value
})

const importTaskMessageText = computed(() => {
  const message = importTask.value?.message?.trim()
  if (!message) return ''
  // 后端进度 message 已由“进度 x/y”展示，避免在导入中重复显示“处理中 x/y”。
  return redundantImportTaskMessagePattern.test(message) ? '' : message
})

function importProgressText(task: OAuthBatchImportTaskStatusResponse): string {
  return t('providers.migration.progress9b408a', { value: String(task.processed), value_2: String(task.total) })
}

function importResultSummaryText(task: OAuthBatchImportTaskStatusResponse): string {
  return t('providers.migration.succeededFailed89649e', { value: String(task.success), value_2: String(task.failed) })
}

function importErrorSampleText(item: OAuthBatchImportTaskStatusResponse['error_samples'][number]): string {
  return `#${item.index + 1} ${(item.error || t('ui.importFailed'))}`
}

function stopImportPolling() {
  if (importPollTimer) {
    clearTimeout(importPollTimer)
    importPollTimer = null
  }
  importPolling.value = false
}

function getImportTaskStatusText(status: OAuthBatchImportTaskStatus): string {
  switch (status) {
    case 'submitted':
      return t('ui.taskSubmitted')
    case 'processing':
      return t('ui.importing5c27a92')
    case 'completed':
      return t('ui.importCompleted613fef')
    case 'failed':
      return t('ui.importFailed')
    default:
      return t('ui.processing694b71b')
  }
}

function getOAuthSuccessMessage(
  action: 'authorize' | 'import' | 'create',
  options?: { email?: string | null; replaced?: boolean }
): string {
  const email = typeof options?.email === 'string' ? options.email.trim() : ''
  const replaced = options?.replaced === true
  const actionKeys = { authorize: 'ui.authorization', import: 'ui.import', create: 'ui.create' } as const
  const actionText = t(actionKeys[action])

  if (email) {
    return replaced
      ? t('providers.migration.succeededReplacedExistingAccountf3b50a', { value: actionText, value_2: String(email) })
      : t('providers.migration.succeeded274d3e', { value: actionText, value_2: String(email) })
  }
  return replaced
    ? t('providers.migration.succeededReplacedExistingAccountb1af2b', { value: actionText })
    : t('providers.migration.succeededAccountAddeddaec8b', { value: actionText })
}

function getBatchImportSuccessMessage(task: OAuthBatchImportTaskStatusResponse): string {
  const replacedCount = Math.max(task.replaced_count ?? 0, 0)
  const createdCount = Math.max(task.created_count ?? task.success - replacedCount, 0)
  const parts: string[] = []

  if (createdCount > 0) {
    parts.push(t('providers.migration.addedc4018b', { value: String(createdCount) }))
  }
  if (replacedCount > 0) {
    parts.push(t('providers.migration.replaced89153e', { value: String(replacedCount) }))
  }
  if (task.failed > 0) {
    parts.push(t('providers.migration.failed281422', { value: String(task.failed) }))
  }

  if (parts.length === 0) {
    return task.failed > 0 ? t('providers.migration.batchImportCompleteFailed8b59e0', { value: String(task.failed) }) : t('ui.batchImportComplete')
  }
  if (task.failed === 0 && createdCount > 0 && replacedCount === 0) {
    return t('providers.migration.batchImportSucceededAccountsAddedc39555', { value: String(createdCount) })
  }
  if (task.failed === 0 && createdCount === 0 && replacedCount > 0) {
    return t('providers.migration.batchImportSucceededExistingAccountsd6acfc', { value: String(replacedCount) })
  }

  const prefix = task.failed > 0
    ? t('ui.batchImportComplete')
    : t('ui.batchImportSucceeded')
  return isEnglishLocale()
    ? `${prefix}: ${parts.join(', ')}`
    : `${prefix}：${parts.join('，')}`
}

function scheduleImportPoll(taskId: string, delayMs = 1200) {
  stopImportPolling()
  importPollTimer = setTimeout(() => {
    void pollImportTaskStatus(taskId)
  }, delayMs)
}

async function pollImportTaskStatus(taskId: string) {
  if (!props.providerId || importPolling.value) return

  importPolling.value = true
  try {
    const task = await getBatchImportOAuthTaskStatus(props.providerId, taskId)
    importTask.value = task

    if (task.status === 'completed') {
      stopImportPolling()
      importing.value = false
      if (task.success > 0) {
        success(getBatchImportSuccessMessage(task))
        emit('saved')
        handleClose()
      } else {
        showError((task.error || t('ui.batchImportFailed')), t('ui.importFailed'))
      }
      return
    }

    if (task.status === 'failed') {
      stopImportPolling()
      importing.value = false
      showError(((task.error || task.message) || t('ui.batchImportFailed')), t('ui.importFailed'))
      return
    }

    scheduleImportPoll(taskId)
  } catch {
    if (importing.value) {
      scheduleImportPoll(taskId, 2000)
    }
  } finally {
    importPolling.value = false
  }
}

function stopDevicePolling() {
  if (devicePollTimer) {
    clearTimeout(devicePollTimer)
    devicePollTimer = null
  }
  if (countdownTimer) {
    clearInterval(countdownTimer)
    countdownTimer = null
  }
}

function resetDeviceRuntimeState() {
  stopDevicePolling()
  totp.stop()
  device.value.callback_url = ''
  device.value.callback_required = false
  device.value.starting = false
  device.value.completing = false
  device.value.session_id = ''
  device.value.user_code = ''
  device.value.verification_uri = ''
  device.value.verification_uri_complete = ''
  device.value.expires_at = 0
  device.value.interval = 5
  device.value.status = 'idle'
  device.value.error = ''
}

function isKiroDeviceAuthOptionDisabled(_authType: DeviceAuthType): boolean {
  if (!isKiroProvider.value) return false
  if (device.value.starting) {
    return !isSocialDeviceAuth.value
  }
  if (!device.value.session_id) return false
  if (isSocialDeviceAuth.value && device.value.status === 'pending') {
    return false
  }
  return true
}

function selectWindsurfLoginOption(loginOption: WindsurfLoginOption) {
  if (!isWindsurfProvider.value) return
  if (device.value.auth_type === loginOption && device.value.session_id && device.value.status === 'pending') return
  deviceAuthRequestId += 1
  resetDeviceRuntimeState()
  device.value.auth_type = loginOption
}

function selectDeviceAuthType(authType: DeviceAuthType) {
  if (device.value.auth_type === authType) return
  if (isKiroDeviceAuthOptionDisabled(authType)) return

  deviceAuthRequestId += 1
  resetDeviceRuntimeState()
  device.value.auth_type = authType
  if (authType === 'google' || authType === 'github') {
    void ensureKiroSocialDeviceAuth()
  }
}

function resetDevice() {
  deviceAuthRequestId += 1
  stopDevicePolling()
  totp.stop()
  const { auth_type, start_url, region, totp_secret } = device.value
  device.value = createInitialDeviceState()
  device.value.auth_type = isXaiProvider.value
    ? 'device'
    : isWindsurfProvider.value
      ? (auth_type === 'google' || auth_type === 'github' ? auth_type : 'default')
      : auth_type
  device.value.start_url = start_url
  device.value.region = region
  device.value.totp_secret = totp_secret
  if (isXaiProvider.value) {
    void ensureXaiDeviceAuth()
  } else if (!isWindsurfProvider.value && (device.value.auth_type === 'google' || device.value.auth_type === 'github')) {
    void ensureKiroSocialDeviceAuth()
  }
}

function resetForm() {
  oauthInitRequestId += 1
  oauthCompleteRequestId += 1
  deviceAuthRequestId += 1
  agentIdentityRequestId += 1
  cookieAuthorizeRequestId += 1
  oauth.value = createInitialOAuthState()
  stopImportPolling()
  stopCookieAuthorizePolling()
  stopDevicePolling()
  totp.stop()
  device.value = createInitialDeviceState()
  if (isWindsurfProvider.value) {
    device.value.auth_type = 'default'
  } else if (isXaiProvider.value) {
    device.value.auth_type = 'device'
  }
  importText.value = ''
  importing.value = false
  importTask.value = null
  importInputResetKey.value += 1
  windsurfImportMethod.value = 'email_password'
  windsurfEmail.value = ''
  windsurfPassword.value = ''
  windsurfAccountName.value = ''
  agentIdentityInput.value = ''
  creatingAgentIdentity.value = false
  cookieInput.value = ''
  cookieAuthorizing.value = false
  cookieAuthorizeTask.value = null
  cookieAuthorizeSubmittedEntries.value = []
  proxyPopoverOpen.value = false
  selectedProxyNodeId.value = ''
  mode.value = defaultMode.value
}

function switchMode(newMode: DialogMode) {
  if (mode.value === newMode) return
  if (newMode === 'oauth' && !showAuthorizationMode.value) return
  if (newMode === 'cookie' && !isClaudeCodeProvider.value) return
  if (newMode === 'agent_identity' && !isCodexProvider.value) return
  if (importing.value || creatingAgentIdentity.value || cookieAuthorizing.value) return

  mode.value = newMode
  if (newMode === 'oauth') {
    if (isKiroProvider.value) {
      void ensureKiroSocialDeviceAuth()
    } else if (isXaiProvider.value) {
      void ensureXaiDeviceAuth()
    } else if (!oauth.value.authorization_url && !oauth.value.starting) {
      initOAuth()
    }
  }
}

function handleDialogUpdate(value: boolean) {
  if (!value) {
    handleClose()
  }
}

function handleClose() {
  resetForm()
  emit('close')
}

function openAuthorizationUrl() {
  const url = oauth.value.authorization_url
  if (!url) return
  const safeUrl = safeExternalHttpsUrl(url)
  if (!safeUrl) {
    showError(t('ui.theOauthServiceReturnedAnUnsafeAuthorizationUrl'))
    return
  }
  window.open(safeUrl, '_blank', 'noopener,noreferrer')
}

async function initOAuth() {
  if (!props.providerId) return
  if (!showAuthorizationMode.value) return
  if (isDeviceBrowserProvider.value) return
  if (oauth.value.starting) return

  const requestId = ++oauthInitRequestId
  oauth.value.starting = true
  try {
    const resp = await startProviderLevelOAuth(props.providerId)
    if (requestId !== oauthInitRequestId) return
    oauth.value.authorization_url = resp.authorization_url
    oauth.value.redirect_uri = resp.redirect_uri
    oauth.value.instructions = resp.instructions
    oauth.value.provider_type = resp.provider_type
  } catch (err: unknown) {
    if (requestId !== oauthInitRequestId) return
    const errorMessage = localizedApiError(err, t('ui.failedToInitializeAuthorization'))
    showError(errorMessage, t('ui.error'))
    mode.value = 'import'
  } finally {
    if (requestId === oauthInitRequestId) {
      oauth.value.starting = false
    }
  }
}

async function handleCompleteOAuth() {
  if (oauth.value.completing) return
  if (!canCompleteOAuth.value || !props.providerId) return
  const requestId = ++oauthCompleteRequestId
  oauth.value.completing = true
  try {
    const result = await completeProviderLevelOAuth(props.providerId, {
      callback_url: oauth.value.callback_url.trim(),
      proxy_node_id: selectedProxyNodeId.value || undefined,
    })
    if (requestId !== oauthCompleteRequestId) return
    success(getOAuthSuccessMessage('authorize', result))
    emit('saved')
    handleClose()
  } catch (err: unknown) {
    if (requestId !== oauthCompleteRequestId) return
    const errorMessage = localizedApiError(err, t('ui.failedToCompleteAuthorization'))
    showError(errorMessage, t('ui.error'))
  } finally {
    if (requestId === oauthCompleteRequestId) {
      oauth.value.completing = false
    }
  }
}

async function handleCookieAuthorize() {
  if (!canAuthorizeWithCookie.value || !props.providerId) return

  const entries = [...cookieEntries.value]
  const requestId = ++cookieAuthorizeRequestId
  cookieAuthorizing.value = true
  try {
    if (entries.length === 1) {
      const result = await authorizeProviderWithCookie(props.providerId, {
        cookie: entries[0],
        proxy_node_id: selectedProxyNodeId.value || undefined,
      })
      if (requestId !== cookieAuthorizeRequestId) return

      success(getOAuthSuccessMessage('authorize', result))
      emit('saved')
      handleClose()
      return
    }

    const task = await startProviderCookieAuthorizeTask(props.providerId, {
      cookies: entries,
      proxy_node_id: selectedProxyNodeId.value || undefined,
    })
    if (requestId !== cookieAuthorizeRequestId) return

    cookieAuthorizeTask.value = task
    cookieAuthorizeSubmittedEntries.value = entries
    scheduleCookieAuthorizePoll(task.task_id, requestId, 0)
  } catch (err: unknown) {
    if (requestId !== cookieAuthorizeRequestId) return
    const errorMessage = localizedApiError(err, t('ui.cookieAuthorizationFailed'))
    showError(errorMessage, t('ui.error'))
  } finally {
    if (requestId === cookieAuthorizeRequestId && !cookieAuthorizeTask.value) {
      cookieAuthorizing.value = false
    }
  }
}

function stopCookieAuthorizePolling() {
  if (cookieAuthorizePollTimer) {
    clearTimeout(cookieAuthorizePollTimer)
    cookieAuthorizePollTimer = null
  }
  cookieAuthorizePolling.value = false
}

function scheduleCookieAuthorizePoll(taskId: string, requestId: number, delayMs = 1200) {
  stopCookieAuthorizePolling()
  cookieAuthorizePollTimer = setTimeout(() => {
    void pollCookieAuthorizeTaskStatus(taskId, requestId)
  }, delayMs)
}

function cookieAuthorizeBatchSummary(task: OAuthBatchImportTaskStatusResponse): string {
  const replaced = Math.max(task.replaced_count ?? 0, 0)
  const created = Math.max(task.created_count ?? task.success - replaced, 0)
  const successDetail = t('providers.migration.succeededAddedReplacedd6df17', { value: String(task.success), value_2: String(created), value_3: String(replaced) })

  return task.failed > 0
    ? t('providers.migration.batchAuthorizationCompleteFailed5cf5e1', { value: String(successDetail), value_2: String(task.failed) })
    : t('providers.migration.batchAuthorizationSucceededdc77ec', { value: String(successDetail) })
}

function cookieAuthorizeFailureReasons(task: OAuthBatchImportTaskStatusResponse): string[] {
  const reasons: string[] = []
  const seenIndexes = new Set<number>()
  for (const item of task.error_samples) {
    const index = item.index
    const detail = item.error?.trim()
    if (
      item.status !== 'error'
      || !Number.isInteger(index)
      || index < 0
      || index >= task.total
      || seenIndexes.has(index)
      || !detail
      || detail.length > 512
    ) {
      continue
    }

    const normalized = detail.toLowerCase()
    if (
      normalized.includes('sessionkey')
      || normalized.includes('sk-ant-')
      || normalized.includes('cookie:')
    ) {
      continue
    }

    seenIndexes.add(index)
    reasons.push(`#${index + 1} ${detail}`)
    if (reasons.length === 2) break
  }
  return reasons
}

function cookieAuthorizeBatchResultMessage(task: OAuthBatchImportTaskStatusResponse): string {
  const summary = cookieAuthorizeBatchSummary(task)
  const reasons = cookieAuthorizeFailureReasons(task)
  if (reasons.length === 0) return summary
  return `${summary}${isEnglishLocale() ? '; ' : '；'}${reasons.join(isEnglishLocale() ? '; ' : '；')}`
}

function failedCookieAuthorizeEntries(
  task: OAuthBatchImportTaskStatusResponse,
  entries: string[],
): string[] {
  const failedIndexes = task.error_samples
    .filter(item => item.status === 'error' && Number.isInteger(item.index))
    .map(item => item.index)
    .filter(index => index >= 0 && index < entries.length)

  // Keep every original line if the response is incomplete, so credentials are never discarded.
  if (new Set(failedIndexes).size !== task.failed) return entries
  return failedIndexes.map(index => entries[index])
}

function handleCookieAuthorizeBatchResult(
  task: OAuthBatchImportTaskStatusResponse,
  entries: string[],
) {
  const message = cookieAuthorizeBatchResultMessage(task)
  cookieAuthorizeTask.value = null
  cookieAuthorizeSubmittedEntries.value = []

  if (task.failed === 0) {
    success(message)
    emit('saved')
    handleClose()
    return
  }

  if (task.success > 0) {
    cookieInput.value = failedCookieAuthorizeEntries(task, entries).join('\n')
    emit('saved')
    warning(message, t('ui.batchAuthorize'))
    return
  }

  showError(message, t('ui.error'))
}

async function pollCookieAuthorizeTaskStatus(taskId: string, requestId: number) {
  if (!props.providerId || cookieAuthorizePolling.value || requestId !== cookieAuthorizeRequestId) return

  cookieAuthorizePolling.value = true
  try {
    const task = await getProviderCookieAuthorizeTaskStatus(props.providerId, taskId)
    if (requestId !== cookieAuthorizeRequestId) return
    cookieAuthorizeTask.value = task

    if (task.status === 'completed') {
      stopCookieAuthorizePolling()
      cookieAuthorizing.value = false
      handleCookieAuthorizeBatchResult(task, [...cookieAuthorizeSubmittedEntries.value])
      return
    }

    if (task.status === 'failed') {
      stopCookieAuthorizePolling()
      cookieAuthorizing.value = false
      cookieAuthorizeTask.value = null
      cookieAuthorizeSubmittedEntries.value = []
      showError(
        ((task.error || task.message) || t('ui.cookieAuthorizationFailed')),
        t('ui.cookieAuthorizationFailed'),
      )
      return
    }

    scheduleCookieAuthorizePoll(taskId, requestId)
  } catch {
    if (cookieAuthorizing.value && requestId === cookieAuthorizeRequestId) {
      scheduleCookieAuthorizePoll(taskId, requestId, 2000)
    }
  } finally {
    if (requestId === cookieAuthorizeRequestId) {
      cookieAuthorizePolling.value = false
    }
  }
}

function handleImportInputError(payload: { message: string; title?: string }) {
  showError(payload.message, payload.title ? payload.title : undefined)
}

function setWindsurfImportMethod(method: WindsurfImportMethod) {
  if (!isWindsurfProvider.value || importing.value) return
  windsurfImportMethod.value = method
  importTask.value = null
}

async function handleWindsurfEmailPasswordImport() {
  if (!props.providerId) return

  const email = windsurfEmail.value.trim()
  const password = windsurfPassword.value.trim()
  if (!email || !password) {
    showError(t('auth.login.required'), t('ui.formatError'))
    return
  }

  importing.value = true
  try {
    const result = await importProviderRefreshToken(props.providerId, {
      email,
      password,
      name: windsurfAccountName.value.trim() || email,
      proxy_node_id: selectedProxyNodeId.value || undefined,
    })
    success(getOAuthSuccessMessage('import', result))
    emit('saved')
    handleClose()
  } catch (err: unknown) {
    const errorMessage = localizedApiError(err, t('ui.importFailed'))
    showError(errorMessage, t('ui.error'))
  } finally {
    importing.value = false
  }
}

async function handleImport() {
  if (!canImport.value || !props.providerId) return
  if (isWindsurfEmailPasswordImport.value) {
    await handleWindsurfEmailPasswordImport()
    return
  }

  const inputText = importText.value.trim()
  if (!inputText) {
    showError(t('ui.enterCredentialData'), t('ui.formatError'))
    return
  }

  const normalizedCredentials = normalizeBatchImportCredentials(inputText)
  if (!normalizedCredentials.ok) {
    showError(normalizedCredentials.message, t('ui.formatError'))
    return
  }

  importing.value = true
  let keepImporting = false
  try {
    const proxyNodeId = selectedProxyNodeId.value || undefined
    // Kiro, Codex Agent Identity, and sub2api exports need their full JSON on the batch path.
    if (
      isKiroProvider.value
      || normalizedCredentials.isBatch
      || requiresCodexBatchImport(normalizedCredentials.credentials, isCodexProvider.value)
    ) {
      const task = await startBatchImportOAuthTask(props.providerId, normalizedCredentials.credentials, proxyNodeId)
      importTask.value = {
        task_id: task.task_id,
        provider_id: props.providerId,
        provider_type: props.providerType || '',
        status: task.status,
        total: task.total,
        processed: task.processed,
        success: task.success,
        failed: task.failed,
        created_count: task.created_count ?? 0,
        replaced_count: task.replaced_count ?? 0,
        progress_percent: task.progress_percent,
        message: task.message || null,
        error: null,
        error_samples: [],
        created_at: Math.floor(Date.now() / 1000),
        started_at: null,
        finished_at: null,
        updated_at: Math.floor(Date.now() / 1000),
      }
      keepImporting = true
      scheduleImportPoll(task.task_id, 400)
    } else {
      // 单条导入
      const parsed = parseImportText(normalizedCredentials.credentials, props.providerType)
      if (!parsed) {
        showError(t('ui.unableToParseTheInputCheckTheFormat'), t('ui.formatError'))
        return
      }
      const result = await importProviderRefreshToken(props.providerId, {
        ...parsed,
        proxy_node_id: proxyNodeId,
      })
      success(getOAuthSuccessMessage('import', result))
      emit('saved')
      handleClose()
    }
  } catch (err: unknown) {
    const errorMessage = localizedApiError(err, t('ui.importFailed'))
    showError(errorMessage, t('ui.error'))
  } finally {
    if (!keepImporting) {
      importing.value = false
    }
  }
}

async function handleCreateAgentIdentity() {
  if (!canCreateAgentIdentity.value || !props.providerId) return

  const resolvedInput = resolveAgentIdentityAccessToken(agentIdentityInput.value)
  if (!resolvedInput.ok) {
    showError(resolvedInput.message, t('ui.formatError'))
    return
  }
  const requestId = ++agentIdentityRequestId
  creatingAgentIdentity.value = true
  try {
    const result = await importProviderRefreshToken(props.providerId, {
      access_token: resolvedInput.accessToken,
      create_agent_identity: true,
      proxy_node_id: selectedProxyNodeId.value || undefined,
    })
    if (requestId !== agentIdentityRequestId) return

    if (result.task_ready === false) {
      warning(t('ui.agentIdentityWasSavedItsTaskWillInitializeInThe'), t('ui.saved1bd91a7'))
    } else {
      success(getOAuthSuccessMessage('create', result))
    }
    emit('saved')
    handleClose()
  } catch (err: unknown) {
    if (requestId !== agentIdentityRequestId) return
    const errorMessage = localizedApiError(err, t('ui.failedToCreateAgentIdentity'))
    showError(errorMessage, t('ui.error'))
  } finally {
    if (requestId === agentIdentityRequestId) {
      creatingAgentIdentity.value = false
    }
  }
}

// ==== 设备授权 ====

function openDeviceVerificationUrl() {
  const url = device.value.verification_uri_complete || device.value.verification_uri
  if (!url) return
  const safeUrl = safeExternalHttpsUrl(url)
  if (!safeUrl) {
    showError(t('ui.theOauthServiceReturnedAnUnsafeDeviceVerificationUrl'))
    return
  }
  window.open(safeUrl, '_blank', 'noopener,noreferrer')
}

function startCountdown() {
  if (countdownTimer) clearInterval(countdownTimer)
  deviceCountdown.value = Math.max(0, Math.round((device.value.expires_at - Date.now()) / 1000))
  countdownTimer = setInterval(() => {
    deviceCountdown.value = Math.max(0, Math.round((device.value.expires_at - Date.now()) / 1000))
    if (deviceCountdown.value <= 0 && countdownTimer) {
      clearInterval(countdownTimer)
      countdownTimer = null
    }
  }, 1000)
}

async function startDeviceAuth() {
  if (!props.providerId) return
  if (device.value.starting) return
  const requestId = ++deviceAuthRequestId
  const requestedAuthType = device.value.auth_type
  device.value.callback_url = ''
  device.value.callback_required = false
  device.value.session_id = ''
  device.value.user_code = ''
  device.value.verification_uri = ''
  device.value.verification_uri_complete = ''
  device.value.status = 'idle'
  device.value.starting = true
  device.value.error = ''
  try {
    const isWindsurf = isWindsurfProvider.value
    const isXai = isXaiProvider.value
    const isBuilderID = requestedAuthType === 'builder_id'
    const isSocial = !isXai && (requestedAuthType === 'google' || requestedAuthType === 'github')
    const windsurfLoginOption: WindsurfLoginOption = isSocial ? requestedAuthType : 'default'
    const authTypeForRequest = isWindsurf
      ? 'browser'
      : isXai
        ? 'device'
        : (requestedAuthType === 'default' ? 'google' : requestedAuthType)
    const resp = await startDeviceAuthorize(props.providerId, {
      auth_type: authTypeForRequest,
      login_option: isWindsurf ? windsurfLoginOption : undefined,
      start_url: (isWindsurf || isXai) ? undefined : (isBuilderID ? BUILDER_ID_START_URL : (isSocial ? undefined : (device.value.start_url.trim() || undefined))),
      region: (isWindsurf || isXai) ? undefined : (isBuilderID || isSocial ? BUILDER_ID_REGION : (device.value.region.trim() || undefined)),
      proxy_node_id: selectedProxyNodeId.value || undefined,
    })
    if (requestId !== deviceAuthRequestId || device.value.auth_type !== requestedAuthType) return
    device.value.session_id = resp.session_id
    device.value.user_code = resp.user_code
    device.value.verification_uri = resp.verification_uri
    device.value.verification_uri_complete = resp.verification_uri_complete
    device.value.expires_at = Date.now() + resp.expires_in * 1000
    device.value.interval = resp.interval || 5
    device.value.callback_required = resp.callback_required === true || isSocial || isWindsurf
    device.value.status = 'pending'
    startCountdown()
    if (!device.value.callback_required) {
      scheduleDevicePoll()
    }
    // 如果配置了 TOTP secret，启动验证码生成
    if (!device.value.callback_required && device.value.totp_secret.trim()) {
      totp.start(device.value.totp_secret.trim())
    }
  } catch (err: unknown) {
    if (requestId !== deviceAuthRequestId || device.value.auth_type !== requestedAuthType) return
    const errorMessage = localizedApiError(err, t('ui.failedToStartDeviceAuthorization'))
    showError(errorMessage, t('ui.error'))
    device.value.status = 'error'
    device.value.error = errorMessage
  } finally {
    if (requestId === deviceAuthRequestId && device.value.auth_type === requestedAuthType) {
      device.value.starting = false
    }
  }
}

async function ensureKiroSocialDeviceAuth() {
  if (!props.open || !props.providerId || !isKiroProvider.value || !isSocialDeviceAuth.value) return
  if (device.value.starting) return
  if (device.value.session_id && device.value.status === 'pending') return
  await startDeviceAuth()
}

async function ensureXaiDeviceAuth() {
  if (!props.open || !props.providerId || !isXaiProvider.value) return
  if (device.value.starting) return
  if (device.value.session_id && (device.value.status === 'pending' || device.value.status === 'authorized')) return
  await startDeviceAuth()
}

function scheduleDevicePoll() {
  if (devicePollTimer) clearTimeout(devicePollTimer)
  devicePollTimer = setTimeout(() => pollDevice(), device.value.interval * 1000)
}

async function completeDeviceAuth() {
  if (device.value.completing || !canCompleteDeviceAuth.value) return
  device.value.completing = true
  try {
    await pollDevice(true)
  } finally {
    device.value.completing = false
  }
}

function normalizeWindsurfSubmittedCredential(value: string): { callback_url?: string, token?: string } {
  const trimmed = value.trim()
  if (!trimmed) return {}
  if (/^https?:\/\//i.test(trimmed)) {
    return { callback_url: trimmed }
  }

  const query = trimmed.replace(/^[?#&]+/, '')
  const params = new URLSearchParams(query)
  const hasTokenParam = ['token', 'auth_token', 'access_token'].some(key => params.has(key))
  const hasStateParam = params.has('state')
  if (hasTokenParam && hasStateParam) {
    return { callback_url: `https://windsurf.com/show-auth-token?${query}` }
  }
  if (hasTokenParam) {
    return { token: params.get('token') || params.get('auth_token') || params.get('access_token') || trimmed }
  }

  return { token: trimmed }
}

async function pollDevice(withCallback = false) {
  if (!props.providerId || !device.value.session_id || device.value.status !== 'pending') return

  try {
    const submittedCredential = withCallback ? device.value.callback_url.trim() : ''
    const windsurfSubmitted = isWindsurfProvider.value
      ? normalizeWindsurfSubmittedCredential(submittedCredential)
      : {}
    const result = await pollDeviceAuthorize(props.providerId, {
      session_id: device.value.session_id,
      callback_url: withCallback ? (windsurfSubmitted.callback_url || (!isWindsurfProvider.value ? submittedCredential : undefined)) : undefined,
      token: withCallback ? windsurfSubmitted.token : undefined,
    })

    switch (result.status) {
      case 'authorized':
        stopDevicePolling()
        totp.stop()
        device.value.status = 'authorized'
        success(getOAuthSuccessMessage('authorize', result))
        emit('saved')
        handleClose()
        return
      case 'pending':
        if (!device.value.callback_required) {
          scheduleDevicePoll()
        }
        return
      case 'slow_down':
        device.value.interval = Math.min(device.value.interval + 5, 30)
        scheduleDevicePoll()
        return
      case 'expired':
        stopDevicePolling()
        device.value.status = 'expired'
        device.value.error = result.error || t('ui.deviceCodeExpired')
        return
      case 'error':
        stopDevicePolling()
        device.value.status = 'error'
        device.value.error = result.error || t('ui.authorizationFailed')
        return
    }
  } catch (err: unknown) {
    if (withCallback) {
      const errorMessage = localizedApiError(err, t('ui.failedToCompleteAuthorization'))
      showError(errorMessage, t('ui.error'))
    }
    // 网络错误等，继续轮询
    if (!withCallback && !device.value.callback_required) {
      scheduleDevicePoll()
    }
  }
}

onBeforeUnmount(() => {
  stopImportPolling()
  stopCookieAuthorizePolling()
  stopDevicePolling()
})

watch(
  () => props.open,
  (newOpen) => {
    if (newOpen) {
      proxyNodesStore.ensureLoaded()
      mode.value = defaultMode.value
      if (!showAuthorizationMode.value) {
        return
      }
      if (isWindsurfProvider.value) {
        device.value.auth_type = 'default'
      } else if (isXaiProvider.value) {
        device.value.auth_type = 'device'
        void ensureXaiDeviceAuth()
      } else if (isKiroProvider.value) {
        void ensureKiroSocialDeviceAuth()
      } else {
        initOAuth()
      }
    } else {
      resetForm()
    }
  },
  { immediate: true },
)

watch(
  () => [props.open, props.providerId, props.providerType] as const,
  () => {
    if (props.open && !showAuthorizationMode.value) {
      mode.value = 'import'
      return
    }
    if (props.open && mode.value === 'agent_identity' && !isCodexProvider.value) {
      mode.value = defaultMode.value
    }
    if (props.open && mode.value === 'cookie' && !isClaudeCodeProvider.value) {
      mode.value = defaultMode.value
    }
    if (props.open && isWindsurfProvider.value && mode.value === 'oauth') {
      device.value.auth_type = ['default', 'google', 'github'].includes(device.value.auth_type)
        ? device.value.auth_type
        : 'default'
    } else if (props.open && isXaiProvider.value && mode.value === 'oauth') {
      device.value.auth_type = 'device'
      void ensureXaiDeviceAuth()
    } else if (props.open && isKiroProvider.value && mode.value === 'oauth') {
      void ensureKiroSocialDeviceAuth()
    }
  },
)
</script>
