import type { UsageRecord } from '../types'

export type UsageRecordStreamResolution = Pick<
  UsageRecord,
  'id' | 'is_stream' | 'upstream_is_stream' | 'client_requested_stream' | 'client_is_stream'
>

export function mergeUsageRecordFirstByteTimeMs(
  existingValue: number | null | undefined,
  nextValue: number | null | undefined
): number | null | undefined {
  // Millisecond timing is floored, so 0 still means the first byte was observed.
  const existingIsResolved = typeof existingValue === 'number' &&
    Number.isFinite(existingValue) &&
    existingValue >= 0
  const nextIsResolved = typeof nextValue === 'number' &&
    Number.isFinite(nextValue) &&
    nextValue >= 0

  if (existingIsResolved && nextIsResolved) {
    return Math.max(existingValue, nextValue)
  }
  if (existingIsResolved) {
    return existingValue
  }
  if (nextIsResolved) {
    return nextValue
  }
  return existingValue == null ? existingValue : undefined
}

export function syncUsageRecordStreamResolution(
  records: UsageRecord[],
  resolved: UsageRecordStreamResolution
): UsageRecord[] {
  let changed = false

  const nextRecords = records.map((record) => {
    if (record.id !== resolved.id) {
      return record
    }

    changed = true
    return {
      ...record,
      is_stream: resolved.is_stream,
      upstream_is_stream: typeof resolved.upstream_is_stream === 'boolean'
        ? resolved.upstream_is_stream
        : resolved.is_stream,
      client_requested_stream: typeof resolved.client_requested_stream === 'boolean'
        ? resolved.client_requested_stream
        : resolved.client_is_stream,
      client_is_stream: typeof resolved.client_is_stream === 'boolean'
        ? resolved.client_is_stream
        : resolved.client_requested_stream,
    }
  })

  return changed ? nextRecords : records
}
