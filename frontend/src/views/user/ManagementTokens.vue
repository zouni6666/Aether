<template>
  <div class="space-y-6 pb-8">
    <!-- 访问令牌表格 -->
    <Card
      variant="default"
      class="overflow-hidden"
    >
      <!-- 标题和操作栏 -->
      <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <div>
            <h3 class="text-sm sm:text-base font-semibold">
              {{ isAdminPage ? t('ui.managementToken') : t('guide.nav.modules.managementTokens') }}
            </h3>
            <p class="text-xs text-muted-foreground mt-0.5">
              <template v-if="quota">
                {{ t('ui.created') }} {{ quota.used }}/{{ quota.max }} {{ t('ui.tokens') }} <span
                  v-if="quota.used >= quota.max"
                  class="text-destructive font-medium"
                >{{ t('ui.limitReachedf3c5acc') }}</span>
              </template>
              <template v-else-if="canManageTokens">
                {{ t('ui.tokensForProgrammaticAccessToTheManagementApi') }}
              </template>
              <template v-else>
                {{ t('ui.onlyAdministratorsCanCreateAndManageTheseTokens') }}
              </template>
            </p>
          </div>

          <!-- 操作按钮 -->
          <div class="flex items-center gap-2">
            <!-- 新增按钮 -->
            <Button
              v-if="canManageTokens"
              variant="ghost"
              size="icon"
              class="h-8 w-8"
              :title="t('ui.createNewToken')"
              :disabled="quota ? quota.used >= quota.max : false"
              @click="openCreateDialog"
            >
              <Plus class="w-3.5 h-3.5" />
            </Button>

            <!-- 刷新按钮 -->
            <RefreshButton
              :loading="loading"
              @click="loadTokens"
            />
          </div>
        </div>
      </div>

      <!-- 加载状态 -->
      <div
        v-if="loading"
        class="flex items-center justify-center py-12"
      >
        <LoadingState :message="t('common.loading')" />
      </div>

      <!-- 空状态 -->
      <div
        v-else-if="tokens.length === 0"
        class="flex items-center justify-center py-12"
      >
        <EmptyState
          :title="t('ui.noAccessTokens')"
          :description="t('ui.createAnAccessTokenToUseTheManagementApi')"
          :icon="KeyRound"
        >
          <template #actions>
            <Button
              v-if="canManageTokens"
              size="lg"
              class="shadow-lg shadow-primary/20"
              @click="openCreateDialog"
            >
              <Plus class="mr-2 h-4 w-4" /> {{ t('ui.createAccessToken') }}
            </Button>
          </template>
        </EmptyState>
      </div>

      <!-- 桌面端表格 -->
      <div
        v-else
        class="hidden md:block overflow-x-auto"
      >
        <Table>
          <TableHeader>
            <TableRow class="border-b border-border/60 hover:bg-transparent">
              <TableHead class="min-w-[180px] h-12 font-semibold">
                {{ t('stats.column.name') }}
              </TableHead>
              <TableHead class="min-w-[160px] h-12 font-semibold">
                {{ t('ui.token') }}
              </TableHead>
              <TableHead
                v-if="isAdminPage"
                class="min-w-[160px] h-12 font-semibold"
              >
                {{ t('ui.owner') }}
              </TableHead>
              <TableHead class="min-w-[150px] h-12 font-semibold">
                {{ t('ui.permissions') }}
              </TableHead>
              <TableHead class="min-w-[150px] h-12 font-semibold">
                {{ t('ui.ipRestrictions477036d') }}
              </TableHead>
              <TableHead class="min-w-[80px] h-12 font-semibold text-center">
                {{ t('ui.usageCount') }}
              </TableHead>
              <TableHead class="min-w-[70px] h-12 font-semibold text-center">
                {{ t('ui.status6320b4a') }}
              </TableHead>
              <TableHead class="min-w-[100px] h-12 font-semibold">
                {{ t('ui.time') }}
              </TableHead>
              <TableHead
                v-if="canManageTokens"
                class="min-w-[100px] h-12 font-semibold text-center"
              >
                {{ t('ui.actionsed31fbb') }}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow
              v-for="token in paginatedTokens"
              :key="token.id"
              class="border-b border-border/40 hover:bg-muted/30 transition-colors"
            >
              <!-- 名称 -->
              <TableCell class="py-4">
                <div class="flex-1 min-w-0">
                  <div
                    class="text-sm font-semibold truncate"
                    :title="token.name"
                  >
                    {{ token.name }}
                  </div>
                  <div
                    v-if="token.description"
                    class="text-xs text-muted-foreground mt-0.5 truncate"
                    :title="token.description"
                  >
                    {{ token.description }}
                  </div>
                </div>
              </TableCell>

              <!-- Token 显示 -->
              <TableCell class="py-4">
                <div class="flex items-center gap-1.5">
                  <code class="text-xs font-mono text-muted-foreground bg-muted/30 px-2 py-1 rounded">
                    {{ token.token_display }}
                  </code>
                  <Button
                    v-if="canManageTokens"
                    variant="ghost"
                    size="icon"
                    class="h-6 w-6"
                    :title="t('ui.regenerateToken')"
                    @click="confirmRegenerate(token)"
                  >
                    <RefreshCw class="h-3.5 w-3.5" />
                  </Button>
                </div>
              </TableCell>

              <TableCell
                v-if="isAdminPage"
                class="py-4"
              >
                <div class="text-sm font-medium truncate">
                  {{ token.user?.username || token.user?.email || token.user_id }}
                </div>
                <div
                  v-if="token.user?.email"
                  class="text-xs text-muted-foreground truncate"
                >
                  {{ token.user.email }}
                </div>
              </TableCell>

              <TableCell class="py-4">
                <Badge
                  variant="secondary"
                  class="font-medium px-2 py-1"
                >
                  {{ token.permission_summary || permissionModeText(token.permission_mode) }}
                </Badge>
              </TableCell>

              <TableCell class="py-4 text-xs text-muted-foreground">
                <div class="truncate">
                  {{ token.allowed_ips?.length ? token.allowed_ips.join(', ') : t('ui.noRestriction') }}
                </div>
                <div class="mt-1">
                  {{ token.last_used_ip ? t('ui.lastIpValue', { value: token.last_used_ip }) : t('ui.noLastIp') }}
                </div>
              </TableCell>

              <!-- 使用次数 -->
              <TableCell class="py-4 text-center">
                <span class="text-sm font-medium">
                  {{ formatNumber(token.usage_count || 0) }}
                </span>
              </TableCell>

              <!-- 状态 -->
              <TableCell class="py-4 text-center">
                <Badge
                  :variant="getStatusVariant(token)"
                  class="font-medium px-3 py-1"
                >
                  {{ getStatusText(token) }}
                </Badge>
              </TableCell>

              <!-- 时间 -->
              <TableCell class="py-4 text-sm text-muted-foreground">
                <div class="text-xs">
                  {{ t('ui.createdAtdc4d662') }} {{ formatDateOnly(token.created_at) }}
                </div>
                <div class="text-xs mt-1">
                  {{ token.last_used_at ? t('ui.lastUsedValue', { value: formatTimeAgo(token.last_used_at) }) : t('ui.neverUsed') }}
                </div>
              </TableCell>

              <!-- 操作按钮 -->
              <TableCell
                v-if="canManageTokens"
                class="py-4"
              >
                <div class="flex justify-center gap-1">
                  <Button
                    variant="ghost"
                    size="icon"
                    class="h-8 w-8"
                    :title="t('ui.edit0518365')"
                    @click="openEditDialog(token)"
                  >
                    <Pencil class="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    class="h-8 w-8"
                    :title="token.is_active ? t('ui.disable') : t('ui.enablef4f0ead')"
                    @click="toggleToken(token)"
                  >
                    <Power class="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    class="h-8 w-8"
                    :title="t('ui.delete')"
                    @click="confirmDelete(token)"
                  >
                    <Trash2 class="h-4 w-4" />
                  </Button>
                </div>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>

      <!-- 移动端卡片列表 -->
      <div
        v-if="!loading && tokens.length > 0"
        class="md:hidden space-y-3 p-4"
      >
        <Card
          v-for="token in paginatedTokens"
          :key="token.id"
          variant="default"
          class="group hover:shadow-md hover:border-primary/30 transition-all duration-200"
        >
          <div class="p-4">
            <!-- 第一行：名称、状态、操作 -->
            <div class="flex items-center justify-between mb-2">
              <div class="flex items-center gap-2 min-w-0 flex-1">
                <h3 class="text-sm font-semibold text-foreground truncate">
                  {{ token.name }}
                </h3>
                <Badge
                  :variant="getStatusVariant(token)"
                  class="text-xs px-1.5 py-0"
                >
                  {{ getStatusText(token) }}
                </Badge>
              </div>
              <div
                v-if="canManageTokens"
                class="flex items-center gap-0.5 flex-shrink-0"
              >
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7"
                  :title="t('ui.edit0518365')"
                  @click="openEditDialog(token)"
                >
                  <Pencil class="h-3.5 w-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7"
                  :title="token.is_active ? t('ui.disable') : t('ui.enablef4f0ead')"
                  @click="toggleToken(token)"
                >
                  <Power class="h-3.5 w-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7"
                  :title="t('ui.delete')"
                  @click="confirmDelete(token)"
                >
                  <Trash2 class="h-3.5 w-3.5" />
                </Button>
              </div>
            </div>

            <!-- Token 显示 -->
            <div class="flex items-center gap-2 text-xs mb-2">
              <code class="font-mono text-muted-foreground">{{ token.token_display }}</code>
              <Button
                v-if="canManageTokens"
                variant="ghost"
                size="icon"
                class="h-5 w-5"
                :title="t('ui.regenerate')"
                @click="confirmRegenerate(token)"
              >
                <RefreshCw class="h-3 w-3" />
              </Button>
            </div>

            <!-- 统计信息 -->
            <div class="flex items-center gap-3 text-xs text-muted-foreground">
              <span>{{ formatNumber(token.usage_count || 0) }} {{ t('ui.uses') }}</span>
              <span>·</span>
              <span>{{ token.permission_summary || permissionModeText(token.permission_mode) }}</span>
              <span>·</span>
              <span>{{ token.last_used_at ? formatTimeAgo(token.last_used_at) : t('ui.neverUsed') }}</span>
            </div>
          </div>
        </Card>
      </div>

      <!-- 分页 -->
      <Pagination
        v-if="totalTokens > 0"
        :current="currentPage"
        :total="totalTokens"
        :page-size="pageSize"
        cache-key="management-tokens-page-size"
        @update:current="currentPage = $event"
        @update:page-size="handlePageSizeChange"
      />
    </Card>

    <!-- 创建/编辑 Token 对话框 -->
    <Dialog
      v-model="showCreateDialog"
      size="lg"
    >
      <template #header>
        <div class="border-b border-border px-6 py-4">
          <div class="flex items-center gap-3">
            <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-primary/10 flex-shrink-0">
              <KeyRound class="h-5 w-5 text-primary" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-lg font-semibold text-foreground leading-tight">
                {{ editingToken ? t('ui.editAccessToken') : t('ui.createAccessToken') }}
              </h3>
              <p class="text-xs text-muted-foreground">
                {{ editingToken ? t('ui.editTokenSettings') : t('ui.createATokenToAccessTheManagementApi') }}
              </p>
            </div>
          </div>
        </div>
      </template>

      <div class="space-y-4">
        <!-- 名称 -->
        <div class="space-y-2">
          <Label
            for="token-name"
            class="text-sm font-semibold"
          >{{ t('ui.name') }}</Label>
          <Input
            id="token-name"
            v-model="formData.name"
            :placeholder="t('ui.eGCiCdAutomation')"
            class="h-11 border-border/60"
            autocomplete="off"
            required
          />
        </div>

        <!-- 描述 -->
        <div class="space-y-2">
          <Label
            for="token-description"
            class="text-sm font-semibold"
          >{{ t('ui.descriptiondc2ba46') }}</Label>
          <Input
            id="token-description"
            v-model="formData.description"
            :placeholder="t('ui.purposeOptional')"
            class="h-11 border-border/60"
            autocomplete="off"
          />
        </div>

        <!-- IP 限制 -->
        <div class="space-y-2">
          <Label
            for="token-ips"
            class="text-sm font-semibold"
          >{{ t('ui.ipRestrictions477036d') }}</Label>
          <Input
            id="token-ips"
            v-model="formData.allowedIpsText"
            :placeholder="t('ui.eG192168100002410')"
            class="h-11 border-border/60"
            autocomplete="off"
          />
          <p class="text-xs text-muted-foreground">
            {{ t('ui.leaveEmptyForNoRestrictionSupportsIpCidrIpv4Wildcardsc2888de') }}
          </p>
        </div>

        <!-- 过期时间 -->
        <div class="space-y-2">
          <Label
            for="token-expires"
            class="text-sm font-semibold"
          >{{ t('ui.expirationTime') }}</Label>
          <Input
            id="token-expires"
            v-model="formData.expiresAt"
            type="datetime-local"
            class="h-11 border-border/60"
          />
          <p class="text-xs text-muted-foreground">
            {{ t('ui.leaveEmptyForNoExpiration') }}
          </p>
        </div>

        <!-- 权限 -->
        <div class="space-y-3">
          <Label class="text-sm font-semibold">{{ t('ui.permissions') }}</Label>
          <div class="grid grid-cols-3 gap-2">
            <Button
              type="button"
              :variant="permissionMode === 'full' ? 'default' : 'outline'"
              class="h-9"
              @click="setPermissionMode('full')"
            >
              {{ t('ui.fullAccess') }}
            </Button>
            <Button
              type="button"
              :variant="permissionMode === 'read_only' ? 'default' : 'outline'"
              class="h-9"
              @click="setPermissionMode('read_only')"
            >
              {{ t('ui.readOnly') }}
            </Button>
            <Button
              type="button"
              :variant="permissionMode === 'custom' ? 'default' : 'outline'"
              class="h-9"
              @click="setPermissionMode('custom')"
            >
              {{ t('ui.custom4eafa9e') }}
            </Button>
          </div>

          <div
            v-if="permissionMode === 'custom'"
            class="flex flex-wrap items-center gap-2"
          >
            <Button
              type="button"
              variant="outline"
              class="h-8 px-3 text-xs"
              @click="setCustomPermissions('none')"
            >
              {{ t('ui.disableAll') }}
            </Button>
            <Button
              type="button"
              variant="outline"
              class="h-8 px-3 text-xs"
              @click="setCustomPermissions('read_only')"
            >
              {{ t('ui.setAllToReadOnly') }}
            </Button>
            <Button
              type="button"
              variant="outline"
              class="h-8 px-3 text-xs"
              @click="setCustomPermissions('full')"
            >
              {{ t('ui.grantAllPermissions') }}
            </Button>
          </div>

          <div
            v-if="permissionMode === 'custom'"
            class="max-h-72 overflow-y-auto rounded-md border border-border/60"
          >
            <div
              v-for="group in permissionGroups"
              :key="group.scope"
              class="border-b border-border/50 last:border-b-0 px-3 py-2"
            >
              <div class="flex items-center justify-between gap-3">
                <div class="text-sm font-medium">
                  {{ group.label }}
                </div>
                <div class="flex items-center gap-3">
                  <label class="inline-flex items-center gap-1.5 text-xs text-muted-foreground">
                    <Checkbox
                      :checked="isPermissionGroupDenied(group)"
                      @update:checked="togglePermissionGroupDenied(group, $event)"
                    />
                    <span>{{ t('ui.deny') }}</span>
                  </label>
                  <label
                    v-for="item in group.items"
                    :key="item.key"
                    class="inline-flex items-center gap-1.5 text-xs text-muted-foreground"
                  >
                    <Checkbox
                      :checked="selectedPermissions.includes(item.key)"
                      @update:checked="togglePermission(item.key, $event)"
                    />
                    <span>{{ item.access_label }}</span>
                  </label>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <Button
          variant="outline"
          class="h-11 px-6"
          @click="closeDialog"
        >
          {{ t('common.cancel') }}
        </Button>
        <Button
          class="h-11 px-6 shadow-lg shadow-primary/20"
          :disabled="saving || !isFormValid"
          @click="saveToken"
        >
          <Loader2
            v-if="saving"
            class="animate-spin h-4 w-4 mr-2"
          />
          {{ saving ? t('ui.saving') : (editingToken ? t('ui.save') : t('ui.create')) }}
        </Button>
      </template>
    </Dialog>

    <!-- 新 Token 创建成功对话框 -->
    <Dialog
      v-model="showTokenDialog"
      size="lg"
      persistent
    >
      <template #header>
        <div class="border-b border-border px-6 py-4">
          <div class="flex items-center gap-3">
            <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-emerald-100 dark:bg-emerald-900/30 flex-shrink-0">
              <CheckCircle class="h-5 w-5 text-emerald-600 dark:text-emerald-400" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-lg font-semibold text-foreground leading-tight">
                {{ isRegenerating ? t('ui.tokenRegenerated') : t('ui.createdSuccessfully') }}
              </h3>
              <p class="text-xs text-muted-foreground">
                {{ t('ui.storeThisTokenSecurelyItIsShownOnlyOnce') }}
              </p>
            </div>
          </div>
        </div>
      </template>

      <div class="space-y-4">
        <div class="space-y-2">
          <Label class="text-sm font-medium">{{ t('guide.nav.modules.managementTokens') }}</Label>
          <div class="flex items-center gap-2">
            <Input
              type="text"
              :value="newTokenValue"
              readonly
              class="flex-1 font-mono text-sm bg-muted/50 h-11"
              @click="($event.target as HTMLInputElement)?.select()"
            />
            <Button
              class="h-11"
              @click="copyToClipboard(newTokenValue)"
            >
              {{ t('common.copy') }}
            </Button>
          </div>
        </div>
        <div class="p-3 rounded-lg bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800">
          <div class="flex gap-2">
            <AlertTriangle class="h-4 w-4 text-amber-600 dark:text-amber-400 flex-shrink-0 mt-0.5" />
            <p class="text-sm text-amber-800 dark:text-amber-200">
              {{ t('ui.thisTokenIsShownOnlyOnceStoreItSecurelyBefore') }}
            </p>
          </div>
        </div>
      </div>

      <template #footer>
        <Button
          class="h-10 px-5"
          @click="showTokenDialog = false"
        >
          {{ t('ui.iHaveSavedItSecurely') }}
        </Button>
      </template>
    </Dialog>

    <!-- 删除确认对话框 -->
    <AlertDialog
      v-model="showDeleteDialog"
      type="danger"
      :title="t('ui.confirmDeletion')"
      :description="t('ui.deleteTokenValueThisCannotBeUndone', { name: tokenToDelete?.name ?? '' })"
      :confirm-text="t('ui.delete')"
      :loading="deleting"
      @confirm="deleteToken"
      @cancel="showDeleteDialog = false"
    />

    <!-- 重新生成确认对话框 -->
    <AlertDialog
      v-model="showRegenerateDialog"
      type="warning"
      :title="t('ui.confirmRegeneration')"
      :description="t('ui.regeneratingWillImmediatelyInvalidateTheOriginalTokenRegenerateValue', { name: tokenToRegenerate?.name ?? '' })"
      :confirm-text="t('ui.regenerate')"
      :loading="regenerating"
      @confirm="regenerateToken"
      @cancel="showRegenerateDialog = false"
    />
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { ref, computed, onMounted, reactive, watch } from 'vue'
import { useRoute } from 'vue-router'
import {
  adminManagementTokenApi,
  managementTokenApi,
  type ManagementToken,
  type ManagementTokenPermissionCatalogItem
} from '@/api/management-tokens'
import { useAuthStore } from '@/stores/auth'
import Card from '@/components/ui/card.vue'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import Badge from '@/components/ui/badge.vue'
import Checkbox from '@/components/ui/checkbox.vue'
import { Dialog, Pagination } from '@/components/ui'
import { LoadingState, AlertDialog, EmptyState } from '@/components/common'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow
} from '@/components/ui'
import RefreshButton from '@/components/ui/refresh-button.vue'
import {
  Plus,
  KeyRound,
  Trash2,
  Loader2,
  CheckCircle,
  Power,
  Pencil,
  RefreshCw,
  AlertTriangle
} from 'lucide-vue-next'
import { useToast } from '@/composables/useToast'
import { log } from '@/utils/logger'
import { parseApiError } from '@/utils/errorParser'
import { useClipboard } from '@/composables/useClipboard'
import { formatDateOnly, formatNumber, formatTimeAgo } from '@/utils/format'

const { success, error: showError } = useToast()
const { copyToClipboard } = useClipboard()
const route = useRoute()
const authStore = useAuthStore()

const isAdminPage = computed(() => route.path.startsWith('/admin'))
const canManageTokens = computed(() => authStore.isAdmin)

// 数据
const tokens = ref<ManagementToken[]>([])
const totalTokens = ref(0)
const loading = ref(false)
const saving = ref(false)
const deleting = ref(false)
const regenerating = ref(false)

// 配额信息
const quota = ref<{ used: number; max: number } | null>(null)

// 分页
const currentPage = ref(1)
const pageSize = ref(10)

const paginatedTokens = computed(() => tokens.value)

const permissionCatalog = ref<ManagementTokenPermissionCatalogItem[]>([])
const allPermissionKeys = ref<string[]>([])
const readOnlyPermissionKeys = ref<string[]>([])
const permissionMode = ref<'full' | 'read_only' | 'custom'>('full')
const selectedPermissions = ref<string[]>([])

const permissionGroups = computed(() => {
  const groups = new Map<string, { scope: string; label: string; items: ManagementTokenPermissionCatalogItem[] }>()
  for (const item of permissionCatalog.value) {
    const group = groups.get(item.scope)
    if (group) {
      group.items.push(item)
    } else {
      groups.set(item.scope, {
        scope: item.scope,
        label: item.scope_label,
        items: [item]
      })
    }
  }
  return Array.from(groups.values())
})

// 监听分页变化
watch([currentPage, pageSize], () => {
  loadTokens()
})

function handlePageSizeChange(newSize: number) {
  pageSize.value = newSize
  currentPage.value = 1
}

// 对话框状态
const showCreateDialog = ref(false)
const showTokenDialog = ref(false)
const showDeleteDialog = ref(false)
const showRegenerateDialog = ref(false)

// 表单数据
const editingToken = ref<ManagementToken | null>(null)
const formData = reactive({
  name: '',
  description: '',
  allowedIpsText: '',
  expiresAt: ''
})

const newTokenValue = ref('')
const isRegenerating = ref(false)
const tokenToDelete = ref<ManagementToken | null>(null)
const tokenToRegenerate = ref<ManagementToken | null>(null)

// 表单验证
const isFormValid = computed(() => {
  if (formData.name.trim().length === 0) return false
  if (permissionMode.value === 'custom' && selectedPermissions.value.length === 0) {
    return false
  }
  return true
})

function getStatusVariant(token: ManagementToken): 'success' | 'secondary' | 'destructive' {
  if (token.expires_at && isExpired(token.expires_at)) {
    return 'destructive'
  }
  return token.is_active ? 'success' : 'secondary'
}

function getStatusText(token: ManagementToken): string {
  if (token.expires_at && isExpired(token.expires_at)) {
    return t('ui.expired')
  }
  return token.is_active ? t('ui.activee85e590') : t('ui.disable')
}

function isExpired(dateString: string): boolean {
  return new Date(dateString) < new Date()
}

// 加载数据
onMounted(() => {
  loadTokens()
})

watch(
  canManageTokens,
  (allowed) => {
    if (allowed) {
      void loadPermissionCatalog()
    }
  },
  { immediate: true }
)

async function loadTokens() {
  loading.value = true
  try {
    const skip = (currentPage.value - 1) * pageSize.value
    const response = isAdminPage.value
      ? await adminManagementTokenApi.listAllTokens({ skip, limit: pageSize.value })
      : await managementTokenApi.listTokens({ skip, limit: pageSize.value })

    tokens.value = response.items
    totalTokens.value = response.total

    quota.value = response.quota ?? null

    // 如果当前页超出范围，重置到第一页
    if (tokens.value.length === 0 && currentPage.value > 1) {
      currentPage.value = 1
    }
  } catch (err: unknown) {
    log.error('加载 Management Tokens 失败:', err)
    showError(parseApiError(err, t('ui.failedToLoad')))
  } finally {
    loading.value = false
  }
}

async function loadPermissionCatalog() {
  if (permissionCatalog.value.length > 0) return
  try {
    const response = await adminManagementTokenApi.getPermissionCatalog()
    permissionCatalog.value = response.items
    allPermissionKeys.value = response.all_permissions
    readOnlyPermissionKeys.value = response.read_only_permissions
    if (selectedPermissions.value.length === 0) {
      selectedPermissions.value = [...response.all_permissions]
    }
  } catch (err: unknown) {
    log.error('加载 Management Token 权限目录失败:', err)
    showError(parseApiError(err, t('ui.failedToLoadAvailablePermissions')))
  }
}

function openCreateDialog() {
  if (!canManageTokens.value) return
  resetForm()
  permissionMode.value = 'full'
  selectedPermissions.value = [...allPermissionKeys.value]
  void loadPermissionCatalog()
  showCreateDialog.value = true
}

// 打开编辑对话框
function openEditDialog(token: ManagementToken) {
  if (!canManageTokens.value) return
  editingToken.value = token
  formData.name = token.name
  formData.description = token.description || ''
  formData.allowedIpsText = (token.allowed_ips && token.allowed_ips.length > 0)
    ? token.allowed_ips.join(', ')
    : ''
  formData.expiresAt = token.expires_at
    ? toLocalDatetimeString(new Date(token.expires_at))
    : ''
  const mode = token.permission_mode === 'read_only' || token.permission_mode === 'custom'
    ? token.permission_mode
    : 'full'
  permissionMode.value = mode
  selectedPermissions.value = token.permissions?.length
    ? [...token.permissions]
    : (mode === 'read_only' ? [...readOnlyPermissionKeys.value] : [...allPermissionKeys.value])
  void loadPermissionCatalog()
  showCreateDialog.value = true
}

// 关闭对话框
function closeDialog() {
  showCreateDialog.value = false
  resetForm()
}

function resetForm() {
  editingToken.value = null
  formData.name = ''
  formData.description = ''
  formData.allowedIpsText = ''
  formData.expiresAt = ''
  permissionMode.value = 'full'
  selectedPermissions.value = [...allPermissionKeys.value]
}

function setPermissionMode(mode: 'full' | 'read_only' | 'custom') {
  permissionMode.value = mode
  if (mode === 'full') {
    selectedPermissions.value = [...allPermissionKeys.value]
  } else if (mode === 'read_only') {
    selectedPermissions.value = [...readOnlyPermissionKeys.value]
  } else if (selectedPermissions.value.length === 0) {
    selectedPermissions.value = [...readOnlyPermissionKeys.value]
  }
}

function setCustomPermissions(mode: 'none' | 'read_only' | 'full') {
  if (mode === 'none') {
    selectedPermissions.value = []
  } else if (mode === 'read_only') {
    selectedPermissions.value = [...readOnlyPermissionKeys.value]
  } else {
    selectedPermissions.value = [...allPermissionKeys.value]
  }
}

function togglePermission(key: string, checked: boolean) {
  const next = new Set(selectedPermissions.value)
  if (checked) {
    next.add(key)
  } else {
    next.delete(key)
  }
  selectedPermissions.value = Array.from(next).sort()
}

function isPermissionGroupDenied(group: { items: ManagementTokenPermissionCatalogItem[] }): boolean {
  return group.items.every(item => !selectedPermissions.value.includes(item.key))
}

function togglePermissionGroupDenied(
  group: { items: ManagementTokenPermissionCatalogItem[] },
  checked: boolean
) {
  const next = new Set(selectedPermissions.value)
  for (const item of group.items) {
    next.delete(item.key)
  }
  if (!checked) {
    const readPermission = group.items.find(item => item.access === 'read')
    if (readPermission) {
      next.add(readPermission.key)
    }
  }
  selectedPermissions.value = Array.from(next).sort()
}

async function resolveFormPermissions(): Promise<string[]> {
  if (permissionCatalog.value.length === 0) {
    await loadPermissionCatalog()
  }
  if (allPermissionKeys.value.length === 0) {
    throw new Error(t('ui.availablePermissionsCouldNotBeLoaded'))
  }
  if (permissionMode.value === 'full') {
    return [...allPermissionKeys.value]
  }
  if (permissionMode.value === 'read_only') {
    return [...readOnlyPermissionKeys.value]
  }
  return [...selectedPermissions.value]
}

function permissionModeText(mode?: ManagementToken['permission_mode']): string {
  switch (mode) {
    case 'legacy_full':
      return t('ui.legacyFullPermissions')
    case 'full':
      return t('ui.fullPermissions')
    case 'read_only':
      return t('ui.readOnly')
    case 'custom':
      return t('ui.custom4eafa9e')
    default:
      return t('ui.notConfigured')
  }
}

// 保存 Token
async function saveToken() {
  if (!isFormValid.value) return

  saving.value = true
  try {
    const allowedIps = formData.allowedIpsText
      .split(',')
      .map(ip => ip.trim())
      .filter(ip => ip)

    // 将本地时间转换为 UTC ISO 字符串
    const expiresAtUtc = formData.expiresAt
      ? new Date(formData.expiresAt).toISOString()
      : null
    const permissions = await resolveFormPermissions()

    if (editingToken.value) {
      // 更新
      const tokenId = editingToken.value.id
      const result = isAdminPage.value
        ? await adminManagementTokenApi.updateToken(tokenId, {
          name: formData.name,
          description: formData.description.trim() || null,
          allowed_ips: allowedIps.length > 0 ? allowedIps : null,
          permissions,
          expires_at: expiresAtUtc
        })
        : await managementTokenApi.updateToken(tokenId, {
          name: formData.name,
          description: formData.description.trim() || null,
          allowed_ips: allowedIps.length > 0 ? allowedIps : null,
          permissions,
          expires_at: expiresAtUtc
        })
      // 局部更新：直接替换列表中对应的记录
      const index = tokens.value.findIndex(t => t.id === tokenId)
      if (index !== -1) {
        tokens.value[index] = result.data
      }
      success(t('ui.tokenUpdated'))
    } else {
      // 创建
      const payload = {
        name: formData.name,
        description: formData.description || undefined,
        allowed_ips: allowedIps.length > 0 ? allowedIps : undefined,
        permissions,
        expires_at: expiresAtUtc
      }
      const result = isAdminPage.value
        ? await adminManagementTokenApi.createToken(payload)
        : await managementTokenApi.createToken(payload)
      newTokenValue.value = result.token
      isRegenerating.value = false
      showTokenDialog.value = true
      success(t('ui.tokenCreated'))
      await loadTokens()
    }

    closeDialog()
  } catch (err: unknown) {
    log.error('保存 Token 失败:', err)
    showError(parseApiError(err, t('ui.failedToSave')))
  } finally {
    saving.value = false
  }
}

// 切换状态
async function toggleToken(token: ManagementToken) {
  if (!canManageTokens.value) return
  try {
    const result = isAdminPage.value
      ? await adminManagementTokenApi.toggleToken(token.id)
      : await managementTokenApi.toggleToken(token.id)

    const index = tokens.value.findIndex(t => t.id === token.id)
    if (index !== -1) {
      tokens.value[index] = result.data
    }
    success(result.data.is_active ? t('ui.tokenEnabled') : t('ui.tokenDisabled'))
  } catch (err: unknown) {
    log.error('切换状态失败:', err)
    showError(t('ui.operationFailed'))
  }
}

// 删除
function confirmDelete(token: ManagementToken) {
  if (!canManageTokens.value) return
  tokenToDelete.value = token
  showDeleteDialog.value = true
}

async function deleteToken() {
  if (!tokenToDelete.value) return

  deleting.value = true
  try {
    if (isAdminPage.value) {
      await adminManagementTokenApi.deleteToken(tokenToDelete.value.id)
    } else {
      await managementTokenApi.deleteToken(tokenToDelete.value.id)
    }

    showDeleteDialog.value = false
    success(t('ui.tokenDeleted'))
    await loadTokens()
  } catch (err: unknown) {
    log.error('删除 Token 失败:', err)
    showError(t('ui.deletionFailed'))
  } finally {
    deleting.value = false
    tokenToDelete.value = null
  }
}

// 重新生成
function confirmRegenerate(token: ManagementToken) {
  if (!canManageTokens.value) return
  tokenToRegenerate.value = token
  showRegenerateDialog.value = true
}

async function regenerateToken() {
  if (!tokenToRegenerate.value) return

  regenerating.value = true
  try {
    const result = isAdminPage.value
      ? await adminManagementTokenApi.regenerateToken(tokenToRegenerate.value.id)
      : await managementTokenApi.regenerateToken(tokenToRegenerate.value.id)
    newTokenValue.value = result.token
    isRegenerating.value = true
    showRegenerateDialog.value = false
    showTokenDialog.value = true
    await loadTokens()
    success(t('ui.tokenRegenerated'))
  } catch (err: unknown) {
    log.error('重新生成失败:', err)
    showError(t('ui.failedToRegenerateToken'))
  } finally {
    regenerating.value = false
    tokenToRegenerate.value = null
  }
}

function toLocalDatetimeString(date: Date): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  return `${year}-${month}-${day}T${hours}:${minutes}`
}

</script>
