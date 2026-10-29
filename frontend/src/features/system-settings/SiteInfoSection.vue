<template>
  <CardSection
    :title="t('ui.siteInformation')"
    :description="t('ui.customizeTheSiteNameAndSubtitleUsedAcrossTheNavigation')"
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
          for="site-name"
          class="block text-sm font-medium"
        > {{ t('ui.siteName') }} </Label>
        <Input
          id="site-name"
          :model-value="siteName"
          type="text"
          placeholder="Aether"
          class="mt-1"
          @update:model-value="$emit('update:siteName', $event)"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.shownInTheNavigationBarSignInPageTitleAnd') }}
        </p>
      </div>
      <div>
        <Label
          for="site-subtitle"
          class="block text-sm font-medium"
        > {{ t('ui.siteSubtitle') }} </Label>
        <Input
          id="site-subtitle"
          :model-value="siteSubtitle"
          type="text"
          placeholder="AI Gateway"
          class="mt-1"
          @update:model-value="$emit('update:siteSubtitle', $event)"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.shownBelowTheBrandNameInTheNavigationBar') }}
        </p>
      </div>
    </div>
  </CardSection>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import { CardSection } from '@/components/layout'

defineProps<{
  siteName: string
  siteSubtitle: string
  loading: boolean
  hasChanges: boolean
}>()

defineEmits<{
  save: []
  'update:siteName': [value: string]
  'update:siteSubtitle': [value: string]
}>()

const { t } = useI18n()

</script>
