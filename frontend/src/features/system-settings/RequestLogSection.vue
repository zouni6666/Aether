<template>
  <CardSection
    :title="t('ui.requestRecords')"
    :description="t('ui.controlHowRequestResponseDetailsAreStoredAndWhatContent')"
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
          for="request-log-level"
          class="block text-sm font-medium mb-2"
        > {{ t('ui.recordDetailLevel') }} </Label>
        <Select
          :model-value="requestRecordLevel"
          @update:model-value="$emit('update:requestRecordLevel', $event)"
        >
          <SelectTrigger
            id="request-log-level"
            class="mt-1"
          >
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="basic">
              {{ t('ui.basicBasicInformation1kbItem') }}
            </SelectItem>
            <SelectItem value="headers">
              {{ t('ui.headersIncludesRequestHeaders23kbItem') }}
            </SelectItem>
            <SelectItem value="full">
              {{ t('ui.fullFullRequestAndResponse') }}
            </SelectItem>
          </SelectContent>
        </Select>
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.sensitiveInformationIsAutomaticallyRedacted') }}
        </p>
      </div>

      <div>
        <Label
          for="sensitive-headers"
          class="block text-sm font-medium"
        > {{ t('ui.sensitiveRequestHeaders') }} </Label>
        <Input
          id="sensitive-headers"
          :model-value="sensitiveHeadersStr"
          placeholder="authorization, x-api-key, cookie"
          class="mt-1"
          @update:model-value="$emit('update:sensitiveHeadersStr', $event)"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.commaSeparatedRequestHeadersThatWillBeRedacted') }}
        </p>
      </div>

      <div class="flex items-start gap-3 pt-1">
        <Switch
          id="compact-base64-images"
          :model-value="compactBase64Images"
          :disabled="loading"
          @update:model-value="$emit('update:compactBase64Images', $event)"
        />
        <div>
          <Label
            for="compact-base64-images"
            class="text-sm font-medium cursor-pointer"
          > {{ t('ui.compactBase64Images') }} </Label>
          <p class="mt-1 text-xs text-muted-foreground">
            {{ t('ui.replaceDataImageBase64ImagesWithPlaceholdersThatShowTheir') }}
          </p>
        </div>
      </div>

      <div>
        <Label
          for="request-record-max-body-size-kb"
          class="block text-sm font-medium"
        > {{ t('ui.bodySizeLimitKb') }} </Label>
        <div class="relative mt-1">
          <Input
            id="request-record-max-body-size-kb"
            :model-value="maxBodySizeKb"
            type="number"
            min="0"
            max="1048576"
            step="1"
            class="pr-12"
            @update:model-value="$emit('update:maxBodySizeKb', Math.min(1048576, Math.max(0, Math.trunc(Number($event) || 0))))"
          />
          <span class="absolute inset-y-0 right-3 flex items-center text-xs text-muted-foreground pointer-events-none">
            KB
          </span>
        </div>
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('ui.limitEachRequestResponseBodyIndependently0MeansUnlimited') }}
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
import Switch from '@/components/ui/switch.vue'
import Select from '@/components/ui/select.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectValue from '@/components/ui/select-value.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import { CardSection } from '@/components/layout'

defineProps<{
  requestRecordLevel: string
  sensitiveHeadersStr: string
  compactBase64Images: boolean
  maxBodySizeKb: number
  loading: boolean
  hasChanges: boolean
}>()

defineEmits<{
  save: []
  'update:requestRecordLevel': [value: string]
  'update:sensitiveHeadersStr': [value: string]
  'update:compactBase64Images': [value: boolean]
  'update:maxBodySizeKb': [value: number]
}>()

const { t } = useI18n()

</script>
