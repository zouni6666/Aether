<template>
  <div class="space-y-6 pb-8">
    <TableCard :title="t('ui.standaloneBalanceApiKeys')">
      <template #actions>
        <!-- 搜索框 -->
        <div class="relative">
          <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground z-10 pointer-events-none" />
          <Input
            v-model="searchQuery"
            type="text"
            :placeholder="t('ui.searchc304ad3')"
            class="h-8 w-28 sm:w-40 pl-8 pr-2 text-xs"
          />
        </div>

        <!-- 分隔线 -->
        <div class="hidden sm:block h-4 w-px bg-border" />

        <!-- 状态筛选 -->
        <div class="xl:hidden">
          <Select
            v-model="filterStatus"
          >
            <SelectTrigger class="w-20 sm:w-28 h-8 text-xs border-border/60">
              <SelectValue :placeholder="t('ui.allStatuses')" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="status in statusFilters"
                :key="status.value"
                :value="status.value"
              >
                {{ status.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- 余额类型筛选 -->
        <div class="xl:hidden">
          <Select
            v-model="filterBalance"
          >
            <SelectTrigger class="w-20 sm:w-28 h-8 text-xs border-border/60">
              <SelectValue :placeholder="t('ui.allTypes')" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="balance in balanceFilters"
                :key="balance.value"
                :value="balance.value"
              >
                {{ balance.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- 分隔线 -->
        <div class="hidden sm:block h-4 w-px bg-border" />

        <!-- 创建独立 Key 按钮 -->
        <Button
          variant="ghost"
          size="icon"
          class="h-8 w-8"
          :title="t('ui.createStandaloneKey')"
          @click="openCreateDialog"
        >
          <Plus class="w-3.5 h-3.5" />
        </Button>

        <!-- 刷新按钮 -->
        <RefreshButton
          :loading="loading"
          @click="refreshApiKeys"
        />
      </template>

      <!-- 加载状态 -->
      <LoadingState
        v-if="loading"
        :message="t('common.loading')"
        size="lg"
      />

      <div v-else>
        <div class="hidden xl:block overflow-x-auto">
          <Table>
            <TableHeader>
              <TableRow class="border-b border-border/60 hover:bg-transparent">
                <TableHead class="w-[200px] h-12 font-semibold">
                  {{ t('ui.keyDetails') }}
                </TableHead>
                <SortableTableHead
                  class="w-[240px] h-12 font-semibold"
                  column-key="balance"
                  :sortable="false"
                  :filter-active="filterBalance !== 'all'"
                  :filter-title="t('ui.filterBalanceType')"
                  filter-content-class="w-40 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
                >
                  {{ t('ui.wallet') }} <template #filter="{ close }">
                    <TableFilterMenu
                      v-model="filterBalance"
                      :options="balanceFilters"
                      @select="close"
                    />
                  </template>
                </SortableTableHead>
                <TableHead class="w-[190px] h-12 font-semibold">
                  {{ t('ui.usageLimits') }}
                </TableHead>
                <TableHead class="w-[140px] h-12 font-semibold">
                  {{ t('ui.createdAt') }}
                </TableHead>
                <TableHead class="w-[110px] h-12 font-semibold">
                  {{ t('ui.validity') }}
                </TableHead>
                <TableHead class="w-[140px] h-12 font-semibold">
                  {{ t('ui.lastUsed39a9046') }}
                </TableHead>
                <SortableTableHead
                  class="w-[100px] h-12 font-semibold"
                  column-key="status"
                  :sortable="false"
                  :filter-active="filterStatus !== 'all'"
                  :filter-title="t('ui.filterStatus')"
                  filter-content-class="w-40 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
                >
                  {{ t('ui.status6320b4a') }} <template #filter="{ close }">
                    <TableFilterMenu
                      v-model="filterStatus"
                      :options="statusFilters"
                      @select="close"
                    />
                  </template>
                </SortableTableHead>
                <TableHead class="w-[130px] h-12 font-semibold text-center">
                  {{ t('ui.actionsed31fbb') }}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow v-if="filteredApiKeys.length === 0">
                <TableCell
                  colspan="8"
                  class="h-64 text-center"
                >
                  <EmptyState
                    :type="hasActiveFilters ? 'filter' : 'empty'"
                    :icon="hasActiveFilters ? undefined : Key"
                    :title="hasActiveFilters ? t('ui.noMatchingKeys7d94d1b') : t('ui.noStandaloneBalanceKeys')"
                    :description="hasActiveFilters ? t('ui.tryAdjustingFilters') : t('ui.createAStandaloneBalanceKeyToGetStarted')"
                    :action-text="hasActiveFilters ? '清除筛选' : undefined"
                    action-variant="outline"
                    action-size="sm"
                    size="sm"
                    @action="clearFilters"
                  />
                </TableCell>
              </TableRow>
              <TableRow
                v-for="apiKey in filteredApiKeys"
                :key="apiKey.id"
                class="border-b border-border/40 hover:bg-muted/30 transition-colors"
              >
                <TableCell class="py-4">
                  <div class="space-y-1">
                    <div
                      class="text-sm font-medium text-foreground truncate"
                      :title="apiKey.name || t('ui.untitledKey')"
                    >
                      {{ apiKey.name || t('ui.untitledKey') }}
                    </div>
                    <div class="flex items-center gap-1.5">
                      <code class="text-xs font-mono text-muted-foreground">
                        {{ apiKey.key_display || '****' }}
                      </code>
                      <Button
                        variant="ghost"
                        size="icon"
                        class="h-6 w-6"
                        :title="t('ui.copyFullKey')"
                        @click="copyKeyPrefix(apiKey)"
                      >
                        <Copy class="h-3 w-3" />
                      </Button>
                    </div>
                  </div>
                </TableCell>
                <TableCell class="py-4">
                  <div class="space-y-1.5">
                    <div class="flex items-center gap-1 text-[11px] text-muted-foreground">
                      <span>{{ t('ui.balance17f82df') }}</span>
                      <Badge
                        v-if="isApiKeyUnlimited(apiKey)"
                        variant="secondary"
                        class="h-5 px-1.5 py-0 text-[10px] font-medium"
                      >
                        {{ t('ui.unlimitedQuota') }}
                      </Badge>
                      <span
                        v-else
                        class="text-sm font-medium tabular-nums"
                        :class="isNegativeWalletAmount(getApiKeyWalletTotalBalance(apiKey)) ? 'text-rose-600' : 'text-foreground'"
                      >
                        {{ formatWalletAmount(getApiKeyWalletTotalBalance(apiKey), '-') }}
                      </span>
                    </div>
                    <div class="flex items-center gap-2 text-[11px] text-muted-foreground flex-wrap">
                      <span> {{ t('ui.consumed7e0201d') }} <span class="font-medium tabular-nums text-foreground">${{ getApiKeyWalletConsumed(apiKey).toFixed(2) }}</span>
                      </span>
                    </div>
                  </div>
                </TableCell>
                <TableCell class="py-4">
                  <div class="space-y-1 text-xs">
                    <div class="text-muted-foreground">
                      {{ t('ui.requests82c54b3') }} <span class="font-medium text-foreground">{{ (apiKey.total_requests || 0).toLocaleString() }}</span>
                    </div>
                    <div class="text-muted-foreground">
                      Tokens: <span class="font-medium text-foreground">{{ formatApiKeyTotalTokens(apiKey) }}</span>
                    </div>
                    <div class="flex items-center gap-1 text-muted-foreground">
                      <span>{{ t('ui.rateLimit') }}</span>
                      <Badge
                        v-if="isRateLimitInherited(apiKey.rate_limit) || isRateLimitUnlimited(apiKey.rate_limit)"
                        variant="secondary"
                        class="h-5 px-1.5 py-0 text-[10px] font-medium"
                      >
                        {{ formatRateLimitInheritable(apiKey.rate_limit) }}
                      </Badge>
                      <span
                        v-else
                        class="font-medium text-foreground"
                      >
                        {{ formatRateLimitInheritable(apiKey.rate_limit) }}
                      </span>
                    </div>
                    <div class="flex items-center gap-1 text-muted-foreground">
                      <span>{{ t('ui.concurrencye213a23') }}</span>
                      <Badge
                        v-if="isConcurrentLimitInherited(apiKey.concurrent_limit) || isConcurrentLimitUnlimited(apiKey.concurrent_limit)"
                        variant="secondary"
                        class="h-5 px-1.5 py-0 text-[10px] font-medium"
                      >
                        {{ formatConcurrentLimitInheritable(apiKey.concurrent_limit) }}
                      </Badge>
                      <span
                        v-else
                        class="font-medium text-foreground"
                      >
                        {{ formatConcurrentLimitInheritable(apiKey.concurrent_limit) }}
                      </span>
                    </div>
                  </div>
                </TableCell>
                <TableCell class="py-4">
                  <div class="text-xs">
                    <span class="text-foreground">{{ formatDateTime(apiKey.created_at) }}</span>
                  </div>
                </TableCell>
                <TableCell class="py-4">
                  <div class="text-xs">
                    <div
                      v-if="apiKey.expires_at"
                      class="space-y-1"
                    >
                      <div class="text-foreground">
                        {{ formatDateTime(apiKey.expires_at) }}
                      </div>
                      <div class="text-muted-foreground">
                        {{ getRelativeTime(apiKey.expires_at) }}
                      </div>
                    </div>
                    <div
                      v-else
                      class="text-muted-foreground"
                    >
                      {{ t('ui.neverExpires') }}
                    </div>
                  </div>
                </TableCell>
                <TableCell class="py-4">
                  <div class="text-xs">
                    <span
                      v-if="apiKey.last_used_at"
                      class="text-foreground"
                    >{{ formatDateTime(apiKey.last_used_at) }}</span>
                    <span
                      v-else
                      class="text-muted-foreground"
                    >{{ t('ui.noRecords') }}</span>
                  </div>
                </TableCell>
                <TableCell class="w-[100px] py-4">
                  <div class="flex flex-col items-start gap-1.5">
                    <Badge
                      :variant="apiKey.is_active ? 'success' : 'destructive'"
                      class="h-5 px-1.5 py-0 text-[10px] font-medium"
                    >
                      {{ apiKey.is_active ? t('ui.activee85e590') : t('ui.disable') }}
                    </Badge>
                    <Badge
                      v-if="getApiKeyWallet(apiKey.id)"
                      :variant="walletStatusBadge(getApiKeyWalletStatus(apiKey.id))"
                      class="h-5 px-1.5 py-0 text-[10px] font-medium"
                    >
                      {{ walletStatusLabel(getApiKeyWalletStatus(apiKey.id)) }}
                    </Badge>
                  </div>
                </TableCell>
                <TableCell class="py-4">
                  <div class="flex justify-center gap-1">
                    <Button
                      variant="ghost"
                      size="icon"
                      class="h-7 w-7"
                      :title="t('ui.oneClickInstallAndConfigureCli')"
                      @click="openInstallDialog(apiKey)"
                    >
                      <Terminal class="h-3.5 w-3.5" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      class="h-7 w-7"
                      :title="t('ui.edit0518365')"
                      @click="editApiKey(apiKey)"
                    >
                      <SquarePen class="h-3.5 w-3.5" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      class="h-7 w-7"
                      :title="t('ui.walletOperation')"
                      @click="openAddBalanceDialog(apiKey)"
                    >
                      <DollarSign class="h-3.5 w-3.5" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      class="h-7 w-7"
                      :title="apiKey.is_active ? t('ui.disable') : t('ui.enablef4f0ead')"
                      @click="toggleApiKey(apiKey)"
                    >
                      <Power class="h-3.5 w-3.5" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      class="h-7 w-7"
                      :title="t('ui.delete')"
                      @click="deleteApiKey(apiKey)"
                    >
                      <Trash2 class="h-3.5 w-3.5" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>

        <div class="xl:hidden bg-muted/[0.14] p-3 sm:p-4">
          <EmptyState
            v-if="filteredApiKeys.length === 0"
            :type="hasActiveFilters ? 'filter' : 'empty'"
            :icon="hasActiveFilters ? undefined : Key"
            :title="hasActiveFilters ? t('ui.noMatchingKeys7d94d1b') : t('ui.noStandaloneBalanceKeys')"
            :description="hasActiveFilters ? t('ui.tryAdjustingFilters') : t('ui.createAStandaloneBalanceKeyToGetStarted')"
            :action-text="hasActiveFilters ? '清除筛选' : undefined"
            action-variant="outline"
            action-size="sm"
            size="sm"
            @action="clearFilters"
          />

          <div
            v-else
            class="space-y-3.5"
          >
            <div
              v-for="apiKey in filteredApiKeys"
              :key="apiKey.id"
              class="rounded-lg border border-border/60 bg-card p-3"
            >
              <div class="space-y-3">
                <div class="flex items-start gap-3">
                  <div class="min-w-0 flex-1 space-y-2">
                    <div class="flex items-center gap-2">
                      <code class="inline-flex max-w-[190px] sm:max-w-[240px] truncate rounded-lg bg-muted px-3 py-1.5 text-[11px] font-mono font-semibold text-foreground/90">
                        {{ apiKey.key_display || '****' }}
                      </code>
                      <Button
                        variant="ghost"
                        size="icon"
                        class="h-7 w-7 flex-shrink-0 hover:bg-muted"
                        :title="t('ui.copyFullKey')"
                        @click="copyKeyPrefix(apiKey)"
                      >
                        <Copy class="h-3.5 w-3.5" />
                      </Button>
                    </div>
                    <div
                      class="truncate text-sm font-medium text-foreground"
                      :class="{ 'text-muted-foreground': !apiKey.name }"
                      :title="apiKey.name || t('ui.untitledKey')"
                    >
                      {{ apiKey.name || t('ui.untitledKey') }}
                    </div>
                  </div>
                </div>

                <div class="flex flex-wrap items-center gap-1.5">
                  <Badge
                    :variant="apiKey.is_active ? 'success' : 'destructive'"
                    class="h-5 px-1.5 py-0 text-[10px] font-medium"
                  >
                    {{ apiKey.is_active ? t('ui.activee85e590') : t('ui.disable') }}
                  </Badge>
                  <Badge
                    v-if="getApiKeyWallet(apiKey.id)"
                    :variant="walletStatusBadge(getApiKeyWalletStatus(apiKey.id))"
                    class="h-5 px-1.5 py-0 text-[10px] font-medium"
                  >
                    {{ walletStatusLabel(getApiKeyWalletStatus(apiKey.id)) }}
                  </Badge>
                  <Badge
                    variant="secondary"
                    class="h-5 px-1.5 py-0 text-[10px] font-medium"
                  >
                    {{ formatRateLimitInheritable(apiKey.rate_limit) }}
                  </Badge>
                  <Badge
                    variant="secondary"
                    class="h-5 px-1.5 py-0 text-[10px] font-medium"
                  >
                    {{ formatConcurrentLimitInheritable(apiKey.concurrent_limit) }}
                  </Badge>
                  <Badge
                    v-if="apiKey.auto_delete_on_expiry"
                    variant="secondary"
                    class="h-5 px-1.5 py-0 text-[10px] font-medium"
                  >
                    {{ t('ui.deleteAutomaticallyOnExpiration') }}
                  </Badge>
                </div>

                <div class="rounded-lg border border-border/60 bg-muted/30 p-3">
                  <div class="flex items-start justify-between gap-3">
                    <div class="space-y-1">
                      <p class="text-[11px] text-muted-foreground">
                        {{ t('ui.balance17f82df') }}
                      </p>
                      <Badge
                        v-if="isApiKeyUnlimited(apiKey)"
                        variant="secondary"
                        class="h-5 px-1.5 py-0 text-[10px] font-medium"
                      >
                        {{ t('ui.unlimitedQuota') }}
                      </Badge>
                      <p
                        v-else
                        class="text-sm font-medium tabular-nums leading-none"
                        :class="isNegativeWalletAmount(getApiKeyWalletTotalBalance(apiKey)) ? 'text-rose-600' : 'text-foreground'"
                      >
                        {{ formatWalletAmount(getApiKeyWalletTotalBalance(apiKey), '-') }}
                      </p>
                    </div>
                    <div class="text-right">
                      <p class="text-[11px] text-muted-foreground">
                        {{ t('ui.consumed7e0201d') }}
                      </p>
                      <p class="text-sm font-medium tabular-nums text-foreground">
                        ${{ getApiKeyWalletConsumed(apiKey).toFixed(2) }}
                      </p>
                    </div>
                  </div>
                </div>

                <div class="grid grid-cols-2 gap-2.5 text-xs">
                  <div class="rounded-lg border border-border/50 bg-background/70 p-2.5">
                    <div class="mb-1 text-muted-foreground">
                      {{ t('ui.requestCount') }}
                    </div>
                    <div class="font-medium text-foreground">
                      {{ (apiKey.total_requests || 0).toLocaleString() }}
                    </div>
                  </div>
                  <div class="rounded-lg border border-border/50 bg-background/70 p-2.5">
                    <div class="mb-1 text-muted-foreground">
                      Tokens
                    </div>
                    <div class="font-medium text-foreground">
                      {{ formatApiKeyTotalTokens(apiKey) }}
                    </div>
                  </div>
                  <div class="col-span-2 rounded-lg border border-border/50 bg-background/70 p-2.5">
                    <div class="mb-1 text-muted-foreground">
                      {{ t('ui.validity') }}
                    </div>
                    <div class="font-medium text-foreground">
                      {{ apiKey.expires_at ? formatDateTime(apiKey.expires_at) : t('ui.neverExpires') }}
                    </div>
                    <div
                      v-if="apiKey.expires_at"
                      class="text-[11px] text-muted-foreground"
                    >
                      {{ getRelativeTime(apiKey.expires_at) }}
                    </div>
                  </div>
                </div>

                <div class="rounded-lg bg-muted/35 p-2.5 text-[11px] text-muted-foreground">
                  <div class="flex items-center justify-between gap-2">
                    <span>{{ t('ui.create') }}</span>
                    <span class="font-medium text-foreground">{{ formatDateTime(apiKey.created_at) }}</span>
                  </div>
                  <div class="mt-1 flex items-center justify-between gap-2">
                    <span>{{ t('ui.lastUsed39a9046') }}</span>
                    <span
                      v-if="apiKey.last_used_at"
                      class="font-medium text-foreground"
                    >{{ formatDateTime(apiKey.last_used_at) }}</span>
                    <span v-else>{{ t('ui.noRecords') }}</span>
                  </div>
                  <div
                    v-if="apiKey.expires_at"
                    class="mt-1 flex items-center justify-between gap-2"
                  >
                    <span>{{ t('ui.afterExpiration') }}</span>
                    <span>{{ apiKey.auto_delete_on_expiry ? t('ui.deleteAutomatically') : t('ui.disableOnly') }}</span>
                  </div>
                </div>

                <div class="grid grid-cols-2 gap-2 pt-0.5">
                  <Button
                    variant="outline"
                    size="sm"
                    class="h-8 text-xs"
                    @click="openInstallDialog(apiKey)"
                  >
                    <Terminal class="mr-1.5 h-3.5 w-3.5" /> {{ t('ui.install') }}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    class="h-8 text-xs"
                    @click="editApiKey(apiKey)"
                  >
                    <SquarePen class="mr-1.5 h-3.5 w-3.5" /> {{ t('ui.edit0518365') }}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    class="h-8 text-xs"
                    @click="openAddBalanceDialog(apiKey)"
                  >
                    <DollarSign class="mr-1.5 h-3.5 w-3.5" /> {{ t('ui.wallet4cd1c82') }}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    class="h-8 text-xs"
                    @click="toggleApiKey(apiKey)"
                  >
                    <Power class="mr-1.5 h-3.5 w-3.5" />
                    {{ apiKey.is_active ? t('ui.disable') : t('ui.enablef4f0ead') }}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    class="col-span-2 h-8 border-rose-200 text-xs text-rose-600 hover:bg-rose-50 dark:border-rose-900/60 dark:hover:bg-rose-950/40"
                    @click="deleteApiKey(apiKey)"
                  >
                    <Trash2 class="mr-1.5 h-3.5 w-3.5" /> {{ t('ui.delete') }}
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <template #pagination>
        <Pagination
          v-if="!loading && apiKeys.length > 0"
          :current="currentPage"
          :total="total"
          :page-size="limit"
          :show-page-size-selector="false"
          @update:current="handlePageChange"
        />
      </template>
    </TableCard>

    <!-- 创建/编辑独立Key对话框 -->
    <StandaloneKeyFormDialog
      ref="keyFormDialogRef"
      :open="showKeyFormDialog"
      :api-key="editingKeyData"
      @close="closeKeyFormDialog"
      @submit="handleKeyFormSubmit"
    />

    <!-- 新 Key 显示对话框 -->
    <Dialog
      v-model="showNewKeyDialog"
      size="lg"
    >
      <template #header>
        <div class="border-b border-border px-6 py-4">
          <div class="flex items-center gap-3">
            <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-emerald-100 dark:bg-emerald-900/30 flex-shrink-0">
              <CheckCircle class="h-5 w-5 text-emerald-600 dark:text-emerald-400" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-lg font-semibold text-foreground leading-tight">
                {{ t('ui.createdSuccessfully') }}
              </h3>
              <p class="text-xs text-muted-foreground">
                {{ t('ui.keepItSafeAndDoNotShareIt') }}
              </p>
            </div>
          </div>
        </div>
      </template>

      <div class="space-y-4">
        <div class="space-y-2">
          <Label class="text-sm font-medium">API Key</Label>
          <div class="flex items-center gap-2">
            <Input
              ref="keyInput"
              type="text"
              :value="newKeyValue"
              readonly
              class="flex-1 font-mono text-sm bg-muted/50 h-11"
              @click="selectKey"
            />
            <Button
              class="h-11"
              @click="copyKey"
            >
              {{ t('common.copy') }}
            </Button>
          </div>
        </div>
      </div>

      <template #footer>
        <Button
          class="h-10 px-5"
          @click="closeNewKeyDialog"
        >
          {{ t('ui.confirmfac2a67') }}
        </Button>
      </template>
    </Dialog>

    <!-- 一键安装并配置 CLI 对话框 -->
    <ApiKeyInstallDialog
      v-model:open="showInstallDialog"
      :api-key="selectedInstallApiKey"
      :create-session="adminApi.createApiKeyInstallSession"
    />

    <WalletOpsDrawer
      :open="showWalletActionDrawer"
      :wallet="walletActionTarget?.wallet || null"
      :owner-name="walletActionTarget?.apiKey.name || walletActionTarget?.apiKey.key_display || '未命名 Key'"
      :owner-subtitle="walletActionTarget?.apiKey.key_display || walletActionTarget?.apiKey.username || ''"
      :context-label="t('ui.standaloneKeyWallet')"
      accent="blue"
      :show-refunds="false"
      @close="closeWalletActionDrawer"
      @changed="handleWalletDrawerChanged"
    />
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { ref, computed, onMounted } from 'vue'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { useClipboard } from '@/composables/useClipboard'
import { adminApi, type AdminApiKey, type CreateStandaloneApiKeyRequest } from '@/api/admin'
import type { AdminWallet } from '@/api/admin-wallets'
import { walletStatusBadge, walletStatusLabel } from '@/features/wallet/utils/walletDisplay'
import WalletOpsDrawer from '@/features/wallet/components/WalletOpsDrawer.vue'
import { EmptyState, LoadingState } from '@/components/common'

import {
  Dialog,
  TableCard,
  Button,
  Badge,
  Input,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  SortableTableHead,
  TableFilterMenu,
  TableCell,
  Pagination,
  RefreshButton,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  Label
} from '@/components/ui'

import {
  Plus,
  Key,
  Trash2,
  Power,
  DollarSign,
  Copy,
  CheckCircle,
  SquarePen,
  Search,
  Terminal
} from 'lucide-vue-next'

import StandaloneKeyFormDialog, { type StandaloneKeyFormData } from '@/features/api-keys/components/StandaloneKeyFormDialog.vue'
import ApiKeyInstallDialog from '@/features/api-keys/components/ApiKeyInstallDialog.vue'
import { parseApiError } from '@/utils/errorParser'
import { formatTokens, formatRateLimitInheritable, isRateLimitInherited, isRateLimitUnlimited, formatDateTime } from '@/utils/format'
import { log } from '@/utils/logger'

const { success, error } = useToast()
const { confirmDanger } = useConfirm()
const { copyToClipboard } = useClipboard()

const apiKeys = ref<AdminApiKey[]>([])
const apiKeyWalletMap = ref<Record<string, AdminWallet>>({})
const loading = ref(false)
const total = ref(0)
const currentPage = ref(1)
const limit = ref(100)
const showNewKeyDialog = ref(false)
const showInstallDialog = ref(false)
const newKeyValue = ref('')
const keyInput = ref<HTMLInputElement>()
const selectedInstallApiKey = ref<AdminApiKey | null>(null)

// 统一的表单对话框状态
const showKeyFormDialog = ref(false)
const editingKeyData = ref<StandaloneKeyFormData | null>(null)
const keyFormDialogRef = ref<InstanceType<typeof StandaloneKeyFormDialog>>()

const EXPIRY_SOON_DAYS = 7

// 筛选相关
const searchQuery = ref('')
const filterStatus = ref<'all' | 'active' | 'inactive'>('all')
const filterBalance = ref<'all' | 'limited' | 'unlimited'>('all')

const statusFilters = [
  { value: 'all' as const, get label() { return t('ui.allStatuses') } },
  { value: 'active' as const, get label() { return t('ui.activee85e590') } },
  { value: 'inactive' as const, get label() { return t('ui.disable') } }
]

const balanceFilters = [
  { value: 'all' as const, get label() { return t('ui.allTypes') } },
  { value: 'limited' as const, get label() { return t('ui.quota') } },
  { value: 'unlimited' as const, get label() { return t('ui.unlimited8651f50') } }
]

const hasActiveFilters = computed(() => {
  return searchQuery.value !== '' || filterStatus.value !== 'all' || filterBalance.value !== 'all'
})

function clearFilters() {
  searchQuery.value = ''
  filterStatus.value = 'all'
  filterBalance.value = 'all'
}

const skip = computed(() => (currentPage.value - 1) * limit.value)

const activeKeyCount = computed(() => apiKeys.value.filter(key => key.is_active).length)
const _inactiveKeyCount = computed(() => Math.max(0, apiKeys.value.length - activeKeyCount.value))
const limitedKeyCount = computed(() => apiKeys.value.filter(isBalanceLimited).length)
const _unlimitedKeyCount = computed(() => Math.max(0, apiKeys.value.length - limitedKeyCount.value))
const _expiringSoonCount = computed(() => apiKeys.value.filter(key => isExpiringSoon(key)).length)

// 筛选后的 API Keys
const filteredApiKeys = computed(() => {
  let result = apiKeys.value

  // 搜索筛选（支持空格分隔的多关键词 AND 搜索）
  if (searchQuery.value) {
    const keywords = searchQuery.value.toLowerCase().split(/\s+/).filter(k => k.length > 0)
    result = result.filter(key => {
      const searchableText = `${key.name || ''} ${key.key_display || ''} ${key.username || ''} ${key.user_email || ''}`.toLowerCase()
      return keywords.every(keyword => searchableText.includes(keyword))
    })
  }

  // 状态筛选
  if (filterStatus.value === 'active') {
    result = result.filter(key => key.is_active)
  } else if (filterStatus.value === 'inactive') {
    result = result.filter(key => !key.is_active)
  }

  // 余额类型筛选
  if (filterBalance.value === 'limited') {
    result = result.filter(isBalanceLimited)
  } else if (filterBalance.value === 'unlimited') {
    result = result.filter(key => !isBalanceLimited(key))
  }

  return result
})

const showWalletActionDrawer = ref(false)
const walletActionTarget = ref<{ apiKey: AdminApiKey; wallet: AdminWallet } | null>(null)

onMounted(async () => {
  await refreshApiKeys()
})

function buildAdminWalletFromApiKey(apiKey: AdminApiKey): AdminWallet | null {
  if (!apiKey.wallet?.id) {
    return null
  }

  return {
    ...apiKey.wallet,
    id: apiKey.wallet.id,
    user_id: null,
    api_key_id: apiKey.id,
    owner_type: 'api_key',
    owner_name: apiKey.name || apiKey.key_display || null,
    created_at: apiKey.created_at || apiKey.wallet.updated_at || '',
    updated_at: apiKey.wallet.updated_at || apiKey.created_at || '',
  }
}

function buildApiKeyWalletMap(items: AdminApiKey[]): Record<string, AdminWallet> {
  return items.reduce<Record<string, AdminWallet>>((acc, apiKey) => {
    const wallet = buildAdminWalletFromApiKey(apiKey)
    if (wallet) {
      acc[apiKey.id] = wallet
    }
    return acc
  }, {})
}

async function refreshApiKeys() {
  loading.value = true
  try {
    const response = await adminApi.getAllApiKeys({
      skip: skip.value,
      limit: limit.value
    })
    const standaloneKeys = response.api_keys.filter((key) => key.is_standalone === true)
    if (standaloneKeys.length !== response.api_keys.length) {
      log.warn('独立 Key 页面收到了非 standalone 记录，已在前端过滤', {
        received: response.api_keys.length,
        kept: standaloneKeys.length
      })
    }
    apiKeys.value = standaloneKeys
    total.value = response.total
    apiKeyWalletMap.value = buildApiKeyWalletMap(standaloneKeys)
  } catch (err: unknown) {
    log.error('加载独立Keys失败:', err)
    error(parseApiError(err, t('ui.failedToLoadStandaloneKeys')))
  } finally {
    loading.value = false
  }
}

function handlePageChange(page: number) {
  currentPage.value = page
  refreshApiKeys()
}

function openInstallDialog(apiKey: AdminApiKey) {
  selectedInstallApiKey.value = apiKey
  showInstallDialog.value = true
}

async function toggleApiKey(apiKey: AdminApiKey) {
  try {
    const response = await adminApi.toggleApiKey(apiKey.id)
    const index = apiKeys.value.findIndex(k => k.id === apiKey.id)
    if (index !== -1) {
      apiKeys.value[index].is_active = response.is_active
    }
    success(response.message)
  } catch (err: unknown) {
    log.error('切换密钥状态失败:', err)
    error(parseApiError(err, t('ui.operationFailed')))
  }
}

async function deleteApiKey(apiKey: AdminApiKey) {
  const confirmed = await confirmDanger(
    `确定要删除这个独立余额 Key 吗？\n\n${apiKey.name || apiKey.key_display || '****'}\n\n此操作无法撤销。`,
    t('ui.deleteStandaloneKey')
  )

  if (!confirmed) return

  try {
    const response = await adminApi.deleteApiKey(apiKey.id)
    apiKeys.value = apiKeys.value.filter(k => k.id !== apiKey.id)
    total.value = total.value - 1
    delete apiKeyWalletMap.value[apiKey.id]
    success(response.message)
  } catch (err: unknown) {
    log.error('删除密钥失败:', err)
    error(parseApiError(err, t('ui.deletionFailed')))
  }
}

function formatDateForInput(dateString: string): string | undefined {
  const date = new Date(dateString)
  if (Number.isNaN(date.getTime())) {
    return undefined
  }
  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')
  return `${year}-${month}-${day}`
}

function parseDateInput(dateString: string): Date | null {
  const [year, month, day] = dateString.split('-').map(part => Number.parseInt(part, 10))
  if (!year || !month || !day) {
    return null
  }
  const date = new Date(year, month - 1, day)
  return Number.isNaN(date.getTime()) ? null : date
}

function serializeExpiryDate(dateString?: string): string | null {
  if (!dateString) {
    return null
  }
  const date = parseDateInput(dateString)
  if (!date) {
    return null
  }
  date.setHours(23, 59, 59, 999)
  return date.toISOString()
}

function editApiKey(apiKey: AdminApiKey) {
  // 解析过期日期为 YYYY-MM-DD 格式
  // 保留原始日期，不做时间过滤（避免编辑当天过期的 Key 时意外清空）
  let expiresAt: string | undefined = undefined

  if (apiKey.expires_at) {
    expiresAt = formatDateForInput(apiKey.expires_at)
  }

  editingKeyData.value = {
    id: apiKey.id,
    name: apiKey.name || '',
    initial_balance_usd: isApiKeyUnlimited(apiKey) ? undefined : (getApiKeyWalletTotalBalance(apiKey) ?? undefined),
    current_balance_usd: isApiKeyUnlimited(apiKey) ? null : getApiKeyWalletTotalBalance(apiKey),
    unlimited_balance: isApiKeyUnlimited(apiKey),
    expires_at: expiresAt,
    rate_limit: apiKey.rate_limit ?? undefined,
    concurrent_limit: apiKey.concurrent_limit ?? undefined,
    auto_delete_on_expiry: apiKey.auto_delete_on_expiry || false,
    allowed_providers: apiKey.allowed_providers == null ? null : [...apiKey.allowed_providers],
    allowed_api_formats: apiKey.allowed_api_formats == null ? null : [...apiKey.allowed_api_formats],
    allowed_models: apiKey.allowed_models == null ? null : [...apiKey.allowed_models],
    feature_settings: apiKey.feature_settings ?? null
  }

  showKeyFormDialog.value = true
}

function getApiKeyWallet(apiKeyId: string): AdminWallet | null {
  return apiKeyWalletMap.value[apiKeyId] || null
}

function isApiKeyUnlimited(apiKey: AdminApiKey): boolean {
  const wallet = getApiKeyWallet(apiKey.id)
  return wallet?.limit_mode === 'unlimited' || wallet?.unlimited === true
}

function getApiKeyWalletTotalBalance(apiKey: AdminApiKey): number | null {
  if (isApiKeyUnlimited(apiKey)) {
    return null
  }
  const wallet = getApiKeyWallet(apiKey.id)
  return wallet ? wallet.balance : 0
}

function getApiKeyWalletConsumed(apiKey: AdminApiKey): number {
  return getApiKeyWallet(apiKey.id)?.total_consumed ?? (apiKey.total_cost_usd || 0)
}

function getApiKeyWalletStatus(apiKeyId: string): string | null {
  return getApiKeyWallet(apiKeyId)?.status ?? null
}

function formatApiKeyTotalTokens(apiKey: AdminApiKey): string {
  if (apiKey.total_tokens == null) {
    return t('ui.notMeasured')
  }
  return formatTokens(apiKey.total_tokens)
}

function formatConcurrentLimitInheritable(concurrentLimit?: number | null): string {
  if (concurrentLimit == null) return t('ui.unlimitedConcurrency')
  if (concurrentLimit === 0) return t('ui.unlimitedConcurrency')
  return `${concurrentLimit} 并发`
}

function isConcurrentLimitInherited(concurrentLimit?: number | null): boolean {
  return concurrentLimit == null
}

function isConcurrentLimitUnlimited(concurrentLimit?: number | null): boolean {
  return concurrentLimit === 0
}

function formatWalletAmount(value: number | null, nullLabel = '无限制'): string {
  if (value == null) {
    return nullLabel
  }
  return `$${value.toFixed(2)}`
}

function isNegativeWalletAmount(value: number | null): boolean {
  return typeof value === 'number' && value < 0
}

function openAddBalanceDialog(apiKey: AdminApiKey) {
  const wallet = getApiKeyWallet(apiKey.id)
  if (!wallet) {
    error(t('ui.thisKeyWalletHasNotBeenInitializedBalanceOperationsAre'))
    return
  }

  walletActionTarget.value = {
    apiKey,
    wallet
  }
  showWalletActionDrawer.value = true
}

function closeWalletActionDrawer() {
  showWalletActionDrawer.value = false
}

async function handleWalletDrawerChanged() {
  await refreshApiKeys()
  if (!walletActionTarget.value) {
    return
  }

  const latestKey = apiKeys.value.find((item) => item.id === walletActionTarget.value?.apiKey.id)
  const latestWallet = getApiKeyWallet(walletActionTarget.value.apiKey.id)
  if (latestKey) {
    walletActionTarget.value.apiKey = latestKey
  }
  if (latestWallet) {
    walletActionTarget.value.wallet = latestWallet
  }
}

function selectKey() {
  keyInput.value?.select()
}

async function copyKey() {
  await copyToClipboard(newKeyValue.value)
}

async function copyKeyPrefix(apiKey: AdminApiKey) {
  try {
    // 调用后端 API 获取完整密钥
    const response = await adminApi.getFullApiKey(apiKey.id)
    const copied = await copyToClipboard(response.key, false)
    if (copied) {
      success(t('ui.fullKeyCopiedToClipboard'))
    } else {
      error(t('ui.copyFailedPleaseCopyManually'))
    }
  } catch (err) {
    log.error('复制密钥失败:', err)
    error(t('ui.copyFailedPleaseTryAgain'))
  }
}

function closeNewKeyDialog() {
  showNewKeyDialog.value = false
  newKeyValue.value = ''
}

function isBalanceLimited(apiKey: AdminApiKey): boolean {
  return !isApiKeyUnlimited(apiKey)
}

function isExpiringSoon(apiKey: AdminApiKey): boolean {
  if (!apiKey.expires_at) {
    return false
  }

  const expiresAt = new Date(apiKey.expires_at).getTime()
  const now = Date.now()
  const diffDays = (expiresAt - now) / (1000 * 60 * 60 * 24)
  return diffDays > 0 && diffDays <= EXPIRY_SOON_DAYS
}

function getRelativeTime(dateString: string): string {
  const date = new Date(dateString)
  const now = new Date()
  const diff = date.getTime() - now.getTime()

  if (diff < 0) return t('ui.expired')

  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  const hours = Math.floor(diff / (1000 * 60 * 60))

  if (days > 0) return `${days}天后过期`
  if (hours > 0) return `${hours}小时后过期`
  return t('ui.aboutToExpire')
}

// ========== 统一表单对话框方法 ==========

// 打开创建对话框
function openCreateDialog() {
  editingKeyData.value = null
  showKeyFormDialog.value = true
}

// 关闭表单对话框
function closeKeyFormDialog() {
  showKeyFormDialog.value = false
  editingKeyData.value = null
}

// 统一处理表单提交
async function handleKeyFormSubmit(data: StandaloneKeyFormData) {
  // 验证过期日期（如果设置了，必须晚于今天）
  if (data.expires_at) {
    const selectedDate = parseDateInput(data.expires_at)
    if (!selectedDate) {
      error(t('ui.invalidExpirationDate'))
      return
    }
    selectedDate.setHours(0, 0, 0, 0)
    const today = new Date()
    today.setHours(0, 0, 0, 0)
    if (selectedDate <= today) {
      error(t('ui.expirationDateMustBeAfterToday'))
      return
    }
  }

  keyFormDialogRef.value?.setSaving(true)
  try {
    if (data.id) {
      // 更新
      const updateData: Partial<CreateStandaloneApiKeyRequest> = {
        name: data.name || undefined,
        unlimited_balance: Boolean(data.unlimited_balance),
        rate_limit: data.rate_limit ?? null,  // undefined = 跟随系统默认，显式传 null
        concurrent_limit: data.concurrent_limit ?? null,
        expires_at: serializeExpiryDate(data.expires_at),
        auto_delete_on_expiry: data.auto_delete_on_expiry,
        // 空数组表示清除限制（允许全部），后端会将空数组存为 NULL
        allowed_providers: data.allowed_providers,
        allowed_api_formats: data.allowed_api_formats,
        allowed_models: data.allowed_models,
        ip_rules: data.ip_rules,
        feature_settings: data.feature_settings ?? null
      }
      const { message: _, ...updated } = await adminApi.updateApiKey(data.id, updateData)
      // 局部更新：合并字段，避免覆盖丢失列表已有信息
      const index = apiKeys.value.findIndex(k => k.id === data.id)
      if (index !== -1) {
        apiKeys.value[index] = {
          ...apiKeys.value[index],
          ...updated,
        }
        apiKeyWalletMap.value = buildApiKeyWalletMap(apiKeys.value)
      }
      success(t('ui.apiKeyUpdated70523c1'))
    } else {
      // 创建
      const isUnlimited = Boolean(data.unlimited_balance)
      if (!isUnlimited && (!data.initial_balance_usd || data.initial_balance_usd <= 0)) {
        error(t('ui.initialBalanceMustBeGreaterThanZero'))
        return
      }
      const createData: CreateStandaloneApiKeyRequest = {
        name: data.name || undefined,
        initial_balance_usd: isUnlimited ? null : (data.initial_balance_usd as number),
        rate_limit: data.rate_limit ?? null,  // undefined = 跟随系统默认，显式传 null
        concurrent_limit: data.concurrent_limit ?? null,
        expires_at: serializeExpiryDate(data.expires_at),
        auto_delete_on_expiry: data.auto_delete_on_expiry,
        // 空数组表示不设置限制（允许全部），后端会将空数组存为 NULL
        allowed_providers: data.allowed_providers,
        allowed_api_formats: data.allowed_api_formats,
        allowed_models: data.allowed_models,
        ip_rules: data.ip_rules,
        feature_settings: data.feature_settings ?? null
      }
      const response = await adminApi.createStandaloneApiKey(createData)
      newKeyValue.value = response.key
      showNewKeyDialog.value = true
      success(t('ui.standaloneKeyCreated'))
      await refreshApiKeys()
    }
    closeKeyFormDialog()
  } catch (err: unknown) {
    log.error('保存独立Key失败:', err)
    error(parseApiError(err, t('ui.failedToSave')))
  } finally {
    keyFormDialogRef.value?.setSaving(false)
  }
}

</script>
