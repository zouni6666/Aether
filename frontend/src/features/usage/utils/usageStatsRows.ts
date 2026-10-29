import { translate } from '@/i18n'
import { formatApiFormat } from '@/api/endpoints/types/api-format'
import type { ApiFormatStatsItem, EnhancedModelStatsItem, ProviderStatsItem } from '../types'

export interface UsageStatsExtraCell {
  text: string
  class?: string
}

export interface UsageStatsExtraColumn {
  key: string
  header: string
  /** 以次要文字样式渲染整列 */
  muted?: boolean
}

export interface UsageStatsRow {
  key: string
  label: string
  requests: number
  inputTokens: number
  outputTokens: number
  cacheTokens: number
  totalCost: number
  actualCost?: number
  cacheHitRate?: number
  extra: Record<string, UsageStatsExtraCell>
}

export const MODEL_STATS_COLUMNS: UsageStatsExtraColumn[] = [
  { key: 'efficiency', get header() { return translate('ui.efficiency') }, muted: true },
]

export const PROVIDER_STATS_COLUMNS: UsageStatsExtraColumn[] = [
  { key: 'successRate', get header() { return translate('ui.successRate') } },
  { key: 'avgResponseTime', get header() { return translate('ui.averageResponse') }, muted: true },
]

export const API_FORMAT_STATS_COLUMNS: UsageStatsExtraColumn[] = [
  { key: 'avgResponseTime', get header() { return translate('ui.averageResponse') }, muted: true },
]

export function modelStatsRows(items: EnhancedModelStatsItem[]): UsageStatsRow[] {
  return items.map(model => ({
    key: model.model,
    label: model.model.replace('claude-', ''),
    requests: model.request_count,
    inputTokens: model.effective_input_tokens ?? model.total_input_context ?? 0,
    outputTokens: model.output_tokens || 0,
    cacheTokens: model.cache_read_tokens || 0,
    totalCost: model.total_cost,
    actualCost: model.actual_cost,
    cacheHitRate: model.cache_hit_rate,
    extra: {
      efficiency: { text: model.costPerToken },
    },
  }))
}

export function providerSuccessRateClass(rate: number): string {
  return rate < 90 ? 'text-destructive' : ''
}

export function providerStatsRows(items: ProviderStatsItem[]): UsageStatsRow[] {
  return items.map(provider => ({
    key: provider.providerKey ?? provider.providerId ?? provider.provider,
    label: provider.provider,
    requests: provider.requests,
    inputTokens: provider.effectiveInputTokens ?? provider.totalInputContext ?? 0,
    outputTokens: provider.outputTokens || 0,
    cacheTokens: (provider.cacheReadTokens || 0) + (provider.cacheCreationTokens || 0),
    totalCost: provider.totalCost,
    actualCost: provider.actualCost,
    cacheHitRate: provider.cacheHitRate,
    extra: {
      successRate: { text: `${provider.successRate}%`, class: providerSuccessRateClass(provider.successRate) },
      avgResponseTime: { text: provider.avgResponseTime },
    },
  }))
}

export function apiFormatStatsRows(items: ApiFormatStatsItem[]): UsageStatsRow[] {
  return items.map(item => ({
    key: item.api_format,
    label: formatApiFormat(item.api_format),
    requests: item.request_count,
    inputTokens: item.effective_input_tokens ?? item.total_input_context ?? 0,
    outputTokens: item.output_tokens || 0,
    cacheTokens: (item.cache_read_tokens || 0) + (item.cache_creation_tokens || 0),
    totalCost: item.total_cost,
    actualCost: item.actual_cost,
    cacheHitRate: item.cache_hit_rate,
    extra: {
      avgResponseTime: { text: item.avgResponseTime },
    },
  }))
}
