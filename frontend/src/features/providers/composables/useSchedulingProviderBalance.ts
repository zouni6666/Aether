import { inject, onScopeDispose, provide, watch, type InjectionKey } from 'vue'
import type { ProviderWithEndpointsSummary } from '@/api/endpoints'
import { useProviderBalance } from './useProviderBalance'

type BalanceProvider = Pick<ProviderWithEndpointsSummary, 'id' | 'ops_configured'>
type SchedulingBalance = ReturnType<typeof useProviderBalance> & {
  register: (provider: BalanceProvider) => void
}

const schedulingBalanceKey: InjectionKey<SchedulingBalance> = Symbol('scheduling-provider-balance')

/** Share one balance request/cache across the model configurations in this workspace. */
export function provideSchedulingProviderBalance(revision?: () => number) {
  const balance = useProviderBalance()
  const providers = new Map<string, BalanceProvider>()
  let timer: ReturnType<typeof setTimeout> | undefined
  let refreshRequested = false
  let loading = false
  let disposed = false

  async function flush() {
    timer = undefined
    if (loading || disposed || !refreshRequested) return
    refreshRequested = false
    loading = true
    try {
      // loadBalances invalidates older retries on each call. Include every registered
      // provider so a newly mounted model panel cannot strand an older pending balance.
      await balance.loadBalances([...providers.values()], false)
    } finally {
      loading = false
      // A provider can lose its ops configuration while its request is in flight.
      for (const id of Object.keys(balance.balanceCache.value)) {
        if (!providers.has(id)) delete balance.balanceCache.value[id]
      }
      if (!disposed && refreshRequested) schedule()
    }
  }

  function schedule() {
    if (timer === undefined && !loading && !disposed) timer = setTimeout(() => { void flush() }, 80)
  }

  function register(provider: BalanceProvider) {
    if (disposed) return
    if (!provider.ops_configured) {
      if (!providers.delete(provider.id)) return
      delete balance.balanceCache.value[provider.id]
    } else {
      if (providers.has(provider.id)) return
      providers.set(provider.id, { id: provider.id, ops_configured: true })
    }
    refreshRequested = true
    schedule()
  }

  if (revision) {
    watch(revision, () => {
      if (providers.size === 0) return
      refreshRequested = true
      schedule()
    })
  }
  onScopeDispose(() => {
    disposed = true
    if (timer !== undefined) clearTimeout(timer)
    providers.clear()
  })
  provide(schedulingBalanceKey, { ...balance, register })
}

export function useSchedulingProviderBalance() {
  return inject(schedulingBalanceKey, null)
}
