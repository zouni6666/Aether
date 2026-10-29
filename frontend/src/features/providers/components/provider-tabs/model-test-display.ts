
import { translate as t } from '@/i18n'
import { formatApiFormat } from '@/api/endpoints/types/api-format'
import type { TestAttemptDetail, TestCandidateSummary, TestModelFailoverResponse } from '@/api/endpoints/providers'
import type { CandidateRecord } from '@/api/requestTrace'
import {
  extractModelTestImagePreviews,
  extractModelTestResponsePreview,
  formatModelTestDiagnostic,
} from './model-test-request'
import type { ModelTestImagePreview } from './model-test-request'

// 模型测试结果的展示辅助：候选摘要归一化、状态文案、尝试记录格式化与检视数据选取

export type SummaryView = {
  total_candidates: number
  attempted: number
  success: number
  failed: number
  skipped: number
  unused: number
  pending: number
  available: number
  completed: number
  stop_reason?: string | null
  winning_candidate_index?: number | null
  winning_key_name?: string | null
  winning_key_id?: string | null
  winning_auth_type?: string | null
  winning_effective_model?: string | null
  winning_endpoint_api_format?: string | null
  winning_endpoint_base_url?: string | null
  winning_latency_ms?: number | null
  winning_status_code?: number | null
}

export type SummaryBadgeItem = {
  key: string
  label: string
  value: number
  variant: 'default' | 'secondary' | 'outline' | 'success' | 'destructive'
}

export function toCount(value: unknown): number {
  return typeof value === 'number' && Number.isFinite(value) ? Math.max(0, value) : 0
}

export function normalizeCandidateSummary(summary: TestCandidateSummary | null | undefined): SummaryView | null {
  if (!summary) return null
  const success = toCount(summary.success)
  const failed = toCount(summary.failed)
  const skipped = toCount(summary.skipped)
  const unused = toCount(summary.unused)
  const pending = toCount(summary.pending)
  const available = toCount(summary.available)
  return {
    total_candidates: toCount(summary.total_candidates),
    attempted: toCount(summary.attempted),
    success,
    failed,
    skipped,
    unused,
    pending,
    available,
    completed: toCount(summary.completed) || success + failed + skipped + unused,
    stop_reason: summary.stop_reason ?? null,
    winning_candidate_index: summary.winning_candidate_index ?? null,
    winning_key_name: summary.winning_key_name ?? null,
    winning_key_id: summary.winning_key_id ?? null,
    winning_auth_type: summary.winning_auth_type ?? null,
    winning_effective_model: summary.winning_effective_model ?? null,
    winning_endpoint_api_format: summary.winning_endpoint_api_format ?? null,
    winning_endpoint_base_url: summary.winning_endpoint_base_url ?? null,
    winning_latency_ms: summary.winning_latency_ms ?? null,
    winning_status_code: summary.winning_status_code ?? null,
  }
}

export function deriveSummaryFromAttempts(result: TestModelFailoverResponse | null): SummaryView {
  const normalized = normalizeCandidateSummary(result?.candidate_summary)
  if (normalized) return normalized

  const attempts = result?.attempts ?? []
  const success = attempts.filter(attempt => attempt.status === 'success').length
  const failed = attempts.filter(attempt => ['failed', 'cancelled', 'stream_interrupted'].includes(attempt.status)).length
  const skipped = attempts.filter(attempt => attempt.status === 'skipped').length
  const pending = attempts.filter(attempt => ['pending', 'streaming'].includes(attempt.status)).length
  const available = attempts.filter(attempt => attempt.status === 'available').length
  const totalCandidates = Math.max(result?.total_candidates ?? 0, attempts.length)
  const explicitUnused = attempts.filter(attempt => attempt.status === 'unused').length
  const winningAttempt = attempts.find(attempt => attempt.status === 'success') ?? null
  const unused = explicitUnused > 0
    ? explicitUnused
    : success > 0 && winningAttempt
      ? Math.max(0, totalCandidates - winningAttempt.candidate_index - 1)
      : 0
  const attempted = result?.total_attempts ?? attempts.filter(attempt => !['skipped', 'available', 'unused'].includes(attempt.status)).length
  const stopReason = totalCandidates === 0
    ? 'no_candidate'
    : success > 0
      ? 'first_success'
      : attempted === 0 && skipped > 0
        ? 'all_skipped'
        : failed > 0 || skipped > 0
          ? 'exhausted'
          : 'pending'

  return {
    total_candidates: totalCandidates,
    attempted,
    success,
    failed,
    skipped,
    unused,
    pending,
    available,
    completed: success + failed + skipped + unused,
    stop_reason: stopReason,
    winning_candidate_index: winningAttempt?.candidate_index ?? null,
    winning_key_name: winningAttempt?.key_name ?? null,
    winning_key_id: winningAttempt?.key_id ?? null,
    winning_auth_type: winningAttempt?.auth_type ?? null,
    winning_effective_model: winningAttempt?.effective_model ?? null,
    winning_endpoint_api_format: winningAttempt?.endpoint_api_format ?? null,
    winning_endpoint_base_url: winningAttempt?.endpoint_base_url ?? null,
    winning_latency_ms: winningAttempt?.latency_ms ?? null,
    winning_status_code: winningAttempt?.status_code ?? null,
  }
}

export const detailTabs = [
  { name: 'request-headers', get label() { return t('ui.requestHeaders') } },
  { name: 'request-body', get label() { return t('ui.requestBody') } },
  { name: 'response-headers', get label() { return t('ui.responseHeaders') } },
  { name: 'response-body', get label() { return t('ui.responseBody') } },
] as const

export function statusVariant(status: string) {
  if (status === 'success') return 'success' as const
  if (status === 'failed' || status === 'stream_interrupted') return 'destructive' as const
  return 'secondary' as const
}

export function statusLabel(status: string) {
  if (status === 'success') return t('ui.success')
  if (status === 'failed') return t('ui.failed28384d7')
  if (status === 'skipped') return t('ui.skip')
  if (status === 'pending') return t('ui.waiting')
  if (status === 'streaming') return t('ui.testing')
  if (status === 'cancelled') return t('ui.canceled')
  if (status === 'stream_interrupted') return t('ui.streamInterrupted')
  if (status === 'available') return t('ui.awaitingRequest')
  if (status === 'unused') return t('ui.notRun')
  return status
}

export function statusDisplay(item: { status: string; status_code?: number | null }): string {
  const code = item.status_code
  const status = item.status
  if (!code) return statusLabel(status)
  if (status === 'failed' && code >= 200 && code < 300) {
    return t('providers.migration.bodyErrore8bd10', { value: String(code) })
  }
  return String(code)
}

export function attemptRowClass(status: string, selected = false) {
  const statusClass = (() => {
    if (status === 'success') return 'bg-green-500/5'
    if (status === 'failed') return 'bg-red-500/5'
    if (status === 'cancelled') return 'bg-amber-500/5'
    if (status === 'skipped') return 'bg-muted/20'
    return ''
  })()
  const interactionClass = 'cursor-pointer transition-colors hover:bg-muted/40 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary/70'
  const selectedClass = selected ? 'ring-1 ring-primary/70 bg-primary/5' : ''
  return [statusClass, interactionClass, selectedClass].filter(Boolean).join(' ')
}

export function maskKey(key: string): string {
  if (key.length <= 8) return key
  return `${key.slice(0, 4)}...${key.slice(-4)}`
}

export function formatAuthType(authType: string): string {
  const lowered = authType.toLowerCase()
  if (lowered === 'api_key') return 'API Key'
  if (lowered === 'service_account') return 'Service Account'
  if (lowered === 'oauth') return 'OAuth'
  if (lowered === 'codex') return 'Codex OAuth'
  if (lowered === 'antigravity') return 'Antigravity OAuth'
  if (lowered === 'kiro') return 'Kiro OAuth'
  if (lowered === 'grok') return 'Grok OAuth'
  if (lowered === 'xai') return 'xAI OAuth'
  return authType
}

export function formatAttemptIndex(attempt: TestAttemptDetail): string {
  const retryIndex = attempt.retry_index ?? 0
  return retryIndex > 0 ? `#${attempt.candidate_index}.${retryIndex}` : `#${attempt.candidate_index}`
}

export function formatTraceCandidateIndex(candidate: CandidateRecord): string {
  return candidate.retry_index > 0 ? `#${candidate.candidate_index}.${candidate.retry_index}` : `#${candidate.candidate_index}`
}

export function formatTraceCandidateAccount(candidate: CandidateRecord): string {
  return candidate.key_account_label || candidate.key_name || candidate.key_preview || '-'
}

export function extractAttemptRequestModel(attempt: TestAttemptDetail): string | null {
  const body = attempt.request_body
  if (!body || typeof body !== 'object' || Array.isArray(body)) return null

  const model = (body as Record<string, unknown>).model
  if (typeof model === 'string' && model.trim()) return model.trim()

  const conversationState = (body as Record<string, unknown>).conversationState
  if (!conversationState || typeof conversationState !== 'object' || Array.isArray(conversationState)) return null

  const currentMessage = (conversationState as Record<string, unknown>).currentMessage
  if (!currentMessage || typeof currentMessage !== 'object' || Array.isArray(currentMessage)) return null

  const userInputMessage = (currentMessage as Record<string, unknown>).userInputMessage
  if (!userInputMessage || typeof userInputMessage !== 'object' || Array.isArray(userInputMessage)) return null

  const modelId = (userInputMessage as Record<string, unknown>).modelId
  return typeof modelId === 'string' && modelId.trim() ? modelId.trim() : null
}

export function traceCandidateDetail(candidate: CandidateRecord): string {
  if (candidate.skip_reason) return formatModelTestDiagnostic(candidate.skip_reason)
  if (candidate.error_message) return formatModelTestDiagnostic(candidate.error_message)
  if (candidate.status === 'unused') return t('ui.notRequestedAfterSuccess')
  if (candidate.status === 'available') return t('ui.awaitingRequest')
  if (candidate.endpoint_name) return t('providers.migration.endpointafa852', { value: String(formatApiFormat(candidate.endpoint_name)) })
  return ''
}

export function attemptDetail(attempt: TestAttemptDetail): string {
  if (attempt.status === 'cancelled') return t('ui.testCanceled')
  if (attempt.status === 'unused') return t('ui.notRequestedAfterSuccess')
  if (attempt.skip_reason) return formatModelTestDiagnostic(attempt.skip_reason)
  if (attempt.error_message) return formatModelTestDiagnostic(attempt.error_message)
  if (attempt.status === 'success') {
    return extractModelTestResponsePreview(attempt.response_body)
      ?? (attempt.effective_model ? t('providers.migration.requestModel97c199', { value: String(attempt.effective_model) }) : attempt.endpoint_base_url)
  }
  return '-'
}

export function attemptImagePreviews(attempt: TestAttemptDetail): ModelTestImagePreview[] {
  return extractModelTestImagePreviews(attempt.response_body)
}

export function inspectionKey(attempt: TestAttemptDetail): string {
  return `${attempt.candidate_index}:${attempt.retry_index ?? 0}:${attempt.key_id}`
}

export function hasDebugData(attempt: TestAttemptDetail): boolean {
  return Boolean(
    attempt.request_url
    || attempt.request_headers
    || attempt.request_body != null
    || attempt.response_headers
    || attempt.response_body != null,
  )
}

export function getInspectionTabData(
  tabName: typeof detailTabs[number]['name'],
  attempt: TestAttemptDetail | null,
): unknown {
  if (!attempt) return null
  switch (tabName) {
    case 'request-headers':
      return attempt.request_headers
    case 'request-body':
      return attempt.request_body
    case 'response-headers':
      return attempt.response_headers
    case 'response-body':
      return attempt.response_body
  }
}

export function formatJsonDraft(
  draft: string,
  onFormatted: (value: string) => void,
  emptyFallback?: string,
) {
  const normalized = draft.trim()
  if (!normalized) {
    if (emptyFallback !== undefined) {
      onFormatted(emptyFallback)
    }
    return
  }

  try {
    const parsed = JSON.parse(normalized)
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return
    onFormatted(JSON.stringify(parsed, null, 2))
  } catch {
    // keep user input untouched when JSON is invalid
  }
}
