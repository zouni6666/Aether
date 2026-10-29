<template>
  <CardSection
    :title="t('ui.systemInformation')"
    :description="t('ui.currentSystemVersionAndBuildInformation')"
  >
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-2">
        <Label class="text-sm font-medium text-muted-foreground">{{ t('ui.version5c21dcf') }}</Label>
        <span
          v-if="systemVersion"
          class="text-sm font-mono"
        >
          {{ systemVersion }}
        </span>
        <span
          v-else
          class="text-sm text-muted-foreground"
        > {{ t('common.loading') }} </span>
      </div>
    </div>
  </CardSection>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import Label from '@/components/ui/label.vue'
import { CardSection } from '@/components/layout'

defineProps<{
  systemVersion: string
}>()

const { t } = useI18n()

</script>
