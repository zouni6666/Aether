<template>
  <!-- 请求详情抽屉 -->
  <Drawer
    :open="isOpen"
    width="sm:w-[800px] sm:max-w-[90vw]"
    layout="column"
    @close="handleClose"
  >
    <!-- 固定头部 - 整合基本信息 -->
    <div class="sticky top-0 z-10 bg-background border-b px-3 sm:px-6 py-3 sm:py-4 flex-shrink-0">
      <!-- 第一行：标题、模型、状态、操作按钮 -->
      <div class="flex items-center justify-between gap-2 sm:gap-4 mb-3">
        <div class="flex min-w-0 flex-1 items-center gap-2 sm:gap-3 flex-wrap">
          <h3 class="text-lg font-semibold">
            {{ t('ui.requestDetails') }}
          </h3>
          <UsageModelDisplay
            v-if="headerModelRecord"
            :record="headerModelRecord"
            :cyber="detailCyberPolicyError"
            context="detail"
            data-request-detail-model-display
            class="min-w-0 max-w-[18rem] text-sm font-mono text-muted-foreground sm:max-w-none"
            model-row-class="rounded bg-muted px-2 py-0.5"
          />
          <div
            v-else
            data-request-detail-model-display
            class="rounded bg-muted px-2 py-0.5 text-sm font-mono text-muted-foreground"
          >
            -
          </div>
          <Badge
            v-if="detail?.status_code === 200"
            variant="success"
            data-request-lifecycle-status
          >
            {{ detail.status_code }}
          </Badge>
          <Badge
            v-else-if="detail"
            variant="destructive"
            data-request-lifecycle-status
          >
            {{ detail.status_code }}
          </Badge>
          <Badge
            v-if="detail && isUsageWebSocket(detail)"
            variant="outline"
            data-usage-transport="websocket"
            :title="formatUsageWebSocketTransportTitle(detail)"
            class="border-sky-500/50 text-xs text-sky-600 dark:text-sky-400"
          >
            WS
          </Badge>
          <Badge
            v-else-if="detail && resolveUsageStreamLabelSegments(detail).hasConversion"
            data-usage-transport="http"
            :variant="streamBadgeVariant(resolveUsageStreamLabelSegments(detail).client === '流式')"
            :class="streamBadgeVariant(resolveUsageStreamLabelSegments(detail).client === '流式') === 'secondary'
              ? 'text-xs inline-flex items-center gap-1'
              : 'text-xs inline-flex items-center gap-1 border-border/60 text-muted-foreground'"
          >
            <span>HTTP {{ resolveUsageStreamLabelSegments(detail).client }}</span>
            <span class="opacity-60">→</span>
            <span>HTTP {{ resolveUsageStreamLabelSegments(detail).upstream }}</span>
          </Badge>
          <Badge
            v-else-if="detail"
            data-usage-transport="http"
            :variant="streamBadgeVariant(isUsageUpstreamStream(detail))"
            :class="streamBadgeVariant(isUsageUpstreamStream(detail)) === 'secondary'
              ? 'text-xs'
              : 'text-xs border-border/60 text-muted-foreground'"
          >
            HTTP {{ formatUsageStreamLabel(detail) }}
          </Badge>
        </div>
        <div class="flex items-center gap-1 shrink-0">
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8"
            :title="t('ui.replayRequest')"
            :disabled="loading"
            @click="openReplayDialog"
          >
            <Play class="w-4 h-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8"
            :disabled="loading && !autoRefreshing"
            :title="refreshButtonTitle"
            @click="refreshDetail"
          >
            <RefreshCw
              class="w-4 h-4"
              :class="{ 'animate-spin': loading || autoRefreshing, 'text-primary': autoRefreshing }"
            />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8"
            :title="t('ui.close')"
            @click="handleClose"
          >
            <X class="w-4 h-4" />
          </Button>
        </div>
      </div>
      <!-- 第二行：关键元信息 -->
      <div
        v-if="detail"
        class="flex items-center flex-wrap gap-x-2 gap-y-1 text-xs text-muted-foreground sm:gap-x-4"
      >
        <span class="flex min-w-0 items-center gap-1">
          <span class="font-medium text-foreground">ID:</span>
          <span
            class="font-mono"
            :title="fullRequestId"
          >{{ displayRequestId }}</span>
        </span>
        <span class="hidden opacity-40 sm:inline">|</span>
        <span>{{ formatDateTimeFull(detail.created_at, 'N/A') }}</span>
        <span class="hidden opacity-40 sm:inline">|</span>
        <span>{{ formatApiFormat(detail.api_format) }}</span>
        <span class="hidden opacity-40 sm:inline">|</span>
        <span>{{ t('ui.users') }} {{ detail.user?.username || 'Unknown' }}</span>
      </div>
    </div>

    <!-- 可滚动内容区域 -->
    <div class="flex-1 min-h-0 overflow-y-auto px-3 sm:px-6 py-3 sm:py-4 scrollbar-stable">
      <!-- Loading State -->
      <div
        v-if="loading"
        class="py-8 space-y-4"
      >
        <Skeleton class="h-8 w-full" />
        <Skeleton class="h-32 w-full" />
        <Skeleton class="h-64 w-full" />
      </div>

      <!-- Error State -->
      <Card
        v-else-if="error"
        class="border-red-200 dark:border-red-800"
      >
        <div class="p-4">
          <p class="text-sm text-red-600 dark:text-red-400">
            {{ error }}
          </p>
        </div>
      </Card>

      <!-- Detail Content -->
      <div
        v-else-if="detail"
        class="space-y-4"
      >
        <!-- 费用与性能概览 -->
        <RequestCostOverviewCard
          :detail="detail"
          :header-model-record="headerModelRecord"
          :trace-request-metadata="traceRequestMetadata"
          :usage-available="detailUsageAvailable"
          :pricing-available="detailPricingAvailable"
          :total-cost="detailTotalCostForDisplay"
          :display-input-tokens="displayInputTokens"
          :output-rate="detailOutputRate"
        />

        <!-- 请求链路追踪卡片 -->
        <div>
          <HorizontalRequestTimeline
            v-if="showTimeline && traceTimelineRequestId"
            ref="timelineRef"
            :request-id="traceTimelineRequestId"
            :override-status-code="detail.status_code"
            :request-status="detail.status"
            :request-api-format="detail.api_format || null"
            :request-metadata="traceRequestMetadata"
            @trace-state="handleTraceState"
          />
        </div>

        <!-- Tabs 区域 -->
        <Card>
          <div class="p-3 sm:p-4">
            <Tabs
              v-model="activeTab"
              :default-value="activeTab"
            >
              <!-- Tab 行 -->
              <div class="flex items-center border-b pb-2 mb-3">
                <button
                  v-for="tab in visibleTabs"
                  :key="tab.name"
                  class="px-2 sm:px-3 py-1.5 text-sm transition-colors border-b-2 -mb-[9px] whitespace-nowrap"
                  :class="activeTab === tab.name
                    ? 'border-primary text-foreground font-medium'
                    : 'border-transparent text-muted-foreground hover:text-foreground'"
                  @click="activeTab = tab.name"
                >
                  {{ tab.label }}
                </button>
              </div>

              <!-- Tab 内容（统一容器） -->
              <JsonContentPanel
                class="content-block"
                :title="activeJsonPanelTitle"
                :data="activeJsonPanelData"
                :is-dark="isDark"
                :expand-depth="currentExpandDepth"
                :copied="Boolean(copiedStates[activeTab])"
                :custom-copy="true"
                :expand-disabled="viewMode === 'compare' || (supportsConversationView && contentViewMode === 'conversation')"
                :copy-disabled="viewMode === 'compare' || bodyCopying || (supportsConversationView && !hasValidConversation)"
                max-height="500px"
                @update:expand-depth="currentExpandDepth = $event"
                @copy="copyContent(activeTab)"
              >
                <template #toolbar-actions-before>
                  <!-- 区域1：条件性按钮（cURL、视图切换、对比） -->
                  <!-- cURL 复制（仅在请求头/请求体 Tab） -->
                  <template v-if="['request-headers', 'request-body'].includes(activeTab)">
                    <button
                      :title="curlCopied ? t('ui.curlCopied') : t('ui.copyCurl')"
                      class="p-1 rounded transition-colors text-muted-foreground hover:bg-muted"
                      :disabled="curlCopying"
                      @click="copyCurlCommand"
                    >
                      <Check
                        v-if="curlCopied"
                        class="w-3.5 h-3.5 text-green-500"
                      />
                      <Terminal
                        v-else
                        class="w-3.5 h-3.5"
                        :class="{ 'animate-pulse': curlCopying }"
                      />
                    </button>
                  </template>

                  <!-- 请求体/响应体专用：JSON/对话 视图切换 -->
                  <template v-if="supportsConversationView">
                    <button
                      :title="contentViewMode === 'json' ? t('ui.switchToConversationView') : t('ui.switchToJsonView')"
                      class="p-1 rounded transition-colors"
                      :class="hasValidConversation || contentViewMode === 'conversation'
                        ? 'text-muted-foreground hover:bg-muted'
                        : 'text-muted-foreground/40 cursor-not-allowed'"
                      :disabled="!hasValidConversation && contentViewMode === 'json'"
                      @click="toggleContentView"
                    >
                      <Code2
                        v-if="contentViewMode === 'conversation'"
                        class="w-3.5 h-3.5"
                      />
                      <MessageSquareText
                        v-else
                        class="w-3.5 h-3.5"
                      />
                    </button>
                  </template>

                  <!-- 请求头/响应头：对比模式 -->
                  <template v-if="canCompare">
                    <button
                      :title="t('ui.compare')"
                      class="p-1 rounded transition-colors"
                      :class="viewMode === 'compare' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted'"
                      @click="viewMode = 'compare'"
                    >
                      <Columns2 class="w-3.5 h-3.5" />
                    </button>
                  </template>

                  <!-- 区域2：客户端/提供商切换 -->
                  <template v-if="showDataSourceToggle">
                    <div class="w-px h-3.5 bg-border mx-0.5" />
                    <button
                      :title="t('ui.client')"
                      class="p-1 rounded transition-colors"
                      :class="activeDataSource === 'client' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted'"
                      @click="setDataSource('client')"
                    >
                      <Monitor class="w-3.5 h-3.5" />
                    </button>
                    <button
                      :title="t('nav.providers')"
                      class="p-1 rounded transition-colors"
                      :class="activeDataSource === 'provider' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted'"
                      @click="setDataSource('provider')"
                    >
                      <Server class="w-3.5 h-3.5" />
                    </button>
                  </template>

                  <!-- 区域3：常驻按钮（展开/收缩、复制） -->
                  <div class="w-px h-3.5 bg-border mx-0.5" />
                </template>
                <TabsContent
                  v-if="activeTab === 'request-headers'"
                  value="request-headers"
                >
                  <RequestHeadersContent
                    :detail="detail"
                    :view-mode="viewMode"
                    :data-source="dataSource"
                    :current-header-data="currentHeaderData ?? null"
                    :current-expand-depth="currentExpandDepth"
                    :has-provider-headers="hasProviderHeaders"
                    :header-stats="headerStats"
                    :is-dark="isDark"
                  />
                </TabsContent>

                <TabsContent
                  v-if="activeTab === 'request-body'"
                  value="request-body"
                >
                  <div
                    v-if="isRequestBodyLoading"
                    class="p-4"
                  >
                    <Skeleton class="h-32 w-full" />
                  </div>
                  <div
                    v-else-if="requestBodyLoadErrorVisible"
                    class="m-4 rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive"
                  >
                    <div class="flex items-start gap-2">
                      <AlertTriangle class="mt-0.5 h-4 w-4 flex-shrink-0" />
                      <span>{{ bodyLoadErrorMessage }}</span>
                    </div>
                    <Button
                      v-if="canRetryBodyContentLoad"
                      variant="outline"
                      size="sm"
                      class="mt-3 h-8 border-destructive/40 text-destructive hover:bg-destructive/10 hover:text-destructive"
                      @click="retryBodyContentLoad"
                    >
                      <RefreshCw class="mr-1.5 h-3.5 w-3.5" /> {{ t('vscodex.connection.retry') }}
                    </Button>
                  </div>
                  <BodyConversationContent
                    v-else-if="contentViewMode === 'conversation'"
                    :body-document="currentRequestBody"
                    kind="request"
                    :api-format="currentRequestBodyApiFormat"
                    :empty-message="t('ui.noRequestBodyInformation')"
                    @load-error="handleBodyDocumentError"
                  />
                  <JsonContent
                    v-else
                    :data="null"
                    :body-document="currentRequestBody"
                    :view-mode="viewMode"
                    :expand-depth="currentExpandDepth"
                    :is-dark="isDark"
                    :empty-message="t('ui.noRequestBodyInformation')"
                    @load-error="handleBodyDocumentError"
                  />
                </TabsContent>

                <TabsContent
                  v-if="activeTab === 'response-headers'"
                  value="response-headers"
                >
                  <RequestHeadersContent
                    v-if="viewMode === 'compare'"
                    :detail="detail"
                    :view-mode="viewMode"
                    :data-source="dataSource"
                    :current-header-data="currentResponseHeaderData ?? null"
                    :current-expand-depth="currentExpandDepth"
                    :has-provider-headers="hasProviderResponseHeaders"
                    :header-stats="responseHeaderStats"
                    :is-dark="isDark"
                    :client-headers="detail.client_response_headers"
                    :provider-headers="detail.response_headers"
                    :client-label="t('ui.clientResponseHeaders')"
                    :provider-label="t('ui.providerResponseHeaders')"
                    :empty-message="t('ui.noResponseHeadersaf5d1b5')"
                  />
                  <JsonContent
                    v-else
                    :data="currentResponseHeaderData"
                    :view-mode="viewMode"
                    :expand-depth="currentExpandDepth"
                    :is-dark="isDark"
                    :empty-message="t('ui.noResponseHeadersaf5d1b5')"
                  />
                </TabsContent>

                <TabsContent
                  v-if="activeTab === 'response-body'"
                  value="response-body"
                >
                  <div
                    v-if="isResponseBodyLoading"
                    class="p-4"
                  >
                    <Skeleton class="h-32 w-full" />
                  </div>
                  <div
                    v-else-if="responseBodyLoadErrorVisible"
                    class="m-4 rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive"
                  >
                    <div class="flex items-start gap-2">
                      <AlertTriangle class="mt-0.5 h-4 w-4 flex-shrink-0" />
                      <span>{{ bodyLoadErrorMessage }}</span>
                    </div>
                    <Button
                      v-if="canRetryBodyContentLoad"
                      variant="outline"
                      size="sm"
                      class="mt-3 h-8 border-destructive/40 text-destructive hover:bg-destructive/10 hover:text-destructive"
                      @click="retryBodyContentLoad"
                    >
                      <RefreshCw class="mr-1.5 h-3.5 w-3.5" /> {{ t('vscodex.connection.retry') }}
                    </Button>
                  </div>
                  <BodyConversationContent
                    v-else-if="contentViewMode === 'conversation'"
                    :body-document="currentResponseBody"
                    kind="response"
                    :api-format="currentResponseBodyApiFormat"
                    :empty-message="t('ui.noResponseBodyInformation')"
                    @load-error="handleBodyDocumentError"
                  />
                  <JsonContent
                    v-else
                    :data="null"
                    :body-document="currentResponseBody"
                    :view-mode="viewMode"
                    :expand-depth="currentExpandDepth"
                    :is-dark="isDark"
                    :empty-message="t('ui.noResponseBodyInformation')"
                    @load-error="handleBodyDocumentError"
                  />
                </TabsContent>

                <TabsContent
                  v-if="activeTab === 'metadata'"
                  value="metadata"
                >
                  <JsonContent
                    :data="metadataPanelData"
                    :view-mode="viewMode"
                    :expand-depth="currentExpandDepth"
                    :is-dark="isDark"
                    :empty-message="t('ui.noMetadata')"
                  />
                </TabsContent>
              </JsonContentPanel>
            </Tabs>
          </div>
        </Card>
      </div>
    </div>
  </Drawer>

  <!-- 请求回放对话框 -->
  <ReplayDialog
    :is-open="replayDialogOpen"
    :request-id="requestId"
    :detail="detail"
    @close="replayDialogOpen = false"
  />
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { ref, shallowRef, watch, computed, nextTick, onMounted, onBeforeUnmount } from 'vue'
import axios from 'axios'
import Button from '@/components/ui/button.vue'
import Drawer from '@/components/ui/drawer/Drawer.vue'
import { useClipboard } from '@/composables/useClipboard'
import { useDarkMode } from '@/composables/useDarkMode'
import Card from '@/components/ui/card.vue'
import Badge from '@/components/ui/badge.vue'
import Skeleton from '@/components/ui/skeleton.vue'
import Tabs from '@/components/ui/tabs.vue'
import TabsContent from '@/components/ui/tabs-content.vue'
import { AlertTriangle, Check, Columns2, RefreshCw, X, Monitor, Server, MessageSquareText, Code2, Terminal, Play } from 'lucide-vue-next'
import { dashboardApi, type RequestBodyField, type RequestDetail } from '@/api/dashboard'
import { formatRequestBodyLoadError, formatStoredBodyLoadError } from '../utils/body-load-error'
import {
  asRecord,
  formatMetadataDisplayValue,
  mapTraceFinalStatusToRequestStatus,
  resolveRequestStateStatus,
  resolveRequestStateStatusFromDetail,
  detailTotalCost,
  detailCacheCreation5mTokens,
  detailCacheCreation1hTokens,
  getNestedString,
  getCaseInsensitiveString,
  hasContent,
} from '../utils/requestDetailDisplay'
import type { RequestStateStatus } from '../utils/requestDetailDisplay'
import type { ImageProgress, RequestTrace } from '@/api/requestTrace'
import type { UsageRecord } from '../types'
import { formatApiFormat } from '@/api/endpoints/types/api-format'
import {
  formatDateTimeFull,
  formatShortRequestId,
} from '@/utils/format'
import { log } from '@/utils/logger'
import { getEffectiveInputTokens } from '../token-normalization'
import {
  getDisplayOutputRate,
} from '../performance'
import {
  formatUsageStreamLabel,
  isUsageUpstreamStream,
  isUsageWebSocket,
  resolveDisplayRequestStatus,
  resolveUsageStreamLabelSegments,
} from '../utils/status'
import { formatUsageWebSocketTransportTitle } from '../utils/websocketTransport'
import { isCyberPolicyError } from '../utils/cyberError'
import {
  mergeUsageRecordErrorMessage,
  parseUsageTimestampMs,
} from '../utils/recordSync'

// 子组件
import RequestHeadersContent from './RequestDetailDrawer/RequestHeadersContent.vue'
import JsonContent from './RequestDetailDrawer/JsonContent.vue'
import JsonContentPanel from './JsonContentPanel.vue'
import BodyConversationContent from './RequestDetailDrawer/BodyConversationContent.vue'
import { BodyDocument } from '../utils/body-document'
import { BodyDocumentError, MAX_BODY_BYTES } from '../utils/body-document-protocol'
import { useToast } from '@/composables/useToast'
import HorizontalRequestTimeline from './HorizontalRequestTimeline.vue'
import ReplayDialog from './ReplayDialog.vue'
import RequestCostOverviewCard from './RequestCostOverviewCard.vue'
import UsageModelDisplay from './UsageModelDisplay.vue'

const props = defineProps<{
  isOpen: boolean
  requestId: string | null
  summaryRecord?: UsageRecord | null
}>()

const emit = defineEmits<{
  close: []
  requestState: [state: {
    id: string
    requestId?: string | null
    status?: RequestStateStatus
    statusCode?: number | null
    inputTokens?: number | null
    effectiveInputTokens?: number | null
    outputTokens?: number | null
    totalTokens?: number | null
    cacheCreationInputTokens?: number | null
    cacheCreationEphemeral5mInputTokens?: number | null
    cacheCreationEphemeral1hInputTokens?: number | null
    cacheReadInputTokens?: number | null
    cost?: number | null
    actualCost?: number | null
    billingMultiplier?: number | null
    billingCost?: number | null
    routingGroupId?: string | null
    routingGroupName?: string | null
    responseTimeMs?: number | null
    firstByteTimeMs?: number | null
    isStream?: boolean | null
    isWebSocket?: boolean | null
    websocketTransport?: string | null
    usageAvailable?: boolean | null
    usagePricingAvailable?: boolean | null
    inputAudioTokens?: number | null
    outputAudioTokens?: number | null
    upstreamIsStream?: boolean | null
    clientRequestedStream?: boolean | null
    clientIsStream?: boolean | null
    apiFormat?: string | null
    endpointApiFormat?: string | null
    hasFormatConversion?: boolean | null
    chatPiiRedaction?: boolean | null
    targetModel?: string | null
    requestedReasoningEffort?: string | null
    reasoningEffort?: string | null
    serviceTier?: string | null
    actualServiceTier?: string | null
    responseModel?: string | null
    imageProgress?: ImageProgress | null
    errorMessage?: string | null
    updatedAt?: string | null
  }]
}>()

const { t } = useI18n()

const loading = ref(false)
const error = ref<string | null>(null)
const detail = shallowRef<RequestDetail | null>(null)
const detailUsageAvailable = computed(() => detail.value?.usage_available !== false)
const detailPricingAvailable = computed(() => (
  detailUsageAvailable.value && detail.value?.usage_pricing_available !== false
))
const detailTotalCostForDisplay = computed(() => (
  detail.value ? (detailTotalCost(detail.value) ?? 0) : 0
))
const timelineRef = ref<InstanceType<typeof HorizontalRequestTimeline> | null>(null)
const timelineLoaded = ref(false)
const timelineHasTrace = ref(false)
const activeTab = ref('request-body')
const copiedStates = ref<Record<string, boolean>>({})
const viewMode = ref<'compare' | 'formatted' | 'raw'>('formatted')
const currentExpandDepth = ref(0)
const dataSource = ref<'client' | 'provider'>('provider')
const contentViewMode = ref<'json' | 'conversation'>('json')
const { copyToClipboard } = useClipboard()
const { error: showBodyError } = useToast()
const historicalPricing = ref<{
  input_price: string
  output_price: string
  cache_creation_price: string
  cache_read_price: string
  request_price: string
} | null>(null)

type HeaderModelTextField =
  | 'model'
  | 'target_model'
  | 'model_version'
  | 'response_model'
  | 'request_type'
  | 'requested_reasoning_effort'
  | 'reasoning_effort'
  | 'service_tier'
  | 'actual_service_tier'

const FINAL_PROVIDER_HEADER_FIELDS = new Set<HeaderModelTextField>([
  'target_model',
  'response_model',
  'reasoning_effort',
  'service_tier',
  'actual_service_tier',
])

let modelSnapshotRevision = 0
const summaryModelRevision = ref(0)
const detailModelRevision = ref(0)

function usageSnapshotUpdatedAtMs(
  source: UsageRecord | RequestDetail | null | undefined,
): number | null {
  const value = source?.updated_at
  if (typeof value !== 'string' || !value.trim()) return null
  return parseUsageTimestampMs(value)
}

function summaryNullIsNewerForProviderField(
  field: HeaderModelTextField,
  nextDetail: RequestDetail | null | undefined,
): boolean {
  if (!FINAL_PROVIDER_HEADER_FIELDS.has(field) || !props.summaryRecord) return false

  const summaryUpdatedAt = usageSnapshotUpdatedAtMs(props.summaryRecord)
  const detailUpdatedAt = usageSnapshotUpdatedAtMs(nextDetail)
  if (summaryUpdatedAt != null && detailUpdatedAt != null && summaryUpdatedAt !== detailUpdatedAt) {
    return summaryUpdatedAt > detailUpdatedAt
  }

  if (summaryModelRevision.value > detailModelRevision.value) return true

  // A terminal list row is a complete final-provider snapshot. When no
  // comparable timestamps exist, its explicit null must beat a cached detail
  // from an earlier candidate. Non-terminal rows may still be filled by a
  // detail request that completed after the lightweight list response.
  return ['completed', 'failed', 'cancelled'].includes(props.summaryRecord.status ?? '')
}

function readHeaderModelTextField(
  source: UsageRecord | RequestDetail | null | undefined,
  field: HeaderModelTextField,
): { resolved: boolean, value: string | null } {
  if (!source || !Object.prototype.hasOwnProperty.call(source, field)) {
    return { resolved: false, value: null }
  }

  const value = (source as unknown as Record<string, unknown>)[field]
  if (value === null) return { resolved: true, value: null }
  if (typeof value !== 'string') return { resolved: false, value: null }

  const normalized = value.trim()
  return { resolved: true, value: normalized || null }
}

function resolveHeaderModelTextField(
  field: HeaderModelTextField,
  nextDetail: RequestDetail | null | undefined,
): string | null | undefined {
  // Prefer a populated list/active fact so sparse detail cannot make the header
  // flicker. A summary null is often only a lightweight-contract placeholder,
  // though, so a later populated detail is still useful. Final-provider stale
  // facts are cleared when full list/active snapshots merge into the summary.
  const summaryValue = readHeaderModelTextField(props.summaryRecord, field)
  if (summaryValue.value) return summaryValue.value

  const detailValue = readHeaderModelTextField(nextDetail, field)
  if (detailValue.value) {
    if (
      summaryValue.resolved
      && summaryValue.value === null
      && summaryNullIsNewerForProviderField(field, nextDetail)
    ) return null
    return detailValue.value
  }

  return summaryValue.resolved || detailValue.resolved ? null : undefined
}

watch(
  () => [
    props.requestId,
    props.summaryRecord?.status,
    props.summaryRecord?.updated_at,
    props.summaryRecord?.model,
    props.summaryRecord?.target_model,
    props.summaryRecord?.model_version,
    props.summaryRecord?.response_model,
    props.summaryRecord?.request_type,
    props.summaryRecord?.requested_reasoning_effort,
    props.summaryRecord?.reasoning_effort,
    props.summaryRecord?.service_tier,
    props.summaryRecord?.actual_service_tier,
  ],
  () => {
    summaryModelRevision.value = ++modelSnapshotRevision
  },
  { immediate: true },
)

function emitDetailRequestState(nextDetail: RequestDetail) {
  const id = props.requestId
  if (!id) return

  const targetModel = resolveHeaderModelTextField('target_model', nextDetail)
  const requestedReasoningEffort = resolveHeaderModelTextField(
    'requested_reasoning_effort',
    nextDetail,
  )
  const reasoningEffort = resolveHeaderModelTextField('reasoning_effort', nextDetail)
  const serviceTier = resolveHeaderModelTextField('service_tier', nextDetail)
  const actualServiceTier = resolveHeaderModelTextField('actual_service_tier', nextDetail)
  const responseModel = resolveHeaderModelTextField('response_model', nextDetail)

  emit('requestState', {
    id,
    requestId: nextDetail.request_id || nextDetail.id || null,
    status: resolveRequestStateStatusFromDetail(nextDetail),
    statusCode: nextDetail.status_code ?? undefined,
    inputTokens: nextDetail.input_tokens ?? nextDetail.tokens?.input ?? null,
    effectiveInputTokens: displayInputTokens.value,
    outputTokens: nextDetail.output_tokens ?? nextDetail.tokens?.output ?? null,
    totalTokens: nextDetail.total_tokens ?? nextDetail.tokens?.total ?? null,
    cacheCreationInputTokens: nextDetail.cache_creation_input_tokens ?? null,
    cacheCreationEphemeral5mInputTokens: detailCacheCreation5mTokens(nextDetail),
    cacheCreationEphemeral1hInputTokens: detailCacheCreation1hTokens(nextDetail),
    cacheReadInputTokens: nextDetail.cache_read_input_tokens ?? null,
    cost: detailTotalCost(nextDetail),
    actualCost: nextDetail.actual_cost ?? null,
    billingMultiplier: nextDetail.billing_multiplier,
    billingCost: nextDetail.billing_cost,
    routingGroupId: nextDetail.routing_group_id ?? null,
    routingGroupName: nextDetail.routing_group_name ?? null,
    responseTimeMs: nextDetail.response_time_ms ?? undefined,
    firstByteTimeMs: nextDetail.first_byte_time_ms ?? null,
    isStream: nextDetail.is_stream ?? null,
    isWebSocket: nextDetail.is_websocket ?? null,
    websocketTransport: nextDetail.websocket_transport ?? null,
    usageAvailable: nextDetail.usage_available ?? null,
    usagePricingAvailable: nextDetail.usage_pricing_available ?? null,
    inputAudioTokens: nextDetail.input_audio_tokens ?? null,
    outputAudioTokens: nextDetail.output_audio_tokens ?? null,
    upstreamIsStream: nextDetail.upstream_is_stream ?? null,
    clientRequestedStream: nextDetail.client_requested_stream ?? null,
    clientIsStream: nextDetail.client_is_stream ?? null,
    apiFormat: nextDetail.api_format ?? null,
    endpointApiFormat: nextDetail.endpoint_api_format ?? null,
    hasFormatConversion: nextDetail.has_format_conversion ?? null,
    chatPiiRedaction: nextDetail.chat_pii_redaction ?? null,
    ...(targetModel ? { targetModel } : {}),
    ...(requestedReasoningEffort ? { requestedReasoningEffort } : {}),
    ...(reasoningEffort ? { reasoningEffort } : {}),
    ...(serviceTier ? { serviceTier } : {}),
    ...(actualServiceTier ? { actualServiceTier } : {}),
    ...(responseModel ? { responseModel } : {}),
    errorMessage: nextDetail.error_message ?? undefined,
    updatedAt: nextDetail.updated_at ?? undefined,
  })
}

function handleTraceState(state: {
  loaded: boolean
  hasTrace: boolean
  finalStatus?: RequestTrace['final_status'] | null
  statusCode?: number | null
  latencyMs?: number | null
  imageProgress?: ImageProgress | null
  errorMessage?: string | null
}) {
  timelineLoaded.value = state.loaded
  timelineHasTrace.value = state.hasTrace
  const id = props.requestId
  if (!id) return

  const status = resolveRequestStateStatus(
    mapTraceFinalStatusToRequestStatus(state.finalStatus),
    state.statusCode,
    state.errorMessage
  )
  const imageFailed = state.imageProgress?.phase === 'failed'
  if (!status && !state.imageProgress && state.statusCode == null && state.latencyMs == null) return

  emit('requestState', {
    id,
    requestId: detail.value?.request_id || detail.value?.id || null,
    status: imageFailed ? 'failed' : status,
    statusCode: state.statusCode ?? undefined,
    responseTimeMs: state.latencyMs ?? undefined,
    imageProgress: state.imageProgress ?? null,
    errorMessage: state.errorMessage ?? undefined,
  })
}

const autoRefreshTimer = ref<ReturnType<typeof setInterval> | null>(null)
const autoRefreshing = ref(false)
const isPageVisible = ref(typeof document === 'undefined' ? true : !document.hidden)
const curlCopying = ref(false)
const curlCopied = ref(false)
const replayDialogOpen = ref(false)
const bodyLoading = ref(false)
const bodyLoadError = ref<string | null>(null)
const bodyDocuments = shallowRef(new Map<RequestBodyField, BodyDocument>())
const bodyCopying = ref(false)
const bodyLoadFailures = new Map<RequestBodyField, string>()
let bodyLoadController: AbortController | null = null
const showTimeline = ref(false)
const AUTO_REFRESH_INTERVAL_MS = 1000
const TIMELINE_MOUNT_DELAY_MS = 120
let loadDetailRequestId = 0
let bodyLoadRequestId = 0
let loadDetailInFlight = false
let timelineMountTimer: number | null = null

const fullRequestId = computed(() => detail.value?.request_id || detail.value?.id || '-')
const displayRequestId = computed(() => formatShortRequestId(fullRequestId.value))
const refreshButtonTitle = computed(() => {
  if (autoRefreshing.value) return t('ui.stopAutomaticRefresh')
  return isRequestCompleted() ? t('ui.refresh') : t('ui.turnOnAutoRefresh')
})
const displayInputTokens = computed(() => {
  if (!detail.value) return 0
  return getEffectiveInputTokens({
    effective_input_tokens: detail.value.effective_input_tokens,
    input_tokens: detail.value.input_tokens ?? detail.value.tokens?.input,
    cache_read_input_tokens: detail.value.cache_read_input_tokens,
    api_format: detail.value.api_format,
    endpoint_api_format: detail.value.endpoint_api_format,
  })
})

const detailOutputTokens = computed(() => {
  if (!detail.value) return 0
  return detail.value.tokens?.output ?? detail.value.output_tokens ?? 0
})

const detailOutputRate = computed(() => {
  if (!detail.value) return null
  return getDisplayOutputRate({
    output_tokens: detailOutputTokens.value,
    response_time_ms: detail.value.response_time_ms,
    first_byte_time_ms: detail.value.first_byte_time_ms,
    is_stream: detail.value.is_stream,
    upstream_is_stream: detail.value.upstream_is_stream,
  })
})

// 监听标签页切换
watch(activeTab, (newTab) => {
  if (!['request-headers', 'response-headers'].includes(newTab) && viewMode.value === 'compare') {
    viewMode.value = 'formatted'
  }
  // 切换到不支持对话视图的 Tab 时，重置为 JSON 视图
  if (!['request-body', 'response-body'].includes(newTab)) {
    contentViewMode.value = 'json'
  }
  dataSource.value = getDefaultDataSourceForTab(newTab)
})

const { isDark } = useDarkMode()

function streamBadgeVariant(isStream: boolean): 'secondary' | 'outline' {
  if (isDark.value) {
    return isStream ? 'outline' : 'secondary'
  }
  return isStream ? 'secondary' : 'outline'
}

const traceRequestMetadata = computed<Record<string, unknown> | null>(() => {
  const meta = detail.value?.metadata
  if (!meta || typeof meta !== 'object' || Array.isArray(meta)) return null
  return meta as Record<string, unknown>
})

const traceRecord = computed<Record<string, unknown> | null>(() =>
  asRecord(detail.value?.trace ?? null),
)

const traceTimelineRequestId = computed(() =>
  getNestedString(traceRecord.value, 'trace_id')
  ?? getNestedString(traceRequestMetadata.value, 'trace_id')
  ?? getCaseInsensitiveString(asRecord(detail.value?.request_headers ?? null), 'x-trace-id')
  ?? detail.value?.request_id
  ?? detail.value?.id
  ?? null,
)

const metadataPanelData = computed<Record<string, unknown> | null>(() => {
  if (!detail.value) return null

  const merged: Record<string, unknown> = {}
  if (hasContent(detail.value.metadata)) {
    Object.assign(merged, detail.value.metadata || {})
  }
  if (hasContent(detail.value.routing)) {
    merged.routing = detail.value.routing
  }
  if (hasContent(detail.value.body_capture)) {
    merged.body_capture = detail.value.body_capture
  }
  if (hasContent(detail.value.trace)) {
    merged.trace = detail.value.trace
  }
  if (hasContent(detail.value.settlement)) {
    merged.settlement = detail.value.settlement
  }

  return Object.keys(merged).length > 0
    ? formatMetadataDisplayValue(merged) as Record<string, unknown>
    : null
})

const detailForCurrentRequest = computed(() => (
  detailMatchesRequestId(detail.value, props.requestId) ? detail.value : null
))

type AuthoritativeErrorSource = 'summary' | 'detail' | null

function isTerminalRequestState(status: RequestStateStatus | undefined): boolean {
  return status === 'completed' || status === 'failed' || status === 'cancelled'
}

function isSuccessfulTerminalRequestState(status: RequestStateStatus | undefined): boolean {
  return status === 'completed' || status === 'cancelled'
}

const authoritativeErrorSource = computed<AuthoritativeErrorSource>(() => {
  const summary = props.summaryRecord
  const currentDetail = detailForCurrentRequest.value
  if (!summary || !currentDetail) return null

  const summaryStatus = resolveRequestStateStatus(
    summary.status,
    summary.status_code,
    summary.error_message,
  )
  const detailStatus = resolveRequestStateStatusFromDetail(currentDetail)
  const summaryUpdatedAtMs = usageSnapshotUpdatedAtMs(summary)
  const detailUpdatedAtMs = usageSnapshotUpdatedAtMs(currentDetail)

  if (summaryUpdatedAtMs != null && detailUpdatedAtMs != null &&
    summaryUpdatedAtMs !== detailUpdatedAtMs) {
    if (detailUpdatedAtMs > summaryUpdatedAtMs && isTerminalRequestState(detailStatus)) {
      return 'detail'
    }
    if (summaryUpdatedAtMs > detailUpdatedAtMs && isTerminalRequestState(summaryStatus)) {
      return 'summary'
    }
  }

  // Without a comparable timestamp, a successful/cancelled terminal snapshot
  // still has to clear a failure from the other source. Generic failed detail
  // remains non-authoritative so opening the drawer cannot flash away a Cyber
  // refusal already resolved by the list.
  const detailSucceeded = isSuccessfulTerminalRequestState(detailStatus)
  const summarySucceeded = isSuccessfulTerminalRequestState(summaryStatus)
  if (detailSucceeded && !summarySucceeded) return 'detail'
  if (summarySucceeded && !detailSucceeded) return 'summary'
  if (detailSucceeded && summarySucceeded) {
    return detailModelRevision.value >= summaryModelRevision.value ? 'detail' : 'summary'
  }

  return null
})

const headerModelRecord = computed(() => {
  const summary = props.summaryRecord
  const currentDetail = detailForCurrentRequest.value
  if (!summary && !currentDetail) return null

  const authoritativeSource = authoritativeErrorSource.value
  const errorMessage = authoritativeSource === 'summary'
    ? mergeUsageRecordErrorMessage(undefined, summary?.error_message, { authoritative: true })
    : mergeUsageRecordErrorMessage(
      summary?.error_message,
      currentDetail?.error_message,
      { authoritative: authoritativeSource === 'detail' },
    )

  return {
    model: resolveHeaderModelTextField('model', currentDetail) ?? '-',
    target_model: resolveHeaderModelTextField('target_model', currentDetail),
    response_model: resolveHeaderModelTextField('response_model', currentDetail),
    request_type: resolveHeaderModelTextField('request_type', currentDetail),
    requested_reasoning_effort: resolveHeaderModelTextField(
      'requested_reasoning_effort',
      currentDetail,
    ),
    reasoning_effort: resolveHeaderModelTextField('reasoning_effort', currentDetail),
    service_tier: resolveHeaderModelTextField('service_tier', currentDetail),
    error_message: errorMessage,
  }
})
const detailCyberPolicyError = computed(() => {
  const summaryError = props.summaryRecord?.error_message
  const currentDetail = detailForCurrentRequest.value
  const detailErrors = [
    currentDetail?.error_message,
    currentDetail?.upstream_error,
    currentDetail?.failure_summary,
    currentDetail?.response_body,
  ]

  if (authoritativeErrorSource.value === 'summary') {
    return isCyberPolicyError(summaryError)
  }
  if (authoritativeErrorSource.value === 'detail') {
    return isCyberPolicyError(detailErrors)
  }
  return isCyberPolicyError([summaryError, ...detailErrors])
})

function hasBodyContent(flag: boolean | undefined, data: unknown): boolean {
  return Boolean(flag) || hasContent(data)
}

const hasRequestBodyAvailable = computed(() => {
  return hasBodyContent(detail.value?.has_request_body, detail.value?.request_body)
    || hasBodyContent(detail.value?.has_provider_request_body, detail.value?.provider_request_body)
})

const hasResponseBodyAvailable = computed(() => {
  return hasBodyContent(detail.value?.has_response_body, detail.value?.response_body)
    || hasBodyContent(detail.value?.has_client_response_body, detail.value?.client_response_body)
})

const requestBodyField = computed<RequestBodyField>(() => {
  if (dataSource.value === 'provider' && hasBodyContent(detail.value?.has_provider_request_body, detail.value?.provider_request_body)) {
    return 'provider_request_body'
  }
  return hasBodyContent(detail.value?.has_request_body, detail.value?.request_body)
    ? 'request_body' : 'provider_request_body'
})

const responseBodyField = computed<RequestBodyField>(() => {
  if (dataSource.value === 'client' && hasBodyContent(detail.value?.has_client_response_body, detail.value?.client_response_body)) {
    return 'client_response_body'
  }
  return hasBodyContent(detail.value?.has_response_body, detail.value?.response_body)
    ? 'response_body' : 'client_response_body'
})

const activeBodyField = computed(() => {
  if (activeTab.value === 'request-body') return requestBodyField.value
  if (activeTab.value === 'response-body') return responseBodyField.value
  return null
})

watch([activeBodyField, () => props.isOpen, () => detail.value?.id], () => {
  cancelBodyContentLoad()
  void ensureBodyContentLoaded()
})

const isRequestBodyLoading = computed(() => {
  return bodyLoading.value && activeTab.value === 'request-body' && !currentRequestBody.value
})

const isResponseBodyLoading = computed(() => {
  return bodyLoading.value && activeTab.value === 'response-body' && !currentResponseBody.value
})

const requestBodyLoadFailed = computed(() => {
  const errors = detail.value?.body_load_errors
  return Boolean(errors?.[requestBodyField.value])
})

const responseBodyLoadFailed = computed(() => {
  const errors = detail.value?.body_load_errors
  return Boolean(errors?.[responseBodyField.value])
})

const bodyLoadErrorMessage = computed(() => {
  return bodyLoadError.value || formatStoredBodyLoadError(
    activeBodyField.value ? detail.value?.body_load_error_codes?.[activeBodyField.value] : undefined,
  )
})

const canRetryBodyContentLoad = computed(() => {
  const code = activeBodyField.value ? detail.value?.body_load_error_codes?.[activeBodyField.value] : undefined
  return code !== 'too_large' && code !== 'decode_failed'
})

const requestBodyLoadErrorVisible = computed(() => {
  return Boolean(
    (bodyLoadError.value || requestBodyLoadFailed.value) &&
    activeTab.value === 'request-body' &&
    !currentRequestBody.value
  )
})

const responseBodyLoadErrorVisible = computed(() => {
  return Boolean(
    (bodyLoadError.value || responseBodyLoadFailed.value) &&
    activeTab.value === 'response-body' &&
    !currentResponseBody.value
  )
})

function clearTimelineMountTimer() {
  if (timelineMountTimer) {
    clearTimeout(timelineMountTimer)
    timelineMountTimer = null
  }
}

function scheduleTimelineMount() {
  clearTimelineMountTimer()
  if (typeof window === 'undefined') {
    showTimeline.value = true
    return
  }
  timelineMountTimer = window.setTimeout(() => {
    showTimeline.value = true
  }, TIMELINE_MOUNT_DELAY_MS)
}

// 检测是否有提供商请求头
const hasProviderHeaders = computed(() => {
  return !!(detail.value?.provider_request_headers &&
         Object.keys(detail.value.provider_request_headers).length > 0)
})

// 请求体：仅当 provider_request_body 存在时才展示来源切换
const hasProviderRequestBody = computed(() => {
  return hasBodyContent(detail.value?.has_provider_request_body, detail.value?.provider_request_body)
})

// 响应体：只有客户端侧和 provider 侧都存在时才展示来源切换
const hasProviderResponseBody = computed(() => {
  return hasBodyContent(detail.value?.has_response_body, detail.value?.response_body)
    && hasBodyContent(detail.value?.has_client_response_body, detail.value?.client_response_body)
})

// 检测是否有两套响应头（客户端侧 + 提供商侧）
const hasProviderResponseHeaders = computed(() => {
  return !!(detail.value?.response_headers &&
         Object.keys(detail.value.response_headers).length > 0) &&
         !!(detail.value?.client_response_headers &&
         Object.keys(detail.value.client_response_headers).length > 0)
})

// 当前 Tab 是否支持对比模式
const canCompare = computed(() => {
  if (activeTab.value === 'request-headers') return hasProviderHeaders.value
  if (activeTab.value === 'response-headers') return hasProviderResponseHeaders.value
  return false
})

// 响应头 diff 统计（用于对比模式标题栏）
const responseHeaderStats = computed(() => {
  const counts = { added: 0, modified: 0, removed: 0, unchanged: 0 }
  if (!detail.value?.client_response_headers || !detail.value?.response_headers) return counts

  const clientHeaders = detail.value.client_response_headers as Record<string, unknown>
  const providerHeaders = detail.value.response_headers as Record<string, unknown>
  const clientKeys = new Set(Object.keys(clientHeaders))
  const providerKeys = new Set(Object.keys(providerHeaders))
  const allKeys = new Set([...clientKeys, ...providerKeys])

  for (const key of allKeys) {
    if (clientKeys.has(key) && providerKeys.has(key)) {
      counts[clientHeaders[key] === providerHeaders[key] ? 'unchanged' : 'modified']++
    } else if (clientKeys.has(key)) {
      counts.removed++
    } else {
      counts.added++
    }
  }
  return counts
})

// 是否显示客户端/提供商切换按钮
const showDataSourceToggle = computed(() => {
  if (activeTab.value === 'request-headers') return hasProviderHeaders.value
  if (activeTab.value === 'response-headers') return hasProviderResponseHeaders.value
  if (activeTab.value === 'request-body') return hasProviderRequestBody.value
  if (activeTab.value === 'response-body') return hasProviderResponseBody.value
  return false
})

// 当前高亮的数据源（对比模式下两个都不高亮）
const activeDataSource = computed(() => {
  if (['request-headers', 'response-headers'].includes(activeTab.value) && viewMode.value === 'compare') return null
  return dataSource.value
})

// 设置数据源（对比模式下切换数据源需退出对比）
function setDataSource(source: 'client' | 'provider') {
  dataSource.value = source
  if (['request-headers', 'response-headers'].includes(activeTab.value) && viewMode.value === 'compare') {
    viewMode.value = 'formatted'
  }
}

// 获取当前数据源的请求体数据
const currentRequestBody = computed(() => {
  return bodyDocuments.value.get(requestBodyField.value) ?? null
})

// 获取当前数据源的响应体数据
const currentResponseBody = computed(() => {
  return bodyDocuments.value.get(responseBodyField.value) ?? null
})

const currentRequestBodyApiFormat = computed(() => {
  if (!detail.value) return undefined
  if (dataSource.value === 'provider') {
    return detail.value.endpoint_api_format || detail.value.api_format
  }
  return detail.value.api_format || detail.value.endpoint_api_format
})

const currentResponseBodyApiFormat = computed(() => {
  if (!detail.value) return undefined
  if (dataSource.value === 'provider') {
    return detail.value.endpoint_api_format || detail.value.api_format
  }
  return detail.value.api_format || detail.value.endpoint_api_format
})

// 获取当前数据源的请求头数据
const currentHeaderData = computed(() => {
  if (!detail.value) return null
  if (dataSource.value === 'client' && hasContent(detail.value.request_headers)) {
    return detail.value.request_headers
  }
  if (dataSource.value === 'provider' && hasContent(detail.value.provider_request_headers)) {
    return detail.value.provider_request_headers
  }
  // 回退：优先 client，再 provider
  if (hasContent(detail.value.request_headers)) {
    return detail.value.request_headers
  }
  return detail.value.provider_request_headers
})

const activeJsonPanelData = computed(() => {
  switch (activeTab.value) {
    case 'request-headers':
      return currentHeaderData.value
    case 'request-body':
      return null
    case 'response-headers':
      return currentResponseHeaderData.value
    case 'response-body':
      return null
    case 'metadata':
      return metadataPanelData.value
    default:
      return null
  }
})

const activeJsonPanelTitle = computed(() => {
  if (viewMode.value === 'compare') return t('ui.compare')
  if (supportsConversationView.value && contentViewMode.value === 'conversation') return 'Chat'
  return 'JSON'
})

// 当前 Tab 是否支持对话视图
const supportsConversationView = computed(() => {
  return ['request-body', 'response-body'].includes(activeTab.value)
})

// 当前对话数据是否有效（用于禁用按钮）
// 不依赖带 tab/mode 守卫的 renderResult，直接检查 body 数据是否存在
const hasValidConversation = computed(() => {
  if (activeTab.value === 'request-body') {
    return !!currentRequestBody.value
  }
  if (activeTab.value === 'response-body') {
    return !!currentResponseBody.value
  }
  return false
})

const tabs = [
  { name: 'request-headers', get label() { return t('ui.requestHeaders') } },
  { name: 'request-body', get label() { return t('ui.requestBody') } },
  { name: 'response-headers', get label() { return t('ui.responseHeaders') } },
  { name: 'response-body', get label() { return t('ui.responseBody') } },
  { name: 'metadata', get label() { return t('ui.metadata') } },
]

function getDefaultDataSourceForTab(tab: string): 'client' | 'provider' {
  if (!detail.value) {
    if (['request-headers', 'request-body'].includes(tab)) return 'provider'
    if (['response-headers', 'response-body'].includes(tab)) return 'client'
    return dataSource.value
  }

  if (tab === 'request-headers') {
    if (hasContent(detail.value.provider_request_headers)) return 'provider'
    if (hasContent(detail.value.request_headers)) return 'client'
    return 'provider'
  }

  if (tab === 'request-body') {
    if (hasBodyContent(detail.value.has_provider_request_body, detail.value.provider_request_body)) return 'provider'
    if (hasBodyContent(detail.value.has_request_body, detail.value.request_body)) return 'client'
    return 'provider'
  }

  if (tab === 'response-headers') {
    if (hasContent(detail.value.client_response_headers)) return 'client'
    if (hasContent(detail.value.response_headers)) return 'provider'
    return 'client'
  }

  if (tab === 'response-body') {
    if (hasBodyContent(detail.value.has_client_response_body, detail.value.client_response_body)) return 'client'
    if (hasBodyContent(detail.value.has_response_body, detail.value.response_body)) return 'provider'
    return 'client'
  }

  return dataSource.value
}

// 获取当前数据源的响应头数据
const currentResponseHeaderData = computed(() => {
  if (!detail.value) return null
  if (dataSource.value === 'client' && hasContent(detail.value.client_response_headers)) {
    return detail.value.client_response_headers
  }
  if (dataSource.value === 'provider' && hasContent(detail.value.response_headers)) {
    return detail.value.response_headers
  }
  // 回退：优先 client，再 provider
  if (hasContent(detail.value.client_response_headers)) {
    return detail.value.client_response_headers
  }
  return detail.value.response_headers
})

// 根据实际数据决定显示哪些 Tab
const visibleTabs = computed(() => {
  if (!detail.value) return []

  return tabs.filter(tab => {
    switch (tab.name) {
      case 'request-headers':
        return hasContent(detail.value?.request_headers) || hasContent(detail.value?.provider_request_headers)
      case 'request-body':
        return hasRequestBodyAvailable.value
      case 'response-headers':
        return hasContent(detail.value?.response_headers) || hasContent(detail.value?.client_response_headers)
      case 'response-body':
        return hasResponseBodyAvailable.value
      case 'metadata':
        return hasContent(metadataPanelData.value)
      default:
        return false
    }
  })
})

watch([() => props.isOpen, () => props.requestId], async ([isOpen, requestId]) => {
  if (isOpen && requestId) {
    if (!detailMatchesRequestId(detail.value, requestId)) {
      detail.value = null
    }
    await loadDetail(requestId)
  } else if (!isOpen) {
    stopAutoRefresh()
    showTimeline.value = false
    clearTimelineMountTimer()
    resetBodyContentState()
    detail.value = null
    activeTab.value = 'request-headers'
    ++loadDetailRequestId
    loadDetailInFlight = false
  }
})

function detailMatchesRequestId(
  candidate: RequestDetail | null | undefined,
  requestId: string | null | undefined,
): boolean {
  if (!candidate || !requestId) return false
  return candidate.id === requestId || candidate.request_id === requestId
}

async function ensureBodyContentLoaded() {
  const field = activeBodyField.value
  const usageId = props.requestId
  if (!props.isOpen || !usageId || !field || !detailMatchesRequestId(detail.value, usageId)) return
  if (!detail.value || bodyLoading.value) return
  const cached = bodyDocuments.value.get(field)
  if (cached) {
    const documents = new Map(bodyDocuments.value)
    documents.delete(field)
    documents.set(field, cached)
    bodyDocuments.value = documents
    return
  }
  if (!hasBodyContent(detail.value[`has_${field}`], detail.value[field])) return
  const previousFailure = bodyLoadFailures.get(field)
  if (previousFailure) {
    bodyLoadError.value = previousFailure
    return
  }

  const requestId = ++bodyLoadRequestId
  const controller = new AbortController()
  bodyLoadController = controller
  bodyLoading.value = true
  bodyLoadError.value = null
  try {
    const response = await dashboardApi.getRequestBody(detail.value.id, field, controller.signal)
    const document = await BodyDocument.load(response.bytes, response.encoding, controller.signal)
    if (requestId !== bodyLoadRequestId || controller.signal.aborted || !detail.value ||
      !props.isOpen || !detailMatchesRequestId(detail.value, usageId)) {
      document.dispose()
      return
    }
    const documents = new Map(bodyDocuments.value)
    let cachedBytes = document.byteLength
    for (const existing of documents.values()) cachedBytes += existing.byteLength
    for (const [cachedField, existing] of documents) {
      if (documents.size < 2 && cachedBytes <= MAX_BODY_BYTES) break
      existing.dispose()
      documents.delete(cachedField)
      cachedBytes -= existing.byteLength
    }
    documents.set(field, document)
    bodyDocuments.value = documents
    detail.value = { ...detail.value, body_load_errors: { ...detail.value.body_load_errors, [field]: false }, body_load_error_codes: { ...detail.value.body_load_error_codes, [field]: undefined } }
  } catch (err) {
    if (requestId !== bodyLoadRequestId || controller.signal.aborted || axios.isCancel(err)) return
    log.error('Failed to load request bodies:', err)
    bodyLoadError.value = formatRequestBodyLoadError(err)
    bodyLoadFailures.set(field, bodyLoadError.value)
    const code = err instanceof BodyDocumentError ? err.code : axios.isAxiosError(err) ? err.response?.headers?.['x-aether-body-error'] : undefined
    if ((code === 'too_large' || code === 'decode_failed') && detail.value) {
      detail.value = { ...detail.value, body_load_error_codes: { ...detail.value.body_load_error_codes, [field]: code } }
    }
  } finally {
    if (requestId === bodyLoadRequestId) {
      bodyLoading.value = false
      bodyLoadController = null
    }
  }
}

function handleBodyDocumentError(error: unknown) {
  const field = activeBodyField.value
  if (!field) return
  const documents = new Map(bodyDocuments.value)
  documents.get(field)?.dispose()
  documents.delete(field)
  bodyDocuments.value = documents
  bodyLoadError.value = formatRequestBodyLoadError(error)
  bodyLoadFailures.set(field, bodyLoadError.value)
}

function retryBodyContentLoad() {
  if (activeBodyField.value) bodyLoadFailures.delete(activeBodyField.value)
  bodyLoadError.value = null
  void ensureBodyContentLoaded()
}

function cancelBodyContentLoad() {
  ++bodyLoadRequestId
  bodyLoadController?.abort()
  bodyLoadController = null
  bodyLoading.value = false
  bodyLoadError.value = null
}

function resetBodyContentState() {
  cancelBodyContentLoad()
  for (const document of bodyDocuments.value.values()) document.dispose()
  bodyDocuments.value = new Map()
  bodyLoadFailures.clear()
}

async function loadDetail(id: string, silent = false) {
  if (silent && loadDetailInFlight) {
    return
  }
  const requestId = ++loadDetailRequestId
  loadDetailInFlight = true
  if (!silent) {
    if (!detailMatchesRequestId(detail.value, id)) {
      detail.value = null
    }
    loading.value = true
    historicalPricing.value = null
    timelineLoaded.value = false
    timelineHasTrace.value = false
    showTimeline.value = false
    clearTimelineMountTimer()
    resetBodyContentState()
  }
  error.value = null
  try {
    const response = await dashboardApi.getRequestDetail(id, {
      includeBodies: false,
      cacheTtlMs: silent ? 0 : 5_000
    })
    if (requestId !== loadDetailRequestId) return

    const previousDetail = detail.value
    const prevKey = previousDetail?.request_id || previousDetail?.id
    const currKey = response.request_id || response.id
    const sameRequest = !!prevKey && prevKey === currKey
    const bodySnapshotChanged = sameRequest && previousDetail != null &&
      ['pending', 'streaming'].includes(previousDetail.status ?? '') && previousDetail.status !== response.status
    const preserveBodies = sameRequest && !bodySnapshotChanged
    const nextDetail: RequestDetail = {
      ...response,
      status: resolveRequestStateStatusFromDetail(response) ?? response.status,
      request_body: preserveBodies ? previousDetail?.request_body : undefined,
      provider_request_body: preserveBodies ? previousDetail?.provider_request_body : undefined,
      response_body: preserveBodies ? previousDetail?.response_body : undefined,
      client_response_body: preserveBodies ? previousDetail?.client_response_body : undefined,
      body_load_errors: preserveBodies ? previousDetail?.body_load_errors : response.body_load_errors,
      body_load_error_codes: preserveBodies ? previousDetail?.body_load_error_codes : response.body_load_error_codes,
      request_error: response.request_error,
      upstream_error: response.upstream_error,
      client_error: response.client_error,
      failure_summary: response.failure_summary,
      errors: response.errors,
      error_flow: response.error_flow,
      scheduling_failure: response.scheduling_failure,
    }
    detailModelRevision.value = ++modelSnapshotRevision
    detail.value = nextDetail
    if (!sameRequest || bodySnapshotChanged) resetBodyContentState()
    emitDetailRequestState(nextDetail)

    // 首次加载时优先停留在轻量 tab，避免默认触发大 body 加载
    if (!silent) {
      const visibleTabNames = visibleTabs.value.map(t => t.name)
      if (visibleTabNames.includes('request-headers')) {
        activeTab.value = 'request-headers'
      } else if (visibleTabNames.includes('response-headers')) {
        activeTab.value = 'response-headers'
      } else if (visibleTabNames.includes('metadata')) {
        activeTab.value = 'metadata'
      } else if (visibleTabNames.includes('request-body')) {
        activeTab.value = 'request-body'
      } else if (visibleTabNames.includes('response-body')) {
        activeTab.value = 'response-body'
      } else if (visibleTabNames.length > 0) {
        activeTab.value = visibleTabNames[0]
      }
      scheduleTimelineMount()
    }

    // 根据当前 Tab 的数据可用性自动选择默认数据源
    if (!silent || !sameRequest) dataSource.value = getDefaultDataSourceForTab(activeTab.value)
    if (bodySnapshotChanged) void nextTick().then(ensureBodyContentLoaded)

    // 使用请求记录中保存的历史价格
    if (detail.value.input_price_per_1m || detail.value.output_price_per_1m || detail.value.price_per_request) {
      historicalPricing.value = {
        input_price: detail.value.input_price_per_1m ? detail.value.input_price_per_1m.toFixed(4) : 'N/A',
        output_price: detail.value.output_price_per_1m ? detail.value.output_price_per_1m.toFixed(4) : 'N/A',
        cache_creation_price: detail.value.cache_creation_price_per_1m ? detail.value.cache_creation_price_per_1m.toFixed(4) : 'N/A',
        cache_read_price: detail.value.cache_read_price_per_1m ? detail.value.cache_read_price_per_1m.toFixed(4) : 'N/A',
        request_price: detail.value.price_per_request ? detail.value.price_per_request.toFixed(4) : 'N/A'
      }
    }

    // 静默刷新时同步刷新链路追踪
    if (silent) {
      timelineRef.value?.refresh()
    }

    if (props.isOpen) {
      if (isRequestCompleted()) {
        stopAutoRefresh()
      } else if (!silent) {
        startAutoRefresh()
      }
    }
  } catch (err) {
    if (requestId !== loadDetailRequestId) return
    log.error('Failed to load request detail:', err)
    if (!silent) {
      error.value = t('ui.failedToLoadRequestDetails')
    }
  } finally {
    if (!silent && requestId === loadDetailRequestId) {
      loading.value = false
    }
    if (requestId === loadDetailRequestId) {
      loadDetailInFlight = false
    }
  }
}

function handleClose() {
  stopAutoRefresh()
  emit('close')
}

function isRequestCompleted(): boolean {
  if (!detail.value?.status) return true
  const displayStatus = resolveDisplayRequestStatus(detail.value)
  return displayStatus !== 'pending' && displayStatus !== 'streaming'
}

function stopAutoRefresh() {
  if (autoRefreshTimer.value) {
    clearInterval(autoRefreshTimer.value)
    autoRefreshTimer.value = null
  }
  autoRefreshing.value = false
}

function startAutoRefresh() {
  if (autoRefreshTimer.value) {
    autoRefreshing.value = true
    return
  }
  if (!isPageVisible.value || !props.requestId || !props.isOpen) {
    autoRefreshing.value = false
    return
  }
  autoRefreshing.value = true
  autoRefreshTimer.value = setInterval(async () => {
    if (!isPageVisible.value || !props.requestId || !props.isOpen) {
      stopAutoRefresh()
      return
    }
    await loadDetail(props.requestId, true)
    if (isRequestCompleted()) {
      stopAutoRefresh()
    }
  }, AUTO_REFRESH_INTERVAL_MS)
}

async function refreshDetail() {
  if (!props.requestId) return

  // 已完成：单次静默刷新
  if (isRequestCompleted()) {
    await loadDetail(props.requestId, true)
    return
  }

  // 未完成：如果已在自动刷新则停止，否则启动
  if (autoRefreshing.value) {
    stopAutoRefresh()
    return
  }

  autoRefreshing.value = true
  await loadDetail(props.requestId, true)

  // 加载后可能已经完成了
  if (isRequestCompleted()) {
    stopAutoRefresh()
    return
  }

  startAutoRefresh()
}

function handleVisibilityChange() {
  isPageVisible.value = !document.hidden
  if (!isPageVisible.value) {
    stopAutoRefresh()
  }
}

onMounted(() => {
  document.addEventListener('visibilitychange', handleVisibilityChange)
})

onBeforeUnmount(() => {
  document.removeEventListener('visibilitychange', handleVisibilityChange)
  stopAutoRefresh()
  clearTimelineMountTimer()
  loadDetailRequestId += 1
  loadDetailInFlight = false
  resetBodyContentState()
})

async function copyContent(tabName: string) {
  if (!detail.value || viewMode.value === 'compare' || bodyCopying.value) return
  const usageId = detail.value.id
  const document = tabName === 'request-body' ? currentRequestBody.value
    : tabName === 'response-body' ? currentResponseBody.value : null
  bodyCopying.value = true
  try {
    let text: string
    if (document) {
      text = await document.copy(contentViewMode.value === 'conversation' ? {
        kind: tabName === 'request-body' ? 'request' : 'response',
        apiFormat: tabName === 'request-body' ? currentRequestBodyApiFormat.value : currentResponseBodyApiFormat.value,
      } : undefined)
    } else {
      const data = tabName === 'request-headers' ? (dataSource.value === 'provider' ? detail.value.provider_request_headers : detail.value.request_headers)
        : tabName === 'response-headers' ? currentResponseHeaderData.value
          : tabName === 'metadata' ? metadataPanelData.value : null
      if (data == null) return
      text = JSON.stringify(data, null, 2)
    }
    if (!props.isOpen || detail.value?.id !== usageId) return
    if (await copyToClipboard(text, false)) {
      copiedStates.value[tabName] = true
      setTimeout(() => { copiedStates.value[tabName] = false }, 2000)
    } else {
      showBodyError('复制失败，请检查剪贴板权限后重试。')
    }
  } catch (error) {
    if (props.isOpen && detail.value?.id === usageId) showBodyError(formatRequestBodyLoadError(error))
  } finally {
    bodyCopying.value = false
  }
}

// 切换内容视图模式
function toggleContentView() {
  if (contentViewMode.value === 'json') {
    if (hasValidConversation.value) {
      contentViewMode.value = 'conversation'
    }
  } else {
    contentViewMode.value = 'json'
  }
}

// 复制 cURL 命令
async function copyCurlCommand() {
  if (!props.requestId || curlCopying.value) return
  curlCopying.value = true
  try {
    const data = await dashboardApi.getCurlData(props.requestId)
    if (data.curl) {
      copyToClipboard(data.curl, false)
      curlCopied.value = true
      setTimeout(() => { curlCopied.value = false }, 2000)
    }
  } catch (err) {
    log.error('Failed to generate cURL command:', err)
  } finally {
    curlCopying.value = false
  }
}

// 打开请求回放对话框
function openReplayDialog() {
  replayDialogOpen.value = true
}

// 请求头合并对比逻辑
interface HeaderEntry {
  key: string
  status: 'added' | 'modified' | 'removed' | 'unchanged'
  originalValue?: unknown
  newValue?: unknown
}

const mergedHeaderEntries = computed(() => {
  if (!detail.value?.request_headers && !detail.value?.provider_request_headers) {
    return []
  }

  const clientHeaders = detail.value?.request_headers || {}
  const providerHeaders = detail.value?.provider_request_headers || {}

  const clientKeys = new Set(Object.keys(clientHeaders))
  const providerKeys = new Set(Object.keys(providerHeaders))
  const allKeys = new Set([...clientKeys, ...providerKeys])

  const entries: HeaderEntry[] = []

  for (const key of Array.from(allKeys).sort()) {
    const entry: HeaderEntry = { key, status: 'unchanged' }

    if (clientKeys.has(key) && providerKeys.has(key)) {
      if (clientHeaders[key] !== providerHeaders[key]) {
        entry.status = 'modified'
        entry.originalValue = clientHeaders[key]
        entry.newValue = providerHeaders[key]
      } else {
        entry.status = 'unchanged'
        entry.originalValue = clientHeaders[key]
      }
    } else if (clientKeys.has(key)) {
      entry.status = 'removed'
      entry.originalValue = clientHeaders[key]
    } else {
      entry.status = 'added'
      entry.newValue = providerHeaders[key]
    }

    entries.push(entry)
  }

  return entries
})

const headerStats = computed(() => {
  const counts = {
    added: 0,
    modified: 0,
    removed: 0,
    unchanged: 0
  }

  for (const entry of mergedHeaderEntries.value) {
    counts[entry.status]++
  }

  return counts
})

</script>

<style scoped>
/* 内容区融合：子组件的 Card 不再需要自己的边框和圆角，与表头栏融为一体 */
.content-block :deep(.rounded-2xl) {
  border: none !important;
  border-radius: 0 !important;
  box-shadow: none !important;
}
</style>

<style>
/* 滚动条始终预留空间，保持宽度稳定 */
.scrollbar-stable {
  scrollbar-gutter: stable;
}

/* Webkit 浏览器滚动条样式 */
.scrollbar-stable::-webkit-scrollbar {
  width: 8px;
}

.scrollbar-stable::-webkit-scrollbar-track {
  background: transparent;
}

.scrollbar-stable::-webkit-scrollbar-thumb {
  background-color: rgba(128, 128, 128, 0.5);
  border-radius: 4px;
}

.scrollbar-stable::-webkit-scrollbar-thumb:hover {
  background-color: rgba(128, 128, 128, 0.7);
}
</style>
