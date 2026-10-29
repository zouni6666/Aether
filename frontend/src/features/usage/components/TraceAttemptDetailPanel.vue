<template>
  <div
    v-if="selectedGroup && currentAttempt"
    class="detail-panel"
  >
    <div class="panel-header">
      <div class="panel-title">
        <span
          class="title-dot"
          :class="getStatusColorClass(currentAttemptDisplayStatus)"
        />
        <span class="title-text">{{ currentGroupTitle }}</span>
        <a
          v-if="currentAttemptProviderWebsite"
          :href="currentAttemptProviderWebsite"
          target="_blank"
          rel="noopener noreferrer"
          class="provider-link"
          @click.stop
        >
          <ExternalLink class="w-3 h-3" />
        </a>
        <span
          class="status-tag"
          :class="getStatusColorClass(currentAttemptDisplayStatus)"
        >
          {{ currentAttempt.status_code || getStatusLabel(currentAttemptDisplayStatus) }}
        </span>
        <!-- 多 Key 标识 -->
        <template v-if="selectedGroup.retryCount > 0">
          <div class="attempt-switcher">
            <button
              class="attempt-nav-btn"
              :disabled="attemptIndex === 0"
              @click.stop="$emit('navigateAttempt', -1)"
            >
              <ChevronLeft class="w-3 h-3" />
            </button>
            <span class="cache-hint">
              {{ attemptIndex + 1 }}/{{ selectedGroup.allAttempts.length }}
            </span>
            <button
              class="attempt-nav-btn"
              :disabled="attemptIndex === selectedGroup.allAttempts.length - 1"
              @click.stop="$emit('navigateAttempt', 1)"
            >
              <ChevronRight class="w-3 h-3" />
            </button>
          </div>
        </template>
      </div>
      <div class="panel-nav">
        <button
          class="nav-btn"
          :disabled="groupIndex === 0"
          @click.stop="$emit('navigateGroup', -1)"
        >
          <ChevronLeft class="w-4 h-4" />
        </button>
        <span class="nav-info">{{ groupIndex + 1 }} / {{ groupCount }}</span>
        <button
          class="nav-btn"
          :disabled="groupIndex === groupCount - 1"
          @click.stop="$emit('navigateGroup', 1)"
        >
          <ChevronRight class="w-4 h-4" />
        </button>
      </div>
    </div>

    <div class="panel-body">
      <!-- 核心信息网格 -->
      <div class="info-grid">
        <div
          v-if="currentAttemptTimeRange"
          class="info-item"
        >
          <span class="info-label">{{ t('ui.timeRange') }}</span>
          <span class="info-value mono time-range-value">
            {{ formatTime(currentAttemptTimeRange.startIso) }}
            <span class="time-arrow-container">
              <span
                v-if="currentAttemptTimeRange.endIso"
                class="time-duration"
              >+{{ currentAttemptTimeRange.durationLabel }}</span>
              <span class="time-arrow">→</span>
            </span>
            {{ currentAttemptTimeRange.endIso ? formatTime(currentAttemptTimeRange.endIso) : t('ui.inProgress') }}
          </span>
        </div>
        <div
          v-if="currentAttempt.extra_data?.first_byte_time_ms != null"
          class="info-item"
        >
          <span class="info-label">{{ t('ui.firstByteTtfb') }}</span>
          <span class="info-value mono">{{ formatLatency(currentAttempt.extra_data.first_byte_time_ms) }}</span>
        </div>
        <div
          v-if="currentAttemptFormatDisplay"
          class="info-item"
        >
          <span class="info-label">{{ t('ui.format') }}</span>
          <span class="info-value">
            <code class="format-code">{{ currentAttemptFormatDisplay }}</code>
          </span>
        </div>
        <div
          v-if="currentAttemptRequestPathDisplay"
          class="info-item"
        >
          <span class="info-label">{{ t('ui.requestPath') }}</span>
          <span class="info-value">
            <code class="format-code request-path-code">{{ currentAttemptRequestPathDisplay }}</code>
          </span>
        </div>
        <div
          v-if="currentAttemptKeyDisplay"
          class="info-item"
        >
          <span class="info-label">{{ isOAuthType(currentAttempt.key_auth_type) ? t('ui.account') : t('ui.key') }}</span>
          <span class="info-value info-value-stacked">
            <span class="key-name">
              {{ currentAttemptKeyDisplay }}
              <span
                v-if="currentAttempt.key_auth_type && currentAttempt.key_auth_type !== 'api_key'"
                class="auth-type-tag"
              >{{ formatAuthTypeWithPlan(currentAttempt.key_auth_type, currentAttempt.key_oauth_plan_type) }}</span>
            </span>
            <code
              v-if="currentAttempt.key_preview"
              class="key-preview"
            >{{ currentAttempt.key_preview }}</code>
          </span>
        </div>
        <div
          v-if="currentAttemptKeyFormatsDisplay"
          class="info-item"
        >
          <span class="info-label">{{ t('ui.supportedEndpoints') }}</span>
          <span class="info-value info-value-stacked">
            <code class="format-code">{{ currentAttemptKeyFormatsDisplay }}</code>
            <span class="text-xs text-muted-foreground"> {{ t('ui.endpointFormatsAllowedByTheKey') }} </span>
          </span>
        </div>
        <div
          v-if="currentAttempt.extra_data?.proxy"
          class="info-item"
        >
          <span class="info-label">{{ t('ui.proxy') }}</span>
          <span class="info-value info-value-stacked">
            <span class="proxy-name">
              {{ currentAttempt.extra_data.proxy.node_name || currentAttempt.extra_data.proxy.url || t('chart.unknown') }}
              <span
                v-if="currentAttempt.extra_data.proxy.source === 'system'"
                class="text-xs text-muted-foreground ml-1"
              >{{ t('ui.system') }}</span>
            </span>
            <span class="proxy-detail">
              <span
                v-if="currentAttempt.extra_data.proxy.ttfb_ms != null"
                class="text-xs text-muted-foreground"
              >{{ formatLatency(currentAttempt.extra_data.proxy.ttfb_ms) }}</span>
              <span
                v-if="currentAttempt.extra_data.proxy.timing"
                class="text-xs text-muted-foreground"
              >(<!--
                -->{{ proxyTimingBreakdown(currentAttempt.extra_data.proxy) }}<!--
              -->)</span>
            </span>
            <code
              v-if="typeof currentAttempt.extra_data.proxy.node_id === 'string' && currentAttempt.extra_data.proxy.node_id"
              class="text-xs font-mono text-muted-foreground"
            >节点 Key {{ currentAttempt.extra_data.proxy.node_id }}</code>
          </span>
        </div>
        <div
          v-if="currentAttempt.extra_data?.pool_selection"
          class="info-item"
        >
          <span class="info-label">{{ t('ui.poolScheduling') }}</span>
          <span class="info-value info-value-stacked">
            <span class="pool-reason">
              <span
                class="pool-reason-tag"
                :class="'pool-' + currentAttempt.extra_data.pool_selection.reason"
              >
                {{ poolSelectionLabel(currentAttempt.extra_data.pool_selection.reason) }}
              </span>
              <span
                v-if="currentAttempt.extra_data.pool_selection.cost_soft_threshold"
                class="pool-cost-warn"
              >{{ t('ui.nearQuotaLimit') }}</span>
            </span>
            <span
              v-if="currentAttempt.extra_data.pool_selection.cost_window_usage"
              class="text-xs text-muted-foreground"
            >
              {{ formatTokens(currentAttempt.extra_data.pool_selection.cost_window_usage) }}
              <template v-if="currentAttempt.extra_data.pool_selection.cost_limit">
                / {{ formatTokens(currentAttempt.extra_data.pool_selection.cost_limit) }}
              </template>
              tokens
            </span>
          </span>
        </div>
        <div
          v-if="currentAttempt.extra_data?.pool_skip"
          class="info-item"
        >
          <span class="info-label">{{ t('ui.poolSkipped') }}</span>
          <span class="info-value info-value-stacked">
            <span class="pool-skip-type">
              {{ poolSkipLabel(currentAttempt.extra_data.pool_skip.type) }}
            </span>
            <span
              v-if="currentAttempt.extra_data.pool_skip.cooldown_reason"
              class="text-xs text-muted-foreground"
            >
              {{ currentAttempt.extra_data.pool_skip.cooldown_reason }}
              <template v-if="currentAttempt.extra_data.pool_skip.cooldown_ttl != null">
                ({{ currentAttempt.extra_data.pool_skip.cooldown_ttl }}s)
              </template>
            </span>
            <span
              v-if="currentAttempt.extra_data.pool_skip.cost_window_usage"
              class="text-xs text-muted-foreground"
            >
              {{ formatTokens(currentAttempt.extra_data.pool_skip.cost_window_usage) }} tokens
            </span>
          </span>
        </div>
      </div>

      <div
        v-if="currentImageProgress"
        class="image-progress-block"
      >
        <div class="image-progress-header">
          <span class="image-progress-title">{{ t('ui.imageGenerationProgress') }}</span>
          <span
            class="image-progress-phase"
            :class="imageProgressPhaseClass(currentImageProgress.phase)"
          >
            {{ formatImageProgressPhase(currentImageProgress.phase) }}
          </span>
        </div>
        <div class="image-progress-grid">
          <div class="image-progress-item">
            <span class="image-progress-label">{{ t('ui.upstreamTtfb') }}</span>
            <span class="image-progress-value mono">{{ formatLatency(currentImageProgress.upstream_ttfb_ms) }}</span>
          </div>
          <div class="image-progress-item">
            <span class="image-progress-label">{{ t('ui.sseFrames') }}</span>
            <span class="image-progress-value mono">{{ formatProgressCount(currentImageProgress.upstream_sse_frame_count) }}</span>
          </div>
          <div class="image-progress-item">
            <span class="image-progress-label">{{ t('ui.partialImages') }}</span>
            <span class="image-progress-value mono">{{ formatProgressCount(currentImageProgress.partial_image_count) }}</span>
          </div>
          <div class="image-progress-item">
            <span class="image-progress-label">{{ t('ui.lastFrame') }}</span>
            <span class="image-progress-value mono">{{ formatProgressFrameTime(currentImageProgress.last_upstream_frame_at_unix_ms) }}</span>
          </div>
          <template v-if="hasDownstreamHeartbeatProgress">
            <div class="image-progress-item">
              <span class="image-progress-label">{{ t('ui.downstreamHeartbeat') }}</span>
              <span class="image-progress-value mono">{{ formatProgressCount(currentImageProgress.downstream_heartbeat_count) }}</span>
            </div>
            <div class="image-progress-item">
              <span class="image-progress-label">{{ t('ui.heartbeatInterval') }}</span>
              <span class="image-progress-value mono">{{ formatLatency(currentImageProgress.downstream_heartbeat_interval_ms) }}</span>
            </div>
            <div class="image-progress-item">
              <span class="image-progress-label">{{ t('ui.lastHeartbeat') }}</span>
              <span class="image-progress-value mono">{{ formatProgressFrameTime(currentImageProgress.last_downstream_heartbeat_at_unix_ms) }}</span>
            </div>
          </template>
          <div
            v-if="currentImageProgress.last_upstream_event"
            class="image-progress-item full-width"
          >
            <span class="image-progress-label">{{ t('ui.upstreamEvents') }}</span>
            <code class="image-progress-code">{{ currentImageProgress.last_upstream_event }}</code>
          </div>
          <div
            v-if="currentImageProgress.last_client_visible_event"
            class="image-progress-item full-width"
          >
            <span class="image-progress-label">{{ t('ui.clientVisibleEvents') }}</span>
            <code class="image-progress-code">{{ currentImageProgress.last_client_visible_event }}</code>
          </div>
        </div>
      </div>

      <!-- 用量与费用（仅成功节点显示） -->
      <div
        v-if="currentAttempt.status === 'success' && usageData"
        class="usage-section"
      >
        <div class="usage-grid">
          <!-- 输入 输出 -->
          <div class="usage-row">
            <div class="usage-item">
              <span class="usage-label">{{ t('ui.enter') }}</span>
              <span class="usage-tokens">{{ formatTokens(usageData.tokens.input) }}</span>
              <span class="usage-cost">${{ usageData.cost.input.toFixed(6) }}</span>
            </div>
            <div class="usage-divider" />
            <div class="usage-item">
              <span class="usage-label">{{ t('ui.outputfb04add') }}</span>
              <span class="usage-tokens">{{ formatTokens(usageData.tokens.output) }}</span>
              <span class="usage-cost">${{ usageData.cost.output.toFixed(6) }}</span>
            </div>
          </div>
          <!-- 缓存创建 缓存读取（仅在有缓存数据时显示） -->
          <div
            v-if="usageData.tokens.cache_creation || usageData.tokens.cache_read"
            class="usage-row"
          >
            <div class="usage-item">
              <span class="usage-label">{{ t('ui.cacheCreation') }}</span>
              <span class="usage-tokens">{{ formatTokens(usageData.tokens.cache_creation || 0) }}</span>
              <span class="usage-cost">${{ (usageData.cost.cache_creation || 0).toFixed(6) }}</span>
            </div>
            <div class="usage-divider" />
            <div class="usage-item">
              <span class="usage-label">{{ t('ui.cacheRead1ded655') }}</span>
              <span class="usage-tokens">{{ formatTokens(usageData.tokens.cache_read || 0) }}</span>
              <span class="usage-cost">${{ (usageData.cost.cache_read || 0).toFixed(6) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 跳过原因 -->
      <div
        v-if="currentAttemptSkipReasonDisplay"
        class="skip-reason"
      >
        <span class="reason-label">{{ currentAttemptSkipReasonLabel }}</span>
        <span class="reason-content">
          <span class="reason-value">{{ currentAttemptSkipReasonDisplay }}</span>
          <span
            v-if="currentAttemptFailureDiagnostic"
            class="reason-detail"
          >
            <code>{{ currentAttemptFailureDiagnostic.path }}</code>
            {{ currentAttemptFailureDiagnostic.message }}
          </span>
        </span>
      </div>

      <!-- 错误信息：将实际上游响应头和响应体作为同一个对象展示 -->
      <div
        v-if="currentAttempt.status === 'failed' && currentAttemptRequestError"
        class="error-block"
      >
        <div class="error-heading">
          <span class="error-type">{{ t('ui.errorMessage') }}</span>
          <span
            v-if="currentAttemptRequestError.statusCode != null"
            class="error-status-badge"
            :class="currentAttemptRequestError.statusCode >= 400 ? 'is-error' : currentAttemptRequestError.statusCode >= 300 ? 'is-warning' : 'is-success'"
          >
            HTTP {{ currentAttemptRequestError.statusCode }}
          </span>
        </div>
        <div
          v-if="currentAttemptRequestError.upstreamStatusCode != null && currentAttemptRequestError.upstreamStatusCode !== currentAttemptRequestError.statusCode"
          class="error-upstream-status text-xs text-muted-foreground mb-1"
        >
          {{ t('ui.upstreamResponseStatusHttp') }} {{ currentAttemptRequestError.upstreamStatusCode }}{{ t('ui.seeThisAttemptSStatusAbove') }}
        </div>
        <div
          v-if="currentAttemptRequestError.finalStatusCode != null"
          class="error-final-status text-xs text-muted-foreground mb-1"
        >
          {{ t('ui.finalRequestStatusHttp') }} {{ currentAttemptRequestError.finalStatusCode }}{{ t('ui.differsFromThisAttemptSStatus') }}
        </div>
        <div
          v-if="currentAttemptRequestError.message"
          class="error-msg"
        >
          {{ currentAttemptRequestError.message }}
        </div>
        <div
          v-if="currentAttemptRequestError.upstreamResponse"
          class="error-json error-upstream-response-json"
        >
          <JsonContentPanel
            :data="currentAttemptRequestError.upstreamResponse"
            :is-dark="isDark"
            :title="t('ui.upstreamResponse')"
            :empty-message="t('ui.noUpstreamResponse')"
          />
        </div>
      </div>

      <div
        v-if="diagnosticDisplay"
        class="error-json error-diagnostic-json"
      >
        <JsonContentPanel
          :data="diagnosticDisplay"
          :is-dark="isDark"
          :custom-copy="true"
          :copied="diagnosticCopied"
          :copy-disabled="diagnosticCopying"
          :title="t('ui.failureDiagnostics')"
          :empty-message="t('ui.noFailureDiagnostics')"
          @copy="copyFailureDiagnostic"
        />
      </div>
      <p
        v-if="diagnosticDisplay"
        class="text-xs text-muted-foreground mt-1"
      >
        {{ diagnosticCopying ? t('ui.loadingAndRedactingDiagnosticContext') : t('ui.copyLoadsAndRedactsCapturedBodiesMissingForbiddenOrOversized') }}
      </p>

      <!-- 额外数据 -->
      <details
        v-if="currentAttemptExtraDataDisplay"
        class="extra-block"
      >
        <summary class="extra-toggle">
          {{ t('ui.additionalInformation') }}
        </summary>
        <JsonContentPanel
          class="extra-json-panel"
          :data="currentAttemptExtraDataDisplay"
          :is-dark="isDark"
          :empty-message="t('ui.noAdditionalInformation')"
        />
      </details>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { ChevronLeft, ChevronRight, ExternalLink } from 'lucide-vue-next'
import JsonContentPanel from './JsonContentPanel.vue'
import type { CandidateRecord, ImageProgress, RequestTrace } from '@/api/requestTrace'
import { safeExternalWebUrl } from '@/utils/navigationSecurity'
import { formatTokens } from '@/utils/format'
import { formatApiFormat } from '@/api/endpoints/types/api-format'
import { useClipboard } from '@/composables/useClipboard'
import { buildFailureDiagnosticBundle, diagnosticPathFromMessage } from '../utils/failureDiagnostic'
import { prepareDiagnosticExport, sanitizeDiagnostic } from '../utils/diagnosticExport'
import { formatCandidateSkipReason } from '../utils/skipReason'
import {
  formatLatency,
  proxyTimingBreakdown,
  normalizeFormatSignature,
  extractObject,
  readStringField,
  readNumberField,
  hasRenderableValue,
  normalizeImageProgress,
  formatImageProgressPhase,
  imageProgressPhaseClass,
  formatProgressCount,
  formatProgressFrameTime,
  normalizeUpstreamResponseDisplay,
  extractStringList,
  resolveTransportDiagnostics,
  resolveRequestPathFromObject,
  isStreamTerminalDiagnosticMessage,
  isStreamFinishErrorDiagnostic,
  isActionableDiagnosticMessage,
  extractVisibleFailureDiagnostic,
  extractVisibleDiagnosticObjects,
  extractVisibleDiagnosticMessage,
  chooseAttemptRawErrorMessage,
  formatAttemptErrorMessage,
  shouldShowAttemptMessageWithUpstreamResponse,
  diagnosticPathFromObject,
  isOAuthType,
  formatAuthTypeWithPlan,
  poolSelectionLabel,
  poolSkipLabel,
  resolveAttemptTimeRange,
  formatTime,
  getStatusLabel,
  getStatusColorClass,
  getDisplayStatus,
} from '../utils/traceAttemptDisplay'
import type { AttemptTimeRange } from '../utils/traceAttemptDisplay'
import type { TraceNodeGroup, TraceUsageData } from '../utils/traceNodeGroup'
import { getProviderDisplayName } from '../utils/traceNodeGroup'

/**
 * 请求链路时间线中被选中尝试的详情面板：基础信息、图片生成进度、
 * 用量费用、跳过原因、错误信息与失败诊断导出。
 */
const props = defineProps<{
  group: TraceNodeGroup
  attempt: CandidateRecord
  groupIndex: number
  groupCount: number
  attemptIndex: number
  trace: RequestTrace | null
  finalStatus: string
  overrideStatusCode?: number
  requestApiFormat?: string | null
  requestMetadata?: Record<string, unknown> | null
  usageData?: TraceUsageData | null
  isDark: boolean
}>()

defineEmits<{
  navigateGroup: [direction: number]
  navigateAttempt: [direction: number]
}>()

const { t } = useI18n()

const currentAttempt = computed(() => props.attempt)
const selectedGroup = computed(() => props.group)
const trace = computed(() => props.trace)
const computedFinalStatus = computed(() => props.finalStatus)

const currentAttemptProviderWebsite = computed(() =>
  safeExternalWebUrl(currentAttempt.value?.provider_website),
)

const currentAttemptTimeRange = computed<AttemptTimeRange | null>(() => {
  return resolveAttemptTimeRange(currentAttempt.value)
})

const currentAttemptDisplayStatus = computed(() => getDisplayStatus(currentAttempt.value))

const currentGroupTitle = computed(() => {
  if (!selectedGroup.value || !currentAttempt.value) return ''
  if (selectedGroup.value.isPoolGroup) {
    return getProviderDisplayName(currentAttempt.value)
  }
  return selectedGroup.value.providerName
})

const currentImageProgress = computed<ImageProgress | null>(() => {
  const attempt = currentAttempt.value
  if (!attempt) return null
  return normalizeImageProgress(attempt.image_progress)
    ?? normalizeImageProgress(extractObject(attempt.extra_data)?.image_progress)
})

const hasDownstreamHeartbeatProgress = computed(() => {
  const progress = currentImageProgress.value
  return typeof progress?.downstream_heartbeat_count === 'number' ||
    typeof progress?.last_downstream_heartbeat_at_unix_ms === 'number' ||
    typeof progress?.downstream_heartbeat_interval_ms === 'number'
})

const currentAttemptFormatDisplay = computed(() => {
  const attempt = currentAttempt.value
  if (!attempt) return ''
  const extra = extractObject(attempt.extra_data) ?? {}

  const providerRaw = typeof extra.provider_api_format === 'string' ? extra.provider_api_format : ''
  const clientRawFromExtra = typeof extra.client_api_format === 'string' ? extra.client_api_format : ''
  const requestRaw = clientRawFromExtra || (typeof props.requestApiFormat === 'string' ? props.requestApiFormat : '')

  if (!providerRaw && !requestRaw) return ''

  const providerText = providerRaw ? formatApiFormat(providerRaw) : ''
  const requestText = requestRaw ? formatApiFormat(requestRaw) : ''
  const convertedByFlag = extra.needs_conversion === true
  const convertedByDiff = Boolean(
    providerRaw &&
      requestRaw &&
      normalizeFormatSignature(providerRaw) !== normalizeFormatSignature(requestRaw),
  )

  if ((convertedByFlag || convertedByDiff) && requestText && providerText) {
    return `${requestText} -> ${providerText}`
  }

  return providerText || requestText
})

const currentAttemptRequestPathDisplay = computed(() => {
  const attempt = currentAttempt.value
  const fromAttempt = resolveRequestPathFromObject(attempt?.extra_data)
  if (fromAttempt) return fromAttempt

  const fromTrace = resolveRequestPathFromObject(trace.value)
  if (fromTrace) return fromTrace

  const fromRequestMetadata = resolveRequestPathFromObject(props.requestMetadata)
  if (fromRequestMetadata) return fromRequestMetadata

  return ''
})

const currentAttemptKeyDisplay = computed(() => {
  const attempt = currentAttempt.value
  if (!attempt) return ''
  return attempt.key_account_label || attempt.key_name || attempt.key_id || ''
})

const currentAttemptKeyFormatsDisplay = computed(() => {
  const attempt = currentAttempt.value
  if (!attempt) return ''

  const formats = extractStringList(attempt.key_api_formats)
  if (!formats.length) return ''

  return formats
    .map(format => formatApiFormat(format))
    .join(' / ')
})

const currentAttemptSkipReasonDisplay = computed(() => {
  const attempt = currentAttempt.value
  if (!attempt?.skip_reason) return ''

  // transport_unsupported 这类"泛化原因"优先展示后端采集到的具体细节，便于排查。
  if (attempt.skip_reason === 'transport_unsupported') {
    const transportDiagnostics = resolveTransportDiagnostics(attempt)
    const requestPair = extractObject(transportDiagnostics?.request_pair)
    const detailedReason = typeof requestPair?.transport_unsupported_reason === 'string'
      ? requestPair.transport_unsupported_reason.trim()
      : ''
    return detailedReason || formatCandidateSkipReason(attempt.skip_reason)
  }

  return formatCandidateSkipReason(attempt.skip_reason)
})

const currentAttemptSkipReasonLabel = computed(() => {
  const status = currentAttempt.value?.status
  return status === 'skipped' ? '跳过原因（未向上游发起请求）' : '跳过原因'
})

const currentAttemptFailureDiagnostic = computed<{
  path: string
  message: string
} | null>(() => {
  const attempt = currentAttempt.value
  if (!attempt) return null
  const extra = extractObject(attempt.extra_data)
  if (extractObject(extra?.failure_diagnostic)?.safe_to_show === false) return null
  const failureDiagnostic = extractVisibleFailureDiagnostic(extra)
  const error = failureDiagnostic
    ? failureDiagnostic
    : extractObject(extra?.request_body_build_error)
  const path = typeof error?.path === 'string' && error.path.trim()
    ? error.path.trim()
    : ''
  const message = typeof error?.message === 'string' && error.message.trim()
    ? error.message.trim()
    : ''
  if (!path && !message) return null
  return {
    path: path || '$',
    message: formatAttemptErrorMessage(message) || message || '请求体转换失败',
  }
})

const currentAttemptRequestError = computed<{
  message: string
  statusCode?: number
  upstreamStatusCode?: number
  finalStatusCode?: number
  upstreamResponse: Record<string, unknown> | null
  diagnostic: Record<string, unknown> | null
} | null>(() => {
  const attempt = currentAttempt.value
  if (!attempt || attempt.status !== 'failed') return null

  const extra = extractObject(attempt.extra_data)
  const upstreamResponse = extractObject(extra?.upstream_response)
  const errorFlow = extractObject(extra?.error_flow)
  const upstreamStatusCode = readNumberField(upstreamResponse ?? {}, 'status_code')
    ?? readNumberField(upstreamResponse ?? {}, 'statusCode')
  const statusCode = attempt.status_code
    ?? upstreamStatusCode
    ?? readNumberField(errorFlow ?? {}, 'status_code')
    ?? readNumberField(errorFlow ?? {}, 'statusCode')
  const finalStatusCode = !['pending', 'streaming'].includes(computedFinalStatus.value)
    && props.overrideStatusCode != null && props.overrideStatusCode !== statusCode
    ? props.overrideStatusCode
    : undefined
  const flowMessage = errorFlow
    ? readStringField(errorFlow, 'message')
    : ''
  const fallbackMessage = typeof attempt.error_message === 'string' && attempt.error_message.trim()
    ? attempt.error_message.trim()
    : ''
  const fallbackType = typeof attempt.error_type === 'string' && attempt.error_type.trim()
    ? attempt.error_type.trim()
    : ''
  const diagnosticMessage = extractVisibleDiagnosticMessage(extra)
  const rawMessage = chooseAttemptRawErrorMessage(flowMessage || '', fallbackMessage, diagnosticMessage)
  const message = formatAttemptErrorMessage(rawMessage, statusCode) || fallbackType
    || '本次尝试失败，链路追踪未包含详细错误内容。'
  const upstreamResponseDisplay = normalizeUpstreamResponseDisplay(extra?.upstream_response)
  const visibleDiagnosticObjects = extractVisibleDiagnosticObjects(extra)
  const shouldAttachDiagnostic = Boolean(
    visibleDiagnosticObjects.length
      || isActionableDiagnosticMessage(rawMessage),
  )
  const diagnostic = shouldAttachDiagnostic
    ? buildAttemptDiagnosticPayload(
        attempt,
        message || fallbackType || rawMessage || '未知失败',
        statusCode,
        upstreamResponseDisplay,
        rawMessage,
      )
    : null
  const upstreamResponseData: Record<string, unknown> = {}
  const responseHeader = upstreamResponseDisplay?.headers
  const responseBody = upstreamResponseDisplay?.body
  if (hasRenderableValue(responseHeader)) upstreamResponseData.header = responseHeader
  if (hasRenderableValue(responseBody)) upstreamResponseData.body = responseBody
  const response = Object.keys(upstreamResponseData).length > 0
    ? upstreamResponseData
    : null
  const showMessage = shouldShowAttemptMessageWithUpstreamResponse(
    rawMessage || fallbackType,
    upstreamResponseDisplay,
  )

  return {
    message: showMessage ? message : '',
    statusCode,
    upstreamStatusCode,
    finalStatusCode,
    upstreamResponse: response,
    diagnostic,
  }
})

const diagnosticFieldPathFromMessage = diagnosticPathFromMessage

function resolveAttemptDiagnosticBreakpoint(attempt: CandidateRecord, rawMessageOverride = ''): string {
  const extra = extractObject(attempt.extra_data)
  const failureDiagnostic = extractVisibleFailureDiagnostic(extra)
  const requestConversionError = extractObject(extra?.request_conversion_error)
  const requestBodyBuildError = extractObject(extra?.request_body_build_error)
  const errorFlow = extractObject(extra?.error_flow)
  const rawMessage = [
    rawMessageOverride,
    readStringField(errorFlow ?? {}, 'message') ?? '',
    readStringField(failureDiagnostic ?? {}, 'message') ?? '',
    readStringField(requestConversionError ?? {}, 'message') ?? '',
    readStringField(requestBodyBuildError ?? {}, 'message') ?? '',
    typeof attempt.error_message === 'string' ? attempt.error_message : '',
  ].find(item => item.trim()) ?? ''
  const streamFinishReasonPath = isStreamTerminalDiagnosticMessage(rawMessage) ? '$.finish_reason' : ''

  return diagnosticPathFromObject(failureDiagnostic)
    || diagnosticPathFromObject(requestConversionError)
    || diagnosticPathFromObject(requestBodyBuildError)
    || diagnosticFieldPathFromMessage(rawMessage)
    || streamFinishReasonPath
    || '$'
}

function buildAttemptDiagnosticPayload(
  attempt: CandidateRecord,
  summaryInput: string,
  statusCode: number | undefined,
  upstreamResponseDisplay: Record<string, unknown> | null,
  rawMessageForBreakpoint = '',
): Record<string, unknown> | null {
  const extra = extractObject(attempt.extra_data)
  const rawFailureDiagnostic = extractVisibleFailureDiagnostic(extra)
  const rawRequestConversionError = extractObject(extra?.request_conversion_error)
  const rawRequestBodyBuildError = extractObject(extra?.request_body_build_error)
  const hasDiagnostic = Boolean(
    summaryInput
      || rawFailureDiagnostic
      || rawRequestConversionError
      || rawRequestBodyBuildError
      || attempt.error_message
      || attempt.error_type
      || attempt.skip_reason
      || upstreamResponseDisplay,
  )
  if (!hasDiagnostic) return null

  const summary = summaryInput
    || readStringField(rawFailureDiagnostic ?? {}, 'message')
    || readStringField(rawRequestConversionError ?? {}, 'message')
    || readStringField(rawRequestBodyBuildError ?? {}, 'message')
    || (typeof attempt.error_message === 'string' ? formatAttemptErrorMessage(attempt.error_message, attempt.status_code) : '')
    || attempt.error_type
    || currentAttemptSkipReasonDisplay.value
    || '未知失败'
  const breakpoint = resolveAttemptDiagnosticBreakpoint(attempt, rawMessageForBreakpoint)
  const providerFormat = readStringField(extra ?? {}, 'provider_api_format')
  const clientFormat = readStringField(extra ?? {}, 'client_api_format')
    || (typeof props.requestApiFormat === 'string' ? props.requestApiFormat : '')
  const conversionDisplay = clientFormat || providerFormat
    ? `${clientFormat ? formatApiFormat(clientFormat) : '未知请求格式'} → ${providerFormat ? formatApiFormat(providerFormat) : '未知上游格式'}`
    : currentAttemptFormatDisplay.value
  const analysisHint = (() => {
    const raw = `${summary}\n${rawMessageForBreakpoint}\n${attempt.error_message ?? ''}`.toLowerCase()
    if (isStreamFinishErrorDiagnostic(raw)) {
      return t('ui.theUpstreamFinishReasonReportsAStreamingErrorInspectError')
    }
    if (isStreamTerminalDiagnosticMessage(raw)) {
      return t('ui.theFailureIsInUpstreamStreamTerminalValidationCheckThe')
    }
    if (raw.includes('unsupported provider stream event')) {
      return t('ui.theFailureIsInUpstreamStreamingEventParsingOrConversion')
    }
    if (raw.includes('finish reason')) {
      return t('ui.theFailureIsInFinishReasonMappingCheckWhetherAn')
    }
    if (raw.includes('lossy conversion') || raw.includes('无损') || raw.includes('丢失信息') || raw.includes('request_conversion')) {
      return t('ui.theFailureIsInRequestOrResponseConversionCheckWhether')
    }
    return t('ui.startReplayingFromTheBreakpointFieldIfTheBreakpointIsb754dc3')
  })()

  const payload = {
    summary,
    breakpoint,
    analysis_hint: analysisHint,
    request: {
      request_id: trace.value?.request_id ?? attempt.request_id,
      path: currentAttemptRequestPathDisplay.value || null,
      format_conversion: conversionDisplay || null,
      client_api_format: clientFormat || null,
      provider_api_format: providerFormat || null,
      needs_conversion: extra?.needs_conversion ?? null,
      conversion_mode: readStringField(extra ?? {}, 'conversion_mode') ?? null,
    },
    node: {
      candidate_id: attempt.id,
      candidate_index: attempt.candidate_index,
      retry_index: attempt.retry_index,
      provider: attempt.provider_name || attempt.provider_id || null,
      key: currentAttemptKeyDisplay.value || null,
      status: attempt.status,
      status_code: statusCode ?? attempt.status_code ?? null,
      skip_reason: attempt.skip_reason ?? null,
      error_type: attempt.error_type ?? null,
      error_message: attempt.error_message ?? null,
    },
    raw: {
      failure_diagnostic: rawFailureDiagnostic,
      request_conversion_error: rawFailureDiagnostic?.safe_to_show === false || extractObject(extra?.failure_diagnostic)?.safe_to_show === false ? null : rawRequestConversionError,
      request_body_build_error: rawFailureDiagnostic?.safe_to_show === false || extractObject(extra?.failure_diagnostic)?.safe_to_show === false ? null : rawRequestBodyBuildError,
      error_flow: extractObject(extra?.error_flow),
      upstream_response: upstreamResponseDisplay ?? normalizeUpstreamResponseDisplay(extra?.upstream_response),
    },
  }
  return buildFailureDiagnosticBundle(payload, attempt, trace.value, rawMessageForBreakpoint)
}

const { copyToClipboard } = useClipboard()

const diagnosticCopying = ref(false)

const diagnosticCopied = ref(false)

const exportedDiagnostic = ref<Record<string, unknown> | null>(null)

let diagnosticController: AbortController | null = null

let diagnosticCopyTimer: ReturnType<typeof setTimeout> | undefined

const currentAttemptDiagnostic = computed(() => {
  const attempt = currentAttempt.value
  if (!attempt) return null
  if (attempt.status === 'failed') return currentAttemptRequestError.value?.diagnostic ?? null
  if (attempt.status !== 'skipped' || !currentAttemptFailureDiagnostic.value) return null
  return buildAttemptDiagnosticPayload(
    attempt,
    currentAttemptFailureDiagnostic.value.message,
    attempt.status_code,
    normalizeUpstreamResponseDisplay(attempt.extra_data?.upstream_response),
    extractVisibleDiagnosticMessage(attempt.extra_data),
  )
})

const diagnosticDisplay = computed(() => exportedDiagnostic.value
  ?? (currentAttemptDiagnostic.value ? sanitizeDiagnostic(currentAttemptDiagnostic.value) as Record<string, unknown> : null))

function resetDiagnosticCopy() {
  diagnosticController?.abort()
  diagnosticController = null
  clearTimeout(diagnosticCopyTimer)
  diagnosticCopying.value = false
  diagnosticCopied.value = false
  exportedDiagnostic.value = null
}

async function copyFailureDiagnostic() {
  if (!currentAttemptDiagnostic.value || !currentAttempt.value || diagnosticCopying.value) return
  const bundle = JSON.parse(JSON.stringify(currentAttemptDiagnostic.value)) as Record<string, unknown>
  const attempt = JSON.parse(JSON.stringify(currentAttempt.value)) as CandidateRecord
  const currentTrace = trace.value ? { ...trace.value, candidates: [] } : null
  const controller = new AbortController()
  diagnosticController = controller
  diagnosticCopying.value = true
  diagnosticCopied.value = false
  try {
    const diagnostic = await prepareDiagnosticExport(bundle, attempt, currentTrace, controller.signal)
    if (controller.signal.aborted) return
    exportedDiagnostic.value = diagnostic
    const copied = await copyToClipboard(JSON.stringify(diagnostic, null, 2))
    if (controller.signal.aborted) return
    diagnosticCopied.value = copied
    if (copied) diagnosticCopyTimer = setTimeout(() => { diagnosticCopied.value = false }, 2000)
  } catch {
    if (controller.signal.aborted) return
    const fallback = sanitizeDiagnostic({
      ...bundle,
      reproduction: { status: 'insufficient_context', missing_context: ['export_failed'] },
    }) as Record<string, unknown>
    exportedDiagnostic.value = fallback
    diagnosticCopied.value = await copyToClipboard(JSON.stringify(fallback, null, 2))
  } finally {
    if (diagnosticController === controller) {
      diagnosticCopying.value = false
      diagnosticController = null
    }
  }
}

watch([() => props.trace?.request_id, () => currentAttempt.value?.id, () => currentAttempt.value?.error_message], resetDiagnosticCopy)

onBeforeUnmount(resetDiagnosticCopy)

const currentAttemptExtraDataDisplay = computed<Record<string, unknown> | null>(() => {
  const extra = extractObject(currentAttempt.value?.extra_data)
  if (!extra) return null

  const display = { ...extra }
  delete display.upstream_response
  delete display.error_flow
  delete display.client_response
  delete display.provider_response

  return Object.keys(display).length > 0 ? display : null
})

</script>

<style scoped>
/* 详情面板 */
.detail-panel {
  margin-top: 1rem;
  background: hsl(var(--muted) / 0.3);
  border: 1px solid hsl(var(--border));
  border-radius: 14px;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0rem;
  border-bottom: 1px solid hsl(var(--border));
  background: hsl(var(--muted) / 0.4);
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

.title-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.title-dot.status-success { background: #22c55e; }

.title-dot.status-failed { background: #ef4444; }

.title-dot.status-cancelled { background: #f59e0b; }

.title-dot.status-pending { background: #3b82f6; }

.title-dot.status-skipped { background: hsl(var(--foreground)); }

.title-dot.status-available { background: #d1d5db; }

.title-text {
  font-weight: 600;
  font-size: 0.95rem;
}

.panel-nav {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--background));
  border-radius: 6px;
  color: hsl(var(--muted-foreground));
  cursor: pointer;
  transition: all 0.15s ease;
}

.nav-btn:hover:not(:disabled) {
  background: hsl(var(--muted));
  color: hsl(var(--foreground));
  border-color: hsl(var(--muted-foreground) / 0.3);
}

.nav-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.nav-info {
  font-size: 0.8rem;
  font-weight: 500;
  color: hsl(var(--muted-foreground));
  padding: 0 0.5rem;
  min-width: 50px;
  text-align: center;
}

.panel-body {
  padding: 0.75rem 0rem;
}

/* 状态标签 */
.status-tag {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 52px;
  padding: 0.2rem 0.5rem;
  font-size: 0.75rem;
  font-weight: 600;
  border-radius: 6px;
  margin-left: 0.5rem;
}

.status-tag.status-success {
  background: #22c55e20;
  color: #22c55e;
}

.status-tag.status-failed {
  background: #ef444420;
  color: #ef4444;
}

.status-tag.status-cancelled {
  background: #f59e0b20;
  color: #f59e0b;
}

.status-tag.status-pending {
  background: #3b82f620;
  color: #3b82f6;
}

.status-tag.status-skipped {
  background: hsl(var(--foreground) / 0.08);
  color: hsl(var(--foreground));
}

.status-tag.status-available {
  background: hsl(var(--muted));
  color: hsl(var(--muted-foreground));
}

/* 缓存亲和提示 */
.cache-hint {
  display: inline-flex;
  align-items: center;
  padding: 0.15rem 0.5rem;
  font-size: 0.7rem;
  font-weight: 500;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted) / 0.5);
  border-radius: 4px;
}

.attempt-switcher {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  margin-left: 0.5rem;
}

.attempt-nav-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--background));
  border-radius: 9999px;
  color: hsl(var(--muted-foreground));
  cursor: pointer;
  transition: all 0.15s ease;
}

.attempt-nav-btn:hover:not(:disabled) {
  background: hsl(var(--muted));
  color: hsl(var(--foreground));
}

.attempt-nav-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.625rem 1.25rem;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.info-item.full-width {
  grid-column: 1 / -1;
}

.info-label {
  font-size: 0.7rem;
  color: hsl(var(--muted-foreground));
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 500;
}

.info-value {
  font-size: 0.9rem;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.info-value.highlight {
  font-size: 1.1rem;
  font-weight: 600;
  font-family: ui-monospace, monospace;
  color: hsl(var(--primary));
}

.info-value code {
  font-size: 0.7rem;
  padding: 0.15rem 0.375rem;
  background: hsl(var(--muted));
  border-radius: 4px;
  color: hsl(var(--muted-foreground));
  font-family: ui-monospace, monospace;
}

/* 两行堆叠布局 */
.info-value-stacked {
  flex-direction: column;
  align-items: flex-start;
  gap: 0.2rem;
}

/* 格式代码 */
.format-code {
  font-size: 0.75rem;
  padding: 0.1rem 0.3rem;
  background: hsl(var(--muted));
  border-radius: 3px;
  color: hsl(var(--muted-foreground));
  font-family: ui-monospace, monospace;
}

/* Key 信息 */
.key-name {
  font-weight: 500;
}

.key-preview {
  font-size: 0.75rem;
  padding: 0.1rem 0.3rem;
  background: hsl(var(--muted));
  border-radius: 3px;
  color: hsl(var(--muted-foreground));
  font-family: ui-monospace, monospace;
}

/* 认证类型标签 */
.auth-type-tag {
  display: inline-flex;
  align-items: center;
  padding: 0.1rem 0.35rem;
  margin-left: 0.375rem;
  font-size: 0.65rem;
  font-weight: 500;
  color: hsl(var(--primary) / 0.8);
  background: hsl(var(--primary) / 0.08);
  border: 1px solid hsl(var(--primary) / 0.2);
  border-radius: 3px;
}

/* 代理信息 */
.proxy-name {
  font-weight: 500;
}

.proxy-detail {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

/* 号池调度 */
.pool-reason {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.pool-reason-tag {
  display: inline-flex;
  align-items: center;
  padding: 0.15rem 0.5rem;
  font-size: 0.7rem;
  font-weight: 500;
  border-radius: 4px;
  white-space: nowrap;
  border: 1px solid hsl(var(--border));
}

.pool-reason-tag.pool-sticky {
  color: hsl(var(--chart-4));
  border-color: hsl(var(--chart-4) / 0.3);
  background: hsl(var(--chart-4) / 0.08);
}

.pool-reason-tag.pool-lru {
  color: hsl(var(--chart-2));
  border-color: hsl(var(--chart-2) / 0.3);
  background: hsl(var(--chart-2) / 0.08);
}

.pool-reason-tag.pool-random,
.pool-reason-tag.pool-tiebreak {
  color: hsl(var(--muted-foreground));
}

.pool-cost-warn {
  font-size: 0.65rem;
  color: hsl(var(--chart-5));
  font-weight: 500;
}

.pool-skip-type {
  font-weight: 500;
  color: hsl(var(--muted-foreground));
}

.image-progress-block {
  margin-top: 0.875rem;
  padding: 0.75rem;
  border: 1px solid hsl(var(--border) / 0.7);
  border-radius: 8px;
  background: hsl(var(--background) / 0.72);
}

.image-progress-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.625rem;
}

.image-progress-title {
  font-size: 0.82rem;
  font-weight: 600;
}

.image-progress-phase {
  display: inline-flex;
  align-items: center;
  padding: 0.15rem 0.5rem;
  border-radius: 999px;
  font-size: 0.7rem;
  font-weight: 600;
  white-space: nowrap;
  border: 1px solid hsl(var(--border));
}

.image-progress-phase.phase-connecting,
.image-progress-phase.phase-streaming {
  color: #2563eb;
  background: #3b82f614;
  border-color: #3b82f633;
}

.image-progress-phase.phase-completed {
  color: #16a34a;
  background: #22c55e14;
  border-color: #22c55e33;
}

.image-progress-phase.phase-failed {
  color: #dc2626;
  background: #ef444414;
  border-color: #ef444433;
}

.image-progress-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.625rem 0.875rem;
}

.image-progress-item {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.image-progress-item.full-width {
  grid-column: span 2;
}

.image-progress-label {
  font-size: 0.68rem;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
}

.image-progress-value {
  min-width: 0;
  font-size: 0.82rem;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.image-progress-code {
  min-width: 0;
  width: fit-content;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding: 0.12rem 0.35rem;
  border-radius: 4px;
  background: hsl(var(--muted));
  color: hsl(var(--muted-foreground));
  font-size: 0.72rem;
  font-family: ui-monospace, monospace;
}

@media (max-width: 768px) {
  .image-progress-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .image-progress-item.full-width {
    grid-column: 1 / -1;
  }
}

/* Provider 官网链接 */
.provider-link {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.25rem;
  margin-left: 0.25rem;
  color: hsl(var(--muted-foreground));
  border-radius: 4px;
  transition: all 0.15s ease;
}

.provider-link:hover {
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 0.1);
}

.time-arrow {
  color: hsl(var(--muted-foreground));
}

/* 时间范围值 - 紧凑布局 */
.time-range-value {
  gap: 0.25rem !important;
}

/* 箭头容器 - 用于定位持续时间 */
.time-arrow-container {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

/* 持续时间 - 显示在箭头上方 */
.time-duration {
  position: absolute;
  top: -1.1rem;
  left: 50%;
  transform: translateX(-50%);
  font-size: 0.65rem;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
}

/* 用量区域 */
.usage-section {
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px dashed hsl(var(--border));
}

.usage-grid {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  padding: 0.5rem 0.75rem;
  background: hsl(var(--muted) / 0.2);
  border: 1px solid hsl(var(--border) / 0.5);
  border-radius: 8px;
}

.usage-row {
  display: flex;
  align-items: center;
}

.usage-item {
  display: flex;
  align-items: center;
  flex: 1;
}

.usage-label {
  font-size: 0.75rem;
  color: hsl(var(--muted-foreground));
  width: 56px;
  flex-shrink: 0;
}

.usage-tokens {
  font-size: 0.875rem;
  font-weight: 600;
  font-family: ui-monospace, monospace;
  width: 60px;
  flex-shrink: 0;
}

.usage-cost {
  font-size: 0.75rem;
  color: #16a34a;
  font-family: ui-monospace, monospace;
}

.dark .usage-cost {
  color: #4ade80;
}

.usage-divider {
  width: 1px;
  height: 16px;
  background: hsl(var(--border));
  margin: 0 1rem;
}

/* 跳过原因 */
.skip-reason {
  margin-top: 1rem;
  padding: 0.75rem;
  background: hsl(var(--muted) / 0.5);
  border-radius: 8px;
  display: flex;
  gap: 0.75rem;
  font-size: 0.85rem;
}

.reason-label {
  color: hsl(var(--muted-foreground));
  flex-shrink: 0;
}

.reason-value {
  color: hsl(var(--foreground));
}

.reason-content {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.reason-detail {
  color: hsl(var(--muted-foreground));
  line-height: 1.45;
  word-break: break-word;
}

.reason-detail code {
  margin-right: 0.4rem;
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
  background: hsl(var(--background) / 0.8);
  color: hsl(var(--foreground));
  font-size: 0.8rem;
}

/* 错误信息 */
.error-block {
  margin-top: 1rem;
  padding: 0.875rem;
  background: #ef444410;
  border: 1px solid #ef444430;
  border-radius: 8px;
}

.error-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.25rem;
}

.error-type {
  font-size: 0.75rem;
  font-weight: 600;
  color: #ef4444;
  text-transform: uppercase;
  letter-spacing: 0.025em;
}

.error-status-badge {
  flex-shrink: 0;
  padding: 0.125rem 0.45rem;
  border-radius: 999px;
  font-size: 0.72rem;
  font-family: ui-monospace, monospace;
  background: hsl(var(--muted));
  color: hsl(var(--muted-foreground));
}

.error-status-badge.is-success {
  color: #166534;
  background: #22c55e18;
}

.error-status-badge.is-warning {
  color: #92400e;
  background: #f59e0b1f;
}

.error-status-badge.is-error {
  color: #991b1b;
  background: #ef44441f;
}

.error-msg {
  font-size: 0.85rem;
  color: #dc2626;
  word-break: break-word;
}

.error-json {
  margin-top: 0.75rem;
}

.dark .error-status-badge.is-success {
  color: #bbf7d0;
}

.dark .error-status-badge.is-warning {
  color: #fde68a;
}

.dark .error-status-badge.is-error {
  color: #fecaca;
}

/* 额外信息 */
.extra-block {
  margin-top: 1rem;
}

.extra-toggle {
  font-size: 0.8rem;
  color: hsl(var(--muted-foreground));
  cursor: pointer;
  padding: 0.5rem 0;
  user-select: none;
}

.extra-toggle:hover {
  color: hsl(var(--foreground));
}

.extra-json-panel {
  margin-top: 0.5rem;
}
</style>
