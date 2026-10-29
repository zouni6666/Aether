<template>
  <Card class="p-4 space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-semibold">
        {{ title }}
      </h3>
      <span
        v-if="subtitle"
        class="text-xs text-muted-foreground"
      >{{ subtitle }}</span>
    </div>

    <div
      v-if="loading"
      class="p-4"
    >
      <LoadingState />
    </div>
    <div
      v-else-if="providers.length === 0"
      class="p-4"
    >
      <EmptyState
        :title="t('stats.empty.title')"
        :description="t('ui.noMonthlyPlanQuotaDataAvailable')"
      />
    </div>
    <div
      v-else
      class="space-y-4"
    >
      <div
        v-for="provider in providers"
        :key="provider.id"
        class="space-y-2"
      >
        <div class="flex items-center justify-between text-xs">
          <span class="font-medium">{{ provider.name }}</span>
          <span class="text-muted-foreground">
            {{ formatCurrency(provider.used_usd) }} / {{ formatCurrency(provider.quota_usd) }}
          </span>
        </div>
        <div class="h-2 rounded-full bg-muted">
          <div
            class="h-2 rounded-full bg-primary"
            :style="{ width: `${Math.min(provider.usage_percent, 100)}%` }"
          />
        </div>
        <div class="flex items-center justify-between text-[11px] text-muted-foreground">
          <span>{{ t('ui.remainingd6822b0') }} {{ formatCurrency(provider.remaining_usd) }}</span>
          <span v-if="provider.estimated_exhaust_at"> {{ t('ui.estimatedDepletion') }} {{ formatDateShort(provider.estimated_exhaust_at) }}
          </span>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { Card } from '@/components/ui'
import { EmptyState, LoadingState } from '@/components/common'
import { formatCurrency, formatDateShort } from '@/utils/format'
import type { QuotaUsageProvider } from '@/api/admin'

interface Props {
  title: string
  subtitle?: string
  providers: QuotaUsageProvider[]
  loading?: boolean
}

withDefaults(defineProps<Props>(), {
  subtitle: undefined,
  loading: false
})

const { t } = useI18n()
</script>
