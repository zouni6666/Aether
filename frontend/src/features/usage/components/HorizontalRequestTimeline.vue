<template>
  <div class="minimal-request-timeline">
    <!-- Loading State -->
    <div
      v-if="loading"
      class="py-4"
    >
      <Skeleton class="h-32 w-full" />
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

    <!-- Timeline Content -->
    <div
      v-else-if="trace && trace.candidates.length > 0"
      class="space-y-0"
    >
      <Card>
        <div class="p-6">
          <!-- 概览信息 -->
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-3">
              <h4 class="text-sm font-semibold">
                {{ t('ui.requestTrace') }}
              </h4>
              <Badge :variant="getFinalStatusBadgeVariant(computedFinalStatus)">
                {{ getFinalStatusLabel(computedFinalStatus) }}
              </Badge>
              <!-- 被跳过的候选可以收起，避免长时间线上排查时干扰视线 -->
              <button
                v-if="skippedTraceCandidates.length > 0"
                type="button"
                class="skipped-toggle"
                :class="{ active: showSkippedCandidates }"
                :aria-pressed="showSkippedCandidates"
                :title="showSkippedCandidates
                  ? t('ui.hideSkippedCandidates')
                  : t('ui.skippedCandidatesWereDeemedUnavailableDuringSchedulingNoRequestWas')"
                data-timeline-skipped-toggle
                @click="showSkippedCandidates = !showSkippedCandidates"
              >
                {{ showSkippedCandidates ? t('ui.hide') : t('ui.show') }}{{ t('ui.skippedCandidates') }}{{ skippedTraceCandidates.length }}）
              </button>
            </div>
            <div class="text-sm text-muted-foreground">
              {{ formatLatency(totalTraceLatency) }}
            </div>
          </div>

          <!-- 极简时间线轨道（按组显示） -->
          <div class="minimal-track">
            <div
              v-for="(group, groupIndex) in groupedTimeline"
              :key="group.id"
              class="minimal-node-group"
              :class="{
                selected: isGroupSelected(group),
                hovered: isGroupHovered(groupIndex) && !isGroupSelected(group)
              }"
              @mouseenter="hoveredGroupIndex = groupIndex"
              @mouseleave="hoveredGroupIndex = null"
              @click="selectGroup(group)"
            >
              <!-- 节点容器 -->
              <div class="node-container">
                <!-- 节点名称（在节点上方） -->
                <div class="node-label">
                  {{ group.providerName }}
                </div>

                <!-- 主节点（代表首次请求） -->
                <div
                  class="node-dot"
                  :class="[
                    getStatusColorClass(group.primaryStatus),
                    { 'is-first-selected': isGroupSelected(group) && selectedAttemptIndex === 0 }
                  ]"
                  @click.stop="selectFirstAttempt(group)"
                />

                <!-- 子节点（同提供商的其他尝试，不包含首次） -->
                <div
                  v-if="group.retryCount > 0"
                  class="sub-dots"
                >
                  <button
                    v-for="(attempt, idx) in group.allAttempts.slice(1)"
                    :key="attempt.id"
                    type="button"
                    class="sub-dot"
                    :class="[
                      getStatusColorClass(getDisplayStatus(attempt)),
                      { active: isAttemptSelected(group, idx + 1) }
                    ]"
                    :title="formatAttemptDotTitle(attempt)"
                    :aria-label="formatAttemptDotTitle(attempt)"
                    @click.stop="selectAttemptInGroup(group, idx + 1)"
                  />
                </div>
              </div>

              <!-- 连接线 -->
              <div
                v-if="groupIndex < groupedTimeline.length - 1"
                class="node-line-wrapper"
              >
                <div
                  class="node-line"
                  :class="{ 'conversion-boundary': groupIndex + 1 === conversionBoundaryIndex }"
                />
              </div>
            </div>
          </div>

          <!-- 选中详情面板 -->
          <Transition name="slide-up">
            <TraceAttemptDetailPanel
              v-if="selectedGroup && currentAttempt"
              :group="selectedGroup"
              :attempt="currentAttempt"
              :group-index="selectedGroupIndex"
              :group-count="groupedTimeline.length"
              :attempt-index="selectedAttemptIndex"
              :trace="trace"
              :final-status="computedFinalStatus"
              :override-status-code="overrideStatusCode"
              :request-api-format="requestApiFormat"
              :request-metadata="requestMetadata"
              :usage-data="usageData"
              :is-dark="isDark"
              @navigate-group="navigateGroup"
              @navigate-attempt="navigateAttempt"
            />
          </Transition>
        </div>
      </Card>
    </div>

    <!-- Empty State -->
    <Card
      v-else
      class="border-dashed"
    >
      <div class="p-8 text-center">
        <p class="text-sm text-muted-foreground">
          {{ t('ui.noTraceData') }}
        </p>
      </div>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { ref, watch, computed, onBeforeUnmount } from 'vue'
import { isAxiosError } from 'axios'
import Card from '@/components/ui/card.vue'
import Badge from '@/components/ui/badge.vue'
import Skeleton from '@/components/ui/skeleton.vue'
import TraceAttemptDetailPanel from './TraceAttemptDetailPanel.vue'
import { getProviderDisplayName } from '../utils/traceNodeGroup'
import type { TraceNodeGroup, TraceUsageData } from '../utils/traceNodeGroup'
import { requestTraceApi, type RequestTrace, type CandidateRecord, type ImageProgress } from '@/api/requestTrace'
import { log } from '@/utils/logger'
import { parseApiError } from '@/utils/errorParser'
import { useDarkMode } from '@/composables/useDarkMode'
import { resolveTimelineFinalStatus } from '../utils/status'
import {
  getFinalStatusLabel,
  getFinalStatusBadgeVariant,
  formatLatency,
  extractObject,
  normalizeImageProgress,
  normalizeLatencyMs,
  getStatusLabel,
  getStatusColorClass,
  getDisplayStatus,
} from '../utils/traceAttemptDisplay'
import {
  buildPoolGroupVisibleAttempts,
  buildPoolParticipatedCandidates,
  extractPoolGroupId,
  makeAttemptKey,
  TIMELINE_STATUS,
} from '../utils/poolTrace'

const props = defineProps<{
  requestId?: string | null
  /** 外部传入的状态码，用于覆盖 trace.final_status 的判断 */
  overrideStatusCode?: number
  /** 外部传入的请求状态，用于识别已失败/取消的终态请求 */
  requestStatus?: string | null
  /** 请求侧 API 格式（客户端入口格式） */
  requestApiFormat?: string | null
  /** 用量和费用数据 */
  usageData?: TraceUsageData | null
  /** 请求元数据（用于号池调度组装） */
  requestMetadata?: Record<string, unknown> | null
  /** 已获取的追踪数据；传入时不再内部拉取 */
  traceData?: RequestTrace | null
}>()

const emit = defineEmits<{
  selectAttempt: [attempt: CandidateRecord | null]
  traceState: [state: {
    loaded: boolean
    hasTrace: boolean
    finalStatus?: RequestTrace['final_status'] | null
    statusCode?: number | null
    latencyMs?: number | null
    imageProgress?: ImageProgress | null
    errorMessage?: string | null
  }]
}>()

const { t } = useI18n()

// 用量数据（从 props 获取）
const usageData = computed(() => props.usageData)

const loading = ref(false)
const error = ref<string | null>(null)
const internalTrace = ref<RequestTrace | null>(null)
const { isDark } = useDarkMode()
const trace = computed(() => props.traceData ?? internalTrace.value)
const selectedGroupIndex = ref(0)
const selectedAttemptIndex = ref(0)
const selectionPinnedByUser = ref(false)
const hoveredGroupIndex = ref<number | null>(null)
const traceLoadStarted = ref(false)
let tracePollTimer: ReturnType<typeof setTimeout> | null = null
let traceLoadInFlight: Promise<void> | null = null
const TRACE_POLL_INTERVAL_MS = 1000

const STATUS_PRIORITY: Record<string, number> = {
  available: 0,
  unused: 0,
  skipped: 1,
  failed: 2,
  cancelled: 2,
  stream_interrupted: 2,
  pending: 3,
  streaming: 3,
  success: 4,
}

const isParticipatedCandidate = (candidate: CandidateRecord): boolean => {
  return TIMELINE_STATUS.includes(candidate.status)
}

const isLiveCandidate = (candidate: CandidateRecord): boolean => {
  if (candidate.status === 'streaming') return true
  return candidate.status === 'pending' && Boolean(candidate.started_at)
}

// 计算最终状态：优先检查真正已启动的进行中状态，再使用外部状态码
const computedFinalStatus = computed(() => {
  const hasPending = trace.value?.candidates?.some(isLiveCandidate)
  return resolveTimelineFinalStatus({
    hasPendingCandidates: hasPending,
    statusCode: props.overrideStatusCode,
    requestStatus: props.requestStatus ?? usageData.value?.status,
    traceFinalStatus: trace.value?.final_status,
  })
})

const compareBySchedulingOrder = (a: CandidateRecord, b: CandidateRecord): number => {
  if (a.candidate_index !== b.candidate_index) {
    return a.candidate_index - b.candidate_index
  }
  if (a.retry_index !== b.retry_index) {
    return a.retry_index - b.retry_index
  }
  return new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
}

// 候选时间线（按调度顺序排序；lazy 加载的跳过候选通常没有 started_at）
const allTraceCandidates = computed<CandidateRecord[]>(() => {
  if (!trace.value) return []
  return [...trace.value.candidates]
    .filter(c => TIMELINE_STATUS.includes(c.status))
    .sort(compareBySchedulingOrder)
})

/**
 * 被跳过的候选：调度阶段就判定"这次不能用"，从未真正向上游发起请求。
 *
 * 默认展示：它是"为什么这次没用优先级更高的提供商"的直接答案，
 * 隐藏后容易让人误以为该提供商从未参与调度。开关供长时间排查时收起。
 */
const showSkippedCandidates = ref(true)

const skippedTraceCandidates = computed<CandidateRecord[]>(
  () => allTraceCandidates.value.filter(c => c.status === 'skipped'),
)

const rawTimeline = computed<CandidateRecord[]>(() => {
  if (showSkippedCandidates.value) return allTraceCandidates.value
  return allTraceCandidates.value.filter(c => c.status !== 'skipped')
})

const schedulingAudit = computed<Record<string, unknown> | null>(() => {
  const metadata = props.requestMetadata
  if (!metadata || typeof metadata !== 'object' || Array.isArray(metadata)) return null
  const raw = metadata.scheduling_audit
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return null
  return raw as Record<string, unknown>
})

const poolAttemptCandidates = computed<CandidateRecord[]>(() => {
  const auditAttempts = schedulingAudit.value?.attempts
  return buildPoolParticipatedCandidates(
    rawTimeline.value,
    auditAttempts,
    props.requestId,
  )
})

const poolAttemptsByGroup = computed<Map<string, CandidateRecord[]>>(() => {
  const grouped = new Map<string, CandidateRecord[]>()
  for (const attempt of poolAttemptCandidates.value) {
    const groupId =
      extractPoolGroupId(attempt)
      || String(attempt.provider_id || '').trim()
      || '__pool_group__'
    const existing = grouped.get(groupId)
    if (existing) {
      existing.push(attempt)
    } else {
      grouped.set(groupId, [attempt])
    }
  }
  return grouped
})

const poolAttemptKeySet = computed<Set<string>>(() => {
  return new Set(
    poolAttemptCandidates.value.map((item) => makeAttemptKey(item.candidate_index, item.retry_index)),
  )
})

const timeline = computed<CandidateRecord[]>(() => {
  if (poolAttemptCandidates.value.length === 0) return rawTimeline.value
  return rawTimeline.value.filter(
    (candidate) => !poolAttemptKeySet.value.has(makeAttemptKey(candidate.candidate_index, candidate.retry_index)),
  )
})

const normalizeProviderIdentity = (value: unknown): string => {
  if (typeof value !== 'string') return ''
  return value.trim().toLowerCase()
}

const buildProviderGroups = (items: CandidateRecord[]): TraceNodeGroup[] => {
  const groups: TraceNodeGroup[] = []
  let currentGroup: TraceNodeGroup | null = null

  items.forEach((candidate) => {
    const providerKey = candidate.provider_name || '未知'

    if (currentGroup && currentGroup.id === providerKey) {
      currentGroup.allAttempts.push(candidate)
      currentGroup.retryCount++
      currentGroup.endIndex = candidate.candidate_index
      currentGroup.totalLatency += candidate.latency_ms || 0
      if (candidate.extra_data?.needs_conversion) {
        currentGroup.hasConversion = true
      }
      const currentPriority = STATUS_PRIORITY[currentGroup.primaryStatus] ?? 0
      const newPriority = STATUS_PRIORITY[getDisplayStatus(candidate)] ?? 0
      if (newPriority > currentPriority) {
        currentGroup.primaryStatus = getDisplayStatus(candidate)
      }
      return
    }

    currentGroup = {
      id: providerKey,
      providerName: getProviderDisplayName(candidate),
      primary: candidate,
      primaryStatus: getDisplayStatus(candidate),
      allAttempts: [candidate],
      retryCount: 0,
      totalLatency: candidate.latency_ms || 0,
      startIndex: candidate.candidate_index,
      endIndex: candidate.candidate_index,
      hasConversion: candidate.extra_data?.needs_conversion === true,
      providerApiFormat: candidate.extra_data?.provider_api_format || null,
      isPoolGroup: false,
    }
    groups.push(currentGroup)
  })

  return groups
}

// 将相同 Provider 的所有请求合并为组（同提供商的 Key 放在子节点）
const groupedTimeline = computed<TraceNodeGroup[]>(() => {
  const providerGroups = buildProviderGroups(timeline.value.filter(isParticipatedCandidate))
  if (poolAttemptsByGroup.value.size === 0) {
    return providerGroups
  }

  const poolProviderIds = new Set<string>()
  const poolProviderNames = new Set<string>()
  const poolGroups: TraceNodeGroup[] = []

  for (const [groupId, attemptsRaw] of poolAttemptsByGroup.value.entries()) {
    const attempts = [...attemptsRaw].sort(compareBySchedulingOrder)
    if (attempts.length === 0) continue

    const visibleAttempts = buildPoolGroupVisibleAttempts(attempts)
    if (visibleAttempts.length === 0) continue

    const poolPrimaryStatus = visibleAttempts.reduce((best, current) => {
      const bestPriority = STATUS_PRIORITY[best] ?? 0
      const currentStatus = getDisplayStatus(current)
      const currentPriority = STATUS_PRIORITY[currentStatus] ?? 0
      return currentPriority > bestPriority ? currentStatus : best
    }, getDisplayStatus(visibleAttempts[0]))

    const successAttempt = visibleAttempts.find((item) => item.status === 'success')
    const poolPrimary =
      successAttempt || visibleAttempts[visibleAttempts.length - 1] || visibleAttempts[0]
    const startIndex = Math.min(...attempts.map(item => item.candidate_index))
    const endIndex = Math.max(...attempts.map(item => item.candidate_index))

    poolGroups.push({
      id: `pool:${groupId}`,
      providerName: getProviderDisplayName(poolPrimary, { allowAuthTypeFallback: false }),
      primary: poolPrimary,
      primaryStatus: poolPrimaryStatus,
      allAttempts: visibleAttempts,
      retryCount: Math.max(0, visibleAttempts.length - 1),
      totalLatency: visibleAttempts.reduce((sum, item) => sum + (item.latency_ms || 0), 0),
      startIndex,
      endIndex,
      hasConversion: visibleAttempts.some((item) => item.extra_data?.needs_conversion === true),
      providerApiFormat: null,
      isPoolGroup: true,
    })

    for (const attempt of attempts) {
      const providerId = String(attempt.provider_id || '').trim()
      if (providerId) poolProviderIds.add(providerId)
      const providerName = normalizeProviderIdentity(attempt.provider_name)
      if (providerName) poolProviderNames.add(providerName)
    }
  }

  const dedupedProviderGroups = providerGroups.filter((group) => {
    const sameProviderById = group.allAttempts.some((attempt) => {
      const providerId = String(attempt.provider_id || '').trim()
      return providerId !== '' && poolProviderIds.has(providerId)
    })
    if (sameProviderById) return false

    const groupName = normalizeProviderIdentity(group.primary.provider_name || group.providerName)
    if (groupName && poolProviderNames.has(groupName)) return false

    return true
  })

  const allGroups = [...poolGroups, ...dedupedProviderGroups]
  allGroups.sort((a, b) => a.startIndex - b.startIndex)
  return allGroups
})

// 格式转换分界点索引（首个 hasConversion=true 的 group index）
const conversionBoundaryIndex = computed(() => {
  const groups = groupedTimeline.value
  if (!groups || groups.length === 0) return -1
  const idx = groups.findIndex(g => g.hasConversion)
  // 只有当分界点不在最开头时才有意义（前面有 exact 候选）
  if (idx <= 0) return -1
  return idx
})

// The trace aggregate includes every attempted candidate, including failed
// failover attempts. Per-candidate latency remains provider-scoped.
const totalTraceLatency = computed(() => {
  if (!rawTimeline.value || rawTimeline.value.length === 0) return 0

  const aggregateLatency = trace.value?.total_latency_ms
  if (typeof aggregateLatency === 'number' && Number.isFinite(aggregateLatency) && aggregateLatency > 0) {
    return aggregateLatency
  }

  const attemptedLatency = rawTimeline.value.reduce((sum, candidate) => {
    const latency = normalizeLatencyMs(candidate.latency_ms)
    return sum + (latency ?? 0)
  }, 0)
  if (attemptedLatency > 0) {
    return attemptedLatency
  }

  // Historical transport failures may not have latency_ms. Recover the wall
  // clock span from candidate timestamps for those records.
  let earliestStart: number | null = null
  let latestEnd: number | null = null

  for (const candidate of rawTimeline.value) {
    if (candidate.started_at) {
      const startTime = new Date(candidate.started_at).getTime()
      if (earliestStart === null || startTime < earliestStart) {
        earliestStart = startTime
      }
    }
    if (candidate.finished_at) {
      const endTime = new Date(candidate.finished_at).getTime()
      if (latestEnd === null || endTime > latestEnd) {
        latestEnd = endTime
      }
    }
  }

  if (earliestStart !== null && latestEnd !== null) {
    return latestEnd - earliestStart
  }
  return 0
})

// 计算选中的组
const selectedGroup = computed(() => {
  if (!groupedTimeline.value || groupedTimeline.value.length === 0) return null
  return groupedTimeline.value[selectedGroupIndex.value]
})

// 计算当前查看的尝试
const currentAttempt = computed(() => {
  if (!selectedGroup.value) return null
  return selectedGroup.value.allAttempts[selectedAttemptIndex.value] || selectedGroup.value.primary
})

watch(currentAttempt, (attempt) => {
  emit('selectAttempt', attempt ?? null)
}, { immediate: true })

const latestTraceAttemptForState = computed<CandidateRecord | null>(() => {
  const candidates = rawTimeline.value
  for (let index = candidates.length - 1; index >= 0; index -= 1) {
    const candidate = candidates[index]
    // 跳过/未使用候选没有状态码与错误信息，不能代表本次请求的最终结果。
    if (
      candidate.status !== 'available' &&
      candidate.status !== 'unused' &&
      candidate.status !== 'skipped'
    ) {
      return candidate
    }
  }
  return null
})

const latestTraceImageProgress = computed<ImageProgress | null>(() => {
  const candidates = rawTimeline.value
  for (let index = candidates.length - 1; index >= 0; index -= 1) {
    const candidate = candidates[index]
    const progress = normalizeImageProgress(candidate.image_progress)
      ?? normalizeImageProgress(extractObject(candidate.extra_data)?.image_progress)
    if (progress) return progress
  }
  return null
})

watch(
  [trace, loading, latestTraceImageProgress, latestTraceAttemptForState, computedFinalStatus],
  ([value, isLoading, imageProgress, attempt, finalStatus]) => {
    const waitingForInternalTrace = Boolean(props.requestId && !props.traceData && !traceLoadStarted.value && !value)
    emit('traceState', {
      loaded: !isLoading && !waitingForInternalTrace,
      hasTrace: Boolean(value?.candidates?.length),
      finalStatus: finalStatus ?? value?.final_status ?? null,
      statusCode: attempt?.status_code ?? null,
      latencyMs: attempt?.latency_ms ?? value?.total_latency_ms ?? null,
      imageProgress,
      errorMessage: attempt?.error_message ?? null,
    })
  },
  { immediate: true },
)

/**
 * 区分"跳过"与"尝试失败"：被跳过的候选从未发到上游，
 * 不写清楚容易被误读成"上游返回了错误"。
 */

const hasActiveImageProgress = computed(() => {
  return rawTimeline.value.some((candidate) => {
    const progress = normalizeImageProgress(candidate.image_progress)
      ?? normalizeImageProgress(extractObject(candidate.extra_data)?.image_progress)
    if (!progress?.phase) return false
    return progress.phase !== 'upstream_completed' && progress.phase !== 'failed'
  })
})

// 检查组是否被悬浮
const isGroupHovered = (groupIndex: number) => {
  return hoveredGroupIndex.value === groupIndex
}

// 检查组是否被选中
const isGroupSelected = (group: TraceNodeGroup) => {
  return selectedGroupIndex.value === groupedTimeline.value.findIndex(g => g.id === group.id && g.startIndex === group.startIndex)
}

const findGroupIndex = (groups: TraceNodeGroup[], group: TraceNodeGroup): number => {
  return groups.findIndex(g => g.id === group.id && g.startIndex === group.startIndex)
}

const selectedAttemptFromGroups = (groups: TraceNodeGroup[]): CandidateRecord | null => {
  const group = groups[selectedGroupIndex.value]
  if (!group) return null
  return group.allAttempts[selectedAttemptIndex.value] || null
}

const groupHasSuccess = (group: TraceNodeGroup): boolean => {
  return group.allAttempts.some(attempt => getDisplayStatus(attempt) === 'success')
}

const groupsHaveSuccess = (groups: TraceNodeGroup[]): boolean => {
  return groups.some(groupHasSuccess)
}

const groupsHaveLiveCandidate = (groups: TraceNodeGroup[]): boolean => {
  return groups.some(group => group.allAttempts.some(isLiveCandidate))
}

const TERMINAL_ATTEMPT_STATUSES = ['failed', 'cancelled', 'stream_interrupted', 'skipped']

const isTerminalResultAttempt = (attempt: CandidateRecord): boolean => {
  return TERMINAL_ATTEMPT_STATUSES.includes(getDisplayStatus(attempt))
}

const groupsHaveTerminalResult = (groups: TraceNodeGroup[]): boolean => {
  return groups.some(group => group.allAttempts.some(isTerminalResultAttempt))
}

const selectedAttemptMatchesBestSilentState = (groups: TraceNodeGroup[]): boolean => {
  const attempt = selectedAttemptFromGroups(groups)
  if (!attempt) return false

  if (groupsHaveSuccess(groups)) {
    return getDisplayStatus(attempt) === 'success'
  }

  if (groupsHaveLiveCandidate(groups)) {
    return isLiveCandidate(attempt)
  }

  if (groupsHaveTerminalResult(groups)) {
    return isTerminalResultAttempt(attempt)
  }

  return true
}

const selectMostRelevantGroup = (newGroups: TraceNodeGroup[]) => {
  if (!newGroups || newGroups.length === 0) return

  // 查找成功的组
  const successIdx = newGroups.findIndex(groupHasSuccess)
  if (successIdx >= 0) {
    selectedGroupIndex.value = successIdx
    // 选中成功的尝试
    const group = newGroups[successIdx]
    const attemptIdx = group.allAttempts.findIndex(a => getDisplayStatus(a) === 'success')
    selectedAttemptIndex.value = attemptIdx >= 0 ? attemptIdx : 0
    return
  }

  // 查找正在进行的组
  const activeIdx = newGroups.findIndex(g => g.allAttempts.some(isLiveCandidate))
  if (activeIdx >= 0) {
    selectedGroupIndex.value = activeIdx
    // 选中正在进行的尝试，而非最后一个
    const group = newGroups[activeIdx]
    const attemptIdx = group.allAttempts.findIndex(isLiveCandidate)
    selectedAttemptIndex.value = attemptIdx >= 0 ? attemptIdx : group.allAttempts.length - 1
    return
  }

  // 查找最后一个有效结果的组（有实际执行过的状态：failed/cancelled/stream_interrupted/skipped）
  // 从后往前找第一个有效状态的组
  for (let i = newGroups.length - 1; i >= 0; i--) {
    const group = newGroups[i]
    if (TERMINAL_ATTEMPT_STATUSES.includes(group.primaryStatus)) {
      selectedGroupIndex.value = i
      // 选中最后一个有效状态的尝试（从后往前遍历）
      let targetIdx = -1
      for (let j = group.allAttempts.length - 1; j >= 0; j--) {
        if (isTerminalResultAttempt(group.allAttempts[j])) {
          targetIdx = j
          break
        }
      }
      selectedAttemptIndex.value = targetIdx >= 0 ? targetIdx : group.allAttempts.length - 1
      return
    }
  }

  // 都没有有效状态，选择第一个组（避免选到末尾的未执行节点）
  selectedGroupIndex.value = 0
  selectedAttemptIndex.value = 0
}

// 选中一个组
const selectGroup = (group: TraceNodeGroup) => {
  const index = findGroupIndex(groupedTimeline.value, group)
  if (index >= 0) {
    selectionPinnedByUser.value = true
    selectedGroupIndex.value = index
    // 默认选中成功的尝试，或最后一个尝试
    const successIdx = group.allAttempts.findIndex(a => a.status === 'success')
    selectedAttemptIndex.value = successIdx >= 0 ? successIdx : group.allAttempts.length - 1
  }
}

// 选中一个组的首次请求
const selectFirstAttempt = (group: TraceNodeGroup) => {
  const index = findGroupIndex(groupedTimeline.value, group)
  if (index >= 0) {
    selectionPinnedByUser.value = true
    selectedGroupIndex.value = index
    selectedAttemptIndex.value = 0
  }
}

const selectAttemptInGroup = (group: TraceNodeGroup, attemptIndex: number) => {
  const groupIndex = findGroupIndex(groupedTimeline.value, group)
  if (groupIndex < 0) return
  selectionPinnedByUser.value = true
  selectedGroupIndex.value = groupIndex
  selectedAttemptIndex.value = attemptIndex
}

const isAttemptSelected = (group: TraceNodeGroup, attemptIndex: number) => {
  return isGroupSelected(group) && selectedAttemptIndex.value === attemptIndex
}

const formatCandidateAttemptIndex = (attempt: CandidateRecord): string => {
  return attempt.retry_index > 0
    ? `#${attempt.candidate_index}.${attempt.retry_index}`
    : `#${attempt.candidate_index}`
}

const formatAttemptDotTitle = (attempt: CandidateRecord): string => {
  const parts = [
    formatCandidateAttemptIndex(attempt),
    attempt.key_name || attempt.key_account_label || attempt.key_preview || '未知 Key',
    getStatusLabel(getDisplayStatus(attempt)),
  ]
  return parts.filter(Boolean).join(' · ')
}

// 导航到上/下一组
const navigateGroup = (direction: number) => {
  const newIndex = selectedGroupIndex.value + direction
  if (newIndex >= 0 && newIndex < groupedTimeline.value.length) {
    selectionPinnedByUser.value = true
    selectedGroupIndex.value = newIndex
    const group = groupedTimeline.value[newIndex]
    // 默认选中成功的尝试，或最后一个尝试
    const successIdx = group.allAttempts.findIndex(a => a.status === 'success')
    selectedAttemptIndex.value = successIdx >= 0 ? successIdx : group.allAttempts.length - 1
  }
}

const navigateAttempt = (direction: number) => {
  const group = selectedGroup.value
  if (!group) return
  const newIndex = selectedAttemptIndex.value + direction
  if (newIndex >= 0 && newIndex < group.allAttempts.length) {
    selectionPinnedByUser.value = true
    selectedAttemptIndex.value = newIndex
  }
}

// 加载请求追踪数据
const isSilentRefresh = ref(false)
const loadTrace = async (silent = false) => {
  const requestId = props.requestId
  if (!requestId || props.traceData) return
  if (traceLoadInFlight) return traceLoadInFlight

  traceLoadInFlight = (async () => {
    isSilentRefresh.value = silent
    traceLoadStarted.value = true

    if (!silent) {
      loading.value = true
    }
    error.value = null

    try {
      // 始终拉取全部候选：被跳过的候选是"为什么没用某个提供商"的关键证据，
      // 是否显示由前端开关控制，避免再发一次请求。
      internalTrace.value = await requestTraceApi.getRequestTrace(requestId, { attemptedOnly: false })
    } catch (err: unknown) {
      if (isAxiosError(err) && err.response?.status === 404) {
        internalTrace.value = null
        error.value = null
        return
      }
      if (!silent) {
        error.value = parseApiError(err, t('ui.failedToLoad'))
      }
      log.error('加载请求追踪失败:', err)
    } finally {
      if (!silent) {
        loading.value = false
      }
      traceLoadInFlight = null
    }
  })()

  return traceLoadInFlight
}

const propsRequestIsActive = computed(() => {
  const status = props.requestStatus ?? usageData.value?.status
  return status === 'pending' || status === 'streaming'
})

const traceHasActiveCandidate = computed(() => {
  return rawTimeline.value.some(isLiveCandidate)
})

const traceFinalIsTerminal = computed(() => {
  const status = trace.value?.final_status
  return status === 'success' || status === 'failed' || status === 'cancelled'
})

const shouldPollTrace = computed(() => {
  if (!props.requestId || props.traceData) return false
  if (traceHasActiveCandidate.value || hasActiveImageProgress.value) return true
  return propsRequestIsActive.value && !traceFinalIsTerminal.value
})

const stopTracePolling = () => {
  if (tracePollTimer) {
    clearTimeout(tracePollTimer)
    tracePollTimer = null
  }
}

const scheduleTracePolling = () => {
  stopTracePolling()
  if (!shouldPollTrace.value) return

  tracePollTimer = setTimeout(async () => {
    await loadTrace(true)
    scheduleTracePolling()
  }, TRACE_POLL_INTERVAL_MS)
}

// 监听 groupedTimeline 变化，自动选择最有意义的组
watch(groupedTimeline, (newGroups) => {
  if (!newGroups || newGroups.length === 0) return

  // 静默刷新时保留用户手动选择；未手动选择时跟随成功/进行中的 Key。
  if (isSilentRefresh.value) {
    isSilentRefresh.value = false
    if (selectionPinnedByUser.value && selectedAttemptFromGroups(newGroups)) {
      return
    }
    if (selectedAttemptMatchesBestSilentState(newGroups)) {
      return
    }
    selectMostRelevantGroup(newGroups)
    return
  }

  selectMostRelevantGroup(newGroups)
}, { immediate: true })

// 监听 requestId / 外部 trace 变化
watch(
  [() => props.requestId, () => props.traceData],
  () => {
    selectedGroupIndex.value = 0
    selectedAttemptIndex.value = 0
    selectionPinnedByUser.value = false
    traceLoadStarted.value = false

    if (props.traceData) {
      internalTrace.value = null
      loading.value = false
      error.value = null
      return
    }

    if (!props.requestId) {
      internalTrace.value = null
      loading.value = false
      error.value = null
      return
    }

    void loadTrace()
  },
  { immediate: true },
)

watch(shouldPollTrace, () => {
  scheduleTracePolling()
}, { immediate: true })

onBeforeUnmount(() => {
  stopTracePolling()
})

defineExpose({ refresh: () => loadTrace(true) })

</script>

<style scoped>
.minimal-request-timeline {
  width: 100%;
}

/* 极简轨道 - 包装器实现溢出时居左、不溢出时居中 */
.minimal-track {
  display: flex;
  align-items: center;
  justify-content: safe center;
  gap: 64px;
  padding: 2rem 2rem 2.75rem;
  overflow-x: auto;
  overflow-y: hidden;

  /* 优化滚动体验 */
  scrollbar-width: thin; /* Firefox */
  scrollbar-color: hsl(var(--border)) transparent;
}

/* Webkit 滚动条样式 */
.minimal-track::-webkit-scrollbar {
  height: 6px;
}

.minimal-track::-webkit-scrollbar-track {
  background: transparent;
}

.minimal-track::-webkit-scrollbar-thumb {
  background: hsl(var(--border));
  border-radius: 3px;
}

.minimal-track::-webkit-scrollbar-thumb:hover {
  background: hsl(var(--muted-foreground) / 0.5);
}

.minimal-node-group {
  display: flex;
  align-items: center;
  position: relative;
  cursor: pointer;
}

/* 节点容器 */
.node-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  position: relative;
}

/* 节点名称 - 绝对定位在节点上方 */
.node-label {
  position: absolute;
  bottom: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  font-size: 0.65rem;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 主节点 - 同心圆（外圈轮廓 + 间隙 + 内部实心圆） */
.node-dot {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  transition: all 0.2s ease;
  z-index: 2;
  position: relative;
  overflow: visible;
  cursor: pointer;
  /* 外圈轮廓 */
  border: 2px solid currentColor;
  background: transparent;
}

/* 内部实心圆 - 使用 ::before 伪元素 */
.node-dot::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
  transform: translate(-50%, -50%);
}

/* 选中首次时的样式 */
.node-dot.is-first-selected {
  transform: scale(1.1);
}

/* 子节点容器 - 绝对定位在主节点下方 */
.sub-dots {
  position: absolute;
  top: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 6px;
  padding: 0;
  background: transparent;
  z-index: 3;
}

/* 子节点 - 增大点击区域 */
.sub-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
  opacity: 0.5;
  position: relative;
}

/* 扩大点击热区 */
.sub-dot::before {
  content: '';
  position: absolute;
  top: -4px;
  left: -4px;
  right: -4px;
  bottom: -4px;
}

.sub-dot:hover {
  transform: scale(1.2);
  opacity: 0.9;
}

.sub-dot.active {
  opacity: 1;
  transform: scale(1.15);
  box-shadow: 0 0 0 2px hsl(var(--background)), 0 0 0 3px currentColor;
}

/* 子节点状态颜色 */
.sub-dot.status-success { background: #22c55e; color: #22c55e; }

.sub-dot.status-failed { background: #ef4444; color: #ef4444; }

.sub-dot.status-cancelled { background: #f59e0b; color: #f59e0b; }

.sub-dot.status-pending { background: #3b82f6; color: #3b82f6; }

.sub-dot.status-skipped { background: hsl(var(--foreground)); color: hsl(var(--foreground)); }

.sub-dot.status-available { background: #d1d5db; color: #d1d5db; }

/* 选中状态：呼吸动画 + 涟漪效果 */
.minimal-node-group.selected .node-dot {
  animation: breathe 2s ease-in-out infinite;
}

.minimal-node-group.selected .node-dot::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid currentColor;
  background: transparent;
  transform: translate(-50%, -50%);
  animation: ripple 1.5s ease-out infinite;
  z-index: -1;
}

/* 悬停状态：只有放大效果 */
.minimal-node-group.hovered .node-dot {
  transform: scale(1.3);
}

@keyframes breathe {
  0%, 100% { transform: scale(1.3); }
  50% { transform: scale(1.5); }
}

@keyframes ripple {
  0% {
    transform: translate(-50%, -50%) scale(1);
    opacity: 0.4;
  }
  100% {
    transform: translate(-50%, -50%) scale(2.5);
    opacity: 0;
  }
}

/* 状态颜色 - 同心圆使用 color */
.node-dot.status-success { color: #22c55e; }

.node-dot.status-failed { color: #ef4444; }

.node-dot.status-cancelled { color: #f59e0b; }

.node-dot.status-pending { color: #3b82f6; }

.node-dot.status-skipped { color: hsl(var(--foreground)); }

.node-dot.status-available { color: #d1d5db; }

/* 连接线容器 */
.node-line-wrapper {
  position: absolute;
  right: -64px;
  top: 50%;
  transform: translateY(-50%);
  width: 64px;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.node-line {
  width: 100%;
  height: 2px;
  background: hsl(var(--border));
}

/* 格式转换分界线 */
.node-line.conversion-boundary {
  background: none;
  height: 0;
  border-top: 2px dashed hsl(var(--muted-foreground) / 0.4);
}

/* 「显示/隐藏被跳过候选」开关：低调的描边小按钮，不抢主状态徽标的视觉重心 */
.skipped-toggle {
  padding: 0.125rem 0.5rem;
  border: 1px dashed hsl(var(--border));
  border-radius: 9999px;
  background: transparent;
  color: hsl(var(--muted-foreground));
  font-size: 0.75rem;
  line-height: 1.5;
  cursor: pointer;
  transition: all 0.15s ease;
}

.skipped-toggle:hover {
  color: hsl(var(--foreground));
  border-color: hsl(var(--muted-foreground) / 0.5);
}

.skipped-toggle.active {
  border-style: solid;
  border-color: hsl(var(--primary) / 0.5);
  color: hsl(var(--primary));
}

/* 动画 */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.25s ease;
}

.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  transform: translateY(10px);
}
</style>
