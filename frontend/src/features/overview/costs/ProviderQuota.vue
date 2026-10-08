<template>
  <section class="space-y-3 border-t pt-5">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
        <h2 class="text-sm font-semibold">
          {{ t('提供商套餐额度', 'Provider plan quota') }}
        </h2>
        <span class="text-xs text-muted-foreground">{{ t('当前配置 · 全站', 'Current configuration · Site-wide') }}</span>
      </div>
      <Button
        variant="ghost"
        size="icon"
        class="h-8 w-8"
        :disabled="loading"
        :title="t('刷新额度', 'Refresh quota')"
        :aria-label="t('刷新额度', 'Refresh quota')"
        @click="refresh"
      >
        <RefreshCw class="h-3.5 w-3.5" />
      </Button>
    </div>
    <OverviewStatus
      :loading="loading"
      :error="error"
      @retry="refresh"
    />
    <div
      v-if="data?.providers.length"
      class="overflow-x-auto"
    >
      <table class="w-full text-sm">
        <thead class="border-y text-xs text-muted-foreground">
          <tr>
            <th class="py-3 text-left">
              {{ t('提供商', 'Provider') }}
            </th>
            <th class="px-3 py-3 text-right">
              {{ t('已使用 / 额度', 'Used / Quota') }}
            </th>
            <th class="px-3 py-3 text-right">
              {{ t('剩余', 'Remaining') }}
            </th>
            <th class="min-w-32 px-3 py-3 text-left">
              {{ t('使用进度', 'Utilization') }}
            </th>
            <th class="px-3 py-3 text-left">
              {{ t('到期时间', 'Expires') }}
            </th>
            <th class="px-3 py-3 text-left">
              {{ t('预计耗尽', 'Estimated depletion') }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="provider in data.providers"
            :key="provider.id"
            class="border-b"
          >
            <td class="max-w-56 break-words py-3 font-medium">
              {{ provider.name }}
            </td>
            <td class="whitespace-nowrap px-3 py-3 text-right tabular-nums">
              {{ usd(provider.used_usd) }} / {{ usd(provider.quota_usd) }}
            </td>
            <td class="whitespace-nowrap px-3 py-3 text-right tabular-nums">
              {{ usd(provider.remaining_usd) }}
            </td>
            <td class="px-3 py-3">
              <span class="text-xs tabular-nums">{{ percent(provider.usage_percent / 100) }}</span><div class="mt-1 h-1.5 w-24 overflow-hidden rounded bg-muted">
                <div
                  class="h-full"
                  :class="provider.usage_percent >= 90 ? 'bg-amber-500' : 'bg-emerald-600'"
                  :style="{ width: `${Math.min(100, Math.max(0, provider.usage_percent))}%` }"
                />
              </div>
            </td>
            <td class="whitespace-nowrap px-3 py-3 text-xs">
              {{ timestamp(provider.quota_expires_at, timezone) }}
            </td>
            <td class="whitespace-nowrap px-3 py-3 text-xs">
              {{ timestamp(provider.estimated_exhaust_at, timezone) }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <p
      v-else-if="data"
      class="py-6 text-center text-sm text-muted-foreground"
    >
      {{ t('暂无已配置的提供商额度', 'No provider quota configured') }}
    </p>
    <p
      v-if="data"
      class="text-xs text-muted-foreground"
    >
      {{ t('读取时间', 'Retrieved') }} {{ timestamp(data.retrievedAt, timezone) }}
    </p>
  </section>
</template>

<script setup lang="ts">
import { RefreshCw } from 'lucide-vue-next'
import { adminApi } from '@/api/admin'
import { Button } from '@/components/ui'
import OverviewStatus from '../components/OverviewStatus.vue'
import { useOverviewRequest } from '../useOverviewRequest'
import { useOverviewI18n } from '../i18n'
import { money, percent, timestamp } from '../format'
defineProps<{ timezone: string }>()
const { t } = useOverviewI18n()
const { data, loading, error, refresh } = useOverviewRequest(() => 'provider-quota', async () => ({ ...await adminApi.getQuotaUsage(), retrievedAt: new Date().toISOString() }))
function usd(value: number) { return money({ value: String(value), currency: 'USD', basis: 'provider_plan', status: 'known' }) }
</script>
