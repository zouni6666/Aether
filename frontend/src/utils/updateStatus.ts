import { translate } from '@/i18n'
import type { CheckUpdateResponse } from '@/api/admin'

export function describeUpdateStatus(status: CheckUpdateResponse | null): string {
  if (!status) return translate('ui.checking')
  if (status.has_update) return translate('ui.newVersionAvailable')
  if (status.error) return translate('ui.checkFailed')
  return translate('ui.upToDate')
}

export function buildUpdateErrorStatus(
  previousStatus: CheckUpdateResponse | null,
  error: unknown
): CheckUpdateResponse {
  return {
    current_version: previousStatus?.current_version || '',
    latest_version: null,
    has_update: false,
    updatable: false,
    update_blocker: null,
    release_url: null,
    release_notes: null,
    published_at: null,
    error: error instanceof Error ? error.message : '检查更新失败'
  }
}
