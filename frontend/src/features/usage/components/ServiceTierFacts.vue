<template>
  <dl
    class="grid grid-cols-1 gap-x-4 gap-y-1.5 text-xs"
    :class="hasPriceMultiplier ? 'sm:grid-cols-3' : 'sm:grid-cols-2'"
    data-testid="service-tier-facts"
  >
    <div class="flex min-w-0 items-baseline justify-between gap-3 sm:block">
      <dt class="text-muted-foreground">
        {{ t('ui.upstreamRequestTier') }}
      </dt>
      <dd
        class="truncate font-mono font-medium text-foreground sm:mt-0.5"
        :title="formatServiceTierFact(requested) || '-'"
      >
        {{ formatServiceTierFact(requested) || '-' }}
      </dd>
    </div>
    <div class="flex min-w-0 items-baseline justify-between gap-3 sm:block">
      <dt class="text-muted-foreground">
        {{ t('ui.billingTier') }}
      </dt>
      <dd
        class="truncate font-mono font-medium text-foreground sm:mt-0.5"
        :title="formatServiceTierFact(requested) || '-'"
      >
        {{ formatServiceTierFact(requested) || '-' }}
      </dd>
    </div>
    <div
      v-if="hasPriceMultiplier"
      class="flex min-w-0 items-baseline justify-between gap-3 sm:block"
      data-testid="service-tier-price-multiplier"
    >
      <dt class="text-muted-foreground">
        {{ multiplierTierLabel }} {{ t('ui.multiplier') }}
      </dt>
      <dd class="truncate font-mono font-medium text-foreground sm:mt-0.5">
        {{ formattedPriceMultiplier }}×
      </dd>
    </div>
  </dl>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed } from 'vue'
import { formatServiceTierFact } from '../utils/service-tier'

const props = defineProps<{
  requested: string | null
  priceMultiplier?: number | null
}>()

const { t } = useI18n()

const hasPriceMultiplier = computed(() => (
  typeof props.priceMultiplier === 'number'
  && Number.isFinite(props.priceMultiplier)
  && props.priceMultiplier >= 0
))

const multiplierTierLabel = computed(() => (
  formatServiceTierFact(props.requested) ?? '处理层级'
))

const formattedPriceMultiplier = computed(() => (
  hasPriceMultiplier.value ? String(props.priceMultiplier) : ''
))

</script>
