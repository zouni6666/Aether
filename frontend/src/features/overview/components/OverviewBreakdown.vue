<template>
  <section class="min-w-0 space-y-3">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <h2 class="text-sm font-semibold">
        {{ title }}
      </h2><slot />
    </div>
    <div class="overflow-x-auto">
      <table class="w-full text-sm">
        <thead class="border-b text-xs text-muted-foreground">
          <tr>
            <th class="py-3 text-left font-medium">
              {{ t('维度', 'Dimension') }}
            </th><th class="px-3 py-3 text-right font-medium">
              {{ t('请求数', 'Requests') }}
            </th><th class="px-3 py-3 text-right font-medium">
              Tokens
            </th><th class="px-3 py-3 text-right font-medium">
              {{ t('请求成功率', 'Request success') }}
            </th><th class="py-3 text-right font-medium">
              {{ t('计费消耗', 'Billable consumption') }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="item in items"
            :key="item.id ?? '__unknown__'"
            class="border-b last:border-0"
          >
            <td class="max-w-64 break-words py-3">
              <button
                v-if="item.id"
                class="text-left hover:text-primary hover:underline"
                @click="$emit('select', item)"
              >
                {{ item.label || item.id || t('未知', 'Unknown') }}
              </button>
              <span v-else>{{ item.label || t('未知', 'Unknown') }}</span>
            </td><td class="whitespace-nowrap px-3 py-3 text-right tabular-nums">
              {{ count(item.request_count) }}
            </td><td class="whitespace-nowrap px-3 py-3 text-right tabular-nums">
              {{ count(item.total_tokens) }}
            </td><td class="whitespace-nowrap px-3 py-3 text-right tabular-nums">
              {{ percent(item.success_rate?.value) }}
            </td><td
              class="whitespace-nowrap py-3 text-right tabular-nums"
              :title="item.billable_amount?.value ?? ''"
            >
              {{ money(item.billable_amount) }}
            </td>
          </tr><tr v-if="!items.length">
            <td
              colspan="5"
              class="py-10 text-center text-muted-foreground"
            >
              {{ t('此时间范围暂无数据', 'No data in this range') }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { OverviewBreakdown } from '@/api/overview'
import { count, money, percent } from '../format'
import { useOverviewI18n } from '../i18n'
defineProps<{ title: string; items: OverviewBreakdown[] }>()
defineEmits<{ select: [item: OverviewBreakdown] }>()
const { t } = useOverviewI18n()
</script>
