<template>
  <TableRow
    class="border-b border-border/40 hover:bg-muted/30 transition-colors"
    :class="expanded ? 'bg-muted/20' : ''"
  >
    <TableCell class="w-[28px] min-w-[28px] max-w-[28px] p-0 pl-2 text-center">
      <button
        type="button"
        class="inline-flex h-5 w-5 items-center justify-center rounded-md text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1"
        :title="expanded ? t('ui.collapseData') : t('ui.expandData')"
        @click="$emit('toggle-details')"
      >
        <ChevronDown
          v-if="expanded"
          class="h-3.5 w-3.5"
        />
        <ChevronRight
          v-else
          class="h-3.5 w-3.5"
        />
      </button>
    </TableCell>
    <TableCell class="py-4 min-w-0">
      <div class="flex items-center gap-1.5 min-w-0">
        <span
          class="min-w-0 truncate text-sm font-semibold"
          :title="node.name"
        >{{ node.name }}</span>
        <Badge
          v-if="node.is_manual"
          variant="outline"
          class="text-[10px] px-1.5 py-0"
        >
          {{ t('ui.manual') }}
        </Badge>
        <Badge
          v-if="node.tunnel_mode"
          variant="outline"
          class="text-[10px] px-1.5 py-0"
        >
          Tunnel
        </Badge>
        <Badge
          v-if="schedulingBadge"
          :variant="schedulingBadge.variant"
          class="text-[10px] px-1.5 py-0"
        >
          {{ schedulingBadge.label }}
        </Badge>
        <HardwareTooltip :node="node" />
      </div>
    </TableCell>
    <TableCell class="py-4 min-w-0">
      <code
        class="block min-w-0 truncate text-xs text-muted-foreground"
        :title="proxyNodeAddress(node)"
      >{{ proxyNodeAddress(node) }}</code>
    </TableCell>
    <TableCell class="py-4 min-w-0">
      <span
        class="block min-w-0 truncate text-sm text-muted-foreground"
        :title="formatProxyNodeRegion(node.region)"
      >{{ formatProxyNodeRegion(node.region) }}</span>
    </TableCell>
    <TableCell class="py-4 text-center">
      <Badge
        :variant="proxyNodeStatusVariant(node.status)"
        :title="proxyNodeStatusTitle(node)"
        class="font-medium px-2.5 py-0.5 text-xs"
      >
        {{ proxyNodeStatusLabel(node) }}
      </Badge>
    </TableCell>
    <TableCell class="py-4 text-center">
      <span class="text-sm tabular-nums">{{ formatProxyNodeNumber(node.total_requests) }}</span>
    </TableCell>
    <TableCell class="py-4 text-center">
      <span
        class="text-sm tabular-nums"
        :class="proxyNodeFailureRate(node) > 5 ? 'text-destructive font-medium' : ''"
      >{{ formatProxyNodeFailureRate(node) }}</span>
    </TableCell>
    <TableCell class="py-4 text-center">
      <span class="text-sm tabular-nums">{{ node.avg_latency_ms != null ? `${node.avg_latency_ms.toFixed(0)}ms` : '-' }}</span>
    </TableCell>
    <TableCell class="py-4 text-center">
      <span class="text-sm tabular-nums">{{ node.is_manual ? '-' : proxyNodeVersion(node) }}</span>
    </TableCell>
    <TableCell class="py-4 min-w-0">
      <span class="block min-w-0 truncate text-xs text-muted-foreground">{{ formatProxyNodeTime(node.last_heartbeat_at, locale) }}</span>
    </TableCell>
    <TableCell class="py-4 text-center">
      <div class="flex items-center justify-center gap-0.5">
        <Button
          variant="ghost"
          size="icon"
          class="h-8 w-8"
          :title="testing ? t('ui.testing7b6187c') : t('ui.testConnectivity')"
          :disabled="testing"
          @click="$emit('test')"
        >
          <Loader2
            v-if="testing"
            class="h-4 w-4 animate-spin"
          />
          <Activity
            v-else
            class="h-4 w-4"
          />
        </Button>
        <Button
          v-if="node.is_manual"
          variant="ghost"
          size="icon"
          class="h-8 w-8"
          :title="t('ui.edit0518365')"
          @click="$emit('edit')"
        >
          <SquarePen class="h-4 w-4" />
        </Button>
        <Button
          v-if="!node.is_manual"
          variant="ghost"
          size="icon"
          class="h-8 w-8"
          :title="t('ui.remoteConfig')"
          @click="$emit('config')"
        >
          <Settings class="h-4 w-4" />
        </Button>
        <Button
          v-if="!node.is_manual"
          variant="ghost"
          size="icon"
          class="h-8 w-8"
          :title="t('ui.connectionEvents')"
          @click="$emit('view-events')"
        >
          <History class="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          class="h-8 w-8"
          :title="t('ui.delete')"
          @click="$emit('delete')"
        >
          <Trash2 class="h-4 w-4" />
        </Button>
      </div>
    </TableCell>
  </TableRow>
  <TableRow
    v-if="expanded"
    class="border-b border-border/40 hover:bg-transparent"
  >
    <TableCell
      colspan="11"
      class="p-0"
    >
      <ProxyNodeDataPanel
        :node="node"
        :state="detailState"
        @refresh="$emit('refresh-details')"
      />
    </TableCell>
  </TableRow>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Activity, ChevronDown, ChevronRight, History, Loader2, Settings, SquarePen, Trash2 } from 'lucide-vue-next'
import { Badge, Button, TableCell, TableRow } from '@/components/ui'
import type { ProxyNode } from '@/api/proxy-nodes'
import { useI18n } from '@/i18n'
import HardwareTooltip from './HardwareTooltip.vue'
import ProxyNodeDataPanel from './ProxyNodeDataPanel.vue'
import {
  formatProxyNodeFailureRate,
  formatProxyNodeNumber,
  formatProxyNodeRegion,
  formatProxyNodeTime,
  proxyNodeAddress,
  proxyNodeFailureRate,
  proxyNodeSchedulingBadge,
  proxyNodeStatusLabel,
  proxyNodeStatusTitle,
  proxyNodeStatusVariant,
  proxyNodeVersion,
} from './proxy-node-display'
import type { ProxyNodeDetailState } from './proxy-node-types'

const props = defineProps<{
  node: ProxyNode
  expanded: boolean
  testing: boolean
  detailState?: ProxyNodeDetailState | null
}>()

defineEmits<{
  'toggle-details': []
  'refresh-details': []
  test: []
  edit: []
  config: []
  'view-events': []
  delete: []
}>()

const { t, locale } = useI18n()
const schedulingBadge = computed(() => proxyNodeSchedulingBadge(props.node))
</script>
