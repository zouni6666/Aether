<template>
  <div class="space-y-1.5">
    <Select
      :model-value="modelValue"
      :disabled="disabled || proxyNodesStore.loading || nodeOptions.length === 0"
      @update:model-value="(v: string) => $emit('update:modelValue', v)"
    >
      <SelectTrigger
        :class="triggerClass"
        :aria-label="triggerAriaLabel"
      >
        <SelectValue
          :placeholder="proxyNodesStore.loading
            ? t('ui.loadingNodeList')
            : nodeOptions.length === 0
              ? t('ui.noAvailableNodes')
              : t('ui.selectProxyNode40ff86b')"
        />
      </SelectTrigger>
      <SelectContent>
        <SelectItem
          v-for="node in nodeOptions"
          :key="node.id"
          :value="node.id"
        >
          {{ node.name }}{{ node.region ? ` · ${formatRegion(node.region, '')}` : '' }} ({{ node.ip }}:{{ node.port }})
        </SelectItem>
      </SelectContent>
    </Select>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from '@/components/ui'
import { useProxyNodesStore } from '@/stores/proxy-nodes'
import { useI18n } from '@/i18n'
import { formatRegion } from '@/utils/region'

const props = withDefaults(defineProps<{
  modelValue: string
  triggerClass?: string
  triggerAriaLabel?: string
  disabled?: boolean
}>(), {
  triggerClass: '',
  triggerAriaLabel: undefined,
  disabled: false,
})

defineEmits<{
  'update:modelValue': [value: string]
}>()

const { t } = useI18n()

const proxyNodesStore = useProxyNodesStore()

/** 在线节点 + 保留当前已选节点（可能已离线） */
const nodeOptions = computed(() => {
  const online = proxyNodesStore.onlineNodes
  if (props.modelValue) {
    const found = online.find(n => n.id === props.modelValue)
    if (!found) {
      const allNode = proxyNodesStore.nodes.find(n => n.id === props.modelValue)
      if (allNode) return [allNode, ...online]
    }
  }
  return online
})

/** 供父组件调用：启用代理时懒加载节点列表 */
function ensureLoaded() {
  return proxyNodesStore.ensureLoaded()
}

onMounted(ensureLoaded)

defineExpose({ ensureLoaded })
</script>
