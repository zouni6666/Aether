<template>
  <div class="mb-4 flex items-start justify-between gap-3">
    <div class="min-w-0">
      <h4 class="text-balance break-words text-base font-semibold text-foreground">
        {{ editing ? t('ui.editGroup') : t('ui.createGroup') }}
      </h4>
      <p class="text-xs text-muted-foreground">
        {{ isDefault ? t('ui.currentlyTheDefaultGroupForAllUsers') : t('ui.configureAccessLimitsThroughAdditionalGroups') }}
      </p>
    </div>
    <div
      v-if="editing"
      class="flex items-center gap-1"
    >
      <Button
        variant="ghost"
        size="icon"
        class="h-10 w-10"
        :class="isDefault ? 'text-emerald-500 hover:text-emerald-500' : ''"
        :disabled="saving || isDefault"
        :title="isDefault ? t('ui.defaultRegistrationGroup') : t('ui.setAsDefaultRegistrationGroup')"
        @click="$emit('setDefault')"
      >
        <BadgeCheck class="h-4 w-4" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="h-10 w-10"
        :disabled="saving || isDefault"
        :title="t('ui.deleteGroup')"
        @click="$emit('delete')"
      >
        <Trash2 class="h-4 w-4" />
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { BadgeCheck, Trash2 } from 'lucide-vue-next'
import { Button } from '@/components/ui'
import { useI18n } from '@/i18n'

defineProps<{
  editing: boolean
  isDefault: boolean
  saving: boolean
}>()

defineEmits<{
  setDefault: []
  delete: []
}>()

const { t } = useI18n()
</script>
