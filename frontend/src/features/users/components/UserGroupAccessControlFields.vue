<template>
  <div class="space-y-4 border-t border-border/60 pt-5">
    <div class="flex items-center justify-between gap-2 border-b border-border/60 pb-2">
      <span class="text-sm font-medium">{{ t('ui.groupPermissions') }}</span>
      <span class="flex items-center gap-1 text-[11px] text-muted-foreground">
        {{ t('ui.groupPermissionsAccumulateKeysCanNarrowThemFurther') }}
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger as-child>
              <button
                type="button"
                class="inline-flex h-10 w-10 items-center justify-center rounded-full text-muted-foreground outline-none transition-colors hover:bg-muted/60 hover:text-primary focus-visible:ring-2 focus-visible:ring-ring"
                :title="helpText"
                :aria-label="t('ui.viewGroupPermissionMergeRules')"
              >
                <Info class="h-4 w-4" />
              </button>
            </TooltipTrigger>
            <TooltipContent class="max-w-72 text-xs leading-5">
              {{ helpText }}
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </span>
    </div>

    <div class="space-y-2">
      <Label class="text-sm font-medium">{{ t('ui.allowedProviders') }}</Label>
      <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
        <div class="flex min-h-10 w-full items-center gap-2 sm:w-auto sm:shrink-0">
          <Switch
            :model-value="form.allowed_providers_mode === 'unrestricted'"
            @update:model-value="setProvidersUnrestricted"
          />
          <span class="text-xs text-muted-foreground sm:sr-only">
            {{ form.allowed_providers_mode === 'unrestricted' ? t('ui.noRestriction') : t('ui.selectProvider') }}
          </span>
        </div>
        <div class="min-w-0 flex-1">
          <MultiSelect
            :model-value="form.allowed_providers"
            :options="providerOptions"
            :search-threshold="0"
            :disabled="form.allowed_providers_mode === 'unrestricted'"
            :placeholder="form.allowed_providers_mode === 'unrestricted' ? t('ui.noRestrictionOnAnyOption') : t('ui.selectProvider')"
            :empty-text="t('ui.noOptions')"
            @update:model-value="(value) => updateForm({ allowed_providers: value })"
          />
        </div>
      </div>
    </div>

    <div class="space-y-2">
      <Label class="text-sm font-medium">{{ t('ui.allowedEndpoints') }}</Label>
      <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
        <div class="flex min-h-10 w-full items-center gap-2 sm:w-auto sm:shrink-0">
          <Switch
            :model-value="form.allowed_api_formats_mode === 'unrestricted'"
            @update:model-value="setApiFormatsUnrestricted"
          />
          <span class="text-xs text-muted-foreground sm:sr-only">
            {{ form.allowed_api_formats_mode === 'unrestricted' ? t('ui.noRestriction') : t('ui.selectEndpoint') }}
          </span>
        </div>
        <div class="min-w-0 flex-1">
          <MultiSelect
            :model-value="form.allowed_api_formats"
            :options="apiFormatOptions"
            :search-threshold="0"
            :disabled="form.allowed_api_formats_mode === 'unrestricted'"
            :placeholder="form.allowed_api_formats_mode === 'unrestricted' ? t('ui.noRestrictionOnAnyOption') : t('ui.selectEndpoint')"
            :empty-text="t('ui.noOptions')"
            @update:model-value="(value) => updateForm({ allowed_api_formats: value })"
          />
        </div>
      </div>
    </div>

    <div class="space-y-2">
      <Label class="text-sm font-medium">{{ t('ui.allowedModels879ffef') }}</Label>
      <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
        <div class="flex min-h-10 w-full items-center gap-2 sm:w-auto sm:shrink-0">
          <Switch
            :model-value="form.allowed_models_mode === 'unrestricted'"
            @update:model-value="setModelsUnrestricted"
          />
          <span class="text-xs text-muted-foreground sm:sr-only">
            {{ form.allowed_models_mode === 'unrestricted' ? t('ui.noRestriction') : t('ui.selectModel') }}
          </span>
        </div>
        <div class="min-w-0 flex-1">
          <MultiSelect
            :model-value="form.allowed_models"
            :options="modelOptions"
            :search-threshold="0"
            :disabled="form.allowed_models_mode === 'unrestricted'"
            :placeholder="form.allowed_models_mode === 'unrestricted' ? t('ui.noRestrictionOnAnyOption') : t('ui.selectModel')"
            :empty-text="t('ui.noOptions')"
            @update:model-value="(value) => updateForm({ allowed_models: value })"
          />
        </div>
      </div>
    </div>

    <div class="space-y-2">
      <Label class="text-sm font-medium">{{ t('ui.rateLimitRequestsMin') }}</Label>
      <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
        <div class="flex min-h-10 w-full items-center gap-2 sm:w-auto sm:shrink-0">
          <Switch
            :model-value="form.rate_limit_mode === 'system'"
            @update:model-value="setSystemRateLimit"
          />
          <span class="text-xs text-muted-foreground sm:sr-only">
            {{ form.rate_limit_mode === 'system' ? t('ui.systemDefault') : t('ui.custom4eafa9e') }}
          </span>
        </div>
        <div class="min-w-0 flex-1">
          <Input
            :model-value="form.rate_limit ?? ''"
            type="number"
            min="0"
            max="10000"
            class="h-10"
            :disabled="form.rate_limit_mode === 'system'"
            :placeholder="form.rate_limit_mode === 'system' ? t('ui.useSystemDefault') : t('ui.0NoRateLimit')"
            @update:model-value="updateRateLimit"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Info } from 'lucide-vue-next'
import {
  Input,
  Label,
  Switch,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui'
import { MultiSelect } from '@/components/common'
import { parseNumberInput } from '@/utils/form'
import { useI18n } from '@/i18n'
import type { UserGroupFormState, UserSelectOption } from './user-management-types'

const props = defineProps<{
  form: UserGroupFormState
  providerOptions: UserSelectOption[]
  apiFormatOptions: UserSelectOption[]
  modelOptions: UserSelectOption[]
  helpText: string
}>()

const emit = defineEmits<{
  'update:form': [value: UserGroupFormState]
}>()

const { t } = useI18n()

function updateForm(patch: Partial<UserGroupFormState>): void {
  emit('update:form', { ...props.form, ...patch })
}

function setProvidersUnrestricted(value: boolean): void {
  updateForm({ allowed_providers_mode: value ? 'unrestricted' : 'specific' })
}

function setApiFormatsUnrestricted(value: boolean): void {
  updateForm({ allowed_api_formats_mode: value ? 'unrestricted' : 'specific' })
}

function setModelsUnrestricted(value: boolean): void {
  updateForm({ allowed_models_mode: value ? 'unrestricted' : 'specific' })
}

function setSystemRateLimit(value: boolean): void {
  updateForm({ rate_limit_mode: value ? 'system' : 'custom' })
}

function updateRateLimit(value: string | number): void {
  updateForm({ rate_limit: parseNumberInput(value, { min: 0, max: 10000 }) })
}
</script>
