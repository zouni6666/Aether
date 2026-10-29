<template>
  <Card class="overflow-hidden flex flex-col">
    <div class="px-3 py-2 border-b flex-shrink-0">
      <h3 class="text-sm font-medium">
        {{ title }}
      </h3>
    </div>
    <div class="overflow-auto max-h-[320px]">
      <Table class="text-sm">
        <TableHeader>
          <TableRow>
            <TableHead class="h-8 px-2">
              {{ nameHeader }}
            </TableHead>
            <TableHead class="h-8 px-2 text-right">
              {{ t('stats.metric.requests') }}
            </TableHead>
            <TableHead class="h-8 px-2 text-right">
              <div class="flex flex-col text-xs gap-0.5 whitespace-nowrap">
                <span>{{ t('ui.inputOutput') }}</span>
                <span class="text-muted-foreground font-normal">{{ t('ui.cache') }}</span>
              </div>
            </TableHead>
            <TableHead class="h-8 px-2 text-right">
              {{ t('ui.cost7fe8603') }}
            </TableHead>
            <TableHead class="h-8 px-2 text-right">
              {{ t('ui.cacheHitRate') }}
            </TableHead>
            <TableHead
              v-for="column in extraColumns"
              :key="column.key"
              class="h-8 px-2 text-right"
            >
              {{ column.header }}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-if="rows.length === 0">
            <TableCell
              :colspan="5 + extraColumns.length"
              class="text-center py-6 text-muted-foreground px-2"
            >
              {{ emptyText }}
            </TableCell>
          </TableRow>
          <TableRow
            v-for="row in rows"
            :key="row.key"
          >
            <TableCell class="font-medium py-2 px-2">
              {{ row.label }}
            </TableCell>
            <TableCell class="text-right py-2 px-2">
              {{ row.requests }}
            </TableCell>
            <TableCell class="text-right py-2 px-2">
              <div class="flex flex-col items-end text-xs gap-0.5 whitespace-nowrap">
                <span>{{ formatTokens(row.inputTokens) }} / {{ formatTokens(row.outputTokens) }}</span>
                <span class="text-muted-foreground">{{ formatTokens(row.cacheTokens) }}</span>
              </div>
            </TableCell>
            <TableCell class="text-right py-2 px-2">
              <div class="flex flex-col items-end text-xs gap-0.5">
                <span class="text-primary font-medium">{{ formatCurrency(row.totalCost) }}</span>
                <span
                  v-if="isAdmin && row.actualCost !== undefined"
                  class="text-muted-foreground text-[10px]"
                >
                  {{ formatCurrency(row.actualCost) }}
                </span>
              </div>
            </TableCell>
            <TableCell class="text-right py-2 px-2">
              <span>{{ formatHitRate(row.cacheHitRate) }}</span>
            </TableCell>
            <TableCell
              v-for="column in extraColumns"
              :key="column.key"
              class="text-right py-2 px-2"
              :class="column.muted ? 'text-muted-foreground' : ''"
            >
              <span :class="row.extra[column.key]?.class">{{ row.extra[column.key]?.text ?? '-' }}</span>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import Card from '@/components/ui/card.vue'
import Table from '@/components/ui/table.vue'
import TableHeader from '@/components/ui/table-header.vue'
import TableBody from '@/components/ui/table-body.vue'
import TableRow from '@/components/ui/table-row.vue'
import TableHead from '@/components/ui/table-head.vue'
import TableCell from '@/components/ui/table-cell.vue'
import { formatTokens, formatCurrency, formatHitRate } from '@/utils/format'
import type { UsageStatsExtraColumn, UsageStatsRow } from '../utils/usageStatsRows'

/**
 * 用量统计表：模型 / 提供商 / API 格式三种维度共用同一布局，
 * 前五列固定，维度特有的列通过 extraColumns 追加。
 */
withDefaults(defineProps<{
  title: string
  nameHeader: string
  emptyText: string
  rows: UsageStatsRow[]
  isAdmin: boolean
  extraColumns?: UsageStatsExtraColumn[]
}>(), {
  extraColumns: () => [],
})

const { t } = useI18n()
</script>
