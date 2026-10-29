<template>
  <!-- 提供商详情抽屉 -->
  <Drawer
    :open="open && (loading || !!provider)"
    appear
    backdrop-class="bg-black/30"
    :close-on-backdrop="!hasBlockingDialogOpen"
    :close-on-escape="!hasBlockingDialogOpen"
    @close="handleClose"
  >
    <!-- 加载状态 -->
    <div
      v-if="loading"
      class="flex items-center justify-center py-12"
    >
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
    </div>

    <template v-else-if="provider">
      <ProviderDetailHeader
        v-model:provider-proxy-popover-open="providerProxyPopoverOpen"
        :provider="provider"
        :endpoints="endpoints"
        :loading-provider-endpoints="loadingProviderEndpoints"
        :system-format-conversion-enabled="systemFormatConversionEnabled"
        :has-failover-rules="hasFailoverRules"
        :provider-proxy-node-name="getProviderProxyNodeName()"
        :saving-provider-proxy="savingProviderProxy"
        @toggle-format-conversion="toggleFormatConversion"
        @toggle-keep-priority-on-conversion="toggleKeepPriorityOnConversion"
        @open-failover-rules="failoverRulesDialogOpen = true"
        @set-provider-proxy="setProviderProxy"
        @clear-provider-proxy="clearProviderProxy"
        @edit="$emit('edit', $event)"
        @toggle-status="$emit('toggleStatus', $event)"
        @close="handleClose"
        @edit-endpoint="handleEditEndpoint"
        @add-endpoint="showAddEndpointDialog"
      />

      <div class="space-y-6 p-4 sm:p-6">
        <!-- 配额使用情况 -->
        <ProviderMonthlyQuotaCard
          v-if="provider.billing_type === 'monthly_quota' && provider.monthly_quota_usd"
          :used="provider.monthly_used_usd"
          :quota="provider.monthly_quota_usd"
          :reset-day="provider.quota_reset_day"
        />

        <!-- 密钥管理 -->
        <Card class="overflow-hidden">
          <div class="p-4 border-b border-border/60">
            <div class="flex items-center justify-between">
              <h3 class="text-sm font-semibold">
                {{ isKeyManagedProviderType(provider.provider_type) ? t('ui.keyManagement') : t('ui.accountManagement') }}
              </h3>
              <div class="flex flex-wrap items-center justify-end gap-2">
                <Button
                  v-if="endpoints.length > 0 && provider.provider_type === 'custom'"
                  variant="outline"
                  size="sm"
                  class="h-9"
                  @click="keyBatchImportDialogOpen = true"
                >
                  <ListPlus class="mr-1.5 h-3.5 w-3.5" /> {{ t('ui.bulkImport') }}
                </Button>
                <Button
                  v-if="endpoints.length > 0"
                  variant="outline"
                  size="sm"
                  class="h-9"
                  @click="handleAddKeyToFirstEndpoint"
                >
                  <Plus class="w-3.5 h-3.5 mr-1.5" />
                  {{ isKeyManagedProviderType(provider.provider_type) ? t('guide.nav.concepts.addKey') : t('ui.addAccount') }}
                </Button>
              </div>
            </div>
          </div>

          <!-- 密钥列表 -->
          <div
            v-if="loadingProviderKeys && allKeys.length === 0"
            class="flex items-center justify-center gap-2 py-12 text-sm text-muted-foreground"
          >
            <Loader2 class="w-4 h-4 animate-spin" />
            {{ t('ui.loadingf020e46') }}{{ isKeyManagedProviderType(provider.provider_type) ? t('ui.key') : t('ui.account') }}
          </div>

          <div
            v-else-if="allKeys.length > 0"
            class="divide-y divide-border/40"
            :class="shouldPaginateKeys && 'flex flex-col'"
          >
            <div
              v-for="({ key, endpoint }, localIdx) in paginatedKeys"
              :key="key.id"
              class="px-4 py-2.5 hover:bg-muted/30 transition-colors group/item"
              :class="{
                'opacity-50': keyDragState.isDragging && keyDragState.draggedIndex === getGlobalKeyIndex(localIdx),
                'bg-primary/5 border-l-2 border-l-primary': keyDragState.targetIndex === getGlobalKeyIndex(localIdx) && keyDragState.isDragging,
                'opacity-40 bg-muted/20': !key.is_active
              }"
              draggable="true"
              @dragstart="handleKeyDragStart($event, getGlobalKeyIndex(localIdx))"
              @dragend="handleKeyDragEnd"
              @dragover="handleKeyDragOver($event, getGlobalKeyIndex(localIdx))"
              @dragleave="handleKeyDragLeave"
              @drop="handleKeyDrop($event, getGlobalKeyIndex(localIdx))"
            >
              <!-- 第一行：名称 + 状态 + 操作按钮 -->
              <div class="flex items-center justify-between gap-2">
                <div class="flex items-center gap-2 flex-1 min-w-0">
                  <!-- 拖拽手柄 -->
                  <div class="cursor-grab active:cursor-grabbing text-muted-foreground/30 group-hover/item:text-muted-foreground transition-colors shrink-0">
                    <GripVertical class="w-4 h-4" />
                  </div>
                  <ProviderKeyIdentityBlock
                    :api-key="key"
                    :masked-secret-label="getProviderMaskedSecretLabel(key, provider.provider_type)"
                    :oauth-plan-label="key.oauth_plan_type ? formatOAuthPlanType(key.oauth_plan_type ?? '') : null"
                    :oauth-plan-class="key.oauth_plan_type ? getOAuthPlanTypeClass(key.oauth_plan_type) : ''"
                    :oauth-org-badge="getOAuthOrgBadge(key)"
                    :kiro-subscription-label="shouldShowKiroSubscriptionBadge(key) ? getKiroSubscriptionBadgeLabel(key) : null"
                    :kiro-subscription-class="shouldShowKiroSubscriptionBadge(key) ? getOAuthPlanTypeClass(getKiroSubscriptionBadgeLabel(key)) : ''"
                    :can-export-credential="canExportOAuthCredential(key)"
                    :show-o-auth-refresh-control="shouldShowOAuthRefreshControl(key, provider.provider_type)"
                    :account-level-block="isAccountLevelBlock(key)"
                    :oauth-status="getKeyOAuthExpires(key)"
                    :oauth-status-title="getOAuthStatusTitle(key)"
                    :oauth-refresh-button-title="getOAuthRefreshButtonTitle(key)"
                    :can-refresh-credential="canRefreshOAuthCredential(key)"
                    :clearing-o-auth-invalid="clearingOAuthInvalidKeyId === key.id"
                    :refreshing-o-auth="refreshingOAuthKeyId === key.id"
                    :antigravity-inactive="provider.provider_type === 'antigravity' && key.is_active && isOAuthManagedCredential(key) && !hasAntigravityQuotaDisplayData(key)"
                    @copy-name="copyToClipboard"
                    @download-credential="downloadRefreshToken(key)"
                    @copy-full-key="copyFullKey(key)"
                    @clear-o-auth-invalid="handleClearOAuthInvalid(key)"
                    @refresh-o-auth="handleRefreshOAuth(key)"
                  />
                </div>
                <ProviderKeyActionCluster
                  :api-key="key"
                  :provider-type="provider.provider_type ?? null"
                  :recoverable="isKeyRecoverable(key)"
                  :recover-title="getRecoverKeyTitle(key)"
                  :circuit-breaker-title="getKeyCircuitBreakerTitle(key)"
                  :circuit-probe-countdown="getKeyCircuitProbeCountdown(key)"
                  :health-score-bar-class="getHealthScoreBarColor(key.health_score || 0)"
                  :health-score-text-class="getHealthScoreColor(key.health_score || 0)"
                  :proxy-popover-open="proxyPopoverOpenKeyId === key.id"
                  :proxy-node-name="getKeyProxyNodeName(key) ?? undefined"
                  :saving-proxy="savingProxyKeyId === key.id"
                  :toggling="togglingKeyId === key.id"
                  @recover="handleRecoverKey(key)"
                  @permissions="handleKeyPermissions(key)"
                  @update:proxy-popover-open="(v: boolean) => handleProxyPopoverToggle(key.id, v)"
                  @clear-proxy="clearKeyProxy(key)"
                  @set-proxy="(v: string) => setKeyProxy(key, v)"
                  @edit="handleEditKey(endpoint, key)"
                  @open-antigravity-quota="openAntigravityQuotaDialog(key)"
                  @toggle-active="toggleKeyActive(key)"
                  @delete="handleDeleteKey(key)"
                />
              </div>
              <ProviderKeyQuotaSummary
                :provider-key="key"
                :provider-type="provider.provider_type"
                :refreshing="refreshingQuota"
                :consuming-reset-credit="consumingCodexResetCreditKeyId === key.id"
                :can-consume-reset-credit="canConsumeCodexResetCredit(key)"
                :countdown-tick="countdownTick"
                @consume-reset-credit="handleConsumeCodexResetCredit(key)"
              />
              <!-- 第二行：优先级 + API 格式（展开显示） + 统计信息 -->
              <div class="flex items-center gap-1.5 mt-1 text-[11px] text-muted-foreground">
                <!-- 优先级放最前面，支持点击编辑 -->
                <span
                  v-if="editingPriorityKey !== key.id"
                  :title="t('ui.editPriority')"
                  class="font-medium text-foreground/80 cursor-pointer hover:text-primary hover:underline"
                  @click="startEditPriority(key)"
                >P{{ key.internal_priority }}</span>
                <input
                  v-else
                  ref="priorityInputRef"
                  v-model="editingPriorityValue"
                  type="text"
                  inputmode="numeric"
                  pattern="[0-9]*"
                  class="w-8 h-5 px-1 text-[11px] text-center border rounded bg-background focus:outline-none focus:ring-1 focus:ring-primary font-medium text-foreground/80"
                  @keydown="(e) => handlePriorityKeydown(e, key)"
                  @blur="handlePriorityBlur(key)"
                >
                <!-- 自动获取模型状态 -->
                <template v-if="key.auto_fetch_models">
                  <span class="text-muted-foreground/40">|</span>
                  <span
                    class="cursor-help"
                    :class="key.last_models_fetch_error ? 'text-amber-600 dark:text-amber-400' : ''"
                    :title="getAutoFetchStatusTitle(key)"
                  >
                    {{ key.last_models_fetch_error ? t('models.pricingSource.syncFailed') : t('ui.autoSync') }}
                  </span>
                </template>
                <!-- RPM 限制信息（第二位） -->
                <template v-if="key.rpm_limit || key.is_adaptive">
                  <span class="text-muted-foreground/40">|</span>
                  <span v-if="key.is_adaptive">
                    {{ key.learned_rpm_limit != null ? `${key.learned_rpm_limit}` : t('ui.probing') }} RPM
                    <span class="text-muted-foreground/60">({{ t('ui.adaptive') }})</span>
                  </span>
                  <span v-else>{{ key.rpm_limit }} RPM</span>
                </template>
                <span class="text-muted-foreground/40">|</span>
                <!-- API 格式：展开显示每个格式、倍率、熔断状态 -->
                <template
                  v-for="(format, idx) in getKeyApiFormats(key, endpoint)"
                  :key="format"
                >
                  <span
                    v-if="idx > 0"
                    class="text-muted-foreground/40"
                  >/</span>
                  <span :class="{ 'text-destructive': isFormatCircuitOpen(key, format) }">
                    {{ formatApiFormatShort(format) }}
                  </span>
                  <span
                    v-if="editingMultiplierKey !== key.id || editingMultiplierFormat !== format"
                    :title="t('ui.editMultiplier')"
                    class="cursor-pointer hover:text-primary hover:underline"
                    :class="{ 'text-destructive': isFormatCircuitOpen(key, format) }"
                    @click="startEditMultiplier(key, format)"
                  >{{ getKeyRateMultiplier(key, format) }}x</span>
                  <input
                    v-else
                    ref="multiplierInputRef"
                    v-model="editingMultiplierValue"
                    type="text"
                    inputmode="decimal"
                    pattern="[0-9]*\.?[0-9]*"
                    class="w-10 h-5 px-1 text-[11px] text-center border rounded bg-background focus:outline-none focus:ring-1 focus:ring-primary font-medium text-foreground/80"
                    @keydown="(e) => handleMultiplierKeydown(e, key, format)"
                    @blur="handleMultiplierBlur(key, format)"
                  >
                  <span
                    v-if="getFormatProbeCountdown(key, format)"
                    :class="{ 'text-destructive': isFormatCircuitOpen(key, format) }"
                  >{{ getFormatProbeCountdown(key, format) }}</span>
                </template>
              </div>
            </div>
            <!-- 分页控制 -->
            <div
              v-if="shouldPaginateKeys"
              class="px-4 py-2 flex items-center justify-between text-xs text-muted-foreground mt-auto"
            >
              <span>{{ t('ui.total76e547a') }} {{ providerKeysTotal }} {{ t('ui.items3a11681') }}{{ isKeyManagedProviderType(provider.provider_type) ? t('ui.key') : t('ui.account') }}</span>
              <div class="flex items-center gap-1.5">
                <Button
                  variant="ghost"
                  size="sm"
                  class="h-6 px-2 text-xs"
                  :disabled="loadingProviderKeys || currentKeyPage <= 1"
                  @click="goToKeyPage(currentKeyPage - 1)"
                >
                  ‹
                </Button>
                <span class="tabular-nums">{{ currentKeyPage }} / {{ totalKeyPages }}</span>
                <Button
                  variant="ghost"
                  size="sm"
                  class="h-6 px-2 text-xs"
                  :disabled="loadingProviderKeys || currentKeyPage >= totalKeyPages"
                  @click="goToKeyPage(currentKeyPage + 1)"
                >
                  ›
                </Button>
              </div>
            </div>
          </div>

          <!-- 空状态 -->
          <div
            v-else
            class="p-8 text-center text-muted-foreground"
          >
            <Key class="w-12 h-12 mx-auto mb-3 opacity-50" />
            <p class="text-sm">
              {{ isKeyManagedProviderType(provider.provider_type) ? t('ui.noKeysConfigured') : t('ui.noAccountsConfigured') }}
            </p>
            <p class="text-xs mt-1">
              {{ endpoints.length > 0
                ? isKeyManagedProviderType(provider.provider_type) ? t('ui.addYourFirstKeyToGetStarted') : t('ui.addYourFirstAccountToGetStarted')
                : t('ui.addAnEndpointBeforeAddingKeys') }}
            </p>
          </div>
        </Card>

        <!-- 模型查看 -->
        <ModelsTab
          v-if="provider"
          :key="`models-${provider.id}`"
          :provider="provider"
          :models="providerModels"
          :endpoints="endpoints"
          :provider-keys="providerKeys"
          :loading="loadingProviderModels"
          @edit-model="handleEditModel"
          @batch-assign="handleBatchAssign"
          @refresh="loadEndpoints"
        />

        <!-- 模型映射 -->
        <ModelMappingTab
          v-if="provider"
          ref="modelMappingTabRef"
          :key="`mapping-${provider.id}`"
          :provider="provider"
          :endpoints="endpoints"
          :provider-keys="providerKeys"
          :models="providerModels"
          :mapping-preview="providerMappingPreview"
          :loading="loadingProviderMappingPreview"
          @refresh="handleModelMappingChanged"
        />
      </div>
    </template>
  </Drawer>

  <!-- 端点表单对话框（管理/编辑） -->
  <EndpointFormDialog
    v-if="provider && open && endpointDialogOpen"
    v-model="endpointDialogOpen"
    :provider="provider"
    :endpoints="endpoints"
    :system-format-conversion-enabled="systemFormatConversionEnabled"
    :provider-format-conversion-enabled="provider.enable_format_conversion"
    @endpoint-created="handleEndpointChanged"
    @endpoint-updated="handleEndpointChanged"
  />

  <!-- 密钥编辑对话框 -->
  <KeyFormDialog
    v-if="open && keyFormDialogOpen"
    :open="keyFormDialogOpen"
    :endpoint="currentEndpoint"
    :editing-key="editingKey"
    :provider-id="provider ? provider.id : null"
    :provider-type="provider?.provider_type || null"
    :available-api-formats="availableKeyApiFormats"
    @close="keyFormDialogOpen = false"
    @saved="handleKeyChanged"
  />

  <ProviderKeyBatchImportDialog
    v-if="open && keyBatchImportDialogOpen && provider?.provider_type === 'custom'"
    :open="keyBatchImportDialogOpen"
    :provider-id="provider.id"
    :provider-name="provider.name"
    :available-api-formats="availableKeyApiFormats"
    @close="keyBatchImportDialogOpen = false"
    @saved="handleKeyChanged"
  />

  <!-- OAuth 账号对话框 -->
  <OAuthAccountDialog
    v-if="open && oauthAccountDialogOpen && provider"
    :open="oauthAccountDialogOpen"
    :provider-id="provider.id"
    :provider-type="provider.provider_type ?? null"
    @close="oauthAccountDialogOpen = false"
    @saved="handleKeyChanged"
  />

  <!-- OAuth 密钥编辑对话框 -->
  <OAuthKeyEditDialog
    v-if="open && oauthKeyEditDialogOpen"
    :open="oauthKeyEditDialogOpen"
    :editing-key="editingKey"
    @close="oauthKeyEditDialogOpen = false"
    @saved="handleKeyChanged"
  />

  <!-- 模型权限对话框 -->
  <KeyAllowedModelsEditDialog
    v-if="open && keyPermissionsDialogOpen"
    :open="keyPermissionsDialogOpen"
    :api-key="editingKey"
    :provider-id="providerId || ''"
    @close="keyPermissionsDialogOpen = false"
    @saved="handleKeyChanged"
  />

  <!-- 删除密钥确认对话框 -->
  <AlertDialog
    v-if="open && deleteKeyConfirmOpen"
    :model-value="deleteKeyConfirmOpen"
    :title="t('ui.deleteKey')"
    :description="formatDeleteKeyConfirmDescription()"
    :confirm-text="t('ui.delete')"
    :cancel-text="t('common.cancel')"
    type="danger"
    @update:model-value="deleteKeyConfirmOpen = $event"
    @confirm="confirmDeleteKey"
    @cancel="deleteKeyConfirmOpen = false"
  />

  <!-- 添加/编辑模型对话框 -->
  <ProviderModelFormDialog
    v-if="open && modelFormDialogOpen && provider"
    :open="modelFormDialogOpen"
    :provider-id="provider.id"
    :provider-name="provider.name"
    :editing-model="editingModel"
    @update:open="modelFormDialogOpen = $event"
    @saved="handleModelSaved"
  />

  <!-- 批量关联模型对话框 -->
  <BatchAssignModelsDialog
    v-if="open && batchAssignDialogOpen && provider"
    :open="batchAssignDialogOpen"
    :provider-id="provider.id"
    :provider-name="provider.name"
    @update:open="handleBatchAssignDialogOpenUpdate"
    @changed="handleBatchAssignChanged"
  />

  <!-- Antigravity 配额详情弹窗 -->
  <AntigravityQuotaDialog
    v-if="antigravityQuotaDialogKey"
    :open="antigravityQuotaDialogOpen"
    :metadata="antigravityQuotaDialogKey.upstream_metadata ?? null"
    :quota-snapshot="antigravityQuotaDialogKey.status_snapshot?.quota ?? null"
    :key-name="antigravityQuotaDialogKey.name || t('ui.unnamedKey')"
    :provider-id="providerId ?? undefined"
    :key-id="antigravityQuotaDialogKey.id"
    @update:open="antigravityQuotaDialogOpen = $event"
  />

  <!-- 故障转移规则弹窗 -->
  <FailoverRulesDialog
    v-if="open && failoverRulesDialogOpen"
    :open="failoverRulesDialogOpen"
    :provider="provider ?? null"
    @update:open="failoverRulesDialogOpen = $event"
    @saved="loadProvider()"
  />
</template>

<script setup lang="ts">
import { ref, watch, computed, nextTick } from 'vue'
import {
  Plus,
  Key,
  ListPlus,
  Loader2,
  GripVertical,
} from 'lucide-vue-next'
import { parseApiError } from '@/utils/errorParser'
import Drawer from '@/components/ui/drawer/Drawer.vue'
import { useI18n } from '@/i18n'
import Button from '@/components/ui/button.vue'
import Card from '@/components/ui/card.vue'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { useClipboard } from '@/composables/useClipboard'
import { useCountdownTimer, formatCountdown } from '@/composables/useCountdownTimer'
import {
  getProvider,
  getProviderEndpoints,
  updateProvider,
  getProviderModels,
  getProviderMappingPreview,
  type ProviderMappingPreviewResponse,
  type ProviderWithEndpointsSummary,
} from '@/api/endpoints'
import { adminApi } from '@/api/admin'
import KeyFormDialog from './KeyFormDialog.vue'
import KeyAllowedModelsEditDialog from './KeyAllowedModelsEditDialog.vue'
import ModelsTab from './provider-tabs/ModelsTab.vue'
import BatchAssignModelsDialog from './BatchAssignModelsDialog.vue'
import OAuthAccountDialog from './OAuthAccountDialog.vue'
import OAuthKeyEditDialog from './OAuthKeyEditDialog.vue'
import ModelMappingTab from '@/features/providers/components/provider-tabs/ModelMappingTab.vue'
import EndpointFormDialog from '@/features/providers/components/EndpointFormDialog.vue'
import ProviderModelFormDialog from '@/features/providers/components/ProviderModelFormDialog.vue'
import AlertDialog from '@/components/common/AlertDialog.vue'
import AntigravityQuotaDialog from '@/features/providers/components/AntigravityQuotaDialog.vue'
import FailoverRulesDialog from '@/features/providers/components/FailoverRulesDialog.vue'
import ProviderDetailHeader from '@/features/providers/components/ProviderDetailHeader.vue'
import ProviderKeyBatchImportDialog from '@/features/providers/components/ProviderKeyBatchImportDialog.vue'
import ProviderKeyActionCluster from '@/features/providers/components/ProviderKeyActionCluster.vue'
import ProviderKeyIdentityBlock from '@/features/providers/components/ProviderKeyIdentityBlock.vue'
import ProviderMonthlyQuotaCard from '@/features/providers/components/ProviderMonthlyQuotaCard.vue'
import ProviderKeyQuotaSummary from '@/features/providers/components/ProviderKeyQuotaSummary.vue'
import { useProxyNodesStore } from '@/stores/proxy-nodes'
import { refreshQuotaInBackground } from '@/features/providers/utils/refreshQuotaInBackground'
import { useProviderKeyQuotaPresentation } from '../composables/useProviderKeyQuotaPresentation'
import {
  getClaudeCodeQuotaUpdatedAt,
  getGeminiCliQuotaUpdatedAt,
  hasGeminiCliQuotaDisplayData,
} from '../composables/useProviderKeyQuotaPresentation'
import {
  getCodexQuotaDisplay,
  hasCodexQuotaDisplayData,
  getCodexResetCreditAvailableCount,
  getVisibleCodexResetCreditItems,
  getPendingCodexResetCreditIdempotencyKey,
  getCodexCredentialGeneration,
  hasPendingCodexResetCredit,
  getKiroQuotaDisplay,
  hasKiroQuotaDisplayData,
  getXaiQuotaDisplay,
  hasXaiQuotaDisplayData,
  getGrokQuotaDisplay,
  getWindsurfQuotaDisplay,
  hasGrokQuotaDisplayData,
  hasWindsurfQuotaDisplayData,
  getChatGPTWebQuotaDisplay,
  hasChatGPTWebQuotaDisplayData,
  getAntigravityQuotaUpdatedAt,
  getKiroSubscriptionBadgeLabel,
} from '@/features/providers/utils/providerKeyQuotaDisplay'
import {
  deleteEndpointKey,
  recoverKeyHealth,
  getProviderKeysPage,
  updateProviderKey,
  revealEndpointKey,
  exportKey,
  refreshProviderOAuth,
  refreshProviderQuota,
  consumeCodexResetCredit,
  clearOAuthInvalid,
  type ProviderEndpoint,
  type EndpointAPIKey,
  type Model,
  API_FORMAT_ORDER,
  sortApiFormats,
} from '@/api/endpoints'
import type {
  UpstreamMetadata,
  QuotaStatusSnapshot,
} from '@/api/endpoints/types'
import { formatApiFormatShort } from '@/api/endpoints/types/api-format'
import { isOAuthAccountProviderType, isKeyManagedProviderType } from '../utils/providerTypeUtils'
import { getOAuthOrgBadge } from '@/features/providers/utils/oauthIdentity'
import { getOAuthRefreshFeedback } from '@/features/providers/utils/oauthRefreshFeedback'
import {
  canEditOAuthCredential,
  canExportOAuthCredential,
  canRefreshOAuthCredential,
  isOAuthManagedCredential,
  getProviderMaskedSecretLabel,
  shouldShowOAuthRefreshControl,
} from '@/features/providers/utils/providerKeyAuth'
import {
  getAccountStatusDisplay,
  getAccountStatusTitle,
  getOAuthRefreshButtonTitle as resolveOAuthRefreshButtonTitle,
  getOAuthStatusDisplay,
  getOAuthStatusDisplayWithFallback,
  getOAuthStatusTitle as resolveOAuthStatusTitle,
} from '@/features/providers/utils/providerKeyStatus'
import {
  clearPendingCodexResetCreditIdempotencyKey,
  clearPendingCodexResetCreditIdempotencyKeyForOutcome,
  createCodexResetCreditIdempotencyKey,
  formatCodexResetCreditExpiresAt,
  readPendingCodexResetCreditIdempotencyKey,
  rememberPendingCodexResetCreditIdempotencyKey,
} from './codex-reset-credit-display'

// 扩展端点类型,包含密钥列表
interface ProviderEndpointWithKeys extends ProviderEndpoint {
  keys?: EndpointAPIKey[]
  rpm_limit?: number
}

interface Props {
  providerId: string | null
  open: boolean
  initialProvider?: ProviderWithEndpointsSummary | null
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'update:open', value: boolean): void
  (e: 'edit', provider: ProviderWithEndpointsSummary): void
  (e: 'toggleStatus', provider: ProviderWithEndpointsSummary): void
  (e: 'refresh'): void
}>()

const { t, locale } = useI18n()

const { error: showError, success: showSuccess, warning: showWarning } = useToast()
const { confirm } = useConfirm()
const { copyToClipboard } = useClipboard()
const { tick: countdownTick, start: startCountdownTimer, stop: stopCountdownTimer } = useCountdownTimer()
const {
  hasAntigravityQuotaDisplayData,
  hasClaudeCodeQuotaDisplayData,
} = useProviderKeyQuotaPresentation(countdownTick)

function localizedApiError(err: unknown, fallback: string): string {
  return parseApiError(err, fallback)
}

const loading = ref(false)
const provider = ref<ProviderWithEndpointsSummary | null>(null)
const endpoints = ref<ProviderEndpointWithKeys[]>([])
const providerKeys = ref<EndpointAPIKey[]>([])  // Provider 级别的 keys
const providerModels = ref<Model[]>([])  // Provider 级别的 models
const providerMappingPreview = ref<ProviderMappingPreviewResponse | null>(null)  // 映射预览
const loadingProviderEndpoints = ref(false)
const loadingProviderKeys = ref(false)
const loadingProviderModels = ref(false)
const loadingProviderMappingPreview = ref(false)
let providerLoadRequestId = 0
let endpointsLoadRequestId = 0
let keysLoadRequestId = 0
let mappingPreviewLoadRequestId = 0
const DEFAULT_PROVIDER_KEYS_PAGE_SIZE = 3
const CUSTOM_PROVIDER_KEYS_PAGE_SIZE = 4

function applyProviderSnapshot(updated: ProviderWithEndpointsSummary): void {
  if (provider.value?.id === updated.id) {
    Object.assign(provider.value, updated)
    return
  }
  provider.value = updated
}

// A provider opened from scheduling may not belong to the resource page's loaded slice.
// Apply edits supplied by the parent even when the drawer loaded its own initial snapshot.
watch(() => props.initialProvider, (updated) => {
  if (!props.open || !updated || updated.id !== props.providerId) return
  providerLoadRequestId += 1
  applyProviderSnapshot(updated)
  loading.value = false
})

function getProviderKeysPageSize(providerType?: string | null): number {
  return (providerType || '').trim().toLowerCase() === 'custom'
    ? CUSTOM_PROVIDER_KEYS_PAGE_SIZE
    : DEFAULT_PROVIDER_KEYS_PAGE_SIZE
}

// 系统级格式转换配置
const systemFormatConversionEnabled = ref(false)

// 端点相关状态
const endpointDialogOpen = ref(false)

// 密钥相关状态
const keyFormDialogOpen = ref(false)
const keyBatchImportDialogOpen = ref(false)
const keyPermissionsDialogOpen = ref(false)
const oauthAccountDialogOpen = ref(false)
const oauthKeyEditDialogOpen = ref(false)
const currentEndpoint = ref<ProviderEndpoint | null>(null)
const editingKey = ref<EndpointAPIKey | null>(null)
const deleteKeyConfirmOpen = ref(false)
const keyToDelete = ref<EndpointAPIKey | null>(null)
const togglingKeyId = ref<string | null>(null)

// 密钥显示状态：key_id -> 完整密钥
const revealedKeys = ref<Map<string, string>>(new Map())

// 模型相关状态
const modelFormDialogOpen = ref(false)
const editingModel = ref<Model | null>(null)
const batchAssignDialogOpen = ref(false)
const modelMappingTabRef = ref<InstanceType<typeof ModelMappingTab> | null>(null)

// 密钥列表拖拽排序状态
const keyDragState = ref({
  isDragging: false,
  draggedIndex: null as number | null,
  targetIndex: null as number | null
})

// 点击编辑优先级相关状态
const editingPriorityKey = ref<string | null>(null)
const editingPriorityValue = ref<number>(0)
const priorityInputRef = ref<HTMLInputElement[] | null>(null)
const prioritySaving = ref(false)

// OAuth 刷新状态
const refreshingOAuthKeyId = ref<string | null>(null)

// OAuth 失效清除状态
const clearingOAuthInvalidKeyId = ref<string | null>(null)

// Codex reset credit 消费状态
const consumingCodexResetCreditKeyId = ref<string | null>(null)

// 限额刷新状态（Codex / Antigravity）
const refreshingQuota = ref(false)

// Antigravity 配额详情弹窗状态
const antigravityQuotaDialogOpen = ref(false)
const antigravityQuotaDialogKey = ref<EndpointAPIKey | null>(null)

// 故障转移规则
const failoverRulesDialogOpen = ref(false)
const FAILOVER_RULE_ARRAY_KEYS = [
  'success_failover_patterns',
  'error_stop_patterns',
  'stop_status_codes',
  'stop_on_status_codes',
  'early_stop_status_codes',
  'non_retryable_status_codes',
  'continue_on_status_codes',
  'retryable_status_codes',
  'retry_on_status_codes',
  'continue_status_codes',
] as const
const hasFailoverRules = computed(() => {
  const rules = provider.value?.failover_rules
  if (!rules) return false
  return FAILOVER_RULE_ARRAY_KEYS.some(key => (rules[key]?.length || 0) > 0)
    || typeof rules.max_retries === 'number'
    || rules.stop_on_transport_errors === true
})

// Provider 级别代理配置状态
const proxyNodesStore = useProxyNodesStore()
const providerProxyPopoverOpen = ref(false)
const savingProviderProxy = ref(false)

// Key 级别代理配置状态
const savingProxyKeyId = ref<string | null>(null)
const proxyPopoverOpenKeyId = ref<string | null>(null)

// 点击编辑倍率相关状态
const editingMultiplierKey = ref<string | null>(null)
const editingMultiplierFormat = ref<string | null>(null)
const editingMultiplierValue = ref<number>(1.0)
const multiplierInputRef = ref<HTMLInputElement[] | null>(null)
const multiplierSaving = ref(false)

// 任意模态窗口打开时,阻止抽屉被误关闭
const hasBlockingDialogOpen = computed(() =>
  endpointDialogOpen.value ||
  keyFormDialogOpen.value ||
  keyBatchImportDialogOpen.value ||
  keyPermissionsDialogOpen.value ||
  oauthAccountDialogOpen.value ||
  oauthKeyEditDialogOpen.value ||
  deleteKeyConfirmOpen.value ||
  modelFormDialogOpen.value ||
  batchAssignDialogOpen.value ||
  antigravityQuotaDialogOpen.value ||
  modelMappingTabRef.value?.dialogOpen
)

// 当前后端分页页内的密钥列表。key 通过 api_formats 字段确定支持的格式，endpoint 可能为 undefined。
const allKeys = computed(() => {
  return providerKeys.value.map(key => ({ key, endpoint: undefined as ProviderEndpointWithKeys | undefined }))
})

const availableKeyApiFormats = computed(() => {
  const formatSet = new Set<string>()

  for (const format of provider.value?.api_formats || []) {
    if (format) {
      formatSet.add(format)
    }
  }

  for (const endpoint of endpoints.value) {
    if (endpoint.api_format) {
      formatSet.add(endpoint.api_format)
    }
  }

  return sortApiFormats([...formatSet])
})

function syncCurrentSelections(
  nextEndpoints: ProviderEndpointWithKeys[] = endpoints.value,
  nextProviderKeys: EndpointAPIKey[] = providerKeys.value
) {
  if (currentEndpoint.value) {
    currentEndpoint.value = nextEndpoints.find(endpoint => endpoint.id === currentEndpoint.value?.id) ?? null
  }

  if (!editingKey.value) {
    return
  }

  const latestKeys: EndpointAPIKey[] = []
  const seenKeyIds = new Set<string>()

  for (const key of nextProviderKeys) {
    if (!seenKeyIds.has(key.id)) {
      seenKeyIds.add(key.id)
      latestKeys.push(key)
    }
  }

  for (const endpoint of nextEndpoints) {
    for (const key of endpoint.keys || []) {
      if (!seenKeyIds.has(key.id)) {
        seenKeyIds.add(key.id)
        latestKeys.push(key)
      }
    }
  }

  const latestEditingKey = latestKeys.find(key => key.id === editingKey.value?.id) || null
  editingKey.value = latestEditingKey

  if (!latestEditingKey) {
    keyFormDialogOpen.value = false
    keyPermissionsDialogOpen.value = false
    oauthKeyEditDialogOpen.value = false
  }
}

// ===== 账号列表后端分页 =====
const providerKeysTotal = ref(0)
const currentKeyPage = ref(1)
const keyPageSize = ref(DEFAULT_PROVIDER_KEYS_PAGE_SIZE)
const totalKeyPages = computed(() => Math.max(1, Math.ceil(providerKeysTotal.value / keyPageSize.value)))
const shouldPaginateKeys = computed(() => totalKeyPages.value > 1)
const paginatedKeys = computed(() => allKeys.value)

function getGlobalKeyIndex(localIdx: number): number {
  return localIdx
}

async function goToKeyPage(page: number) {
  const nextPage = Math.min(Math.max(page, 1), totalKeyPages.value)
  if (nextPage === currentKeyPage.value && providerKeys.value.length > 0) return
  await loadProviderKeysPage(nextPage)
}

// 合并监听 providerId 和 open，避免同一 tick 内两个 watcher 都触发导致重复请求
watch(
  [() => props.providerId, () => props.open],
  async ([newId, newOpen], [_oldId, oldOpen]) => {
    if (newOpen && newId) {
      if (!oldOpen || provider.value?.id !== newId) {
        currentKeyPage.value = 1
        providerKeysTotal.value = 0
      }
      const hasInitialProvider = props.initialProvider?.id === newId
      if (hasInitialProvider) {
        provider.value = props.initialProvider
        keyPageSize.value = getProviderKeysPageSize(provider.value?.provider_type)
        loading.value = false
      }
      void loadSystemFormatConversionConfig()
      if (!hasInitialProvider) {
        await loadProvider()
      }
      const endpointsPromise = loadEndpoints()
      // 仅在抽屉刚打开时启动倒计时
      if (newOpen && !oldOpen) {
        startCountdownTimer()
      }
      // 优先完成端点、密钥和模型的首屏数据，再请求计算量较大的映射预览。
      // 同时校验抽屉状态，避免关闭或切换 Provider 后启动无用请求。
      void endpointsPromise.then(() => {
        if (!props.open || props.providerId !== newId) return
        void loadMappingPreview()
        void autoRefreshQuotaInBackground()
      })
    } else if (!newOpen && oldOpen) {
      // 使在途请求失效，避免关闭后旧响应回写
      providerLoadRequestId += 1
      endpointsLoadRequestId += 1
      keysLoadRequestId += 1
      mappingPreviewLoadRequestId += 1

      // 停止倒计时定时器
      stopCountdownTimer()
      // 重置所有状态
      loading.value = false
      provider.value = null
      endpoints.value = []
      providerKeys.value = []  // 清空 Provider 级别的 keys
      providerKeysTotal.value = 0
      currentKeyPage.value = 1
      keyPageSize.value = DEFAULT_PROVIDER_KEYS_PAGE_SIZE
      providerModels.value = []
      providerMappingPreview.value = null
      loadingProviderEndpoints.value = false
      loadingProviderKeys.value = false
      loadingProviderModels.value = false
      loadingProviderMappingPreview.value = false

      // 重置所有对话框状态
      endpointDialogOpen.value = false
      keyFormDialogOpen.value = false
      keyBatchImportDialogOpen.value = false
      keyPermissionsDialogOpen.value = false
      oauthAccountDialogOpen.value = false
      oauthKeyEditDialogOpen.value = false
      deleteKeyConfirmOpen.value = false
      batchAssignDialogOpen.value = false
      antigravityQuotaDialogOpen.value = false
      antigravityQuotaDialogKey.value = null

      // 重置临时数据
      currentEndpoint.value = null
      editingKey.value = null
      keyToDelete.value = null

      // 清除已显示的密钥（安全考虑）
      revealedKeys.value.clear()
    }
  },
  { immediate: true },
)

// 关闭抽屉
function handleClose() {
  if (!hasBlockingDialogOpen.value) {
    emit('update:open', false)
  }
}

// 切换格式转换开关
async function toggleFormatConversion() {
  if (!provider.value) return
  const newValue = !provider.value.enable_format_conversion
  try {
    const updated = await updateProvider(provider.value.id, { enable_format_conversion: newValue })
    applyProviderSnapshot(updated)
    showSuccess(newValue ? t('ui.formatConversionEnabled') : t('ui.formatConversionDisabled2c1997b'))
    emit('refresh')
  } catch {
    showError(t('ui.failedToToggleFormatConversion'))
  }
}

async function toggleKeepPriorityOnConversion() {
  if (!provider.value) return
  const formatConversionAvailable =
    provider.value.enable_format_conversion || systemFormatConversionEnabled.value
  if (!formatConversionAvailable) return
  const newValue = !provider.value.keep_priority_on_conversion
  try {
    const updated = await updateProvider(provider.value.id, {
      keep_priority_on_conversion: newValue,
    })
    applyProviderSnapshot(updated)
    showSuccess(newValue ? t('ui.priorityPreservationDuringFormatConversionEnabled') : t('ui.priorityPreservationDuringFormatConversionDisabled'))
    emit('refresh')
  } catch {
    showError(t('ui.failedToChangeFormatConversionPriorityPreservation'))
  }
}

function getProviderProxyNodeName(): string {
  const nodeId = provider.value?.proxy?.node_id
  if (!nodeId) return t('ui.unknownNode')
  const node = proxyNodesStore.nodes.find(n => n.id === nodeId)
  return node ? node.name : `${nodeId.slice(0, 8)}...`
}

async function setProviderProxy(nodeId: string) {
  if (!provider.value) return
  savingProviderProxy.value = true
  try {
    const updated = await updateProvider(provider.value.id, {
      proxy: { node_id: nodeId, enabled: true },
    })
    applyProviderSnapshot(updated)
    providerProxyPopoverOpen.value = false
    showSuccess(t('ui.proxyNodeSet'))
    emit('refresh')
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.failedToSetProxy')))
  } finally {
    savingProviderProxy.value = false
  }
}

async function clearProviderProxy() {
  if (!provider.value) return
  savingProviderProxy.value = true
  try {
    const updated = await updateProvider(provider.value.id, { proxy: null })
    applyProviderSnapshot(updated)
    providerProxyPopoverOpen.value = false
    showSuccess(t('ui.providerProxyCleared'))
    emit('refresh')
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.failedToClearProxy')))
  } finally {
    savingProviderProxy.value = false
  }
}

// 显示端点管理对话框
function showAddEndpointDialog() {
  endpointDialogOpen.value = true
}

// ===== 端点事件处理 =====
function handleEditEndpoint(_endpoint: ProviderEndpoint) {
  // 点击任何端点都打开管理对话框
  endpointDialogOpen.value = true
}

async function handleEndpointChanged() {
  await Promise.all([loadProvider(), loadEndpoints(), loadMappingPreview()])
  emit('refresh')
}

// ===== 密钥事件处理 =====
function handleAddKey(endpoint: ProviderEndpoint) {
  currentEndpoint.value = endpoint
  editingKey.value = null
  keyFormDialogOpen.value = true
}

// 添加密钥/账号（如果有多个端点则添加到第一个）
function handleAddKeyToFirstEndpoint() {
  if (endpoints.value.length === 0) return

  // OAuth 账号型提供商：打开 OAuth 账号对话框
  if (isOAuthAccountProviderType(provider.value?.provider_type)) {
    oauthAccountDialogOpen.value = true
  } else {
    // 密钥型提供商（custom/vertex_ai）：打开密钥表单对话框
    handleAddKey(endpoints.value[0])
  }
}

function handleEditKey(endpoint: ProviderEndpoint | undefined, key: EndpointAPIKey) {
  currentEndpoint.value = endpoint || null
  editingKey.value = key
  // OAuth 密钥使用专门的编辑对话框
  if (canEditOAuthCredential(key)) {
    oauthKeyEditDialogOpen.value = true
  } else {
    keyFormDialogOpen.value = true
  }
}

function handleKeyPermissions(key: EndpointAPIKey) {
  editingKey.value = key
  keyPermissionsDialogOpen.value = true
}

// 复制完整密钥或认证配置
async function copyFullKey(key: EndpointAPIKey) {
  const cached = revealedKeys.value.get(key.id)
  if (cached) {
    copyToClipboard(cached)
    return
  }

  // 否则先获取再复制
  try {
    const result = await revealEndpointKey(key.id)
    let textToCopy: string

    if (result.auth_type === 'service_account' && result.auth_config) {
      // Service Account 类型：复制 auth_config JSON
      textToCopy = typeof result.auth_config === 'string'
        ? result.auth_config
        : JSON.stringify(result.auth_config, null, 2)
    } else {
      // API Key 类型：复制 api_key
      textToCopy = result.api_key || ''
    }

    revealedKeys.value.set(key.id, textToCopy)
    copyToClipboard(textToCopy)
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.failedToRetrieveKey')), t('ui.error'))
  }
}

// 下载 OAuth 凭据文件（后端统一导出，前端只负责下载）
async function downloadRefreshToken(key: EndpointAPIKey) {
  try {
    const data = await exportKey(key.id)
    const providerType = provider.value?.provider_type || 'unknown'
    const safeName = ((typeof data.email === 'string' && data.email) || key.name || key.id.slice(0, 8)).replace(/[^a-zA-Z0-9_\-@.]/g, '_')

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `aether_${providerType}_${safeName}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.exportFailed')), t('ui.error'))
  }
}

function handleDeleteKey(key: EndpointAPIKey) {
  keyToDelete.value = key
  deleteKeyConfirmOpen.value = true
}

function formatDeleteKeyConfirmDescription(): string {
  const keyName = keyToDelete.value?.api_key_masked || keyToDelete.value?.name || ''
  return t('providers.migration.deleteKey56506e', { value: String(keyName) })
}

async function confirmDeleteKey() {
  if (!keyToDelete.value) return

  const keyId = keyToDelete.value.id
  deleteKeyConfirmOpen.value = false
  keyToDelete.value = null

  try {
    await deleteEndpointKey(keyId)
    showSuccess(t('ui.keyDeleted'))
    // 刷新端点列表及模型数据（删除 Key 触发自动解除模型关联）
    await Promise.all([loadProvider(), loadEndpoints(), loadMappingPreview()])
    emit('refresh')
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.failedToDeleteKey')), t('ui.error'))
  }
}

async function handleRecoverKey(key: EndpointAPIKey) {
  try {
    const result = await recoverKeyHealth(key.id)
    showSuccess((result.message || t('ui.keyFullyRecovered')))
    await Promise.all([loadProvider(), loadEndpoints()])
    emit('refresh')
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.failedToRestoreKey')), t('ui.error'))
  }
}

async function handleRefreshOAuth(key: EndpointAPIKey) {
  if (refreshingOAuthKeyId.value) return
  refreshingOAuthKeyId.value = key.id
  try {
    const result = await refreshProviderOAuth(key.id)
    const refreshedExpiresAt = typeof result.expires_at === 'number' ? result.expires_at : null
    let refreshedKey: EndpointAPIKey | null = null
    // 更新本地数据
    const keyInList = providerKeys.value.find(k => k.id === key.id)
    if (keyInList) {
      keyInList.oauth_expires_at = refreshedExpiresAt
    }
    // 只重新加载当前 keys 页，避免整个表格刷新
    if (props.providerId) {
      const freshPage = await getProviderKeysPage(props.providerId, {
        page: currentKeyPage.value,
        page_size: keyPageSize.value,
      }).catch(() => null)
      if (freshPage) {
        const mergedKeys = freshPage.keys.map((item) => {
          if (item.id !== key.id) return item
          if (refreshedExpiresAt == null) return item
          if (typeof item.oauth_expires_at === 'number' && item.oauth_expires_at >= refreshedExpiresAt) {
            return item
          }
          return { ...item, oauth_expires_at: refreshedExpiresAt }
        })
        providerKeys.value = mergedKeys
        providerKeysTotal.value = freshPage.total
        currentKeyPage.value = freshPage.page
        keyPageSize.value = freshPage.page_size
        syncCurrentSelections(endpoints.value, mergedKeys)
        refreshedKey = mergedKeys.find(item => item.id === key.id) ?? null
      }
    }
    const feedback = getOAuthRefreshFeedback({
      accountStateRecheckAttempted: result.account_state_recheck_attempted,
      accountStateRecheckError: result.account_state_recheck_error,
      snapshot: refreshedKey,
    })
    if (feedback.tone === 'warning') {
      showWarning(feedback.message)
    } else {
      showSuccess(feedback.message)
    }
    emit('refresh')
    // Token 刷新可能激活账号并更新配额，完成后再同步一次父列表。
    void autoRefreshQuotaInBackground().then((changed) => {
      if (changed) emit('refresh')
    })
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.failedToRefreshToken')), t('ui.error'))
    await Promise.all([loadProvider(), loadEndpoints()])
    emit('refresh')
  } finally {
    refreshingOAuthKeyId.value = null
  }
}

// 判断是否为账号级别的封禁（刷新 token 无法修复）
function isAccountLevelBlock(key: EndpointAPIKey): boolean {
  const account = getAccountStatusDisplay(key)
  const oauth = getOAuthStatusDisplay(key, countdownTick.value)
  return account.blocked && !oauth?.isInvalid
}

function formatClearOAuthInvalidConfirmMessage(key: EndpointAPIKey): string {
  const keyName = key.name || key.id.slice(0, 8)
  return t('providers.migration.confirmThatAccountHasBeend65cff', { value: String(keyName) })
}

// 清除 OAuth 失效标记
async function handleClearOAuthInvalid(key: EndpointAPIKey) {
  if (clearingOAuthInvalidKeyId.value) return

  const confirmed = await confirm({
    title: t('ui.clearAccountIssueMarker'),
    message: formatClearOAuthInvalidConfirmMessage(key),
    confirmText: t('ui.confirmClear'),
    variant: 'info',
  })
  if (!confirmed) return

  clearingOAuthInvalidKeyId.value = key.id
  try {
    await clearOAuthInvalid(key.id)
    showSuccess(t('ui.oauthIssueMarkerCleared'))
    // 更新本地数据
    const keyInList = providerKeys.value.find(k => k.id === key.id)
    if (keyInList) {
      keyInList.oauth_invalid_at = null
      keyInList.oauth_invalid_reason = null
      if (keyInList.status_snapshot) {
        keyInList.status_snapshot = {
          ...keyInList.status_snapshot,
          oauth: {
            ...keyInList.status_snapshot.oauth,
            code: 'none',
            label: null,
            reason: null,
            invalid_at: null,
            requires_reauth: false,
          },
          account: {
            ...keyInList.status_snapshot.account,
            code: 'ok',
            label: null,
            reason: null,
            blocked: false,
            recoverable: false,
          },
        }
      }
    }
    await Promise.all([loadProvider(), loadEndpoints()])
    emit('refresh')
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.failedToClear')), t('ui.error'))
    await Promise.all([loadProvider(), loadEndpoints()])
    emit('refresh')
  } finally {
    clearingOAuthInvalidKeyId.value = null
  }
}

function codexResetCreditOutcomeFeedback(
  result: Awaited<ReturnType<typeof consumeCodexResetCredit>>,
): { tone: 'success' | 'warning'; message: string } {
  switch (result.outcome) {
    case 'reset':
      return { tone: 'success', message: t('ui.codexQuotaResetUsedAccountQuotaRefreshedb0ca5d5') }
    case 'already_redeemed':
      return { tone: 'success', message: t('ui.resetRequestProcessedAccountQuotaRefreshed') }
    case 'nothing_to_reset':
      return { tone: 'warning', message: t('ui.noCodexQuotaWindowsNeedAReset') }
    case 'no_credit':
      return { tone: 'warning', message: t('ui.noCodexQuotaResetsAvailable') }
    default:
      return { tone: 'warning', message: t('ui.theResetReturnedAnUnknownResultCheckTheLatestAccount') }
  }
}

function codexResetCreditActiveIdempotencyKeyFromError(error: unknown): string | null {
  if (typeof error !== 'object' || error === null || !('response' in error)) return null
  const response = (error as { response?: { data?: unknown } }).response
  if (typeof response?.data !== 'object' || response.data === null) return null
  const activeKey = (response.data as Record<string, unknown>).active_idempotency_key
  return typeof activeKey === 'string' && activeKey.trim() ? activeKey.trim() : null
}

function codexResetCreditCredentialChangedFromError(error: unknown): boolean {
  if (typeof error !== 'object' || error === null || !('response' in error)) return false
  const response = (error as { response?: { data?: unknown } }).response
  if (typeof response?.data !== 'object' || response.data === null) return false
  return (response.data as Record<string, unknown>).outcome === 'credential_changed'
}

async function handleConsumeCodexResetCredit(key: EndpointAPIKey) {
  if (!canConsumeCodexResetCredit(key)) return

  const credentialGeneration = getCodexCredentialGeneration(key)
  if (credentialGeneration === undefined) return
  const pendingIdempotencyKey = getPendingCodexResetCreditIdempotencyKey(key)
  const earliest = getVisibleCodexResetCreditItems(key)[0]
  const detailMessage = earliest
    ? t('providers.migration.nextToExpireExpiresAtf3a4e3', { value: String(earliest.displayKey), value_2: String(formatCodexResetCreditExpiresAt(earliest.expiresAt)) })
    : ''
  const confirmed = await confirm({
    title: t('ui.confirmCodexQuotaReset'),
    message: pendingIdempotencyKey
      ? t('ui.thePendingCodexResetRequestWillBeCheckedAgain')
      : `${t('ui.usesOneCodexQuotaResetAccountQuotaWillRefreshWhen')}${detailMessage}`,
    confirmText: t('ui.confirmReset'),
    cancelText: t('common.cancel'),
    variant: 'warning',
  })
  if (!confirmed) return

  consumingCodexResetCreditKeyId.value = key.id
  try {
    const idempotencyKey = pendingIdempotencyKey
      || readPendingCodexResetCreditIdempotencyKey(key.id, credentialGeneration)
      || createCodexResetCreditIdempotencyKey()
    rememberPendingCodexResetCreditIdempotencyKey(key.id, idempotencyKey, credentialGeneration)
    const result = await consumeCodexResetCredit(key.id, {
      idempotency_key: idempotencyKey,
      expected_credential_generation: credentialGeneration,
    })
    clearPendingCodexResetCreditIdempotencyKeyForOutcome(key.id, result.outcome)
    applyQuotaResults([{
      key_id: result.key_id,
      status: result.refresh_status === 'success' ? 'success' : result.status,
      metadata: result.metadata,
      quota_snapshot: result.quota_snapshot,
    }])

    const feedback = codexResetCreditOutcomeFeedback(result)
    if (feedback.tone === 'success') {
      showSuccess(feedback.message)
    } else {
      showWarning(feedback.message)
    }
    if (result.refresh_status === 'failed') {
      showWarning((result.refresh_error || t('ui.resetRequestProcessedButTheLatestQuotaCouldNotBe')))
    }
    emit('refresh')
  } catch (err: unknown) {
    const activeIdempotencyKey = codexResetCreditActiveIdempotencyKeyFromError(err)
    if (codexResetCreditCredentialChangedFromError(err)) {
      clearPendingCodexResetCreditIdempotencyKey(key.id)
    } else if (activeIdempotencyKey) {
      rememberPendingCodexResetCreditIdempotencyKey(
        key.id,
        activeIdempotencyKey,
        credentialGeneration,
      )
    }
    showError(localizedApiError(err, t('ui.failedToUseCodexQuotaReset')), t('ui.error'))
    await Promise.all([loadProvider(), loadEndpoints()])
    emit('refresh')
  } finally {
    consumingCodexResetCreditKeyId.value = null
  }
}

// Codex / Gemini CLI / Antigravity / Kiro / Windsurf / ChatGPT Web：打开抽屉后自动后台刷新（配额缓存缺失/过期，或 Token 即将过期时触发）
const AUTO_QUOTA_REFRESH_STALE_SECONDS = 5 * 60
// 与后端 OAuth 懒刷新阈值对齐：到期前 2 分钟内视为需要刷新
const AUTO_TOKEN_REFRESH_SKEW_SECONDS = 2 * 60

function canConsumeCodexResetCredit(key: EndpointAPIKey): boolean {
  return provider.value?.provider_type === 'codex'
    && getCodexCredentialGeneration(key) !== undefined
    && (hasPendingCodexResetCredit(key) || (getCodexResetCreditAvailableCount(key) ?? 0) > 0)
    && !consumingCodexResetCreditKeyId.value
}

function shouldShowKiroSubscriptionBadge(key: EndpointAPIKey): boolean {
  if (provider.value?.provider_type !== 'kiro') return false

  const kiroLabel = getKiroSubscriptionBadgeLabel(key)
  if (!kiroLabel) return false

  const oauthPlanLabel = formatOAuthPlanType(key.oauth_plan_type ?? '')
  if (!oauthPlanLabel) return true

  return oauthPlanLabel.trim().toLowerCase() !== kiroLabel.trim().toLowerCase()
}

function shouldAutoRefreshCodexQuota(): boolean {
  if (provider.value?.provider_type !== 'codex') return false
  const now = Math.floor(Date.now() / 1000)

  for (const { key } of allKeys.value) {
    if (!key.is_active) continue

    if (isTokenExpiringSoon(key, now)) return true

    // reset-credit 独立于 Token 刷新；这里只按账号配额缓存决定是否后台更新
    if (!hasCodexQuotaDisplayData(key)) {
      return true
    }
    // 配额数据超过 5 分钟未更新，也触发刷新
    const updatedAt = getCodexQuotaDisplay(key)?.updated_at
    if (typeof updatedAt !== 'number' || (now - updatedAt) > AUTO_QUOTA_REFRESH_STALE_SECONDS) {
      return true
    }
  }

  return false
}

// 检查 OAuth Token 是否即将过期（Codex / Gemini CLI / Antigravity / Kiro / Windsurf / ChatGPT Web）
function isTokenExpiringSoon(key: EndpointAPIKey, now: number): boolean {
  const oauthCode = String(key.status_snapshot?.oauth?.code || '').trim().toLowerCase()
  if (oauthCode && oauthCode !== 'valid' && oauthCode !== 'expiring') {
    return false
  }
  return typeof key.oauth_expires_at === 'number'
    && (key.oauth_expires_at - now) <= AUTO_TOKEN_REFRESH_SKEW_SECONDS
}

function shouldAutoRefreshAntigravityQuota(): boolean {
  if (provider.value?.provider_type !== 'antigravity') return false
  const now = Math.floor(Date.now() / 1000)

  for (const { key } of allKeys.value) {
    if (!key.is_active) continue

    if (isTokenExpiringSoon(key, now)) return true

    // 只要有一个活跃 key 没有配额/为空/过期，就刷新一次（接口会批量刷新所有活跃 key）
    if (!hasAntigravityQuotaDisplayData(key)) {
      return true
    }
    const updatedAt = getAntigravityQuotaUpdatedAt(key)
    if (typeof updatedAt !== 'number' || (now - updatedAt) > AUTO_QUOTA_REFRESH_STALE_SECONDS) {
      return true
    }
  }

  return false
}

function shouldAutoRefreshGeminiCliQuota(): boolean {
  if (provider.value?.provider_type !== 'gemini_cli') return false
  const now = Math.floor(Date.now() / 1000)

  for (const { key } of allKeys.value) {
    if (!key.is_active) continue

    if (isTokenExpiringSoon(key, now)) return true

    if (!hasGeminiCliQuotaDisplayData(key)) {
      return true
    }

    const updatedAt = getGeminiCliQuotaUpdatedAt(key)
    if (typeof updatedAt !== 'number' || (now - updatedAt) > AUTO_QUOTA_REFRESH_STALE_SECONDS) {
      return true
    }
  }

  return false
}

function shouldAutoRefreshClaudeCodeQuota(): boolean {
  if (provider.value?.provider_type !== 'claude_code') return false
  const now = Math.floor(Date.now() / 1000)

  for (const { key } of allKeys.value) {
    if (!key.is_active) continue
    if (isTokenExpiringSoon(key, now)) return true
    if (!hasClaudeCodeQuotaDisplayData(key)) return true

    const updatedAt = getClaudeCodeQuotaUpdatedAt(key)
    if (typeof updatedAt !== 'number' || (now - updatedAt) > AUTO_QUOTA_REFRESH_STALE_SECONDS) {
      return true
    }
  }

  return false
}

function shouldAutoRefreshKiroQuota(): boolean {
  if (provider.value?.provider_type !== 'kiro') return false
  const now = Math.floor(Date.now() / 1000)

  for (const { key } of allKeys.value) {
    if (!key.is_active) continue

    if (isTokenExpiringSoon(key, now)) return true

    // 只要有一个活跃 key 没有配额数据，就刷新一次
    if (!hasKiroQuotaDisplayData(key)) {
      return true
    }
    // 配额数据超过 5 分钟未更新，也触发刷新
    const updatedAt = getKiroQuotaDisplay(key)?.updated_at
    if (typeof updatedAt !== 'number' || (now - updatedAt) > AUTO_QUOTA_REFRESH_STALE_SECONDS) {
      return true
    }
  }

  return false
}

function shouldAutoRefreshGrokQuota(): boolean {
  if (provider.value?.provider_type !== 'grok') return false
  const now = Math.floor(Date.now() / 1000)

  for (const { key } of allKeys.value) {
    if (!key.is_active) continue

    if (isTokenExpiringSoon(key, now)) return true

    if (!hasGrokQuotaDisplayData(key)) {
      return true
    }

    const updatedAt = getGrokQuotaDisplay(key)?.updated_at
    if (typeof updatedAt !== 'number' || (now - updatedAt) > AUTO_QUOTA_REFRESH_STALE_SECONDS) {
      return true
    }
  }

  return false
}

function shouldAutoRefreshXaiQuota(): boolean {
  if (provider.value?.provider_type !== 'xai') return false
  const now = Math.floor(Date.now() / 1000)

  for (const { key } of allKeys.value) {
    if (!key.is_active) continue

    if (isTokenExpiringSoon(key, now)) return true

    if (!hasXaiQuotaDisplayData(key)) {
      return true
    }

    const updatedAt = getXaiQuotaDisplay(key)?.updated_at
    if (typeof updatedAt !== 'number' || (now - updatedAt) > AUTO_QUOTA_REFRESH_STALE_SECONDS) {
      return true
    }
  }

  return false
}

function shouldAutoRefreshWindsurfQuota(): boolean {
  if (provider.value?.provider_type !== 'windsurf') return false
  const now = Math.floor(Date.now() / 1000)

  for (const { key } of allKeys.value) {
    if (!key.is_active) continue

    if (isTokenExpiringSoon(key, now)) return true

    if (!hasWindsurfQuotaDisplayData(key)) {
      return true
    }

    const updatedAt = getWindsurfQuotaDisplay(key)?.updated_at
    if (typeof updatedAt !== 'number' || (now - updatedAt) > AUTO_QUOTA_REFRESH_STALE_SECONDS) {
      return true
    }
  }

  return false
}

function shouldAutoRefreshChatGPTWebQuota(): boolean {
  if (provider.value?.provider_type !== 'chatgpt_web') return false
  const now = Math.floor(Date.now() / 1000)

  for (const { key } of allKeys.value) {
    if (!key.is_active) continue

    if (isTokenExpiringSoon(key, now)) return true

    if (!hasChatGPTWebQuotaDisplayData(key)) {
      return true
    }

    const updatedAt = getChatGPTWebQuotaDisplay(key)?.updated_at
    if (typeof updatedAt !== 'number' || (now - updatedAt) > AUTO_QUOTA_REFRESH_STALE_SECONDS) {
      return true
    }
  }

  return false
}

function defaultQuotaSnapshot(): QuotaStatusSnapshot {
  return {
    code: 'unknown',
    exhausted: false,
    usage_ratio: null,
    updated_at: null,
    reset_seconds: null,
    plan_type: null,
  }
}

function wrapQuotaMetadataForProvider(
  providerType: string,
  metadata: Record<string, unknown> | undefined,
): UpstreamMetadata | null {
  if (!metadata) return null
  if (providerType in metadata) {
    return metadata as UpstreamMetadata
  }
  return { [providerType]: metadata } as UpstreamMetadata
}

// 将配额刷新结果就地应用到现有 key 上，避免重新拉列表导致分页重置
function applyQuotaResults(
  results: { key_id: string; status: string; metadata?: Record<string, unknown>; quota_snapshot?: QuotaStatusSnapshot }[],
): number {
  const providerType = provider.value?.provider_type
  if (!providerType) return 0

  let applied = 0
  for (const r of results) {
    const target = providerKeys.value.find(k => k.id === r.key_id)
    if (!target) continue

    let changed = false
    const wrappedMetadata = wrapQuotaMetadataForProvider(providerType, r.metadata)
    if (wrappedMetadata) {
      target.upstream_metadata = { ...target.upstream_metadata, ...wrappedMetadata } as typeof target.upstream_metadata
      changed = true
    }

    if (r.quota_snapshot) {
      target.status_snapshot = {
        oauth: target.status_snapshot?.oauth ?? {
          code: 'none',
          label: null,
          reason: null,
          expires_at: null,
          invalid_at: null,
          source: null,
          requires_reauth: false,
          expiring_soon: false,
        },
        account: target.status_snapshot?.account ?? {
          code: 'ok',
          label: null,
          reason: null,
          blocked: false,
          source: null,
          recoverable: false,
        },
        quota: {
          ...defaultQuotaSnapshot(),
          ...(target.status_snapshot?.quota ?? {}),
          ...r.quota_snapshot,
        },
      }
      changed = true
    }

    if (changed) {
      applied += 1
    }
  }
  return applied
}

// 通用的自动刷新配额函数（支持 Codex、Gemini CLI、Antigravity、Kiro、Windsurf 和 ChatGPT Web）
async function autoRefreshQuotaInBackground(): Promise<boolean> {
  const providerId = props.providerId
  if (!providerId) return false
  if (refreshingQuota.value) return false

  const providerType = provider.value?.provider_type
  if (providerType !== 'codex' && providerType !== 'claude_code' && providerType !== 'gemini_cli' && providerType !== 'antigravity' && providerType !== 'kiro' && providerType !== 'windsurf' && providerType !== 'chatgpt_web' && providerType !== 'grok' && providerType !== 'xai') return false

  // 检查是否需要刷新
  let shouldRefresh = false
  if (providerType === 'codex') {
    shouldRefresh = shouldAutoRefreshCodexQuota()
  } else if (providerType === 'claude_code') {
    shouldRefresh = shouldAutoRefreshClaudeCodeQuota()
  } else if (providerType === 'gemini_cli') {
    shouldRefresh = shouldAutoRefreshGeminiCliQuota()
  } else if (providerType === 'antigravity') {
    shouldRefresh = shouldAutoRefreshAntigravityQuota()
  } else if (providerType === 'kiro') {
    shouldRefresh = shouldAutoRefreshKiroQuota()
  } else if (providerType === 'grok') {
    shouldRefresh = shouldAutoRefreshGrokQuota()
  } else if (providerType === 'xai') {
    shouldRefresh = shouldAutoRefreshXaiQuota()
  } else if (providerType === 'windsurf') {
    shouldRefresh = shouldAutoRefreshWindsurfQuota()
  } else if (providerType === 'chatgpt_web') {
    shouldRefresh = shouldAutoRefreshChatGPTWebQuota()
  }
  if (!shouldRefresh) return false

  let hadCachedQuota = false
  if (providerType === 'codex') {
    hadCachedQuota = allKeys.value.some(({ key }) => key.is_active && hasCodexQuotaDisplayData(key))
  } else if (providerType === 'claude_code') {
    hadCachedQuota = allKeys.value.some(({ key }) => key.is_active && hasClaudeCodeQuotaDisplayData(key))
  } else if (providerType === 'gemini_cli') {
    hadCachedQuota = allKeys.value.some(({ key }) => key.is_active && hasGeminiCliQuotaDisplayData(key))
  } else if (providerType === 'antigravity') {
    hadCachedQuota = allKeys.value.some(({ key }) => key.is_active && hasAntigravityQuotaDisplayData(key))
  } else if (providerType === 'kiro') {
    hadCachedQuota = allKeys.value.some(({ key }) => key.is_active && hasKiroQuotaDisplayData(key))
  } else if (providerType === 'grok') {
    hadCachedQuota = allKeys.value.some(({ key }) => key.is_active && hasGrokQuotaDisplayData(key))
  } else if (providerType === 'xai') {
    hadCachedQuota = allKeys.value.some(({ key }) => key.is_active && hasXaiQuotaDisplayData(key))
  } else if (providerType === 'windsurf') {
    hadCachedQuota = allKeys.value.some(({ key }) => key.is_active && hasWindsurfQuotaDisplayData(key))
  } else if (providerType === 'chatgpt_web') {
    hadCachedQuota = allKeys.value.some(({ key }) => key.is_active && hasChatGPTWebQuotaDisplayData(key))
  }

  refreshingQuota.value = true
  const isCurrent = () => props.open && props.providerId === providerId
  try {
    const result = await refreshQuotaInBackground({
      refresh: () => refreshProviderQuota(providerId),
      isCurrent,
      retryInitialEmptyQuota: providerType === 'antigravity' && !hadCachedQuota,
    })
    if (!result) return false
    const applied = applyQuotaResults(result.results)
    if (result.success <= 0 && applied === 0 && !hadCachedQuota && providerType === 'antigravity') {
      showWarning(t('ui.quotaInformationIsNotReadyYetPleaseRefreshAgainShortly'), t('ui.notice'))
    }
    return applied > 0
  } catch (err: unknown) {
    if (isCurrent() && !hadCachedQuota && providerType === 'antigravity') {
      showError(localizedApiError(err, t('ui.failedToRefreshQuotaInTheBackground')), t('ui.error'))
    }
    return false
  } finally {
    refreshingQuota.value = false
  }
}

async function openAntigravityQuotaDialog(key: EndpointAPIKey) {
  const providerId = props.providerId
  if (!providerId) return
  antigravityQuotaDialogKey.value = key
  antigravityQuotaDialogOpen.value = true

  // 没有配额数据时主动获取
  if (!hasAntigravityQuotaDisplayData(key)) {
    if (refreshingQuota.value) return
    refreshingQuota.value = true
    try {
      const result = await refreshProviderQuota(providerId)
      if (providerId !== props.providerId) return
      applyQuotaResults(result.results)
      // 更新弹窗引用的 key 数据
      const updated = allKeys.value.find(({ key: k }) => k.id === key.id)
      if (updated) {
        antigravityQuotaDialogKey.value = updated.key
      }
    } catch {
      // 静默失败，弹窗会显示"暂无配额数据"
    } finally {
      refreshingQuota.value = false
    }
  }
}

function applyUpdatedKeySnapshot(updatedKey: EndpointAPIKey) {
  const index = providerKeys.value.findIndex(key => key.id === updatedKey.id)
  if (index >= 0) {
    providerKeys.value.splice(index, 1, updatedKey)
  }
  if (editingKey.value?.id === updatedKey.id) {
    editingKey.value = updatedKey
  }
  syncCurrentSelections(endpoints.value, providerKeys.value)
}

async function handleKeyChanged(updatedKey?: EndpointAPIKey) {
  if (updatedKey) applyUpdatedKeySnapshot(updatedKey)
  await Promise.all([loadProvider(), loadEndpoints(), loadMappingPreview()])
  if (updatedKey) applyUpdatedKeySnapshot(updatedKey)
  emit('refresh')
  // 添加/修改 key 后自动获取已支持 provider 的配额（新 key 的 upstream_metadata 为空）
  void autoRefreshQuotaInBackground().then((changed) => {
    if (changed) emit('refresh')
  })
}

// 切换密钥启用状态
async function toggleKeyActive(key: EndpointAPIKey) {
  if (togglingKeyId.value) return

  togglingKeyId.value = key.id
  try {
    const newStatus = !key.is_active
    const updated = await updateProviderKey(key.id, { is_active: newStatus })
    Object.assign(key, updated)
    key.is_active = newStatus
    await Promise.all([loadProvider(), loadEndpoints()])
    showSuccess(newStatus ? t('ui.keyEnabled') : t('ui.keyDisabled0261100'))
    emit('refresh')
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.operationFailed')), t('ui.error'))
  } finally {
    togglingKeyId.value = null
  }
}

// ===== Key 级别代理配置 =====

/** 获取 Key 当前代理节点的名称（用于显示） */
function getKeyProxyNodeName(key: EndpointAPIKey): string | null {
  if (!key.proxy?.node_id) return null
  const node = proxyNodesStore.nodes.find(n => n.id === key.proxy?.node_id)
  return node ? node.name : `${key.proxy.node_id.slice(0, 8)  }...`
}

/** 切换代理 Popover 的打开/关闭状态 */
function handleProxyPopoverToggle(keyId: string, open: boolean) {
  proxyPopoverOpenKeyId.value = open ? keyId : null
  if (open) {
    proxyNodesStore.ensureLoaded()
  }
}

/** 设置 Key 的代理节点 */
async function setKeyProxy(key: EndpointAPIKey, nodeId: string) {
  savingProxyKeyId.value = key.id
  try {
    await updateProviderKey(key.id, {
      proxy: { node_id: nodeId, enabled: true },
    })
    key.proxy = { node_id: nodeId, enabled: true }
    proxyPopoverOpenKeyId.value = null
    showSuccess(t('ui.proxyNodeSet'))
    emit('refresh')
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.failedToSetProxy')), t('ui.error'))
  } finally {
    savingProxyKeyId.value = null
  }
}

/** 清除 Key 的代理节点（回退到 Provider 级别代理） */
async function clearKeyProxy(key: EndpointAPIKey) {
  savingProxyKeyId.value = key.id
  try {
    await updateProviderKey(key.id, { proxy: null })
    key.proxy = null
    proxyPopoverOpenKeyId.value = null
    showSuccess(t('ui.accountProxyClearedProviderLevelProxyWillBeUsed'))
    emit('refresh')
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.failedToClearProxy')), t('ui.error'))
  } finally {
    savingProxyKeyId.value = null
  }
}

// ===== 模型事件处理 =====
// 处理编辑模型
function handleEditModel(model: Model) {
  editingModel.value = model
  modelFormDialogOpen.value = true
}

// 处理打开批量关联对话框
function handleBatchAssign() {
  batchAssignDialogOpen.value = true
}

function handleBatchAssignDialogOpenUpdate(value: boolean) {
  batchAssignDialogOpen.value = value
}

// 处理批量关联完成
async function handleBatchAssignChanged() {
  await Promise.all([loadProvider(), loadEndpoints(), loadMappingPreview()])
  emit('refresh')
}

// 处理模型映射变更
async function handleModelMappingChanged() {
  await Promise.all([loadProvider(), loadEndpoints(), loadMappingPreview()])
  emit('refresh')
}

// 处理模型保存完成
async function handleModelSaved() {
  editingModel.value = null
  await Promise.all([loadProvider(), loadEndpoints(), loadMappingPreview()])
  emit('refresh')
}

// ===== 点击编辑优先级 =====
function startEditPriority(key: EndpointAPIKey) {
  editingPriorityKey.value = key.id
  editingPriorityValue.value = key.internal_priority ?? 0
  prioritySaving.value = false
  nextTick(() => {
    // v-for 中的 ref 是数组，取第一个元素
    const input = Array.isArray(priorityInputRef.value) ? priorityInputRef.value[0] : priorityInputRef.value
    input?.focus()
    input?.select()
  })
}

function cancelEditPriority() {
  editingPriorityKey.value = null
  prioritySaving.value = false
}

function handlePriorityKeydown(e: KeyboardEvent, key: EndpointAPIKey) {
  if (e.key === 'Enter') {
    e.preventDefault()
    e.stopPropagation()
    if (!prioritySaving.value) {
      prioritySaving.value = true
      savePriority(key)
    }
  } else if (e.key === 'Escape') {
    e.preventDefault()
    cancelEditPriority()
  }
}

function handlePriorityBlur(key: EndpointAPIKey) {
  // 如果已经在保存中（Enter触发），不重复保存
  if (prioritySaving.value) return
  savePriority(key)
}

async function savePriority(key: EndpointAPIKey) {
  const keyId = editingPriorityKey.value
  const newPriority = parseInt(String(editingPriorityValue.value), 10) || 0

  if (!keyId || newPriority < 0) {
    cancelEditPriority()
    return
  }

  // 如果优先级没有变化，直接取消编辑
  if (key.internal_priority === newPriority) {
    cancelEditPriority()
    return
  }

  cancelEditPriority()

  try {
    await updateProviderKey(keyId, { internal_priority: newPriority })
    showSuccess(t('ui.priorityUpdated'))
    // 更新本地数据 - 更新 providerKeys 中的数据
    const keyToUpdate = providerKeys.value.find(k => k.id === keyId)
    if (keyToUpdate) {
      keyToUpdate.internal_priority = newPriority
    }
    // 重新排序
    providerKeys.value.sort((a, b) => (a.internal_priority ?? 0) - (b.internal_priority ?? 0))
    emit('refresh')
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.failedToUpdatePriority')), t('ui.error'))
  }
}

// ===== 点击编辑倍率 =====
function startEditMultiplier(key: EndpointAPIKey, format: string) {
  editingMultiplierKey.value = key.id
  editingMultiplierFormat.value = format
  editingMultiplierValue.value = getKeyRateMultiplier(key, format)
  multiplierSaving.value = false
  nextTick(() => {
    const input = Array.isArray(multiplierInputRef.value) ? multiplierInputRef.value[0] : multiplierInputRef.value
    input?.focus()
    input?.select()
  })
}

function cancelEditMultiplier() {
  editingMultiplierKey.value = null
  editingMultiplierFormat.value = null
}

function handleMultiplierKeydown(e: KeyboardEvent, key: EndpointAPIKey, format: string) {
  if (e.key === 'Enter') {
    e.preventDefault()
    e.stopPropagation()
    saveMultiplier(key, format)
  } else if (e.key === 'Escape') {
    e.preventDefault()
    multiplierSaving.value = true // 阻止 blur 触发保存
    cancelEditMultiplier()
  }
}

function handleMultiplierBlur(key: EndpointAPIKey, format: string) {
  if (multiplierSaving.value) return
  saveMultiplier(key, format)
}

async function saveMultiplier(key: EndpointAPIKey, format: string) {
  // 防止重复调用（Enter 触发后阻止 blur 再次进入）
  if (multiplierSaving.value) return
  multiplierSaving.value = true

  const keyId = editingMultiplierKey.value
  const newMultiplier = parseFloat(String(editingMultiplierValue.value))

  // 验证输入有效性
  if (!keyId || isNaN(newMultiplier)) {
    showError(t('ui.enterAValidMultiplier'))
    cancelEditMultiplier()
    multiplierSaving.value = false
    return
  }

  // 验证合理范围
  if (newMultiplier <= 0 || newMultiplier > 100) {
    showError(t('ui.multiplierMustBeBetween001And100'))
    cancelEditMultiplier()
    multiplierSaving.value = false
    return
  }

  // 如果倍率没有变化,直接取消编辑（使用精度容差比较浮点数）
  const currentMultiplier = getKeyRateMultiplier(key, format)
  if (Math.abs(currentMultiplier - newMultiplier) < 0.0001) {
    cancelEditMultiplier()
    multiplierSaving.value = false
    return
  }

  cancelEditMultiplier()

  try {
    // 构建 rate_multipliers 对象
    const rateMultipliers = { ...(key.rate_multipliers || {}) }
    rateMultipliers[format] = newMultiplier

    await updateProviderKey(keyId, { rate_multipliers: rateMultipliers })
    showSuccess(t('ui.multiplierUpdated'))

    // 更新本地数据
    const keyToUpdate = providerKeys.value.find(k => k.id === keyId)
    if (keyToUpdate) {
      keyToUpdate.rate_multipliers = rateMultipliers
    }
    emit('refresh')
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.failedToUpdateMultiplier')), t('ui.error'))
  } finally {
    multiplierSaving.value = false
  }
}

// ===== 密钥列表拖拽排序 =====
function handleKeyDragStart(event: DragEvent, index: number) {
  keyDragState.value.isDragging = true
  keyDragState.value.draggedIndex = index
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', String(index))
  }
}

function handleKeyDragEnd() {
  keyDragState.value.isDragging = false
  keyDragState.value.draggedIndex = null
  keyDragState.value.targetIndex = null
}

function handleKeyDragOver(event: DragEvent, index: number) {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
  if (keyDragState.value.draggedIndex !== index) {
    keyDragState.value.targetIndex = index
  }
}

function handleKeyDragLeave() {
  keyDragState.value.targetIndex = null
}

async function handleKeyDrop(event: DragEvent, targetIndex: number) {
  event.preventDefault()

  const draggedIndex = keyDragState.value.draggedIndex
  if (draggedIndex === null || draggedIndex === targetIndex) {
    handleKeyDragEnd()
    return
  }

  const keys = allKeys.value.map(item => item.key)
  if (draggedIndex < 0 || draggedIndex >= keys.length || targetIndex < 0 || targetIndex >= keys.length) {
    handleKeyDragEnd()
    return
  }

  const draggedKey = keys[draggedIndex]
  const targetKey = keys[targetIndex]
  const draggedPriority = draggedKey.internal_priority ?? 0
  const targetPriority = targetKey.internal_priority ?? 0

  // 如果是同组内拖拽（同优先级），忽略操作
  if (draggedPriority === targetPriority) {
    handleKeyDragEnd()
    return
  }

  handleKeyDragEnd()

  try {
    // 记录每个 key 的原始优先级
    const originalPriorityMap = new Map<string, number>()
    keys.forEach(k => {
      originalPriorityMap.set(k.id, k.internal_priority ?? 0)
    })

    // 重排数组：将被拖动项移到目标位置
    const items = [...keys]
    items.splice(draggedIndex, 1)
    items.splice(targetIndex, 0, draggedKey)

    // 按新顺序分配优先级：被拖动项单独成组，其他同组项保持在一起
    const groupNewPriority = new Map<number, number>()
    let currentPriority = 1
    const newPriorityMap = new Map<string, number>()

    items.forEach(key => {
      const originalPriority = originalPriorityMap.get(key.id) ?? 0

      if (key === draggedKey) {
        // 被拖动的项单独成组
        newPriorityMap.set(key.id, currentPriority)
        currentPriority++
      } else {
        if (groupNewPriority.has(originalPriority)) {
          // 同组的其他项使用相同的新优先级
          newPriorityMap.set(key.id, groupNewPriority.get(originalPriority) ?? currentPriority)
        } else {
          // 新组，分配新优先级
          groupNewPriority.set(originalPriority, currentPriority)
          newPriorityMap.set(key.id, currentPriority)
          currentPriority++
        }
      }
    })

    // 更新所有优先级发生变化的 key
    const updatePromises = keys.map(key => {
      const oldPriority = key.internal_priority ?? 0
      const newPriority = newPriorityMap.get(key.id)
      if (newPriority !== undefined && oldPriority !== newPriority) {
        return updateProviderKey(key.id, { internal_priority: newPriority })
      }
      return Promise.resolve()
    })

    await Promise.all(updatePromises)
    showSuccess(t('ui.priorityUpdated'))
    await loadEndpoints()
    emit('refresh')
  } catch (err: unknown) {
    showError(localizedApiError(err, t('ui.failedToUpdatePriority')), t('ui.error'))
    await loadEndpoints()
  }
}

// 获取密钥的 API 格式列表（按指定顺序排序）
function getKeyApiFormats(key: EndpointAPIKey, endpoint?: ProviderEndpointWithKeys): string[] {
  const providerType = provider.value?.provider_type
  let formats: string[] = []

  if (
    providerType
    && isOAuthAccountProviderType(providerType)
    && isOAuthManagedCredential(key)
  ) {
    formats = [...availableKeyApiFormats.value]
  } else if (key.api_formats && key.api_formats.length > 0) {
    formats = [...key.api_formats]
  } else if (endpoint) {
    formats = [endpoint.api_format]
  }
  // 使用统一的排序函数
  return sortApiFormats(formats)
}

// 获取密钥在指定 API 格式下的成本倍率
function getKeyRateMultiplier(key: EndpointAPIKey, format: string): number {
  if (key.rate_multipliers && key.rate_multipliers[format] !== undefined) {
    return key.rate_multipliers[format]
  }
  return 1.0
}

// OAuth 订阅类型格式化
function formatOAuthPlanType(planType: string): string {
  const labels: Record<string, string> = {
    plus: 'Plus',
    pro: 'Pro',
    free: 'Free',
    paid: 'Paid',
    team: 'Team',
    enterprise: 'Enterprise',
    ultra: 'Ultra',
    basic: 'Basic',
    super: 'Super',
    heavy: 'Heavy',
  }
  return labels[planType.toLowerCase()] || planType
}

// OAuth 订阅类型样式
function getOAuthPlanTypeClass(planType: string): string {
  const classes: Record<string, string> = {
    plus: 'border-green-500/50 text-green-600 dark:text-green-400',
    pro: 'border-blue-500/50 text-blue-600 dark:text-blue-400',
    free: 'border-primary/50 text-primary',
    paid: 'border-blue-500/50 text-blue-600 dark:text-blue-400',
    team: 'border-purple-500/50 text-purple-600 dark:text-purple-400',
    enterprise: 'border-amber-500/50 text-amber-600 dark:text-amber-400',
    ultra: 'border-amber-500/50 text-amber-600 dark:text-amber-400',
    'pro+': 'border-purple-500/50 text-purple-600 dark:text-purple-400',
    power: 'border-amber-500/50 text-amber-600 dark:text-amber-400',
    basic: 'border-primary/50 text-primary',
    super: 'border-green-500/50 text-green-600 dark:text-green-400',
    heavy: 'border-amber-500/50 text-amber-600 dark:text-amber-400',
  }
  return classes[planType.toLowerCase()] || ''
}

// OAuth 状态信息（包括失效和过期）
function getKeyOAuthExpires(key: EndpointAPIKey) {
  return getOAuthStatusDisplayWithFallback(key, countdownTick.value)
}

function getOAuthRefreshButtonTitle(key: EndpointAPIKey): string {
  return resolveOAuthRefreshButtonTitle(key, countdownTick.value)
}

// OAuth 状态的 title 提示
function getOAuthStatusTitle(key: EndpointAPIKey): string {
  const accountTitle = getAccountStatusTitle(key)
  if (accountTitle && isAccountLevelBlock(key)) {
    return accountTitle
  }
  return resolveOAuthStatusTitle(key, countdownTick.value)
}

// 健康度颜色
function getHealthScoreColor(score: number): string {
  if (score >= 0.8) return 'text-green-600 dark:text-green-400'
  if (score >= 0.5) return 'text-yellow-600 dark:text-yellow-400'
  return 'text-red-600 dark:text-red-400'
}

function getHealthScoreBarColor(score: number): string {
  if (score >= 0.8) return 'bg-green-500 dark:bg-green-400'
  if (score >= 0.5) return 'bg-yellow-500 dark:bg-yellow-400'
  return 'bg-red-500 dark:bg-red-400'
}

function isKeyRecoverable(key: EndpointAPIKey): boolean {
  return Boolean(
    key.circuit_breaker_open
    || (key.health_score !== undefined && key.health_score < 0.5)
  )
}

function getOpenCircuitEntries(key: EndpointAPIKey): Array<[string, NonNullable<EndpointAPIKey['circuit_breaker_by_format']>[string]]> {
  return Object.entries(key.circuit_breaker_by_format || {})
    .filter(([, value]) => value?.open === true)
}

function getKeyCircuitProbeCountdown(key: EndpointAPIKey): string {
  void countdownTick.value
  const nextProbe = getOpenCircuitEntries(key)
    .map(([, value]) => {
      if (typeof value.next_probe_at_unix_secs === 'number' && Number.isFinite(value.next_probe_at_unix_secs)) {
        return value.next_probe_at_unix_secs * 1000
      }
      if (value.next_probe_at) {
        const ms = new Date(value.next_probe_at).getTime()
        return Number.isFinite(ms) ? ms : null
      }
      return null
    })
    .filter((value): value is number => value !== null)
    .sort((a, b) => a - b)[0]
  if (!nextProbe) {
    return ''
  }
  const diffMs = nextProbe - Date.now()
  return diffMs > 0 ? ` ${formatCountdown(diffMs)}` : ` ${t('ui.probing')}`
}

function getKeyCircuitBreakerTitle(key: EndpointAPIKey): string {
  const entries = getOpenCircuitEntries(key)
  if (entries.length === 0) return t('ui.circuitBreakerIsOpen')
  const parts = entries.map(([format, value]) => {
    const label = formatApiFormatShort(format)
    const reason = value.reason ? `${t('ui.reasonf029d9c')}: ${value.reason}` : `${t('ui.reasonf029d9c')}: ${t('ui.consecutiveFailures')}`
    const interval = typeof value.probe_interval_minutes === 'number'
      ? `${t('ui.probeInterval')}: ${value.probe_interval_minutes} ${t('ui.minutesbd957bc')}`
      : ''
    const countdown = getFormatProbeCountdown(key, format).trim()
    return [label, reason, interval, countdown ? `${t('ui.status6320b4a')}: ${countdown}` : '']
      .filter(Boolean)
      .join(' / ')
  })
  parts.push(t('ui.useTheRecoverButtonToResetTheCircuitBreaker'))
  return parts.join('\n')
}

function getRecoverKeyTitle(key: EndpointAPIKey): string {
  if (key.circuit_breaker_open) {
    return t('ui.resetCircuitBreakerAndRecoverHealth')
  }
  return t('ui.refreshHealthStatus')
}

// 获取自动获取模型状态的 title 提示
function getAutoFetchStatusTitle(key: EndpointAPIKey): string {
  const parts: string[] = [t('ui.autoModelFetchIsEnabled')]

  if (key.last_models_fetch_at) {
    const date = new Date(key.last_models_fetch_at)
    parts.push(`${t('ui.lastSync')}: ${date.toLocaleString(locale.value)}`)
  }

  if (key.last_models_fetch_error) {
    parts.push(`${t('ui.error')}: ${key.last_models_fetch_error}`)
  }

  return parts.join('\n')
}

// 检查指定格式是否熔断
function isFormatCircuitOpen(key: EndpointAPIKey, format: string): boolean {
  if (!key.circuit_breaker_by_format) return false
  const formatData = key.circuit_breaker_by_format[format]
  return formatData?.open === true
}

// 获取指定格式的探测倒计时（如果熔断，返回带空格前缀的倒计时文本）
function getFormatProbeCountdown(key: EndpointAPIKey, format: string): string {
  // 触发响应式更新
  void countdownTick.value

  if (!key.circuit_breaker_by_format) return ''
  const formatData = key.circuit_breaker_by_format[format]
  if (!formatData?.open) return ''

  // 半开状态
  if (formatData.half_open_until) {
    const halfOpenUntil = new Date(formatData.half_open_until)
    const now = new Date()
    if (halfOpenUntil > now) {
      return ` ${t('ui.probing')}`
    }
  }
  // 等待探测
  if (formatData.next_probe_at_unix_secs || formatData.next_probe_at) {
    const nextProbeMs = typeof formatData.next_probe_at_unix_secs === 'number'
      ? formatData.next_probe_at_unix_secs * 1000
      : new Date(formatData.next_probe_at || '').getTime()
    const diffMs = nextProbeMs - Date.now()
    if (diffMs > 0) {
      return ` ${formatCountdown(diffMs)}`
    } else {
      return ` ${t('ui.probing')}`
    }
  }
  return ''
}

// 加载系统级格式转换配置
async function loadSystemFormatConversionConfig() {
  try {
    const result = await adminApi.getSystemConfig('enable_format_conversion')
    systemFormatConversionEnabled.value = result.value === true
  } catch {
    // 获取失败时默认为关闭
    systemFormatConversionEnabled.value = false
  }
}

// 加载 Provider 信息
async function loadProvider() {
  if (!props.providerId) return
  const requestId = ++providerLoadRequestId
  const shouldShowSpinner = !provider.value || provider.value.id !== props.providerId

  try {
    if (shouldShowSpinner) {
      loading.value = true
    }
    // 系统级格式转换配置只影响一个图标状态，不应阻塞详情抽屉首屏。
    void loadSystemFormatConversionConfig()
    const providerData = await getProvider(props.providerId)
    if (requestId !== providerLoadRequestId) return
    applyProviderSnapshot(providerData)
    keyPageSize.value = getProviderKeysPageSize(providerData.provider_type)

    if (!provider.value) {
      throw new Error(t('ui.providerDoesNotExist'))
    }
  } catch (err: unknown) {
    if (requestId !== providerLoadRequestId) return
    showError(localizedApiError(err, t('ui.failedToLoad')), t('ui.error'))
  } finally {
    if (requestId === providerLoadRequestId && shouldShowSpinner) {
      loading.value = false
    }
  }
}

async function loadProviderKeysPage(page = currentKeyPage.value) {
  if (!props.providerId) return
  const providerId = props.providerId
  const requestId = ++keysLoadRequestId
  loadingProviderKeys.value = true

  try {
    const result = await getProviderKeysPage(providerId, {
      page,
      page_size: keyPageSize.value,
    })
    if (requestId !== keysLoadRequestId || props.providerId !== providerId) return

    const nextTotalPages = Math.max(1, Math.ceil(result.total / result.page_size))
    if (result.keys.length === 0 && result.total > 0 && result.page > nextTotalPages) {
      await loadProviderKeysPage(nextTotalPages)
      return
    }

    providerKeys.value = result.keys
    providerKeysTotal.value = result.total
    currentKeyPage.value = Math.min(result.page, nextTotalPages)
    keyPageSize.value = result.page_size
    syncCurrentSelections(endpoints.value, result.keys)
  } catch (err: unknown) {
    if (requestId !== keysLoadRequestId || props.providerId !== providerId) return
    providerKeys.value = []
    providerKeysTotal.value = 0
    syncCurrentSelections(endpoints.value, [])
    showError(localizedApiError(err, t('ui.failedToLoadKeys')), t('ui.error'))
  } finally {
    if (requestId === keysLoadRequestId) {
      loadingProviderKeys.value = false
    }
  }
}

// 加载端点列表
async function loadEndpoints() {
  if (!props.providerId) return
  const providerId = props.providerId
  const requestId = ++endpointsLoadRequestId
  loadingProviderEndpoints.value = true
  loadingProviderKeys.value = true
  loadingProviderModels.value = true

  const sortEndpoints = (items: ProviderEndpoint[]): ProviderEndpointWithKeys[] => {
    return [...items].sort((a, b) => {
      const aIdx = API_FORMAT_ORDER.indexOf(a.api_format)
      const bIdx = API_FORMAT_ORDER.indexOf(b.api_format)
      if (aIdx === -1 && bIdx === -1) return 0
      if (aIdx === -1) return 1
      if (bIdx === -1) return -1
      return aIdx - bIdx
    })
  }

  const endpointsPromise = getProviderEndpoints(providerId)
    .then((endpointsList) => {
      if (requestId !== endpointsLoadRequestId) return
      const sortedEndpoints = sortEndpoints(endpointsList)
      endpoints.value = sortedEndpoints
      syncCurrentSelections(sortedEndpoints, providerKeys.value)
    })
    .catch((err: unknown) => {
      if (requestId !== endpointsLoadRequestId) return
      endpoints.value = []
      syncCurrentSelections([], providerKeys.value)
      showError(localizedApiError(err, t('ui.failedToLoadEndpoints')), t('ui.error'))
    })
    .finally(() => {
      if (requestId === endpointsLoadRequestId) {
        loadingProviderEndpoints.value = false
      }
    })

  const providerKeysPromise = loadProviderKeysPage(currentKeyPage.value)

  const modelsPromise = getProviderModels(providerId)
    .catch(() => [])
    .then((modelsResult) => {
      if (requestId !== endpointsLoadRequestId) return
      providerModels.value = modelsResult
    })
    .finally(() => {
      if (requestId === endpointsLoadRequestId) {
        loadingProviderModels.value = false
      }
    })

  await Promise.allSettled([endpointsPromise, providerKeysPromise, modelsPromise])
}

// 加载映射预览（独立于 loadEndpoints，不阻塞首屏渲染）
async function loadMappingPreview() {
  if (!props.providerId) return
  const requestId = ++mappingPreviewLoadRequestId
  loadingProviderMappingPreview.value = true
  try {
    const preview = await getProviderMappingPreview(props.providerId)
    if (requestId !== mappingPreviewLoadRequestId) return
    providerMappingPreview.value = preview
  } catch {
    if (requestId !== mappingPreviewLoadRequestId) return
    providerMappingPreview.value = null
  } finally {
    if (requestId === mappingPreviewLoadRequestId) {
      loadingProviderMappingPreview.value = false
    }
  }
}

</script>

<style scoped>
/* 轻量滚动条（用于 Antigravity 模型配额等小区域） */
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: hsl(var(--muted-foreground) / 0.2);
  border-radius: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background-color: hsl(var(--muted-foreground) / 0.4);
}
</style>
