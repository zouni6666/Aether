<template>
  <div
    v-if="loading"
    class="rounded-lg border border-dashed border-border/60 bg-muted/20 px-4 py-8 text-center text-sm text-muted-foreground"
    role="status"
  >
    {{ legacyT('正在加载用户套餐...') }}
  </div>
  <div
    v-else-if="!entitlements.length"
    class="rounded-lg border border-dashed border-border/60 bg-muted/20 px-4 py-8 text-center text-sm text-muted-foreground"
  >
    {{ emptyText || text('暂无套餐记录', 'No plan records') }}
  </div>
  <div
    v-else
    class="space-y-2.5"
  >
    <div
      v-for="item in entitlements"
      :key="item.id"
      class="rounded-lg border border-border bg-card/80 p-3"
    >
      <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div class="min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-2">
            <span class="break-words font-medium text-foreground">{{ item.plan_title || item.plan?.title || item.plan_id }}</span>
            <Badge
              :variant="item.active ? 'success' : 'secondary'"
              class="h-5 px-1.5 py-0 text-[10px]"
            >
              {{ statusLabel(item) }}
            </Badge>
          </div>
          <div class="mt-2 flex flex-wrap gap-1.5">
            <Badge
              v-for="(label, index) in labels(item.entitlements)"
              :key="`${label}-${index}`"
              variant="outline"
              class="h-5 px-1.5 py-0 text-[10px]"
            >
              {{ label }}
            </Badge>
          </div>
        </div>
        <div class="flex shrink-0 items-start gap-2">
          <div class="text-left text-[11px] text-muted-foreground sm:text-right">
            <div>{{ legacyT('开始：') }}{{ dateTime(item.starts_at) }}</div>
            <div>{{ legacyT('到期：') }}{{ dateTime(item.expires_at) }}</div>
            <div v-if="showUpdatedAt">
              {{ text('更新：', 'Updated: ') }}{{ dateTime(item.updated_at || item.created_at) }}
            </div>
          </div>
          <slot
            name="actions"
            :item="item"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Badge } from '@/components/ui'
import { useI18n } from '@/i18n'
import type { AdminUserPlanEntitlement } from '@/api/users'
import type { BillingEntitlement } from '@/api/billing'
import { billingEntitlementLabels } from '@/utils/billingEntitlements'

const props = defineProps<{
  entitlements: AdminUserPlanEntitlement[]
  loading?: boolean
  emptyText?: string
  showUpdatedAt?: boolean
  timezone?: string
  formatDateTime?: (value?: string | null) => string
  entitlementLabels?: (items: BillingEntitlement[] | undefined) => string[]
}>()
const { locale, legacyT } = useI18n()
const text = (zh: string, en: string) => locale.value === 'zh-CN' ? zh : en
const labels = (items: BillingEntitlement[]) => props.entitlementLabels?.(items) ?? billingEntitlementLabels(items, legacyT)
function dateTime(value?: string | null) {
  if (props.formatDateTime) return props.formatDateTime(value)
  return value ? new Date(value).toLocaleString(locale.value, {
    year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', timeZone: props.timezone,
  }) : '-'
}
function statusLabel(item: AdminUserPlanEntitlement) {
  if (item.active) return text('生效中', 'Active')
  if (item.status === 'revoked') return text('已撤销', 'Revoked')
  if (item.status === 'replaced') return text('已替换', 'Replaced')
  if (item.status === 'expired' || (item.status === 'active' && item.expires_at && Date.parse(item.expires_at) <= Date.now())) return text('已到期', 'Expired')
  if (item.status === 'active' && item.starts_at && Date.parse(item.starts_at) > Date.now()) return text('待生效', 'Scheduled')
  return item.status
}
</script>
