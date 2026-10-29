<template>
  <Popover
    :open="open"
    @update:open="(value: boolean) => $emit('update:open', value)"
  >
    <PopoverTrigger as-child>
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7"
        :class="[nodeId ? 'text-blue-500' : '', triggerClass]"
        :disabled="saving"
        :title="nodeId ? t('ui.proxyValue', { name: nodeName ?? '' }) : t('ui.setProxyNode')"
        @click.stop
      >
        <Globe class="w-3.5 h-3.5" />
      </Button>
    </PopoverTrigger>
    <PopoverContent
      class="w-72 p-3"
      side="bottom"
      align="end"
    >
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-xs font-medium">{{ t('ui.proxyNodes23f1b6a') }}</span>
          <Button
            v-if="nodeId"
            variant="ghost"
            size="sm"
            class="h-6 px-2 text-[10px] text-muted-foreground"
            :disabled="saving"
            @click="$emit('clear')"
          >
            {{ t('ui.clear') }}
          </Button>
        </div>
        <ProxyNodeSelect
          :model-value="nodeId"
          trigger-class="h-8"
          @update:model-value="(value: string) => $emit('select', value)"
        />
        <p class="text-[10px] text-muted-foreground">
          {{ nodeId ? t('ui.currentlyUsingAnIndividualProxy') : t('ui.notSetUsesProviderLevelProxy') }}
        </p>
      </div>
    </PopoverContent>
  </Popover>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { Globe } from 'lucide-vue-next'
import { Button, Popover, PopoverContent, PopoverTrigger } from '@/components/ui'
import ProxyNodeSelect from '@/features/providers/components/ProxyNodeSelect.vue'

/** 号池密钥的独立代理节点设置弹层 */
withDefaults(defineProps<{
  nodeId: string
  nodeName?: string | null
  open: boolean
  saving?: boolean
  triggerClass?: string
}>(), {
  nodeName: null,
  saving: false,
  triggerClass: '',
})

defineEmits<{
  'update:open': [value: boolean]
  select: [nodeId: string]
  clear: []
}>()

const { t } = useI18n()

</script>
