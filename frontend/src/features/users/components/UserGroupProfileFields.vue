<template>
  <div class="space-y-4">
    <div class="space-y-2">
      <Label class="text-sm font-medium">{{ t('stats.column.name') }}</Label>
      <Input
        :model-value="name"
        class="h-10"
        :placeholder="t('ui.exampleProductionTeam')"
        @update:model-value="$emit('update:name', $event)"
      />
    </div>

    <div class="space-y-2">
      <Label class="text-sm font-medium">{{ t('stats.column.members') }}</Label>
      <MultiSelect
        :model-value="memberUserIds"
        :options="userOptions"
        :search-threshold="0"
        :disabled="membersDisabled"
        :placeholder="t('userStats.select.user')"
        :empty-text="t('ui.noUsers')"
        :no-results-text="t('ui.noMatchingUsersFound8546f1e')"
        @update:model-value="$emit('update:memberUserIds', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Input, Label } from '@/components/ui'
import { MultiSelect } from '@/components/common'
import { useI18n } from '@/i18n'
import type { UserSelectOption } from './user-management-types'

defineProps<{
  name: string
  memberUserIds: string[]
  userOptions: UserSelectOption[]
  membersDisabled: boolean
}>()

defineEmits<{
  'update:name': [value: string]
  'update:memberUserIds': [value: string[]]
}>()

const { t } = useI18n()
</script>
