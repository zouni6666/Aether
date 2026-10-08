import { onScopeDispose, ref, shallowRef, toValue, watch, type WatchSource } from 'vue'

export function useOverviewRequest<T>(source: WatchSource, fetcher: (signal: AbortSignal) => Promise<T>, options: { scopeKey?: WatchSource<string> } = {}) {
  const data = shallowRef<T | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  let controller: AbortController | null = null
  let generation = 0
  let previousScope: string | undefined
  async function refresh() {
    controller?.abort()
    controller = new AbortController()
    const current = ++generation
    loading.value = true
    error.value = null
    try {
      const result = await fetcher(controller.signal)
      if (current === generation) data.value = result
    } catch (cause) {
      if (current !== generation || controller.signal.aborted) return
      const response = cause as { response?: { data?: { detail?: string; message?: string } }; message?: string }
      error.value = response.response?.data?.detail || response.response?.data?.message || response.message || 'Request failed'
    } finally {
      if (current === generation) loading.value = false
    }
  }
  watch(source, () => {
    const scope = options.scopeKey ? toValue(options.scopeKey) : undefined
    if (!options.scopeKey || scope !== previousScope) data.value = null
    previousScope = scope
    void refresh()
  }, { immediate: true })
  onScopeDispose(() => { generation += 1; controller?.abort() })
  return { data, loading, error, refresh }
}
