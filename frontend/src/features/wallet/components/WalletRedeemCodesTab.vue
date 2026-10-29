<template>
  <div class="space-y-5">
    <div class="rounded-2xl border border-border/60 bg-background p-4 space-y-4">
      <div class="flex items-center justify-between gap-3">
        <div>
          <h4 class="text-sm font-semibold">
            {{ t('ui.generateRedemptionCodesInBulk') }}
          </h4>
          <p class="text-xs text-muted-foreground mt-1">
            {{ t('ui.newlyGeneratedCodesCanBeRevealedDuringThisSessionAfter') }}
          </p>
        </div>
        <RefreshButton
          :loading="loadingRedeemBatches || loadingRedeemCodes"
          @click="loadBatches"
        />
      </div>

      <div class="grid gap-3 lg:grid-cols-4">
        <div class="space-y-1.5">
          <Label>{{ t('ui.batchName') }}</Label>
          <Input v-model="redeemBatchForm.name" />
        </div>
        <div class="space-y-1.5">
          <Label>{{ t('ui.faceValueUsd') }}</Label>
          <Input
            v-model.number="redeemBatchForm.amount_usd"
            type="number"
            min="0.01"
            step="0.01"
          />
        </div>
        <div class="space-y-1.5">
          <Label>{{ t('ui.numberToGenerate') }}</Label>
          <Input
            v-model.number="redeemBatchForm.total_count"
            type="number"
            min="1"
            step="1"
          />
        </div>
        <div class="space-y-1.5">
          <Label>{{ t('ui.expirationOptional') }}</Label>
          <Input
            v-model="redeemBatchForm.expires_at"
            type="datetime-local"
          />
        </div>
      </div>

      <div class="space-y-1.5">
        <Label>{{ t('ui.notesOptional') }}</Label>
        <Textarea
          v-model="redeemBatchForm.description"
          rows="3"
          :placeholder="t('ui.forExampleHolidayCampaignOfflineChannelCommunityDistribution')"
        />
      </div>

      <div class="flex flex-wrap justify-end gap-2">
        <Button
          variant="outline"
          :disabled="!canExportLatestGeneratedRedeemCodes"
          @click="exportLatestGeneratedRedeemCodes"
        >
          {{ t('ui.exportLatestBatch') }}
        </Button>
        <Button
          :disabled="submittingRedeemBatch"
          @click="submitRedeemCodeBatch"
        >
          {{ submittingRedeemBatch ? t('ui.generating') : t('ui.generateRedemptionCodes') }}
        </Button>
      </div>

      <div
        v-if="latestGeneratedRedeemBatch"
        class="rounded-xl border border-border/60 bg-muted/20 p-3 text-xs text-muted-foreground"
      >
        {{ t('ui.latestGeneratedBatch') }} <span class="font-medium text-foreground">{{ latestGeneratedRedeemBatch.name }}</span>
        · {{ latestGeneratedRedeemCodes.length }} {{ t('ui.redemptionCodes') }}
      </div>
    </div>

    <div class="grid gap-5 xl:grid-cols-[1.1fr_1fr]">
      <div class="space-y-4">
        <div class="flex flex-wrap items-center gap-2">
          <Select v-model="redeemBatchStatusFilter">
            <SelectTrigger class="w-[180px]">
              <SelectValue :placeholder="t('ui.batchStatus')" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">
                {{ t('ui.allStatuses') }}
              </SelectItem>
              <SelectItem value="active">
                {{ t('ui.available4d99c97') }}
              </SelectItem>
              <SelectItem value="disabled">
                {{ t('ui.disableda8c3698') }}
              </SelectItem>
            </SelectContent>
          </Select>
          <div class="text-sm text-muted-foreground">
            {{ t('ui.total76e547a') }} {{ redeemBatchTotal }} {{ t('ui.batches') }}
          </div>
        </div>

        <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
          <div class="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{{ t('ui.batch') }}</TableHead>
                  <TableHead>{{ t('ui.faceValue') }}</TableHead>
                  <TableHead>{{ t('ui.quantity') }}</TableHead>
                  <TableHead>{{ t('ui.status6320b4a') }}</TableHead>
                  <TableHead class="text-right">
                    {{ t('ui.actionsed31fbb') }}
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow
                  v-for="batch in redeemBatches"
                  :key="batch.id"
                  class="hover:bg-muted/20"
                  :class="batch.id === selectedRedeemBatchId ? 'bg-muted/30 ring-1 ring-border/60' : ''"
                >
                  <TableCell class="min-w-[220px]">
                    <div class="text-sm font-medium">
                      {{ batch.name }}
                    </div>
                    <div class="text-xs text-muted-foreground mt-1">
                      {{ t('ui.expires') }} {{ formatDateTime(batch.expires_at) }}
                    </div>
                  </TableCell>
                  <TableCell class="tabular-nums">
                    {{ formatCurrency(batch.amount_usd) }}
                  </TableCell>
                  <TableCell class="text-xs text-muted-foreground">
                    {{ batch.redeemed_count }} / {{ batch.total_count }} {{ t('ui.used9845c16') }}
                  </TableCell>
                  <TableCell>
                    <Badge :variant="batch.status === 'active' ? 'success' : 'secondary'">
                      {{ batch.status === 'active' ? t('ui.available4d99c97') : t('ui.disableda8c3698') }}
                    </Badge>
                  </TableCell>
                  <TableCell class="text-right">
                    <div class="flex justify-end gap-2">
                      <Button
                        size="sm"
                        :variant="batch.id === selectedRedeemBatchId ? 'default' : 'outline'"
                        @click="selectRedeemBatch(batch)"
                      >
                        {{ batch.id === selectedRedeemBatchId ? t('ui.currentlyViewing') : t('ui.viewCodes') }}
                      </Button>
                      <Button
                        v-if="batch.status === 'active'"
                        size="sm"
                        variant="destructive"
                        @click="disableRedeemBatch(batch.id)"
                      >
                        {{ t('ui.disableBatch') }}
                      </Button>
                      <Button
                        v-if="batch.status === 'disabled'"
                        size="sm"
                        variant="destructive"
                        :disabled="batch.redeemed_count > 0"
                        @click="deleteRedeemBatch(batch)"
                      >
                        {{ t('ui.deleteBatch') }}
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
                <TableRow v-if="!loadingRedeemBatches && redeemBatches.length === 0">
                  <TableCell
                    colspan="5"
                    class="py-10"
                  >
                    <EmptyState
                      :title="t('ui.noRedemptionCodeBatches')"
                      :description="t('ui.createdBatchesWillAppearHere')"
                    />
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
        </div>

        <Pagination
          :current="redeemBatchPage"
          :total="redeemBatchTotal"
          :page-size="redeemBatchPageSize"
          @update:current="handleRedeemBatchPageChange"
          @update:page-size="handleRedeemBatchPageSizeChange"
        />
      </div>

      <div
        ref="redeemCodesPanelRef"
        class="space-y-4"
      >
        <div class="flex flex-wrap items-center justify-between gap-2">
          <div>
            <h4 class="text-sm font-semibold">
              {{ currentRedeemBatch?.name || t('ui.redemptionCodes59b83cf') }}
            </h4>
            <p class="text-xs text-muted-foreground mt-1">
              {{ currentRedeemBatch ? t('ui.valueValueValueRemaining', { value: formatCurrency(currentRedeemBatch.amount_usd), count: currentRedeemBatch.active_count }) : t('ui.selectABatchOnTheLeft') }}
            </p>
          </div>
          <div class="flex flex-wrap items-center gap-3">
            <div class="flex items-center gap-2">
              <span class="text-xs text-muted-foreground">{{ t('ui.revealCodes') }}</span>
              <Switch
                :model-value="showPlainRedeemCodes"
                :disabled="!canRevealPlainRedeemCodes"
                @update:model-value="showPlainRedeemCodes = Boolean($event)"
              />
            </div>
            <Select v-model="redeemCodeStatusFilter">
              <SelectTrigger class="w-[180px]">
                <SelectValue :placeholder="t('ui.codeStatus')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">
                  {{ t('ui.allStatuses') }}
                </SelectItem>
                <SelectItem value="active">
                  {{ t('ui.available4d99c97') }}
                </SelectItem>
                <SelectItem value="disabled">
                  {{ t('ui.disableda8c3698') }}
                </SelectItem>
                <SelectItem value="redeemed">
                  {{ t('ui.redeemed') }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <div class="text-xs text-muted-foreground">
          {{
            canRevealPlainRedeemCodes
              ? t('ui.thisBatchWasGeneratedInTheCurrentSessionAndIts')
              : t('ui.onlyTheLatestBatchGeneratedInThisSessionCanBe')
          }}
        </div>

        <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
          <div class="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{{ t('ui.redeemCodeabbb615') }}</TableHead>
                  <TableHead>{{ t('ui.status6320b4a') }}</TableHead>
                  <TableHead>{{ t('ui.redeemedBy') }}</TableHead>
                  <TableHead>{{ t('ui.linkedOrder0e96780') }}</TableHead>
                  <TableHead class="text-right">
                    {{ t('ui.actionsed31fbb') }}
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow
                  v-for="code in redeemCodes"
                  :key="code.id"
                >
                  <TableCell class="font-mono text-xs">
                    {{ displayRedeemCode(code) }}
                  </TableCell>
                  <TableCell>
                    <Badge :variant="redeemCodeStatusBadge(code.status)">
                      {{ redeemCodeStatusLabel(code.status) }}
                    </Badge>
                  </TableCell>
                  <TableCell class="text-xs text-muted-foreground">
                    {{ code.redeemed_by_user_name || code.redeemed_by_user_id || '-' }}
                  </TableCell>
                  <TableCell class="font-mono text-xs">
                    {{ code.redeemed_order_no || code.redeemed_payment_order_id || '-' }}
                  </TableCell>
                  <TableCell class="text-right">
                    <Button
                      v-if="code.status === 'active'"
                      size="sm"
                      variant="outline"
                      @click="disableRedeemCode(code.id)"
                    >
                      {{ t('ui.disable4e6fd0e') }}
                    </Button>
                  </TableCell>
                </TableRow>
                <TableRow v-if="!loadingRedeemCodes && redeemCodes.length === 0">
                  <TableCell
                    colspan="5"
                    class="py-10"
                  >
                    <EmptyState
                      :title="t('ui.noRedemptionCodes')"
                      :description="t('ui.selectABatchOnTheLeftToViewItsCodes')"
                    />
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
        </div>

        <Pagination
          :current="redeemCodePage"
          :total="redeemCodeTotal"
          :page-size="redeemCodePageSize"
          @update:current="handleRedeemCodePageChange"
          @update:page-size="handleRedeemCodePageSizeChange"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { computed, nextTick, reactive, ref, watch } from 'vue'
import {
  Badge,
  Button,
  Input,
  Label,
  Pagination,
  RefreshButton,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Textarea,
} from '@/components/ui'
import { EmptyState } from '@/components/common'
import {
  adminPaymentsApi,
  type RedeemCodeBatch,
  type RedeemCodeRecord,
} from '@/api/admin-payments'
import { useConfirm } from '@/composables/useConfirm'
import { useToast } from '@/composables/useToast'
import { parseApiError } from '@/utils/errorParser'
import { formatDateTime } from '@/utils/format'
import { log } from '@/utils/logger'
import {
  formatWalletCurrency as formatCurrency,
  redeemCodeStatusBadge,
  redeemCodeStatusLabel,
} from '../utils/walletDisplay'

/**
 * 钱包管理的兑换码标签页：批量生成、批次列表与兑换码明细。
 * 数据由父组件在切到该标签时通过 load() 触发加载。
 */
const { success, error: showError } = useToast()
const { confirmDanger } = useConfirm()

const loadingRedeemBatches = ref(false)

const loadingRedeemCodes = ref(false)

const submittingRedeemBatch = ref(false)

const redeemBatches = ref<RedeemCodeBatch[]>([])

const redeemBatchTotal = ref(0)

const redeemBatchPage = ref(1)

const redeemBatchPageSize = ref(20)

const redeemBatchStatusFilter = ref('all')

const redeemCodes = ref<RedeemCodeRecord[]>([])

const redeemCodeTotal = ref(0)

const redeemCodePage = ref(1)

const redeemCodePageSize = ref(20)

const redeemCodeStatusFilter = ref('all')

const selectedRedeemBatchId = ref<string | null>(null)

const currentRedeemBatch = ref<RedeemCodeBatch | null>(null)

const latestGeneratedRedeemBatch = ref<RedeemCodeBatch | null>(null)

const latestGeneratedRedeemCodes = ref<Array<{ id: string; code: string; masked_code: string }>>([])

const showPlainRedeemCodes = ref(false)

const redeemCodesPanelRef = ref<HTMLElement | null>(null)

const redeemBatchForm = reactive({
  name: '',
  amount_usd: 10,
  total_count: 20,
  expires_at: '',
  description: '',
})

const canRevealPlainRedeemCodes = computed(
  () =>
    !!currentRedeemBatch.value &&
    currentRedeemBatch.value.id === latestGeneratedRedeemBatch.value?.id &&
    latestGeneratedRedeemCodes.value.length > 0
)

const canExportLatestGeneratedRedeemCodes = computed(
  () => !!latestGeneratedRedeemBatch.value && latestGeneratedRedeemCodes.value.length > 0
)

watch(redeemBatchStatusFilter, () => {
  redeemBatchPage.value = 1
  void loadBatches()
})

watch(redeemCodeStatusFilter, () => {
  redeemCodePage.value = 1
  void loadRedeemCodes()
})

watch(canRevealPlainRedeemCodes, (enabled) => {
  if (!enabled) {
    showPlainRedeemCodes.value = false
  }
})

async function loadBatches() {
  loadingRedeemBatches.value = true
  try {
    const offset = (redeemBatchPage.value - 1) * redeemBatchPageSize.value
    const resp = await adminPaymentsApi.listRedeemCodeBatches({
      status: redeemBatchStatusFilter.value !== 'all' ? redeemBatchStatusFilter.value : undefined,
      limit: redeemBatchPageSize.value,
      offset,
    })
    redeemBatches.value = resp.items
    redeemBatchTotal.value = resp.total

    if (selectedRedeemBatchId.value) {
      const latest = resp.items.find(item => item.id === selectedRedeemBatchId.value)
      if (latest) {
        currentRedeemBatch.value = latest
        await loadRedeemCodes(latest.id)
      } else {
        selectedRedeemBatchId.value = null
        currentRedeemBatch.value = null
        redeemCodes.value = []
        redeemCodeTotal.value = 0
      }
    }
  } catch (error) {
    log.error('加载兑换码批次失败:', error)
    showError(parseApiError(error, '加载兑换码批次失败'))
  } finally {
    loadingRedeemBatches.value = false
  }
}

async function loadRedeemCodes(batchId = selectedRedeemBatchId.value || undefined) {
  if (!batchId) {
    redeemCodes.value = []
    redeemCodeTotal.value = 0
    return
  }
  loadingRedeemCodes.value = true
  try {
    const offset = (redeemCodePage.value - 1) * redeemCodePageSize.value
    const resp = await adminPaymentsApi.listRedeemCodes(batchId, {
      status: redeemCodeStatusFilter.value !== 'all' ? redeemCodeStatusFilter.value : undefined,
      limit: redeemCodePageSize.value,
      offset,
    })
    currentRedeemBatch.value = resp.batch
    selectedRedeemBatchId.value = resp.batch.id
    redeemCodes.value = resp.items
    redeemCodeTotal.value = resp.total
  } catch (error) {
    log.error('加载兑换码列表失败:', error)
    showError(parseApiError(error, '加载兑换码列表失败'))
  } finally {
    loadingRedeemCodes.value = false
  }
}

async function selectRedeemBatch(batch: RedeemCodeBatch) {
  currentRedeemBatch.value = batch
  selectedRedeemBatchId.value = batch.id
  redeemCodePage.value = 1
  await loadRedeemCodes(batch.id)
  await nextTick()
  redeemCodesPanelRef.value?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

function exportRedeemCodesCsv(batch: RedeemCodeBatch, codes: Array<{ id: string; code: string; masked_code: string }>) {
  const header = ['id', 'batch_name', 'code', 'masked_code']
  const rows = codes.map(code => [code.id, batch.name, code.code, code.masked_code])
  const csv = [header, ...rows]
    .map(row => row.map(cell => `"${String(cell).replaceAll('"', '""')}"`).join(','))
    .join('\n')
  const blob = new Blob([`\uFEFF${csv}`], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `redeem-codes-${batch.name}-${batch.id}.csv`
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  URL.revokeObjectURL(url)
}

async function submitRedeemCodeBatch() {
  if (!redeemBatchForm.name.trim()) {
    showError('请填写批次名称')
    return
  }
  if (!redeemBatchForm.amount_usd || redeemBatchForm.amount_usd <= 0) {
    showError('请填写有效面额')
    return
  }
  if (!redeemBatchForm.total_count || redeemBatchForm.total_count <= 0) {
    showError('请填写有效数量')
    return
  }

  submittingRedeemBatch.value = true
  try {
    const payload = {
      name: redeemBatchForm.name.trim(),
      amount_usd: redeemBatchForm.amount_usd,
      total_count: redeemBatchForm.total_count,
      expires_at: redeemBatchForm.expires_at ? new Date(redeemBatchForm.expires_at).toISOString() : undefined,
      description: redeemBatchForm.description.trim() || undefined,
    }
    const resp = await adminPaymentsApi.createRedeemCodeBatch(payload)
    latestGeneratedRedeemBatch.value = resp.batch
    latestGeneratedRedeemCodes.value = resp.codes
    showPlainRedeemCodes.value = true
    success(t('ui.redemptionCodeBatchCreated'))
    redeemBatchForm.name = ''
    redeemBatchForm.description = ''
    redeemBatchForm.expires_at = ''
    currentRedeemBatch.value = resp.batch
    selectedRedeemBatchId.value = resp.batch.id
    await loadBatches()
    await loadRedeemCodes(resp.batch.id)
  } catch (error) {
    log.error('创建兑换码批次失败:', error)
    showError(parseApiError(error, '创建兑换码批次失败'))
  } finally {
    submittingRedeemBatch.value = false
  }
}

function exportLatestGeneratedRedeemCodes() {
  if (!latestGeneratedRedeemBatch.value || latestGeneratedRedeemCodes.value.length === 0) {
    showError('当前没有可导出的新生成兑换码')
    return
  }
  exportRedeemCodesCsv(latestGeneratedRedeemBatch.value, latestGeneratedRedeemCodes.value)
  success(t('ui.csvExported'))
}

function displayRedeemCode(code: RedeemCodeRecord) {
  if (!showPlainRedeemCodes.value || !canRevealPlainRedeemCodes.value) {
    return code.masked_code
  }
  return latestGeneratedRedeemCodes.value.find(item => item.id === code.id)?.code || code.masked_code
}

async function disableRedeemBatch(batchId: string) {
  try {
    await adminPaymentsApi.disableRedeemCodeBatch(batchId)
    success(t('ui.batchDisabled'))
    await loadBatches()
  } catch (error) {
    log.error('停用兑换码批次失败:', error)
    showError(parseApiError(error, '停用兑换码批次失败'))
  }
}

async function deleteRedeemBatch(batch: RedeemCodeBatch) {
  if (batch.redeemed_count > 0) {
    showError('已有兑换记录的批次不能删除')
    return
  }
  if (!(await confirmDanger(t('ui.deleteBatchValueThisCannotBeUndone', { name: batch.name }), t('ui.deleteBatch')))) {
    return
  }

  try {
    await adminPaymentsApi.deleteRedeemCodeBatch(batch.id)
    success(t('ui.batchDeleted'))
    if (selectedRedeemBatchId.value === batch.id) {
      selectedRedeemBatchId.value = null
      currentRedeemBatch.value = null
      redeemCodes.value = []
      redeemCodeTotal.value = 0
      showPlainRedeemCodes.value = false
    }
    if (latestGeneratedRedeemBatch.value?.id === batch.id) {
      latestGeneratedRedeemBatch.value = null
      latestGeneratedRedeemCodes.value = []
      showPlainRedeemCodes.value = false
    }
    await loadBatches()
  } catch (error) {
    log.error('删除兑换码批次失败:', error)
    showError(parseApiError(error, '删除兑换码批次失败'))
  }
}

async function disableRedeemCode(codeId: string) {
  try {
    await adminPaymentsApi.disableRedeemCode(codeId)
    success(t('ui.redemptionCodeDisabled'))
    await Promise.all([loadRedeemCodes(), loadBatches()])
  } catch (error) {
    log.error('停用兑换码失败:', error)
    showError(parseApiError(error, '停用兑换码失败'))
  }
}

function handleRedeemBatchPageChange(page: number) {
  redeemBatchPage.value = page
  void loadBatches()
}

function handleRedeemBatchPageSizeChange(size: number) {
  redeemBatchPageSize.value = size
  redeemBatchPage.value = 1
  void loadBatches()
}

function handleRedeemCodePageChange(page: number) {
  redeemCodePage.value = page
  void loadRedeemCodes()
}

function handleRedeemCodePageSizeChange(size: number) {
  redeemCodePageSize.value = size
  redeemCodePage.value = 1
  void loadRedeemCodes()
}

defineExpose({ load: loadBatches })

</script>
