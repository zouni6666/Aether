<template>
  <div class="space-y-6 pb-8">
    <Card
      variant="default"
      class="overflow-hidden"
    >
      <PoolManagementHeader
        v-model:provider-id="selectedProviderIdProxy"
        v-model:status="statusFilter"
        v-model:search="searchQuery"
        :providers="poolProviders"
        :provider-select-disabled="providerSelectDisabled"
        :status-options="poolKeyStatusFilterOptions"
        :meta-text="poolHeaderMetaText"
        :pool-scheduling-label="poolSchedulingLabel"
        :show-adaptive-hot-pool-metrics-button="showAdaptiveHotPoolMetricsButton"
        :selected-count="selectedKeyCount"
        :is-all-filtered-selected="isAllFilteredPoolKeysSelected"
        :selection-disabled="keyPage.total === 0 || poolKeySelectionBusy"
        :batch-actions-disabled="selectedKeyCount === 0 || poolKeySelectionBusy"
        :refresh-loading="refreshCurrentPageLoading"
        :refresh-title="refreshButtonTitle"
        @view-provider="openProviderDrawer"
        @prefetch-provider="prefetchProviderDetailDrawer"
        @import="showImportDialog = true"
        @scheduling="openSchedulingDialog"
        @demand-metrics="showDemandMetricsDialog = true"
        @advanced="openAdvancedDialog"
        @toggle-select-all="toggleAllFilteredPoolKeys"
        @batch-action="openAccountBatchDialog"
        @refresh="refreshCurrentPage"
      />

      <!-- Loading (initial) -->
      <div
        v-if="overviewLoading"
        class="flex items-center justify-center py-16"
      >
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
      </div>

      <!-- No providers -->
      <div
        v-else-if="poolProviders.length === 0"
        class="flex flex-col items-center justify-center py-16 text-center"
      >
        <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-muted">
          <Database class="h-8 w-8 text-muted-foreground" />
        </div>
        <p class="text-sm text-muted-foreground mt-4">
          {{ t('ui.noProviderse1e945c') }}
        </p>
        <p class="text-xs text-muted-foreground mt-1">
          {{ t('ui.addAProviderFirst') }}
        </p>
      </div>

      <!-- No provider selected -->
      <div
        v-else-if="!selectedProviderId"
        class="flex flex-col items-center justify-center py-16 text-center"
      >
        <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-muted">
          <Database class="h-8 w-8 text-muted-foreground" />
        </div>
        <p class="text-sm text-muted-foreground mt-4">
          {{ t('ui.selectAProviderToViewItsAccounts') }}
        </p>
      </div>

      <!-- Loading keys -->
      <div
        v-else-if="keysLoading && keyPage.keys.length === 0"
        class="flex items-center justify-center py-16"
      >
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
      </div>

      <template v-else>
        <!-- Desktop table -->
        <div
          v-if="keyPage.keys.length > 0 || hasPoolKeyFilters"
          class="hidden xl:block overflow-x-auto"
        >
          <Table class="w-full table-fixed">
            <colgroup>
              <col :style="{ width: desktopColumnWidths.name }">
              <col
                v-if="showAccountQuotaColumn"
                :style="{ width: desktopColumnWidths.quota }"
              >
              <col :style="{ width: desktopColumnWidths.stats }">
              <col :style="{ width: desktopColumnWidths.imported }">
              <col :style="{ width: desktopColumnWidths.lastUsed }">
              <col :style="{ width: desktopColumnWidths.score }">
              <col :style="{ width: desktopColumnWidths.status }">
              <col :style="{ width: desktopColumnWidths.actions }">
            </colgroup>
            <TableHeader>
              <TableRow class="border-b border-border/60 hover:bg-transparent">
                <TableHead
                  class="px-4 font-semibold whitespace-nowrap"
                  :style="{ width: desktopColumnWidths.name }"
                >
                  <div class="flex items-center gap-2">
                    <Checkbox
                      class="h-3.5 w-3.5 shrink-0"
                      :checked="selectAllFilteredPoolKeys || isCurrentPoolKeyPageFullySelected"
                      :indeterminate="!selectAllFilteredPoolKeys && isCurrentPoolKeyPagePartiallySelected"
                      :disabled="keyPage.keys.length === 0 || poolKeySelectionBusy || selectAllFilteredPoolKeys"
                      :aria-label="t('ui.selectAccountsOnThisPage')"
                      data-testid="pool-select-page-desktop"
                      @update:checked="toggleCurrentPoolKeyPage"
                    />
                    <div class="flex items-baseline gap-2">
                      <span class="leading-none">{{ t('stats.column.name') }}</span>
                      <span
                        v-if="selectedKeyCount > 0"
                        class="text-[11px] font-medium leading-none tabular-nums text-primary"
                        aria-live="polite"
                        data-testid="pool-selected-count-desktop"
                      >
                        {{ selectedKeyCountLabel }}
                      </span>
                    </div>
                  </div>
                </TableHead>
                <TableHead
                  v-if="showAccountQuotaColumn"
                  class="font-semibold whitespace-nowrap"
                  :style="{ width: desktopColumnWidths.quota }"
                >
                  {{ t('ui.quotae284fc9') }}
                </TableHead>
                <TableHead
                  class="px-2 font-semibold text-center whitespace-nowrap"
                  :style="{ width: desktopColumnWidths.stats }"
                >
                  <span>{{ t('ui.statistics89abe8c') }}</span>
                </TableHead>
                <SortableTableHead
                  class="font-semibold text-center whitespace-nowrap"
                  column-key="imported_at"
                  :active-key="sortBy"
                  :direction="sortOrder"
                  default-direction="desc"
                  align="center"
                  :style="{ width: desktopColumnWidths.imported }"
                  :title="t('ui.sortByImportTime')"
                  @sort="handleTableSort"
                >
                  {{ t('ui.importedAt') }}
                </SortableTableHead>
                <SortableTableHead
                  class="font-semibold text-center whitespace-nowrap"
                  column-key="last_used_at"
                  :active-key="sortBy"
                  :direction="sortOrder"
                  default-direction="desc"
                  align="center"
                  :style="{ width: desktopColumnWidths.lastUsed }"
                  :title="t('ui.sortByLastUsed')"
                  @sort="handleTableSort"
                >
                  {{ t('ui.lastUsed') }}
                </SortableTableHead>
                <SortableTableHead
                  class="font-semibold text-center whitespace-nowrap"
                  column-key="score"
                  :active-key="sortBy"
                  :direction="sortOrder"
                  default-direction="desc"
                  align="center"
                  :style="{ width: desktopColumnWidths.score }"
                  :title="t('ui.sortByScore')"
                  @sort="handleTableSort"
                >
                  {{ t('ui.score') }}
                </SortableTableHead>
                <SortableTableHead
                  class="font-semibold text-center whitespace-nowrap"
                  column-key="status"
                  :sortable="false"
                  align="center"
                  :filter-active="statusFilter !== 'all'"
                  :filter-title="t('ui.filterStatus')"
                  filter-content-class="w-44 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
                  :style="{ width: desktopColumnWidths.status }"
                >
                  {{ t('ui.status6320b4a') }} <template #filter="{ close }">
                    <TableFilterMenu
                      v-model="statusFilter"
                      :options="poolKeyStatusFilterOptions"
                      @select="close"
                    />
                  </template>
                </SortableTableHead>
                <TableHead
                  class="px-2 font-semibold text-center whitespace-nowrap"
                  :style="{ width: desktopColumnWidths.actions }"
                >
                  {{ t('ui.actionsed31fbb') }}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for="key in keyPage.keys"
                :key="key.key_id"
                class="border-b border-border/40 last:border-b-0 hover:bg-muted/30 transition-colors"
                :class="getPoolKeyRowClass(key.key_id)"
              >
                <TableCell
                  class="px-4 py-3 align-top"
                >
                  <div class="flex min-w-0 items-center gap-2">
                    <Checkbox
                      class="h-3.5 w-3.5 shrink-0"
                      :checked="isPoolKeySelected(key.key_id)"
                      :disabled="poolKeySelectionBusy || selectAllFilteredPoolKeys"
                      :aria-label="t('ui.selectAccountValue', { name: key.key_name || key.key_id })"
                      :data-testid="`pool-select-desktop-${key.key_id}`"
                      @update:checked="togglePoolKeySelection(key.key_id, $event === true)"
                    />
                    <div class="min-w-0 flex-1">
                      <div class="flex items-center gap-1.5 min-w-0">
                        <span class="text-sm truncate block">
                          {{ key.key_name || t('ui.untitled') }}
                        </span>
                      </div>
                      <div class="flex items-center flex-wrap gap-1 text-[11px] text-muted-foreground mt-0.5 min-w-0">
                        <input
                          v-if="editingPriorityKeyId === key.key_id"
                          :value="editingPriorityValue"
                          type="number"
                          min="1"
                          max="999999"
                          autofocus
                          class="h-[18px] w-10 rounded border border-primary/50 bg-background px-1 text-[10px] tabular-nums text-foreground outline-none ring-1 ring-primary/30 shrink-0 [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
                          @input="(e) => editingPriorityValue = Number((e.target as HTMLInputElement).value || 0)"
                          @blur="(e) => finishEditInternalPriority(key, e)"
                          @keydown.enter.prevent="(e) => finishEditInternalPriority(key, e)"
                          @keydown.esc.prevent="cancelEditInternalPriority"
                        >
                        <button
                          v-else
                          type="button"
                          class="h-4 px-1 rounded text-[10px] tabular-nums text-muted-foreground hover:text-foreground hover:bg-muted/40 transition-colors shrink-0"
                          :title="t('ui.editPriority')"
                          @click="startEditInternalPriority(key)"
                        >
                          P{{ key.internal_priority ?? 50 }}
                        </button>
                        <Button
                          v-if="canExportOAuthCredential(key)"
                          variant="ghost"
                          size="icon"
                          class="h-4 w-4 shrink-0"
                          :title="t('ui.downloadOauthAuthorizationFile')"
                          @click.stop="downloadRefreshToken(key)"
                        >
                          <Download class="w-2.5 h-2.5" />
                        </Button>
                        <Button
                          v-else-if="key.agent_identity !== true"
                          variant="ghost"
                          size="icon"
                          class="h-4 w-4 shrink-0"
                          :title="t('ui.copyKey')"
                          @click.stop="copyFullKey(key)"
                        >
                          <Copy class="w-2.5 h-2.5" />
                        </Button>
                        <span class="font-mono">
                          {{ getProviderMaskedSecretLabel(key, selectedProviderType) }}
                        </span>
                        <template v-if="keyUiStateMap[key.key_id]?.showOAuthRefreshControl">
                          <Button
                            variant="ghost"
                            size="icon"
                            class="h-4 w-4 shrink-0"
                            :disabled="refreshingOAuthKeyId === key.key_id || !keyUiStateMap[key.key_id]?.canRefreshToken"
                            :title="keyUiStateMap[key.key_id]?.oauthRefreshButtonTitle || ''"
                            @click.stop="handleRefreshOAuth(key)"
                          >
                            <RefreshCw
                              class="w-2.5 h-2.5"
                              :class="{ 'animate-spin': refreshingOAuthKeyId === key.key_id }"
                            />
                          </Button>
                          <span
                            v-if="keyUiStateMap[key.key_id]?.visibleOAuthState"
                            class="text-[10px]"
                            :class="{
                              'text-destructive': keyUiStateMap[key.key_id]?.visibleOAuthState?.isInvalid || keyUiStateMap[key.key_id]?.visibleOAuthState?.isExpired,
                              'text-warning': keyUiStateMap[key.key_id]?.visibleOAuthState?.isExpiringSoon && !keyUiStateMap[key.key_id]?.visibleOAuthState?.isExpired && !keyUiStateMap[key.key_id]?.visibleOAuthState?.isInvalid,
                              'text-muted-foreground': !keyUiStateMap[key.key_id]?.visibleOAuthState?.isExpired && !keyUiStateMap[key.key_id]?.visibleOAuthState?.isExpiringSoon && !keyUiStateMap[key.key_id]?.visibleOAuthState?.isInvalid
                            }"
                            :title="keyUiStateMap[key.key_id]?.oauthStatusTitle || ''"
                          >
                            {{ keyUiStateMap[key.key_id]?.visibleOAuthState?.text }}
                          </span>
                        </template>
                        <Badge
                          v-if="keyUiStateMap[key.key_id]?.planLabel"
                          variant="outline"
                          class="text-[9px] px-1 py-0 h-4 shrink-0"
                          :class="keyUiStateMap[key.key_id]?.planClass || ''"
                        >
                          {{ keyUiStateMap[key.key_id]?.planLabel }}
                        </Badge>
                        <Badge
                          v-if="keyUiStateMap[key.key_id]?.oauthOrgBadge"
                          variant="secondary"
                          class="text-[9px] px-1 py-0 h-4 shrink-0"
                          :title="keyUiStateMap[key.key_id]?.oauthOrgBadge?.title"
                        >
                          {{ keyUiStateMap[key.key_id]?.oauthOrgBadge?.label }}
                        </Badge>
                      </div>
                    </div>
                  </div>
                </TableCell>
                <TableCell
                  v-if="showAccountQuotaColumn"
                  class="py-3 align-top"
                >
                  <PoolKeyQuotaPanel
                    :items="quotaProgressDisplayMap[key.key_id] || []"
                    :account-quota-text="keyUiStateMap[key.key_id]?.accountQuotaText"
                    :fallback-text="keyUiStateMap[key.key_id]?.quotaFallbackText"
                    :text-class="keyUiStateMap[key.key_id]?.quotaTextClass || ''"
                    :reset-credit-text="getCodexResetCreditCountText(key)"
                    :reset-credit-items="getCodexResetCreditItemTexts(key)"
                    :can-consume-reset-credit="canConsumeCodexResetCredit(key)"
                    :consuming-reset-credit="consumingCodexResetCreditKeyId === key.key_id"
                    @consume-reset-credit="handleConsumeCodexResetCredit(key)"
                  />
                </TableCell>
                <TableCell class="py-3 px-2 align-top">
                  <PoolKeyStatsPanel
                    :cycle="isPoolKeyCycleStatsDisplay(key)"
                    :cycle-groups="getPoolKeyCycleStatsGroups(key)"
                    :account-metrics="getPoolKeyAccountStatsMetrics(key)"
                  />
                </TableCell>
                <TableCell class="py-3 text-center align-top">
                  <span class="text-[10px] text-muted-foreground whitespace-nowrap">
                    {{ keyUiStateMap[key.key_id]?.importedAtRelative || '-' }}
                  </span>
                </TableCell>
                <TableCell class="py-3 text-center align-top">
                  <span class="text-[10px] text-muted-foreground whitespace-nowrap">
                    {{ keyUiStateMap[key.key_id]?.lastUsedRelative || '-' }}
                  </span>
                </TableCell>
                <TableCell class="py-3 text-center align-top">
                  <div class="inline-flex items-center justify-center gap-1">
                    <span class="font-mono text-xs tabular-nums text-foreground/90">
                      {{ formatPoolScore(key.pool_score?.score) }}
                    </span>
                    <PoolKeyScorePopover
                      v-if="key.pool_score"
                      :score="key.pool_score"
                      :open="scoreDesktopPopoverOpenKeyId === key.key_id"
                      @update:open="(open: boolean) => handleScoreDesktopPopoverToggle(key.key_id, open)"
                    />
                  </div>
                </TableCell>
                <TableCell class="py-3 text-center align-top">
                  <Badge
                    :variant="keyUiStateMap[key.key_id]?.schedulingBadgeVariant || 'default'"
                    class="text-[10px]"
                    :title="keyUiStateMap[key.key_id]?.schedulingTitle || ''"
                  >
                    {{ keyUiStateMap[key.key_id]?.schedulingBadgeLabel }}
                  </Badge>
                </TableCell>
                <TableCell class="py-3 px-2 align-top">
                  <PoolKeyActionButtons
                    :pool-key="key"
                    layout="desktop"
                    :action-ids="keyUiStateMap[key.key_id]?.desktopActionIds || []"
                    :can-refresh-token="keyUiStateMap[key.key_id]?.canRefreshToken ?? false"
                    :refresh-button-title="keyUiStateMap[key.key_id]?.oauthRefreshButtonTitle || ''"
                    :refreshing-oauth="refreshingOAuthKeyId === key.key_id"
                    :resetting-cycle="resettingCycleKeyId === key.key_id"
                    :toggling="togglingKeyId === key.key_id"
                    :deleting="deletingKeyId === key.key_id"
                    :saving-proxy="savingProxyKeyId === key.key_id"
                    :proxy-node-name="getKeyProxyNodeName(key)"
                    :proxy-popover-open="proxyDesktopPopoverOpenKeyId === key.key_id"
                    @update:proxy-popover-open="(v: boolean) => handleProxyDesktopPopoverToggle(key.key_id, v)"
                    @select-proxy="(v: string) => setKeyProxy(key, v)"
                    @clear-proxy="clearKeyProxy(key)"
                    @download="downloadRefreshToken(key)"
                    @copy="copyFullKey(key)"
                    @refresh-oauth="handleRefreshOAuth(key)"
                    @clear-cooldown="clearCooldown(key.key_id)"
                    @permissions="handleKeyPermissions(key)"
                    @reset-cycle-stats="handleResetCycleStats(key)"
                    @edit="handleEditKey(key)"
                    @toggle-active="toggleKeyActive(key)"
                    @delete="handleDeleteKey(key)"
                  />
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>

        <!-- Mobile card list -->
        <div
          v-if="keyPage.keys.length > 0"
          class="xl:hidden divide-y divide-border/40"
        >
          <div
            v-for="key in keyPage.keys"
            :key="key.key_id"
            class="p-4 sm:p-5 hover:bg-muted/30 transition-colors"
            :class="getPoolKeyRowClass(key.key_id)"
          >
            <div class="space-y-3">
              <div class="flex items-center gap-1.5">
                <Checkbox
                  class="h-3.5 w-3.5 shrink-0"
                  :checked="isPoolKeySelected(key.key_id)"
                  :disabled="poolKeySelectionBusy || selectAllFilteredPoolKeys"
                  :aria-label="t('ui.selectAccountValue', { name: key.key_name || key.key_id })"
                  :data-testid="`pool-select-mobile-${key.key_id}`"
                  @update:checked="togglePoolKeySelection(key.key_id, $event === true)"
                />
                <div class="min-w-0 truncate text-sm font-medium">
                  {{ key.key_name || t('ui.untitled') }}
                </div>
              </div>

              <div class="flex flex-wrap items-center gap-1.5">
                <Badge
                  :variant="keyUiStateMap[key.key_id]?.schedulingBadgeVariant || 'default'"
                  class="text-[10px] shrink-0"
                  :title="keyUiStateMap[key.key_id]?.schedulingTitle || ''"
                >
                  {{ keyUiStateMap[key.key_id]?.schedulingBadgeLabel }}
                </Badge>
                <span
                  v-if="key.cooldown_ttl_seconds"
                  class="inline-flex items-center rounded-full border border-red-500/30 bg-red-500/10 px-2 py-0.5 text-[10px] font-medium leading-4 text-red-700 dark:text-red-300"
                > {{ t('ui.cooldown') }} {{ formatTTL(key.cooldown_ttl_seconds) }}
                </span>
                <template
                  v-for="item in keyUiStateMap[key.key_id]?.mobileTagItems || []"
                  :key="`${key.key_id}-${item.key}`"
                >
                  <button
                    v-if="item.key === 'priority'"
                    type="button"
                    class="inline-flex max-w-full items-center rounded-full border px-2 py-0.5 text-[10px] font-medium leading-4 hover:border-primary/40 hover:text-foreground"
                    :class="getMobileTagClass(item)"
                    :title="t('ui.valueClickToEditPriority', { value: item.label })"
                    @click="quickEditInternalPriority(key)"
                  >
                    {{ item.label }}
                  </button>
                  <Badge
                    v-else-if="item.key === 'plan'"
                    variant="outline"
                    class="text-[9px] px-1 py-0 h-4 shrink-0"
                    :class="keyUiStateMap[key.key_id]?.planClass || ''"
                  >
                    {{ item.label }}
                  </Badge>
                  <Badge
                    v-else-if="item.key === 'org'"
                    variant="secondary"
                    class="text-[9px] px-1 py-0 h-4 shrink-0"
                    :title="keyUiStateMap[key.key_id]?.oauthOrgBadge?.title"
                  >
                    {{ item.label }}
                  </Badge>
                  <span
                    v-else
                    class="inline-flex max-w-full items-center rounded-full border px-2 py-0.5 text-[10px] font-medium leading-4"
                    :class="getMobileTagClass(item)"
                    :title="item.label"
                  >
                    {{ item.label }}
                  </span>
                </template>
              </div>

              <div class="overflow-x-auto rounded-xl border border-border/50 bg-muted/30 px-3 py-2 text-[11px] text-muted-foreground">
                <div class="space-y-1 text-center">
                  <PoolKeyStatsPanel
                    :cycle="isPoolKeyCycleStatsDisplay(key)"
                    :cycle-groups="getPoolKeyCycleStatsGroups(key)"
                    :account-metrics="getPoolKeyAccountStatsMetrics(key)"
                    variant="mobile"
                  />
                  <div class="flex items-center justify-between gap-2 border-t border-border/40 pt-1 mt-1">
                    <span class="text-muted-foreground">{{ t('ui.import') }}</span>
                    <span class="font-medium text-foreground/90">{{ keyUiStateMap[key.key_id]?.importedAtRelative || '-' }}</span>
                  </div>
                  <div class="flex items-center justify-between gap-2">
                    <span class="text-muted-foreground">{{ t('ui.lastUsed') }}</span>
                    <span class="font-medium text-foreground/90">{{ keyUiStateMap[key.key_id]?.lastUsedRelative || '-' }}</span>
                  </div>
                  <div class="flex items-center justify-between gap-2">
                    <span class="text-muted-foreground">{{ t('ui.score') }}</span>
                    <div class="flex items-center gap-1">
                      <span class="font-mono font-medium text-foreground/90 tabular-nums">
                        {{ formatPoolScore(key.pool_score?.score) }}
                      </span>
                      <PoolKeyScorePopover
                        v-if="key.pool_score"
                        :score="key.pool_score"
                        :open="scoreMobilePopoverOpenKeyId === key.key_id"
                        @update:open="(open: boolean) => handleScoreMobilePopoverToggle(key.key_id, open)"
                      />
                    </div>
                  </div>
                </div>
              </div>

              <PoolKeyQuotaPanel
                v-if="showAccountQuotaColumn"
                :items="quotaProgressDisplayMap[key.key_id] || []"
                :account-quota-text="keyUiStateMap[key.key_id]?.accountQuotaText"
                :fallback-text="keyUiStateMap[key.key_id]?.quotaFallbackText"
                :text-class="keyUiStateMap[key.key_id]?.quotaTextClass || ''"
                :reset-credit-text="getCodexResetCreditCountText(key)"
                :reset-credit-items="getCodexResetCreditItemTexts(key)"
                :can-consume-reset-credit="canConsumeCodexResetCredit(key)"
                :consuming-reset-credit="consumingCodexResetCreditKeyId === key.key_id"
                variant="mobile"
                @consume-reset-credit="handleConsumeCodexResetCredit(key)"
              />

              <PoolKeyActionButtons
                :pool-key="key"
                layout="mobile"
                :action-ids="keyUiStateMap[key.key_id]?.mobileActionIds || []"
                :can-refresh-token="keyUiStateMap[key.key_id]?.canRefreshToken ?? false"
                :refresh-button-title="keyUiStateMap[key.key_id]?.oauthRefreshButtonTitle || ''"
                :refreshing-oauth="refreshingOAuthKeyId === key.key_id"
                :resetting-cycle="resettingCycleKeyId === key.key_id"
                :toggling="togglingKeyId === key.key_id"
                :deleting="deletingKeyId === key.key_id"
                :saving-proxy="savingProxyKeyId === key.key_id"
                :proxy-node-name="getKeyProxyNodeName(key)"
                :proxy-popover-open="proxyMobilePopoverOpenKeyId === key.key_id"
                @update:proxy-popover-open="(v: boolean) => handleProxyMobilePopoverToggle(key.key_id, v)"
                @select-proxy="(v: string) => setKeyProxy(key, v)"
                @clear-proxy="clearKeyProxy(key)"
                @download="downloadRefreshToken(key)"
                @copy="copyFullKey(key)"
                @refresh-oauth="handleRefreshOAuth(key)"
                @clear-cooldown="clearCooldown(key.key_id)"
                @permissions="handleKeyPermissions(key)"
                @reset-cycle-stats="handleResetCycleStats(key)"
                @edit="handleEditKey(key)"
                @toggle-active="toggleKeyActive(key)"
                @delete="handleDeleteKey(key)"
              />
            </div>
          </div>
        </div>

        <!-- Empty keys -->
        <div
          v-if="keyPage.keys.length === 0 && !keysLoading && keysLoadedOnce"
          class="flex flex-col items-center justify-center py-16 text-center"
        >
          <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-muted">
            <KeyRound class="h-8 w-8 text-muted-foreground" />
          </div>
          <p class="text-sm text-muted-foreground mt-4">
            {{ hasPoolKeyFilters ? t('ui.noMatchingAccounts') : t('ui.noAccounts') }}
          </p>
          <Button
            v-if="hasPoolKeyFilters"
            variant="outline"
            size="sm"
            class="mt-3"
            @click="clearPoolKeyFilters"
          >
            {{ t('ui.clearFilters') }}
          </Button>
          <Button
            v-else
            variant="outline"
            size="sm"
            class="mt-3"
            @click="showImportDialog = true"
          >
            <Upload class="w-3.5 h-3.5 mr-1.5" /> {{ t('ui.addAccount') }}
          </Button>
        </div>

        <!-- Pagination -->
        <Pagination
          v-if="keyPage.keys.length > 0"
          :current="currentPage"
          :total="keyPage.total"
          :page-size="pageSize"
          cache-key="pool-keys-page-size"
          @update:current="currentPage = $event"
          @update:page-size="pageSize = $event"
        />
      </template>
    </Card>

    <!-- Dialogs -->
    <OAuthAccountDialog
      v-if="selectedProviderId"
      :open="showImportDialog"
      :provider-id="selectedProviderId"
      :provider-type="selectedProviderType || null"
      @close="showImportDialog = false"
      @saved="handleAccountDialogSaved"
    />
    <PoolSchedulingDialog
      v-if="selectedProviderId"
      v-model="showSchedulingDialog"
      :provider-id="selectedProviderId"
      :provider-type="selectedProviderType"
      :current-config="selectedProviderConfig"
      @saved="handleSchedulingSaved"
    />
    <PoolAdvancedDialog
      v-if="selectedProviderId"
      v-model="showAdvancedDialog"
      :provider-id="selectedProviderId"
      :provider-type="selectedProviderType"
      :current-config="selectedProviderConfig"
      :current-claude-config="selectedProviderClaudeConfig"
      @saved="handleSchedulingSaved"
    />
    <PoolDemandMetricsDialog
      v-model="showDemandMetricsDialog"
      :provider-name="selectedProviderOverview?.provider_name"
      :samples="providerDemandMetricSamples"
    />
    <ProviderDetailDrawer
      v-if="providerDrawerMounted && selectedProviderId"
      :open="providerDrawerOpen"
      :provider-id="selectedProviderId"
      :initial-provider="selectedProviderData"
      @update:open="providerDrawerOpen = $event"
      @edit="openProviderEditDialog"
      @toggle-status="toggleSelectedProviderStatus"
      @refresh="handleProviderDrawerRefresh"
    />
    <ProviderFormDialog
      v-model="providerEditDialogOpen"
      :provider="providerToEdit"
      @provider-updated="handleProviderEditSaved"
    />
    <PoolAccountBatchDialog
      v-if="selectedProviderId"
      v-model="showAccountBatchDialog"
      :provider-id="selectedProviderId"
      :provider-name="selectedProviderData?.name || ''"
      :provider-type="selectedProviderData?.provider_type ?? selectedProviderOverview?.provider_type ?? undefined"
      :batch-concurrency="selectedProviderConfig?.batch_concurrency"
      :selected-keys="selectedPoolKeys"
      :select-all-filtered="selectAllFilteredPoolKeys"
      :selected-count="selectedKeyCount"
      :selection-filters="poolKeySelectionFilters"
      :initial-action="pendingAccountBatchAction"
      @changed="handleAccountBatchChanged"
      @edit-config="openKeyBatchEditDialog"
    />
    <PoolKeyBatchEditDialog
      v-if="selectedProviderId"
      :open="keyBatchEditDialogOpen"
      :provider-id="selectedProviderId"
      :provider-name="selectedProviderData?.name || ''"
      :key-ids="keyBatchEditKeyIds"
      :available-api-formats="selectedProviderData?.api_formats || []"
      @close="closeKeyBatchEditDialog"
      @saved="handleKeyBatchEditSaved"
    />
    <KeyFormDialog
      v-if="selectedProviderId"
      :open="keyFormDialogOpen"
      :endpoint="null"
      :provider-type="selectedProviderData?.provider_type ?? selectedProviderOverview?.provider_type ?? null"
      :editing-key="editingKey"
      :provider-id="selectedProviderId"
      :available-api-formats="selectedProviderData?.api_formats || []"
      @close="closeKeyFormDialog"
      @saved="handleDialogSaved"
    />
    <OAuthKeyEditDialog
      :open="oauthKeyEditDialogOpen"
      :editing-key="editingKey"
      @close="closeOAuthEditDialog"
      @saved="handleDialogSaved"
    />
    <KeyAllowedModelsEditDialog
      v-if="selectedProviderId"
      :open="keyPermissionsDialogOpen"
      :api-key="editingKey"
      :provider-id="selectedProviderId || ''"
      @close="closeKeyPermissionsDialog"
      @saved="handleDialogSaved"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, defineAsyncComponent } from 'vue'
import {
  Upload,
  RefreshCw,
  Database,
  KeyRound,
  Download,
  Copy,
} from 'lucide-vue-next'

import {
  Card,
  Badge,
  Button,
  Checkbox,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  SortableTableHead,
  TableFilterMenu,
  TableCell,
  Pagination,
} from '@/components/ui'
import { useToast } from '@/composables/useToast'
import { useClipboard } from '@/composables/useClipboard'
import { useCountdownTimer } from '@/composables/useCountdownTimer'
import { useConfirm } from '@/composables/useConfirm'
import { useRouteQuery } from '@/composables/useRouteQuery'
import { useBatchSelection } from '@/composables/useBatchSelection'
import { useI18n } from '@/i18n'
import { parseApiError } from '@/utils/errorParser'
import {
  getPoolOverview,
  getPoolSchedulingPresets,
  listPoolKeys,
  clearPoolCooldown,
} from '@/api/endpoints/pool'
import {
  revealEndpointKey,
  exportKey,
  deleteEndpointKey,
  updateProviderKey,
  refreshProviderQuota,
  consumeCodexResetCredit,
  resetProviderKeyCycleStats,
} from '@/api/endpoints/keys'
import { refreshProviderOAuth } from '@/api/endpoints/provider_oauth'
import type {
  PoolOverviewItem,
  PoolKeyDetail,
  PoolKeySelectionRequest,
  PoolKeysPageResponse,
  PoolPresetMeta,
} from '@/api/endpoints/pool'
import type {
  ClaudeCodeAdvancedConfig,
  EndpointAPIKey,
  PoolAdvancedConfig,
  ProviderWithEndpointsSummary,
} from '@/api/endpoints/types/provider'
import { getProvider, updateProvider } from '@/api/endpoints'
import { useProxyNodesStore } from '@/stores/proxy-nodes'
import PoolSchedulingDialog from '@/features/pool/components/PoolSchedulingDialog.vue'
import PoolAdvancedDialog from '@/features/pool/components/PoolAdvancedDialog.vue'
import PoolDemandMetricsDialog from '@/features/pool/components/PoolDemandMetricsDialog.vue'
import PoolAccountBatchDialog from '@/features/pool/components/PoolAccountBatchDialog.vue'
import PoolKeyBatchEditDialog from '@/features/pool/components/PoolKeyBatchEditDialog.vue'
import PoolManagementHeader from '@/features/pool/components/PoolManagementHeader.vue'
import PoolKeyQuotaPanel from '@/features/pool/components/PoolKeyQuotaPanel.vue'
import PoolKeyStatsPanel from '@/features/pool/components/PoolKeyStatsPanel.vue'
import PoolKeyScorePopover from '@/features/pool/components/PoolKeyScorePopover.vue'
import PoolKeyActionButtons from '@/features/pool/components/PoolKeyActionButtons.vue'
import KeyAllowedModelsEditDialog from '@/features/providers/components/KeyAllowedModelsEditDialog.vue'
import KeyFormDialog from '@/features/providers/components/KeyFormDialog.vue'
import OAuthKeyEditDialog from '@/features/providers/components/OAuthKeyEditDialog.vue'
import OAuthAccountDialog from '@/features/providers/components/OAuthAccountDialog.vue'
import ProviderFormDialog from '@/features/providers/components/ProviderFormDialog.vue'
import {
  splitPoolMobileActions,
  type PoolMobileActionId,
  type PoolMobileTagItem,
} from '@/features/pool/utils/poolMobilePresentation'
import {
  buildPoolManagementQueryPatch,
  readPoolManagementViewState,
  resolvePoolManagementPageAfterLoad,
  type PoolManagementSortBy,
  type PoolManagementSortOrder,
  type PoolManagementViewState,
  writePoolManagementViewState,
} from '@/features/pool/utils/poolManagementState'
import type { PoolBatchActionValue } from '@/features/pool/utils/poolBatchActions'
import {
  buildAccountTotalStatsDisplay,
  buildPoolStatsDisplay,
  type PoolCodexCycleStatsGroup,
  type PoolStatsDisplay,
  type PoolStatsMetric,
} from '@/features/pool/utils/poolStatsDisplay'
import { resetCodexCycleUsageWindows } from '@/features/pool/utils/poolCycleStats'
import { mergePoolKeyQuotaSnapshots } from '@/features/pool/utils/poolQuotaRefresh'
import {
  getQuotaProgressLabel,
  getQuotaProgressCountdown,
  getQuotaProgressResetDisplayText,
  getQuotaProgressMeterDisplayText,
  getQuotaSnapshotProviderType,
  formatQuotaValue,
  resolvePoolKeyPlanType,
  parseQuotaProgressItems,
  getQuotaRemainingClassByRemaining,
  getQuotaRemainingBarColorByRemaining,
} from '@/features/pool/utils/poolKeyQuotaProgress'
import type { QuotaProgressItem, QuotaProgressDisplayItem } from '@/features/pool/utils/poolKeyQuotaProgress'
import {
  getSchedulingBadgeLabel,
  getSchedulingBadgeVariant,
  getSchedulingTitle,
  formatTTL,
  getRowClass,
  getMobileTagItems,
  getMobileTagClass,
  getVisibleOAuthState,
  getOAuthRefreshButtonTitle,
  getOAuthStatusTitle,
  getQuotaTextClass,
  formatPoolScore,
  formatRelativeTime,
  formatPoolKeyImportedAt,
} from '@/features/pool/utils/poolKeyStatus'
import type { PoolStatusVariant } from '@/features/pool/utils/poolKeyStatus'
import {
  clearPendingCodexResetCreditIdempotencyKey,
  clearPendingCodexResetCreditIdempotencyKeyForOutcome,
  createCodexResetCreditIdempotencyKey,
  formatCodexResetCreditCount,
  formatCodexResetCreditExpiresAt,
  getCodexResetCreditAvailableCount,
  getCodexResetCreditReservationIdempotencyKey,
  getVisibleCodexResetCreditItems,
  mergeCodexQuotaDisplays,
  readPendingCodexResetCreditIdempotencyKey,
  rememberPendingCodexResetCreditIdempotencyKey,
} from '@/features/providers/components/codex-reset-credit-display'
import { getOAuthOrgBadge } from '@/features/providers/utils/oauthIdentity'
import { formatOAuthPlanType, getOAuthPlanTypeClass } from '@/features/providers/utils/oauthPlanType'
import { getOAuthRefreshFeedback } from '@/features/providers/utils/oauthRefreshFeedback'
import {
  canEditOAuthCredential,
  canExportOAuthCredential,
  canRefreshOAuthCredential,
  getProviderMaskedSecretLabel,
  isOAuthManagedCredential,
  isServiceAccountCredential,
  shouldShowOAuthRefreshControl,
} from '@/features/providers/utils/providerKeyAuth'
import {
  getOAuthStatusDisplayWithFallback,
} from '@/features/providers/utils/providerKeyStatus'
import {
  getGeminiCliAccountCreditsText,
  getQuotaDisplayText,
} from '@/features/providers/utils/providerKeyQuota'

const loadProviderDetailDrawer = () => import('@/features/providers/components/ProviderDetailDrawer.vue')
const ProviderDetailDrawer = defineAsyncComponent(loadProviderDetailDrawer)

function prefetchProviderDetailDrawer(): void {
  void loadProviderDetailDrawer().catch(() => {})
}

const { success, error: showError, warning: showWarning } = useToast()
const { t } = useI18n()
const { confirm } = useConfirm()
const { copyToClipboard } = useClipboard()
const { tick: countdownTick, start: startCountdownTimer } = useCountdownTimer()
const proxyNodesStore = useProxyNodesStore()
const { getQueryValue, patchQuery } = useRouteQuery()

const poolManagementViewStorage = typeof window === 'undefined' ? undefined : window.sessionStorage
const restoredViewState = readPoolManagementViewState(
  {
    providerId: getQueryValue('providerId'),
    search: getQueryValue('search'),
    status: getQueryValue('status'),
    page: getQueryValue('page'),
    pageSize: getQueryValue('pageSize'),
    sortBy: getQueryValue('sortBy'),
    sortOrder: getQueryValue('sortOrder'),
  },
  poolManagementViewStorage,
)

// --- Overview ---
const poolProviders = ref<PoolOverviewItem[]>([])
const overviewLoading = ref(true)
let overviewRequestId = 0
let selectProviderRequestId = 0
let providerDataRequestId = 0
let keysRequestId = 0
let keysSearchDebounceTimer: number | null = null
let providerDetailDrawerPrefetchIdleId: number | null = null
let providerDetailDrawerPrefetchTimer: number | null = null
const keysSearchPending = ref(false)
let demandMetricsPollingTimer: number | null = null
let demandMetricsRequestId = 0
let suppressFiltersWatch = false
let hasHydratedInitialProviderSelection = false
const POOL_OVERVIEW_CACHE_TTL_MS = 10 * 1000
const POOL_KEYS_CACHE_TTL_MS = 10 * 1000
const POOL_SCHEDULING_PRESETS_CACHE_TTL_MS = 5 * 60 * 1000
const POOL_DEMAND_METRICS_SAMPLES_LIMIT = 120
const POOL_DEMAND_METRICS_POLL_INTERVAL_MS = 10 * 1000

type IdleCallbackWindow = typeof window & {
  requestIdleCallback?: (callback: () => void, options?: { timeout: number }) => number
  cancelIdleCallback?: (handle: number) => void
}

function scheduleProviderDetailDrawerPrefetch(): void {
  const idleWindow = window as IdleCallbackWindow
  if (idleWindow.requestIdleCallback) {
    providerDetailDrawerPrefetchIdleId = idleWindow.requestIdleCallback(() => {
      providerDetailDrawerPrefetchIdleId = null
      prefetchProviderDetailDrawer()
    }, { timeout: 1500 })
    return
  }
  providerDetailDrawerPrefetchTimer = window.setTimeout(() => {
    providerDetailDrawerPrefetchTimer = null
    prefetchProviderDetailDrawer()
  }, 800)
}

function cancelProviderDetailDrawerPrefetch(): void {
  const idleWindow = window as IdleCallbackWindow
  if (providerDetailDrawerPrefetchIdleId !== null) {
    idleWindow.cancelIdleCallback?.(providerDetailDrawerPrefetchIdleId)
    providerDetailDrawerPrefetchIdleId = null
  }
  if (providerDetailDrawerPrefetchTimer !== null) {
    clearTimeout(providerDetailDrawerPrefetchTimer)
    providerDetailDrawerPrefetchTimer = null
  }
}

interface PoolDemandMetricSample {
  providerId: string
  sampledAt: number
  hotCount: number
  desiredHot: number
  inFlight: number
  emaInFlight: number
  burstPending: boolean
}

const showDemandMetricsDialog = ref(false)
const providerDemandMetricSamples = ref<PoolDemandMetricSample[]>([])
const poolKeyStatusFilterOptions: Array<{ value: PoolManagementViewState['status'], label: string }> = [
  { value: 'all', get label() { return t('ui.allStatuses') } },
  { value: 'available', get label() { return t('ui.available4d99c97') } },
  { value: 'cooldown', get label() { return t('ui.coolingDown') } },
  { value: 'inactive', get label() { return t('ui.disabledbc5a87a') } },
  { value: 'invalid', get label() { return t('ui.invalid') } },
  { value: 'expired', get label() { return t('ui.expired') } },
  { value: 'account_banned', get label() { return t('ui.accountBanned') } },
  { value: 'quota_exhausted', get label() { return t('ui.quotaExhausted') } },
  { value: 'account_forbidden', get label() { return t('ui.accessRestricted') } },
  { value: 'account_disabled', get label() { return t('ui.accountDisabled') } },
  { value: 'workspace_deactivated', get label() { return t('ui.workspaceDisabled') } },
  { value: 'account_verification', get label() { return t('ui.verificationRequired') } },
  { value: 'account_quarantined', get label() { return t('ui.accountIsolation') } },
  { value: 'account_blocked', get label() { return t('ui.accountIssue') } },
  { value: 'rate_limited', get label() { return t('ui.rateLimited6d5ac6f') } },
  { value: 'cost_exhausted', get label() { return t('ui.limitExceeded') } },
]

async function loadOverview(options: { cacheTtlMs?: number, silent?: boolean } = {}) {
  const requestId = ++overviewRequestId
  if (!options.silent) {
    overviewLoading.value = true
  }
  try {
    const res = await getPoolOverview({ cacheTtlMs: options.cacheTtlMs ?? 0 })
    if (requestId !== overviewRequestId) return
    const allProviders = Array.isArray(res.items) ? res.items : []
    const enabledProviders = allProviders.filter(item => item.pool_enabled)
    poolProviders.value = enabledProviders

    const queryProviderId = getQueryValue('providerId')
    const queryProviderExists = Boolean(
      queryProviderId && enabledProviders.some(item => item.provider_id === queryProviderId),
    )
    const currentSelectedId = selectedProviderId.value
    const currentSelectedExists = Boolean(
      currentSelectedId && enabledProviders.some(item => item.provider_id === currentSelectedId),
    )
    const selectedId = currentSelectedExists
      ? currentSelectedId
      : (queryProviderExists ? queryProviderId : currentSelectedId)
    const selectedStillExists = Boolean(
      selectedId && enabledProviders.some(item => item.provider_id === selectedId),
    )

    if (selectedStillExists && selectedId) {
      // 页面刷新时可能先恢复了选中的 Provider，但列表请求尚未触发；
      // overview 回来后补一次初始化拉取，确保空态不会卡住。
      if (!hasHydratedInitialProviderSelection || selectedId !== selectedProviderId.value) {
        await selectProvider(selectedId, {
          preserveSearch: true,
          preserveStatus: true,
          preservePagination: true,
          cacheTtlMs: options.cacheTtlMs ? POOL_KEYS_CACHE_TTL_MS : 0,
        })
      }
      return
    }

    if (enabledProviders.length > 0) {
      const fallbackProviderId = enabledProviders[0].provider_id
      const shouldPreserveViewState = Boolean(selectedId)
      await selectProvider(fallbackProviderId, {
        preserveSearch: shouldPreserveViewState,
        preserveStatus: shouldPreserveViewState,
        preservePagination: shouldPreserveViewState,
        cacheTtlMs: options.cacheTtlMs ? POOL_KEYS_CACHE_TTL_MS : 0,
      })
    } else {
      selectedProviderId.value = null
      selectedProviderData.value = null
      keysLoadedOnce.value = false
      resetPoolKeySelection(true)
      providerDrawerOpen.value = false
      showAccountBatchDialog.value = false
      keyBatchEditDialogOpen.value = false
      keyBatchEditKeyIds.value = []
      resetKeyPage()
    }
  } catch (err) {
    if (requestId !== overviewRequestId) return
    if (!options.silent) {
      showError(parseApiError(err))
    } else {
      showWarning(parseApiError(err, t('ui.failedToSyncProviderOverview')))
    }
  } finally {
    if (requestId === overviewRequestId) {
      overviewLoading.value = false
    }
  }
}

async function handleSchedulingSaved(updatedProvider: ProviderWithEndpointsSummary) {
  if (!selectedProviderId.value || updatedProvider.id !== selectedProviderId.value) return
  // 保存前发出的详情读取不得覆盖这次保存返回的新配置。
  providerDataRequestId += 1
  // 优先回写保存接口返回值，避免弹窗立即重开时读到旧配置。
  if (selectedProviderData.value) {
    Object.assign(selectedProviderData.value, updatedProvider)
  } else {
    selectedProviderData.value = updatedProvider
  }
  showSchedulingDialog.value = false
  showAdvancedDialog.value = false
  await Promise.all([loadKeys({ silent: true }), loadOverview({ silent: true })])
}

// --- Provider Selection ---
const selectedProviderId = ref<string | null>(restoredViewState.providerId)
const selectedProviderData = ref<ProviderWithEndpointsSummary | null>(null)

// Proxy for Select v-model (string, not string|null)
const selectedProviderIdProxy = computed({
  get: () => selectedProviderId.value ?? '',
  set: (val: string) => {
    if (val && val !== selectedProviderId.value) {
      void selectProvider(val, { cacheTtlMs: POOL_KEYS_CACHE_TTL_MS })
    }
  },
})

const providerSelectDisabled = computed(() => poolProviders.value.length === 0)

const selectedProviderConfig = computed<PoolAdvancedConfig | null>(() => {
  return (selectedProviderData.value as Record<string, unknown> | null)?.pool_advanced as PoolAdvancedConfig | null ?? null
})

const selectedProviderClaudeConfig = computed(() => {
  return (selectedProviderData.value as Record<string, unknown> | null)?.claude_code_advanced as ClaudeCodeAdvancedConfig | null ?? null
})

function defaultEnabledPresetCount(providerType: string): number {
  return ['codex', 'windsurf'].includes(providerType) ? 2 : 1
}

const DEFAULT_PRESET_LABELS: Record<string, string> = {
  lru: 'LRU',
  free_team_first: 'Free/Team',
  free_first: 'Free',
  team_first: 'Team',
  plus_first: 'Plus',
  pro_first: 'Pro',
  get recent_refresh() { return t('ui.refreshFirst') },
  get quota_balanced() { return t('ui.balancedQuota') },
  get single_account() { return t('ui.singleAccountFirst') },
}
const presetLabelsByName = ref<Record<string, string>>({ ...DEFAULT_PRESET_LABELS })

function normalizePresetName(value: unknown): string {
  return String(value ?? '').trim().toLowerCase()
}

async function loadSchedulingPresetMetas(options: { cacheTtlMs?: number } = {}): Promise<void> {
  try {
    const metas = await getPoolSchedulingPresets({ cacheTtlMs: options.cacheTtlMs ?? 0 })
    const next: Record<string, string> = {}
    for (const meta of metas as PoolPresetMeta[]) {
      const name = normalizePresetName(meta.name)
      if (!name) continue
      const label = String(meta.label ?? '').trim()
      next[name] = label || name
    }
    if (Object.keys(next).length > 0) {
      presetLabelsByName.value = next
    }
  } catch {
    presetLabelsByName.value = { ...DEFAULT_PRESET_LABELS }
  }
}

const selectedProviderOverview = computed<PoolOverviewItem | null>(() => {
  const selectedId = selectedProviderId.value
  if (!selectedId) return null
  return poolProviders.value.find(item => item.provider_id === selectedId) || null
})

const showAdaptiveHotPoolMetricsButton = computed(() => {
  if (!selectedProviderId.value) return false
  return selectedProviderConfig.value?.probing_enabled === true
})

function normalizeDemandMetricNumber(value: unknown): number {
  const normalized = Number(value ?? 0)
  if (!Number.isFinite(normalized) || normalized <= 0) return 0
  return normalized
}

function buildDemandMetricSample(overview: PoolOverviewItem): PoolDemandMetricSample {
  return {
    providerId: overview.provider_id,
    sampledAt: Date.now(),
    hotCount: Math.floor(normalizeDemandMetricNumber(overview.provider_hot_count)),
    desiredHot: Math.floor(normalizeDemandMetricNumber(overview.provider_desired_hot)),
    inFlight: Math.floor(normalizeDemandMetricNumber(overview.provider_in_flight)),
    emaInFlight: normalizeDemandMetricNumber(overview.provider_ema_in_flight),
    burstPending: overview.provider_burst_pending === true,
  }
}

function appendDemandMetricSample(overview: PoolOverviewItem | null): void {
  if (!overview || !showDemandMetricsDialog.value || !showAdaptiveHotPoolMetricsButton.value) return
  const nextSample = buildDemandMetricSample(overview)
  const existing = providerDemandMetricSamples.value.filter(
    sample => sample.providerId === overview.provider_id,
  )
  const lastSample = existing[existing.length - 1]
  if (
    lastSample
    && nextSample.sampledAt - lastSample.sampledAt < 1000
    && lastSample.hotCount === nextSample.hotCount
    && lastSample.desiredHot === nextSample.desiredHot
    && lastSample.inFlight === nextSample.inFlight
    && lastSample.emaInFlight === nextSample.emaInFlight
    && lastSample.burstPending === nextSample.burstPending
  ) {
    providerDemandMetricSamples.value = existing
    return
  }
  providerDemandMetricSamples.value = [...existing, nextSample]
    .slice(-POOL_DEMAND_METRICS_SAMPLES_LIMIT)
}

function stopDemandMetricsPolling(): void {
  if (demandMetricsPollingTimer !== null) {
    window.clearInterval(demandMetricsPollingTimer)
    demandMetricsPollingTimer = null
  }
}

async function refreshDemandMetricsOverview(): Promise<void> {
  const providerId = selectedProviderId.value
  if (!showDemandMetricsDialog.value || !showAdaptiveHotPoolMetricsButton.value || !providerId) {
    return
  }

  const requestId = ++demandMetricsRequestId
  try {
    const res = await getPoolOverview({ cacheTtlMs: 0 })
    if (
      requestId !== demandMetricsRequestId
      || !showDemandMetricsDialog.value
      || selectedProviderId.value !== providerId
    ) {
      return
    }
    const allProviders = Array.isArray(res.items) ? res.items : []
    const enabledProviders = allProviders.filter(item => item.pool_enabled)
    poolProviders.value = enabledProviders
    appendDemandMetricSample(
      enabledProviders.find(item => item.provider_id === providerId) || null,
    )
  } catch {
    // 指标弹窗只做尽力刷新，失败不打断主流程。
  }
}

function startDemandMetricsPolling(): void {
  stopDemandMetricsPolling()
  appendDemandMetricSample(selectedProviderOverview.value)
  void refreshDemandMetricsOverview()
  demandMetricsPollingTimer = window.setInterval(() => {
    if (!showDemandMetricsDialog.value || !showAdaptiveHotPoolMetricsButton.value) return
    if (document.visibilityState === 'hidden') return
    void refreshDemandMetricsOverview()
  }, POOL_DEMAND_METRICS_POLL_INTERVAL_MS)
}

const poolSchedulingLabel = computed(() => {
  if (!selectedProviderConfig.value && selectedProviderOverview.value?.pool_enabled === false) {
    return t('ui.disabled')
  }

  const cfg = selectedProviderConfig.value

  // No pool_advanced config at all: use default enabled presets count
  if (!cfg) return t('ui.valueDimensions', { count: defaultEnabledPresetCount(selectedProviderType.value) })

  const presets = Array.isArray(cfg.scheduling_presets) ? cfg.scheduling_presets : []
  const presetLabels = presetLabelsByName.value

  if (presets.length > 0) {
    // New format: object list with { preset, enabled }
    const first = presets[0]
    if (typeof first === 'object' && first !== null && 'preset' in first) {
      const enabledCount = (presets as Array<{ preset: string; enabled?: boolean }>)
        .filter(p => p.enabled !== false)
        .length
      return enabledCount > 0
        ? t('ui.valueDimensions', { count: enabledCount })
        : t('feedback.pool.noEnabledDimensions')
    }

    // Legacy string list format
    if (typeof first === 'string') {
      const labels = (presets as string[])
        .map(p => presetLabels[normalizePresetName(p)])
        .filter(Boolean)
      if (labels.length > 0) return t('ui.valueDimensions', { count: labels.length })
    }
  }

  // Fallback: legacy scheduling_mode field
  if (cfg.scheduling_mode === 'multi_score') {
    return t('ui.multiFactorScoring')
  }

  const lruEnabled = cfg.scheduling_mode === 'lru' || cfg.lru_enabled === true
  const stickyTtl = Number(cfg.sticky_session_ttl_seconds ?? 3600)
  const stickyEnabled = Number.isFinite(stickyTtl) && stickyTtl > 0

  if (lruEnabled && stickyEnabled) return t('ui.lruStickySessions')
  if (lruEnabled) return 'LRU'
  if (!cfg.scheduling_mode && (cfg.lru_enabled === null || cfg.lru_enabled === undefined)) {
    return t('ui.valueDimensions', { count: defaultEnabledPresetCount(selectedProviderType.value) })
  }
  if (stickyEnabled) return t('ui.sticky')
  return t('ui.random')
})

const selectedProviderType = computed(() => {
  const fromDetail = String(selectedProviderData.value?.provider_type || '').trim().toLowerCase()
  if (fromDetail) return fromDetail
  const fromOverview = selectedProviderOverview.value?.provider_type
  return String(fromOverview || '').trim().toLowerCase()
})

const selectedProviderStatusText = computed(() => {
  if (!selectedProviderId.value) return ''
  const providerActive = selectedProviderData.value?.is_active
  if (providerActive === false) return t('ui.disable')
  if (providerActive === true) return t('ui.enablef4f0ead')
  if (selectedProviderOverview.value?.pool_enabled === false) return t('ui.disable')
  if (selectedProviderOverview.value?.pool_enabled === true) return t('ui.enablef4f0ead')
  return ''
})

function formatDemandEma(value: number | undefined): string {
  const normalized = Number(value ?? 0)
  if (!Number.isFinite(normalized) || normalized <= 0) return '0.0'
  return normalized.toFixed(1)
}

const selectedProviderDemandMetaText = computed(() => {
  const overview = selectedProviderOverview.value
  if (!overview) return ''
  const segments: string[] = []
  const desiredHot = Number(overview.provider_desired_hot ?? 0)
  const hotCount = Number(overview.provider_hot_count ?? 0)
  const inFlight = Number(overview.provider_in_flight ?? 0)
  if (Number.isFinite(desiredHot) && desiredHot > 0) {
    segments.push(`热池 ${hotCount} / ${desiredHot}`)
    segments.push(`EMA ${formatDemandEma(overview.provider_ema_in_flight)}`)
  }
  if (Number.isFinite(inFlight) && inFlight > 0) {
    segments.push(`in-flight ${inFlight}`)
  }
  if (overview.provider_burst_pending) {
    segments.push('补热中')
  }
  return segments.join(' | ')
})

const poolHeaderMetaText = computed(() => {
  return [
    selectedProviderType.value,
    selectedProviderStatusText.value,
    selectedProviderDemandMetaText.value,
  ].filter(Boolean).join(' | ')
})

watch(showDemandMetricsDialog, (open) => {
  if (open) {
    startDemandMetricsPolling()
  } else {
    stopDemandMetricsPolling()
  }
})

watch(selectedProviderId, () => {
  providerDemandMetricSamples.value = []
  if (showDemandMetricsDialog.value) {
    appendDemandMetricSample(selectedProviderOverview.value)
  }
})

watch(selectedProviderOverview, (overview) => {
  appendDemandMetricSample(overview)
})

watch(showAdaptiveHotPoolMetricsButton, (enabled) => {
  if (!enabled && showDemandMetricsDialog.value) {
    showDemandMetricsDialog.value = false
  }
})

const showAccountQuotaColumn = computed(() => {
  return selectedProviderType.value === 'codex'
    || selectedProviderType.value === 'claude_code'
    || selectedProviderType.value === 'gemini_cli'
    || selectedProviderType.value === 'kiro'
    || selectedProviderType.value === 'windsurf'
    || selectedProviderType.value === 'antigravity'
    || selectedProviderType.value === 'grok'
    || selectedProviderType.value === 'chatgpt_web'
    || selectedProviderType.value === 'xai'
})

const desktopColumnWidths = computed(() => {
  if (showAccountQuotaColumn.value) {
    return {
      name: '19%',
      quota: '18%',
      stats: '15%',
      imported: '10%',
      lastUsed: '8%',
      score: '9%',
      status: '7%',
      actions: '14%',
    }
  }
  return {
    name: '31%',
    quota: '0%',
    stats: '15%',
    imported: '11%',
    lastUsed: '11%',
    score: '9%',
    status: '8%',
    actions: '15%',
  }
})

async function selectProvider(
  id: string,
  options: {
    preserveSearch?: boolean
    preserveStatus?: boolean
    preservePagination?: boolean
    cacheTtlMs?: number
  } = {},
) {
  const requestId = ++selectProviderRequestId
  hasHydratedInitialProviderSelection = true
  selectedProviderId.value = id
  selectedProviderData.value = null
  showSchedulingDialog.value = false
  showAdvancedDialog.value = false
  resetPoolKeySelection(true)
  providerDrawerOpen.value = false
  editingKeyDetail.value = null
  showAccountBatchDialog.value = false
  keyBatchEditDialogOpen.value = false
  keyBatchEditKeyIds.value = []
  keyPermissionsDialogOpen.value = false
  keyFormDialogOpen.value = false
  oauthKeyEditDialogOpen.value = false
  proxyDesktopPopoverOpenKeyId.value = null
  proxyMobilePopoverOpenKeyId.value = null
  scoreDesktopPopoverOpenKeyId.value = null
  scoreMobilePopoverOpenKeyId.value = null
  suppressFiltersWatch = true
  if (!options.preservePagination) {
    currentPage.value = 1
  }
  if (!options.preserveSearch) {
    searchQuery.value = ''
  }
  if (!options.preserveStatus) {
    statusFilter.value = 'all'
  }
  suppressFiltersWatch = false
  if (keysSearchDebounceTimer !== null) {
    clearTimeout(keysSearchDebounceTimer)
    keysSearchDebounceTimer = null
  }
  keysSearchPending.value = false
  keysLoadedOnce.value = false
  resetKeyPage(currentPage.value, pageSize.value)
  const keysTask = loadKeys({ cacheTtlMs: options.cacheTtlMs ?? 0 })
  // Provider summary is non-blocking for key list rendering.
  void loadProviderData(id)
  await keysTask
  if (requestId !== selectProviderRequestId) return
}

async function loadProviderData(id: string, options: { preserveOnError?: boolean } = {}) {
  const requestId = ++providerDataRequestId
  try {
    const providerData = await getProvider(id)
    if (requestId !== providerDataRequestId || selectedProviderId.value !== id) return
    if (selectedProviderData.value?.id === providerData.id) {
      Object.assign(selectedProviderData.value, providerData)
    } else {
      selectedProviderData.value = providerData
    }
  } catch {
    if (requestId !== providerDataRequestId || selectedProviderId.value !== id) return
    if (options.preserveOnError) {
      showWarning(t('ui.failedToSyncProviderDetailsCurrentDataHasBeenRetained'))
    } else {
      selectedProviderData.value = null
    }
  }
}

async function refresh() {
  await loadKeys()
}

// --- Keys ---
function createEmptyKeyPage(page = 1, pageSizeValue = 50): PoolKeysPageResponse {
  return { total: 0, page, page_size: pageSizeValue, keys: [] }
}

const keyPage = ref<PoolKeysPageResponse>(createEmptyKeyPage())
const keysLoading = ref(false)
const poolKeySelectionBusy = computed(() => keysLoading.value || keysSearchPending.value)
const keysLoadedOnce = ref(false)
const poolKeyPageItems = computed(() => keyPage.value.keys)
const poolKeyFilteredTotal = computed(() => keyPage.value.total)
const {
  selectedIds: selectedPoolKeyIds,
  selectedIdSet: selectedPoolKeyIdSet,
  selectedCount: selectedKeyCount,
  selectAllFiltered: selectAllFilteredPoolKeys,
  isAllFilteredSelected: isAllFilteredPoolKeysSelected,
  isCurrentPageFullySelected: isCurrentPoolKeyPageFullySelected,
  rememberItems: rememberPoolKeys,
  knownItemsById: knownPoolKeysById,
  resetSelection: resetPoolKeySelection,
  toggleOne: togglePoolKeySelection,
  toggleSelectFiltered: toggleSelectFilteredPoolKeys,
  toggleSelectCurrentPage: toggleCurrentPoolKeyPage,
} = useBatchSelection<PoolKeyDetail>({
  pageItems: poolKeyPageItems,
  filteredTotal: poolKeyFilteredTotal,
  getItemId: key => key.key_id,
})
const selectedKeyCountLabel = computed(() => t('models.management.selectedCount', { count: selectedKeyCount.value }))
const selectedPoolKeys = computed(() => selectedPoolKeyIds.value
  .map(keyId => knownPoolKeysById.value[keyId])
  .filter((key): key is PoolKeyDetail => Boolean(key)))
const selectedOnCurrentPoolKeyPageCount = computed(() => poolKeyPageItems.value
  .filter(key => selectedPoolKeyIdSet.value.has(key.key_id)).length)
const isCurrentPoolKeyPagePartiallySelected = computed(() => (
  selectedOnCurrentPoolKeyPageCount.value > 0
  && !isCurrentPoolKeyPageFullySelected.value
))

watch(poolKeyPageItems, (keys) => rememberPoolKeys(keys), { immediate: true })

function isPoolKeySelected(keyId: string): boolean {
  return selectAllFilteredPoolKeys.value || selectedPoolKeyIdSet.value.has(keyId)
}

function toggleAllFilteredPoolKeys(): void {
  if (poolKeySelectionBusy.value || keyPage.value.total === 0) return
  toggleSelectFilteredPoolKeys(!isAllFilteredPoolKeysSelected.value)
}

function getPoolKeyRowClass(keyId: string): string {
  return [
    keyUiStateMap.value[keyId]?.rowClass || '',
    isPoolKeySelected(keyId) ? 'bg-primary/5' : '',
  ].filter(Boolean).join(' ')
}

const refreshingCurrentPageQuota = ref(false)
const searchQuery = ref(restoredViewState.search)
const statusFilter = ref(restoredViewState.status)
const poolKeySelectionFilters = computed<PoolKeySelectionRequest>(() => {
  const search = searchQuery.value.trim()
  return {
    ...(search ? { search } : {}),
    status: statusFilter.value,
  }
})
const currentPage = ref(restoredViewState.page)
const pageSize = ref(restoredViewState.pageSize)
const sortBy = ref<PoolManagementSortBy | null>(restoredViewState.sortBy)
const sortOrder = ref<PoolManagementSortOrder>(restoredViewState.sortOrder)
const hasPoolKeyFilters = computed(() => searchQuery.value.trim().length > 0 || statusFilter.value !== 'all')
const MANUAL_QUOTA_REFRESH_COOLDOWN_SECONDS = 5 * 60
const refreshingOAuthKeyId = ref<string | null>(null)
const consumingCodexResetCreditKeyId = ref<string | null>(null)
const resettingCycleKeyId = ref<string | null>(null)
const savingProxyKeyId = ref<string | null>(null)
const proxyDesktopPopoverOpenKeyId = ref<string | null>(null)
const proxyMobilePopoverOpenKeyId = ref<string | null>(null)
const scoreDesktopPopoverOpenKeyId = ref<string | null>(null)
const scoreMobilePopoverOpenKeyId = ref<string | null>(null)
const deletingKeyId = ref<string | null>(null)
const togglingKeyId = ref<string | null>(null)
const editingPriorityKeyId = ref<string | null>(null)
const editingPriorityValue = ref<number>(0)
const prioritySavingKeyId = ref<string | null>(null)

const keyPermissionsDialogOpen = ref(false)
const keyFormDialogOpen = ref(false)
const keyBatchEditDialogOpen = ref(false)
const keyBatchEditKeyIds = ref<string[]>([])
const oauthKeyEditDialogOpen = ref(false)
const editingKeyDetail = ref<PoolKeyDetail | null>(null)

function clearPoolKeyFilters() {
  if (!hasPoolKeyFilters.value) return
  resetPoolKeySelection(true)
  if (keysSearchDebounceTimer !== null) {
    clearTimeout(keysSearchDebounceTimer)
    keysSearchDebounceTimer = null
  }
  keysSearchPending.value = false
  suppressFiltersWatch = true
  searchQuery.value = ''
  statusFilter.value = 'all'
  suppressFiltersWatch = false
  if (currentPage.value !== 1) {
    currentPage.value = 1
    return
  }
  void loadKeys({ cacheTtlMs: POOL_KEYS_CACHE_TTL_MS })
}

watch(
  () => getQueryValue('search') ?? '',
  (value) => {
    if (searchQuery.value === value) return
    searchQuery.value = value
  },
  { immediate: true },
)

watch(
  () => readPoolManagementViewState({ status: getQueryValue('status') }).status,
  (value) => {
    if (statusFilter.value === value) return
    suppressFiltersWatch = true
    statusFilter.value = value
    suppressFiltersWatch = false
  },
  { immediate: true },
)

watch(
  () => readPoolManagementViewState({ page: getQueryValue('page') }).page,
  (value) => {
    if (currentPage.value === value) return
    currentPage.value = value
  },
  { immediate: true },
)

watch(
  () => readPoolManagementViewState({ pageSize: getQueryValue('pageSize') }).pageSize,
  (value) => {
    if (pageSize.value === value) return
    pageSize.value = value
  },
  { immediate: true },
)

watch(
  () => readPoolManagementViewState({
    sortBy: getQueryValue('sortBy'),
    sortOrder: getQueryValue('sortOrder'),
  }),
  (value) => {
    if (sortBy.value === value.sortBy && sortOrder.value === value.sortOrder) return
    sortBy.value = value.sortBy
    sortOrder.value = value.sortOrder
  },
  { immediate: true },
)

watch(
  () => getQueryValue('providerId'),
  (value) => {
    if (overviewLoading.value) return
    if (!value || value === selectedProviderId.value) return
    if (!poolProviders.value.some(item => item.provider_id === value)) return
    void selectProvider(value, {
      preserveSearch: true,
      preserveStatus: true,
      preservePagination: true,
      cacheTtlMs: POOL_KEYS_CACHE_TTL_MS,
    })
  },
)

watch(
  [selectedProviderId, searchQuery, statusFilter, currentPage, pageSize, sortBy, sortOrder],
  ([providerId, search, status, page, pageSizeValue, sortByValue, sortOrderValue]) => {
    const nextState: PoolManagementViewState = {
      providerId,
      search,
      status: status as PoolManagementViewState['status'],
      page,
      pageSize: pageSizeValue,
      sortBy: sortByValue,
      sortOrder: sortOrderValue,
      statsMode: 'current_cycle',
    }
    patchQuery(buildPoolManagementQueryPatch(nextState))
    writePoolManagementViewState(nextState, poolManagementViewStorage)
  },
  { immediate: true },
)

type PoolKeyUiState = {
  rowClass: string
  schedulingBadgeLabel: string
  schedulingBadgeVariant: PoolStatusVariant
  schedulingTitle: string
  oauthOrgBadge: ReturnType<typeof getOAuthOrgBadge>
  visibleOAuthState: ReturnType<typeof getOAuthStatusDisplayWithFallback>
  oauthStatusTitle: string
  oauthRefreshButtonTitle: string
  showOAuthRefreshControl: boolean
  canRefreshToken: boolean
  planLabel: string
  planClass: string
  accountQuotaText: string | null
  quotaFallbackText: string | null
  quotaTextClass: string
  importedAtRelative: string
  lastUsedRelative: string
  statsDisplay: PoolStatsDisplay
  mobileTagItems: PoolMobileTagItem[]
  mobileActionIds: PoolMobileActionId[]
  desktopActionIds: PoolMobileActionId[]
}

const quotaProgressMap = computed<Record<string, QuotaProgressItem[]>>(() => {
  const map: Record<string, QuotaProgressItem[]> = {}
  for (const key of keyPage.value.keys) {
    map[key.key_id] = parseQuotaProgressItems(key, selectedProviderType.value, t)
  }
  return map
})

const quotaProgressDisplayMap = computed<Record<string, QuotaProgressDisplayItem[]>>(() => {
  const map: Record<string, QuotaProgressDisplayItem[]> = {}
  for (const key of keyPage.value.keys) {
    map[key.key_id] = (quotaProgressMap.value[key.key_id] || []).map(item => {
      // 倒计时归零表示窗口已越过重置时间点：按“已重置”展示 100%，
      // 不再显示重置前的旧用量文本，与后端读取口径、调度口径保持一致。
      const expired = !item.numericOnly && getQuotaProgressCountdown(item, countdownTick.value)?.isExpired === true
      const remainingPercent = expired ? 100 : item.remainingPercent
      return {
        label: getQuotaProgressLabel(item.label),
        remainingPercent,
        resetText: getQuotaProgressResetDisplayText(item, countdownTick.value),
        meterText: item.numericOnly
          ? item.detail || formatQuotaValue(remainingPercent)
          : getQuotaProgressMeterDisplayText(item, remainingPercent, expired),
        barClass: getQuotaRemainingBarColorByRemaining(remainingPercent),
        meterClass: getQuotaRemainingClassByRemaining(remainingPercent),
        numericOnly: item.numericOnly,
      }
    })
  }
  return map
})

const keyUiStateMap = computed<Record<string, PoolKeyUiState>>(() => {
  const map: Record<string, PoolKeyUiState> = {}

  const tick = countdownTick.value
  const providerType = selectedProviderType.value
  for (const key of keyPage.value.keys) {
    const visibleOAuthState = getVisibleOAuthState(key, tick)
    const oauthOrgBadge = getOAuthOrgBadge(key)
    const accountQuotaText = getGeminiCliAccountCreditsText(key, providerType)
    const quotaFallbackText = getQuotaDisplayText(key, providerType)
    const planType = resolvePoolKeyPlanType(key)
    const canRefreshToken = canRefreshOAuthCredential(key)
    const showOAuthRefreshControl = shouldShowOAuthRefreshControl(key, selectedProviderType.value)

    map[key.key_id] = {
      rowClass: getRowClass(key, tick),
      schedulingBadgeLabel: getSchedulingBadgeLabel(key, tick),
      schedulingBadgeVariant: getSchedulingBadgeVariant(key, tick),
      schedulingTitle: getSchedulingTitle(key, tick),
      oauthOrgBadge,
      visibleOAuthState,
      oauthStatusTitle: visibleOAuthState ? getOAuthStatusTitle(key, tick) : '',
      oauthRefreshButtonTitle: showOAuthRefreshControl ? getOAuthRefreshButtonTitle(key, tick) : '',
      showOAuthRefreshControl,
      canRefreshToken,
      planLabel: planType ? formatOAuthPlanType(planType) : '',
      planClass: planType ? getOAuthPlanTypeClass(planType) : '',
      accountQuotaText,
      quotaFallbackText,
      quotaTextClass: accountQuotaText || quotaFallbackText
        ? getQuotaTextClass(accountQuotaText || quotaFallbackText || '')
        : '',
      importedAtRelative: formatPoolKeyImportedAt(key),
      lastUsedRelative: key.last_used_at ? formatRelativeTime(key.last_used_at) : '-',
      statsDisplay: buildPoolStatsDisplay(key, selectedProviderType.value, 'current_cycle'),
      mobileTagItems: getMobileTagItems(key, tick),
      mobileActionIds: splitPoolMobileActions({
        canDownloadOrCopy: true,
        showRefreshToken: showOAuthRefreshControl,
        canResetCycleStats: canResetCycleStats(key),
        canClearCooldown: Boolean(key.cooldown_reason),
        hasProxy: true,
      }).primary,
      desktopActionIds: [
        ...(key.cooldown_reason ? ['clear_cooldown' as const] : []),
        ...(canResetCycleStats(key) ? ['reset_cycle_stats' as const] : []),
        'permissions',
        'proxy',
        'edit',
        'toggle',
        'delete',
      ],
    }
  }

  return map
})

function getPoolKeyStatsDisplay(key: PoolKeyDetail): PoolStatsDisplay {
  return keyUiStateMap.value[key.key_id]?.statsDisplay
    ?? buildPoolStatsDisplay(key, selectedProviderType.value, 'current_cycle')
}

function isPoolKeyCycleStatsDisplay(key: PoolKeyDetail): boolean {
  return getPoolKeyStatsDisplay(key).kind === 'codex_cycle'
}

function getPoolKeyCycleStatsGroups(key: PoolKeyDetail): PoolCodexCycleStatsGroup[] {
  const display = getPoolKeyStatsDisplay(key)
  return display.kind === 'codex_cycle' ? display.groups : []
}

function getPoolKeyAccountStatsMetrics(key: PoolKeyDetail): PoolStatsMetric[] {
  const display = getPoolKeyStatsDisplay(key)
  return display.kind === 'account_total'
    ? display.metrics
    : buildAccountTotalStatsDisplay(key).metrics
}

const quotaRefreshSupported = computed(() => {
  return selectedProviderType.value === 'codex'
    || selectedProviderType.value === 'claude_code'
    || selectedProviderType.value === 'kiro'
    || selectedProviderType.value === 'gemini_cli'
    || selectedProviderType.value === 'windsurf'
    || selectedProviderType.value === 'antigravity'
    || selectedProviderType.value === 'grok'
    || selectedProviderType.value === 'chatgpt_web'
    || selectedProviderType.value === 'xai'
})

function canResetCycleStats(_key: PoolKeyDetail): boolean {
  return selectedProviderType.value === 'codex' && Boolean(_key.key_id)
}

const refreshCurrentPageLoading = computed(() => {
  return keysLoading.value || refreshingCurrentPageQuota.value
})

function resetKeyPage(page = currentPage.value, pageSizeValue = pageSize.value): void {
  keyPage.value = createEmptyKeyPage(page, pageSizeValue)
}

function refreshOverviewInBackground(): void {
  void loadOverview({ silent: true })
}

function clampActiveKeyCount(current: unknown, total: unknown, delta: number): number {
  const currentValue = Number(current)
  const nextValue = Math.max(0, (Number.isFinite(currentValue) ? currentValue : 0) + delta)
  const totalValue = Number(total)
  if (!Number.isFinite(totalValue)) return nextValue
  return Math.min(Math.max(0, totalValue), nextValue)
}

function isManualInactiveReason(reason: { code?: string; source?: string }): boolean {
  const code = String(reason.code || '').trim().toLowerCase()
  return code === 'inactive' || code === 'manual_disabled'
}

function applyPoolKeyActiveState(key: PoolKeyDetail, nextStatus: boolean): void {
  const previousStatus = key.is_active
  key.is_active = nextStatus

  if (nextStatus) {
    const remainingReasons = (key.scheduling_reasons ?? []).filter(
      reason => !isManualInactiveReason(reason),
    )
    key.scheduling_reasons = remainingReasons
    const remainingBlockingReason = remainingReasons.find(reason => reason.blocking)
    if (remainingBlockingReason) {
      key.scheduling_reason = remainingBlockingReason.code
      key.scheduling_label = remainingBlockingReason.label
      key.scheduling_status = remainingBlockingReason.code === 'cooldown' ? 'degraded' : 'blocked'
    } else if (key.cooldown_reason) {
      key.scheduling_reason = 'cooldown'
      key.scheduling_label = '冷却中'
      key.scheduling_status = 'degraded'
    } else {
      key.scheduling_reason = 'available'
      key.scheduling_label = '可用'
      key.scheduling_status = 'available'
    }
  } else {
    key.scheduling_label = '已禁用'
    key.scheduling_status = 'blocked'
    key.scheduling_reason = 'inactive'
    key.scheduling_reasons = [{
      code: 'inactive',
      get label() { return t('ui.disabledbc5a87a') },
      blocking: true,
      source: 'manual',
      ttl_seconds: null,
      detail: null,
    }]
  }

  if (previousStatus === nextStatus) return
  const delta = nextStatus ? 1 : -1
  const overview = poolProviders.value.find(item => item.provider_id === selectedProviderId.value)
  if (overview) {
    overview.active_keys = clampActiveKeyCount(overview.active_keys, overview.total_keys, delta)
  }
  if (selectedProviderData.value) {
    selectedProviderData.value.active_keys = clampActiveKeyCount(
      selectedProviderData.value.active_keys,
      selectedProviderData.value.total_keys,
      delta,
    )
  }
}

function applyQuotaRefreshResultToCurrentPage(result: Awaited<ReturnType<typeof refreshProviderQuota>>): void {
  keyPage.value.keys = mergePoolKeyQuotaSnapshots(keyPage.value.keys, result.results)
}

function getCodexResetCredits(key: PoolKeyDetail) {
  if (getQuotaSnapshotProviderType(key, selectedProviderType.value) !== 'codex') return null

  const snapshot = key.status_snapshot?.quota?.reset_credits
  const snapshotUpdatedAt = key.status_snapshot?.quota?.updated_at
  const snapshotDisplay = snapshot
    ? {
        ...(typeof snapshotUpdatedAt === 'number' ? { updated_at: snapshotUpdatedAt } : {}),
        reset_credits: snapshot,
      }
    : null
  return mergeCodexQuotaDisplays(snapshotDisplay, key.upstream_metadata?.codex)?.reset_credits ?? null
}

function getCodexCredentialGeneration(key: PoolKeyDetail): string | null | undefined {
  const codex = key.upstream_metadata?.codex
  return codex && typeof codex === 'object'
    ? codex.credential_generation?.trim() || null
    : undefined
}

function getPendingCodexResetCreditIdempotencyKey(key: PoolKeyDetail): string | null {
  const codex = key.upstream_metadata?.codex
  const serverReservation = getCodexResetCreditReservationIdempotencyKey(codex)
  if (serverReservation) return serverReservation
  const generation = getCodexCredentialGeneration(key)
  return generation === undefined
    ? null
    : readPendingCodexResetCreditIdempotencyKey(key.key_id, generation)
}

function getClaudeCodeResetCredits(key: PoolKeyDetail) {
  if (getQuotaSnapshotProviderType(key, selectedProviderType.value) !== 'claude_code') return null
  return key.status_snapshot?.quota?.reset_credits
    ?? key.upstream_metadata?.claude_code?.reset_credits
    ?? null
}

function getCodexResetCreditCountText(key: PoolKeyDetail): string | null {
  const claudeCredits = getClaudeCodeResetCredits(key)
  if (claudeCredits) {
    const claudeCount = getCodexResetCreditAvailableCount(claudeCredits)
    return claudeCount === null ? null : formatCodexResetCreditCount(claudeCount)
  }
  const count = getCodexResetCreditAvailableCount(getCodexResetCredits(key))
  return count === null && !getPendingCodexResetCreditIdempotencyKey(key)
    ? null
    : formatCodexResetCreditCount(count)
}

function getCodexResetCreditItemTexts(key: PoolKeyDetail): string[] {
  return getVisibleCodexResetCreditItems(
    getClaudeCodeResetCredits(key) ?? getCodexResetCredits(key),
    undefined,
    3,
  )
    .map(item => `${item.displayKey} ${formatCodexResetCreditExpiresAt(item.expiresAt)}`)
}

function canConsumeCodexResetCredit(key: PoolKeyDetail): boolean {
  return getQuotaSnapshotProviderType(key, selectedProviderType.value) === 'codex'
    && getCodexCredentialGeneration(key) !== undefined
    && (getPendingCodexResetCreditIdempotencyKey(key) !== null
      || (getCodexResetCreditAvailableCount(getCodexResetCredits(key)) ?? 0) > 0)
    && consumingCodexResetCreditKeyId.value === null
}

async function handleConsumeCodexResetCredit(key: PoolKeyDetail): Promise<void> {
  if (!canConsumeCodexResetCredit(key)) return
  const generation = getCodexCredentialGeneration(key)
  if (generation === undefined) return
  const pendingIdempotencyKey = getPendingCodexResetCreditIdempotencyKey(key)
  const confirmed = await confirm({
    get title() { return t('ui.confirmCodexQuotaReset') },
    message: pendingIdempotencyKey
      ? '将继续确认上次尚未完成的 Codex 重置请求。'
      : '将消耗 1 次 Codex 重置机会，完成后自动刷新账号额度。',
    get confirmText() { return t('ui.confirmReset') },
    get cancelText() { return t('common.cancel') },
    variant: 'warning',
  })
  if (!confirmed) return

  consumingCodexResetCreditKeyId.value = key.key_id
  try {
    const idempotencyKey = pendingIdempotencyKey
      || readPendingCodexResetCreditIdempotencyKey(key.key_id, generation)
      || createCodexResetCreditIdempotencyKey()
    rememberPendingCodexResetCreditIdempotencyKey(key.key_id, idempotencyKey, generation)
    const result = await consumeCodexResetCredit(key.key_id, {
      idempotency_key: idempotencyKey,
      expected_credential_generation: generation,
    })
    clearPendingCodexResetCreditIdempotencyKeyForOutcome(key.key_id, result.outcome)
    keyPage.value.keys = mergePoolKeyQuotaSnapshots(keyPage.value.keys, [{
      key_id: result.key_id,
      key_name: key.key_name,
      status: result.refresh_status === 'success' ? 'success' : result.status as 'success',
      metadata: result.metadata,
      quota_snapshot: result.quota_snapshot,
    }])
    if (result.outcome === 'reset' || result.outcome === 'already_redeemed') {
      success(t('ui.codexQuotaResetUsedAccountQuotaRefreshed'))
    } else {
      showWarning(result.message || t('ui.resetRequestProcessedCheckTheLatestQuota'))
    }
  } catch (err: unknown) {
    const responseData = typeof err === 'object' && err !== null && 'response' in err
      ? (err as { response?: { data?: Record<string, unknown> } }).response?.data
      : undefined
    if (responseData?.outcome === 'credential_changed') {
      clearPendingCodexResetCreditIdempotencyKey(key.key_id)
    } else if (typeof responseData?.active_idempotency_key === 'string') {
      rememberPendingCodexResetCreditIdempotencyKey(
        key.key_id,
        responseData.active_idempotency_key,
        generation,
      )
    }
    showError(parseApiError(err, t('ui.failedToUseCodexQuotaReset')))
  } finally {
    consumingCodexResetCreditKeyId.value = null
  }
}

function normalizeQuotaUpdatedAt(raw: number | null | undefined): number | null {
  const value = Number(raw ?? 0)
  if (!Number.isFinite(value) || value <= 0) return null
  if (value > 1_000_000_000_000) {
    return Math.floor(value / 1000)
  }
  return Math.floor(value)
}

const currentPageQuotaRefreshStats = computed(() => {
  void countdownTick.value
  const seen = new Set<string>()
  const eligibleIds: string[] = []
  let cooledDownCount = 0
  let minRemainingSeconds = 0
  const nowSeconds = Math.floor(Date.now() / 1000)
  for (const key of keyPage.value.keys) {
    const id = String(key.key_id || '').trim()
    if (!id || seen.has(id)) continue
    seen.add(id)
    const updatedAt = normalizeQuotaUpdatedAt(key.quota_updated_at ?? null)
    if (updatedAt == null) {
      eligibleIds.push(id)
      continue
    }
    const remaining = MANUAL_QUOTA_REFRESH_COOLDOWN_SECONDS - (nowSeconds - updatedAt)
    if (remaining > 0) {
      cooledDownCount += 1
      if (minRemainingSeconds <= 0 || remaining < minRemainingSeconds) {
        minRemainingSeconds = remaining
      }
      continue
    }
    eligibleIds.push(id)
  }
  return {
    total: seen.size,
    eligibleIds,
    cooledDownCount,
    minRemainingSeconds,
  }
})

async function refreshCurrentPageQuotaInBackground(
  options: { silent?: boolean; reloadAfter?: boolean | 'silent' } = {},
): Promise<boolean> {
  if (!selectedProviderId.value || !quotaRefreshSupported.value) return false

  const providerId = selectedProviderId.value
  const quotaStats = currentPageQuotaRefreshStats.value
  if (quotaStats.eligibleIds.length === 0) {
    if (!options.silent && quotaStats.total > 0 && quotaStats.cooledDownCount > 0) {
      const waitText = quotaStats.minRemainingSeconds > 0
        ? formatTTL(quotaStats.minRemainingSeconds)
        : '稍后'
      showWarning(`当前页额度均在冷却中，请 ${waitText} 后再试`)
    }
    return false
  }

  if (refreshingCurrentPageQuota.value) {
    return false
  }

  refreshingCurrentPageQuota.value = true
  try {
    const result = await refreshProviderQuota(providerId, quotaStats.eligibleIds)
    applyQuotaRefreshResultToCurrentPage(result)
    const successCount = Number(result.success || 0)
    const failedCount = Number(result.failed || 0)
    const skippedCount = Math.max(quotaStats.total - quotaStats.eligibleIds.length, 0)

    // 刷新当前页数据，展示最新额度与状态
    if (selectedProviderId.value === providerId && options.reloadAfter !== false) {
      await loadKeys({ silent: options.reloadAfter === 'silent' })
    }

    if (!options.silent) {
      const skippedText = skippedCount > 0 ? `，冷却跳过 ${skippedCount}` : ''
      const firstFailureMessage = result.results.find(item => item.status !== 'success')?.message?.trim()
      if (successCount === 0 && failedCount > 0 && firstFailureMessage) {
        showError(`当前页额度刷新失败：${firstFailureMessage}${skippedText}`)
      } else {
        success(`当前页额度刷新完成：成功 ${successCount}，失败 ${failedCount}${skippedText}`)
      }
    }
    return true
  } catch (err) {
    showError(parseApiError(err, t('ui.failedToRefreshQuotaForThisPage')))
    return false
  } finally {
    refreshingCurrentPageQuota.value = false
  }
}

const refreshButtonTitle = computed(() => {
  if (refreshCurrentPageLoading.value) return t('ui.refreshing')
  if (!selectedProviderId.value) return t('ui.refresh')
  if (!quotaRefreshSupported.value) return t('ui.refreshData')

  const quotaStats = currentPageQuotaRefreshStats.value
  if (quotaStats.total === 0) return t('ui.refreshDataAndQuota')
  if (quotaStats.eligibleIds.length === 0 && quotaStats.cooledDownCount > 0) {
    const waitText = quotaStats.minRemainingSeconds > 0
      ? formatTTL(quotaStats.minRemainingSeconds)
      : '稍后'
    return `刷新数据（额度冷却 ${waitText}）`
  }
  if (quotaStats.cooledDownCount > 0) {
    return `刷新数据和额度（可刷新 ${quotaStats.eligibleIds.length}/${quotaStats.total}）`
  }
  return t('ui.refreshDataAndQuota')
})

async function refreshCurrentPage() {
  const quotaDidReload = await refreshCurrentPageQuotaInBackground({ reloadAfter: true })
  if (!quotaDidReload) {
    await refresh()
  }
}

async function loadKeys(options: { cacheTtlMs?: number, silent?: boolean } = {}) {
  if (!selectedProviderId.value) return
  const requestId = ++keysRequestId
  const providerId = selectedProviderId.value
  const page = currentPage.value
  const pageSizeValue = pageSize.value
  const search = searchQuery.value || undefined
  const status = statusFilter.value
  const sortByValue = sortBy.value || undefined
  if (!options.silent) {
    keysLoading.value = true
  }
  try {
    const nextPage = await listPoolKeys(providerId, {
      page,
      page_size: pageSizeValue,
      search,
      status,
      sort_by: sortByValue || undefined,
      sort_order: sortByValue ? sortOrder.value : undefined,
    }, {
      cacheTtlMs: options.cacheTtlMs ?? 0,
    })
    if (requestId !== keysRequestId || selectedProviderId.value !== providerId) return
    const resolvedPage = resolvePoolManagementPageAfterLoad({
      requestedPage: page,
      pageSize: pageSizeValue,
      total: nextPage.total,
    })
    if (resolvedPage !== page) {
      currentPage.value = resolvedPage
      return
    }
    keyPage.value = nextPage
    keysLoadedOnce.value = true
  } catch (err) {
    if (requestId !== keysRequestId || selectedProviderId.value !== providerId) return
    if (!options.silent) {
      resetKeyPage(page, pageSizeValue)
      keysLoadedOnce.value = true
      showError(parseApiError(err))
    } else {
      showWarning(parseApiError(err, t('ui.failedToSyncAccounts')))
    }
  } finally {
    if (requestId === keysRequestId) {
      keysLoading.value = false
    }
  }
}

watch(currentPage, () => {
  void loadKeys({ cacheTtlMs: POOL_KEYS_CACHE_TTL_MS })
})

watch(pageSize, () => {
  void loadKeys({ cacheTtlMs: POOL_KEYS_CACHE_TTL_MS })
})

watch(statusFilter, () => {
  if (suppressFiltersWatch) return
  resetPoolKeySelection(true)
  if (keysSearchDebounceTimer !== null) {
    clearTimeout(keysSearchDebounceTimer)
    keysSearchDebounceTimer = null
  }
  keysSearchPending.value = false
  currentPage.value = 1
  void loadKeys({ cacheTtlMs: POOL_KEYS_CACHE_TTL_MS })
})

watch([sortBy, sortOrder], () => {
  if (currentPage.value !== 1) {
    currentPage.value = 1
    return
  }
  void loadKeys({ cacheTtlMs: POOL_KEYS_CACHE_TTL_MS })
})

watch(searchQuery, () => {
  if (suppressFiltersWatch) return
  resetPoolKeySelection(true)
  currentPage.value = 1
  if (keysSearchDebounceTimer !== null) {
    clearTimeout(keysSearchDebounceTimer)
  }
  keysSearchPending.value = true
  keysSearchDebounceTimer = window.setTimeout(() => {
    keysSearchDebounceTimer = null
    keysSearchPending.value = false
    void loadKeys({ cacheTtlMs: POOL_KEYS_CACHE_TTL_MS })
  }, 300)
})

function normalizeAuthTypeForEdit(key: PoolKeyDetail): EndpointAPIKey['auth_type'] {
  if (isOAuthManagedCredential(key)) return 'oauth'
  if (isServiceAccountCredential(key)) return 'service_account'
  if ((key.auth_type || '').trim().toLowerCase() === 'bearer') return 'bearer'
  return 'api_key'
}

function toEndpointApiKey(key: PoolKeyDetail): EndpointAPIKey {
  const nowIso = new Date().toISOString()
  return {
    id: key.key_id,
    provider_id: selectedProviderId.value || '',
    api_formats: key.api_formats || [],
    api_key_masked: getProviderMaskedSecretLabel(key, selectedProviderType.value),
    auth_type: normalizeAuthTypeForEdit(key),
    auth_type_by_format: key.auth_type_by_format ?? null,
    credential_kind: key.credential_kind ?? null,
    runtime_auth_kind: key.runtime_auth_kind ?? null,
    oauth_managed: key.oauth_managed ?? undefined,
    agent_identity: key.agent_identity ?? undefined,
    oauth_header_auth: key.oauth_header_auth ?? undefined,
    can_refresh_oauth: key.can_refresh_oauth ?? undefined,
    can_export_oauth: key.can_export_oauth ?? undefined,
    can_edit_oauth: key.can_edit_oauth ?? undefined,
    name: key.key_name || '未命名',
    rate_multipliers: key.rate_multipliers ?? null,
    internal_priority: key.internal_priority ?? 50,
    rpm_limit: key.rpm_limit ?? null,
    concurrent_limit: key.concurrent_limit ?? null,
    allowed_models: key.allowed_models ?? null,
    capabilities: key.capabilities ?? null,
    cache_ttl_minutes: key.cache_ttl_minutes ?? 5,
    max_probe_interval_minutes: key.max_probe_interval_minutes ?? 32,
    health_score: key.health_score ?? 1,
    circuit_breaker_open: key.circuit_breaker_open ?? false,
    consecutive_failures: 0,
    request_count: 0,
    success_count: 0,
    error_count: 0,
    success_rate: 0,
    avg_response_time_ms: 0,
    is_active: key.is_active,
    note: key.note || '',
    last_used_at: key.last_used_at || undefined,
    created_at: key.created_at || nowIso,
    updated_at: nowIso,
    auto_fetch_models: key.auto_fetch_models ?? false,
    locked_models: key.locked_models || [],
    model_include_patterns: key.model_include_patterns || [],
    model_exclude_patterns: key.model_exclude_patterns || [],
    oauth_expires_at: key.oauth_expires_at ?? null,
    oauth_email: null,
    oauth_plan_type: key.oauth_plan_type ?? null,
    oauth_account_id: key.oauth_account_id ?? null,
    oauth_account_user_id: key.oauth_account_user_id ?? null,
    oauth_account_name: key.oauth_account_name ?? null,
    oauth_organizations: key.oauth_organizations ?? [],
    oauth_temporary: key.oauth_temporary ?? false,
    oauth_invalid_at: key.oauth_invalid_at ?? null,
    oauth_invalid_reason: key.oauth_invalid_reason ?? null,
    status_snapshot: key.status_snapshot ?? null,
    proxy: key.proxy ?? null,
  }
}

const editingKey = computed<EndpointAPIKey | null>(() => {
  if (!editingKeyDetail.value) return null
  return toEndpointApiKey(editingKeyDetail.value)
})

function sortCurrentPageKeysByPriority() {
  keyPage.value.keys = [...keyPage.value.keys].sort((a, b) => {
    const pa = Number(a.internal_priority ?? 50)
    const pb = Number(b.internal_priority ?? 50)
    if (pa !== pb) return pa - pb
    return (a.created_at || '').localeCompare(b.created_at || '')
  })
}

function handleTableSort(payload: { key: string, direction: PoolManagementSortOrder }) {
  if (payload.key !== 'imported_at' && payload.key !== 'last_used_at' && payload.key !== 'score') return
  sortBy.value = payload.key
  sortOrder.value = payload.direction
}

function startEditInternalPriority(key: PoolKeyDetail) {
  editingPriorityKeyId.value = key.key_id
  editingPriorityValue.value = Number(key.internal_priority ?? 50)
}

function cancelEditInternalPriority() {
  editingPriorityKeyId.value = null
  editingPriorityValue.value = 0
}

async function applyInternalPriority(key: PoolKeyDetail, nextPriority: number) {
  const normalized = Math.max(1, Math.min(999999, Math.floor(nextPriority)))
  if (Number(key.internal_priority ?? 50) === normalized) return

  prioritySavingKeyId.value = key.key_id
  try {
    await updateProviderKey(key.key_id, { internal_priority: normalized })
    key.internal_priority = normalized
    sortCurrentPageKeysByPriority()
    success(t('ui.accountPriorityUpdated'))
  } catch (err) {
    showError(parseApiError(err, t('ui.failedToUpdatePriority')))
  } finally {
    prioritySavingKeyId.value = null
  }
}

async function quickEditInternalPriority(key: PoolKeyDetail) {
  const raw = window.prompt('设置账号优先级（1-999999，数字越小越优先）', String(key.internal_priority ?? 50))
  if (raw === null) return
  const parsed = Number(raw)
  if (!Number.isFinite(parsed)) {
    showWarning(t('ui.enterAValidNumber'))
    return
  }
  await applyInternalPriority(key, parsed)
}

async function finishEditInternalPriority(
  key: PoolKeyDetail,
  event: FocusEvent | KeyboardEvent,
) {
  if (prioritySavingKeyId.value) return
  const target = event.target as HTMLInputElement | null
  const raw = target?.value ?? String(editingPriorityValue.value)
  const parsed = Number(raw)
  const nextPriority = Number.isFinite(parsed) ? parsed : Number(key.internal_priority ?? 50)
  cancelEditInternalPriority()
  await applyInternalPriority(key, nextPriority)
}

function handleEditKey(key: PoolKeyDetail) {
  editingKeyDetail.value = key
  if (canEditOAuthCredential(key)) {
    oauthKeyEditDialogOpen.value = true
  } else {
    keyFormDialogOpen.value = true
  }
}

function handleKeyPermissions(key: PoolKeyDetail) {
  editingKeyDetail.value = key
  keyPermissionsDialogOpen.value = true
}

function openKeyBatchEditDialog(keyIds: string[]): void {
  keyBatchEditKeyIds.value = [...new Set(keyIds)]
  keyBatchEditDialogOpen.value = keyBatchEditKeyIds.value.length > 0
}

function closeKeyBatchEditDialog(): void {
  keyBatchEditDialogOpen.value = false
  keyBatchEditKeyIds.value = []
}

async function handleKeyBatchEditSaved(): Promise<void> {
  resetPoolKeySelection(true)
  await Promise.all([loadKeys({ silent: true }), loadOverview({ silent: true })])
}

async function handleDialogSaved() {
  editingKeyDetail.value = null
  await loadKeys({ silent: true })
}

function closeKeyFormDialog() {
  keyFormDialogOpen.value = false
  editingKeyDetail.value = null
}

function closeOAuthEditDialog() {
  oauthKeyEditDialogOpen.value = false
  editingKeyDetail.value = null
}

function closeKeyPermissionsDialog() {
  keyPermissionsDialogOpen.value = false
  editingKeyDetail.value = null
}

function getKeyProxyNodeName(key: PoolKeyDetail): string | null {
  if (!key.proxy?.node_id) return null
  const node = proxyNodesStore.nodes.find(n => n.id === key.proxy?.node_id)
  return node ? node.name : `${key.proxy.node_id.slice(0, 8)}...`
}

function handleScoreDesktopPopoverToggle(keyId: string, open: boolean) {
  scoreDesktopPopoverOpenKeyId.value = open ? keyId : null
  if (open) {
    scoreMobilePopoverOpenKeyId.value = null
  }
}

function handleScoreMobilePopoverToggle(keyId: string, open: boolean) {
  scoreMobilePopoverOpenKeyId.value = open ? keyId : null
  if (open) {
    scoreDesktopPopoverOpenKeyId.value = null
  }
}

function handleProxyDesktopPopoverToggle(keyId: string, open: boolean) {
  proxyDesktopPopoverOpenKeyId.value = open ? keyId : null
  if (open) {
    proxyMobilePopoverOpenKeyId.value = null
  }
  if (open) {
    proxyNodesStore.ensureLoaded()
  }
}

function handleProxyMobilePopoverToggle(keyId: string, open: boolean) {
  proxyMobilePopoverOpenKeyId.value = open ? keyId : null
  if (open) {
    proxyDesktopPopoverOpenKeyId.value = null
  }
  if (open) {
    proxyNodesStore.ensureLoaded()
  }
}

async function setKeyProxy(key: PoolKeyDetail, nodeId: string) {
  savingProxyKeyId.value = key.key_id
  try {
    await updateProviderKey(key.key_id, {
      proxy: { node_id: nodeId, enabled: true },
    })
    key.proxy = { node_id: nodeId, enabled: true }
    proxyDesktopPopoverOpenKeyId.value = null
    proxyMobilePopoverOpenKeyId.value = null
    success(t('ui.proxyNodeSet'))
  } catch (err) {
    showError(parseApiError(err, t('ui.failedToSetProxy')))
  } finally {
    savingProxyKeyId.value = null
  }
}

async function clearKeyProxy(key: PoolKeyDetail) {
  savingProxyKeyId.value = key.key_id
  try {
    await updateProviderKey(key.key_id, { proxy: null })
    key.proxy = null
    proxyDesktopPopoverOpenKeyId.value = null
    proxyMobilePopoverOpenKeyId.value = null
    success(t('ui.accountProxyClearedProviderLevelProxyWillBeUsed'))
  } catch (err) {
    showError(parseApiError(err, t('ui.failedToClearProxy')))
  } finally {
    savingProxyKeyId.value = null
  }
}

async function handleDeleteKey(key: PoolKeyDetail) {
  const confirmed = await confirm({
    get title() { return t('ui.deleteAccount') },
    message: `确定要删除账号 "${key.key_name || key.key_id.slice(0, 8)}" 吗？`,
    get confirmText() { return t('ui.delete') },
    variant: 'destructive',
  })
  if (!confirmed) return

  deletingKeyId.value = key.key_id
  try {
    await deleteEndpointKey(key.key_id)
    success(t('ui.accountDeleted'))
    togglePoolKeySelection(key.key_id, false)
    // 乐观更新：直接从本地列表移除，避免等待网络重载
    keyPage.value.keys = keyPage.value.keys.filter(k => k.key_id !== key.key_id)
    keyPage.value.total = Math.max(0, keyPage.value.total - 1)
    // 当前页已空且不是第一页时，自动跳转到前一页
    if (keyPage.value.keys.length === 0 && currentPage.value > 1) {
      currentPage.value--
    }
    refreshOverviewInBackground()
  } catch (err) {
    showError(parseApiError(err, t('ui.failedToDeleteAccount')))
  } finally {
    deletingKeyId.value = null
  }
}

async function copyFullKey(key: PoolKeyDetail) {
  try {
    const result = await revealEndpointKey(key.key_id)
    let textToCopy = ''

    if (result.auth_type === 'service_account' && result.auth_config) {
      textToCopy = typeof result.auth_config === 'string'
        ? result.auth_config
        : JSON.stringify(result.auth_config, null, 2)
    } else if (result.auth_type === 'oauth') {
      textToCopy = result.refresh_token || ''
    } else {
      textToCopy = result.api_key || ''
    }

    if (!textToCopy) {
      showError(t('ui.noContentAvailableToCopy'))
      return
    }

    await copyToClipboard(textToCopy)
  } catch (err) {
    showError(parseApiError(err, t('ui.failedToRetrieveKey')))
  }
}

async function downloadRefreshToken(key: PoolKeyDetail) {
  try {
    const data = await exportKey(key.key_id)
    const providerType = selectedProviderType.value || 'unknown'
    const email = typeof data.email === 'string' ? data.email : ''
    const safeName = (email || key.key_name || key.key_id.slice(0, 8)).replace(/[^a-zA-Z0-9_\-@.]/g, '_')

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `aether_${providerType}_${safeName}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  } catch (err) {
    showError(parseApiError(err, t('ui.exportFailed')))
  }
}

async function handleRefreshOAuth(key: PoolKeyDetail) {
  if (refreshingOAuthKeyId.value) return

  refreshingOAuthKeyId.value = key.key_id
  try {
    const result = await refreshProviderOAuth(key.key_id)
    const refreshedExpiresAt = typeof result.expires_at === 'number' ? result.expires_at : null
    const target = keyPage.value.keys.find(k => k.key_id === key.key_id)
    if (target) {
      target.oauth_expires_at = refreshedExpiresAt
    }
    await loadKeys({ silent: true })
    if (refreshedExpiresAt != null) {
      const reloadedTarget = keyPage.value.keys.find(k => k.key_id === key.key_id)
      if (
        reloadedTarget
        && (typeof reloadedTarget.oauth_expires_at !== 'number'
          || reloadedTarget.oauth_expires_at < refreshedExpiresAt)
      ) {
        reloadedTarget.oauth_expires_at = refreshedExpiresAt
      }
    }
    const refreshedKey = keyPage.value.keys.find(k => k.key_id === key.key_id) ?? null
    const feedback = getOAuthRefreshFeedback({
      accountStateRecheckAttempted: result.account_state_recheck_attempted,
      accountStateRecheckError: result.account_state_recheck_error,
      snapshot: refreshedKey,
    })
    if (feedback.tone === 'warning') {
      showWarning(feedback.message)
    } else {
      success(feedback.message)
    }
  } catch (err) {
    showError(parseApiError(err, t('ui.failedToRefreshToken')))
    await Promise.all([loadKeys({ silent: true }), loadOverview({ silent: true })])
  } finally {
    refreshingOAuthKeyId.value = null
  }
}

// --- Actions ---
async function clearCooldown(keyId: string) {
  if (!selectedProviderId.value) return
  try {
    const res = await clearPoolCooldown(selectedProviderId.value, keyId)
    success(res.message)
    const key = keyPage.value.keys.find(item => item.key_id === keyId)
    if (key) {
      key.cooldown_reason = null
      key.cooldown_ttl_seconds = null
      if (key.scheduling_reason === 'cooldown') {
        key.scheduling_reason = key.is_active ? 'available' : 'inactive'
        key.scheduling_status = key.is_active ? 'available' : 'blocked'
        key.scheduling_label = key.is_active ? '可用' : '已禁用'
      }
      key.scheduling_reasons = key.scheduling_reasons?.filter(
        item => item.code !== 'cooldown',
      )
    }
    await Promise.all([loadKeys({ silent: true }), loadOverview({ silent: true })])
  } catch (err) {
    showError(parseApiError(err))
  }
}

async function handleResetCycleStats(key: PoolKeyDetail) {
  if (resettingCycleKeyId.value || !canResetCycleStats(key)) return

  const confirmed = await confirm({
    get title() { return t('ui.resetCycleStatistics') },
    message: `确定要将账号 "${key.key_name || key.key_id.slice(0, 8)}" 的 5H / 周统计从当前时间重新开始计算吗？`,
    get confirmText() { return t('ui.reset') },
  })
  if (!confirmed) return

  resettingCycleKeyId.value = key.key_id
  try {
    const result = await resetProviderKeyCycleStats(key.key_id)
    success(result.message || t('ui.cycleStatisticsReset'))
    if (key.status_snapshot?.quota?.windows) {
      const resetAt = Number(result.reset_at)
      key.status_snapshot = {
        ...key.status_snapshot,
        quota: {
          ...key.status_snapshot.quota,
          windows: resetCodexCycleUsageWindows(key.status_snapshot.quota.windows, resetAt),
        },
      }
    }
    await loadKeys({ silent: true })
  } catch (err) {
    showError(parseApiError(err, t('ui.failedToResetCycleStatistics')))
  } finally {
    resettingCycleKeyId.value = null
  }
}

async function toggleKeyActive(key: PoolKeyDetail) {
  if (togglingKeyId.value) return
  togglingKeyId.value = key.key_id
  try {
    const nextStatus = !key.is_active
    await updateProviderKey(key.key_id, { is_active: nextStatus })
    applyPoolKeyActiveState(key, nextStatus)
    await Promise.all([loadKeys({ silent: true }), loadOverview({ silent: true })])
    success(nextStatus ? t('ui.accountEnabled') : t('ui.accountDisabled4a3a19a'))
  } catch (err) {
    showError(parseApiError(err))
  } finally {
    togglingKeyId.value = null
  }
}

// --- Dialogs ---
const showImportDialog = ref(false)
const showSchedulingDialog = ref(false)
const showAdvancedDialog = ref(false)
const providerDrawerOpen = ref(false)
const providerDrawerMounted = ref(false)
const providerEditDialogOpen = ref(false)
const providerToEdit = ref<ProviderWithEndpointsSummary | null>(null)
const showAccountBatchDialog = ref(false)
const pendingAccountBatchAction = ref<PoolBatchActionValue | null>(null)
const togglingProviderStatus = ref(false)

async function ensureSelectedProviderDetail(): Promise<boolean> {
  const providerId = selectedProviderId.value
  if (!providerId) return false
  if (selectedProviderData.value?.id !== providerId) {
    await loadProviderData(providerId, { preserveOnError: true })
  }
  if (selectedProviderData.value?.id === providerId) return true
  showWarning(t('ui.providerDetailsHaveNotLoadedSchedulingSettingsCannotBeEdited'))
  return false
}

async function openSchedulingDialog() {
  if (await ensureSelectedProviderDetail()) {
    showSchedulingDialog.value = true
  }
}

async function openAdvancedDialog() {
  if (await ensureSelectedProviderDetail()) {
    showAdvancedDialog.value = true
  }
}

function openAccountBatchDialog(action: PoolBatchActionValue = 'refresh_quota'): void {
  if (!selectedProviderId.value || selectedKeyCount.value === 0) return
  pendingAccountBatchAction.value = action
  showAccountBatchDialog.value = true
}

watch(showAccountBatchDialog, (open) => {
  if (!open) pendingAccountBatchAction.value = null
})

function openProviderDrawer(): void {
  prefetchProviderDetailDrawer()
  if (!selectedProviderId.value) return
  providerDrawerMounted.value = true
  providerDrawerOpen.value = true
}

async function handleProviderDrawerRefresh(): Promise<void> {
  const providerId = selectedProviderId.value
  if (!providerId) return

  await Promise.all([
    loadKeys({ silent: true }),
    loadOverview({ silent: true }),
    loadProviderData(providerId, { preserveOnError: true }),
  ])
  resetPoolKeySelection(true)
}

async function openProviderEditDialog(provider?: ProviderWithEndpointsSummary): Promise<void> {
  const providerId = provider?.id || selectedProviderId.value
  if (!providerId) return

  try {
    const latest = await getProvider(providerId)
    if (selectedProviderId.value !== providerId) return
    if (selectedProviderData.value?.id === latest.id) {
      Object.assign(selectedProviderData.value, latest)
      providerToEdit.value = selectedProviderData.value
    } else {
      selectedProviderData.value = latest
      providerToEdit.value = latest
    }
  } catch (err) {
    if (selectedProviderId.value !== providerId) return
    const fallbackProvider = provider ?? selectedProviderData.value
    if (!fallbackProvider) {
      showError(parseApiError(err, t('ui.failedToRefreshProviderStatus')))
      return
    }
    providerToEdit.value = fallbackProvider
  }

  providerEditDialogOpen.value = true
}

async function handleProviderEditSaved(updatedProvider: ProviderWithEndpointsSummary): Promise<void> {
  if (selectedProviderId.value === updatedProvider.id) {
    if (selectedProviderData.value) {
      Object.assign(selectedProviderData.value, updatedProvider)
      providerToEdit.value = selectedProviderData.value
    } else {
      selectedProviderData.value = updatedProvider
      providerToEdit.value = updatedProvider
    }
  }
  providerEditDialogOpen.value = false
  await loadOverview({ silent: true })
}

async function toggleSelectedProviderStatus(provider?: ProviderWithEndpointsSummary): Promise<void> {
  if (togglingProviderStatus.value) return
  const providerId = selectedProviderId.value
  const current = provider?.id === providerId ? provider : selectedProviderData.value
  if (!providerId || !current) return

  const nextStatus = !current.is_active
  if (!nextStatus) {
    const confirmed = await confirm({
      get title() { return t('ui.disableProvider') },
      message: `禁用后该提供商（${current.name}）将不再参与调度，是否继续？`,
      get confirmText() { return t('ui.confirmDisabling') },
      variant: 'destructive',
    })
    if (!confirmed) return
  }

  togglingProviderStatus.value = true
  try {
    const updated = await updateProvider(providerId, { is_active: nextStatus })
    Object.assign(current, updated)
    if (selectedProviderId.value === providerId && selectedProviderData.value !== current) {
      if (selectedProviderData.value) {
        Object.assign(selectedProviderData.value, updated)
      } else {
        selectedProviderData.value = updated
      }
    }
    success(nextStatus ? t('ui.providerEnabled') : t('ui.providerDisabled'))
    await loadOverview({ silent: true })
  } catch (err) {
    showError(parseApiError(err, nextStatus ? t('ui.failedToEnableProvider') : t('ui.failedToDisableProvider')))
  } finally {
    togglingProviderStatus.value = false
  }
}

async function handleAccountBatchChanged(): Promise<void> {
  resetPoolKeySelection(true)
  await Promise.all([loadKeys({ silent: true }), loadOverview({ silent: true })])
}

async function handleAccountDialogSaved() {
  showImportDialog.value = false
  await Promise.all([loadKeys({ silent: true }), loadOverview({ silent: true })])
  // 导入账号后补一次静默额度刷新，避免新账号在列表里暂无额度信息
  await refreshCurrentPageQuotaInBackground({ silent: true, reloadAfter: 'silent' })
}

// --- Init ---
onMounted(() => {
  startCountdownTimer()
  void loadSchedulingPresetMetas({ cacheTtlMs: POOL_SCHEDULING_PRESETS_CACHE_TTL_MS })
  void loadOverview({ cacheTtlMs: POOL_OVERVIEW_CACHE_TTL_MS })
  scheduleProviderDetailDrawerPrefetch()
})

onBeforeUnmount(() => {
  cancelProviderDetailDrawerPrefetch()
  stopDemandMetricsPolling()
  if (keysSearchDebounceTimer !== null) {
    clearTimeout(keysSearchDebounceTimer)
    keysSearchDebounceTimer = null
  }
  keysSearchPending.value = false
  overviewRequestId += 1
  selectProviderRequestId += 1
  providerDataRequestId += 1
  keysRequestId += 1
})
</script>
