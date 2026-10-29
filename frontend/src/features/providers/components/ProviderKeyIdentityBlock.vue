<template>
  <div class="flex flex-col min-w-0">
    <div class="flex items-center gap-1.5">
      <span
        class="text-sm font-medium truncate"
        :class="apiKey.name ? 'cursor-pointer hover:text-primary transition-colors' : ''"
        :title="apiKey.name ? t('ui.clickToCopy') : ''"
        data-testid="provider-key-name"
        @click.stop="apiKey.name && $emit('copyName', apiKey.name)"
      >
        {{ apiKey.name || t('ui.unnamedKey') }}
      </span>

      <Badge
        v-if="oauthPlanLabel"
        variant="outline"
        class="text-[10px] px-1.5 py-0 shrink-0"
        :class="oauthPlanClass"
        data-testid="provider-key-oauth-plan"
      >
        {{ oauthPlanLabel }}
      </Badge>

      <Badge
        v-if="oauthOrgBadge"
        variant="secondary"
        class="text-[9px] px-1 py-0 h-4 shrink-0"
        :title="oauthOrgBadge.title"
        data-testid="provider-key-oauth-org"
      >
        {{ oauthOrgBadge.label }}
      </Badge>

      <Badge
        v-if="kiroSubscriptionLabel"
        variant="outline"
        class="text-[10px] px-1.5 py-0 shrink-0"
        :class="kiroSubscriptionClass"
        data-testid="provider-key-kiro-plan"
      >
        {{ kiroSubscriptionLabel }}
      </Badge>
    </div>

    <div class="flex items-center gap-1">
      <span class="text-[11px] font-mono text-muted-foreground">
        {{ maskedSecretLabel }}
      </span>

      <Button
        v-if="canExportCredential"
        variant="ghost"
        size="icon"
        class="h-4 w-4 shrink-0"
        :title="t('ui.downloadOauthAuthorizationFile')"
        @click.stop="$emit('downloadCredential')"
      >
        <Download class="w-2.5 h-2.5" />
      </Button>
      <Button
        v-else-if="apiKey.agent_identity !== true"
        variant="ghost"
        size="icon"
        class="h-4 w-4 shrink-0"
        :title="t('ui.copyKey')"
        @click.stop="$emit('copyFullKey')"
      >
        <Copy class="w-2.5 h-2.5" />
      </Button>

      <template v-if="showOAuthRefreshControl">
        <template v-if="accountLevelBlock">
          <Badge
            variant="destructive"
            class="text-[10px] px-1.5 py-0 shrink-0 gap-0.5"
            :title="oauthStatusTitle"
            data-testid="provider-key-account-block"
          >
            <ShieldX class="w-2.5 h-2.5" />
            {{ t('ui.accountIssue') }}
          </Badge>
          <Button
            variant="ghost"
            size="icon"
            class="h-4 w-4 shrink-0 text-destructive hover:text-destructive"
            :disabled="clearingOAuthInvalid"
            :title="t('ui.clearIssueMarkerAfterConfirmingTheAccountHasCompletedVerification')"
            @click.stop="$emit('clearOAuthInvalid')"
          >
            <RefreshCw
              class="w-2.5 h-2.5"
              :class="{ 'animate-spin': clearingOAuthInvalid }"
            />
          </Button>
        </template>

        <template v-else>
          <span
            class="text-[10px]"
            :class="oauthStatusClass"
            :title="oauthStatusTitle"
            data-testid="provider-key-oauth-status"
          >
            {{ oauthStatus?.text }}
          </span>
          <Badge
            v-if="apiKey.oauth_temporary"
            variant="outline"
            class="text-[10px] px-1.5 py-0 shrink-0"
            :title="t('ui.importedWithAccessTokenOnlyItCannotRefreshAutomaticallyAnd')"
            data-testid="provider-key-temporary"
          >
            {{ t('ui.temporary') }}
          </Badge>
          <Button
            variant="ghost"
            size="icon"
            class="h-4 w-4 shrink-0"
            :disabled="refreshingOAuth || !canRefreshCredential"
            :title="oauthRefreshButtonTitle"
            @click.stop="$emit('refreshOAuth')"
          >
            <RefreshCw
              class="w-2.5 h-2.5"
              :class="{ 'animate-spin': refreshingOAuth }"
            />
          </Button>
        </template>
      </template>

      <span
        v-if="antigravityInactive"
        class="text-[10px] text-orange-500 dark:text-orange-400"
        :title="t('ui.thisAccountHasNotCompletedGeminiCodeAssistActivationSo')"
        data-testid="provider-key-antigravity-inactive"
      >
        {{ t('ui.accountNotActivated') }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Copy, Download, RefreshCw, ShieldX } from 'lucide-vue-next'
import Button from '@/components/ui/button.vue'
import Badge from '@/components/ui/badge.vue'
import { useI18n } from '@/i18n'
import type { EndpointAPIKey } from '@/api/endpoints'
import type { OAuthStatusInfo } from '@/composables/useCountdownTimer'

interface OAuthOrgBadgeDisplay {
  label: string
  title: string
}

const props = withDefaults(defineProps<{
  apiKey: EndpointAPIKey
  maskedSecretLabel: string
  oauthPlanLabel?: string | null
  oauthPlanClass?: string
  oauthOrgBadge?: OAuthOrgBadgeDisplay | null
  kiroSubscriptionLabel?: string | null
  kiroSubscriptionClass?: string
  canExportCredential?: boolean
  showOAuthRefreshControl?: boolean
  accountLevelBlock?: boolean
  oauthStatus?: OAuthStatusInfo | null
  oauthStatusTitle?: string
  oauthRefreshButtonTitle?: string
  canRefreshCredential?: boolean
  clearingOAuthInvalid?: boolean
  refreshingOAuth?: boolean
  antigravityInactive?: boolean
}>(), {
  oauthPlanLabel: null,
  oauthPlanClass: '',
  oauthOrgBadge: null,
  kiroSubscriptionLabel: null,
  kiroSubscriptionClass: '',
  canExportCredential: false,
  showOAuthRefreshControl: false,
  accountLevelBlock: false,
  oauthStatus: null,
  oauthStatusTitle: '',
  oauthRefreshButtonTitle: '',
  canRefreshCredential: false,
  clearingOAuthInvalid: false,
  refreshingOAuth: false,
  antigravityInactive: false,
})

defineEmits<{
  (e: 'copyName', name: string): void
  (e: 'downloadCredential'): void
  (e: 'copyFullKey'): void
  (e: 'clearOAuthInvalid'): void
  (e: 'refreshOAuth'): void
}>()

const { t } = useI18n()

const oauthStatusClass = computed(() => {
  const status = props.oauthStatus
  return {
    'text-destructive': status?.isInvalid || status?.isExpired,
    'text-warning': status?.isExpiringSoon && !status?.isExpired && !status?.isInvalid,
    'text-muted-foreground': !status?.isExpired && !status?.isExpiringSoon && !status?.isInvalid,
  }
})
</script>
