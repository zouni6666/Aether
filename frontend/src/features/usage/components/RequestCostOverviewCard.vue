<template>
  <Card>
    <div class="p-3 sm:p-4">
      <div class="mb-4 text-sm">
        <div class="sm:hidden">
          <span class="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground/70">{{ detailPricingLabel }}</span>
        </div>
        <div class="mt-2 grid grid-cols-[max-content_max-content_max-content_max-content_max-content] items-center gap-x-2 overflow-x-auto whitespace-nowrap text-xs sm:hidden">
          <span>
            <span class="text-muted-foreground">{{ t('ui.totalCost') }}</span>
            <span
              class="ml-1 font-bold"
              :class="detailPricingAvailable ? 'text-green-600 dark:text-green-400' : 'text-muted-foreground'"
              data-request-detail-total-cost
            >
              {{ detailPricingAvailable ? `$${detailTotalCostForDisplay.toFixed(6)}` : (detailUsageAvailable ? t('ui.unpriced') : t('ui.unavailable460b357')) }}
            </span>
          </span>
          <span class="text-muted-foreground">|</span>
          <span>
            <span class="text-muted-foreground">{{ t('ui.duration') }}</span>
            <span class="ml-1 font-bold">
              {{ formatDurationMs(detail.end_to_end_first_byte_time_ms ?? detail.first_byte_time_ms) }} / {{ formatDurationMs(detail.end_to_end_time_ms ?? detail.response_time_ms) }}
            </span>
          </span>
          <span class="text-muted-foreground">|</span>
          <span>
            <span class="text-muted-foreground">{{ t('ui.outputSpeed') }}</span>
            <span class="ml-1 font-bold text-primary">{{ formatOutputRateValue(detailOutputRate) }}tps</span>
          </span>
        </div>
        <div class="hidden flex-wrap items-center gap-x-2 gap-y-1 sm:flex">
          <span class="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground/70">{{ detailPricingLabel }}</span>
          <span class="text-muted-foreground">|</span>
          <span class="whitespace-nowrap">
            <span class="text-muted-foreground">{{ t('ui.totalCost') }}</span>
            <span
              class="ml-1 font-bold"
              :class="detailPricingAvailable ? 'text-green-600 dark:text-green-400' : 'text-muted-foreground'"
              data-request-detail-total-cost
            >
              {{ detailPricingAvailable ? `$${detailTotalCostForDisplay.toFixed(6)}` : (detailUsageAvailable ? t('ui.unpriced') : t('ui.unavailable460b357')) }}
            </span>
          </span>
          <span class="text-muted-foreground">|</span>
          <span class="whitespace-nowrap">
            <span class="text-muted-foreground">{{ t('ui.duration') }}</span>
            <span class="ml-1 font-bold">
              {{ formatDurationMs(detail.end_to_end_first_byte_time_ms ?? detail.first_byte_time_ms) }} / {{ formatDurationMs(detail.end_to_end_time_ms ?? detail.response_time_ms) }}
            </span>
          </span>
          <span class="text-muted-foreground">|</span>
          <span class="whitespace-nowrap">
            <span class="text-muted-foreground">{{ t('ui.outputSpeed') }}</span>
            <span class="ml-1 font-bold text-primary">{{ formatOutputRateValue(detailOutputRate) }}tps</span>
          </span>
        </div>
        <ServiceTierFacts
          v-if="hasServiceTierFacts && processingTierPriceMultiplier === null"
          class="mt-3"
          :requested="serviceTierFacts.requested"
          :price-multiplier="processingTierPriceMultiplier"
        />
        <div
          v-if="detail.chat_pii_redaction"
          data-request-detail-pii-redaction
          class="mt-3 flex items-start gap-2 rounded-lg border border-emerald-500/30 bg-emerald-500/5 px-3 py-2 text-xs text-emerald-700 dark:text-emerald-400"
        >
          <ShieldCheck class="mt-0.5 h-3.5 w-3.5 shrink-0" />
          <span>{{ t('ui.thisRequestTriggeredSensitiveInformationProtectionSensitiveInformationWasReplaced') }}</span>
        </div>
      </div>

      <!-- 分隔线 -->
      <Separator class="mb-4" />

      <div
        v-if="!detailUsageAvailable"
        data-request-detail-usage-unavailable
        class="rounded-lg border border-dashed border-muted-foreground/30 bg-muted/20 p-3 text-xs text-muted-foreground"
      >
        {{ t('ui.theUpstreamDidNotProvideVerifiableTokenOrCostUsagebb920c5') }}
      </div>
      <div
        v-else-if="!detailPricingAvailable"
        data-request-detail-usage-unpriced
        class="rounded-lg border border-dashed border-muted-foreground/30 bg-muted/20 p-3 text-xs text-muted-foreground"
      >
        {{ t('ui.theUpstreamProvidedAuthoritativeTokenUsageIncludingAudioTokensThat') }} <span v-if="(detail.input_audio_tokens || 0) > 0 || (detail.output_audio_tokens || 0) > 0"> {{ t('ui.audioInputOutput') }}{{ detail.input_audio_tokens || 0 }} / {{ detail.output_audio_tokens || 0 }}。
        </span>
      </div>

      <!-- ========== 1. Token分阶段成本 ========== -->
      <div
        v-if="detailPricingAvailable && hasTokenCost"
        class="space-y-2 mb-3"
      >
        <!-- 阶梯标题 -->
        <div class="text-xs text-muted-foreground flex items-center gap-2 flex-wrap">
          <span class="font-medium text-foreground">{{ t('ui.tokenBilling') }}</span>
          <span class="font-mono font-medium text-foreground">
            <template v-if="processingTierPriceMultiplier !== null">
              ${{ tokenCostBaseTotal.toFixed(6) }} × {{ processingTierPriceMultiplier }} ({{ processingTierLabel }} {{ t('ui.tier0eeaab6') }} </template>
            <template v-else>
              ${{ tokenCostTotal.toFixed(6) }}
            </template>
          </span>
          <span class="text-muted-foreground/60">{{ t('ui.input') }} {{ formatTokens(displayInputTokens) }} {{ t('ui.cacheWrite') }} {{ cacheCreationSummaryText }} {{ t('ui.cacheRead') }} {{ formatTokens(detail.cache_read_input_tokens || 0) }})</span>
          <Badge
            v-if="displayTiers.length > 1"
            variant="outline"
            class="text-[10px] px-1.5 py-0 h-4"
          >
            {{ t('ui.matchedTier') }} {{ currentTierIndex + 1 }} {{ t('ui.tier0e13242') }}
          </Badge>
        </div>

        <!-- 阶梯展示 -->
        <div
          v-for="(tier, index) in displayTiers"
          :key="index"
          class="rounded-lg p-3 space-y-2"
          :class="index === currentTierIndex
            ? 'bg-primary/5 border border-primary/30'
            : 'bg-muted/20 border border-border/50 opacity-60'"
        >
          <!-- 阶梯标题行 -->
          <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-1 sm:gap-2 text-xs">
            <div class="flex items-center gap-2">
              <span
                class="font-medium"
                :class="index === currentTierIndex ? 'text-primary' : 'text-muted-foreground'"
              > {{ t('ui.noaf50a0f') }} {{ index + 1 }} {{ t('ui.tier0e13242') }} </span>
              <span class="text-muted-foreground">
                {{ getTierRangeText(tier, index, displayTiers) }}
              </span>
              <Badge
                v-if="index === currentTierIndex"
                variant="default"
                class="text-[10px] px-1.5 py-0 h-4"
              >
                {{ t('ui.current') }}
              </Badge>
            </div>
            <!-- 单价信息 -->
            <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-muted-foreground sm:hidden">
              <div class="grid grid-cols-[max-content_1fr] items-baseline gap-x-1">
                <span>{{ t('ui.enter') }}</span>
                <span class="text-right">{{ formatPricePerMillion(tier.input_price_per_1m) }}</span>
              </div>
              <div
                class="grid grid-cols-[max-content_1fr] items-baseline gap-x-1"
              >
                <span>{{ t('ui.outputfb04add') }}</span>
                <span class="text-right">{{ formatPricePerMillion(tier.output_price_per_1m) }}</span>
              </div>
              <template v-if="getTierActiveCacheCreationDisplay(tier) || shouldShowCacheReadPrice(tier)">
                <div
                  v-if="getTierActiveCacheCreationDisplay(tier)"
                  class="grid grid-cols-[max-content_1fr] items-baseline gap-x-1"
                >
                  <span>{{ getTierActiveCacheCreationDisplay(tier)?.label }}</span>
                  <span class="text-right">${{ formatPrice(getTierActiveCacheCreationDisplay(tier)?.price || 0) }}/M</span>
                </div>
                <div v-else />
                <div
                  v-if="shouldShowCacheReadPrice(tier)"
                  class="grid grid-cols-[max-content_1fr] items-baseline gap-x-1"
                >
                  <span>{{ t('ui.cacheRead1ded655') }}</span>
                  <span class="text-right">${{ formatPrice(getTierActiveCacheReadPrice(tier) ?? 0) }}/M</span>
                </div>
                <div v-else />
              </template>
            </div>
            <div class="text-muted-foreground hidden items-center gap-2 flex-wrap sm:flex">
              <span>{{ t('ui.enter') }} {{ formatPricePerMillion(tier.input_price_per_1m) }}</span>
              <span>{{ t('ui.outputfb04add') }} {{ formatPricePerMillion(tier.output_price_per_1m) }}</span>
              <span v-if="getTierActiveCacheCreationDisplay(tier)">
                {{ getTierActiveCacheCreationDisplay(tier)?.label }}
                ${{ formatPrice(getTierActiveCacheCreationDisplay(tier)?.price || 0) }}/M
              </span>
              <span v-if="shouldShowCacheReadPrice(tier)"> {{ t('ui.cacheRead3de9e07') }}{{ formatPrice(getTierActiveCacheReadPrice(tier) ?? 0) }}/M
              </span>
            </div>
          </div>

          <!-- 当前阶梯的详细计算 -->
          <template v-if="index === currentTierIndex">
            <div class="grid gap-y-2 sm:hidden">
              <div class="grid grid-cols-[64px_minmax(0,1fr)_92px] items-center gap-x-2">
                <span class="text-xs text-muted-foreground">{{ t('ui.enter') }}</span>
                <span class="text-sm font-semibold font-mono text-right tabular-nums">{{ displayInputTokens }}</span>
                <span class="text-xs font-mono text-right tabular-nums">${{ displayInputCost.toFixed(6) }}</span>
              </div>
              <div class="grid grid-cols-[64px_minmax(0,1fr)_92px] items-center gap-x-2">
                <span class="text-xs text-muted-foreground">{{ t('ui.outputfb04add') }}</span>
                <span class="text-sm font-semibold font-mono text-right tabular-nums">{{ detail.tokens?.output || detail.output_tokens || 0 }}</span>
                <span class="text-xs font-mono text-right tabular-nums">${{ displayOutputCost.toFixed(6) }}</span>
              </div>
              <div class="grid grid-cols-[64px_minmax(0,1fr)_92px] items-center gap-x-2">
                <span class="text-xs text-muted-foreground">{{ t('ui.cacheCreation') }}</span>
                <span class="text-sm font-semibold font-mono text-right tabular-nums">{{ totalCacheCreationTokens }}</span>
                <span class="text-xs font-mono text-right tabular-nums">${{ displayCacheCreationCost.toFixed(6) }}</span>
              </div>
              <div class="grid grid-cols-[64px_minmax(0,1fr)_92px] items-center gap-x-2">
                <span class="text-xs text-muted-foreground">{{ t('ui.cacheRead1ded655') }}</span>
                <span class="text-sm font-semibold font-mono text-right tabular-nums">{{ detail.cache_read_input_tokens || 0 }}</span>
                <span class="text-xs font-mono text-right tabular-nums">${{ displayCacheReadCost.toFixed(6) }}</span>
              </div>
            </div>
            <!-- 输入 输出 -->
            <div class="hidden items-center sm:flex">
              <div class="flex items-center flex-1">
                <span class="text-xs text-muted-foreground w-[56px]">{{ t('ui.enter') }}</span>
                <span class="text-sm font-semibold font-mono flex-1 text-center">{{ displayInputTokens }}</span>
                <span class="text-xs font-mono">${{ displayInputCost.toFixed(6) }}</span>
              </div>
              <Separator
                orientation="vertical"
                class="h-4 mx-4"
              />
              <div class="flex items-center flex-1">
                <span class="text-xs text-muted-foreground w-[56px]">{{ t('ui.outputfb04add') }}</span>
                <span class="text-sm font-semibold font-mono flex-1 text-center">{{ detail.tokens?.output || detail.output_tokens || 0 }}</span>
                <span class="text-xs font-mono">${{ displayOutputCost.toFixed(6) }}</span>
              </div>
            </div>
            <!-- 缓存创建 缓存读取 -->
            <div class="hidden items-center sm:flex">
              <div class="flex items-center flex-1">
                <span class="text-xs text-muted-foreground w-[56px]">{{ cacheCreationSplitRows.length > 0 ? t('ui.totalCacheWrites') : t('ui.cacheCreation') }}</span>
                <span class="text-sm font-semibold font-mono flex-1 text-center">{{ totalCacheCreationTokens }}</span>
                <span class="text-xs font-mono">${{ displayCacheCreationCost.toFixed(6) }}</span>
              </div>
              <Separator
                orientation="vertical"
                class="h-4 mx-4"
              />
              <div class="flex items-center flex-1">
                <span class="text-xs text-muted-foreground w-[56px]">{{ t('ui.cacheRead1ded655') }}</span>
                <span class="text-sm font-semibold font-mono flex-1 text-center">{{ detail.cache_read_input_tokens || 0 }}</span>
                <span class="text-xs font-mono">${{ displayCacheReadCost.toFixed(6) }}</span>
              </div>
            </div>
            <!-- 缓存创建 5m/1h 细分 -->
            <div
              v-if="cacheCreationSplitRows.length > 0"
              class="grid gap-y-1 sm:hidden"
            >
              <div
                v-for="row in cacheCreationSplitRows"
                :key="row.key"
                class="grid grid-cols-[64px_minmax(0,1fr)_68px_92px] items-center gap-x-2 text-xs text-muted-foreground/70"
              >
                <span>{{ row.label }}</span>
                <span class="font-mono text-right tabular-nums text-foreground/90">{{ formatTokens(row.tokens) }}</span>
                <span
                  class="text-right"
                  :class="row.pricePer1M === null ? 'invisible' : ''"
                >
                  {{ row.pricePer1M !== null ? `$${formatPrice(row.pricePer1M)}/M` : '-' }}
                </span>
                <span
                  class="font-mono text-right tabular-nums"
                  :class="row.cost === null ? 'invisible' : ''"
                >
                  {{ row.cost !== null ? `$${row.cost.toFixed(6)}` : '-' }}
                </span>
              </div>
            </div>
            <div
              v-if="cacheCreationSplitRows.length > 0"
              class="hidden space-y-1 pl-[56px] sm:block"
            >
              <div
                v-for="row in cacheCreationSplitRows"
                :key="row.key"
                class="flex items-center gap-4 text-xs text-muted-foreground/70"
              >
                <span class="w-[72px]">{{ row.label }}</span>
                <span class="font-mono text-foreground/90">{{ formatTokens(row.tokens) }}</span>
                <span v-if="row.pricePer1M !== null">${{ formatPrice(row.pricePer1M) }}/M</span>
                <span
                  v-if="row.cost !== null"
                  class="font-mono"
                >${{ row.cost.toFixed(6) }}</span>
              </div>
            </div>
          </template>
        </div>
      </div>

      <!-- ========== 3. 按次计费 ========== -->
      <div
        v-if="detailPricingAvailable && perRequestCost > 0 && !detail.video_billing"
        class="space-y-2 mb-3"
      >
        <div class="flex items-center gap-2 text-xs">
          <span class="font-medium text-foreground">{{ t('ui.perRequestBilling') }}</span>
          <span class="font-mono font-medium text-foreground">${{ perRequestCost.toFixed(6) }}</span>
        </div>
        <div class="rounded-lg p-3 bg-primary/5 border border-primary/30 space-y-2">
          <div
            v-if="effectivePricePerRequest > 0"
            class="flex items-center justify-end text-xs"
          >
            <span class="text-muted-foreground">${{ effectivePricePerRequest.toFixed(6) }}{{ t('ui.requestad9c0d7') }}</span>
          </div>
          <div class="flex items-center">
            <div class="flex items-center flex-1">
              <span class="text-xs text-muted-foreground w-[56px]">{{ t('ui.requestCount') }}</span>
              <span class="text-sm font-semibold font-mono flex-1 text-center">1</span>
              <span class="text-xs font-mono font-medium">${{ perRequestCost.toFixed(6) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- ========== 4. 图片输出计费 ========== -->
      <div
        v-if="detailPricingAvailable && hasImageBillingDetail"
        class="rounded-lg p-3 space-y-2 bg-primary/5 border border-primary/30 mb-3"
      >
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-1 sm:gap-2 text-xs">
          <div class="flex items-center gap-2 flex-wrap">
            <span class="font-medium text-primary">{{ t('ui.imageOutput') }}</span>
            <Badge
              variant="outline"
              class="text-[10px] px-1.5 py-0 h-4"
            >
              {{ imageOutputBillingLabel }}
            </Badge>
            <span
              v-if="imageOutputPricingDescriptor"
              class="text-muted-foreground font-mono"
            >{{ imageOutputPricingDescriptor }}</span>
          </div>
          <div class="text-muted-foreground flex items-center gap-2 flex-wrap">
            <span
              v-if="imageOutputPricePerImage !== null"
              class="font-mono"
            >{{ formatTokens(imageOutputCount) }} {{ t('ui.images') }}{{ imageOutputPricePerImage.toFixed(6) }}{{ t('ui.image04480b6') }}{{ imageOutputCostTotal.toFixed(6) }}</span>
          </div>
        </div>

        <div class="flex items-center">
          <div class="flex items-center flex-1">
            <span class="text-xs text-muted-foreground w-[56px]">{{ t('ui.quantity') }}</span>
            <span class="text-sm font-semibold font-mono flex-1 text-center">{{ formatTokens(imageOutputCount) }}</span>
            <span class="text-xs font-mono">${{ imageOutputCostTotal.toFixed(6) }}</span>
          </div>
          <Separator
            orientation="vertical"
            class="h-4 mx-4"
          />
          <div class="flex items-center flex-1">
            <span class="text-xs text-muted-foreground w-[56px]">{{ t('ui.format') }}</span>
            <span class="text-sm font-semibold font-mono flex-1 text-center">{{ imageOutputFormat || '-' }}</span>
            <span class="text-xs font-mono text-muted-foreground">{{ imageOutputBillingLabel }}</span>
          </div>
        </div>
      </div>

      <!-- ========== 5. 视频/图像/音频计费（独立隔离，与Token计费风格一致） ========== -->
      <div
        v-if="detailPricingAvailable && detail.video_billing"
        class="rounded-lg p-3 space-y-2 bg-primary/5 border border-primary/30"
      >
        <!-- 标题行（与阶梯标题行风格一致） -->
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-1 sm:gap-2 text-xs">
          <div class="flex items-center gap-2">
            <span class="font-medium text-primary">
              {{ getTaskTypeLabel(detail.video_billing.task_type) }}
            </span>
            <span
              v-if="detail.video_billing.resolution"
              class="text-muted-foreground"
            >
              {{ detail.video_billing.resolution }}
            </span>
          </div>
          <!-- 费用计算公式 -->
          <div class="text-muted-foreground flex items-center gap-2 flex-wrap">
            <span
              v-if="detail.video_billing.duration_seconds && detail.video_billing.video_price_per_second"
              class="font-mono"
            >
              {{ detail.video_billing.duration_seconds.toFixed(1) }}s × ${{ detail.video_billing.video_price_per_second.toFixed(4) }}/s = ${{ videoCostTotal.toFixed(6) }}
            </span>
            <span
              v-else-if="detail.video_billing.video_price_per_second"
              class="font-mono"
            >
              ${{ detail.video_billing.video_price_per_second.toFixed(4) }}{{ t('ui.second') }} </span>
          </div>
        </div>

        <!-- 费用详情（与Token详情行风格一致） -->
        <div class="flex items-center">
          <div class="flex items-center flex-1">
            <span class="text-xs text-muted-foreground w-[56px]">
              {{ detail.video_billing.task_type === 'video' ? t('ui.duration04a95e0') : detail.video_billing.task_type === 'audio' ? t('ui.duration04a95e0') : t('ui.quantity') }}
            </span>
            <span class="text-sm font-semibold font-mono flex-1 text-center">
              {{ detail.video_billing.duration_seconds ? formatMediaDuration(detail.video_billing.duration_seconds) : '1' }}
            </span>
            <span class="text-xs font-mono">${{ videoCostTotal.toFixed(6) }}</span>
          </div>
          <Separator
            orientation="vertical"
            class="h-4 mx-4 invisible"
          />
          <div class="flex items-center flex-1 invisible">
            <span class="text-xs text-muted-foreground w-[56px]">{{ t('ui.placeholder064eb44') }}</span>
            <span class="text-sm font-semibold font-mono flex-1 text-center">0</span>
            <span class="text-xs font-mono">$0.000000</span>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed } from 'vue'
import { ShieldCheck } from 'lucide-vue-next'
import Card from '@/components/ui/card.vue'
import Badge from '@/components/ui/badge.vue'
import Separator from '@/components/ui/separator.vue'
import ServiceTierFacts from './ServiceTierFacts.vue'
import type { RequestDetail } from '@/api/dashboard'
import { formatTokens } from '@/utils/format'
import { formatDurationMs, formatOutputRateValue } from '../performance'
import {
  formatPricePerMillion,
  resolveProcessingTierPriceMultiplier,
  resolveSettlementPricingSnapshot,
  resolveSettlementPricingSourceLabel,
  resolveSettlementPricingTiers,
} from '../utils/settlement-pricing'
import {
  formatServiceTierFact,
  hasServiceTierFact,
  resolveServiceTierFacts,
  type ServiceTierFactSource,
} from '../utils/service-tier'
import {
  asRecord,
  toNumber,
  getNestedValue,
  getNestedNumber,
  getNestedString,
  normalizeCacheTtlPricing,
  toFiniteNumber,
  getTierCachePriceForTTL,
  getTierMatchedCachePricingEntry,
  hasTierCacheCreationSplitPricing,
  formatCacheTtlLabel,
  getTaskTypeLabel,
  parseImageSizePixels,
  formatImagePriceBucket,
  formatPixels,
  formatPrice,
  getTierRangeText,
  formatMediaDuration,
} from '../utils/requestDetailDisplay'
import type { CacheTTLPriceEntry, PricingTierLike, JsonRecord } from '../utils/requestDetailDisplay'

/**
 * 请求详情的费用与性能概览卡片：总费用、耗时、输出速度、
 * 以及 Token 阶梯 / 按次 / 图片输出 / 媒体任务四类计费明细。
 */
const props = defineProps<{
  detail: RequestDetail
  headerModelRecord: ServiceTierFactSource | null
  traceRequestMetadata: Record<string, unknown> | null
  usageAvailable: boolean
  pricingAvailable: boolean
  totalCost: number
  displayInputTokens: number
  outputRate: number | null
}>()

const { t } = useI18n()

const detail = computed(() => props.detail)
const headerModelRecord = computed(() => props.headerModelRecord)
const traceRequestMetadata = computed(() => props.traceRequestMetadata)
const detailUsageAvailable = computed(() => props.usageAvailable)
const detailPricingAvailable = computed(() => props.pricingAvailable)
const detailTotalCostForDisplay = computed(() => props.totalCost)
const displayInputTokens = computed(() => props.displayInputTokens)
const detailOutputRate = computed(() => props.outputRate)

const serviceTierFacts = computed(() => resolveServiceTierFacts(headerModelRecord.value))

const hasServiceTierFacts = computed(() => hasServiceTierFact(serviceTierFacts.value))

const settlementPricingSnapshot = computed(() => resolveSettlementPricingSnapshot(detail.value))

const processingTierLabel = computed(() => (
  formatServiceTierFact(serviceTierFacts.value.requested)
  ?? formatServiceTierFact(getNestedString(settlementPricingSnapshot.value, 'billing_processing_tier'))
  ?? '处理'
))

const processingTierPriceMultiplier = computed(() => (
  resolveProcessingTierPriceMultiplier(detail.value)
))

const settlementInfo = computed<JsonRecord | null>(() =>
  asRecord(detail.value?.settlement ?? null),
)

const billingSnapshot = computed<JsonRecord | null>(() =>
  asRecord(settlementInfo.value?.billing_snapshot)
  ?? asRecord(traceRequestMetadata.value?.billing_snapshot),
)

const billingResolvedVariables = computed<JsonRecord | null>(() =>
  asRecord(billingSnapshot.value?.resolved_variables),
)

const billingResolvedDimensions = computed<JsonRecord | null>(() =>
  asRecord(billingSnapshot.value?.resolved_dimensions),
)

const billingCostBreakdown = computed<JsonRecord | null>(() =>
  asRecord(billingSnapshot.value?.cost_breakdown),
)

const billingTierInfo = computed<JsonRecord | null>(() =>
  asRecord(billingSnapshot.value?.tier_info),
)

// 价格来源优先使用 v3 结算快照，旧 tiered_pricing.source 仅作回退。
const priceSourceLabel = computed(() => {
  if (!detail.value) return t('ui.historicalPricing')
  return resolveSettlementPricingSourceLabel(detail.value) ?? '历史定价'
})

const detailPricingLabel = computed(() => {
  if (!detailUsageAvailable.value) return t('ui.usageUnavailable')
  if (!detailPricingAvailable.value) return t('ui.audioUsageUnpriced')
  return priceSourceLabel.value
})

const cacheCreationInputTokens5m = computed(() => {
  if (!detail.value) return 0
  return toNumber(detail.value.cache_creation_input_tokens_5m)
    ?? getNestedNumber(detail.value as unknown as JsonRecord, 'cache_creation_ephemeral_5m_input_tokens')
    ?? 0
})

const cacheCreationInputTokens1h = computed(() => {
  if (!detail.value) return 0
  return toNumber(detail.value.cache_creation_input_tokens_1h)
    ?? getNestedNumber(detail.value as unknown as JsonRecord, 'cache_creation_ephemeral_1h_input_tokens')
    ?? 0
})

const totalCacheCreationTokens = computed(() => {
  if (!detail.value) return 0
  const classified = cacheCreationInputTokens5m.value + cacheCreationInputTokens1h.value
  const total = detail.value.cache_creation_input_tokens || 0
  if (total === 0 && classified > 0) return classified
  return total
})

const effectiveInputPricePer1M = computed(() =>
  getNestedNumber(billingResolvedVariables.value, 'input_price_per_1m')
  ?? toNumber(detail.value?.input_price_per_1m)
)

const effectiveOutputPricePer1M = computed(() =>
  getNestedNumber(billingResolvedVariables.value, 'output_price_per_1m')
  ?? toNumber(detail.value?.output_price_per_1m)
)

const effectiveCacheCreationPricePer1M = computed(() =>
  getNestedNumber(billingResolvedVariables.value, 'cache_creation_price_per_1m')
  ?? toNumber(detail.value?.cache_creation_price_per_1m)
)

const effectiveCacheReadPricePer1M = computed(() =>
  getNestedNumber(billingResolvedVariables.value, 'cache_read_price_per_1m')
  ?? toNumber(detail.value?.cache_read_price_per_1m)
)

const effectivePricePerRequest = computed(() =>
  getNestedNumber(billingResolvedVariables.value, 'price_per_request')
  ?? toNumber(detail.value?.price_per_request)
  ?? 0,
)

const effectiveInputCost = computed(() =>
  getNestedNumber(billingCostBreakdown.value, 'input_cost')
  ?? toNumber(detail.value?.input_cost)
  ?? 0,
)

const effectiveOutputCost = computed(() =>
  getNestedNumber(billingCostBreakdown.value, 'output_cost')
  ?? toNumber(detail.value?.output_cost)
  ?? 0,
)

const effectiveCacheCreationCost = computed(() => {
  const snapshotCost = [
    getNestedNumber(billingCostBreakdown.value, 'cache_creation_uncategorized_cost'),
    getNestedNumber(billingCostBreakdown.value, 'cache_creation_ephemeral_5m_cost'),
    getNestedNumber(billingCostBreakdown.value, 'cache_creation_ephemeral_1h_cost'),
  ].reduce<number>((sum, value) => sum + (value ?? 0), 0)
  if (snapshotCost > 0) return snapshotCost
  return toNumber(detail.value?.cache_creation_cost) ?? 0
})

const effectiveCacheReadCost = computed(() =>
  getNestedNumber(billingCostBreakdown.value, 'cache_read_cost')
  ?? toNumber(detail.value?.cache_read_cost)
  ?? 0,
)

// Fast/Priority 的倍率目录是由后端从 Standard 目录物化出来的。详情页的阶梯与
// 分项成本保留 Standard 基准值，倍率统一显示在 Token 计费标题中，避免把倍率
// 隐藏在每个单价和每个分项成本里。
function removeProcessingTierMultiplier(value: number): number {
  const multiplier = processingTierPriceMultiplier.value
  if (multiplier === null || multiplier <= 0) return value
  return value / multiplier
}

const displayInputCost = computed(() => removeProcessingTierMultiplier(effectiveInputCost.value))

const displayOutputCost = computed(() => removeProcessingTierMultiplier(effectiveOutputCost.value))

const displayCacheCreationCost = computed(() => removeProcessingTierMultiplier(effectiveCacheCreationCost.value))

const displayCacheReadCost = computed(() => removeProcessingTierMultiplier(effectiveCacheReadCost.value))

const effectiveRequestCost = computed(() => {
  const snapshotCost = getNestedNumber(billingCostBreakdown.value, 'request_cost')
  if (snapshotCost !== null) return snapshotCost
  if (effectivePricePerRequest.value > 0) {
    return toNumber(detail.value?.request_cost) ?? 0
  }
  return 0
})

const effectiveImageOutputCost = computed(() =>
  getNestedNumber(billingCostBreakdown.value, 'image_output_cost')
  ?? toNumber(detail.value?.image_output_cost)
  ?? 0,
)

const imageOutputCostTotal = computed(() => effectiveImageOutputCost.value)

const imageOutputPricePerImage = computed(() =>
  getNestedNumber(billingResolvedVariables.value, 'image_output_price_per_image'),
)

const imageOutputCount = computed(() =>
  getNestedNumber(billingResolvedDimensions.value, 'image_count')
  ?? getNestedNumber(traceRequestMetadata.value, 'billing_dimensions', 'image_count')
  ?? getNestedNumber(traceRequestMetadata.value, 'dimensions', 'image_count')
  ?? 0,
)

const imageOutputSize = computed(() =>
  getNestedString(billingResolvedDimensions.value, 'image_size')
  ?? getNestedString(traceRequestMetadata.value, 'billing_dimensions', 'image_size')
  ?? getNestedString(traceRequestMetadata.value, 'dimensions', 'image_size'),
)

const imageOutputQuality = computed(() =>
  getNestedString(billingResolvedDimensions.value, 'image_quality')
  ?? getNestedString(traceRequestMetadata.value, 'billing_dimensions', 'image_quality')
  ?? getNestedString(traceRequestMetadata.value, 'dimensions', 'image_quality'),
)

const imageOutputFormat = computed(() =>
  getNestedString(billingResolvedDimensions.value, 'image_output_format')
  ?? getNestedString(traceRequestMetadata.value, 'billing_dimensions', 'image_output_format')
  ?? getNestedString(traceRequestMetadata.value, 'dimensions', 'image_output_format'),
)

const imagePriceKey = computed(() => {
  const snapshotKey = getNestedString(billingResolvedDimensions.value, 'image_price_key')
  if (snapshotKey) return snapshotKey
  const fallbackKey = [imageOutputSize.value, imageOutputQuality.value].filter(Boolean).join(':')
  return fallbackKey || null
})

const imageOutputPriceBucket = computed(() =>
  getNestedString(billingResolvedDimensions.value, 'image_output_price_bucket'),
)

const imageOutputPixels = computed(() =>
  getNestedNumber(billingResolvedDimensions.value, 'image_pixels')
  ?? parseImageSizePixels(imageOutputSize.value),
)

const imageOutputPricingMode = computed(() =>
  getNestedString(billingResolvedDimensions.value, 'image_output_pricing_mode'),
)

const imageOutputPricingEnabled = computed(() =>
  getNestedValue(billingResolvedDimensions.value, 'image_output_pricing_enabled') === true
    || imageOutputPricingMode.value === 'matrix'
    || imageOutputPricingMode.value === 'pixel_tiers'
    || imageOutputPricingMode.value === 'per_image'
    || imageOutputCostTotal.value > 0,
)

const imageOutputMatrixEnabled = computed(() => {
  if (imageOutputPricingMode.value) return imageOutputPricingMode.value === 'matrix'
  return getNestedValue(billingResolvedDimensions.value, 'image_output_matrix_enabled') === true
})

const imageOutputRangeEnabled = computed(() => {
  if (imageOutputPricingMode.value) return imageOutputPricingMode.value === 'pixel_tiers'
  return getNestedValue(billingResolvedDimensions.value, 'image_output_range_enabled') === true
})

const imageOutputBillingLabel = computed(() => {
  if (imageOutputMatrixEnabled.value) return t('ui.matrixPricing')
  if (imageOutputRangeEnabled.value) return t('ui.pixelRange')
  return t('ui.defaultPricing')
})

const imageOutputPricingDescriptor = computed(() => {
  if (imageOutputMatrixEnabled.value && imagePriceKey.value) return imagePriceKey.value

  const parts: string[] = []
  if (imageOutputPriceBucket.value && imageOutputPriceBucket.value !== 'default') {
    parts.push(formatImagePriceBucket(imageOutputPriceBucket.value))
  }
  const sizeQuality = [imageOutputSize.value, imageOutputQuality.value].filter(Boolean).join(' / ')
  if (sizeQuality) parts.push(sizeQuality)
  if (imageOutputRangeEnabled.value && imageOutputPixels.value !== null) {
    parts.push(formatPixels(imageOutputPixels.value))
  }
  if (parts.length > 0) return parts.join(' · ')
  if (imageOutputPriceBucket.value === 'default') return t('ui.defaultPrice')
  return null
})

const hasImageBillingDetail = computed(() =>
  imageOutputPricingEnabled.value && (imageOutputCount.value > 0 || imageOutputCostTotal.value > 0),
)

const fallbackCacheTtlPricing = computed<CacheTTLPriceEntry[]>(() => {
  const tierPricing = normalizeCacheTtlPricing(billingTierInfo.value?.cache_ttl_pricing)
  if (tierPricing.length > 0) return tierPricing

  const rows: CacheTTLPriceEntry[] = []
  const cache5mCreationPrice = getNestedNumber(
    billingResolvedVariables.value,
    'cache_creation_ephemeral_5m_price_per_1m',
  )
  const cache1hCreationPrice = getNestedNumber(
    billingResolvedVariables.value,
    'cache_creation_ephemeral_1h_price_per_1m',
  )

  if (cache5mCreationPrice !== null) {
    rows.push({
      ttl_minutes: 5,
      cache_creation_price_per_1m: cache5mCreationPrice,
      cache_read_price_per_1m: effectiveCacheReadPricePer1M.value,
    })
  }
  if (cache1hCreationPrice !== null) {
    rows.push({
      ttl_minutes: 60,
      cache_creation_price_per_1m: cache1hCreationPrice,
      cache_read_price_per_1m: effectiveCacheReadPricePer1M.value,
    })
  }
  return rows
})

const activeCacheTtlMinutes = computed(() => {
  const snapshotTtl = getNestedNumber(billingSnapshot.value, 'resolved_dimensions', 'cache_ttl_minutes')
  if (snapshotTtl !== null && snapshotTtl > 0) {
    return Math.trunc(snapshotTtl)
  }
  if (cacheCreationInputTokens1h.value > 0 && cacheCreationInputTokens5m.value === 0) {
    return 60
  }
  if (cacheCreationInputTokens5m.value > 0 && cacheCreationInputTokens1h.value === 0) {
    return 5
  }
  return null
})

// 统一的阶梯显示数据
// 如果有 tiered_pricing，使用它；否则用历史价格构建单阶梯
function restoreBaseTierPricing(tier: PricingTierLike): PricingTierLike {
  const multiplier = processingTierPriceMultiplier.value
  if (multiplier === null || multiplier <= 0) return tier

  const restore = (value: number | null | undefined): number | null | undefined => {
    if (value === null || value === undefined) return value
    return value / multiplier
  }

  return {
    ...tier,
    input_price_per_1m: restore(tier.input_price_per_1m),
    output_price_per_1m: restore(tier.output_price_per_1m),
    cache_creation_price_per_1m: restore(tier.cache_creation_price_per_1m),
    cache_read_price_per_1m: restore(tier.cache_read_price_per_1m),
    cache_ttl_pricing: Array.isArray(tier.cache_ttl_pricing)
      ? tier.cache_ttl_pricing.map(entry => ({
          ...entry,
          cache_creation_price_per_1m: restore(entry.cache_creation_price_per_1m),
          cache_read_price_per_1m: restore(entry.cache_read_price_per_1m),
        }))
      : tier.cache_ttl_pricing,
  }
}

const displayTiers = computed(() => {
  if (!detail.value) return []

  // 优先展示结算快照中已解析的价格目录，旧字段仅作回退。
  const resolvedTiers = resolveSettlementPricingTiers(detail.value)
  if (resolvedTiers) {
    return (resolvedTiers as PricingTierLike[]).map(restoreBaseTierPricing)
  }

  // 否则用历史价格构建单阶梯（无上限）
  return [restoreBaseTierPricing({
    up_to: null,
    input_price_per_1m: effectiveInputPricePer1M.value ?? 0,
    output_price_per_1m: effectiveOutputPricePer1M.value ?? 0,
    cache_creation_price_per_1m: effectiveCacheCreationPricePer1M.value,
    cache_read_price_per_1m: effectiveCacheReadPricePer1M.value,
    cache_ttl_pricing: fallbackCacheTtlPricing.value,
  })]
})

// 当前命中的阶梯索引
const currentTierIndex = computed(() => {
  if (!detail.value) return 0

  // 如果有阶梯定价，使用它的 tier_index
  if (detail.value.tiered_pricing?.tier_index !== undefined) {
    return detail.value.tiered_pricing.tier_index
  }

  const snapshotTierIndex = getNestedNumber(billingSnapshot.value, 'tier_index')
  if (snapshotTierIndex !== null) {
    return Math.max(0, Math.trunc(snapshotTierIndex))
  }

  // 单阶梯时默认是第0阶
  return 0
})

const currentTier = computed<PricingTierLike | null>(() => {
  const tier = displayTiers.value[currentTierIndex.value]
  if (!tier || typeof tier !== 'object') return null
  return tier as PricingTierLike
})

const cacheCreationSummaryText = computed(() => {
  if (!detail.value) return '0'

  const total = totalCacheCreationTokens.value
  const cache5m = cacheCreationInputTokens5m.value
  const cache1h = cacheCreationInputTokens1h.value

  if (cache5m <= 0 && cache1h <= 0) {
    return formatTokens(total)
  }

  const parts: string[] = []
  if (cache5m > 0) parts.push(`5min ${formatTokens(cache5m)}`)
  if (cache1h > 0) parts.push(`1h ${formatTokens(cache1h)}`)
  const remaining = Math.max(0, total - cache5m - cache1h)
  if (remaining > 0) parts.push(`其他 ${formatTokens(remaining)}`)
  return parts.join(' + ')
})

const cacheCreationSplitRows = computed(() => {
  if (!detail.value) return []

  const rows: Array<{
    key: string
    label: string
    tokens: number
    pricePer1M: number | null
    cost: number | null
  }> = []

  const cache5m = cacheCreationInputTokens5m.value
  const cache1h = cacheCreationInputTokens1h.value

  if (cache5m > 0) {
    const pricePer1M = getActiveCachePriceForTTL(5, 'cache_creation_price_per_1m')
    rows.push({
      key: '5m',
      get label() { return t('ui.5MinuteCacheWrite') },
      tokens: cache5m,
      pricePer1M,
      cost: pricePer1M !== null ? (cache5m * pricePer1M) / 1_000_000 : null,
    })
  }

  if (cache1h > 0) {
    const pricePer1M = getActiveCachePriceForTTL(60, 'cache_creation_price_per_1m')
    rows.push({
      key: '1h',
      get label() { return t('ui.1HourCacheWrite') },
      tokens: cache1h,
      pricePer1M,
      cost: pricePer1M !== null ? (cache1h * pricePer1M) / 1_000_000 : null,
    })
  }

  return rows
})

// 总输入上下文（输入 + 缓存创建 + 缓存读取）
const _totalInputContext = computed(() => {
  if (!detail.value) return 0

  // 优先使用 tiered_pricing 中的值
  if (detail.value.tiered_pricing?.total_input_context !== undefined) {
    return detail.value.tiered_pricing.total_input_context
  }

  // 否则手动计算
  const input = detail.value.tokens?.input || detail.value.input_tokens || 0
  const cacheCreation = totalCacheCreationTokens.value
  const cacheRead = detail.value.cache_read_input_tokens || 0
  return input + cacheCreation + cacheRead
})

// Token 费用总计
const tokenCostTotal = computed(() => {
  if (!detail.value) return 0
  return effectiveInputCost.value
    + effectiveOutputCost.value
    + effectiveCacheCreationCost.value
    + effectiveCacheReadCost.value
})

const tokenCostBaseTotal = computed(() => (
  displayInputCost.value
  + displayOutputCost.value
  + displayCacheCreationCost.value
  + displayCacheReadCost.value
))

// 按次计费费用（非视频任务时）
const perRequestCost = computed(() => {
  if (!detail.value) return 0
  // 视频任务的 request_cost 实际上是视频费用，不算按次
  if (detail.value.video_billing) return 0
  return effectiveRequestCost.value
})

// 视频/图像/音频费用
const videoCostTotal = computed(() => {
  if (!detail.value?.video_billing) return 0
  return detail.value.video_billing.video_cost
    || detail.value.video_billing.cost
    || detail.value.request_cost
    || 0
})

// 是否有 Token 费用（用于决定是否显示 Token 计费区块）
const hasTokenCost = computed(() => {
  if (!detail.value) return false
  const inputTokens = detail.value.tokens?.input || detail.value.input_tokens || 0
  const outputTokens = detail.value.tokens?.output || detail.value.output_tokens || 0
  const cacheCreation = totalCacheCreationTokens.value
  const cacheRead = detail.value.cache_read_input_tokens || 0
  return (inputTokens + outputTokens + cacheCreation + cacheRead) > 0 || tokenCostTotal.value > 0
})

function getTierActiveCacheCreationDisplay(
  tier: PricingTierLike | null | undefined,
): { label: string, price: number } | null {
  if (hasTierCacheCreationSplitPricing(tier)) {
    const activeTtl = activeCacheTtlMinutes.value
    if (activeTtl !== null) {
      const matchedEntry = getTierMatchedCachePricingEntry(tier, activeTtl)
      const matchedPrice = getTierCachePriceForTTL(tier, activeTtl, 'cache_creation_price_per_1m')
      if (matchedEntry && matchedPrice !== null) {
        return {
          label: formatCacheTtlLabel(matchedEntry.ttl_minutes),
          price: matchedPrice,
        }
      }
      const fallbackPrice = toFiniteNumber(tier?.cache_creation_price_per_1m)
      if (fallbackPrice !== null) {
        return {
          get label() { return t('ui.cacheCreation') },
          price: fallbackPrice,
        }
      }
    }
  }

  const fallbackPrice = toFiniteNumber(tier?.cache_creation_price_per_1m)
  if (fallbackPrice === null) return null
  return {
    get label() { return t('ui.cacheCreation') },
    price: fallbackPrice,
  }
}

function getActiveCachePriceForTTL(
  ttlMinutes: number,
  priceKey: 'cache_creation_price_per_1m' | 'cache_read_price_per_1m',
): number | null {
  const tierPrice = getTierCachePriceForTTL(currentTier.value, ttlMinutes, priceKey)
  if (tierPrice !== null) return tierPrice

  if (priceKey === 'cache_creation_price_per_1m') {
    return toFiniteNumber(detail.value?.cache_creation_price_per_1m)
  }
  return toFiniteNumber(detail.value?.cache_read_price_per_1m)
}

function getTierActiveCacheReadPrice(tier: PricingTierLike | null | undefined): number | null {
  const activeTtl = activeCacheTtlMinutes.value
  if (activeTtl !== null) {
    const matchedPrice = getTierCachePriceForTTL(tier, activeTtl, 'cache_read_price_per_1m')
    if (matchedPrice !== null) return matchedPrice
  }

  return toFiniteNumber(tier?.cache_read_price_per_1m)
    ?? effectiveCacheReadPricePer1M.value
    ?? null
}

function shouldShowCacheReadPrice(tier: PricingTierLike | null | undefined): boolean {
  return getTierActiveCacheReadPrice(tier) !== null
}

</script>
