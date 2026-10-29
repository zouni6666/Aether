<template>
  <div class="hidden xl:block overflow-x-auto">
    <Table class="min-w-[1040px] table-fixed">
      <TableHeader>
        <TableRow class="border-b border-border/60 hover:bg-transparent">
          <TableHead class="w-[28px] min-w-[28px] max-w-[28px] h-12 p-0 pl-2" />
          <TableHead class="w-[150px] h-12 font-semibold">
            {{ t('stats.column.name') }}
          </TableHead>
          <TableHead class="w-[190px] h-12 font-semibold">
            {{ t('ui.address') }}
          </TableHead>
          <TableHead class="w-[90px] h-12 font-semibold">
            {{ t('ui.region') }}
          </TableHead>
          <SortableTableHead
            class="w-[90px] h-12 font-semibold text-center"
            column-key="status"
            :sortable="false"
            align="center"
            :filter-active="filterStatus !== 'all'"
            :filter-title="t('ui.filterStatus')"
            filter-content-class="w-36 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
          >
            {{ t('ui.status6320b4a') }}
            <template #filter="{ close }">
              <TableFilterMenu
                :model-value="filterStatus"
                :options="statusOptions"
                @update:model-value="(value) => $emit('update:filterStatus', value)"
                @select="close"
              />
            </template>
          </SortableTableHead>
          <TableHead class="w-[86px] h-12 font-semibold text-center">
            {{ t('ui.totalRequests') }}
          </TableHead>
          <TableHead class="w-[86px] h-12 font-semibold text-center">
            {{ t('ui.failureRate') }}
          </TableHead>
          <TableHead class="w-[86px] h-12 font-semibold text-center">
            {{ t('ui.latency') }}
          </TableHead>
          <TableHead class="w-[90px] h-12 font-semibold text-center">
            {{ t('ui.version') }}
          </TableHead>
          <TableHead class="w-[130px] h-12 font-semibold">
            {{ t('ui.lastHeartbeat') }}
          </TableHead>
          <TableHead class="w-[120px] h-12 font-semibold text-center">
            {{ t('ui.actionsed31fbb') }}
          </TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <template
          v-for="node in nodes"
          :key="node.id"
        >
          <TableRow
            class="border-b border-border/40 hover:bg-muted/30 transition-colors"
            :class="expandedNodeIds.has(node.id) ? 'bg-muted/20' : ''"
          >
            <TableCell class="w-[28px] min-w-[28px] max-w-[28px] p-0 pl-2 text-center">
              <button
                type="button"
                class="inline-flex h-5 w-5 items-center justify-center rounded-md text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1"
                :title="expandedNodeIds.has(node.id) ? t('ui.collapseData') : t('ui.expandData')"
                @click="$emit('toggle-details', node)"
              >
                <ChevronDown
                  v-if="expandedNodeIds.has(node.id)"
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
                  v-if="proxyNodeSchedulingBadge(node)"
                  :variant="proxyNodeSchedulingBadge(node)?.variant"
                  class="text-[10px] px-1.5 py-0"
                >
                  {{ (proxyNodeSchedulingBadge(node)?.label || "") }}
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
                  :title="testingNodeIds.has(node.id) ? t('ui.testing7b6187c') : t('ui.testConnectivity')"
                  :disabled="testingNodeIds.has(node.id)"
                  @click="$emit('test', node)"
                >
                  <Loader2
                    v-if="testingNodeIds.has(node.id)"
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
                  @click="$emit('edit', node)"
                >
                  <SquarePen class="h-4 w-4" />
                </Button>
                <Button
                  v-if="!node.is_manual"
                  variant="ghost"
                  size="icon"
                  class="h-8 w-8"
                  :title="t('ui.remoteConfig')"
                  @click="$emit('config', node)"
                >
                  <Settings class="h-4 w-4" />
                </Button>
                <Button
                  v-if="!node.is_manual"
                  variant="ghost"
                  size="icon"
                  class="h-8 w-8"
                  :title="t('ui.connectionEvents')"
                  @click="$emit('view-events', node)"
                >
                  <History class="h-4 w-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-8 w-8"
                  :title="t('ui.delete')"
                  @click="$emit('delete', node)"
                >
                  <Trash2 class="h-4 w-4" />
                </Button>
              </div>
            </TableCell>
          </TableRow>
          <TableRow
            v-if="expandedNodeIds.has(node.id)"
            class="border-b border-border/40 hover:bg-transparent"
          >
            <TableCell
              colspan="11"
              class="p-0"
            >
              <ProxyNodeDataPanel
                :node="node"
                :state="nodeDetails[node.id]"
                @refresh="$emit('refresh-details', node)"
              />
            </TableCell>
          </TableRow>
        </template>
        <TableRow v-if="nodes.length === 0">
          <TableCell
            colspan="11"
            class="py-12 text-center text-muted-foreground text-sm"
          >
            {{ loading ? t('common.loading') : t('ui.noProxyNodes') }}
          </TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </div>

  <div class="xl:hidden divide-y divide-border/40">
    <ProxyNodeMobileCard
      v-for="node in nodes"
      :key="node.id"
      :node="node"
      :expanded="expandedNodeIds.has(node.id)"
      :testing="testingNodeIds.has(node.id)"
      :detail-state="nodeDetails[node.id]"
      @toggle-details="$emit('toggle-details', node)"
      @refresh-details="$emit('refresh-details', node)"
      @test="$emit('test', node)"
      @edit="$emit('edit', node)"
      @config="$emit('config', node)"
      @delete="$emit('delete', node)"
    />
    <div
      v-if="nodes.length === 0"
      class="p-8 text-center text-muted-foreground text-sm"
    >
      {{ loading ? t('common.loading') : t('ui.noProxyNodes') }}
    </div>
  </div>

  <Pagination
    :current="currentPage"
    :total="total"
    :page-size="pageSize"
    cache-key="proxy-nodes-page-size"
    @update:current="$emit('update:currentPage', $event)"
    @update:page-size="$emit('update:pageSize', $event)"
  />
</template>

<script setup lang="ts">
import { Activity, ChevronDown, ChevronRight, History, Loader2, Settings, SquarePen, Trash2 } from 'lucide-vue-next'
import { Badge, Button, Pagination, SortableTableHead, Table, TableBody, TableCell, TableFilterMenu, TableHead, TableHeader, TableRow } from '@/components/ui'
import type { ProxyNode } from '@/api/proxy-nodes'
import { useI18n } from '@/i18n'
import HardwareTooltip from './HardwareTooltip.vue'
import ProxyNodeMobileCard from './ProxyNodeMobileCard.vue'
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
import type { ProxyNodeDetailState, ProxyNodeStatusFilterOption } from './proxy-node-types'

defineProps<{
  nodes: ProxyNode[]
  total: number
  loading: boolean
  filterStatus: string
  statusOptions: ProxyNodeStatusFilterOption[]
  currentPage: number
  pageSize: number
  expandedNodeIds: Set<string>
  testingNodeIds: Set<string>
  nodeDetails: Record<string, ProxyNodeDetailState>
}>()

defineEmits<{
  'update:filterStatus': [value: string]
  'update:currentPage': [value: number]
  'update:pageSize': [value: number]
  'toggle-details': [node: ProxyNode]
  'refresh-details': [node: ProxyNode]
  test: [node: ProxyNode]
  edit: [node: ProxyNode]
  config: [node: ProxyNode]
  'view-events': [node: ProxyNode]
  delete: [node: ProxyNode]
}>()

const { t, locale } = useI18n()
</script>
