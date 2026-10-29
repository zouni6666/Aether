<template>
  <Dialog
    :model-value="open"
    :title="t('ui.userGroupscf4c1d2')"
    :description="t('ui.manageUserGroupsTheDefaultRegistrationGroupMembersAndGroup')"
    size="4xl"
    persistent
    @update:model-value="handleDialogUpdate"
  >
    <div class="grid gap-3 lg:min-h-[560px] lg:grid-cols-[17rem_minmax(0,1fr)] lg:gap-4">
      <UserGroupListPanel
        :loading="loading"
        :groups="groups"
        :selected-group-id="editingGroupId"
        @create="startCreate"
        @select="selectGroup"
      />

      <div class="min-w-0 bg-background sm:rounded-xl sm:border sm:border-border/70 sm:p-4">
        <UserGroupEditorHeader
          :editing="Boolean(editingGroupId)"
          :is-default="Boolean(selectedGroup?.is_default)"
          :saving="saving"
          @set-default="toggleDefault"
          @delete="deleteSelectedGroup"
        />

        <div class="space-y-5">
          <UserGroupProfileFields
            :name="form.name"
            :member-user-ids="memberUserIds"
            :user-options="userOptions"
            :members-disabled="Boolean(selectedGroup?.is_default)"
            @update:name="form.name = $event"
            @update:member-user-ids="memberUserIds = $event"
          />

          <UserGroupAccessControlFields
            v-model:form="form"
            :provider-options="providerOptions"
            :api-format-options="apiFormatOptions"
            :model-options="modelOptions"
            :help-text="groupPolicyHelpTextLocalized"
          />
        </div>
      </div>
    </div>

    <template #footer>
      <Button
        variant="outline"
        :disabled="saving"
        @click="emit('close')"
      >
        {{ t('ui.close') }}
      </Button>
      <Button
        :disabled="saving || !form.name.trim()"
        @click="saveGroup"
      >
        {{ t('ui.save') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  Button,
  Dialog,
} from '@/components/ui'
import { useUsersStore } from '@/stores/users'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { parseApiError } from '@/utils/errorParser'
import { useI18n } from '@/i18n'
import { useUserAccessControlOptions } from '@/features/users/composables/useUserAccessControlOptions'
import UserGroupAccessControlFields from './UserGroupAccessControlFields.vue'
import UserGroupEditorHeader from './UserGroupEditorHeader.vue'
import UserGroupListPanel from './UserGroupListPanel.vue'
import UserGroupProfileFields from './UserGroupProfileFields.vue'
import type {
  ListPolicyMode,
  RateLimitPolicyMode,
  UpsertUserGroupRequest,
  User,
  UserGroup,
} from '@/api/users'
import type { UserGroupFormState } from './user-management-types'

const props = defineProps<{
  open: boolean
  usersVersion: number
}>()

const emit = defineEmits<{
  close: []
  changed: []
}>()

const usersStore = useUsersStore()
const { success, error } = useToast()
const { confirmDanger, confirmInfo } = useConfirm()
const { t } = useI18n()
const {
  providerOptions,
  apiFormatOptions,
  modelOptions,
  loadAccessControlOptions,
} = useUserAccessControlOptions()

const loading = ref(false)
const saving = ref(false)
const groups = ref<UserGroup[]>([])
const dialogUsers = ref<User[]>([])
const editingGroupId = ref<string | null>(null)
const memberUserIds = ref<string[]>([])
const USER_OPTIONS_CACHE_TTL_MS = 30 * 1000
let dialogUsersLoadedAt = 0
let dialogUsersLoadedVersion = -1

const groupPolicyHelpTextLocalized = computed(() => t('ui.modelsProvidersAndEndpointsAccumulateAcrossMultipleUserGroupsUnrestricted'))

const form = ref<UserGroupFormState>(createEmptyForm())

const selectedGroup = computed(() => groups.value.find((group) => group.id === editingGroupId.value) ?? null)
const userOptions = computed(() => dialogUsers.value.map((user) => ({
  label: `${user.username}${user.email ? ` (${user.email})` : ''}`,
  value: user.id,
})))

watch(
  () => props.open,
  (open) => {
    if (!open) return
    void loadDialogData()
    void loadAccessControlOptions().catch((err) => {
      error(parseApiError(err, t('ui.failedToLoadAccessControlOptions')), t('ui.failedToLoadAccessControlOptions'))
    })
  },
)

function handleDialogUpdate(value: boolean): void {
  if (!value) emit('close')
}

async function loadDialogData(): Promise<void> {
  loading.value = true
  try {
    const [groupsResponse] = await Promise.all([
      usersStore.listUserGroups(),
      ensureDialogUsers(),
    ])
    groups.value = groupsResponse.items
    if (editingGroupId.value && !groups.value.some((group) => group.id === editingGroupId.value)) {
      editingGroupId.value = null
    }
    const nextGroup = editingGroupId.value
      ? groups.value.find((group) => group.id === editingGroupId.value) ?? null
      : groups.value[0] ?? null
    if (nextGroup) {
      await selectGroup(nextGroup.id)
    } else {
      startCreate()
    }
  } catch (err) {
    error(parseApiError(err, t('ui.failedToLoadUserGroups')), t('ui.failedToLoadUserGroups'))
  } finally {
    loading.value = false
  }
}

async function ensureDialogUsers(): Promise<void> {
  const now = Date.now()
  const isSameUserVersion = dialogUsersLoadedVersion === props.usersVersion
  if (
    isSameUserVersion
    && dialogUsersLoadedAt > 0
    && now - dialogUsersLoadedAt < USER_OPTIONS_CACHE_TTL_MS
  ) {
    return
  }

  dialogUsers.value = await usersStore.listAllUsers({
    cacheTtlMs: isSameUserVersion ? USER_OPTIONS_CACHE_TTL_MS : 0,
    cacheKeySuffix: isSameUserVersion ? undefined : `users-version-${props.usersVersion}`,
  })
  dialogUsersLoadedAt = Date.now()
  dialogUsersLoadedVersion = props.usersVersion
}

async function selectGroup(groupId: string): Promise<void> {
  const group = groups.value.find((item) => item.id === groupId)
  if (!group) return
  editingGroupId.value = group.id
  form.value = {
    name: group.name,
    allowed_providers_mode: normalizeListMode(group.allowed_providers_mode),
    allowed_api_formats_mode: normalizeListMode(group.allowed_api_formats_mode),
    allowed_models_mode: normalizeListMode(group.allowed_models_mode),
    allowed_providers: group.allowed_providers ? [...group.allowed_providers] : [],
    allowed_api_formats: group.allowed_api_formats ? [...group.allowed_api_formats] : [],
    allowed_models: group.allowed_models ? [...group.allowed_models] : [],
    rate_limit_mode: normalizeRateMode(group.rate_limit_mode),
    rate_limit: group.rate_limit ?? undefined,
  }
  try {
    const members = await usersStore.listUserGroupMembers(group.id)
    memberUserIds.value = members.map((member) => member.user_id)
  } catch (err) {
    memberUserIds.value = []
    error(parseApiError(err, t('ui.failedToLoadGroupMembers')), t('ui.failedToLoadGroupMembers'))
  }
}

function normalizeListMode(mode: ListPolicyMode): ListPolicyMode {
  return mode === 'specific' ? 'specific' : 'unrestricted'
}

function normalizeRateMode(mode: RateLimitPolicyMode): RateLimitPolicyMode {
  return mode === 'custom' ? 'custom' : 'system'
}

function createEmptyForm(): UserGroupFormState {
  return {
    name: '',
    allowed_providers_mode: 'unrestricted',
    allowed_api_formats_mode: 'unrestricted',
    allowed_models_mode: 'unrestricted',
    allowed_providers: [],
    allowed_api_formats: [],
    allowed_models: [],
    rate_limit_mode: 'system',
    rate_limit: undefined,
  }
}

function startCreate(): void {
  editingGroupId.value = null
  form.value = createEmptyForm()
  memberUserIds.value = []
}

async function toggleDefault(): Promise<void> {
  const group = selectedGroup.value
  if (!group || group.is_default) return
  const confirmed = await confirmInfo(
    t('users.groups.setDefaultConfirm', { name: group.name }),
    t('ui.setAsDefaultRegistrationGroup'),
  )
  if (!confirmed) return
  saving.value = true
  try {
    await usersStore.setDefaultUserGroup(group.id)
    success(t('ui.defaultRegistrationGroupUpdated'))
    emit('changed')
    await loadDialogData()
  } catch (err) {
    error(parseApiError(err, t('ui.failedToSetDefaultRegistrationGroup')), t('ui.failedToSetDefaultRegistrationGroup'))
  } finally {
    saving.value = false
  }
}

function buildPayload(): UpsertUserGroupRequest {
  return {
    name: form.value.name.trim(),
    allowed_providers_mode: form.value.allowed_providers_mode,
    allowed_api_formats_mode: form.value.allowed_api_formats_mode,
    allowed_models_mode: form.value.allowed_models_mode,
    allowed_providers: form.value.allowed_providers_mode === 'specific'
      ? [...form.value.allowed_providers]
      : null,
    allowed_api_formats: form.value.allowed_api_formats_mode === 'specific'
      ? [...form.value.allowed_api_formats]
      : null,
    allowed_models: form.value.allowed_models_mode === 'specific'
      ? [...form.value.allowed_models]
      : null,
    rate_limit_mode: form.value.rate_limit_mode,
    rate_limit: form.value.rate_limit_mode === 'custom'
      ? (form.value.rate_limit ?? 0)
      : null,
  }
}

async function saveGroup(): Promise<void> {
  if (!form.value.name.trim()) return
  saving.value = true
  try {
    const saved = editingGroupId.value
      ? await usersStore.updateUserGroup(editingGroupId.value, buildPayload())
      : await usersStore.createUserGroup(buildPayload())
    if (!saved.is_default) {
      await usersStore.replaceUserGroupMembers(saved.id, memberUserIds.value)
    }
    success(t('ui.userGroupSaved'))
    emit('changed')
    editingGroupId.value = saved.id
    await loadDialogData()
  } catch (err) {
    error(parseApiError(err, t('ui.failedToSaveUserGroup')), t('ui.failedToSaveUserGroup'))
  } finally {
    saving.value = false
  }
}

async function deleteSelectedGroup(): Promise<void> {
  if (!selectedGroup.value) return
  const group = selectedGroup.value
  const confirmed = await confirmDanger(
    t('users.groups.deleteConfirm', { name: group.name }),
    t('ui.deleteUserGroup'),
  )
  if (!confirmed) return
  saving.value = true
  try {
    await usersStore.deleteUserGroup(group.id)
    success(t('ui.userGroupDeleted'))
    emit('changed')
    editingGroupId.value = null
    await loadDialogData()
  } catch (err) {
    error(parseApiError(err, t('ui.failedToDeleteUserGroup')), t('ui.failedToDeleteUserGroup'))
  } finally {
    saving.value = false
  }
}
</script>
