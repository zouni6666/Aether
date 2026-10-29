<template>
  <CardSection
    :title="t('ui.networkProxy')"
    :description="t('ui.configureTheDefaultProxyForProviderOutboundRequestsOnlyAffects')"
  >
    <template #actions>
      <Button
        size="sm"
        :disabled="loading || !hasChanges"
        @click="$emit('save')"
      >
        {{ loading ? t('ui.saving') : t('ui.save') }}
      </Button>
    </template>
    <div class="max-w-md">
      <Label class="block text-sm font-medium mb-1"> {{ t('ui.defaultProxyNode') }} </Label>
      <Select
        :model-value="proxyNodeId || '__direct__'"
        @update:model-value="(v: string) => $emit('update:proxyNodeId', v === '__direct__' ? null : v)"
      >
        <SelectTrigger>
          <SelectValue :placeholder="t('ui.directConnectionNoProxy')" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="__direct__">
            {{ t('ui.directConnectionNoProxy') }}
          </SelectItem>
          <SelectItem
            v-for="node in selectableNodes"
            :key="node.id"
            :value="node.id"
          >
            {{ node.name }}{{ node.region ? ` · ${node.region}` : '' }} ({{ node.ip }}:{{ node.port }})
          </SelectItem>
        </SelectContent>
      </Select>
      <p class="mt-1 text-xs text-muted-foreground">
        {{ t('ui.appliesToProvidersWithoutAnIndividualProxyAffectsModelApi') }}
      </p>
    </div>
  </CardSection>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'

import { computed } from 'vue'
import Button from '@/components/ui/button.vue'
import Label from '@/components/ui/label.vue'
import Select from '@/components/ui/select.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectValue from '@/components/ui/select-value.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import { CardSection } from '@/components/layout'

interface ProxyNode {
  id: string
  name: string
  region?: string | null
  ip: string
  port: number
}

const props = defineProps<{
  proxyNodeId: string | null
  onlineNodes: ProxyNode[]
  allNodes: ProxyNode[]
  loading: boolean
  hasChanges: boolean
}>()

defineEmits<{
  save: []
  'update:proxyNodeId': [value: string | null]
}>()

const { t } = useI18n()

const selectableNodes = computed(() => {
  if (!props.proxyNodeId) {
    return props.onlineNodes
  }
  const exists = props.onlineNodes.some(node => node.id === props.proxyNodeId)
  if (exists) {
    return props.onlineNodes
  }
  const selected = props.allNodes.find(node => node.id === props.proxyNodeId)
  return selected ? [selected, ...props.onlineNodes] : props.onlineNodes
})

</script>
