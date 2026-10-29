<template>
  <CardSection
    :title="t('ui.basicConfiguration')"
    :description="t('ui.configureSystemDefaults')"
  >
    <template #actions>
      <Button
        size="sm"
        :disabled="loading || !hasChanges"
        @click="$emit('save')"
      >
        {{ loading ? t('ui.saving') : t('ui.save') }}
      </Button>
    </template>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div>
        <Label
          for="default-quota"
          class="block text-sm font-medium"
        > {{ t('ui.defaultUserInitialGrantUsd') }} </Label>
        <Input
          id="default-quota"
          :model-value="defaultUserInitialGiftUsd"
          type="number"
          step="0.01"
          placeholder="10.00"
          class="mt-1"
          @update:model-value="$emit('update:defaultUserInitialGiftUsd', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.defaultInitialGrantForNewUsers') }}
        </p>
      </div>

      <div>
        <Label
          for="rate-limit"
          class="block text-sm font-medium"
        > {{ t('ui.defaultRateLimitRequestsMin') }} </Label>
        <Input
          id="rate-limit"
          :model-value="rateLimitPerMinute"
          type="number"
          placeholder="0"
          class="mt-1"
          @update:model-value="$emit('update:rateLimitPerMinute', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.0MeansNoDefaultLimitUsersAndStandaloneKeysWithout') }}
        </p>
      </div>

      <div>
        <Label
          for="password-policy-level"
          class="block text-sm font-medium mb-2"
        > {{ t('ui.passwordPolicy') }} </Label>
        <Select
          :model-value="passwordPolicyLevel"
          @update:model-value="$emit('update:passwordPolicyLevel', $event)"
        >
          <SelectTrigger
            id="password-policy-level"
            class="mt-1"
          >
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="weak">
              {{ t('ui.weakPasswordAtLeast6Characters') }}
            </SelectItem>
            <SelectItem value="medium">
              {{ t('ui.mediumPasswordAtLeast8CharactersWithLettersAndNumbers') }}
            </SelectItem>
            <SelectItem value="strong">
              {{ t('ui.strongPasswordAtLeast8CharactersWithUppercaseLowercaseNumbers') }}
            </SelectItem>
          </SelectContent>
        </Select>
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.affectsValidationRulesForRegistrationUserCreationPasswordResetAnd') }}
        </p>
      </div>

      <div class="flex items-center h-full">
        <div class="flex items-center space-x-2">
          <Checkbox
            id="enable-registration"
            :checked="enableRegistration"
            @update:checked="$emit('update:enableRegistration', $event)"
          />
          <div>
            <Label
              for="enable-registration"
              class="cursor-pointer"
            > {{ t('ui.openUserRegistration') }} </Label>
            <p class="text-xs text-muted-foreground">
              {{ t('ui.allowNewUsersToSelfRegisterAccounts') }}
            </p>
          </div>
        </div>
      </div>

      <div class="flex items-center h-full">
        <div class="flex items-center space-x-2">
          <Checkbox
            id="auto-delete-expired-keys"
            :checked="autoDeleteExpiredKeys"
            @update:checked="$emit('update:autoDeleteExpiredKeys', $event)"
          />
          <div>
            <Label
              for="auto-delete-expired-keys"
              class="cursor-pointer"
            > {{ t('ui.autoDeleteExpiredKeys') }} </Label>
            <p class="text-xs text-muted-foreground">
              {{ t('ui.whenDisabledExpiredStandaloneBalanceKeysAreOnlyDisabled') }}
            </p>
          </div>
        </div>
      </div>

      <div class="flex items-center h-full">
        <div class="flex items-center space-x-2">
          <Checkbox
            id="enable-format-conversion"
            :checked="enableFormatConversion"
            @update:checked="$emit('update:enableFormatConversion', $event)"
          />
          <div>
            <Label
              for="enable-format-conversion"
              class="cursor-pointer"
            > {{ t('ui.globalFormatConversion') }} </Label>
            <p class="text-xs text-muted-foreground">
              {{ t('ui.whenEnabledAllProvidersAreForcedToAcceptCrossFormat') }}
            </p>
          </div>
        </div>
      </div>

      <div class="md:col-span-2 grid grid-cols-1 md:grid-cols-2 gap-4 border-t pt-5">
        <div class="flex items-center h-full">
          <div class="flex items-center space-x-2">
            <Checkbox
              id="turnstile-enabled"
              :checked="turnstileEnabled"
              @update:checked="$emit('update:turnstileEnabled', $event)"
            />
            <div>
              <Label
                for="turnstile-enabled"
                class="cursor-pointer"
              > {{ t('ui.registrationHumanVerification') }} </Label>
              <p class="text-xs text-muted-foreground">
                {{ t('ui.whenEnabledRegistrationAndEmailVerificationCodeSendingRequireCloudflare') }}
              </p>
            </div>
          </div>
        </div>

        <div>
          <Label
            for="turnstile-site-key"
            class="block text-sm font-medium"
          >
            Turnstile Site Key
          </Label>
          <Input
            id="turnstile-site-key"
            :model-value="turnstileSiteKey || ''"
            type="text"
            placeholder="0x4AAAA..."
            class="mt-1"
            @update:model-value="$emit('update:turnstileSiteKey', String($event || '').trim() || null)"
          />
        </div>

        <div>
          <div class="flex items-center justify-between">
            <Label
              for="turnstile-secret-key"
              class="block text-sm font-medium"
            >
              Turnstile Secret Key
            </Label>
            <Button
              v-if="turnstileSecretConfigured"
              type="button"
              variant="link"
              size="sm"
              class="h-auto p-0 text-xs"
              :disabled="loading"
              @click="$emit('clearTurnstileSecret')"
            >
              {{ t('ui.clear1ef3de0') }}
            </Button>
          </div>
          <Input
            id="turnstile-secret-key"
            :model-value="turnstileSecretKey"
            type="password"
            :placeholder="turnstileSecretConfigured ? t('ui.configuredLeaveEmptyToKeepUnchanged') : t('ui.enterSecretKey')"
            class="mt-1"
            autocomplete="new-password"
            @update:model-value="$emit('update:turnstileSecretKey', String($event || ''))"
          />
        </div>

        <div>
          <Label
            for="turnstile-hostnames"
            class="block text-sm font-medium"
          > {{ t('ui.allowedHostnames') }} </Label>
          <Input
            id="turnstile-hostnames"
            :model-value="turnstileAllowedHostnamesStr"
            type="text"
            placeholder="example.com, app.example.com"
            class="mt-1"
            @update:model-value="$emit('update:turnstileAllowedHostnamesStr', String($event || ''))"
          />
          <p class="mt-1 text-xs text-muted-foreground">
            {{ t('ui.leaveEmptyToSkipExtraValidationOfTheHostnameReturned') }}
          </p>
        </div>
      </div>

      <div class="md:col-span-2 grid grid-cols-1 md:grid-cols-2 gap-4 border-t pt-5">
        <div class="flex items-center h-full">
          <div class="flex items-center space-x-2">
            <Checkbox
              id="referral-enabled"
              :checked="referralEnabled"
              @update:checked="$emit('update:referralEnabled', $event)"
            />
            <div>
              <Label
                for="referral-enabled"
                class="cursor-pointer"
              > {{ t('nav.referralRewards') }} </Label>
              <p class="text-xs text-muted-foreground">
                {{ t('ui.whenEnabledGrantsCanBeRewardedByTopUpRatio') }}
              </p>
            </div>
          </div>
        </div>

        <div>
          <Label
            for="referral-reward-mode"
            class="block text-sm font-medium mb-2"
          > {{ t('ui.rewardMethod') }} </Label>
          <Select
            :model-value="referralRewardMode"
            @update:model-value="$emit('update:referralRewardMode', $event)"
          >
            <SelectTrigger id="referral-reward-mode">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="percent">
                {{ t('ui.byTopUpRatio') }}
              </SelectItem>
              <SelectItem value="headcount">
                {{ t('ui.byReferredUsers') }}
              </SelectItem>
              <SelectItem value="both">
                {{ t('ui.enableBoth') }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div>
          <Label
            for="referral-recharge-percent"
            class="block text-sm font-medium"
          > {{ t('ui.topUpRewardRatio') }} </Label>
          <Input
            id="referral-recharge-percent"
            :model-value="referralRechargePercent"
            type="number"
            min="0"
            max="100"
            step="0.01"
            class="mt-1"
            @update:model-value="$emit('update:referralRechargePercent', Number($event))"
          />
        </div>

        <div>
          <Label
            for="referral-headcount-amount"
            class="block text-sm font-medium"
          > {{ t('ui.perUserRewardAmountUsd') }} </Label>
          <Input
            id="referral-headcount-amount"
            :model-value="referralHeadcountAmountUsd"
            type="number"
            min="0"
            step="0.01"
            class="mt-1"
            @update:model-value="$emit('update:referralHeadcountAmountUsd', Number($event))"
          />
        </div>

        <div>
          <Label
            for="referral-headcount-trigger"
            class="block text-sm font-medium mb-2"
          > {{ t('ui.perUserRewardTrigger') }} </Label>
          <Select
            :model-value="referralHeadcountTrigger"
            @update:model-value="$emit('update:referralHeadcountTrigger', $event)"
          >
            <SelectTrigger id="referral-headcount-trigger">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="registration">
                {{ t('auth.register.successTitle') }}
              </SelectItem>
              <SelectItem
                value="email_verified"
                :disabled="requireEmailVerification !== true"
              >
                {{ t('ui.emailVerificationComplete') }}
              </SelectItem>
              <SelectItem value="first_paid_order">
                {{ t('ui.firstRealPaymentComplete') }}
              </SelectItem>
            </SelectContent>
          </Select>
          <p class="mt-2 text-xs text-muted-foreground">
            {{ t('ui.rewardsForEmailVerificationRequireRegistrationEmailVerificationToBe') }}
          </p>
          <p
            v-if="referralHeadcountTrigger === 'email_verified' && requireEmailVerification !== true"
            class="mt-1 text-xs text-destructive"
            role="alert"
          >
            {{ verificationLookupFailed ? t('ui.unableToVerifyTheEmailVerificationConfigurationRefreshAndTry') : requireEmailVerification === null ? t('ui.checkingTheEmailVerificationConfigurationTryAgainShortly') : t('ui.registrationEmailVerificationIsDisabledChangeTheTriggerOrEnable') }}
          </p>
        </div>
      </div>

      <div class="md:col-span-2 grid grid-cols-1 md:grid-cols-2 gap-4 border-t pt-5">
        <div class="flex items-center h-full">
          <div class="flex items-center space-x-2">
            <Checkbox
              id="privacy-policy-enabled"
              :checked="registrationPrivacyPolicyEnabled"
              @update:checked="$emit('update:registrationPrivacyPolicyEnabled', $event)"
            />
            <div>
              <Label
                for="privacy-policy-enabled"
                class="cursor-pointer"
              > {{ t('ui.registrationPrivacyConfirmation') }} </Label>
              <p class="text-xs text-muted-foreground">
                {{ t('ui.whenEnabledUsersMustConfirmTheCurrentVersionDuringRegistration') }}
              </p>
            </div>
          </div>
        </div>

        <div>
          <Label
            for="privacy-policy-version"
            class="block text-sm font-medium"
          > {{ t('ui.privacyPolicyVersion') }} </Label>
          <Input
            id="privacy-policy-version"
            :model-value="registrationPrivacyPolicyVersion"
            type="text"
            placeholder="2026-05-16"
            class="mt-1"
            @update:model-value="$emit('update:registrationPrivacyPolicyVersion', String($event || '').trim())"
          />
        </div>

        <div>
          <Label
            for="privacy-policy-format"
            class="block text-sm font-medium mb-2"
          > {{ t('ui.privacyPolicyFormat') }} </Label>
          <Select
            :model-value="registrationPrivacyPolicyFormat"
            @update:model-value="$emit('update:registrationPrivacyPolicyFormat', $event)"
          >
            <SelectTrigger id="privacy-policy-format">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="markdown">
                Markdown
              </SelectItem>
              <SelectItem value="html">
                HTML
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div class="md:col-span-2">
          <Label
            for="privacy-policy-content"
            class="block text-sm font-medium"
          > {{ t('ui.privacyPolicyContent') }} </Label>
          <Textarea
            id="privacy-policy-content"
            :model-value="registrationPrivacyPolicyContent"
            rows="8"
            class="mt-1"
            :placeholder="t('ui.enterMarkdownOrHtmlContent')"
            @update:model-value="$emit('update:registrationPrivacyPolicyContent', $event)"
          />
        </div>
      </div>
    </div>
  </CardSection>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
defineProps<{
  defaultUserInitialGiftUsd: number
  rateLimitPerMinute: number
  enableRegistration: boolean
  passwordPolicyLevel: string
  turnstileEnabled: boolean
  turnstileSiteKey: string | null
  turnstileSecretKey: string
  turnstileSecretConfigured: boolean
  turnstileAllowedHostnamesStr: string
  referralEnabled: boolean
  referralRewardMode: string
  referralRechargePercent: number
  referralHeadcountAmountUsd: number
  referralHeadcountTrigger: string
  registrationPrivacyPolicyEnabled: boolean
  registrationPrivacyPolicyFormat: string
  registrationPrivacyPolicyContent: string
  registrationPrivacyPolicyVersion: string
  autoDeleteExpiredKeys: boolean
  enableFormatConversion: boolean
  loading: boolean
  hasChanges: boolean
}>()

defineEmits<{
  save: []
  'update:defaultUserInitialGiftUsd': [value: number]
  'update:rateLimitPerMinute': [value: number]
  'update:enableRegistration': [value: boolean]
  'update:passwordPolicyLevel': [value: string]
  'update:turnstileEnabled': [value: boolean]
  'update:turnstileSiteKey': [value: string | null]
  'update:turnstileSecretKey': [value: string]
  'update:turnstileAllowedHostnamesStr': [value: string]
  clearTurnstileSecret: []
  'update:referralEnabled': [value: boolean]
  'update:referralRewardMode': [value: string]
  'update:referralRechargePercent': [value: number]
  'update:referralHeadcountAmountUsd': [value: number]
  'update:referralHeadcountTrigger': [value: string]
  'update:registrationPrivacyPolicyEnabled': [value: boolean]
  'update:registrationPrivacyPolicyFormat': [value: string]
  'update:registrationPrivacyPolicyContent': [value: string]
  'update:registrationPrivacyPolicyVersion': [value: string]
  'update:autoDeleteExpiredKeys': [value: boolean]
  'update:enableFormatConversion': [value: boolean]
}>()

const { t } = useI18n()

import { onMounted, ref } from 'vue'
import { authApi } from '@/api/auth'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import Textarea from '@/components/ui/textarea.vue'
import Checkbox from '@/components/ui/checkbox.vue'
import Select from '@/components/ui/select.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectValue from '@/components/ui/select-value.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import { CardSection } from '@/components/layout'

const requireEmailVerification = ref<boolean | null>(null)
const verificationLookupFailed = ref(false)
onMounted(async () => {
  try {
    const settings = await authApi.getRegistrationSettings()
    requireEmailVerification.value = settings.require_email_verification
  } catch {
    verificationLookupFailed.value = true
    // 无法核实时保持未知，避免把依赖配置误报为已开启。
  }
})

</script>
