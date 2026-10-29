import type { ImageProgress } from '@/api/requestTrace'
import type { RequestStatus } from '../types'
import { normalizeRequestStatus } from './status'

// 请求详情抽屉回传的状态更新与列表记录之间的比较与归一化

export function sameImageProgress(left?: ImageProgress | null, right?: ImageProgress | null): boolean {
  if (!left && !right) return true
  if (!left || !right) return false
  return left.phase === right.phase &&
    left.upstream_ttfb_ms === right.upstream_ttfb_ms &&
    left.upstream_sse_frame_count === right.upstream_sse_frame_count &&
    left.last_upstream_event === right.last_upstream_event &&
    left.last_upstream_frame_at_unix_ms === right.last_upstream_frame_at_unix_ms &&
    left.partial_image_count === right.partial_image_count &&
    left.last_client_visible_event === right.last_client_visible_event &&
    left.downstream_heartbeat_count === right.downstream_heartbeat_count &&
    left.last_downstream_heartbeat_at_unix_ms === right.last_downstream_heartbeat_at_unix_ms &&
    left.downstream_heartbeat_interval_ms === right.downstream_heartbeat_interval_ms
}

export function resolveDetailUpdateStatus(update: {
  status?: RequestStatus
  statusCode?: number | null
  imageProgress?: ImageProgress | null
  errorMessage?: string | null
}): RequestStatus | undefined {
  const status = normalizeRequestStatus(update.status)
  const hasFailureSignal =
    (typeof update.statusCode === 'number' && update.statusCode >= 400) ||
    (typeof update.errorMessage === 'string' && update.errorMessage.trim().length > 0) ||
    update.imageProgress?.phase === 'failed'

  if ((status == null || status === 'pending' || status === 'streaming') && hasFailureSignal) {
    return 'failed'
  }
  return status
}
