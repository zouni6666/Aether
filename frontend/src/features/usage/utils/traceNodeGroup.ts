import { translate } from '@/i18n'
import type { CandidateRecord } from '@/api/requestTrace'

// 节点组类型
export interface TraceNodeGroup {
  id: string
  providerName: string
  primary: CandidateRecord
  primaryStatus: string
  allAttempts: CandidateRecord[]  // 当前展示的尝试（含主节点）
  retryCount: number
  totalLatency: number  // 所有尝试的总延迟
  startIndex: number
  endIndex: number
  hasConversion: boolean  // 组内是否有格式转换候选
  providerApiFormat: string | null  // 提供商 API 格式（如 openai:responses）
  isPoolGroup?: boolean
}

// 用量数据类型
export interface TraceUsageData {
  status?: string | null
  tokens: {
    input: number
    output: number
    cache_creation: number
    cache_read: number
  }
  cost: {
    input: number
    output: number
    cache_creation: number
    cache_read: number
    per_request: number
    total: number
  }
  pricing: {
    input?: number
    output?: number
    cache_creation?: number
    cache_read?: number
    per_request?: number
  }
}

export const AUTH_TYPE_PROVIDER_LABEL_MAP: Record<string, string> = {
  codex: 'Codex',
  kiro: 'Kiro',
  antigravity: 'Antigravity',
  claude_code: 'Claude Code',
  gemini_cli: 'Gemini CLI',
}

export const getProviderDisplayName = (
  attempt: CandidateRecord | null | undefined,
  options: { allowAuthTypeFallback?: boolean } = {},
): string => {
  const allowAuthTypeFallback = options.allowAuthTypeFallback ?? true
  if (!attempt) return translate('chart.unknown')
  const providerName = String(attempt.provider_name || '').trim()
  if (providerName) return providerName
  if (allowAuthTypeFallback) {
    const authType = String(attempt.key_auth_type || '').trim().toLowerCase()
    if (authType && AUTH_TYPE_PROVIDER_LABEL_MAP[authType]) {
      return AUTH_TYPE_PROVIDER_LABEL_MAP[authType]
    }
  }
  return translate('chart.unknown')
}
