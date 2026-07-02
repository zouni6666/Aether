<template>
  <div class="hidden xl:block overflow-x-auto">
    <Table>
      <TableHeader>
        <TableRow class="border-b border-border/60 hover:bg-transparent">
          <TableHead class="w-[28px] min-w-[28px] max-w-[28px] h-12 p-0 pl-2" />
          <TableHead class="w-[160px] h-12 font-semibold">
            {{ legacyT('名称') }}
          </TableHead>
          <TableHead class="w-[180px] h-12 font-semibold">
            {{ legacyT('地址') }}
          </TableHead>
          <TableHead class="w-[100px] h-12 font-semibold">
            {{ legacyT('区域') }}
          </TableHead>
          <SortableTableHead
            class="w-[90px] h-12 font-semibold text-center"
            column-key="status"
            :sortable="false"
            align="center"
            :filter-active="filterStatus !== 'all'"
            :filter-title="legacyT('筛选状态')"
            filter-content-class="w-36 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
          >
            {{ legacyT('状态') }}
            <template #filter="{ close }">
              <TableFilterMenu
                :model-value="filterStatus"
                :options="statusOptions.map((option) => ({ ...option, label: legacyT(option.label) }))"
                @update:model-value="(value) => $emit('update:filterStatus', value)"
                @select="close"
              />
            </template>
          </SortableTableHead>
          <TableHead class="w-[100px] h-12 font-semibold text-center">
            {{ legacyT('总请求') }}
          </TableHead>
          <TableHead class="w-[100px] h-12 font-semibold text-center">
            {{ legacyT('失败率') }}
          </TableHead>
          <TableHead class="w-[100px] h-12 font-semibold text-center">
            {{ legacyT('延迟') }}
          </TableHead>
          <TableHead class="w-[120px] h-12 font-semibold text-center">
            {{ legacyT('版本') }}
          </TableHead>
          <TableHead class="w-[160px] h-12 font-semibold">
            {{ legacyT('最后心跳') }}
          </TableHead>
          <TableHead class="w-[140px] h-12 font-semibold text-center">
            {{ legacyT('操作') }}
          </TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <ProxyNodeTableRow
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
          @view-events="$emit('view-events', node)"
          @delete="$emit('delete', node)"
        />
        <TableRow v-if="nodes.length === 0">
          <TableCell
            colspan="11"
            class="py-12 text-center text-muted-foreground text-sm"
          >
            {{ loading ? legacyT('加载中...') : legacyT('暂无代理节点') }}
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
      {{ loading ? legacyT('加载中...') : legacyT('暂无代理节点') }}
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
import { Pagination, SortableTableHead, Table, TableBody, TableCell, TableFilterMenu, TableHead, TableHeader, TableRow } from '@/components/ui'
import type { ProxyNode } from '@/api/proxy-nodes'
import { useI18n } from '@/i18n'
import ProxyNodeMobileCard from './ProxyNodeMobileCard.vue'
import ProxyNodeTableRow from './ProxyNodeTableRow.vue'
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

const { legacyT } = useI18n()
</script>
