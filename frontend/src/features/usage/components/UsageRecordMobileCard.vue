<template>
  <div
    class="border-b border-border/40 px-3 py-2.5"
    :class="isAdmin ? 'cursor-pointer active:bg-muted/30 transition-colors' : ''"
    @click="isAdmin && $emit('showDetail', record.id)"
  >
    <!-- 第一行：模型 + 费用 -->
    <div class="flex items-start justify-between gap-2">
      <div class="min-w-0 flex-1">
        <div class="flex min-w-0 flex-wrap items-center gap-1.5">
          <UsageModelDisplay
            :record="record"
            model-class="text-[15px] font-semibold leading-5"
            stack-full-width
          />
          <ShieldCheck
            v-if="record.chat_pii_redaction"
            data-usage-attempt-marker="pii-redaction"
            class="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400 flex-shrink-0"
            :title="t('ui.thisRequestTriggeredSensitiveInformationProtectionTheContentSentUpstream')"
            :aria-label="t('ui.sensitiveInformationProtectionTriggered')"
          />
          <!-- 状态 Badge -->
          <Badge
            v-if="isUsageRecordFailed(record)"
            variant="destructive"
            class="whitespace-nowrap text-[10px] px-1.5 h-4 leading-4 inline-flex items-center flex-shrink-0"
          >
            {{ t('ui.failed28384d7') }}
          </Badge>
          <Badge
            v-else-if="getDisplayStatus(record) === 'pending'"
            variant="outline"
            class="whitespace-nowrap animate-pulse border-muted-foreground/30 text-muted-foreground text-[10px] px-1.5 h-4 leading-4 inline-flex items-center flex-shrink-0"
          >
            {{ t('ui.waiting251bc40') }}
          </Badge>
          <Badge
            v-else-if="getDisplayStatus(record) === 'streaming'"
            variant="outline"
            class="whitespace-nowrap animate-pulse border-primary/50 text-primary text-[10px] px-1.5 h-4 leading-4 inline-flex items-center flex-shrink-0"
          >
            {{ t('ui.transfer') }}
          </Badge>
          <Badge
            v-else-if="record.status === 'cancelled'"
            variant="outline"
            class="whitespace-nowrap border-amber-500/50 text-amber-600 dark:text-amber-400 text-[10px] px-1.5 h-4 leading-4 inline-flex items-center flex-shrink-0"
          >
            {{ t('common.cancel') }}
          </Badge>
          <Badge
            v-else-if="isUsageWebSocket(record)"
            variant="outline"
            data-usage-transport="websocket"
            :title="getWebSocketTransportTitle(record)"
            class="whitespace-nowrap border-sky-500/50 text-sky-600 dark:text-sky-400 text-[10px] px-1.5 h-4 leading-4 inline-flex items-center flex-shrink-0"
          >
            WS
          </Badge>
          <Badge
            v-else-if="getStreamModeSegments(record).hasConversion"
            :variant="streamBadgeVariant(getStreamModeSegments(record).client === '流式')"
            :class="(streamBadgeVariant(getStreamModeSegments(record).client === '流式') === 'secondary')
              ? 'whitespace-nowrap text-[10px] px-1.5 h-4 leading-4 inline-flex items-center gap-0.5 flex-shrink-0'
              : 'whitespace-nowrap border-border/60 text-muted-foreground text-[10px] px-1.5 h-4 leading-4 inline-flex items-center gap-0.5 flex-shrink-0'"
          >
            <span>{{ getStreamModeSegments(record).client }}</span>
            <span class="opacity-60">→</span>
            <span>{{ getStreamModeSegments(record).upstream }}</span>
          </Badge>
          <Badge
            v-else
            :variant="streamBadgeVariant(getUpstreamStream(record))"
            :class="(streamBadgeVariant(getUpstreamStream(record)) === 'secondary')
              ? 'whitespace-nowrap text-[10px] px-1.5 h-4 leading-4 inline-flex items-center flex-shrink-0'
              : 'whitespace-nowrap border-border/60 text-muted-foreground text-[10px] px-1.5 h-4 leading-4 inline-flex items-center flex-shrink-0'"
          >
            {{ getStreamModeLabel(record) }}
          </Badge>
        </div>
      </div>
      <UsageCostDisplay
        :record="record"
        :show-actual-cost="showActualCost"
        compact
        class="shrink-0"
      />
    </div>

    <!-- 第二行：时间 + API格式 -->
    <div class="mt-1.5 flex min-w-0 items-center gap-1.5 text-[10px] leading-3.5 text-muted-foreground">
      <span class="shrink-0 tabular-nums text-foreground whitespace-nowrap">
        {{ formatRecordTime(record.created_at) }}
      </span>
      <span class="shrink-0 tabular-nums whitespace-nowrap">
        {{ formatRecordShortDate(record.created_at) }}
      </span>
      <template v-if="record.api_format">
        <span class="text-muted-foreground/40">·</span>
        <span class="min-w-0 truncate">{{ formatApiFormat(record.api_format) }}</span>
      </template>
    </div>

    <!-- 用户与上游提供商信息 -->
    <div
      v-if="isAdmin"
      class="mt-1 min-w-0 truncate text-[10px] leading-3.5 text-muted-foreground"
      :title="formatRecordUserSegment(record)"
    >
      {{ formatRecordUserSegment(record) }}
    </div>
    <div
      v-if="isAdmin"
      class="mt-1 flex min-w-0 items-center gap-1.5 text-[10px] leading-3.5"
    >
      <UsageProviderDisplay
        :record="record"
        class="flex-1"
      />
      <!-- 手机与桌面保持相同的标记优先级：发生故障转移时优先显示转移标记。 -->
      <Shuffle
        v-if="record.has_fallback"
        data-usage-attempt-marker="fallback"
        class="w-3.5 h-3.5 text-amber-600 dark:text-amber-400 flex-shrink-0"
        :title="t('ui.thisRequestFailedOverToAnotherProvider')"
        :aria-label="t('ui.providerFailover')"
      />
      <Ban
        v-if="record.has_skipped_candidate && !record.has_fallback"
        data-usage-attempt-marker="skipped-candidate"
        class="w-3.5 h-3.5 text-slate-500 dark:text-slate-400 flex-shrink-0"
        :title="skippedCandidateTooltip(record)"
        :aria-label="skippedCandidateAriaLabel(record)"
      />
      <RefreshCcw
        v-if="record.has_retry"
        data-usage-attempt-marker="retry"
        class="w-3.5 h-3.5 text-blue-600 dark:text-blue-400 flex-shrink-0"
        :title="t('ui.thisRequestWasRetried')"
        :aria-label="t('ui.retried')"
      />
    </div>

    <!-- 第四行：性能指标 -->
    <div class="mt-1.5 flex min-w-0 flex-wrap items-center gap-x-1.5 gap-y-1 text-[10px] leading-3.5 text-muted-foreground">
      <span
        class="min-w-0 truncate whitespace-nowrap tabular-nums text-foreground"
        :title="getRecordPerformanceTitle(record)"
      >
        <span class="text-muted-foreground">{{ t('ui.durationSpeed') }}</span>
        <template v-if="getDisplayStatus(record) === 'pending' || getDisplayStatus(record) === 'streaming'">
          <span class="ml-1">{{ formatRecordDurationSeconds(record.first_byte_time_ms) }}</span>
          <span class="text-muted-foreground"> / </span>
          <ElapsedTimeText
            class="text-primary"
            :created-at="record.created_at"
            :response-time-updated-at="record.response_time_updated_at ?? null"
            :status="getDisplayStatus(record)"
            :response-time-ms="record.response_time_ms ?? null"
          />
          <span class="text-muted-foreground"> / </span>
          <span>{{ formatOutputRate(getRecordDisplayOutputRate(record)) }}</span>
        </template>
        <span
          v-else-if="hasRecordDisplayLatency(record)"
          class="ml-1"
        >{{ formatRecordLatencyPair(record) }} / {{ formatOutputRate(getRecordDisplayOutputRate(record)) }}</span>
        <span
          v-else
          class="ml-1"
        >- / {{ formatOutputRate(getRecordDisplayOutputRate(record)) }}</span>
      </span>
      <span class="text-muted-foreground/40">·</span>
      <span
        class="min-w-0 truncate whitespace-nowrap tabular-nums text-foreground"
        :title="hasRecordCacheTokens(record) ? getRecordCacheTokensTitle(record) : undefined"
      >
        <span class="text-muted-foreground">Tokens</span>
        <span
          v-if="record.usage_available !== false"
          class="ml-1"
        >{{ formatTokens(getRecordEffectiveInputTokens(record)) }} / {{ formatTokens(record.output_tokens || 0) }}</span>
        <span
          v-else
          data-usage-unavailable="tokens"
          class="ml-1 text-muted-foreground"
          :title="t('ui.theUpstreamDidNotProvideVerifiableTokenOrCostUsage')"
        >{{ t('ui.unavailable460b357') }}</span>
        <template v-if="record.usage_available !== false && hasRecordCacheTokens(record)">
          <span class="text-muted-foreground"> | </span>
          <span>{{ formatOptionalTokens(getRecordCacheReadTokens(record)) }} / {{ formatOptionalTokens(getRecordCacheCreationTokens(record)) }}</span>
        </template>
        <template v-if="record.usage_available !== false && ((record.input_audio_tokens || 0) > 0 || (record.output_audio_tokens || 0) > 0)">
          <span class="text-muted-foreground"> {{ t('ui.audio') }} </span>
          <span>{{ formatOptionalTokens(record.input_audio_tokens) }} / {{ formatOptionalTokens(record.output_audio_tokens) }}</span>
        </template>
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { ShieldCheck, Shuffle, Ban, RefreshCcw } from 'lucide-vue-next'
import { Badge } from '@/components/ui'
import { formatTokens } from '@/utils/format'
import { formatApiFormat } from '@/api/endpoints/types/api-format'
import { useDarkMode } from '@/composables/useDarkMode'
import { formatOutputRate } from '../performance'
import { isUsageRecordFailed, isUsageWebSocket } from '../utils/status'
import { formatUsageWebSocketTransportTitle as getWebSocketTransportTitle } from '../utils/websocketTransport'
import {
  getDisplayStatus,
  getStreamModeLabel,
  getStreamModeSegments,
  getUpstreamStream,
  formatRecordShortDate,
  formatRecordTime,
  formatRecordUserSegment,
  getRecordEffectiveInputTokens,
  getRecordCacheReadTokens,
  getRecordCacheCreationTokens,
  formatOptionalTokens,
  hasRecordCacheTokens,
  getRecordCacheTokensTitle,
  formatRecordLatencyPair,
  formatRecordDurationSeconds,
  hasRecordDisplayLatency,
  getRecordDisplayOutputRate,
  getRecordPerformanceTitle,
  skippedCandidateTooltip,
  skippedCandidateAriaLabel,
} from '../utils/usageRecordDisplay'
import type { UsageRecord } from '../types'
import ElapsedTimeText from './ElapsedTimeText.vue'
import UsageModelDisplay from './UsageModelDisplay.vue'
import UsageCostDisplay from './UsageCostDisplay.vue'
import UsageProviderDisplay from './UsageProviderDisplay.vue'

/** 使用记录列表在窄屏下的单条卡片，与桌面表格行展示同一条记录 */
defineProps<{
  record: UsageRecord
  isAdmin: boolean
  showActualCost: boolean
}>()

defineEmits<{
  showDetail: [id: string]
}>()

const { t } = useI18n()

const { isDark } = useDarkMode()

function streamBadgeVariant(isStream: boolean): 'secondary' | 'outline' {
  if (isDark.value) {
    return isStream ? 'outline' : 'secondary'
  }
  return isStream ? 'secondary' : 'outline'
}

</script>
