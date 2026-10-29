<template>
  <div class="space-y-4 rounded-2xl border bg-background p-4 shadow-sm">
    <div class="flex items-start gap-3">
      <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary">
        <ShieldCheck class="h-4 w-4" />
      </div>
      <div class="min-w-0 space-y-1">
        <h4 class="text-sm font-semibold text-foreground">
          {{ t('ui.batchSetQuota') }}
        </h4>
        <p class="text-xs leading-relaxed text-muted-foreground">
          {{ t('ui.quotaRemainsAUserAccountPropertyManageModelsEndpointsProviders') }}
        </p>
      </div>
    </div>

    <div class="rounded-xl border border-border/70 bg-muted/20 p-3">
      <div class="space-y-2">
        <div>
          <Label class="text-sm font-medium">{{ t('poolQuota.claudeCode.unknownWindow') }}</Label>
          <p class="mt-1 text-[11px] text-muted-foreground">
            {{ t('ui.appliesToAllTargetUsers') }}
          </p>
        </div>
        <Select
          :model-value="modelValue"
          @update:model-value="($event === 'skip' || $event === 'wallet' || $event === 'unlimited') && $emit('update:modelValue', $event)"
        >
          <SelectTrigger class="h-9 w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="skip">
              {{ t('ui.doNotChange') }}
            </SelectItem>
            <SelectItem value="wallet">
              {{ t('ui.limitedByWalletBalance') }}
            </SelectItem>
            <SelectItem value="unlimited">
              {{ t('ui.unlimitedQuota') }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Label, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui'
import { ShieldCheck } from 'lucide-vue-next'
import { useI18n } from '@/i18n'
import type { UserBatchQuotaMode } from './user-management-types'

defineProps<{
  modelValue: UserBatchQuotaMode
}>()

defineEmits<{
  'update:modelValue': [value: UserBatchQuotaMode]
}>()

const { t } = useI18n()
</script>
