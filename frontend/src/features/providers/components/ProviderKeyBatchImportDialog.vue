<template>
  <Dialog
    :model-value="open"
    :title="t('ui.importKeysInBulk')"
    :description="providerName ? t('ui.valueNameAndKeyAreBothRequired', { name: providerName }) : t('ui.bothNameAndKeyAreRequired')"
    :icon="ListPlus"
    size="4xl"
    persistent
    @update:model-value="handleDialogUpdate"
  >
    <div class="space-y-3.5">
      <nav
        class="grid grid-cols-3 gap-1.5 rounded-xl bg-muted/40 p-1.5"
        :aria-label="t('ui.bulkImportSteps')"
      >
        <button
          v-for="step in steps"
          :key="step.id"
          type="button"
          class="flex min-h-10 items-center justify-center gap-2 rounded-lg px-2 text-xs font-medium transition-[background-color,box-shadow,color,scale] active:scale-[0.96] disabled:cursor-default disabled:opacity-50 sm:text-sm"
          :class="currentStep === step.id
            ? 'bg-background text-foreground shadow-[0_0_0_1px_rgb(0_0_0/0.06),0_1px_2px_rgb(0_0_0/0.06)] dark:shadow-[0_0_0_1px_rgb(255_255_255/0.08)]'
            : 'text-muted-foreground hover:bg-background/60 hover:text-foreground'"
          :disabled="!canNavigateToStep(step.id)"
          @click="goToStep(step.id)"
        >
          <span
            class="flex h-5 w-5 shrink-0 items-center justify-center rounded-md text-[10px] tabular-nums"
            :class="currentStep === step.id ? 'bg-foreground text-background' : 'bg-muted text-muted-foreground'"
          >{{ step.id }}</span>
          <span class="truncate">{{ step.label }}</span>
        </button>
      </nav>

      <section
        v-if="currentStep === 1"
        class="overflow-hidden rounded-xl bg-background shadow-[0_0_0_1px_rgb(0_0_0/0.07),0_1px_2px_-1px_rgb(0_0_0/0.08),0_3px_8px_-3px_rgb(0_0_0/0.08)] dark:shadow-[0_0_0_1px_rgb(255_255_255/0.09)]"
      >
        <header class="flex min-h-14 items-center justify-between gap-3 border-b border-border/60 bg-muted/15 px-4 py-3">
          <div class="flex min-w-0 items-center gap-3">
            <span class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-foreground text-xs font-semibold text-background">1</span>
            <div class="min-w-0">
              <h3 class="text-balance text-sm font-semibold text-foreground">
                {{ t('ui.pasteNamesAndKeys') }}
              </h3>
              <p class="text-pretty text-[11px] leading-4 text-muted-foreground">
                {{ t('ui.oneEntryPerLineSeparatedByExactlyFourHyphens') }}
              </p>
            </div>
          </div>
          <Badge
            :variant="parsed.errors.length > 0 ? 'destructive' : parsed.items.length > 0 ? 'success' : 'secondary'"
            class="shrink-0 tabular-nums"
          >
            {{ inputStatusText }}
          </Badge>
        </header>

        <div class="min-w-0">
          <Label
            for="provider-key-batch-input"
            class="sr-only"
          >{{ t('ui.keyList') }}</Label>
          <Textarea
            id="provider-key-batch-input"
            v-model="inputText"
            class="h-[280px] min-h-[220px] max-h-[520px] !resize-y !rounded-none !border-0 !bg-transparent !px-4 !py-4 font-mono text-[13px] leading-6 !shadow-none !ring-0 focus-visible:!ring-0"
            spellcheck="false"
            :placeholder="t('providers.migration.2b292cd1')"
          />
          <div
            v-if="parsed.errors.length > 0"
            class="mx-4 mb-3 space-y-1 rounded-lg bg-destructive/5 px-3 py-2 text-[11px] text-destructive ring-1 ring-destructive/20"
          >
            <div
              v-for="(item, index) in parsed.errors.slice(0, 6)"
              :key="`${item.lineNumber}-${index}`"
            >
              {{ item.lineNumber ? t('ui.lineValue', { value: item.lineNumber }) : '' }}{{ item.message }}
            </div>
            <div
              v-if="parsed.errors.length > 6"
              class="font-medium"
            >
              {{ t('ui.another') }} {{ parsed.errors.length - 6 }} {{ t('ui.issues') }}
            </div>
          </div>
          <div class="flex flex-wrap items-center gap-2 border-t border-border/50 bg-muted/10 px-4 py-2.5 text-[11px] text-muted-foreground">
            <span class="rounded-md bg-muted px-2 py-1 font-mono text-foreground/80">{{ t('ui.nameKey') }}</span>
            <span>{{ t('ui.nameAndKeyCannotBeEmpty') }}</span>
            <span class="ml-auto hidden tabular-nums sm:inline">{{ t('ui.recognized') }} {{ parsed.items.length }} {{ t('ui.itemsf004f1d') }}</span>
          </div>
        </div>
      </section>

      <section
        v-else-if="currentStep === 2"
        class="overflow-hidden rounded-xl bg-background shadow-[0_0_0_1px_rgb(0_0_0/0.07),0_1px_2px_-1px_rgb(0_0_0/0.08),0_3px_8px_-3px_rgb(0_0_0/0.08)] dark:shadow-[0_0_0_1px_rgb(255_255_255/0.09)]"
      >
        <header class="flex min-h-[72px] items-center gap-3 px-4 py-3">
          <span class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-foreground text-xs font-semibold text-background">
            2
          </span>
          <span class="min-w-0 flex-1">
            <span class="block text-sm font-semibold">{{ t('ui.sharedConfiguration') }}</span>
            <span class="mt-1 flex flex-wrap gap-1.5">
              <span
                v-for="item in settingsSummaryItems"
                :key="item"
                class="rounded-md bg-muted px-2 py-0.5 text-[10px] leading-4 text-muted-foreground"
              >{{ item }}</span>
            </span>
          </span>
          <Badge
            :variant="selectedApiFormats.length > 0 ? 'success' : 'destructive'"
            class="ml-auto shrink-0 tabular-nums"
          >
            {{ selectedApiFormats.length }} {{ t('ui.formats') }}
          </Badge>
        </header>

        <div class="border-t border-border/60 bg-muted/10 p-3 sm:p-4">
          <ProviderKeyImportSettingsFields
            :auth-type="authType"
            :api-formats="selectedApiFormats"
            :settings="settings"
            :available-api-formats="availableApiFormats"
            @update:auth-type="authType = $event"
            @update:api-formats="selectedApiFormats = $event"
            @update:settings="updateGlobalSettings"
          />
        </div>
      </section>

      <section
        v-else
        class="overflow-hidden rounded-xl bg-background shadow-[0_0_0_1px_rgb(0_0_0/0.07),0_1px_2px_-1px_rgb(0_0_0/0.08),0_3px_8px_-3px_rgb(0_0_0/0.08)] dark:shadow-[0_0_0_1px_rgb(255_255_255/0.09)]"
      >
        <header class="flex min-h-[72px] items-center gap-3 border-b border-border/60 bg-muted/15 px-4 py-3">
          <span class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-foreground text-xs font-semibold text-background">3</span>
          <div class="min-w-0 flex-1">
            <h3 class="text-balance text-sm font-semibold">
              {{ t('ui.reviewEachEntry') }}
            </h3>
            <p class="text-pretty text-[11px] leading-4 text-muted-foreground">
              {{ t('ui.expandAKeyToEditItOrOverrideSharedSettings') }}
            </p>
          </div>
          <div class="shrink-0 text-right text-[11px] text-muted-foreground">
            <div><span class="font-semibold tabular-nums text-foreground">{{ reviewItems.length }}</span> {{ t('ui.keysa80b9b9') }}</div>
            <div v-if="customizedItemCount > 0">
              <span class="tabular-nums">{{ customizedItemCount }}</span> {{ t('ui.customConfigurations') }}
            </div>
          </div>
        </header>

        <div
          v-if="reviewErrorItemCount > 0"
          class="border-b border-destructive/15 bg-destructive/5 px-4 py-2 text-xs text-destructive"
        >
          {{ reviewErrorItemCount }} {{ t('ui.keysMustBeCorrectedBeforeImport') }}
        </div>

        <div class="max-h-[min(52vh,520px)] divide-y divide-border/50 overflow-y-auto overscroll-contain">
          <article
            v-for="entry in pagedReviewItems"
            :key="entry.item.lineNumber"
            class="bg-background"
          >
            <div class="flex min-h-14 items-center gap-3 px-3 py-2 sm:px-4">
              <span class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-muted text-[10px] tabular-nums text-muted-foreground">
                {{ entry.index + 1 }}
              </span>
              <div class="min-w-0 flex-1">
                <div class="flex min-w-0 items-center gap-2">
                  <span class="truncate text-xs font-semibold">{{ entry.item.name || t('ui.nameMissing') }}</span>
                  <Badge
                    v-if="reviewErrorsByIndex.has(entry.index)"
                    variant="destructive"
                    class="shrink-0 text-[10px]"
                  >
                    {{ t('ui.needsCorrection') }}
                  </Badge>
                </div>
                <div class="mt-0.5 truncate font-mono text-[10px] text-muted-foreground">
                  {{ maskSecret(entry.item.apiKey) }}
                </div>
              </div>
              <div class="hidden shrink-0 items-center gap-1.5 sm:flex">
                <span class="rounded-md bg-muted px-2 py-0.5 text-[10px] text-muted-foreground">{{ effectiveAuthLabel(entry.item) }}</span>
                <span class="rounded-md bg-muted px-2 py-0.5 text-[10px] text-muted-foreground">{{ effectiveFormatCount(entry.item) }} {{ t('ui.formats') }}</span>
                <span
                  class="rounded-md px-2 py-0.5 text-[10px]"
                  :class="entry.item.customized ? 'bg-primary/10 text-primary' : 'bg-muted text-muted-foreground'"
                >{{ entry.item.customized ? t('ui.separateConfiguration') : t('ui.sharedConfiguration') }}</span>
              </div>
              <button
                type="button"
                class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg text-muted-foreground transition-[background-color,color,scale] hover:bg-muted hover:text-foreground active:scale-[0.96]"
                :aria-label="t('ui.editValue', { name: entry.item.name || `第 ${entry.index + 1} 项` })"
                :title="editingItemIndex === entry.index ? t('ui.collapseEditor') : t('ui.editThisKey')"
                @click="toggleItemEditor(entry.index)"
              >
                <Pencil class="h-4 w-4" />
              </button>
            </div>

            <div
              v-if="editingItemIndex === entry.index"
              class="space-y-4 border-t border-border/50 bg-muted/10 p-3 sm:p-4"
            >
              <div class="grid gap-3 sm:grid-cols-2">
                <div class="space-y-1.5">
                  <Label class="text-xs">{{ t('stats.column.name') }}</Label>
                  <Input
                    v-model="entry.item.name"
                    class="h-10"
                    :placeholder="t('ui.required')"
                  />
                </div>
                <div class="space-y-1.5">
                  <Label class="text-xs">Key</Label>
                  <Input
                    v-model="entry.item.apiKey"
                    class="h-10 font-mono text-xs"
                    :placeholder="t('ui.required')"
                  />
                </div>
              </div>

              <div class="flex min-h-12 items-center justify-between gap-3 rounded-lg bg-background px-3 shadow-[0_0_0_1px_rgb(0_0_0/0.06)] dark:shadow-[0_0_0_1px_rgb(255_255_255/0.08)]">
                <div>
                  <div class="text-xs font-medium">
                    {{ t('ui.configureThisKeySeparately') }}
                  </div>
                  <div class="text-[11px] text-muted-foreground">
                    {{ t('ui.overrideTheSharedConfigurationFromStep2') }}
                  </div>
                </div>
                <Switch
                  :model-value="entry.item.customized"
                  @update:model-value="setItemCustomized(entry.item, $event)"
                />
              </div>

              <ProviderKeyImportSettingsFields
                v-if="entry.item.customized"
                :auth-type="entry.item.authType"
                :api-formats="entry.item.apiFormats"
                :settings="entry.item.settings"
                :available-api-formats="availableApiFormats"
                @update:auth-type="entry.item.authType = $event"
                @update:api-formats="entry.item.apiFormats = $event"
                @update:settings="entry.item.settings = $event"
              />

              <div
                v-if="reviewErrorsByIndex.has(entry.index)"
                class="space-y-1 rounded-lg bg-destructive/5 px-3 py-2 text-[11px] text-destructive"
              >
                <div
                  v-for="message in reviewErrorsByIndex.get(entry.index)"
                  :key="message"
                >
                  {{ message }}
                </div>
              </div>
            </div>
          </article>
        </div>

        <div
          v-if="reviewPageCount > 1"
          class="flex min-h-12 items-center justify-between gap-3 border-t border-border/60 bg-muted/10 px-3 sm:px-4"
        >
          <Button
            variant="ghost"
            size="sm"
            class="h-9"
            :disabled="reviewPage === 1"
            @click="changeReviewPage(reviewPage - 1)"
          >
            <ChevronLeft class="mr-1 h-4 w-4" /> {{ t('userStats.pagination.previous') }}
          </Button>
          <span class="text-[11px] tabular-nums text-muted-foreground">{{ reviewPage }} / {{ reviewPageCount }}</span>
          <Button
            variant="ghost"
            size="sm"
            class="h-9"
            :disabled="reviewPage === reviewPageCount"
            @click="changeReviewPage(reviewPage + 1)"
          >
            {{ t('userStats.pagination.next') }} <ChevronRight class="ml-1 h-4 w-4" />
          </Button>
        </div>
      </section>
    </div>

    <template #footer>
      <div class="flex w-full flex-col-reverse gap-2 sm:flex-row sm:justify-end">
        <Button
          class="w-full sm:w-auto"
          variant="outline"
          :disabled="importing"
          @click="handleBack"
        >
          <ArrowLeft
            v-if="currentStep > 1"
            class="mr-2 h-4 w-4"
          />
          {{ currentStep === 1 ? t('common.cancel') : t('ui.previousStep') }}
        </Button>
        <Button
          class="w-full sm:w-auto"
          :disabled="primaryActionDisabled"
          @click="handlePrimaryAction"
        >
          <Loader2
            v-if="importing"
            class="mr-2 h-4 w-4 animate-spin"
          />
          <ListPlus
            v-else-if="currentStep === 3"
            class="mr-2 h-4 w-4"
          />
          <ArrowRight
            v-else
            class="mr-2 h-4 w-4"
          />
          {{ primaryActionLabel }}
        </Button>
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, reactive, ref, watch } from 'vue'
import {
  ArrowLeft,
  ArrowRight,
  ChevronLeft,
  ChevronRight,
  ListPlus,
  Loader2,
  Pencil,
} from 'lucide-vue-next'
import {
  Badge,
  Button,
  Dialog,
  Input,
  Label,
  Switch,
  Textarea,
} from '@/components/ui'
import ProviderKeyImportSettingsFields from './ProviderKeyImportSettingsFields.vue'
import { batchImportPoolKeys, type PoolKeySettingsPatch } from '@/api/endpoints/pool'
import { useToast } from '@/composables/useToast'
import { parseApiError } from '@/utils/errorParser'
import { parseProviderKeyBatchImport } from '@/features/providers/utils/providerKeyBatchImport'

type WizardStep = 1 | 2 | 3
type AuthType = 'api_key' | 'bearer'
type ImportSettings = Required<Pick<PoolKeySettingsPatch,
  'internal_priority' | 'rpm_limit' | 'concurrent_limit' | 'cache_ttl_minutes'
  | 'max_probe_interval_minutes' | 'is_active' | 'note' | 'proxy_node_id'
>>

interface ReviewImportItem {
  lineNumber: number
  name: string
  apiKey: string
  customized: boolean
  authType: AuthType
  apiFormats: string[]
  settings: ImportSettings
}

const props = defineProps<{
  open: boolean
  providerId: string
  providerName?: string
  availableApiFormats: string[]
}>()

const emit = defineEmits<{
  close: []
  saved: []
}>()

const { t } = useI18n()

const REVIEW_PAGE_SIZE = 50
const steps: ReadonlyArray<{ id: WizardStep; label: string }> = [
  { id: 1, get label() { return t('ui.importContent') } },
  { id: 2, get label() { return t('ui.sharedConfiguration') } },
  { id: 3, get label() { return t('ui.reviewEachEntry') } },
]

const { success, warning, error: showError } = useToast()
const currentStep = ref<WizardStep>(1)
const inputText = ref('')
const importing = ref(false)
const authType = ref<AuthType>('api_key')
const selectedApiFormats = ref<string[]>([])
const settings = reactive<ImportSettings>(createDefaultSettings())
const reviewItems = ref<ReviewImportItem[]>([])
const reviewPage = ref(1)
const editingItemIndex = ref<number | null>(null)

const parsed = computed(() => parseProviderKeyBatchImport(inputText.value))
const canContinueInput = computed(() => (
  parsed.value.items.length > 0 && parsed.value.errors.length === 0
))
const canContinueSettings = computed(() => selectedApiFormats.value.length > 0)
const inputStatusText = computed(() => {
  if (parsed.value.errors.length > 0) return t('providers.migration.issues463bcd', { value: String(parsed.value.errors.length) })
  if (parsed.value.items.length > 0) return t('providers.migration.validEntriesc3414d', { value: String(parsed.value.items.length) })
  return t('ui.waitingForInput')
})
const reviewErrorsByIndex = computed(() => {
  const errors = new Map<number, string[]>()
  const seenNames = new Set<string>()
  const seenKeys = new Set<string>()

  reviewItems.value.forEach((item, index) => {
    const messages: string[] = []
    const name = item.name.trim()
    const apiKey = item.apiKey.trim()
    if (!name) messages.push(t('ui.nameIsRequired'))
    else if (name.length > 100) messages.push(t('ui.nameMustBe100CharactersOrFewer'))
    else if (seenNames.has(name)) messages.push(t('ui.nameDuplicatesAnEarlierEntry'))
    else seenNames.add(name)

    if (!apiKey) messages.push(t('ui.keyIsRequired'))
    else if (seenKeys.has(apiKey)) messages.push(t('ui.keyDuplicatesAnEarlierEntry'))
    else seenKeys.add(apiKey)

    if (item.customized && item.apiFormats.length === 0) {
      messages.push(t('ui.selectAtLeastOneApiFormatForIndividualConfiguration'))
    }
    if (messages.length > 0) errors.set(index, messages)
  })
  return errors
})
const reviewErrorItemCount = computed(() => reviewErrorsByIndex.value.size)
const customizedItemCount = computed(() => (
  reviewItems.value.filter(item => item.customized).length
))
const reviewPageCount = computed(() => (
  Math.max(1, Math.ceil(reviewItems.value.length / REVIEW_PAGE_SIZE))
))
const pagedReviewItems = computed(() => {
  const start = (reviewPage.value - 1) * REVIEW_PAGE_SIZE
  return reviewItems.value
    .slice(start, start + REVIEW_PAGE_SIZE)
    .map((item, offset) => ({ item, index: start + offset }))
})
const canImport = computed(() => (
  !importing.value
  && reviewItems.value.length > 0
  && reviewErrorsByIndex.value.size === 0
  && canContinueSettings.value
))
const primaryActionDisabled = computed(() => {
  if (importing.value) return true
  if (currentStep.value === 1) return !canContinueInput.value
  if (currentStep.value === 2) return !canContinueSettings.value
  return !canImport.value
})
const primaryActionLabel = computed(() => {
  if (importing.value) return t('ui.importingd0e2dee')
  if (currentStep.value === 1) return t('ui.nextSharedConfiguration')
  if (currentStep.value === 2) return t('ui.nextReviewEntries')
  return t('providers.migration.importKeysb1464e', { value: String(reviewItems.value.length) })
})
const settingsSummaryItems = computed(() => {
  const rpm = settings.rpm_limit == null ? t('ui.adaptiveRpm') : `RPM ${settings.rpm_limit}`
  const concurrent = settings.concurrent_limit == null || settings.concurrent_limit === 0
    ? t('ui.unlimitedConcurrency')
    : t('providers.migration.concurrencya525e6', { value: String(settings.concurrent_limit) })
  const proxy = settings.proxy_node_id ? t('ui.dedicatedProxy') : t('ui.useProviderProxy')
  return [authType.value === 'bearer' ? 'Bearer' : 'API Key', rpm, concurrent, proxy]
})

watch(
  () => props.open,
  (open) => {
    if (!open) return
    currentStep.value = 1
    inputText.value = ''
    importing.value = false
    authType.value = 'api_key'
    selectedApiFormats.value = [...props.availableApiFormats]
    Object.assign(settings, createDefaultSettings())
    reviewItems.value = []
    reviewPage.value = 1
    editingItemIndex.value = null
  },
  { immediate: true },
)

watch(inputText, () => {
  reviewItems.value = []
  reviewPage.value = 1
  editingItemIndex.value = null
})

function createDefaultSettings(): ImportSettings {
  return {
    internal_priority: 50,
    rpm_limit: null,
    concurrent_limit: null,
    cache_ttl_minutes: 5,
    max_probe_interval_minutes: 32,
    is_active: true,
    note: '',
    proxy_node_id: '',
  }
}

function copySettings(source: ImportSettings): ImportSettings {
  return { ...source }
}

function buildSettingsPayload(
  source: ImportSettings,
  includeEmptyProxy = false,
): PoolKeySettingsPatch {
  return {
    internal_priority: source.internal_priority,
    rpm_limit: source.rpm_limit,
    concurrent_limit: source.concurrent_limit,
    cache_ttl_minutes: source.cache_ttl_minutes,
    max_probe_interval_minutes: source.max_probe_interval_minutes,
    is_active: source.is_active,
    note: source.note?.trim() || null,
    ...((source.proxy_node_id || includeEmptyProxy)
      ? { proxy_node_id: source.proxy_node_id || null }
      : {}),
  }
}

function handleDialogUpdate(value: boolean): void {
  if (!value) emit('close')
}

function canNavigateToStep(step: WizardStep): boolean {
  return step <= currentStep.value
}

function goToStep(step: WizardStep): void {
  if (canNavigateToStep(step)) currentStep.value = step
}

function handleBack(): void {
  if (currentStep.value === 1) {
    emit('close')
    return
  }
  currentStep.value = (currentStep.value - 1) as WizardStep
}

function handlePrimaryAction(): void {
  if (primaryActionDisabled.value) return
  if (currentStep.value === 1) {
    currentStep.value = 2
    return
  }
  if (currentStep.value === 2) {
    prepareReviewItems()
    currentStep.value = 3
    return
  }
  void submitImport()
}

function prepareReviewItems(): void {
  if (reviewItems.value.length > 0) return
  reviewItems.value = parsed.value.items.map(item => ({
    lineNumber: item.lineNumber,
    name: item.name,
    apiKey: item.apiKey,
    customized: false,
    authType: authType.value,
    apiFormats: [...selectedApiFormats.value],
    settings: copySettings(settings),
  }))
  reviewPage.value = 1
  editingItemIndex.value = null
}

function toggleItemEditor(index: number): void {
  editingItemIndex.value = editingItemIndex.value === index ? null : index
}

function setItemCustomized(item: ReviewImportItem, customized: boolean): void {
  item.customized = customized
  if (!customized) return
  item.authType = authType.value
  item.apiFormats = [...selectedApiFormats.value]
  item.settings = copySettings(settings)
}

function changeReviewPage(page: number): void {
  reviewPage.value = Math.min(Math.max(page, 1), reviewPageCount.value)
  editingItemIndex.value = null
}

function effectiveAuthLabel(item: ReviewImportItem): string {
  const resolved = item.customized ? item.authType : authType.value
  return resolved === 'bearer' ? 'Bearer' : 'API Key'
}

function effectiveFormatCount(item: ReviewImportItem): number {
  return item.customized ? item.apiFormats.length : selectedApiFormats.value.length
}

function updateGlobalSettings(nextSettings: ImportSettings): void {
  Object.assign(settings, nextSettings)
}

function maskSecret(secret: string): string {
  if (secret.length <= 10) return `${secret.slice(0, 3)}•••`
  return `${secret.slice(0, 6)}••••${secret.slice(-4)}`
}

async function submitImport(): Promise<void> {
  if (!canImport.value) return
  importing.value = true
  try {
    const result = await batchImportPoolKeys(props.providerId, {
      keys: reviewItems.value.map(item => ({
        name: item.name.trim(),
        api_key: item.apiKey.trim(),
        auth_type: item.customized ? item.authType : authType.value,
        ...(item.customized
          ? {
              api_formats: item.apiFormats,
              settings: buildSettingsPayload(item.settings, true),
            }
          : {}),
      })),
      api_formats: selectedApiFormats.value,
      settings: buildSettingsPayload(settings),
    })
    if (result.imported > 0) emit('saved')
    if (result.errors.length > 0) {
      warning(t('providers.migration.importedFailedbe0f08', { value: String(result.imported), value_2: String(result.errors.length) }))
      return
    }
    success(t('providers.migration.importedKeysecd789', { value: String(result.imported) }))
    emit('close')
  } catch (error) {
    showError(parseApiError(error, t('ui.failedToImportKeysInBulk')))
  } finally {
    importing.value = false
  }
}

</script>
