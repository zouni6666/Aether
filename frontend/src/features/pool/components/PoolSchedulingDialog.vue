<template>
  <Dialog
    :model-value="modelValue"
    :title="t('ui.poolScheduling')"
    :description="t('ui.manageKeyDistributionModesAndOrderingPreferencesInThePool')"
    size="lg"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="max-h-[calc(100dvh-13rem)] space-y-5 overflow-y-auto overscroll-contain pr-1 sm:max-h-[min(72vh,42rem)] sm:space-y-6 sm:pr-2">
      <!-- Section 1: 分配模式 (distribution_mode 互斥组, 四选一) -->
      <div class="space-y-4 rounded-2xl border border-border/60 bg-card/70 p-4">
        <div class="space-y-1">
          <h3 class="text-sm font-medium">
            {{ t('ui.distributionMode') }}
          </h3>
          <p class="text-xs text-muted-foreground">
            {{ t('ui.chooseOneBaseDistributionModeForKeys') }}
          </p>
        </div>

        <div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
          <button
            v-for="{ index, item } in distributionItems"
            :key="item.preset"
            type="button"
            class="min-h-11 rounded-xl border px-3 py-2.5 text-sm font-medium leading-tight transition-all duration-200"
            :disabled="!item.applicable"
            :class="[
              activeDistributionPreset === item.preset
                ? 'border-primary bg-primary text-primary-foreground shadow-sm shadow-primary/20'
                : item.applicable
                  ? 'border-border/60 bg-background text-foreground hover:border-border hover:bg-muted/40'
                  : 'border-border/30 bg-muted/20 text-muted-foreground/50 cursor-not-allowed'
            ]"
            @click="item.applicable && selectDistribution(index, item.preset)"
          >
            {{ item.label }}
          </button>
        </div>

        <div
          v-if="activeDistributionLabel || activeDistributionDesc"
          class="rounded-xl border border-primary/15 bg-primary/5 px-3 py-2.5"
        >
          <p
            v-if="activeDistributionDesc"
            class="mt-1 text-xs leading-5 text-muted-foreground"
          >
            {{ activeDistributionDesc }}
          </p>
          <div
            v-if="activeDistributionItem?.modeOptions.length"
            data-testid="pool-cache-affinity-secondary-mode"
            class="mt-2 flex w-fit flex-wrap gap-1 rounded-lg bg-muted/50 p-1"
          >
            <button
              v-for="modeOpt in activeDistributionItem.modeOptions"
              :key="modeOpt.value"
              type="button"
              :data-mode="modeOpt.value"
              class="rounded-md px-2.5 py-1 text-xs font-medium transition-all"
              :class="[
                activeDistributionItem.mode === modeOpt.value
                  ? 'bg-primary text-primary-foreground shadow-sm'
                  : 'text-muted-foreground hover:bg-background/70 hover:text-foreground'
              ]"
              @click="setPresetModeByPreset(activeDistributionItem.preset, modeOpt.value)"
            >
              {{ modeOpt.label }}
            </button>
          </div>
        </div>
      </div>

      <div
        v-if="isCodex"
        class="flex items-start justify-between gap-4 rounded-2xl border border-border/60 bg-card/70 p-4"
      >
        <div class="space-y-1">
          <label
            for="pool-reserve-minimum-quota"
            class="text-sm font-medium"
          > {{ t('ui.reserveMinimumQuota') }} </label>
          <p
            id="pool-reserve-minimum-quota-description"
            class="text-xs leading-5 text-muted-foreground"
          >
            {{ t('ui.markAccountsAsExhaustedAndStopSchedulingWhenRemainingQuota') }}
          </p>
        </div>
        <Switch
          id="pool-reserve-minimum-quota"
          v-model="reserveMinimumQuota"
          aria-describedby="pool-reserve-minimum-quota-description"
          :disabled="loading"
          class="mt-0.5 shrink-0"
        />
      </div>

      <!-- Section 2: 策略调度 (非互斥, 可叠加组合 + 拖拽排序) -->
      <div class="space-y-4 rounded-2xl border border-border/60 bg-card/70 p-4">
        <div class="space-y-1">
          <div class="flex flex-wrap items-center gap-2">
            <h3 class="text-sm font-medium">
              {{ t('ui.strategyScheduling') }}
            </h3>
            <span class="rounded-full bg-muted px-2 py-0.5 text-[11px] text-muted-foreground"> {{ t('ui.enableddfb8022') }} {{ enabledStrategyCount }} {{ t('ui.items') }} </span>
          </div>
          <p class="text-xs text-muted-foreground">
            {{ t('ui.layerOrderingFactorsOnTopOfTheDistributionModeMultiple') }}
          </p>
          <p class="text-xs text-muted-foreground">
            {{ t('ui.desktopSupportsDragSortingMobileCanMovePrioritiesUpAnd') }}
          </p>
        </div>

        <div class="space-y-1.5">
          <div
            v-for="{ index, item } in strategyItems"
            :key="item.preset"
            class="group rounded-xl border px-3 py-2.5 transition-all duration-200"
            :class="[
              !item.applicable
                ? 'border-border/40 bg-muted/20 opacity-80'
                : draggedIndex === index
                  ? 'border-primary/50 bg-primary/5 shadow-md'
                  : dragOverIndex === index
                    ? 'border-primary/30 bg-primary/5'
                    : item.enabled
                      ? 'border-primary/20 bg-primary/5 hover:border-primary/30'
                      : 'border-border/60 bg-background hover:border-border hover:bg-muted/30'
            ]"
            :draggable="item.applicable"
            @dragstart="item.applicable && handleDragStart(index, $event)"
            @dragend="handleDragEnd"
            @dragover.prevent="item.applicable && handleDragOver(index)"
            @dragleave="handleDragLeave"
            @drop="item.applicable && handleDrop(index)"
          >
            <div class="flex items-start gap-2.5">
              <!-- Drag handle -->
              <div
                class="hidden rounded-lg p-1 transition-colors sm:flex sm:shrink-0"
                :class="item.applicable
                  ? 'cursor-grab active:cursor-grabbing text-muted-foreground/40 group-hover:text-muted-foreground'
                  : 'text-muted-foreground/20 cursor-default'"
              >
                <GripVertical class="h-4 w-4" />
              </div>

              <div class="min-w-0 flex-1">
                <div class="flex items-start gap-2.5">
                  <div class="min-w-0 flex-1">
                    <div class="flex flex-wrap items-center gap-2">
                      <span
                        class="text-sm font-medium"
                        :class="!item.applicable ? 'text-muted-foreground' : 'text-foreground'"
                      >{{ item.label }}</span>
                      <span
                        v-if="getStrategyPriority(index)"
                        class="rounded-full bg-primary/10 px-2 py-0.5 text-[11px] font-medium text-primary"
                      >
                        #{{ getStrategyPriority(index) }}
                      </span>
                      <span
                        v-if="!item.applicable"
                        class="rounded-full border border-border/60 bg-muted px-2 py-0.5 text-[11px] text-muted-foreground"
                      > {{ t('ui.currentlyUnavailable') }} </span>
                    </div>
                    <p class="mt-0.5 text-xs leading-5 text-muted-foreground">
                      {{ item.desc }}
                    </p>
                  </div>

                  <div
                    v-if="item.applicable"
                    class="flex shrink-0 items-center gap-1.5 sm:hidden"
                  >
                    <button
                      type="button"
                      class="inline-flex h-7 items-center justify-center rounded-lg border border-border/60 px-2.5 text-[11px] font-medium text-foreground transition-colors hover:bg-muted disabled:cursor-not-allowed disabled:opacity-40"
                      :disabled="!canMoveStrategy(index, -1)"
                      @click="moveStrategy(index, -1)"
                    >
                      {{ t('ui.moveUp') }}
                    </button>
                    <button
                      type="button"
                      class="inline-flex h-7 items-center justify-center rounded-lg border border-border/60 px-2.5 text-[11px] font-medium text-foreground transition-colors hover:bg-muted disabled:cursor-not-allowed disabled:opacity-40"
                      :disabled="!canMoveStrategy(index, 1)"
                      @click="moveStrategy(index, 1)"
                    >
                      {{ t('ui.moveDown') }}
                    </button>
                  </div>

                  <Switch
                    :model-value="item.enabled && item.applicable"
                    :disabled="!item.applicable"
                    class="mt-0.5 shrink-0"
                    @update:model-value="(v: boolean) => togglePreset(index, v)"
                  />
                </div>

                <div
                  v-if="(item.modeOptions.length > 0 && item.enabled && item.applicable) || item.applicable"
                  class="mt-2 flex flex-col gap-1.5 sm:flex-row sm:items-center sm:justify-between"
                >
                  <!-- Mode sub-config -->
                  <div
                    v-if="item.modeOptions.length > 0 && item.enabled && item.applicable"
                    class="flex flex-wrap gap-1 rounded-lg bg-muted/40 p-1"
                  >
                    <button
                      v-for="modeOpt in item.modeOptions"
                      :key="modeOpt.value"
                      type="button"
                      class="rounded-md px-2.5 py-1 text-xs font-medium transition-all"
                      :class="[
                        item.mode === modeOpt.value
                          ? 'bg-primary text-primary-foreground shadow-sm'
                          : 'text-muted-foreground hover:bg-background/70 hover:text-foreground'
                      ]"
                      @click="setPresetModeByPreset(item.preset, modeOpt.value)"
                    >
                      {{ modeOpt.label }}
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <Button
        variant="outline"
        class="min-w-[96px] flex-1 sm:flex-none"
        :disabled="loading"
        @click="emit('update:modelValue', false)"
      >
        {{ t('common.cancel') }}
      </Button>
      <Button
        class="min-w-[96px] flex-1 sm:flex-none"
        :disabled="loading"
        @click="handleSave"
      >
        {{ loading ? t('ui.saving') : t('ui.save') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, ref, watch } from 'vue'
import { GripVertical } from 'lucide-vue-next'
import { Dialog, Button, Switch } from '@/components/ui'
import { useToast } from '@/composables/useToast'
import { parseApiError } from '@/utils/errorParser'
import { getProvider, updateProvider } from '@/api/endpoints'
import { getPoolSchedulingPresets } from '@/api/endpoints/pool'
import {
  mergePoolAdvancedPatch,
  moveStrategyItem,
  normalizeMutexSelection,
} from '@/features/pool/utils/poolSchedulingDialog'
import type { PoolPresetMeta } from '@/api/endpoints/pool'
import type {
  PoolAdvancedConfig,
  SchedulingPresetItem,
  ProviderWithEndpointsSummary,
} from '@/api/endpoints/types/provider'

interface PresetModeOption {
  value: string
  label: string
}

interface PresetListItem {
  preset: string
  label: string
  desc: string
  enabled: boolean
  mode: string | null
  modeOptions: PresetModeOption[]
  applicable: boolean
  mutexGroup: string | null
  evidenceHint: string
}

const props = defineProps<{
  modelValue: boolean
  providerId: string
  providerType?: string
  currentConfig: PoolAdvancedConfig | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  saved: [provider: ProviderWithEndpointsSummary]
}>()

const { t } = useI18n()

const DISTRIBUTION_GROUP = 'distribution_mode'

const FALLBACK_PRESET_DEFS: PoolPresetMeta[] = [
  {
    name: 'cache_affinity',
    get label() { return t('ui.cacheAffinity951ca1e') },
    get description() { return t('ui.reuseAKeyForEachUserInitialAssignmentsCanBe') },
    mutex_group: DISTRIBUTION_GROUP,
    get evidence_hint() { return t('ui.reuseTheStickyKeyFirstThenUseTheSelectedSecondary') },
    providers: [],
    default_enabled: true,
    modes: [
      { value: 'single_account', get label() { return t('ui.singleAccountFirst') } },
      { value: 'lru', get label() { return t('ui.lruRotation') } },
    ],
    default_mode: 'single_account',
  },
  {
    name: 'lru',
    get label() { return t('ui.lruRotation0d37c40') },
    get description() { return t('ui.preferTheLeastRecentlyUsedKeys') },
    mutex_group: DISTRIBUTION_GROUP,
    get evidence_hint() { return t('ui.useLruTimestampsPreferringTheLeastRecentlyUsedAccount') },
    providers: [],
    modes: null,
    default_mode: null,
  },
  {
    name: 'single_account',
    get label() { return t('ui.singleAccountFirst') },
    get description() { return t('ui.concentrateUsageOnTheSameAccountReverseLru') },
    mutex_group: DISTRIBUTION_GROUP,
    get evidence_hint() { return t('ui.useInternalPriorityFirstThenReverseLruToConcentrateRequests') },
    providers: [],
    modes: null,
    default_mode: null,
  },
  {
    name: 'load_balance',
    get label() { return t('ui.loadBalancing9550295') },
    get description() { return t('ui.randomlyDistributeKeyUsageToBalanceLoad') },
    mutex_group: DISTRIBUTION_GROUP,
    get evidence_hint() { return t('ui.assignRandomScoresForEvenDistribution') },
    providers: [],
    modes: null,
    default_mode: null,
  },
  {
    name: 'free_team_first',
    get label() { return t('ui.preferFreeTeam') },
    get description() { return t('ui.legacySettingPreferFreeTeamOrBoth') },
    get evidence_hint() { return t('ui.usePlanTypeWithLegacyFreeOnlyTeamOnlyOr') },
    providers: ['codex', 'grok', 'kiro', 'windsurf', 'xai'],
    modes: [
      { value: 'free_only', label: 'Free' },
      { value: 'team_only', label: 'Team' },
      { value: 'both', label: 'Free + Team' },
    ],
    default_mode: 'both',
  },
  {
    name: 'free_first',
    get label() { return t('ui.preferFree') },
    get description() { return t('ui.preferFreeAccountsBasedOnPlanType') },
    get evidence_hint() { return t('ui.usePlanTypePrioritizingFreeAccounts') },
    providers: ['codex', 'grok', 'kiro', 'windsurf', 'xai'],
    modes: null,
    default_mode: null,
  },
  {
    name: 'team_first',
    get label() { return t('ui.preferTeam') },
    get description() { return t('ui.preferTeamAccountsBasedOnPlanType') },
    get evidence_hint() { return t('ui.usePlanTypePrioritizingTeamAccounts') },
    providers: ['codex', 'grok', 'kiro', 'windsurf', 'xai'],
    modes: null,
    default_mode: null,
  },
  {
    name: 'plus_first',
    get label() { return t('ui.preferPlus') },
    get description() { return t('ui.preferPlusAccountsBasedOnPlanType') },
    get evidence_hint() { return t('ui.usePlanTypePrioritizingPlusAccounts') },
    providers: ['codex', 'grok', 'kiro', 'windsurf', 'xai'],
    modes: null,
    default_mode: null,
  },
  {
    name: 'pro_first',
    get label() { return t('ui.preferPro') },
    get description() { return t('ui.preferProAccountsBasedOnPlanType') },
    get evidence_hint() { return t('ui.usePlanTypePrioritizingProAccounts') },
    providers: ['codex', 'grok', 'kiro', 'windsurf', 'xai'],
    modes: null,
    default_mode: null,
  },
  {
    name: 'quota_balanced',
    get label() { return t('ui.quotaBalance') },
    get description() { return t('ui.preferAccountsWithTheLowestQuotaUsage') },
    get evidence_hint() { return t('ui.useAccountQuotaUtilizationFallingBackToWindowCostUsage') },
    providers: [],
    modes: null,
    default_mode: null,
  },
  {
    name: 'recent_refresh',
    get label() { return t('ui.quotaRefreshFirst') },
    get description() { return t('ui.preferAccountsWhoseQuotaWillRefreshSoon') },
    get evidence_hint() { return t('ui.useTheAccountQuotaResetCountdownNextResetResetSeconds') },
    providers: ['codex', 'grok', 'kiro', 'windsurf', 'xai'],
    default_enabled_providers: ['codex', 'windsurf'],
    modes: null,
    default_mode: null,
  },
  {
    name: 'priority_first',
    get label() { return t('ui.priorityFirst') },
    get description() { return t('ui.scheduleByAccountPriorityLowerNumbersAreHigherPriority') },
    get evidence_hint() { return t('ui.useInternalPriorityWithDragAndDropOrManualEditing') },
    providers: [],
    modes: null,
    default_mode: null,
  },
  {
    name: 'health_first',
    get label() { return t('ui.healthFirst') },
    get description() { return t('ui.preferAccountsWithHigherHealthScoresAndFewerFailures') },
    get evidence_hint() { return t('ui.useAggregateHealthByFormatScoresIncludingCircuitBreakerAnd') },
    providers: [],
    modes: null,
    default_mode: null,
  },
  {
    name: 'latency_first',
    get label() { return t('ui.latencyFirst') },
    get description() { return t('ui.preferAccountsWithLowerRecentLatency') },
    get evidence_hint() { return t('ui.useAveragePoolLatencyOverLatencyWindowSeconds') },
    providers: [],
    modes: null,
    default_mode: null,
  },
  {
    name: 'cost_first',
    get label() { return t('ui.costFirst') },
    get description() { return t('ui.preferAccountsWithLowerWindowCost') },
    get evidence_hint() { return t('ui.useWindowCostOrTokenUsageFallingBackToQuota') },
    providers: [],
    modes: null,
    default_mode: null,
  },
]

const { success, error: showError } = useToast()
const loading = ref(false)
const reserveMinimumQuota = ref(false)
const isCodex = computed(() => normalizeProviderType(props.providerType) === 'codex')
let dialogRevision = 0
const presetDefs = ref<PoolPresetMeta[]>([])
const presetDefsLoaded = ref(false)
const loadingPresetDefs = ref(false)

const draggedIndex = ref<number | null>(null)
const dragOverIndex = ref<number | null>(null)
const presetList = ref<PresetListItem[]>([])

function normalizeProviderType(value: string | undefined): string {
  return (value || '').trim().toLowerCase()
}

function normalizePresetName(value: unknown): string {
  return String(value ?? '').trim().toLowerCase()
}

function normalizeMode(value: unknown): string | null {
  const normalized = String(value ?? '').trim().toLowerCase()
  return normalized || null
}

function normalizeMutexGroup(value: unknown): string | null {
  const normalized = String(value ?? '').trim().toLowerCase()
  return normalized || null
}

const FALLBACK_ORDER = FALLBACK_PRESET_DEFS.map(d => d.name)

function normalizePresetDefs(defs: PoolPresetMeta[]): PoolPresetMeta[] {
  const ordered: PoolPresetMeta[] = []
  const seen = new Set<string>()
  for (const raw of defs) {
    const name = normalizePresetName(raw.name)
    if (!name || seen.has(name)) continue
    seen.add(name)
    const providers = Array.isArray(raw.providers)
      ? raw.providers.map(p => normalizeProviderType(p)).filter(Boolean)
      : []
    const modes = Array.isArray(raw.modes)
      ? raw.modes
        .map(mode => ({
          value: normalizePresetName(mode.value),
          label: String(mode.label ?? '').trim() || String(mode.value ?? '').trim(),
        }))
        .filter(mode => Boolean(mode.value))
      : null
    const defaultMode = normalizeMode(raw.default_mode)
    const defaultEnabledProviders = Array.isArray(raw.default_enabled_providers)
      ? raw.default_enabled_providers.map(p => normalizeProviderType(p)).filter(Boolean)
      : []
    ordered.push({
      name,
      label: String(raw.label ?? '').trim() || name,
      description: String(raw.description ?? '').trim(),
      providers,
      default_enabled: raw.default_enabled === true,
      default_enabled_providers: defaultEnabledProviders,
      modes: modes && modes.length > 0 ? modes : null,
      default_mode: defaultMode,
      mutex_group: normalizeMutexGroup(raw.mutex_group),
      evidence_hint: String(raw.evidence_hint ?? '').trim() || null,
    })
  }
  // Re-order by FALLBACK_PRESET_DEFS order; unknown presets go to the end.
  ordered.sort((a, b) => {
    const ia = FALLBACK_ORDER.indexOf(a.name)
    const ib = FALLBACK_ORDER.indexOf(b.name)
    return (ia === -1 ? 9999 : ia) - (ib === -1 ? 9999 : ib)
  })
  return ordered
}

function getPresetDefs(): PoolPresetMeta[] {
  if (presetDefs.value.length > 0) {
    return presetDefs.value
  }
  return FALLBACK_PRESET_DEFS
}

async function ensurePresetDefsLoaded(): Promise<void> {
  if (presetDefsLoaded.value || loadingPresetDefs.value) return
  loadingPresetDefs.value = true
  try {
    const remoteDefs = await getPoolSchedulingPresets()
    const normalized = normalizePresetDefs(Array.isArray(remoteDefs) ? remoteDefs : [])
    if (normalized.length > 0) {
      presetDefs.value = normalized
      presetDefsLoaded.value = true
    }
  } catch (err) {
    showError(parseApiError(err))
  } finally {
    loadingPresetDefs.value = false
  }
}

function isApplicablePreset(def: PoolPresetMeta): boolean {
  const providerType = normalizeProviderType(props.providerType)
  const providers = Array.isArray(def.providers) ? def.providers : []
  if (providers.length === 0) return true
  if (!providerType) return true
  return providers.includes(providerType)
}

function getModeOptions(def: PoolPresetMeta): PresetModeOption[] {
  const modes = Array.isArray(def.modes) ? def.modes : []
  return modes
    .map(mode => ({
      value: normalizePresetName(mode.value),
      label: String(mode.label ?? '').trim() || String(mode.value ?? '').trim(),
    }))
    .filter(mode => Boolean(mode.value))
}

function defaultModeForPreset(def: PoolPresetMeta): string | null {
  const options = getModeOptions(def)
  if (options.length === 0) return null
  const normalizedDefault = normalizeMode(def.default_mode)
  if (normalizedDefault && options.some(option => option.value === normalizedDefault)) {
    return normalizedDefault
  }
  return options[0].value
}

function buildPresetListItem(def: PoolPresetMeta, enabled: boolean, mode?: unknown): PresetListItem {
  return {
    preset: def.name,
    label: def.label,
    desc: def.description,
    enabled,
    mode: mode !== undefined ? resolveMode(def, mode) : defaultModeForPreset(def),
    modeOptions: getModeOptions(def),
    applicable: isApplicablePreset(def),
    mutexGroup: normalizeMutexGroup(def.mutex_group),
    evidenceHint: String(def.evidence_hint ?? '').trim(),
  }
}

function buildDefaultPresetList(): PresetListItem[] {
  const providerType = normalizeProviderType(props.providerType)
  return getPresetDefs().map((def) => {
    const providerDefaults = Array.isArray(def.default_enabled_providers)
      ? def.default_enabled_providers.map(normalizeProviderType)
      : []
    const enabled = def.default_enabled === true
      || (Boolean(providerType) && providerDefaults.includes(providerType))
    return buildPresetListItem(def, enabled)
  })
}

function isNewFormatPresetItem(item: unknown): item is SchedulingPresetItem {
  return typeof item === 'object' && item !== null && 'preset' in item
}

function resolveMode(def: PoolPresetMeta, mode: unknown): string | null {
  const options = getModeOptions(def)
  if (options.length === 0) return null
  const normalized = normalizeMode(mode)
  if (normalized && options.some(option => option.value === normalized)) {
    return normalized
  }
  return defaultModeForPreset(def)
}

function insertMissingByPreferredOrder(
  ordered: PresetListItem[],
  seen: Set<string>,
  defs: PoolPresetMeta[],
  defsByName: Map<string, PoolPresetMeta>,
) {
  // Insert missing presets at their preferred position from FALLBACK_ORDER
  // rather than appending to the end.
  for (const name of FALLBACK_ORDER) {
    if (seen.has(name)) continue
    const def = defsByName.get(name)
    if (!def) continue
    seen.add(name)
    const item = buildPresetListItem(def, false)
    // Find the best insertion point: right after the last item whose
    // FALLBACK_ORDER index is smaller than ours.
    const myIdx = FALLBACK_ORDER.indexOf(name)
    let insertAt = ordered.length
    for (let i = ordered.length - 1; i >= 0; i--) {
      const peerIdx = FALLBACK_ORDER.indexOf(ordered[i].preset)
      if (peerIdx !== -1 && peerIdx < myIdx) {
        insertAt = i + 1
        break
      }
      if (i === 0) insertAt = 0
    }
    ordered.splice(insertAt, 0, item)
  }
  // Any remaining defs not in FALLBACK_ORDER go to the end.
  for (const def of defs) {
    if (seen.has(def.name)) continue
    seen.add(def.name)
    ordered.push(buildPresetListItem(def, false))
  }
}

function reorderDistributionGroup(items: PresetListItem[]): PresetListItem[] {
  // Distribution mode items are rendered as a fixed button group,
  // so their order should always match FALLBACK_ORDER regardless of saved config.
  const distIndexes: number[] = []
  const distItems: PresetListItem[] = []
  items.forEach((item, i) => {
    if (item.mutexGroup === DISTRIBUTION_GROUP) {
      distIndexes.push(i)
      distItems.push(item)
    }
  })
  if (distItems.length <= 1) return items

  distItems.sort((a, b) => {
    const ia = FALLBACK_ORDER.indexOf(a.preset)
    const ib = FALLBACK_ORDER.indexOf(b.preset)
    return (ia === -1 ? 9999 : ia) - (ib === -1 ? 9999 : ib)
  })

  const result = [...items]
  distIndexes.forEach((origIdx, i) => {
    result[origIdx] = distItems[i]
  })
  return result
}

function loadFromConfig(cfg: PoolAdvancedConfig | null): PresetListItem[] {
  const defs = getPresetDefs()
  const defsByName = new Map(defs.map(def => [def.name, def]))
  const defaults = buildDefaultPresetList()
  if (!cfg) return defaults

  const rawPresets = cfg.scheduling_presets
  if (!Array.isArray(rawPresets) || rawPresets.length === 0) {
    if (cfg.scheduling_mode === 'lru' || cfg.lru_enabled === true) {
      return defaults.map(item => ({
        ...item,
        enabled: item.preset === 'lru',
      }))
    }
    return defaults
  }

  const first = rawPresets[0]
  if (isNewFormatPresetItem(first)) {
    const configItems = rawPresets as SchedulingPresetItem[]
    const ordered: PresetListItem[] = []
    const seen = new Set<string>()

    for (const ci of configItems) {
      const presetName = normalizePresetName(ci.preset)
      const def = defsByName.get(presetName)
      if (!def || seen.has(presetName)) continue
      seen.add(presetName)
      ordered.push(buildPresetListItem(def, ci.enabled !== false, ci.mode))
    }

    insertMissingByPreferredOrder(ordered, seen, defs, defsByName)
    return reorderDistributionGroup(normalizeMutexSelection(ordered))
  }

  const legacyPresets = rawPresets as string[]
  const lruEnabled = cfg.lru_enabled !== false
  const ordered: PresetListItem[] = []
  const seen = new Set<string>()

  const lruDef = defsByName.get('lru')
  if (lruDef) {
    ordered.push(buildPresetListItem(lruDef, lruEnabled))
    seen.add('lru')
  }

  for (const name of legacyPresets) {
    const presetName = normalizePresetName(name)
    const def = defsByName.get(presetName)
    if (!def || seen.has(presetName)) continue
    seen.add(presetName)
    ordered.push(buildPresetListItem(def, true, undefined))
  }

  insertMissingByPreferredOrder(ordered, seen, defs, defsByName)
  return reorderDistributionGroup(normalizeMutexSelection(ordered))
}

function togglePreset(index: number, enabled: boolean) {
  const item = presetList.value[index]
  if (!item) return
  item.enabled = enabled
}

function selectDistribution(_anchorIndex: number, presetName: string) {
  presetList.value.forEach(item => {
    if (item.mutexGroup === DISTRIBUTION_GROUP) {
      item.enabled = item.preset === presetName && item.applicable
    }
  })
}

function setPresetModeByPreset(preset: string, mode: string) {
  const targetIndex = presetList.value.findIndex(item => item.preset === preset)
  if (targetIndex < 0) return
  presetList.value[targetIndex].mode = mode
}

const distributionItems = computed(() => {
  const items: { index: number; item: PresetListItem }[] = []
  presetList.value.forEach((item, index) => {
    if (item.mutexGroup === DISTRIBUTION_GROUP) {
      items.push({ index, item })
    }
  })
  return items
})

const activeDistributionPreset = computed(() => {
  const found = distributionItems.value.find(({ item }) => item.enabled && item.applicable)
  return found?.item.preset ?? null
})

const activeDistributionItem = computed(() => {
  const found = distributionItems.value.find(({ item }) => item.enabled && item.applicable)
  return found?.item ?? null
})

const activeDistributionDesc = computed(() => {
  return activeDistributionItem.value?.desc ?? null
})

const activeDistributionLabel = computed(() => {
  return activeDistributionItem.value?.label ?? null
})

const strategyItems = computed(() => {
  const items: { index: number; item: PresetListItem }[] = []
  presetList.value.forEach((item, index) => {
    if (!item.mutexGroup) {
      items.push({ index, item })
    }
  })
  return items
})

const enabledStrategyPriorityMap = computed(() => {
  const priorities = new Map<number, number>()
  let priority = 0

  strategyItems.value.forEach(({ index, item }) => {
    if (!item.enabled || !item.applicable) return
    priority += 1
    priorities.set(index, priority)
  })

  return priorities
})

const enabledStrategyCount = computed(() => enabledStrategyPriorityMap.value.size)

function getStrategyPriority(index: number): number | null {
  return enabledStrategyPriorityMap.value.get(index) ?? null
}

function canMoveStrategy(index: number, direction: -1 | 1): boolean {
  const strategyIndexes = strategyItems.value.map(({ index: currentIndex }) => currentIndex)
  const currentPosition = strategyIndexes.indexOf(index)
  if (currentPosition === -1) return false
  const targetPosition = currentPosition + direction
  return targetPosition >= 0 && targetPosition < strategyIndexes.length
}

function moveStrategy(index: number, direction: -1 | 1) {
  presetList.value = moveStrategyItem(presetList.value, index, direction)
}

function handleDragStart(index: number, event: DragEvent) {
  draggedIndex.value = index
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/html', '')
  }
}

function handleDragEnd() {
  draggedIndex.value = null
  dragOverIndex.value = null
}

function handleDragOver(index: number) {
  dragOverIndex.value = index
}

function handleDragLeave() {
  dragOverIndex.value = null
}

function handleDrop(dropIndex: number) {
  if (draggedIndex.value === null || draggedIndex.value === dropIndex) {
    draggedIndex.value = null
    dragOverIndex.value = null
    return
  }
  const items = [...presetList.value]
  const [draggedItem] = items.splice(draggedIndex.value, 1)
  items.splice(dropIndex, 0, draggedItem)
  presetList.value = items
  draggedIndex.value = null
  dragOverIndex.value = null
}

watch([() => props.modelValue, () => props.providerId], async ([open]) => {
  const revision = ++dialogRevision
  loading.value = false
  if (!open) return
  reserveMinimumQuota.value = props.currentConfig?.reserve_minimum_quota === true
  await ensurePresetDefsLoaded()
  if (!props.modelValue || dialogRevision !== revision) return
  presetList.value = normalizeMutexSelection(loadFromConfig(props.currentConfig))
})

async function handleSave() {
  const providerId = props.providerId
  const revision = dialogRevision
  loading.value = true
  try {
    await ensurePresetDefsLoaded()
    if (!props.modelValue || props.providerId !== providerId || dialogRevision !== revision) return
    if (!presetDefsLoaded.value) {
      showError(t('ui.failedToLoadSchedulingPolicyMetadataTryAgain'))
      return
    }
    presetList.value = normalizeMutexSelection(presetList.value)
    const schedulingPresets: SchedulingPresetItem[] = presetList.value.map(item => {
      const result: SchedulingPresetItem = {
        preset: item.preset,
        enabled: item.enabled && item.applicable,
      }
      if (item.modeOptions.length > 0 && item.mode) {
        result.mode = item.mode
      }
      return result
    })

    const latestProvider = await getProvider(providerId)
    if (!props.modelValue || props.providerId !== providerId || dialogRevision !== revision) return
    const latestAdvanced = latestProvider.pool_advanced
    const mergedAdvanced = mergePoolAdvancedPatch(latestAdvanced, {
      scheduling_presets: schedulingPresets,
      ...(isCodex.value ? { reserve_minimum_quota: reserveMinimumQuota.value } : {}),
    })
    const payload: Parameters<typeof updateProvider>[1] = {
      pool_advanced: mergedAdvanced as PoolAdvancedConfig,
    }
    const updatedProvider = await updateProvider(providerId, payload)
    if (!props.modelValue || props.providerId !== providerId || dialogRevision !== revision) return

    success(t('ui.poolSchedulingSaved'))
    emit('saved', updatedProvider)
    emit('update:modelValue', false)
  } catch (err) {
    showError(parseApiError(err))
  } finally {
    if (dialogRevision === revision) {
      loading.value = false
    }
  }
}

</script>
