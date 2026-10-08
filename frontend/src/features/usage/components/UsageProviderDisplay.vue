<template>
  <div class="flex min-w-0 flex-col gap-0.5">
    <span
      data-usage-provider="routing-group"
      class="truncate text-foreground"
      :title="groupLabel"
    >{{ groupLabel }}</span>
    <span
      data-usage-provider="provider-key"
      class="truncate text-muted-foreground"
      :title="providerKeyLabel"
    >{{ providerKeyLabel }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { UsageRecord } from '../types'

const props = defineProps<{ record: UsageRecord }>()
const groupLabel = computed(() => props.record.routing_group_name?.trim()
  || props.record.routing_group_id?.trim() || '未记录分组')
const providerKeyLabel = computed(() => `${props.record.provider?.trim() || '-'} · ${props.record.provider_key_name?.trim() || '-'}`)
</script>
