/**
 * 解析 API 错误响应，提取友好的错误信息
 */

import { isApiError } from '@/types/api-error'
import { translate, type MessageKey } from '@/i18n'

/**
 * Pydantic 验证错误项
 */
interface ValidationError {
  loc: (string | number)[]
  msg: string
  type: string
  ctx?: Record<string, unknown>
}

/**
 * 已知字段的消息 key。
 */
const fieldNameMap: Record<string, MessageKey> = {
  'api_key': 'feedback.error.field.apiKey',
  'priority': 'feedback.error.field.priority',
  'rpm_limit': 'feedback.error.field.rpmLimit',
  'rate_limit': 'feedback.error.field.rateLimit',
  'daily_limit': 'feedback.error.field.dailyLimit',
  'monthly_limit': 'feedback.error.field.monthlyLimit',
  'allowed_models': 'feedback.error.field.allowedModels',
  'note': 'feedback.error.field.note',
  'is_active': 'feedback.error.field.isActive',
  'endpoint_id': 'feedback.error.field.endpointId',
  'base_url': 'feedback.error.field.baseUrl',
  'timeout': 'feedback.error.field.timeout',
  'max_retries': 'feedback.error.field.maxRetries',
  'weight': 'feedback.error.field.weight',
  'email': 'feedback.error.field.email',
  'username': 'feedback.error.field.username',
  'password': 'feedback.error.field.password',
  'name': 'feedback.error.field.name',
  'display_name': 'feedback.error.field.displayName',
  'description': 'feedback.error.field.description',
  'website': 'feedback.error.field.website',
  'provider_priority': 'feedback.error.field.providerPriority',
  'billing_type': 'feedback.error.field.billingType',
  'monthly_quota_usd': 'feedback.error.field.monthlyQuota',
  'quota_reset_day': 'feedback.error.field.quotaResetDay',
  'quota_expires_at': 'feedback.error.field.quotaExpiresAt',
  'cache_ttl_minutes': 'feedback.error.field.cacheTtl',
  'max_probe_interval_minutes': 'feedback.error.field.maxProbeInterval',
}

/**
 * 已知验证错误使用本地消息，未知错误保留服务端文案。
 */
const errorTypeMap: Record<string, (error: ValidationError) => string> = {
  'string_too_short': (error) => {
    const minLength = error.ctx?.min_length || 3
    return translate('feedback.error.validation.minLength', { count: String(minLength) })
  },
  'string_too_long': (error) => {
    const maxLength = error.ctx?.max_length
    return translate('feedback.error.validation.maxLength', { count: String(maxLength) })
  },
  'value_error.missing': () => translate('feedback.error.validation.required'),
  'missing': () => translate('feedback.error.validation.required'),
  'type_error.none.not_allowed': () => translate('feedback.error.validation.notNull'),
  'value_error': (error) => error.msg,
  'type_error.integer': () => translate('feedback.error.validation.integer'),
  'type_error.float': () => translate('feedback.error.validation.number'),
  'value_error.number.not_ge': (error) => {
    const limit = error.ctx?.limit_value
    return limit !== undefined ? translate('feedback.error.validation.minInclusive', { limit: String(limit) }) : translate('feedback.error.validation.tooSmall')
  },
  'value_error.number.not_le': (error) => {
    const limit = error.ctx?.limit_value
    return limit !== undefined ? translate('feedback.error.validation.maxInclusive', { limit: String(limit) }) : translate('feedback.error.validation.tooLarge')
  },
  'value_error.number.not_gt': (error) => {
    const limit = error.ctx?.limit_value
    return limit !== undefined ? translate('feedback.error.validation.minExclusive', { limit: String(limit) }) : translate('feedback.error.validation.tooSmall')
  },
  'value_error.number.not_lt': (error) => {
    const limit = error.ctx?.limit_value
    return limit !== undefined ? translate('feedback.error.validation.maxExclusive', { limit: String(limit) }) : translate('feedback.error.validation.tooLarge')
  },
  'less_than_equal': (error) => {
    const limit = error.ctx?.le
    return limit !== undefined ? translate('feedback.error.validation.maxInclusive', { limit: String(limit) }) : translate('feedback.error.validation.tooLarge')
  },
  'greater_than_equal': (error) => {
    const limit = error.ctx?.ge
    return limit !== undefined ? translate('feedback.error.validation.minInclusive', { limit: String(limit) }) : translate('feedback.error.validation.tooSmall')
  },
  'less_than': (error) => {
    const limit = error.ctx?.lt
    return limit !== undefined ? translate('feedback.error.validation.maxExclusive', { limit: String(limit) }) : translate('feedback.error.validation.tooLarge')
  },
  'greater_than': (error) => {
    const limit = error.ctx?.gt
    return limit !== undefined ? translate('feedback.error.validation.minExclusive', { limit: String(limit) }) : translate('feedback.error.validation.tooSmall')
  },
  'value_error.email': () => translate('feedback.error.validation.email'),
  'value_error.url': () => translate('feedback.error.validation.url'),
  'type_error.bool': () => translate('feedback.error.validation.boolean'),
  'type_error.list': () => translate('feedback.error.validation.array'),
  'type_error.dict': () => translate('feedback.error.validation.object'),
}

/**
 * 获取字段名称
 */
function getFieldName(loc: (string | number)[]): string {
  if (!loc || loc.length === 0) return translate('feedback.error.field.default')

  const fieldPath = loc.filter(item => item !== 'body').join('.')
  const fieldKey = String(loc[loc.length - 1])

  const key = fieldNameMap[fieldKey]
  return key ? translate(key) : fieldPath || translate('feedback.error.field.default')
}

/**
 * 格式化单个验证错误
 */
function formatValidationError(error: ValidationError): string {
  const fieldName = getFieldName(error.loc)
  const errorFormatter = errorTypeMap[error.type]

  if (errorFormatter) {
    const errorMsg = errorFormatter(error)
    return `${fieldName}: ${errorMsg}`
  }

  // 默认格式
  return `${fieldName}: ${error.msg}`
}

function normalizeKnownApiErrorMessage(message: string): string {
  const text = message.trim()
  if (!text) return message

  const lowered = text.toLowerCase()
  if (
    lowered.includes('refresh_token_reused')
    || lowered.includes('already been used to generate a new access token')
  ) {
    return translate('feedback.error.refresh.reused')
  }

  if (
    lowered.includes('refresh_token_expired')
    || lowered.includes('could not validate your refresh token')
  ) {
    return translate('feedback.error.refresh.expired')
  }

  const prefix = /^token refresh (?:失败|failed):\s*/i
  if (prefix.test(text)) {
    return translate('feedback.error.refresh.reason', { reason: text.replace(prefix, '') })
  }

  return message
}

/**
 * 解析 API 错误响应
 * @param err 错误对象
 * @param defaultMessage 默认错误信息
 * @returns 格式化的错误信息
 */
export function parseApiError(err: unknown, defaultMessage: string = translate('feedback.error.failed')): string {
  if (!err) return defaultMessage

  // 处理网络错误
  if (!isApiError(err) || !err.response) {
    if (err instanceof Error) {
      return normalizeKnownApiErrorMessage(err.message || defaultMessage)
    }
    return translate('feedback.error.network')
  }

  const data = err.response?.data

  // 1. 处理 {error: {type, message}} 格式（ProxyException 返回格式）
  if (data?.error?.message) {
    return normalizeKnownApiErrorMessage(data.error.message)
  }

  const detail = data?.detail

  // 如果没有 detail 字段
  if (!detail) {
    return normalizeKnownApiErrorMessage(data?.message || err.message || defaultMessage)
  }

  // 1. 处理 Pydantic 验证错误（数组格式）
  if (Array.isArray(detail)) {
    const errors = detail
      .map((error: ValidationError) => formatValidationError(error))
      .join('\n')
    return errors || defaultMessage
  }

  // 2. 处理字符串错误
  if (typeof detail === 'string') {
    return normalizeKnownApiErrorMessage(detail)
  }

  // 3. 处理对象错误
  if (typeof detail === 'object') {
    // 可能是自定义错误对象
    if ((detail as Record<string, unknown>).message) {
      return normalizeKnownApiErrorMessage(String((detail as Record<string, unknown>).message))
    }
    // 尝试 JSON 序列化
    try {
      return JSON.stringify(detail, null, 2)
    } catch {
      return defaultMessage
    }
  }

  return defaultMessage
}

/**
 * 解析并提取第一个错误信息（用于简短提示）
 */
export function parseApiErrorShort(err: unknown, defaultMessage: string = translate('feedback.error.failed')): string {
  const fullError = parseApiError(err, defaultMessage)

  // 如果有多行错误，只取第一行
  const lines = fullError.split('\n')
  return lines[0] || defaultMessage
}

/**
 * 解析模型测试响应的错误信息
 * @param result 测试响应结果
 * @returns 格式化的错误信息
 */
export function parseTestModelError(result: {
  error?: string
  data?: {
    response?: {
      status_code?: number
      error?: string | { message?: string }
    }
  }
}): string {
  let errorMsg = result.error || translate('feedback.error.testFailed')

  // 检查HTTP状态码错误
  if (result.data?.response?.status_code) {
    const status = result.data.response.status_code
    if (status === 403) {
      errorMsg = translate('feedback.error.auth.clientNotAllowed')
    } else if (status === 401) {
      errorMsg = translate('feedback.error.auth.expired')
    } else if (status === 404) {
      errorMsg = translate('feedback.error.modelNotFound')
    } else if (status === 429) {
      errorMsg = translate('feedback.error.rateLimit')
    } else if (status >= 500) {
      errorMsg = translate('feedback.error.serverHttp', { status })
    } else {
      errorMsg = translate('feedback.error.requestHttp', { status })
    }
  }

  // 尝试从错误响应中提取更多信息
  if (result.data?.response?.error) {
    if (typeof result.data.response.error === 'string') {
      errorMsg = result.data.response.error
    } else if (result.data.response.error?.message) {
      errorMsg = result.data.response.error.message
    }
  }

  return errorMsg
}

/**
 * 解析上游模型查询错误信息
 * 将后端返回的原始错误信息（如 "HTTP 401: {json...}"）转换为友好的错误提示
 * @param error 错误字符串，格式可能是 "HTTP {status}: {json_body}" 或其他
 * @returns 友好的错误信息
 */
export function parseUpstreamModelError(error: string): string {
  if (!error) return translate('feedback.error.upstream.load')

  // 匹配 "HTTP {status}: {body}" 格式
  const httpMatch = error.match(/^HTTP\s+(\d+):\s*(.*)$/s)
  if (httpMatch) {
    const status = parseInt(httpMatch[1], 10)
    const body = httpMatch[2]

    // 根据状态码生成友好消息
    let friendlyMsg = ''
    if (status === 401) {
      friendlyMsg = translate('feedback.error.upstream.invalidKey')
    } else if (status === 403) {
      friendlyMsg = translate('feedback.error.upstream.permissions')
    } else if (status === 404) {
      friendlyMsg = translate('feedback.error.upstream.notFound')
    } else if (status === 429) {
      friendlyMsg = translate('feedback.error.upstream.rateLimit')
    } else if (status >= 500) {
      friendlyMsg = translate('feedback.error.upstream.unavailable')
    }

    // 尝试从 JSON body 中提取更详细的错误信息
    if (body) {
      try {
        const parsed = JSON.parse(body)
        // 常见的错误格式: {error: {message: "..."}} 或 {error: "..."} 或 {message: "..."}
        let detailMsg = ''
        if (parsed.error?.message) {
          detailMsg = parsed.error.message
        } else if (typeof parsed.error === 'string') {
          detailMsg = parsed.error
        } else if (parsed.message) {
          detailMsg = parsed.message
        } else if (parsed.detail) {
          detailMsg = typeof parsed.detail === 'string' ? parsed.detail : JSON.stringify(parsed.detail)
        }

        // 如果提取到了详细消息，用它来丰富友好消息
        if (detailMsg) {
          // 检查是否是 token/认证相关的错误
          const lowerMsg = detailMsg.toLowerCase()
          if (lowerMsg.includes('invalid token') || lowerMsg.includes('invalid api key')) {
            return translate('feedback.error.upstream.checkKey')
          }
          if (lowerMsg.includes('expired')) {
            return translate('feedback.error.upstream.expiredKey')
          }
          if (lowerMsg.includes('quota') || lowerMsg.includes('exceeded')) {
            return translate('feedback.error.upstream.quota')
          }
          if (lowerMsg.includes('rate limit')) {
            return translate('feedback.error.upstream.rateLimit')
          }
          // 没有匹配特定关键词，但有详细信息，使用它作为补充
          if (friendlyMsg) {
            const truncated = detailMsg.length > 80 ? `${detailMsg.substring(0, 80)  }...` : detailMsg
            return `${friendlyMsg}: ${truncated}`
          }
          // 没有友好消息，直接使用详细信息
          const truncated = detailMsg.length > 100 ? `${detailMsg.substring(0, 100)  }...` : detailMsg
          return truncated
        }
      } catch {
        // JSON 解析失败，忽略
      }
    }

    // 返回友好消息，附加状态码
    if (friendlyMsg) {
      return friendlyMsg
    }
    return translate('feedback.error.upstream.http', { status })
  }

  // 检查是否是请求错误
  if (error.startsWith('Request error:')) {
    const detail = error.replace('Request error:', '').trim()
    if (detail.toLowerCase().includes('timeout')) {
      return translate('feedback.error.upstream.timeout')
    }
    if (detail.toLowerCase().includes('connection')) {
      return translate('feedback.error.upstream.connection')
    }
    return translate('feedback.error.upstream.network')
  }

  // 检查是否是未知 API 格式
  if (error.startsWith('Unknown API format:')) {
    return translate('feedback.error.upstream.format')
  }

  // 如果包含分号，可能是多个错误合并的，取第一个
  if (error.includes('; ')) {
    const firstError = error.split('; ')[0]
    return parseUpstreamModelError(firstError)
  }

  // 默认返回原始错误（截断过长的部分）
  if (error.length > 100) {
    return `${error.substring(0, 100)  }...`
  }
  return error
}
