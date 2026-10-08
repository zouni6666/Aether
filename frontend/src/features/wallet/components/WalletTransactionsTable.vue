<template>
  <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
    <div
      class="overflow-x-auto"
      :aria-busy="loading"
    >
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>时间</TableHead>
            <TableHead>类型</TableHead>
            <TableHead>金额</TableHead>
            <TableHead>余额变化</TableHead>
            <TableHead>说明</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-if="loading && !items.length">
            <TableCell
              colspan="5"
              class="py-10 text-center text-muted-foreground"
            >
              {{ legacyT('加载中...') }}
            </TableCell>
          </TableRow>
          <TableRow
            v-for="tx in items"
            :key="tx.id"
          >
            <TableCell class="text-xs text-muted-foreground whitespace-nowrap">
              {{ formatDateTime(tx.created_at) }}
            </TableCell>
            <TableCell>
              <div class="space-y-1">
                <Badge
                  variant="outline"
                  class="font-mono"
                >
                  {{ walletTransactionCategoryLabel(tx.category) }}
                </Badge>
                <div class="text-[11px] text-muted-foreground">
                  {{ walletTransactionReasonLabel(tx.reason_code) }}
                </div>
              </div>
            </TableCell>
            <TableCell
              class="tabular-nums"
              :class="toFiniteNumber(tx.amount) >= 0 ? 'text-emerald-600' : 'text-rose-600'"
            >
              {{ toFiniteNumber(tx.amount) >= 0 ? '+' : '' }}{{ formatFixed(tx.amount, 4) }}
            </TableCell>
            <TableCell class="text-xs tabular-nums whitespace-nowrap">
              <div>{{ formatFixed(tx.balance_before, 4) }} → {{ formatFixed(tx.balance_after, 4) }}</div>
              <div
                v-if="tx.recharge_balance_before !== null && tx.recharge_balance_before !== undefined && tx.gift_balance_before !== null && tx.gift_balance_before !== undefined"
                class="text-[11px] text-muted-foreground mt-0.5"
              >
                充 {{ formatFixed(tx.recharge_balance_before, 4) }}→{{ formatFixed(tx.recharge_balance_after, 4) }}
                · 赠 {{ formatFixed(tx.gift_balance_before, 4) }}→{{ formatFixed(tx.gift_balance_after, 4) }}
              </div>
            </TableCell>
            <TableCell class="text-xs text-muted-foreground max-w-[260px] truncate">
              {{ tx.description || '-' }}
            </TableCell>
          </TableRow>
          <TableRow v-if="!loading && items.length === 0">
            <TableCell
              colspan="5"
              class="py-10"
            >
              <EmptyState
                title="暂无资金流水"
                description="当前钱包没有资金动作记录"
              />
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n, getI18nLocale } from '@/i18n'
import { Badge, Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui'
import { EmptyState } from '@/components/common'
import type { WalletTransaction } from '@/api/wallet'
import { walletTransactionCategoryLabel, walletTransactionReasonLabel } from '@/utils/walletDisplay'

const props = defineProps<{ items: WalletTransaction[]; loading?: boolean; timezone?: string }>()
const { legacyT } = useI18n()
function formatDateTime(value?: string | null) {
  if (!value) return '-'
  return new Date(value).toLocaleString(getI18nLocale(), {
    year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit',
    timeZone: props.timezone,
  })
}
function toFiniteNumber(value: unknown) {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : 0
}
function formatFixed(value: unknown, digits: number) {
  return toFiniteNumber(value).toFixed(digits)
}
</script>
