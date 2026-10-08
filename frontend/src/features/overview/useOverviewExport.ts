import { onScopeDispose, ref } from 'vue'
import { overviewApi, type OverviewQuery } from '@/api/overview'

export function useOverviewExport() {
  const exporting = ref(false)
  const exportError = ref<string | null>(null)
  let controller: AbortController | null = null
  async function exportCsv(path: 'users' | 'consumption' | 'breakdown', query: OverviewQuery) {
    controller?.abort()
    controller = new AbortController()
    exporting.value = true
    exportError.value = null
    try {
      const blob = await overviewApi.exportCsv(path, query, controller.signal)
      const url = URL.createObjectURL(blob)
      const anchor = document.createElement('a')
      anchor.href = url
      anchor.download = `aether-${path}-${query.from.slice(0, 10)}.csv`
      anchor.click()
      setTimeout(() => URL.revokeObjectURL(url), 1000)
    } catch (cause) {
      if (controller.signal.aborted) return
      const response = cause as { response?: { data?: Blob }; message?: string }
      const body = response.response?.data
      if (body instanceof Blob) {
        try {
          const detail = JSON.parse(await body.text()) as { detail?: string; message?: string }
          exportError.value = detail.detail || detail.message || response.message || 'Export failed'
        } catch { exportError.value = response.message || 'Export failed' }
      } else exportError.value = response.message || 'Export failed'
    } finally { exporting.value = false }
  }
  onScopeDispose(() => controller?.abort())
  return { exporting, exportError, exportCsv }
}
