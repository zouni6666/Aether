<template>
  <Card
    class="min-w-0 overflow-hidden p-3 sm:p-4"
    :data-operations-metric="label"
  >
    <div class="flex h-5 min-w-0 items-center justify-between gap-2">
      <p
        class="truncate text-xs font-semibold text-muted-foreground"
        :title="label"
      >
        {{ label }}
      </p>
      <component
        :is="icon"
        class="h-4 w-4 shrink-0 text-muted-foreground"
        aria-hidden="true"
      />
    </div>
    <div class="mt-2 flex h-9 min-w-0 items-center">
      <MetricValue
        :value="value"
        :max-font-size="30"
        class="w-full font-semibold leading-none tabular-nums text-foreground"
      />
    </div>
    <p
      v-if="note"
      class="mt-1 truncate text-xs leading-5 text-muted-foreground"
      :title="note"
    >
      {{ note }}
    </p>
  </Card>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { Card } from '@/components/ui'
import MetricValue from '../components/MetricValue.vue'

defineProps<{ label: string; value: string; icon: Component; note?: string }>()
</script>
