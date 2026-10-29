import { formatTokens } from '@/utils/format'
import { formatApiFormat } from '@/api/endpoints/types/api-format'
import { getCacheCreationTokens, getCacheReadTokens, getEffectiveInputTokens } from '../token-normalization'
import { formatOutputRateValue, getDisplayOutputRate, getGenerationTimeMs } from '../performance'
import {
  formatUsageStreamLabel,
  isUsageUpstreamStream,
  resolveDisplayRequestStatus,
  resolveUsageStreamLabelSegments,
} from './status'
import { formatServiceTierFact } from './service-tier'
import { formatCandidateSkipReason } from './skipReason'
import { isCyberPolicyError } from './cyberError'
import type { UsageRecord } from '../types'

// 使用记录列表（桌面表格与移动卡片共用）的纯展示辅助函数

export function getDisplayStatus(record: UsageRecord) {
  return resolveDisplayRequestStatus(record)
}

export function getStreamModeLabel(record: UsageRecord): string {
  return formatUsageStreamLabel(record)
}

export function getStreamModeSegments(record: UsageRecord) {
  return resolveUsageStreamLabelSegments(record)
}

export function getUpstreamStream(record: UsageRecord): boolean {
  return isUsageUpstreamStream(record)
}

export function parseRecordDateTime(dateStr: string): Date {
  const utcDateStr = dateStr.includes('Z') || dateStr.includes('+') ? dateStr : `${dateStr}Z`
  return new Date(utcDateStr)
}

export function formatRecordDate(dateStr: string): string {
  const date = parseRecordDateTime(dateStr)
  const year = String(date.getFullYear())
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

export function formatRecordShortDate(dateStr: string): string {
  const date = parseRecordDateTime(dateStr)
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${month}-${day}`
}

export function formatRecordTime(dateStr: string): string {
  const date = parseRecordDateTime(dateStr)
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  const seconds = String(date.getSeconds()).padStart(2, '0')
  return `${hours}:${minutes}:${seconds}`
}

export function getRecordUserName(record: UsageRecord): string {
  return record.username || record.user_email || (record.user_id ? `User ${record.user_id}` : '已删除用户')
}

export function formatRecordUserSegment(record: UsageRecord): string {
  return `${getRecordUserName(record)} / ${record.api_key?.name || '-'}`
}

export function getRecordEffectiveInputTokens(record: UsageRecord): number {
  return getEffectiveInputTokens(record)
}

export function getRecordCacheReadTokens(record: UsageRecord): number {
  return getCacheReadTokens(record)
}

export function getRecordCacheCreationTokens(record: UsageRecord): number {
  return getCacheCreationTokens(record)
}

export function hasPositiveTokens(value: number | null | undefined): boolean {
  return typeof value === 'number' && Number.isFinite(value) && value > 0
}

export function formatOptionalTokens(value: number | null | undefined): string {
  return hasPositiveTokens(value) ? formatTokens(value) : '-'
}

export function hasRecordCacheTokens(record: UsageRecord): boolean {
  return hasPositiveTokens(getRecordCacheReadTokens(record)) || hasPositiveTokens(getRecordCacheCreationTokens(record))
}

export function getRecordCacheTokensTitle(record: UsageRecord): string {
  return [
    `缓存读取: ${formatOptionalTokens(getRecordCacheReadTokens(record))}`,
    `缓存写入: ${formatOptionalTokens(getRecordCacheCreationTokens(record))}`,
  ].join('\n')
}

export function formatRecordLatencyPair(record: UsageRecord): string {
  const firstByte = formatRecordDurationSeconds(
    record.end_to_end_first_byte_time_ms ?? record.first_byte_time_ms,
  )
  const total = formatRecordDurationSeconds(record.end_to_end_time_ms ?? record.response_time_ms)
  return `${firstByte} / ${total}`
}

export function formatRecordDurationSeconds(ms: number | null | undefined): string {
  if (ms == null || !Number.isFinite(ms)) return '-'
  return `${(ms / 1000).toFixed(2)}s`
}

export function hasRecordDisplayLatency(record: UsageRecord): boolean {
  return record.end_to_end_time_ms != null
    || record.end_to_end_first_byte_time_ms != null
    || record.response_time_ms != null
    || record.first_byte_time_ms != null
}

export function getRecordDisplayOutputRate(record: UsageRecord): number | null {
  return getDisplayOutputRate({
    output_tokens: record.output_tokens,
    response_time_ms: record.response_time_ms,
    first_byte_time_ms: record.first_byte_time_ms,
    is_stream: record.is_stream,
    upstream_is_stream: record.upstream_is_stream,
  })
}

export function getRecordPerformanceTitle(record: UsageRecord): string {
  const outputRate = getRecordDisplayOutputRate(record)
  return [
    `端到端首字: ${formatRecordDurationSeconds(record.end_to_end_first_byte_time_ms ?? record.first_byte_time_ms)}`,
    `端到端总耗时: ${formatRecordDurationSeconds(record.end_to_end_time_ms ?? record.response_time_ms)}`,
    `成功候选首字: ${formatRecordDurationSeconds(record.first_byte_time_ms)}`,
    `成功候选耗时: ${formatRecordDurationSeconds(record.response_time_ms)}`,
    `生成耗时: ${formatRecordDurationSeconds(getGenerationTimeMs(record))}`,
    `输出速度: ${formatOutputRateTokensPerSecond(outputRate)}`,
  ].join('\n')
}

export function formatOutputRateTokensPerSecond(outputRate: number | null | undefined): string {
  const value = formatOutputRateValue(outputRate)
  if (value === '-') return value
  return `${value} tokens/s`
}

export function formatUserAgent(value: string | null | undefined): string {
  const userAgent = value?.trim()
  if (!userAgent) return '-'
  return userAgent.length > 48 ? `${userAgent.slice(0, 45)}...` : userAgent
}

// 判断是否应该显示格式转换信息
// 包括：1. 跨格式转换（has_format_conversion=true）2. 同族格式差异
export function shouldShowFormatConversion(record: UsageRecord): boolean {
  if (!record.api_format || !record.endpoint_api_format) {
    return false
  }
  // 跨格式转换
  if (record.has_format_conversion) {
    return true
  }
  // 同族格式差异（精确字符串比较，不区分大小写）
  return record.api_format.trim().toLowerCase() !== record.endpoint_api_format.trim().toLowerCase()
}

// 被跳过候选的提示文案：把"调度阶段排除"和"上游失败"讲清楚，
// 避免用户把"无报错换提供商"误判成调度 bug 或提供商静默出错。
export function skippedCandidateTooltip(record: UsageRecord): string {
  const reasons = (record.skipped_candidate_reasons ?? [])
    .map(formatCandidateSkipReason)
    .filter((reason, index, all) => reason && all.indexOf(reason) === index)

  const header = '本次有候选在调度阶段被跳过，请求未发往该候选（因此不会有上游报错）'
  if (!reasons.length) return header
  return `${header}\n跳过原因：${reasons.join('；')}`
}

export function skippedCandidateAriaLabel(record: UsageRecord): string {
  const reasons = (record.skipped_candidate_reasons ?? []).map(formatCandidateSkipReason)
  return reasons.length ? `有候选被调度跳过：${reasons.join('；')}` : '有候选被调度跳过'
}

export function getApiFormatTooltip(record: UsageRecord): string {
  if (!record.api_format) {
    return ''
  }
  const displayFormat = formatApiFormat(record.api_format)

  // 如果发生了格式转换或同族格式差异，显示详细信息
  if (shouldShowFormatConversion(record)) {
    const endpointApiFormat = record.endpoint_api_format ?? record.api_format
    const endpointDisplayFormat = formatApiFormat(endpointApiFormat)
    const conversionType = record.has_format_conversion ? '格式转换' : '格式兼容（无需转换）'
    return `用户请求格式: ${displayFormat}\n端点原生格式: ${endpointDisplayFormat}\n${conversionType}`
  }

  return displayFormat
}

export function getReasoningEffort(record: UsageRecord): string | null {
  const requested = record.requested_reasoning_effort?.trim()
  const actual = record.reasoning_effort?.trim()
  if (requested && actual && requested.toLowerCase() !== actual.toLowerCase()) {
    return `${requested} -> ${actual}`
  }
  return actual || requested || null
}

export function hasCyberPolicyError(record: UsageRecord): boolean {
  return isCyberPolicyError(record.error_message)
}

export interface ServiceTierBadgePresentation {
  label: string
  className: string
  title: string
  ariaLabel: string
}

export function normalizeServiceTier(value: string | null | undefined): string | null {
  const serviceTier = value?.trim().toLowerCase()
  return serviceTier || null
}

export function canonicalServiceTier(value: string | null): string | null {
  if (value === 'auto' || value === 'default' || value === 'standard') {
    return 'standard'
  }
  if (value === 'fast') {
    return 'priority'
  }
  return value
}

export function buildServiceTierBadgePresentation(
  requestedRaw: string | null,
): ServiceTierBadgePresentation {
  const titleLines: string[] = []
  const requestedLabel = formatServiceTierFact(requestedRaw)
  if (requestedLabel) titleLines.push(`上游请求档位：${requestedLabel}`)
  // Billing is resolved from the same final provider request tier. Keep it
  // explicit in the tooltip without consulting a response-side tier.
  if (requestedLabel) titleLines.push(`计费档位：${requestedLabel}`)
  const title = titleLines.join('\n')
  return {
    label: 'Fast',
    className: '!bg-transparent text-blue-500 dark:text-blue-300',
    title,
    ariaLabel: titleLines.join('，'),
  }
}

export function getServiceTierBadge(record: UsageRecord): ServiceTierBadgePresentation | null {
  const requestedRaw = normalizeServiceTier(record.service_tier)
  const requested = canonicalServiceTier(requestedRaw)
  const requestedFast = requested === 'priority'
  if (!requestedFast) return null
  return buildServiceTierBadgePresentation(requestedRaw)
}

export function getServiceTierTitle(record: UsageRecord): string {
  const badge = getServiceTierBadge(record)
  if (badge) return badge.title

  const requested = formatServiceTierFact(record.service_tier)
  return [
    requested ? `上游请求档位：${requested}` : null,
    requested ? `计费档位：${requested}` : null,
  ].filter((line): line is string => Boolean(line)).join('\n')
}

// 获取模型列的 tooltip
export function getModelTooltip(record: UsageRecord): string {
  const reasoningEffort = getReasoningEffort(record)
  const serviceTierTitle = getServiceTierTitle(record)
  const tierSuffix = serviceTierTitle ? `\n${serviceTierTitle}` : ''
  const cyberSuffix = hasCyberPolicyError(record) ? '\nCyber Policy: blocked' : ''
  const suffix = `${reasoningEffort ? `\nReasoning: ${reasoningEffort}` : ''}${tierSuffix}${cyberSuffix}`
  const requestModel = record.model.trim()
  const mappingModel = record.target_model?.trim()
  const responseModel = record.response_model?.trim()
  return [
    requestModel,
    mappingModel && mappingModel !== requestModel ? `映射模型: ${mappingModel}` : null,
    responseModel && responseModel !== requestModel ? `响应模型: ${responseModel}` : null,
  ].filter((line): line is string => Boolean(line)).join('\n') + suffix
}
