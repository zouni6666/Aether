import { translate } from '@/i18n'
import { getI18nLocale } from '@/i18n'
import type { CandidateProxy, CandidateRecord, ImageProgress } from '@/api/requestTrace'
import { formatApiFormat } from '@/api/endpoints/types/api-format'
import { visibleFailureRecords } from '@/features/usage/utils/failureDiagnostic'

export interface AttemptTimeRange {
  startIso: string
  endIso?: string
  durationLabel?: string
}

// 获取最终状态标签
export const getFinalStatusLabel = (status: string) => {
  const labels: Record<string, string> = {
    get success() { return translate('ui.succeeded') },
    get failed() { return translate('ui.failed') },
    get cancelled() { return translate('ui.canceled') },
    get streaming() { return translate('ui.streaming80abc0b') },
    get pending() { return translate('ui.inProgress') }
  }
  return labels[status] || status
}

// 获取最终状态徽章样式
export type BadgeVariant = 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning' | 'dark'

export const getFinalStatusBadgeVariant = (status: string): BadgeVariant => {
  const variants: Record<string, BadgeVariant> = {
    success: 'success',
    failed: 'destructive',
    cancelled: 'warning',
    streaming: 'secondary',
    pending: 'secondary'
  }
  return variants[status] || 'default'
}

// 格式化延迟（自动调整单位）
export const formatLatency = (ms: number | undefined | null): string => {
  if (ms === undefined || ms === null) return '-'
  if (ms >= 1000) {
    return `${(ms / 1000).toFixed(2)}s`
  }
  return `${ms}ms`
}

// 格式化字节大小
export const formatSize = (bytes: number): string => {
  if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(1)}MB`
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)}KB`
  return `${bytes}B`
}

// 代理 timing 分阶段展示
export const proxyTimingBreakdown = (proxy: CandidateProxy): string => {
  const t = proxy.timing
  if (!t) return ''

  const parts: string[] = []

  // 兼容旧版 timing（含 body_read_ms/decompress_ms）
  const readDecompress = ((t.body_read_ms as number) || 0) + ((t.decompress_ms as number) || 0)
  if (readDecompress > 0) {
    let label = `读取 ${formatLatency(readDecompress)}`
    if (t.decompress_ms != null && t.decompress_ms > 0 && t.wire_size != null && t.body_size != null && (t.body_size as number) > 0) {
      const ratio = Math.round((1 - (t.wire_size as number) / (t.body_size as number)) * 100)
      label += ` ${formatSize(t.wire_size as number)}→${formatSize(t.body_size as number)}`
      if (ratio > 0) label += ` -${ratio}%`
    }
    parts.push(label)
  }

  const ttfbMs = t.ttfb_ms ?? t.upstream_ms
  const responseWaitMs = t.response_wait_ms ?? (
    t.connection_acquire_ms != null && ttfbMs != null
      ? Math.max(0, (ttfbMs as number) - (t.connection_acquire_ms as number))
      : null
  )
  const legacyWaitMs = t.upstream_processing_ms ?? (
    ttfbMs != null && t.connect_ms != null && t.tls_ms != null
      ? Math.max(0, (ttfbMs as number) - (t.connect_ms as number) - (t.tls_ms as number))
      : null
  )

  if (t.dns_ms != null && (t.dns_ms as number) > 0) {
    parts.push(`DNS ${formatLatency(t.dns_ms as number)}`)
  }
  if (t.connection_reused === true) {
    parts.push('复用连接')
  }
  if (t.connect_ms != null && (t.connect_ms as number) > 0) {
    parts.push(`连接 ${formatLatency(t.connect_ms as number)}`)
  }
  if (t.tls_ms != null && (t.tls_ms as number) > 0) {
    parts.push(`TLS ${formatLatency(t.tls_ms as number)}`)
  }
  if (ttfbMs != null && (ttfbMs as number) > 0) {
    parts.push(`TTFB ${formatLatency(ttfbMs as number)}`)
  }
  if (responseWaitMs != null && (responseWaitMs as number) > 0) {
    parts.push(`等待响应头 ${formatLatency(Math.round(responseWaitMs as number))}`)
  } else if (legacyWaitMs != null && (legacyWaitMs as number) > 0) {
    parts.push(`等待响应头(旧版估算) ${formatLatency(Math.round(legacyWaitMs as number))}`)
  }

  // 计算 Aether→代理 之间无法解释的耗时差
  if (proxy.ttfb_ms != null && t.total_ms != null) {
    const gap = (proxy.ttfb_ms as number) - (t.total_ms as number)
    if (gap > 500) {
      parts.push(`传输 ${formatLatency(Math.round(gap))}`)
    }
  }

  return parts.join(' / ')
}

export const normalizeFormatSignature = (value: string): string => {
  return value.trim().toLowerCase()
}

export const extractObject = (value: unknown): Record<string, unknown> | null => {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return null
  }
  return value as Record<string, unknown>
}

export const readStringField = (obj: Record<string, unknown>, key: string): string | undefined => {
  const value = obj[key]
  return typeof value === 'string' && value.trim() ? value.trim() : undefined
}

export const readNumberField = (obj: Record<string, unknown>, key: string): number | undefined => {
  const value = obj[key]
  if (typeof value === 'number' && Number.isFinite(value)) return value
  if (typeof value === 'string' && value.trim()) {
    const parsed = Number(value)
    if (Number.isFinite(parsed)) return parsed
  }
  return undefined
}

export const hasRenderableValue = (value: unknown): boolean => {
  if (value == null) return false
  if (typeof value === 'string') return value.trim().length > 0
  if (typeof value === 'object') return Object.keys(value as Record<string, unknown>).length > 0
  return true
}

export const normalizeImageProgress = (value: unknown): ImageProgress | null => {
  const raw = extractObject(value)
  if (!raw) return null

  const progress: ImageProgress = {
    phase: readStringField(raw, 'phase'),
    upstream_ttfb_ms: readNumberField(raw, 'upstream_ttfb_ms') ?? null,
    upstream_sse_frame_count: readNumberField(raw, 'upstream_sse_frame_count') ?? null,
    last_upstream_event: readStringField(raw, 'last_upstream_event') ?? null,
    last_upstream_frame_at_unix_ms: readNumberField(raw, 'last_upstream_frame_at_unix_ms') ?? null,
    partial_image_count: readNumberField(raw, 'partial_image_count') ?? null,
    last_client_visible_event: readStringField(raw, 'last_client_visible_event') ?? null,
    downstream_heartbeat_count: readNumberField(raw, 'downstream_heartbeat_count') ?? null,
    last_downstream_heartbeat_at_unix_ms: readNumberField(raw, 'last_downstream_heartbeat_at_unix_ms') ?? null,
    downstream_heartbeat_interval_ms: readNumberField(raw, 'downstream_heartbeat_interval_ms') ?? null,
  }

  return Object.values(progress).some(value => value !== undefined && value !== null && value !== '') ? progress : null
}

export const formatImageProgressPhase = (phase?: string | null): string => {
  const labels: Record<string, string> = {
    get upstream_connecting() { return translate('ui.connectingToUpstream') },
    get upstream_streaming() { return translate('ui.upstreamGenerating') },
    get upstream_completed() { return translate('ui.upstreamComplete') },
    get failed() { return translate('ui.failed28384d7') },
  }
  if (!phase) return translate('chart.unknown')
  return labels[phase] || phase
}

export const imageProgressPhaseClass = (phase?: string | null): string => {
  if (phase === 'upstream_completed') return 'phase-completed'
  if (phase === 'failed') return 'phase-failed'
  if (phase === 'upstream_streaming') return 'phase-streaming'
  return 'phase-connecting'
}

export const formatProgressCount = (value?: number | null): string => {
  return typeof value === 'number' && Number.isFinite(value) ? String(value) : '-'
}

export const formatProgressFrameTime = (value?: number | null): string => {
  if (typeof value !== 'number' || !Number.isFinite(value) || value <= 0) return '-'
  const date = new Date(value)
  const time = formatTime(date.toISOString())
  const ageMs = Date.now() - value
  if (ageMs >= 0 && ageMs < 60_000) {
    return `${Math.max(0, Math.round(ageMs / 1000))}s 前 (${time})`
  }
  return time
}

export const normalizeUpstreamResponseDisplay = (value: unknown): Record<string, unknown> | null => {
  const raw = extractObject(value)
  if (!raw) return null
  const statusCode = readNumberField(raw, 'status_code') ?? readNumberField(raw, 'statusCode')
  const headers = raw.headers ?? raw.header
  const body = raw.body
  const bodyRef = readStringField(raw, 'body_ref') ?? readStringField(raw, 'bodyRef')
  const bodyState = readStringField(raw, 'body_state') ?? readStringField(raw, 'bodyState')
  const meaningfulBodyState = bodyState && bodyState.toLowerCase() !== 'none'
    ? bodyState
    : ''

  if (
    statusCode == null &&
    !hasRenderableValue(headers) &&
    !hasRenderableValue(body) &&
    !bodyRef &&
    !meaningfulBodyState
  ) {
    return null
  }

  const data: Record<string, unknown> = {}
  if (statusCode != null) data.status_code = statusCode
  if (hasRenderableValue(headers)) data.headers = headers
  if (hasRenderableValue(body)) data.body = body
  if (bodyRef) data.body_ref = bodyRef
  if (meaningfulBodyState) data.body_state = meaningfulBodyState

  return data
}

export const extractStringList = (value: unknown): string[] => {
  if (Array.isArray(value)) {
    return value
      .map(item => typeof item === 'string' ? item.trim() : '')
      .filter(Boolean)
  }
  if (typeof value === 'string') {
    const raw = value.trim()
    if (!raw) return []
    try {
      return extractStringList(JSON.parse(raw))
    } catch {
      return [raw]
    }
  }
  return []
}

export const resolveTransportDiagnostics = (attempt: CandidateRecord): Record<string, unknown> | null => {
  const extra = extractObject(attempt.extra_data)
  return extractObject(extra?.transport_diagnostics)
}

export const normalizeQueryString = (value: string): string => {
  const trimmed = value.trim()
  if (!trimmed) return ''
  return trimmed.startsWith('?') ? trimmed.slice(1) : trimmed
}

export const resolveRequestPathFromObject = (value: unknown): string => {
  const object = extractObject(value)
  if (!object) return ''

  const pathWithQuery = (
    readStringField(object, 'request_path_and_query')
    || readStringField(object, 'public_path_and_query')
    || readStringField(object, 'path_and_query')
    || readStringField(object, 'request_uri')
    || readStringField(object, 'public_uri')
  )
  if (pathWithQuery) return pathWithQuery

  const path = (
    readStringField(object, 'request_path')
    || readStringField(object, 'public_path')
    || readStringField(object, 'path')
  )
  if (!path) return ''

  const query = normalizeQueryString(
    readStringField(object, 'request_query_string')
    || readStringField(object, 'public_query_string')
    || readStringField(object, 'query_string')
    || readStringField(object, 'query')
    || '',
  )
  if (!query || path.includes('?')) return path
  return `${path}?${query}`
}

export const isGenericExecutionRuntimeStatusMessage = (message: string): boolean =>
  /execution runtime (stream )?returned non-success status \d+/i.test(message)

export const isStreamTerminalDiagnosticMessage = (message: string): boolean =>
  /(?:unsupported provider stream finish reason|upstream stream ended with finish reason)\s*:/i.test(message)

export const isStreamFinishErrorDiagnostic = (message: string): boolean =>
  /(?:unsupported provider stream finish reason|upstream stream ended with finish reason)\s*:\s*error(?:[ \t]*(?:$|\r?\n)|["')])/i.test(message)

export const isLocalSyncFinalizeDiagnostic = (message: string): boolean =>
  /local sync attempt failed before terminal finalization/i.test(message)
  || /unsupported provider stream (event|finish reason)/i.test(message)

export const isActionableDiagnosticMessage = (message: string): boolean =>
  isLocalSyncFinalizeDiagnostic(message)
  || isStreamTerminalDiagnosticMessage(message)
  || isConversionDiagnosticMessage(message)

export const extractVisibleFailureDiagnostic = (
  extra: Record<string, unknown> | null | undefined,
): Record<string, unknown> | null => {
  const failureDiagnostic = extractObject(extra?.failure_diagnostic)
  if (!failureDiagnostic || failureDiagnostic.safe_to_show === false) return null
  return failureDiagnostic
}

export const extractVisibleDiagnosticObjects = (
  extra: Record<string, unknown> | null | undefined,
): Array<Record<string, unknown>> => [
  ...visibleFailureRecords(extra ?? {}),
]

export const extractVisibleDiagnosticMessage = (
  extra: Record<string, unknown> | null | undefined,
): string => {
  for (const diagnostic of extractVisibleDiagnosticObjects(extra)) {
    const message = readStringField(diagnostic, 'message')
    if (message) return message
  }
  return ''
}

export const chooseAttemptRawErrorMessage = (
  flowMessage: string,
  fallbackMessage: string,
  diagnosticMessage: string,
): string => {
  const flow = flowMessage.trim()
  const fallback = fallbackMessage.trim()
  const diagnostic = diagnosticMessage.trim()

  if (flow && fallback && !isActionableDiagnosticMessage(flow) && isActionableDiagnosticMessage(fallback)) {
    return fallback
  }
  if (flow && diagnostic && isGenericExecutionRuntimeStatusMessage(flow) && isActionableDiagnosticMessage(diagnostic)) {
    return diagnostic
  }
  if (fallback && diagnostic && isGenericExecutionRuntimeStatusMessage(fallback) && isActionableDiagnosticMessage(diagnostic)) {
    return diagnostic
  }

  return flow || fallback || diagnostic
}

export const decodeRustDebugString = (value: string): string => {
  try {
    return JSON.parse(`"${value}"`)
  } catch {
    return value.replace(/\\"/g, '"')
  }
}

export const normalizeDiagnosticFieldPath = (field: string): string => {
  const trimmed = field.trim()
  if (!trimmed || trimmed === '$') return '$'
  if (trimmed.startsWith('$')) return trimmed
  if (trimmed.startsWith('[')) return `$${trimmed}`
  return `$.${trimmed}`
}

export const formatConversionPair = (source: string, target: string): string =>
  `${formatApiFormat(source.trim())} → ${formatApiFormat(target.trim())}`

export const extractFieldDetail = (message: string): string => {
  const fieldMatch = message.match(/field\s+([^;=]+?)\s*=\s*("(?:\\.|[^"\\])*"|[^;]+)/i)
  const unsupportedFieldMatch = message.match(/field\s+([^;]+?)\s+is unsupported/i)
  if (fieldMatch?.[1]) {
    return `字段 ${normalizeDiagnosticFieldPath(fieldMatch[1])} = ${fieldMatch[2].trim()}`
  }
  if (unsupportedFieldMatch?.[1]) {
    return `字段 ${normalizeDiagnosticFieldPath(unsupportedFieldMatch[1])} 不支持`
  }
  return ''
}

export const formatUnsupportedStreamEventMessage = (message: string): string => {
  const detail = extractFieldDetail(message)
  const fieldDetail = detail ? `（${detail}）` : ''
  return `流式格式转换失败：上游返回了当前不支持的 stream event${fieldDetail}，无法无损转换到客户端请求格式`
}

export const formatUnsupportedFinishReasonMessage = (message: string): string => {
  const terminalMatch = message.match(/^unsupported provider stream finish reason\s*:\s*(.+)$/i)
  if (terminalMatch?.[1]) {
    return `流式终态校验失败：上游返回了当前不支持的 finish reason（${terminalMatch[1].trim()}），已按失败处理`
  }
  const fieldDetail = extractFieldDetail(message)
  const legacyMatch = message.match(/unsupported provider stream finish reason\s+(.+?)\s+cannot be converted losslessly/i)
  const detail = fieldDetail || (legacyMatch?.[1]
    ? `字段 $.finish_reason = ${legacyMatch[1].trim()}`
    : '')
  return `流式格式转换失败：上游返回了当前不支持的 finish reason${detail ? `（${detail}）` : ''}，无法无损转换到客户端请求格式`
}

export const formatKnownConversionErrorMessage = (message: string): string => {
  const lossy = message.match(/^lossy conversion blocked from\s+(\S+)\s+to\s+(\S+)\s+at\s+([^:]+):\s*(.+)$/i)
  if (lossy) {
    return `格式转换失败：${formatConversionPair(lossy[1], lossy[2])} 在字段 ${normalizeDiagnosticFieldPath(lossy[3])} 会丢失信息：${lossy[4].trim()}`
  }

  const unaudited = message.match(/^unaudited field\s+(.+?)\s+in\s+(.+?)\s+cannot be converted to\s+(\S+):\s*(.+)$/i)
  if (unaudited) {
    return `格式转换失败：${formatConversionPair(unaudited[2], unaudited[3])} 的字段 ${normalizeDiagnosticFieldPath(unaudited[1])} 尚未审计，不能安全转换：${unaudited[4].trim()}`
  }

  const unsupportedField = message.match(/^unsupported field\s+(.+?)\s+in\s+([^:]+(?::[^:]+)?):\s*(.+)$/i)
  if (unsupportedField) {
    return `格式转换失败：${formatApiFormat(unsupportedField[2])} 不支持字段 ${normalizeDiagnosticFieldPath(unsupportedField[1])}：${unsupportedField[3].trim()}`
  }

  const invalidEnum = message.match(/^invalid enum value\s+(.+?)\s+for\s+([\w:-]+)\.(.+)$/i)
  if (invalidEnum) {
    return `格式转换失败：${formatApiFormat(invalidEnum[2])} 字段 ${normalizeDiagnosticFieldPath(invalidEnum[3])} 的枚举值 ${invalidEnum[1].trim()} 无效`
  }

  const invalidTarget = message.match(/^invalid target field\s+(.+?)\s+for\s+(.+?):\s*(.+)$/i)
  if (invalidTarget) {
    return `格式转换失败：目标格式 ${formatApiFormat(invalidTarget[2])} 字段 ${normalizeDiagnosticFieldPath(invalidTarget[1])} 无效：${invalidTarget[3].trim()}`
  }

  const unsupportedFormat = message.match(/^unsupported AI format:\s*(.+)$/i)
  if (unsupportedFormat) {
    return `格式转换失败：不支持的 API 格式 ${unsupportedFormat[1].trim()}`
  }

  const parseEmit = message.match(/^failed to\s+(parse|emit)\s+(.+?)\s+(request|response)$/i)
  if (parseEmit) {
    const action = parseEmit[1].toLowerCase() === 'parse' ? '解析' : '生成'
    const subject = parseEmit[3].toLowerCase() === 'request' ? '请求体' : '响应体'
    return `格式转换失败：无法${action} ${formatApiFormat(parseEmit[2])} ${subject}`
  }

  return ''
}

export const isConversionDiagnosticMessage = (message: string): boolean => {
  const normalized = message.trim()
  if (!normalized || isStreamTerminalDiagnosticMessage(normalized)) return false
  return /conversion|converted|convertible|cannot be converted|lossy conversion|unsupported field|unaudited field|invalid enum value|invalid target field|unsupported ai format|failed to (parse|emit) .+ (request|response)|unsupported provider stream (event|finish reason)|转换|无损|字段 .*不支持/i
    .test(normalized)
}

export const formatAttemptErrorMessage = (message: string, statusCode?: number): string => {
  const normalized = message.trim()
  if (!normalized) return ''
  const directInternal = normalized.match(/^Internal\("((?:\\.|[^"\\])*)"\)$/i)
  if (directInternal?.[1]) {
    return formatAttemptErrorMessage(decodeRustDebugString(directInternal[1]), statusCode)
  }
  const localSyncInternal = normalized.match(/local sync attempt failed before terminal finalization:\s*Internal\("((?:\\.|[^"\\])*)"\)/i)
  if (localSyncInternal?.[1]) {
    return formatAttemptErrorMessage(decodeRustDebugString(localSyncInternal[1]), statusCode)
  }
  if (isStreamFinishErrorDiagnostic(normalized)) {
    return translate('ui.theUpstreamStreamEndedWithAnErrorFinishReasonError')
  }
  if (/unsupported provider stream event cannot be converted losslessly/i.test(normalized)) {
    return formatUnsupportedStreamEventMessage(normalized)
  }
  if (/unsupported provider stream finish reason/i.test(normalized)) {
    return formatUnsupportedFinishReasonMessage(normalized)
  }
  const conversionMessage = formatKnownConversionErrorMessage(normalized)
  if (conversionMessage) {
    return conversionMessage
  }
  if (isGenericExecutionRuntimeStatusMessage(normalized)) {
    return statusCode != null ? `上游返回非成功状态 ${statusCode}` : '上游返回非成功状态'
  }
  return normalized
}

export const shouldShowAttemptMessageWithUpstreamResponse = (
  rawMessage: string,
  upstreamResponse: Record<string, unknown> | null,
): boolean => {
  if (!upstreamResponse || !hasRenderableValue(upstreamResponse.body)) return true
  const normalized = rawMessage.trim()
  if (!normalized) return false
  return isActionableDiagnosticMessage(normalized)
}

export const diagnosticPathFromObject = (value: unknown): string => {
  const object = extractObject(value)
  if (!object) return ''
  return readStringField(object, 'path')
    || readStringField(object, 'field_path')
    || readStringField(object, 'fieldPath')
    || readStringField(object, 'field')
    || ''
}

// 判断是否为 OAuth 类型（provider_type 为具体值时也算 OAuth）
export const isOAuthType = (authType?: string): boolean => {
  if (!authType) return false
  return !['api_key', 'service_account'].includes(authType)
}

// 格式化认证类型（合并 plan 信息，避免冗余）
export const formatAuthTypeWithPlan = (authType: string, planType?: string): string => {
  const labels: Record<string, string> = {
    'oauth': 'OAuth',
    'service_account': 'Service Account',
    'kiro': 'Kiro',
    'codex': 'Codex',
    'antigravity': 'Antigravity',
    'claude_code': 'Claude Code',
    'gemini_cli': 'Gemini CLI',
  }
  const typeName = labels[authType] || authType
  if (planType) {
    return `${typeName} ${planType}`
  }
  return typeName
}

export const poolSelectionLabel = (reason: string): string => {
  const labels: Record<string, string> = {
    get sticky() { return translate('ui.stickySession') },
    lru: 'LRU',
    get random() { return translate('ui.random') },
    get tiebreak() { return translate('ui.randomEqualWeight') },
  }
  return labels[reason] || reason
}

export const poolSkipLabel = (type: string): string => {
  const labels: Record<string, string> = {
    get cooldown() { return translate('ui.coolingDown') },
    get cost_exhausted() { return translate('ui.quotaExhausted') },
    get upstream() { return translate('ui.upstreamSkipped') },
  }
  return labels[type] || type
}

export const TERMINAL_TIME_RANGE_STATUSES = new Set([
  'success',
  'failed',
  'cancelled',
  'stream_interrupted',
])

export const parseTimestampMs = (value?: string | null): number | null => {
  if (!value) return null
  const timestamp = new Date(value).getTime()
  return Number.isFinite(timestamp) ? timestamp : null
}

export const normalizeLatencyMs = (value?: number | null): number | null => {
  if (typeof value !== 'number' || !Number.isFinite(value) || value < 0) return null
  return Math.round(value)
}

export const formatDurationMs = (durationMs: number): string => {
  const safeDurationMs = Math.max(0, Math.round(durationMs))
  if (safeDurationMs >= 1000) {
    return `${(safeDurationMs / 1000).toFixed(2)}s`
  }
  return `${safeDurationMs}ms`
}

export const resolveAttemptTimeRange = (attempt: CandidateRecord | null | undefined): AttemptTimeRange | null => {
  if (!attempt?.started_at) return null

  const startMs = parseTimestampMs(attempt.started_at)
  if (startMs == null) {
    return {
      startIso: attempt.started_at,
      endIso: attempt.finished_at,
      durationLabel: attempt.finished_at ? formatDuration(attempt.started_at, attempt.finished_at) : undefined,
    }
  }

  const rawEndMs = parseTimestampMs(attempt.finished_at)
  const latencyMs = normalizeLatencyMs(attempt.latency_ms)
  let endMs = rawEndMs

  if (
    latencyMs != null &&
    latencyMs > 0 &&
    (endMs == null || endMs <= startMs) &&
    (attempt.finished_at || TERMINAL_TIME_RANGE_STATUSES.has(attempt.status))
  ) {
    endMs = startMs + latencyMs
  }

  if (endMs == null) {
    return { startIso: attempt.started_at }
  }

  return {
    startIso: attempt.started_at,
    endIso: new Date(endMs).toISOString(),
    durationLabel: formatDurationMs(endMs - startMs),
  }
}

// 格式化时间（详细）
export const formatTime = (dateStr: string) => {
  const date = new Date(dateStr)
  const timeStr = date.toLocaleTimeString(getI18nLocale(), {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
  const ms = date.getMilliseconds().toString().padStart(3, '0')
  return `${timeStr}.${ms}`
}

// 格式化持续时间（开始到结束）
export const formatDuration = (startStr: string, endStr: string): string => {
  const start = new Date(startStr).getTime()
  const end = new Date(endStr).getTime()
  return formatDurationMs(end - start)
}

// 获取状态标签
export const getStatusLabel = (status: string) => {
  const labels: Record<string, string> = {
    get available() { return translate('ui.availableNotAttempted') },
    get unused() { return translate('ui.unusedfa73ed8') },
    get pending() { return translate('ui.inProgress') },
    get streaming() { return translate('ui.transferring') },
    get stream_interrupted() { return translate('ui.streamInterrupted') },
    get success() { return translate('ui.success') },
    get failed() { return translate('ui.failed28384d7') },
    get cancelled() { return translate('ui.canceled') },
    get skipped() { return translate('ui.skip') }
  }
  return labels[status] || status
}

// 获取状态颜色类
export const getStatusColorClass = (status: string) => {
  const classes: Record<string, string> = {
    available: 'status-available',
    unused: 'status-available',
    pending: 'status-pending',
    streaming: 'status-pending',
    stream_interrupted: 'status-failed',
    success: 'status-success',
    failed: 'status-failed',
    cancelled: 'status-cancelled',
    skipped: 'status-skipped'
  }
  return classes[status] || 'status-available'
}

// 展示状态：进行中态优先（包括 started 但未 finished 的中间态），再按 HTTP 状态码兜底
export function getDisplayStatus(attempt: CandidateRecord | null | undefined): string {
  if (!attempt) return 'available'
  const code = attempt.status_code
  const isTerminalSuccessCode = typeof code === 'number' && code >= 200 && code < 300

  if (attempt.status === 'success') {
    if (typeof code === 'number' && !isTerminalSuccessCode) {
      return 'failed'
    }
    return 'success'
  }
  if (
    attempt.status === 'failed' ||
    attempt.status === 'cancelled' ||
    attempt.status === 'skipped' ||
    attempt.status === 'stream_interrupted'
  ) {
    return attempt.status
  }
  const hasFinished = Boolean(attempt.finished_at)
  const isExplicitPending = (attempt.status === 'pending' || attempt.status === 'streaming') && !hasFinished
  const isImplicitPending = Boolean(
    attempt.started_at &&
      !hasFinished &&
      !['failed', 'cancelled', 'skipped', 'stream_interrupted'].includes(attempt.status),
  )

  if (isExplicitPending || isImplicitPending) {
    return 'pending'
  }
  if (typeof code === 'number') {
    if (isTerminalSuccessCode) return 'success'
    if (code >= 300) return 'failed'
  }
  return attempt.status
}
