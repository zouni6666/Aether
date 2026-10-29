<template>
  <div class="space-y-6 pb-8">
    <Card
      variant="default"
      class="overflow-hidden"
    >
      <ProxyNodeHeader
        v-model:search-query="searchQuery"
        v-model:filter-status="filterStatus"
        :loading="store.loading"
        :status-options="proxyNodeStatusFilterOptions"
        @open-distribution="showPoolProxyDistributionDialog = true"
        @open-batch-upgrade="showBatchUpgradeDialog = true"
        @open-add="openAddDialog"
        @refresh="refresh"
      />

      <ProxyNodeList
        v-model:filter-status="filterStatus"
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :nodes="paginatedNodes"
        :total="filteredNodes.length"
        :loading="store.loading"
        :status-options="proxyNodeStatusFilterOptions"
        :expanded-node-ids="expandedNodeIds"
        :testing-node-ids="testingNodes"
        :node-details="nodeDetails"
        @toggle-details="toggleNodeDetails"
        @refresh-details="loadNodeDetails"
        @test="handleTest"
        @edit="handleEdit"
        @config="handleConfig"
        @view-events="handleViewEvents"
        @delete="handleDelete"
      />
    </Card>
    <ProxyNodeFormDialog
      :open="showAddDialog"
      :editing-node="editingNode"
      :add-mode="addMode"
      :add-form="addForm"
      :batch-form="batchForm"
      :install-form="installForm"
      :install-system="installSystem"
      :install-loading="installLoading"
      :install-copied="installCopied"
      :proxy-install-command="proxyInstallCommand"
      :proxy-install-hint="proxyInstallHint"
      :batch-parse-result="batchParseResult"
      :adding-node="addingNode"
      :testing-url="testingUrl"
      @update:open="handleDialogClose"
      @update:add-mode="addMode = $event"
      @update:add-form="addForm = $event"
      @update:batch-form="batchForm = $event"
      @update:install-form="installForm = $event"
      @update:install-system="installSystem = $event"
      @refresh-install-command="refreshProxyInstallCommand"
      @copy-install-command="copyProxyInstallCommand"
      @submit-manual="editingNode ? handleUpdateManualNode() : handleAddManualNode()"
      @submit-batch="handleBatchAddManualNodes"
      @test-url="handleTestUrl"
    />

    <ProxyNodeRemoteConfigDialog
      :open="showConfigDialog"
      :node="configNode"
      :form="configForm"
      :saving="savingConfig"
      @update:open="handleConfigDialogClose"
      @update:form="configForm = $event"
      @save="handleSaveConfig"
    />

    <ProxyNodeBatchUpgradeDialog
      :open="showBatchUpgradeDialog"
      :version="batchUpgradeVersion"
      :upgrading="batchUpgrading"
      @update:open="handleBatchUpgradeDialogOpen"
      @update:version="batchUpgradeVersion = $event"
      @submit="handleBatchUpgrade"
    />

    <PoolProxyDistributionDialog
      v-model="showPoolProxyDistributionDialog"
    />

    <ProxyNodeEventsDialog
      :open="showEventsDialog"
      :node="eventsNode"
      :events="nodeEvents"
      :loading="loadingEvents"
      @update:open="handleEventsDialogClose"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useProxyNodesStore } from '@/stores/proxy-nodes'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { useClipboard } from '@/composables/useClipboard'
import { useClientPagination } from '@/composables/useClientPagination'
import { useI18n } from '@/i18n'
import { clearModelsDevCache } from '@/api/models-dev'
import {
  proxyNodesApi,
  type ProxyNode,
  type ProxyNodeEvent,
  type ProxyNodeInstallSession,
  type ProxyNodeMetricsResponse,
  type ProxyNodeRemoteConfig,
  type ProxyNodeSchedulingState,
  type ProxyNodeTestResult,
} from '@/api/proxy-nodes'

import { Card } from '@/components/ui'
import { parseApiError } from '@/utils/errorParser'
import { parseBatchProxyNodeInput } from '@/features/proxy-nodes/utils/proxy-node-batch'
import { proxyNodeDeleteSuccessMessage } from '@/features/proxy-nodes/utils/proxy-node-delete-feedback'
import ProxyNodeBatchUpgradeDialog from '@/features/proxy-nodes/components/ProxyNodeBatchUpgradeDialog.vue'
import ProxyNodeEventsDialog from '@/features/proxy-nodes/components/ProxyNodeEventsDialog.vue'
import ProxyNodeFormDialog from '@/features/proxy-nodes/components/ProxyNodeFormDialog.vue'
import ProxyNodeHeader from '@/features/proxy-nodes/components/ProxyNodeHeader.vue'
import ProxyNodeList from '@/features/proxy-nodes/components/ProxyNodeList.vue'
import ProxyNodeRemoteConfigDialog from '@/features/proxy-nodes/components/ProxyNodeRemoteConfigDialog.vue'
import PoolProxyDistributionDialog from '@/features/pool/components/PoolProxyDistributionDialog.vue'

const { success, error: toastError } = useToast()
const { confirmDanger } = useConfirm()
const { copyToClipboard } = useClipboard()
const { t, locale } = useI18n()
const store = useProxyNodesStore()

const searchQuery = ref('')
const filterStatus = ref('all')
const proxyNodeStatusFilterOptions = [
  { value: 'all', get label() { return t('ui.allStatuses') } },
  { value: 'online', get label() { return t('vscodex.devices.online') } },
  { value: 'offline', get label() { return t('vscodex.devices.offline') } },
]
// 手动添加/编辑对话框
const showAddDialog = ref(false)
const showPoolProxyDistributionDialog = ref(false)
const addingNode = ref(false)
const editingNode = ref<ProxyNode | null>(null)
const addMode = ref<'script' | 'manual' | 'batch'>('script')
const addForm = ref({
  name: '',
  proxy_url: '',
  username: '',
  password: '',
  region: '',
})
const batchForm = ref({
  content: '',
})
const installForm = ref({
  node_name: '',
})
const installSystem = ref<'unix' | 'windows'>('unix')
const installLoading = ref(false)
const installCopied = ref(false)
const proxyInstallSession = ref<ProxyNodeInstallSession | null>(null)
let installCopiedResetTimer: ReturnType<typeof setTimeout> | null = null

const proxyInstallCommand = computed(() => {
  if (!proxyInstallSession.value) return ''
  return installSystem.value === 'windows'
    ? proxyInstallSession.value.powershell_command
    : proxyInstallSession.value.unix_command
})

const proxyInstallHint = computed(() => {
  if (!proxyInstallSession.value) {
    return t('ui.theScriptInstallsOrUpdatesTheProxyProgramAndPreserves')
  }
  const minutes = Math.floor(proxyInstallSession.value.expires_in_seconds / 60)
  return t('feedback.proxy.installExpiry', { minutes })
})

const batchParseResult = computed(() => parseBatchProxyNodeInput(batchForm.value.content))

// 远程配置对话框 (aether-tunnel 节点)
const showConfigDialog = ref(false)
const savingConfig = ref(false)
const configNode = ref<ProxyNode | null>(null)
const configForm = ref({
  allowed_ports: '',
  log_level: 'info',
  heartbeat_interval: '30',
  scheduling_state: 'active' as ProxyNodeSchedulingState,
  upgrade_to: '',
})
const showBatchUpgradeDialog = ref(false)
const batchUpgradeVersion = ref('')
const batchUpgrading = ref(false)

// 连接事件对话框
const showEventsDialog = ref(false)
const eventsNode = ref<ProxyNode | null>(null)
const nodeEvents = ref<ProxyNodeEvent[]>([])
const loadingEvents = ref(false)

interface ProxyNodeDetailState {
  loading: boolean
  error: string | null
  node: ProxyNode | null
  metrics: ProxyNodeMetricsResponse | null
  events: ProxyNodeEvent[]
  loadedAt: number | null
}

const expandedNodeIds = ref(new Set<string>())
const nodeDetails = ref<Record<string, ProxyNodeDetailState>>({})

// 测试连通性
const testingNodes = ref(new Set<string>())
const testingUrl = ref(false)

const filteredNodes = computed(() => {
  let filtered = [...store.nodes]

  if (searchQuery.value) {
    const keywords = searchQuery.value.toLowerCase().split(/\s+/).filter(k => k.length > 0)
    filtered = filtered.filter(node => {
      const text = `${node.name} ${node.ip} ${node.region || ''}`.toLowerCase()
      return keywords.every(kw => text.includes(kw))
    })
  }

  if (filterStatus.value !== 'all') {
    filtered = filtered.filter(node => node.status === filterStatus.value)
  }

  return filtered
})

const { currentPage, pageSize, pageItems: paginatedNodes, resetPage } = useClientPagination(filteredNodes)

watch([searchQuery, filterStatus], resetPage)

watch(() => installForm.value.node_name, () => {
  resetProxyInstallState()
})

watch(installSystem, () => {
  installCopied.value = false
  clearInstallCopiedResetTimer()
})

onMounted(async () => {
  await store.fetchNodes()
})

onBeforeUnmount(() => {
  clearInstallCopiedResetTimer()
})

async function refresh() {
  await store.fetchNodes()
}

function formatConnectivityTestParts(result: ProxyNodeTestResult): string[] {
  const parts = [
    `${t('ui.probe')}: ${formatConnectivityProbe(result.probe_url)}`,
    `${t('ui.timeoute512cf0')}: ${result.timeout_secs}s`,
    `${t('ui.latency')}: ${result.latency_ms != null ? `${result.latency_ms}ms` : t('ui.noSample')}`,
  ]
  if (result.exit_ip) parts.push(`${t('ui.exitIp')}: ${result.exit_ip}`)
  return parts
}

function formatConnectivityResult(message: string, result: ProxyNodeTestResult): string {
  const separator = locale.value === 'en-US' ? ', ' : '，'
  return `${message}${separator}${formatConnectivityTestParts(result).join(separator)}`
}

function formatConnectivityProbe(probeUrl: string) {
  try {
    const url = new URL(probeUrl)
    return `${url.host}${url.pathname === '/' ? '' : url.pathname}`
  } catch {
    return probeUrl
  }
}

async function handleTestUrl() {
  if (!addForm.value.proxy_url || testingUrl.value) return
  testingUrl.value = true
  try {
    const result = await proxyNodesApi.testProxyUrl({
      proxy_url: addForm.value.proxy_url,
      username: addForm.value.username || undefined,
      password: addForm.value.password || undefined,
    })
    if (result.success) {
      success(formatConnectivityResult(t('ui.connectivityTestPassed'), result))
    } else {
      const details = formatConnectivityTestParts(result).join(locale.value === 'en-US' ? ', ' : '，')
      toastError(t('feedback.proxy.connectivityFailure', { details, reason: result.error || t('ui.unknownError') }))
    }
  } catch (err: unknown) {
    toastError(parseApiError(err, t('ui.testRequestFailed')))
  } finally {
    testingUrl.value = false
  }
}

function clearInstallCopiedResetTimer() {
  if (installCopiedResetTimer) {
    clearTimeout(installCopiedResetTimer)
    installCopiedResetTimer = null
  }
}

function resetProxyInstallState() {
  proxyInstallSession.value = null
  installCopied.value = false
  clearInstallCopiedResetTimer()
}

function openAddDialog() {
  editingNode.value = null
  addMode.value = 'script'
  addForm.value = { name: '', proxy_url: '', username: '', password: '', region: '' }
  batchForm.value = { content: '' }
  installForm.value = { node_name: '' }
  resetProxyInstallState()
  showAddDialog.value = true
}

async function refreshProxyInstallCommand() {
  const nodeName = installForm.value.node_name.trim()
  if (!nodeName || installLoading.value) return
  installLoading.value = true
  resetProxyInstallState()
  try {
    proxyInstallSession.value = await store.createInstallSession({ node_name: nodeName })
    success(t('ui.proxyNodeInstallCommandGenerated'))
  } catch (err: unknown) {
    toastError(parseApiError(err, t('ui.failedToGenerateProxyNodeInstallCommand')))
  } finally {
    installLoading.value = false
  }
}

async function copyProxyInstallCommand() {
  if (!proxyInstallCommand.value) return
  const copied = await copyToClipboard(proxyInstallCommand.value, false)
  if (!copied) return
  installCopied.value = true
  success(t('ui.installCommandCopiedToClipboard'))
  clearInstallCopiedResetTimer()
  installCopiedResetTimer = setTimeout(() => {
    installCopied.value = false
    installCopiedResetTimer = null
  }, 2000)
}

async function handleEdit(node: ProxyNode) {
  try {
    const { node: detail } = await proxyNodesApi.getNode(node.id)
    editingNode.value = detail
    addForm.value = {
      name: detail.name,
      proxy_url: detail.proxy_url || '',
      username: detail.proxy_username || '',
      // 密码是 write-only；留空时更新接口会保留已存密码。
      password: '',
      region: detail.region || '',
    }
    addMode.value = 'manual'
    resetProxyInstallState()
    showAddDialog.value = true
  } catch (err: unknown) {
    toastError(parseApiError(err, t('ui.failedToReadProxyNodeDetails')))
  }
}

function handleDialogClose(open: boolean) {
  if (!open) {
    showAddDialog.value = false
    editingNode.value = null
    addMode.value = 'script'
    addForm.value = { name: '', proxy_url: '', username: '', password: '', region: '' }
    batchForm.value = { content: '' }
    installForm.value = { node_name: '' }
    resetProxyInstallState()
  }
}

async function handleUpdateManualNode() {
  if (!editingNode.value || !addForm.value.name || !addForm.value.proxy_url) return

  addingNode.value = true
  try {
    await proxyNodesApi.updateManualNode(editingNode.value.id, {
      name: addForm.value.name,
      proxy_url: addForm.value.proxy_url,
      username: addForm.value.username || undefined,
      // 空密码不发送（保留原值）
      password: addForm.value.password || undefined,
      region: addForm.value.region || undefined,
    })
    success(t('ui.proxyNodeUpdated'))
    handleDialogClose(false)
    await store.fetchNodes()
  } catch (err: unknown) {
    toastError(parseApiError(err, t('update.error.updateFailed')))
  } finally {
    addingNode.value = false
  }
}

async function handleAddManualNode() {
  if (!addForm.value.name || !addForm.value.proxy_url) return

  addingNode.value = true
  try {
    await store.createManualNode({
      name: addForm.value.name,
      proxy_url: addForm.value.proxy_url,
      username: addForm.value.username || undefined,
      password: addForm.value.password || undefined,
      region: addForm.value.region || undefined,
    })
    success(t('ui.proxyNodeAdded'))
    handleDialogClose(false)
  } catch (err: unknown) {
    toastError(parseApiError(err, t('ui.failedToAdd')))
  } finally {
    addingNode.value = false
  }
}

async function handleBatchAddManualNodes() {
  const { nodes, errors } = batchParseResult.value
  if (!batchForm.value.content.trim() || addingNode.value) return
  if (errors.length > 0) {
    toastError(t('ui.batchInputHasValueFormatErrorsFixThemBeforeAdding', { count: errors.length }))
    return
  }
  if (nodes.length === 0) {
    toastError(t('ui.enterAtLeastOneProxyAddressFirst'))
    return
  }

  addingNode.value = true
  const failures: string[] = []
  let successCount = 0

  try {
    for (const node of nodes) {
      try {
        await proxyNodesApi.createManualNode(node)
        successCount += 1
      } catch (err: unknown) {
        failures.push(`${node.name}: ${parseApiError(err, t('ui.failedToAdd'))}`)
      }
    }

    await store.fetchNodes()

    if (successCount > 0 && failures.length === 0) {
      success(t('ui.addedValueProxyNodes', { count: successCount }))
      handleDialogClose(false)
      return
    }

    if (successCount > 0) {
      success(t('ui.addedValueProxyNodesValueFailed', { count: successCount, count2: failures.length }))
    }

    if (failures.length > 0) {
      toastError(failures.slice(0, 3).join('；'))
    }
  } catch (err: unknown) {
    toastError(parseApiError(err, t('ui.batchAddFailed')))
  } finally {
    addingNode.value = false
  }
}

function handleConfig(node: ProxyNode) {
  configNode.value = node
  const rc: ProxyNodeRemoteConfig = node.remote_config ?? {}
  configForm.value = {
    allowed_ports: rc.allowed_ports?.join(', ') || '',
    log_level: rc.log_level || 'info',
    heartbeat_interval: String(rc.heartbeat_interval || node.heartbeat_interval || 30),
    scheduling_state: rc.scheduling_state || 'active',
    upgrade_to: rc.upgrade_to || '',
  }
  showConfigDialog.value = true
}

function handleConfigDialogClose(open: boolean) {
  if (!open) {
    showConfigDialog.value = false
    configNode.value = null
  }
}

async function handleSaveConfig() {
  if (!configNode.value) return
  savingConfig.value = true
  try {
    const data: Partial<ProxyNodeRemoteConfig> = {}
    const portsInput = configForm.value.allowed_ports.trim()
    if (portsInput) {
      data.allowed_ports = portsInput
        .split(',')
        .map((s: string) => parseInt(s.trim()))
        .filter((n: number) => !isNaN(n) && n >= 1 && n <= 65535)
    } else if (configNode.value.remote_config?.allowed_ports) {
      // 输入清空 → 显式发送空数组以清除已有端口白名单
      data.allowed_ports = []
    }
    if (configForm.value.log_level) {
      data.log_level = configForm.value.log_level
    }
    const hb = parseInt(configForm.value.heartbeat_interval)
    if (!isNaN(hb) && hb >= 5) {
      data.heartbeat_interval = hb
    }
    data.scheduling_state = configForm.value.scheduling_state
    const targetVersion = configForm.value.upgrade_to.trim()
    if (targetVersion) {
      data.upgrade_to = targetVersion
    } else if (configNode.value.remote_config?.upgrade_to) {
      data.upgrade_to = null
    }
    await proxyNodesApi.updateNodeConfig(configNode.value.id, data)
    success(t('ui.remoteConfigSavedItWillTakeEffectOnTheNext'))
    handleConfigDialogClose(false)
    await store.fetchNodes()
  } catch (err: unknown) {
    toastError(parseApiError(err, t('ui.failedToSave')))
  } finally {
    savingConfig.value = false
  }
}

async function handleBatchUpgrade() {
  const version = batchUpgradeVersion.value.trim()
  if (!version || batchUpgrading.value) return
  batchUpgrading.value = true
  try {
    const result = await proxyNodesApi.batchUpgrade(version)
    if (result.updated > 0) {
      success(t('ui.wroteUpgradeTargetValueToValueNodesValueNodesDid', { value: result.updated, version: result.version, value3: result.skipped }))
    } else {
      success(t('ui.noTunnelNodesNeedChangesTargetVersionRemainsValue', { version: result.version }))
    }
    resetBatchUpgradeDialog()
    await store.fetchNodes()
  } catch (err: unknown) {
    toastError(parseApiError(err, t('ui.failedToDispatchBatchUpgrade')))
  } finally {
    batchUpgrading.value = false
  }
}

function resetBatchUpgradeDialog() {
  showBatchUpgradeDialog.value = false
  batchUpgradeVersion.value = ''
}

function handleBatchUpgradeDialogOpen(open: boolean) {
  if (open) {
    showBatchUpgradeDialog.value = true
    return
  }
  resetBatchUpgradeDialog()
}

async function handleDelete(node: ProxyNode) {
  const address = node.tunnel_mode ? node.ip : `${node.ip}:${node.port}`
  const confirmed = await confirmDanger(
    t('feedback.proxy.deleteConfirm', { name: node.name, address }),
    t('ui.deleteNode')
  )
  if (!confirmed) return

  try {
    const result = await proxyNodesApi.deleteProxyNode(node.id)
    if (result.cleared_external_models_proxy) {
      clearModelsDevCache()
    }
    await store.fetchNodes()
    success(proxyNodeDeleteSuccessMessage(
      result.cleared_system_proxy,
      result.cleared_external_models_proxy,
    ))
  } catch (err: unknown) {
    toastError(parseApiError(err, t('ui.deletionFailed')))
  }
}

async function handleTest(node: ProxyNode) {
  if (testingNodes.value.has(node.id)) return

  testingNodes.value.add(node.id)
  try {
    const result = await proxyNodesApi.testNode(node.id)
    if (result.success) {
      success(formatConnectivityResult(t('ui.connectivityTestPassed'), result))
    } else {
      const details = formatConnectivityTestParts(result).join(locale.value === 'en-US' ? ', ' : '，')
      toastError(t('feedback.proxy.connectivityFailure', { details, reason: result.error || t('ui.unknownError') }))
    }
  } catch (err: unknown) {
    toastError(parseApiError(err, t('ui.testRequestFailed')))
  } finally {
    testingNodes.value.delete(node.id)
  }
}

function createNodeDetailState(): ProxyNodeDetailState {
  return {
    loading: false,
    error: null,
    node: null,
    metrics: null,
    events: [],
    loadedAt: null,
  }
}

function updateNodeDetailState(nodeId: string, patch: Partial<ProxyNodeDetailState>) {
  nodeDetails.value = {
    ...nodeDetails.value,
    [nodeId]: {
      ...(nodeDetails.value[nodeId] ?? createNodeDetailState()),
      ...patch,
    },
  }
}

function toggleNodeDetails(node: ProxyNode) {
  const next = new Set(expandedNodeIds.value)
  if (next.has(node.id)) {
    next.delete(node.id)
    expandedNodeIds.value = next
    return
  }

  next.add(node.id)
  expandedNodeIds.value = next

  const detailState = nodeDetails.value[node.id]
  if (!detailState?.loadedAt && !detailState?.loading) {
    void loadNodeDetails(node)
  }
}

async function loadNodeDetails(node: ProxyNode) {
  updateNodeDetailState(node.id, { loading: true, error: null })
  const to = Math.floor(Date.now() / 1000)
  const from = to - 24 * 60 * 60
  const eventsFrom = to - 7 * 24 * 60 * 60

  try {
    const [detail, metrics, events] = await Promise.all([
      proxyNodesApi.getNode(node.id),
      proxyNodesApi.listNodeMetrics(node.id, { from, to, step: '1h' }),
      proxyNodesApi.listNodeEvents(node.id, { limit: 8, from: eventsFrom, to }),
    ])
    updateNodeDetailState(node.id, {
      loading: false,
      error: null,
      node: detail.node,
      metrics,
      events: events.items,
      loadedAt: Date.now(),
    })
  } catch (err: unknown) {
    updateNodeDetailState(node.id, {
      loading: false,
      error: parseApiError(err, t('ui.failedToLoadNodeData')),
    })
  }
}

async function handleViewEvents(node: ProxyNode) {
  eventsNode.value = node
  showEventsDialog.value = true
  loadingEvents.value = true
  try {
    const res = await proxyNodesApi.listNodeEvents(node.id, { limit: 50 })
    nodeEvents.value = res.items
  } catch (err: unknown) {
    toastError(parseApiError(err, t('ui.failedToLoadEvents')))
  } finally {
    loadingEvents.value = false
  }
}

function handleEventsDialogClose(open: boolean) {
  showEventsDialog.value = open
  if (!open) {
    eventsNode.value = null
    nodeEvents.value = []
  }
}
</script>
