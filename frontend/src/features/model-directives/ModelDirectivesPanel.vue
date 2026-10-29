<template>
  <div class="space-y-4">
    <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <h3 class="text-base font-semibold">
          {{ t('ui.reasoningParameterMappings') }}
        </h3>
        <p class="mt-1 text-sm text-muted-foreground">
          {{ t('ui.configureModelSuffixesAndRequestParameterMappingsByApiEndpoint') }}
        </p>
      </div>
      <div class="flex items-center">
        <Switch
          :model-value="config.reasoning_effort.enabled"
          :disabled="loading"
          :aria-label="t('ui.enableReasoningParameterMapping')"
          @update:model-value="onReasoningEnabledChange"
        />
      </div>
    </div>

    <div class="overflow-hidden rounded-lg border">
      <div class="hidden gap-3 border-b bg-muted/40 px-4 py-3 text-xs font-medium text-muted-foreground lg:grid lg:grid-cols-[minmax(0,1fr)_minmax(14rem,1fr)_minmax(0,2fr)_auto]">
        <div>{{ t('ui.apiEndpoint') }}</div>
        <div>{{ t('ui.modelSuffix') }}</div>
        <div>{{ t('ui.effectiveMapping') }}</div>
        <div class="text-right">
          {{ t('ui.status6320b4a') }}
        </div>
      </div>
      <div class="divide-y">
        <div
          v-for="format in MODEL_DIRECTIVE_API_FORMATS"
          :key="format.key"
          class="grid grid-cols-1 items-start gap-3 px-4 py-3 lg:grid-cols-[minmax(0,1fr)_minmax(14rem,1fr)_minmax(0,2fr)_auto]"
        >
          <div class="min-w-0">
            <div class="mb-1 text-xs font-medium text-muted-foreground lg:hidden">
              {{ t('ui.apiEndpoint') }}
            </div>
            <div class="text-sm font-medium">
              {{ format.label }}
            </div>
            <code class="mt-1 block max-w-full !whitespace-normal break-all text-xs text-muted-foreground">
              {{ format.parameter }}
            </code>
          </div>
          <div>
            <div class="mb-1 text-xs font-medium text-muted-foreground lg:hidden">
              {{ t('ui.modelSuffix') }}
            </div>
            <div class="flex items-center gap-1.5">
              <Select
                :model-value="selectedSuffixes[format.key] ?? PREFERRED_MODEL_DIRECTIVE_SUFFIX"
                :disabled="loading || !config.reasoning_effort.enabled || !formatConfig(format.key).enabled"
                @update:model-value="value => selectSuffix(format.key, value)"
              >
                <SelectTrigger
                  class="h-9 min-w-0 flex-1 rounded-lg"
                  :aria-label="t('ui.valueModelSuffix', { value: format.label })"
                >
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem
                    v-for="suffix in availableSuffixes(format.key)"
                    :key="suffix"
                    :value="suffix"
                    :text-value="suffixLabel(suffix)"
                  >
                    {{ suffixLabel(suffix) }}
                  </SelectItem>
                </SelectContent>
              </Select>
              <Button
                size="icon"
                variant="ghost"
                class="h-9 w-9 shrink-0 text-muted-foreground"
                :disabled="loading || !config.reasoning_effort.enabled || !formatConfig(format.key).enabled"
                :title="t('ui.createACustomValueSuffix', { value: format.label })"
                :aria-label="t('ui.createACustomValueSuffix', { value: format.label })"
                @click="startAddingSuffix(format.key)"
              >
                <Plus class="h-4 w-4" />
              </Button>
            </div>
            <div
              v-if="addingSuffixes.has(format.key)"
              class="mt-2"
            >
              <div class="flex items-center gap-1.5">
                <Input
                  v-model="customSuffixDrafts[format.key]"
                  class="h-9 min-w-0 flex-1 font-mono text-xs"
                  :disabled="loading"
                  :aria-label="t('ui.customValueSuffixName', { value: format.label })"
                  placeholder="vendor-option"
                  @keyup.enter="addCustomSuffix(format.key)"
                  @keyup.esc="cancelAddingSuffix(format.key)"
                  @update:model-value="clearCustomSuffixError(format.key)"
                />
                <Button
                  size="icon"
                  variant="ghost"
                  class="h-9 w-9 shrink-0"
                  :disabled="loading"
                  :title="t('ui.addCustomValueSuffix', { value: format.label })"
                  :aria-label="t('ui.addCustomValueSuffix', { value: format.label })"
                  @click="addCustomSuffix(format.key)"
                >
                  <Check class="h-4 w-4" />
                </Button>
                <Button
                  size="icon"
                  variant="ghost"
                  class="h-9 w-9 shrink-0 text-muted-foreground"
                  :disabled="loading"
                  :title="t('ui.cancelCreatingACustomValueSuffix', { value: format.label })"
                  :aria-label="t('ui.cancelCreatingACustomValueSuffix', { value: format.label })"
                  @click="cancelAddingSuffix(format.key)"
                >
                  <X class="h-4 w-4" />
                </Button>
              </div>
              <p
                v-if="customSuffixErrors[format.key]"
                class="mt-1 text-xs text-destructive"
                role="alert"
              >
                {{ customSuffixErrors[format.key] }}
              </p>
            </div>
            <p class="mt-1 text-xs text-muted-foreground">
              {{ selectedSuffixDescription(format.key) }}
            </p>
            <div class="mt-2 flex items-center justify-between gap-2">
              <span class="text-xs text-muted-foreground">{{ t('ui.suffixStatus') }}</span>
              <Switch
                :model-value="selectedSuffixEnabled(format.key)"
                :disabled="loading || !config.reasoning_effort.enabled || !formatConfig(format.key).enabled || isPendingCustomSuffix(format.key)"
                :aria-label="t('ui.valueValueSuffix', { value: format.label, value2: selectedSuffixes[format.key] ?? PREFERRED_MODEL_DIRECTIVE_SUFFIX })"
                @update:model-value="value => onSuffixEnabledChange(format.key, value)"
              />
            </div>
          </div>
          <div>
            <div class="mb-1 text-xs font-medium text-muted-foreground lg:hidden">
              {{ t('ui.effectiveMapping') }}
            </div>
            <div class="flex items-start gap-2">
              <div class="min-w-0 flex-1">
                <Textarea
                  :id="mappingInputId(format.key)"
                  :model-value="localMappingParams[mappingKey(format.key)]"
                  class="h-24 min-h-24 resize-none overflow-auto font-mono text-xs leading-5"
                  :disabled="loading || !config.reasoning_effort.enabled || !formatConfig(format.key).enabled"
                  :aria-label="t('modelDirectives.mappingParameters', { format: format.label, suffix: selectedSuffixes[format.key] ?? PREFERRED_MODEL_DIRECTIVE_SUFFIX })"
                  :aria-invalid="Boolean(mappingErrors[mappingKey(format.key)])"
                  :aria-describedby="mappingErrors[mappingKey(format.key)] ? mappingErrorId(format.key) : undefined"
                  :title="t('modelDirectives.effectiveMappingJson')"
                  placeholder="{}"
                  @update:model-value="value => onMappingDraftChange(format.key, value)"
                />
                <p
                  v-if="mappingErrors[mappingKey(format.key)]"
                  :id="mappingErrorId(format.key)"
                  class="mt-1 text-xs text-destructive"
                  role="alert"
                >
                  {{ mappingErrors[mappingKey(format.key)] }}
                </p>
                <p
                  v-else
                  class="mt-1 text-xs text-muted-foreground"
                >
                  {{ mappingStatus(format.key) }}
                </p>
              </div>
              <Button
                v-if="hasCustomMapping(format.key) || isPendingCustomSuffix(format.key)"
                size="icon"
                variant="ghost"
                class="h-9 w-9 shrink-0 text-muted-foreground"
                :disabled="loading || !config.reasoning_effort.enabled || !formatConfig(format.key).enabled"
                :title="resetMappingTitle(format.key, format.label)"
                :aria-label="resetMappingTitle(format.key, format.label)"
                @click="resetMappingOverride(format.key)"
              >
                <RotateCcw class="h-4 w-4" />
              </Button>
              <Button
                size="icon"
                variant="ghost"
                class="h-9 w-9 shrink-0 text-muted-foreground"
                :disabled="loading || !config.reasoning_effort.enabled || !formatConfig(format.key).enabled || !hasMappingParamChanges(format.key)"
                :title="t('ui.saveValueMappingParameters', { value: format.label })"
                :aria-label="t('ui.saveValueMappingParameters', { value: format.label })"
                @click="saveMappingParam(format.key)"
              >
                <Save class="h-4 w-4" />
              </Button>
            </div>
          </div>
          <div class="flex items-center justify-between gap-3 lg:justify-end">
            <span class="text-xs font-medium text-muted-foreground lg:hidden">{{ t('ui.status6320b4a') }}</span>
            <Switch
              :model-value="formatConfig(format.key).enabled"
              :disabled="loading || !config.reasoning_effort.enabled"
              :aria-label="t('ui.valueEndpointParameterMapping', { value: format.label })"
              @update:model-value="value => onApiFormatEnabledChange(format.key, value)"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const props = defineProps<{
  config: ModelDirectivesConfig
  loading: boolean
}>()

const emit = defineEmits<{
  save: [value: ModelDirectivesConfig]
}>()

const { t } = useI18n()

import { reactive, watch } from 'vue'
import { Check, Plus, RotateCcw, Save, X } from 'lucide-vue-next'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui'
import Switch from '@/components/ui/switch.vue'
import Textarea from '@/components/ui/textarea.vue'
import {
  MODEL_DIRECTIVE_SUFFIX_METADATA,
  MODEL_DIRECTIVE_API_FORMATS,
  MODEL_DIRECTIVE_SUFFIXES,
  PREFERRED_MODEL_DIRECTIVE_SUFFIX,
  defaultModelDirectiveSuffixesForApiFormat,
  modelDirectiveEffectiveMappingPreview,
  modelDirectiveMappingOverrideFromEffective,
  normalizeModelDirectiveSuffix,
  updateModelDirectiveMappingOverride,
  updateModelDirectiveSuffixEnabled,
  type ReasoningApiFormatConfig,
  type ModelDirectivesConfig,
} from './modelDirectivesConfig'

const selectedSuffixes = reactive<Record<string, string>>({})
const selectedSuffixTouched = reactive(new Set<string>())
const localMappingParams = reactive<Record<string, string>>({})
const mappingErrors = reactive<Record<string, string>>({})
const dirtyMappingKeys = reactive(new Set<string>())
const addingSuffixes = reactive(new Set<string>())
const customSuffixDrafts = reactive<Record<string, string>>({})
const customSuffixErrors = reactive<Record<string, string>>({})
const localCustomSuffixes = reactive<Record<string, string[]>>({})

watch(() => props.config.reasoning_effort.api_formats, (newFormats) => {
  for (const format of MODEL_DIRECTIVE_API_FORMATS) {
    const fc = newFormats[format.key]
    const currentSelectedSuffix = selectedSuffixes[format.key]
    const selectedSuffix = selectedSuffixTouched.has(format.key)
      && currentSelectedSuffix
      && availableSuffixes(format.key).includes(currentSelectedSuffix)
      ? currentSelectedSuffix
      : preferredSuffix(format.key, fc)
    selectedSuffixes[format.key] = selectedSuffix
    for (const suffix of availableSuffixes(format.key)) {
      const key = mappingKey(format.key, suffix)
      const authoritativeText = effectiveMappingText(format.key, suffix, fc?.mappings?.[suffix])
      if (dirtyMappingKeys.has(key)) {
        if (localMappingParams[key] === authoritativeText) {
          dirtyMappingKeys.delete(key)
          delete mappingErrors[key]
        }
        continue
      }
      localMappingParams[key] = authoritativeText
      delete mappingErrors[key]
    }
  }
}, { immediate: true })

function formatConfig(apiFormat: string): ReasoningApiFormatConfig {
  return props.config.reasoning_effort.api_formats[apiFormat] ?? {
    enabled: true,
    suffixes: [...defaultModelDirectiveSuffixesForApiFormat(apiFormat)],
    mappings: {},
  }
}

function availableSuffixes(apiFormat: string): string[] {
  const configured = props.config.reasoning_effort.api_formats[apiFormat]
  return [...new Set([
    ...defaultModelDirectiveSuffixesForApiFormat(apiFormat),
    ...(configured?.suffixes ?? []),
    ...Object.keys(configured?.mappings ?? {}),
    ...(localCustomSuffixes[apiFormat] ?? []),
  ])]
}

function preferredSuffix(
  apiFormat: string,
  config: ReasoningApiFormatConfig | undefined,
): string {
  const suffixes = availableSuffixes(apiFormat)
  const configured = suffixes.find(suffix => (
    config?.suffixes.includes(suffix) || config?.mappings?.[suffix] !== undefined
  ))
  if (configured && suffixes.includes(PREFERRED_MODEL_DIRECTIVE_SUFFIX)) {
    return config?.suffixes.includes(PREFERRED_MODEL_DIRECTIVE_SUFFIX)
      || config?.mappings?.[PREFERRED_MODEL_DIRECTIVE_SUFFIX] !== undefined
      ? PREFERRED_MODEL_DIRECTIVE_SUFFIX
      : configured
  }
  return configured ?? (suffixes.includes(PREFERRED_MODEL_DIRECTIVE_SUFFIX)
    ? PREFERRED_MODEL_DIRECTIVE_SUFFIX
    : suffixes[0] ?? PREFERRED_MODEL_DIRECTIVE_SUFFIX)
}

function mappingKey(
  apiFormat: string,
  suffix: string = selectedSuffixes[apiFormat] ?? 'low',
): string {
  return `${apiFormat}:${suffix}`
}

function selectSuffix(apiFormat: string, suffix: string) {
  if (!availableSuffixes(apiFormat).includes(suffix)) {
    throw new Error(`Unsupported model directive suffix: ${suffix}`)
  }
  selectedSuffixes[apiFormat] = suffix
  selectedSuffixTouched.add(apiFormat)
  const key = mappingKey(apiFormat, suffix)
  if (!Object.prototype.hasOwnProperty.call(localMappingParams, key)) {
    localMappingParams[key] = effectiveMappingText(
      apiFormat,
      suffix,
      formatConfig(apiFormat).mappings[suffix],
    )
    delete mappingErrors[key]
  }
}

function onMappingDraftChange(apiFormat: string, value: string) {
  const suffix = selectedSuffixes[apiFormat] ?? 'low'
  const key = mappingKey(apiFormat, suffix)
  localMappingParams[key] = value
  if (value === effectiveMappingText(apiFormat, suffix, formatConfig(apiFormat).mappings[suffix])) {
    dirtyMappingKeys.delete(key)
  } else {
    dirtyMappingKeys.add(key)
  }
  delete mappingErrors[key]
}

function suffixLabel(suffix: string): string {
  return MODEL_DIRECTIVE_SUFFIX_METADATA[suffix as keyof typeof MODEL_DIRECTIVE_SUFFIX_METADATA]?.label
    ?? suffix
}

function selectedSuffixDescription(apiFormat: string): string {
  const suffix = selectedSuffixes[apiFormat] ?? 'low'
  return MODEL_DIRECTIVE_SUFFIX_METADATA[suffix as keyof typeof MODEL_DIRECTIVE_SUFFIX_METADATA]?.description
    ?? t('modelDirectives.customDirective')
}

function selectedSuffixEnabled(apiFormat: string): boolean {
  return formatConfig(apiFormat).suffixes.includes(selectedSuffixes[apiFormat] ?? 'low')
}

function mappingInputId(apiFormat: string): string {
  return `model-directive-mapping-${apiFormat.replace(/[^a-z0-9]+/gi, '-')}`
}

function mappingErrorId(apiFormat: string): string {
  return `${mappingInputId(apiFormat)}-error`
}

function effectiveMappingText(
  apiFormat: string,
  suffix: string,
  override: unknown,
): string {
  const mapping = modelDirectiveEffectiveMappingPreview(apiFormat, suffix, override)
  return mapping === undefined ? '' : JSON.stringify(mapping, null, 2)
}

function hasBuiltInMapping(apiFormat: string): boolean {
  const suffix = selectedSuffixes[apiFormat] ?? PREFERRED_MODEL_DIRECTIVE_SUFFIX
  return modelDirectiveEffectiveMappingPreview(apiFormat, suffix, undefined) !== undefined
}

function hasCustomMapping(apiFormat: string): boolean {
  const suffix = selectedSuffixes[apiFormat] ?? 'low'
  return Object.prototype.hasOwnProperty.call(formatConfig(apiFormat).mappings, suffix)
}

function mappingStatus(apiFormat: string): string {
  if (isPendingCustomSuffix(apiFormat)) return t('ui.saveANonEmptyMappingToEnableThisSuffix')
  if (hasCustomMapping(apiFormat) && hasBuiltInMapping(apiFormat)) {
    return t('ui.customOverrideMergedView')
  }
  if (hasCustomMapping(apiFormat)) return t('ui.customMapping')
  if (hasBuiltInMapping(apiFormat)) return t('ui.builtInMappingPreviewAdjustedForTheTargetModelAt')
  return t('ui.mappingNotConfigured')
}

function resetMappingTitle(apiFormat: string, formatLabel: string): string {
  if (isPendingCustomSuffix(apiFormat)) return t('modelDirectives.deletePendingSuffix', { format: formatLabel })
  return hasBuiltInMapping(apiFormat)
    ? t('modelDirectives.restoreBuiltinMapping', { format: formatLabel })
    : t('modelDirectives.deleteCustomMapping', { format: formatLabel })
}

function hasMappingParamChanges(apiFormat: string): boolean {
  const suffix = selectedSuffixes[apiFormat] ?? 'low'
  return (localMappingParams[mappingKey(apiFormat, suffix)] ?? '')
    !== effectiveMappingText(apiFormat, suffix, formatConfig(apiFormat).mappings[suffix])
}

function startAddingSuffix(apiFormat: string) {
  addingSuffixes.add(apiFormat)
  customSuffixDrafts[apiFormat] = ''
  delete customSuffixErrors[apiFormat]
}

function cancelAddingSuffix(apiFormat: string) {
  addingSuffixes.delete(apiFormat)
  delete customSuffixDrafts[apiFormat]
  delete customSuffixErrors[apiFormat]
}

function clearCustomSuffixError(apiFormat: string) {
  delete customSuffixErrors[apiFormat]
}

function addCustomSuffix(apiFormat: string) {
  const rawSuffix = customSuffixDrafts[apiFormat] ?? ''
  const suffix = normalizeModelDirectiveSuffix(rawSuffix)
  if (!suffix || suffix.startsWith('-') || suffix.endsWith('-') || /\s/.test(suffix)) {
    customSuffixErrors[apiFormat] = t('modelDirectives.invalidSuffix')
    return
  }
  if (MODEL_DIRECTIVE_SUFFIXES.includes(suffix as typeof MODEL_DIRECTIVE_SUFFIXES[number])
    && !defaultModelDirectiveSuffixesForApiFormat(apiFormat).includes(
      suffix as typeof MODEL_DIRECTIVE_SUFFIXES[number],
    )) {
    customSuffixErrors[apiFormat] = t('modelDirectives.unsupportedSuffix')
    return
  }

  const existing = availableSuffixes(apiFormat).find(item => item.toLowerCase() === suffix.toLowerCase())
  if (existing) {
    selectedSuffixes[apiFormat] = existing
    selectedSuffixTouched.add(apiFormat)
    cancelAddingSuffix(apiFormat)
    return
  }

  localCustomSuffixes[apiFormat] = [
    ...(localCustomSuffixes[apiFormat] ?? []),
    suffix,
  ]
  selectedSuffixes[apiFormat] = suffix
  selectedSuffixTouched.add(apiFormat)
  const key = mappingKey(apiFormat, suffix)
  localMappingParams[key] = ''
  dirtyMappingKeys.delete(key)
  delete mappingErrors[key]
  cancelAddingSuffix(apiFormat)
}

function isPendingCustomSuffix(apiFormat: string): boolean {
  const suffix = selectedSuffixes[apiFormat] ?? PREFERRED_MODEL_DIRECTIVE_SUFFIX
  return (localCustomSuffixes[apiFormat] ?? []).includes(suffix)
    && !formatConfig(apiFormat).suffixes.includes(suffix)
    && !Object.prototype.hasOwnProperty.call(formatConfig(apiFormat).mappings, suffix)
}

function removeLocalCustomSuffix(apiFormat: string, suffix: string) {
  localCustomSuffixes[apiFormat] = (localCustomSuffixes[apiFormat] ?? [])
    .filter(item => item !== suffix)
}

function onReasoningEnabledChange(value: boolean) {
  emit('save', {
    ...props.config,
    reasoning_effort: { ...props.config.reasoning_effort, enabled: Boolean(value) },
  })
}

function onApiFormatEnabledChange(apiFormat: string, value: boolean) {
  const current = formatConfig(apiFormat)
  emit('save', {
    ...props.config,
    reasoning_effort: {
      ...props.config.reasoning_effort,
      api_formats: {
        ...props.config.reasoning_effort.api_formats,
        [apiFormat]: { ...current, enabled: Boolean(value) },
      },
    },
  })
}

function onSuffixEnabledChange(apiFormat: string, value: boolean) {
  const current = formatConfig(apiFormat)
  const suffix = selectedSuffixes[apiFormat] ?? 'low'
  selectedSuffixTouched.add(apiFormat)
  emit('save', {
    ...props.config,
    reasoning_effort: {
      ...props.config.reasoning_effort,
      api_formats: {
        ...props.config.reasoning_effort.api_formats,
        [apiFormat]: {
          ...current,
          suffixes: updateModelDirectiveSuffixEnabled(current.suffixes, suffix, Boolean(value)),
        },
      },
    },
  })
}

function saveMappingParam(apiFormat: string) {
  const current = formatConfig(apiFormat)
  const suffix = selectedSuffixes[apiFormat] ?? 'low'
  const key = mappingKey(apiFormat, suffix)
  const rawMapping = (localMappingParams[key] ?? '').trim()
  let effectiveMapping: Record<string, unknown> | undefined
  if (rawMapping) {
    try {
      const parsed = JSON.parse(rawMapping)
      if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
        mappingErrors[key] = t('modelDirectives.mappingObjectRequired')
        return
      }
      effectiveMapping = Object.keys(parsed).length > 0
        ? parsed as Record<string, unknown>
        : undefined
    } catch {
      mappingErrors[key] = t('modelDirectives.invalidMappingJson')
      return
    }
  }

  if (isPendingCustomSuffix(apiFormat) && !effectiveMapping) {
    mappingErrors[key] = t('modelDirectives.nonemptyMappingRequired')
    return
  }

  delete mappingErrors[key]
  const mapping = effectiveMapping
    ? modelDirectiveMappingOverrideFromEffective(apiFormat, suffix, effectiveMapping)
    : undefined
  localMappingParams[key] = effectiveMappingText(apiFormat, suffix, mapping)
  const mappings = updateModelDirectiveMappingOverride(current.mappings, suffix, mapping)
  const hasBuiltIn = hasBuiltInMapping(apiFormat)
  const suffixes = isPendingCustomSuffix(apiFormat) && mapping
    ? updateModelDirectiveSuffixEnabled(current.suffixes, suffix, true)
    : !mapping && !hasBuiltIn
      ? updateModelDirectiveSuffixEnabled(current.suffixes, suffix, false)
      : current.suffixes
  emit('save', {
    ...props.config,
    reasoning_effort: {
      ...props.config.reasoning_effort,
      api_formats: {
        ...props.config.reasoning_effort.api_formats,
        [apiFormat]: {
          ...current,
          suffixes,
          mappings,
        },
      },
    },
  })
}

function resetMappingOverride(apiFormat: string) {
  const current = formatConfig(apiFormat)
  const suffix = selectedSuffixes[apiFormat] ?? PREFERRED_MODEL_DIRECTIVE_SUFFIX
  if (isPendingCustomSuffix(apiFormat)) {
    removeLocalCustomSuffix(apiFormat, suffix)
    const key = mappingKey(apiFormat, suffix)
    dirtyMappingKeys.delete(key)
    delete mappingErrors[key]
    delete localMappingParams[key]
    selectedSuffixTouched.delete(apiFormat)
    selectedSuffixes[apiFormat] = preferredSuffix(apiFormat, current)
    return
  }
  const mappings = updateModelDirectiveMappingOverride(current.mappings, suffix, undefined)
  const suffixes = hasBuiltInMapping(apiFormat)
    ? current.suffixes
    : updateModelDirectiveSuffixEnabled(current.suffixes, suffix, false)
  const key = mappingKey(apiFormat, suffix)
  dirtyMappingKeys.delete(key)
  delete mappingErrors[key]
  localMappingParams[key] = effectiveMappingText(apiFormat, suffix, undefined)
  removeLocalCustomSuffix(apiFormat, suffix)
  emit('save', {
    ...props.config,
    reasoning_effort: {
      ...props.config.reasoning_effort,
      api_formats: {
        ...props.config.reasoning_effort.api_formats,
        [apiFormat]: { ...current, suffixes, mappings },
      },
    },
  })
}

</script>
