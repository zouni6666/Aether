import type { FilterStatusValue, UsageRecord } from '../types'

export function isUserLocalOnlyRecordStatus(status: FilterStatusValue): boolean {
  // 这三个标记都由后端在列表响应里直接给出，但用户侧接口不支持作为服务端筛选条件，
  // 因此统一走前端本地过滤。
  return status === 'has_retry'
    || status === 'has_fallback'
    || status === 'has_skipped_candidate'
}

export function shouldUseServerUserRecordFilters(input: {
  search: string
  apiFormat: string
  status: FilterStatusValue
}): boolean {
  if (isUserLocalOnlyRecordStatus(input.status)) return false

  return input.search.trim().length > 0
    || input.apiFormat !== '__all__'
    || input.status !== '__all__'
}

export function isUnknownUsageLabel(value: unknown): boolean {
  if (typeof value !== 'string') return false
  const normalized = value.trim().toLowerCase()
  return normalized === 'unknown' || normalized === 'unknow'
}

export function hasUnknownModelOrProvider(record: UsageRecord): boolean {
  if (isUnknownUsageLabel(record.model)) return true
  if (isUnknownUsageLabel(record.provider)) return true
  return false
}
