<template>
  <!-- Windsurf: 浏览器 session/poll 授权 -->
  <template v-if="variant === 'windsurf'">
    <div class="space-y-4">
      <div class="grid grid-cols-3 gap-1.5">
        <button
          v-for="opt in ([
            { key: 'default', label: '默认' },
            { key: 'google', label: 'Google' },
            { key: 'github', label: 'GitHub' },
          ] as const)"
          :key="opt.key"
          class="h-8 text-xs font-medium rounded-md border transition-colors"
          :class="device.auth_type === opt.key
            ? 'border-primary bg-primary/5 text-foreground'
            : 'border-border text-muted-foreground hover:text-foreground hover:border-foreground/20'"
          @click="$emit('selectWindsurfLoginOption', opt.key)"
        >
          {{ opt.label }}
        </button>
      </div>

      <div
        v-if="device.status === 'error' || device.status === 'expired'"
        class="rounded-xl border border-destructive/20 bg-destructive/5 p-5"
      >
        <div class="flex flex-col items-center text-center space-y-3">
          <div class="w-10 h-10 rounded-full bg-destructive/10 flex items-center justify-center">
            <AlertCircle class="w-5 h-5 text-destructive" />
          </div>
          <div class="space-y-1">
            <p class="text-sm font-medium text-destructive">
              {{ device.status === 'expired' ? t('ui.authorizationExpired') : t('ui.authorizationFailed') }}
            </p>
            <p class="text-xs text-muted-foreground">
              {{ (device.error || t('ui.pleaseTryAgain')) }}
            </p>
          </div>
          <Button
            size="sm"
            variant="outline"
            @click="$emit('reset')"
          >
            {{ t('ui.startOver') }}
          </Button>
        </div>
      </div>

      <div
        v-else-if="device.starting && !device.session_id"
        class="flex items-center justify-center py-12"
      >
        <div class="text-center">
          <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary mx-auto mb-3" />
          <p class="text-xs text-muted-foreground">
            {{ t('ui.preparingSignIn') }}
          </p>
        </div>
      </div>

      <div
        v-else
        class="space-y-4"
      >
        <div class="space-y-2">
          <div class="flex items-center gap-2">
            <span class="flex items-center justify-center w-4 h-4 rounded-full bg-primary/10 text-primary text-[10px] font-semibold shrink-0">1</span>
            <span class="text-xs font-medium">{{ t('ui.goToSignIn') }}</span>
          </div>
          <div class="flex gap-2 pl-6">
            <Button
              size="sm"
              :disabled="device.starting || device.completing || !device.verification_uri_complete"
              @click="$emit('openVerificationUrl')"
            >
              <ExternalLink class="w-3 h-3 mr-1" />
              {{ t('ui.openc771248') }}
            </Button>
            <Button
              size="sm"
              variant="outline"
              :disabled="device.starting || device.completing || !device.verification_uri_complete"
              @click="copyToClipboard(device.verification_uri_complete)"
            >
              <Copy class="w-3 h-3 mr-1" />
              {{ t('common.copy') }}
            </Button>
            <Button
              v-if="!device.session_id"
              size="sm"
              variant="outline"
              :disabled="device.starting"
              @click="$emit('start')"
            >
              {{ t('ui.start') }}
            </Button>
          </div>
        </div>

        <div class="space-y-2">
          <div class="flex items-center gap-2">
            <span class="flex items-center justify-center w-4 h-4 rounded-full bg-primary/10 text-primary text-[10px] font-semibold shrink-0">2</span>
            <span class="text-xs font-medium">{{ t('ui.pasteCallbackUrlOrToken') }}</span>
          </div>
          <div class="pl-6">
            <Textarea
              :model-value="device.callback_url"
              :disabled="device.completing"
              :placeholder="callbackPlaceholder"
              class="min-h-[150px] text-xs font-mono break-all !rounded-xl"
              spellcheck="false"
              @update:model-value="(v: string) => $emit('update:field', 'callback_url', v)"
            />
          </div>
          <div
            v-if="device.session_id && device.status === 'pending'"
            class="pl-6 flex items-center gap-1.5 text-[11px] text-muted-foreground"
          >
            <div class="animate-spin rounded-full h-3 w-3 border-[1.5px] border-primary/30 border-t-primary" />
            <span>{{ sessionRemainingText }}</span>
          </div>
        </div>
      </div>
    </div>
  </template>

  <!-- xAI: 设备授权 -->
  <template v-else-if="variant === 'xai'">
    <div class="space-y-3">
      <div class="h-[265px]">
        <div
          v-if="device.status === 'error' || device.status === 'expired'"
          class="rounded-xl border border-destructive/20 bg-destructive/5 p-5"
        >
          <div class="flex flex-col items-center text-center space-y-3">
            <div class="w-10 h-10 rounded-full bg-destructive/10 flex items-center justify-center">
              <AlertCircle class="w-5 h-5 text-destructive" />
            </div>
            <div class="space-y-1">
              <p class="text-sm font-medium text-destructive">
                {{ device.status === 'expired' ? t('ui.authorizationExpired') : t('ui.authorizationFailed') }}
              </p>
              <p class="text-xs text-muted-foreground">
                {{ (device.error || t('ui.pleaseTryAgain')) }}
              </p>
            </div>
            <Button
              size="sm"
              variant="outline"
              @click="$emit('reset')"
            >
              {{ t('ui.startOver') }}
            </Button>
          </div>
        </div>

        <div
          v-else-if="device.starting && !device.session_id"
          class="flex items-center justify-center py-12"
        >
          <div class="text-center">
            <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary mx-auto mb-3" />
            <p class="text-xs text-muted-foreground">
              {{ t('ui.preparingDeviceAuthorization') }}
            </p>
          </div>
        </div>

        <div
          v-else-if="device.session_id && device.status === 'pending'"
          class="rounded-xl border border-border bg-muted/20 p-5"
        >
          <div class="flex flex-col items-center text-center space-y-4">
            <div class="relative">
              <div class="absolute inset-0 rounded-full bg-primary/20 animate-ping" />
              <div class="relative w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center">
                <ExternalLink class="w-5 h-5 text-primary" />
              </div>
            </div>

            <div class="space-y-1">
              <p class="text-sm font-medium">
                {{ t('ui.enterTheDeviceCodeInYourBrowserToCompleteAuthorization') }}
              </p>
              <p class="text-xs text-muted-foreground">
                {{ t('ui.thisPageUpdatesAutomaticallyAfterAuthorizationIsComplete') }}
              </p>
            </div>

            <div
              v-if="device.user_code"
              class="flex items-center gap-2 rounded-lg border border-border bg-background px-3 py-2"
            >
              <span class="text-lg font-mono font-bold tracking-[0.2em]">{{ device.user_code }}</span>
              <button
                class="p-1 rounded hover:bg-muted transition-colors"
                :title="t('ui.copyDeviceCode')"
                @click="copyToClipboard(device.user_code)"
              >
                <Copy class="w-3.5 h-3.5 text-muted-foreground" />
              </button>
            </div>

            <div class="flex items-center gap-1.5 text-xs text-muted-foreground">
              <div class="animate-spin rounded-full h-3 w-3 border-[1.5px] border-primary/30 border-t-primary" />
              <span>{{ remainingText }}</span>
            </div>

            <div class="flex gap-2 w-full">
              <Button
                class="flex-1"
                size="sm"
                :disabled="!device.verification_uri_complete && !device.verification_uri"
                @click="$emit('openVerificationUrl')"
              >
                <ExternalLink class="w-3.5 h-3.5 mr-1.5" />
                {{ t('ui.openAuthorizationPage') }}
              </Button>
              <Button
                size="sm"
                variant="outline"
                :disabled="!device.verification_uri_complete && !device.verification_uri"
                @click="copyToClipboard(device.verification_uri_complete || device.verification_uri)"
              >
                <Copy class="w-3.5 h-3.5" />
              </Button>
            </div>
          </div>
        </div>

        <div
          v-else
          class="flex h-full flex-col items-center justify-center gap-3"
        >
          <p class="text-xs text-muted-foreground text-center">
            {{ t('ui.signInToGrokCliWithXaiDeviceAuthorizationOr') }}
          </p>
          <Button
            class="w-full"
            :disabled="device.starting"
            @click="$emit('start')"
          >
            {{ device.starting ? t('ui.preparingAuthorization') : t('ui.authorize') }}
          </Button>
        </div>
      </div>
    </div>
  </template>

  <!-- Kiro: 设备授权模式 -->
  <template v-else-if="variant === 'kiro'">
    <div class="space-y-3">
      <!-- 授权类型切换 -->
      <div class="grid grid-cols-2 gap-1.5">
        <button
          v-for="opt in ([
            { key: 'google', label: 'Google' },
            { key: 'github', label: 'GitHub' },
            { key: 'builder_id', label: 'Builder ID' },
            { key: 'identity_center', label: 'Identity Center' },
          ] as const)"
          :key="opt.key"
          class="h-8 text-xs font-medium rounded-md border transition-colors disabled:opacity-60"
          :class="device.auth_type === opt.key
            ? 'border-primary bg-primary/5 text-foreground'
            : 'border-border text-muted-foreground hover:text-foreground hover:border-foreground/20'"
          :disabled="kiroOptionDisabled"
          @click="$emit('selectAuthType', opt.key)"
        >
          {{ opt.label }}
        </button>
      </div>

      <div class="h-[265px]">
        <!-- 错误/过期 -->
        <div
          v-if="device.status === 'error' || device.status === 'expired'"
          class="rounded-xl border border-destructive/20 bg-destructive/5 p-5"
        >
          <div class="flex flex-col items-center text-center space-y-3">
            <div class="w-10 h-10 rounded-full bg-destructive/10 flex items-center justify-center">
              <AlertCircle class="w-5 h-5 text-destructive" />
            </div>
            <div class="space-y-1">
              <p class="text-sm font-medium text-destructive">
                {{ device.status === 'expired' ? t('ui.authorizationExpired') : t('ui.authorizationFailed') }}
              </p>
              <p class="text-xs text-muted-foreground">
                {{ (device.error || t('ui.pleaseTryAgain')) }}
              </p>
            </div>
            <Button
              size="sm"
              variant="outline"
              @click="$emit('reset')"
            >
              {{ t('ui.startOver') }}
            </Button>
          </div>
        </div>

        <!-- Builder ID / Identity Center: 发起中 -->
        <div
          v-else-if="device.starting && !isSocialDeviceAuth"
          class="flex items-center justify-center py-12"
        >
          <div class="text-center">
            <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary mx-auto mb-3" />
            <p class="text-xs text-muted-foreground">
              {{ t('ui.registeringDevice') }}
            </p>
          </div>
        </div>

        <!-- Google / GitHub: 粘贴回调 URL -->
        <div
          v-else-if="isSocialDeviceAuth"
          class="flex h-full flex-col gap-5 pt-1"
        >
          <div class="space-y-2 shrink-0">
            <div class="flex items-center gap-2">
              <span class="flex items-center justify-center w-4 h-4 rounded-full bg-primary/10 text-primary text-[10px] font-semibold shrink-0">1</span>
              <span class="text-xs font-medium">{{ t('ui.goToAuthorization') }}</span>
            </div>
            <div class="flex gap-2 pl-6">
              <Button
                size="sm"
                :disabled="device.starting || device.completing || !device.verification_uri_complete"
                @click="$emit('openVerificationUrl')"
              >
                <ExternalLink class="w-3 h-3 mr-1" />
                {{ t('ui.openc771248') }}
              </Button>
              <Button
                size="sm"
                variant="outline"
                :disabled="device.starting || device.completing || !device.verification_uri_complete"
                @click="copyToClipboard(device.verification_uri_complete)"
              >
                <Copy class="w-3 h-3 mr-1" />
                {{ t('common.copy') }}
              </Button>
            </div>
          </div>

          <div class="flex min-h-0 flex-1 flex-col gap-2">
            <div class="flex items-center gap-2">
              <span class="flex items-center justify-center w-4 h-4 rounded-full bg-primary/10 text-primary text-[10px] font-semibold shrink-0">2</span>
              <span class="text-xs font-medium">{{ t('ui.pasteCallbackUrl') }}</span>
            </div>
            <div class="min-h-0 flex-1 pl-6">
              <Textarea
                :model-value="device.callback_url"
                :disabled="device.completing"
                :placeholder="callbackPlaceholder"
                class="h-full min-h-0 overflow-y-auto text-xs font-mono break-all !rounded-xl"
                spellcheck="false"
                @update:model-value="(v: string) => $emit('update:field', 'callback_url', v)"
              />
            </div>
          </div>
        </div>

        <!-- Builder ID / Identity Center: 等待用户授权 -->
        <div
          v-else-if="device.session_id && device.status === 'pending'"
          class="rounded-xl border border-border bg-muted/20 p-5"
        >
          <div class="flex flex-col items-center text-center space-y-4">
            <div class="relative">
              <div class="absolute inset-0 rounded-full bg-primary/20 animate-ping" />
              <div class="relative w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center">
                <ExternalLink class="w-5 h-5 text-primary" />
              </div>
            </div>

            <div class="space-y-1">
              <p class="text-sm font-medium">
                {{ t('ui.completeAuthorizationInTheBrowser') }}
              </p>
              <p class="text-xs text-muted-foreground">
                {{ t('ui.thisPageUpdatesAutomaticallyAfterAuthorizationIsComplete') }}
              </p>
            </div>

            <div class="flex items-center gap-1.5 text-xs text-muted-foreground">
              <div class="animate-spin rounded-full h-3 w-3 border-[1.5px] border-primary/30 border-t-primary" />
              <span>{{ remainingText }}</span>
            </div>

            <div
              v-if="totpCode"
              class="w-full rounded-lg border border-border bg-background p-3"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <ShieldCheck class="w-3.5 h-3.5 text-primary" />
                  <span class="text-[10px] text-muted-foreground">{{ t('ui.mfaCode') }}</span>
                </div>
                <div class="flex items-center gap-1.5">
                  <span
                    class="text-lg font-mono font-bold tracking-[0.25em]"
                  >{{ totpCode }}</span>
                  <button
                    class="p-1 rounded hover:bg-muted transition-colors"
                    :title="t('ui.copyVerificationCode')"
                    @click="copyToClipboard(totpCode)"
                  >
                    <Copy class="w-3 h-3 text-muted-foreground" />
                  </button>
                </div>
              </div>
              <div class="mt-2 flex items-center gap-2">
                <div class="flex-1 h-1 rounded-full bg-muted overflow-hidden">
                  <div
                    class="h-full rounded-full transition-all duration-1000 ease-linear"
                    :class="totpRemaining <= 5 ? 'bg-red-500' : 'bg-primary'"
                    :style="{ width: `${(totpRemaining / 30) * 100}%` }"
                  />
                </div>
                <span
                  class="text-[10px] font-mono tabular-nums shrink-0"
                  :class="totpRemaining <= 5 ? 'text-red-500' : 'text-muted-foreground'"
                >{{ totpRemaining }}s</span>
              </div>
            </div>

            <div class="flex gap-2 w-full">
              <Button
                class="flex-1"
                size="sm"
                @click="$emit('openVerificationUrl')"
              >
                <ExternalLink class="w-3.5 h-3.5 mr-1.5" />
                {{ t('ui.openAuthorizationPage') }}
              </Button>
              <Button
                size="sm"
                variant="outline"
                @click="copyToClipboard(device.verification_uri_complete)"
              >
                <Copy class="w-3.5 h-3.5" />
              </Button>
            </div>
          </div>
        </div>

        <!-- 初始状态：当前类型配置 -->
        <div
          v-else
          :class="device.auth_type === 'builder_id' ? 'flex h-full flex-col justify-center gap-4' : 'space-y-3'"
        >
          <p
            v-if="isSocialDeviceAuth"
            class="text-xs text-muted-foreground text-center"
          >
            {{ t('ui.afterAuthorizationCopyTheLocalhostCallbackUrlFromTheBrowser') }}
          </p>

          <p
            v-else-if="device.auth_type === 'builder_id'"
            class="text-xs text-muted-foreground text-center"
          >
            {{ t('ui.useAPersonalAwsBuilderIdForDeviceAuthorizationNo') }}
          </p>

          <div
            v-else
            class="space-y-3"
          >
            <div class="space-y-1.5">
              <label class="text-xs font-medium">Start URL</label>
              <input
                :value="device.start_url"
                type="text"
                placeholder="https://your-org.awsapps.com/start"
                class="w-full h-8 px-2 text-xs rounded-md border border-border bg-background font-mono focus:outline-none focus:ring-1 focus:ring-ring focus:relative focus:z-10"
                spellcheck="false"
                @input="(e: Event) => $emit('update:field', 'start_url', (e.target as HTMLInputElement).value)"
              >
            </div>
            <div class="space-y-1.5">
              <label class="text-xs font-medium">Region</label>
              <ComboboxRoot
                :model-value="device.region"
                :open="regionComboboxOpen"
                @update:model-value="(v: string) => { if (v) $emit('update:field', 'region', v) }"
                @update:open="(v: boolean) => { regionComboboxOpen = v; if (v) ensureAwsRegions() }"
              >
                <ComboboxAnchor class="relative w-full">
                  <ComboboxInput
                    :display-value="() => device.region"
                    :placeholder="t('ui.enterOrSelectRegion')"
                    class="w-full h-8 px-2 pr-7 text-xs rounded-md border border-border bg-background font-mono focus:outline-none focus:ring-1 focus:ring-ring focus:relative focus:z-10"
                    spellcheck="false"
                    @input="(e: Event) => regionSearch = (e.target as HTMLInputElement).value"
                    @keydown.enter.prevent="onRegionEnter"
                  />
                  <ComboboxTrigger class="absolute right-1.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground">
                    <ChevronsUpDown class="w-3.5 h-3.5" />
                  </ComboboxTrigger>
                </ComboboxAnchor>
                <ComboboxContent
                  position="popper"
                  class="z-[99] mt-1 max-h-[200px] w-[--radix-combobox-trigger-width] overflow-y-auto rounded-md border border-border bg-popover shadow-md"
                >
                  <ComboboxViewport>
                    <ComboboxEmpty class="px-2 py-1.5 text-xs text-muted-foreground">
                      {{ awsRegionsLoaded ? t('ui.noMatchesPressEnterToUseACustomValue') : t('common.loading') }}
                    </ComboboxEmpty>
                    <ComboboxItem
                      v-for="r in filteredRegions"
                      :key="r"
                      :value="r"
                      class="flex items-center gap-1.5 px-2 py-1.5 text-xs font-mono cursor-pointer rounded-sm outline-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground"
                    >
                      <Check
                        class="w-3 h-3 shrink-0"
                        :class="device.region === r ? 'opacity-100' : 'opacity-0'"
                      />
                      {{ r }}
                    </ComboboxItem>
                  </ComboboxViewport>
                </ComboboxContent>
              </ComboboxRoot>
            </div>
            <div class="space-y-1.5">
              <label class="text-xs font-medium text-muted-foreground">{{ t('ui.totpSecretOptional2fa') }}</label>
              <input
                :value="device.totp_secret"
                type="text"
                :placeholder="t('ui.base32SecretEGJbswy3dpehpk3pxp')"
                class="w-full h-8 px-2 text-xs rounded-md border border-border bg-background font-mono focus:outline-none focus:ring-1 focus:ring-ring focus:relative focus:z-10"
                spellcheck="false"
                @input="(e: Event) => $emit('update:field', 'totp_secret', (e.target as HTMLInputElement).value)"
              >
            </div>
          </div>

          <Button
            class="w-full"
            :disabled="device.starting || (device.auth_type === 'identity_center' && !device.start_url.trim())"
            @click="$emit('start')"
          >
            {{ device.starting ? t('ui.preparingAuthorization') : t('ui.authorize') }}
          </Button>
        </div>
      </div>
    </div>
  </template>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import {
  ComboboxAnchor,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxRoot,
  ComboboxTrigger,
  ComboboxViewport,
} from 'radix-vue'
import { AlertCircle, Check, ChevronsUpDown, Copy, ExternalLink, ShieldCheck } from 'lucide-vue-next'
import { Button, Textarea } from '@/components/ui'
import { getAwsRegions } from '@/api/endpoints'
import { useClipboard } from '@/composables/useClipboard'
import { useI18n } from '@/i18n'
import type {
  DeviceAuthState,
  DeviceAuthType,
  WindsurfLoginOption,
} from '@/features/providers/utils/oauthDeviceAuth'

export type DeviceAuthEditableField = 'callback_url' | 'start_url' | 'region' | 'totp_secret'

/**
 * OAuth 账号对话框里的设备授权面板：Windsurf 浏览器登录、xAI 设备码、
 * Kiro（Google / GitHub 回调、Builder ID、Identity Center）。
 * 会话状态与轮询由父组件持有，面板只负责展示和把用户操作抛回去。
 */
const props = withDefaults(defineProps<{
  variant: 'windsurf' | 'xai' | 'kiro'
  device: DeviceAuthState
  countdownSeconds: number
  totpCode?: string
  totpRemaining?: number
  kiroOptionDisabled?: boolean
}>(), {
  totpCode: '',
  totpRemaining: 0,
  kiroOptionDisabled: false,
})

const emit = defineEmits<{
  'update:field': [field: DeviceAuthEditableField, value: string]
  selectWindsurfLoginOption: [option: WindsurfLoginOption]
  selectAuthType: [authType: DeviceAuthType]
  reset: []
  start: []
  openVerificationUrl: []
}>()

const { t } = useI18n()

const { copyToClipboard } = useClipboard()

const isSocialDeviceAuth = computed(() =>
  props.device.auth_type === 'google' || props.device.auth_type === 'github'
)

const callbackPlaceholder = computed(() =>
  props.variant === 'windsurf'
    ? t('ui.pasteACallbackUrlContainingTokenStateSessionTokenApikey')
    : `http://localhost:49153/oauth/callback?login_option=${props.device.auth_type}&code=...&state=...`
)

const countdownFormatted = computed(() => {
  const s = props.countdownSeconds
  const min = Math.floor(s / 60)
  const sec = s % 60
  return `${min}:${String(sec).padStart(2, '0')}`
})

const sessionRemainingText = computed(() => (
  t('providers.migration.sessionExpiresIn5fcc5f', { value: String(countdownFormatted.value) })
))

const remainingText = computed(() => (
  t('providers.migration.remaining43f955', { value: String(countdownFormatted.value) })
))

// AWS Regions (动态获取 + 进程内缓存)
const awsRegions = ref<string[]>([])
const awsRegionsLoaded = ref(false)
const regionSearch = ref('')
const regionComboboxOpen = ref(false)

const filteredRegions = computed(() => {
  const q = regionSearch.value.trim().toLowerCase()
  if (!q) return awsRegions.value
  return awsRegions.value.filter(r => r.includes(q))
})

async function ensureAwsRegions() {
  if (awsRegionsLoaded.value) return
  try {
    awsRegions.value = await getAwsRegions()
  } catch {
    awsRegions.value = ['us-east-1', 'us-east-2', 'us-west-1', 'us-west-2', 'eu-north-1']
  }
  awsRegionsLoaded.value = true
}

function onRegionEnter() {
  // If no matching item is highlighted, accept the raw input as a custom region value.
  const raw = regionSearch.value.trim()
  if (raw && !filteredRegions.value.includes(raw)) {
    emit('update:field', 'region', raw)
    regionComboboxOpen.value = false
    regionSearch.value = ''
  }
}
</script>
