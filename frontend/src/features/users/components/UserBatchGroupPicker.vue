<template>
  <div class="grid gap-2 rounded-xl border border-border/70 bg-muted/20 p-3 sm:grid-cols-[9rem_minmax(0,1fr)] sm:items-start">
    <div>
      <Label class="text-sm font-medium">{{ t('ui.selectByGroup') }}</Label>
      <p class="mt-1 text-[11px] text-muted-foreground">
        {{ t('ui.canBeCombinedWithDirectUsersOrFilters') }}
      </p>
    </div>
    <MultiSelect
      :model-value="modelValue"
      :options="groupOptions"
      :search-threshold="0"
      :placeholder="t('ui.selectOneOrMoreGroupse395fd4')"
      :empty-text="t('ui.noUserGroups')"
      @update:model-value="$emit('update:modelValue', $event)"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Label } from '@/components/ui'
import { MultiSelect } from '@/components/common'
import { useI18n } from '@/i18n'
import type { UserGroup } from '@/api/users'

const props = defineProps<{
  modelValue: string[]
  groups: UserGroup[]
}>()

defineEmits<{
  'update:modelValue': [value: string[]]
}>()

const { t } = useI18n()

const groupOptions = computed(() => props.groups.map((group) => ({
  label: `${group.name}${group.is_default ? ` (${t('ui.default844b8cc')})` : ''}`,
  value: group.id,
})))
</script>
