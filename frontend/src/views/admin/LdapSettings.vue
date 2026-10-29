<template>
  <PageContainer>
    <PageHeader
      :title="t('ui.ldapConfiguration')"
      :description="t('ui.configureLdapAuthentication')"
    />

    <div class="mt-6 space-y-6">
      <CardSection
        :title="t('ui.ldapServerSettings')"
        :description="t('ui.configureTheLdapServerConnection')"
      >
        <template #actions>
          <div class="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              :disabled="testLoading"
              @click="handleTestConnection"
            >
              {{ testLoading ? t('ui.testing7b6187c') : t('ui.testConnection') }}
            </Button>
            <Button
              size="sm"
              :disabled="saveLoading"
              @click="handleSave"
            >
              {{ saveLoading ? t('ui.saving') : t('ui.save') }}
            </Button>
          </div>
        </template>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <Label
              for="server-url"
              class="block text-sm font-medium"
            > {{ t('ui.serverAddress') }} </Label>
            <Input
              id="server-url"
              v-model="ldapConfig.server_url"
              type="text"
              placeholder="ldap://ldap.example.com:389"
              class="mt-1"
            />
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('ui.formatLdapHost389OrLdapsHost636') }}
            </p>
          </div>

          <div>
            <Label
              for="bind-dn"
              class="block text-sm font-medium"
            > {{ t('ui.bindDn') }} </Label>
            <Input
              id="bind-dn"
              v-model="ldapConfig.bind_dn"
              type="text"
              placeholder="cn=admin,dc=example,dc=com"
              class="mt-1"
            />
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('ui.administratorDnUsedToConnectToTheLdapServer') }}
            </p>
          </div>

          <div>
            <Label
              for="bind-password"
              class="block text-sm font-medium"
            > {{ t('ui.bindPassword') }} </Label>
            <div class="mt-1">
              <Input
                id="bind-password"
                v-model="ldapConfig.bind_password"
                masked
                :placeholder="hasPassword ? t('ui.setLeaveEmptyToKeepUnchanged') : t('ui.enterPassword')"
                autocomplete="new-password"
              />
            </div>
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('ui.passwordForTheBindAccount') }}
            </p>
          </div>

          <div>
            <Label
              for="base-dn"
              class="block text-sm font-medium"
            > {{ t('ui.baseDn') }} </Label>
            <Input
              id="base-dn"
              v-model="ldapConfig.base_dn"
              type="text"
              placeholder="ou=users,dc=example,dc=com"
              class="mt-1"
            />
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('ui.baseDnForUserSearches') }}
            </p>
          </div>

          <div>
            <Label
              for="user-search-filter"
              class="block text-sm font-medium"
            > {{ t('ui.userSearchFilter') }} </Label>
            <Input
              id="user-search-filter"
              v-model="ldapConfig.user_search_filter"
              type="text"
              placeholder="(uid={username})"
              class="mt-1"
            />
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('ui.valueIsReplacedWithTheSignInUsername') }}
            </p>
          </div>

          <div>
            <Label
              for="username-attr"
              class="block text-sm font-medium"
            > {{ t('ui.usernameAttribute') }} </Label>
            <Input
              id="username-attr"
              v-model="ldapConfig.username_attr"
              type="text"
              placeholder="uid"
              class="mt-1"
            />
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('ui.commonValuesUidOpenldapSamaccountnameAd') }}
            </p>
          </div>

          <div>
            <Label
              for="email-attr"
              class="block text-sm font-medium"
            > {{ t('ui.emailAttribute') }} </Label>
            <Input
              id="email-attr"
              v-model="ldapConfig.email_attr"
              type="text"
              placeholder="mail"
              class="mt-1"
            />
          </div>

          <div>
            <Label
              for="display-name-attr"
              class="block text-sm font-medium"
            > {{ t('ui.displayNameAttribute') }} </Label>
            <Input
              id="display-name-attr"
              v-model="ldapConfig.display_name_attr"
              type="text"
              placeholder="cn"
              class="mt-1"
            />
          </div>

          <div>
            <Label
              for="connect-timeout"
              class="block text-sm font-medium"
            > {{ t('ui.connectionTimeoutSeconds') }} </Label>
            <Input
              id="connect-timeout"
              v-model.number="ldapConfig.connect_timeout"
              type="number"
              min="1"
              max="60"
              placeholder="10"
              class="mt-1"
            />
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('ui.timeoutPerLdapOperation160SecondsRecommendedForInternational') }}
            </p>
          </div>
        </div>

        <div class="mt-6 space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <Label class="text-sm font-medium">{{ t('ui.useStarttls') }}</Label>
              <p class="text-xs text-muted-foreground">
                {{ t('ui.enableTlsEncryptionOnANonSslConnection') }}
              </p>
            </div>
            <Switch v-model="ldapConfig.use_starttls" />
          </div>

          <div class="flex items-center justify-between">
            <div>
              <Label class="text-sm font-medium">{{ t('ui.enableLdapAuthentication') }}</Label>
              <p class="text-xs text-muted-foreground">
                {{ t('ui.allowUsersToSignInWithLdapAccounts') }}
              </p>
            </div>
            <Switch v-model="ldapConfig.is_enabled" />
          </div>

          <div class="flex items-center justify-between">
            <div>
              <Label class="text-sm font-medium">{{ t('ui.allowLdapSignInOnly') }}</Label>
              <p class="text-xs text-muted-foreground">
                {{ t('ui.disableLocalAccountSignInAndRequireLdapAuthentication') }}
              </p>
            </div>
            <Switch v-model="ldapConfig.is_exclusive" />
          </div>
        </div>
      </CardSection>
    </div>
  </PageContainer>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { ref, onMounted } from 'vue'
import { PageContainer, PageHeader, CardSection } from '@/components/layout'
import { Button, Input, Label, Switch } from '@/components/ui'
import { useToast } from '@/composables/useToast'
import { adminApi, type LdapConfigUpdateRequest } from '@/api/admin'
import { log } from '@/utils/logger'

const { success, error } = useToast()

const loading = ref(false)
const saveLoading = ref(false)
const testLoading = ref(false)
const hasPassword = ref(false)

const ldapConfig = ref({
  server_url: '',
  bind_dn: '',
  bind_password: '',
  base_dn: '',
  user_search_filter: '(uid={username})',
  username_attr: 'uid',
  email_attr: 'mail',
  display_name_attr: 'cn',
  is_enabled: false,
  is_exclusive: false,
  use_starttls: false,
  connect_timeout: 10,
})

onMounted(async () => {
  await loadConfig()
})

async function loadConfig() {
  loading.value = true
  try {
    const response = await adminApi.getLdapConfig()
    ldapConfig.value = {
      server_url: response.server_url || '',
      bind_dn: response.bind_dn || '',
      bind_password: '',
      base_dn: response.base_dn || '',
      user_search_filter: response.user_search_filter || '(uid={username})',
      username_attr: response.username_attr || 'uid',
      email_attr: response.email_attr || 'mail',
      display_name_attr: response.display_name_attr || 'cn',
      is_enabled: response.is_enabled || false,
      is_exclusive: response.is_exclusive || false,
      use_starttls: response.use_starttls || false,
      connect_timeout: response.connect_timeout || 10,
    }
    hasPassword.value = !!response.has_bind_password
  } catch (err) {
    error(t('ui.failedToLoadLdapConfiguration'))
    log.error(t('ui.failedToLoadLdapConfiguration'), err)
  } finally {
    loading.value = false
  }
}

async function handleSave() {
  saveLoading.value = true
  try {
    const payload: LdapConfigUpdateRequest = {
      server_url: ldapConfig.value.server_url,
      bind_dn: ldapConfig.value.bind_dn,
      base_dn: ldapConfig.value.base_dn,
      user_search_filter: ldapConfig.value.user_search_filter,
      username_attr: ldapConfig.value.username_attr,
      email_attr: ldapConfig.value.email_attr,
      display_name_attr: ldapConfig.value.display_name_attr,
      is_enabled: ldapConfig.value.is_enabled,
      is_exclusive: ldapConfig.value.is_exclusive,
      use_starttls: ldapConfig.value.use_starttls,
      connect_timeout: ldapConfig.value.connect_timeout,
    }

    // 只有输入了新密码才更新密码
    if (ldapConfig.value.bind_password) {
      payload.bind_password = ldapConfig.value.bind_password
    }

    await adminApi.updateLdapConfig(payload)
    success(t('ui.ldapConfigurationSaved'))

    if (ldapConfig.value.bind_password) {
      hasPassword.value = true
    }
    ldapConfig.value.bind_password = ''
  } catch (err) {
    error(t('ui.failedToSaveLdapConfiguration'))
    log.error(t('ui.failedToSaveLdapConfiguration'), err)
  } finally {
    saveLoading.value = false
  }
}

async function handleTestConnection() {
  testLoading.value = true
  try {
    const payload: LdapConfigUpdateRequest = {
      server_url: ldapConfig.value.server_url,
      bind_dn: ldapConfig.value.bind_dn,
      base_dn: ldapConfig.value.base_dn,
      user_search_filter: ldapConfig.value.user_search_filter,
      username_attr: ldapConfig.value.username_attr,
      email_attr: ldapConfig.value.email_attr,
      display_name_attr: ldapConfig.value.display_name_attr,
      is_enabled: ldapConfig.value.is_enabled,
      is_exclusive: ldapConfig.value.is_exclusive,
      use_starttls: ldapConfig.value.use_starttls,
      connect_timeout: ldapConfig.value.connect_timeout,
      ...(ldapConfig.value.bind_password && { bind_password: ldapConfig.value.bind_password }),
    }
    const response = await adminApi.testLdapConnection(payload)
    if (response.success) {
      success(t('ui.ldapConnectionTestSucceeded'))
    } else {
      error(`LDAP 连接测试失败: ${response.message}`)
    }
  } catch (err) {
    error(t('ui.ldapConnectionTestFailed'))
    log.error(t('ui.ldapConnectionTestFailed'), err)
  } finally {
    testLoading.value = false
  }
}

</script>
