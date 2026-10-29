<template>
  <div :class="mobile ? 'flex flex-wrap items-center gap-1.5' : 'flex flex-col items-start gap-1.5'">
    <Badge
      :variant="row.statusVariant"
      class="h-5 px-1.5 py-0 text-[10px] font-medium"
    >
      {{ row.statusLabel }}
    </Badge>
    <Badge
      v-if="row.hasWallet"
      :variant="row.walletStatusVariant"
      class="h-5 px-1.5 py-0 text-[10px] font-medium"
    >
      {{ row.walletStatusLabel }}
    </Badge>
    <Badge
      v-if="mobile"
      variant="secondary"
      class="h-5 px-1.5 py-0 text-[10px] font-medium"
      :title="row.rateLimitSource"
    >
      {{ row.rateLimitLabel }}
    </Badge>
    <Badge
      v-for="group in mobile ? row.user.groups || [] : []"
      :key="group.id"
      variant="outline"
      class="h-5 px-1.5 py-0 text-[10px] font-medium"
    >
      {{ group.name }}
    </Badge>
  </div>
</template>

<script setup lang="ts">
import Badge from '@/components/ui/badge.vue'
import type { UserManagementRow } from './user-management-types'

withDefaults(defineProps<{
  row: UserManagementRow
  mobile?: boolean
}>(), {
  mobile: false,
})

</script>
