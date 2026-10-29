<template>
  <div class="space-y-6 pb-8">
    <Card class="overflow-hidden">
      <div class="px-5 py-4 border-b border-border/60 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h3 class="text-base font-semibold">
            {{ t('nav.walletManagement') }}
          </h3>
          <p class="text-xs text-muted-foreground mt-1">
            {{ t('ui.manageWalletLedgerRefundApprovalsTopUpOrdersAndPayment') }}
          </p>
        </div>
      </div>

      <div class="px-5 py-5">
        <div
          v-if="locationId"
          class="mb-5 space-y-3 rounded-xl border border-border/60 p-4"
          data-testid="wallet-location"
          aria-live="polite"
        >
          <div class="flex flex-wrap items-center justify-between gap-2">
            <span class="text-sm font-semibold">{{ locationLoading ? t('ui.locatingTarget') : locationError ? t('ui.failedToLocateTarget') : t('ui.targetLocated') }}</span>
            <div class="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                @click="copyLocationId"
              >
                {{ t('ui.copyId') }}
              </Button>
              <Button
                v-if="locationError"
                variant="outline"
                size="sm"
                @click="retryLocation"
              >
                {{ t('ui.retryLocatingTarget') }}
              </Button>
            </div>
          </div>
          <p class="break-all font-mono text-xs text-muted-foreground">
            {{ locationId }}
          </p>
          <p
            v-if="locationError"
            role="alert"
            class="text-sm text-rose-600"
          >
            {{ locationError }}
          </p>
          <div
            v-if="locationTarget?.kind === 'wallets'"
            class="grid gap-2 text-sm sm:grid-cols-4"
          >
            <span>{{ ownerDisplayName(locationTarget.item.owner_name, locationTarget.item.owner_type) }}</span>
            <span>{{ t('ui.topUpBalance') }} {{ formatCurrency(locationTarget.item.recharge_balance) }}</span>
            <span>{{ t('ui.grantBalance') }} {{ formatCurrency(locationTarget.item.gift_balance) }}</span>
            <span>{{ walletStatusLabel(locationTarget.item.status) }}</span>
          </div>
          <div
            v-if="locationTarget?.kind === 'orders'"
            class="grid gap-2 text-sm sm:grid-cols-4"
          >
            <span class="break-all">{{ t('ui.order') }} {{ locationTarget.item.order_no }}</span>
            <span>{{ formatCurrency(locationTarget.item.amount_usd) }}</span>
            <span>{{ paymentStatusLabel(locationTarget.item.status) }}</span>
            <span>{{ t('ui.refundsAndProcessingHolds') }} {{ formatCurrency(locationTarget.item.refunded_amount_usd) }}</span>
          </div>
          <Button
            v-if="locationTarget?.kind === 'refunds'"
            variant="outline"
            size="sm"
            @click="openRefundDrawer(locationTarget.item)"
          >
            {{ t('ui.viewTargetRefund') }}
          </Button>
          <Button
            v-if="locationTarget?.kind === 'ledger'"
            variant="outline"
            size="sm"
            @click="openLedgerDrawer(locationTarget.item)"
          >
            {{ t('ui.viewTargetTransaction') }}
          </Button>
          <p class="text-xs text-muted-foreground">
            {{ t('ui.targetDetailsAreIndependentOfPaginationTheGlobalListIs') }}
          </p>
        </div>
        <Tabs v-model="activeTab">
          <TabsList class="tabs-button-list grid w-full max-w-[960px] grid-cols-5">
            <TabsTrigger value="ledger">
              {{ t('ui.walletLedger') }}
            </TabsTrigger>
            <TabsTrigger value="orders">
              {{ t('ui.topUpOrder') }}
            </TabsTrigger>
            <TabsTrigger value="refunds">
              {{ t('ui.refundApprovals') }}
            </TabsTrigger>
            <TabsTrigger value="callbacks">
              {{ t('ui.callbackLogs') }}
            </TabsTrigger>
            <TabsTrigger value="redeem_codes">
              {{ t('ui.redeemCodeabbb615') }}
            </TabsTrigger>
          </TabsList>

          <TabsContent
            value="ledger"
            class="mt-5 space-y-4"
          >
            <div class="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
              <div class="flex flex-wrap items-center gap-2">
                <Select v-model="ledgerCategoryFilter">
                  <SelectTrigger class="w-[170px]">
                    <SelectValue :placeholder="t('ui.primaryCategory')" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">
                      {{ t('ui.allCategories') }}
                    </SelectItem>
                    <SelectItem value="recharge">
                      {{ t('ui.topUp') }}
                    </SelectItem>
                    <SelectItem value="gift">
                      {{ t('ui.giftCredit') }}
                    </SelectItem>
                    <SelectItem value="adjust">
                      {{ t('ui.adjustment') }}
                    </SelectItem>
                    <SelectItem value="refund">
                      {{ t('ui.refundf1c91e1') }}
                    </SelectItem>
                  </SelectContent>
                </Select>

                <Select v-model="ledgerReasonFilter">
                  <SelectTrigger class="w-[180px]">
                    <SelectValue :placeholder="t('ui.secondaryCategory')" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">
                      {{ t('ui.allSecondaryCategories') }}
                    </SelectItem>
                    <SelectItem
                      v-for="option in ledgerReasonOptions"
                      :key="option.value"
                      :value="option.value"
                    >
                      {{ option.label }}
                    </SelectItem>
                  </SelectContent>
                </Select>

                <Select v-model="ledgerOwnerFilter">
                  <SelectTrigger class="w-[170px]">
                    <SelectValue :placeholder="t('ui.ownerType')" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">
                      {{ t('ui.allOwnership') }}
                    </SelectItem>
                    <SelectItem value="user">
                      {{ t('ui.userWallet') }}
                    </SelectItem>
                    <SelectItem value="api_key">
                      {{ t('ui.standaloneKeyWallet') }}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div class="flex items-center justify-between gap-3">
                <div class="text-sm text-muted-foreground">
                  {{ t('ui.total76e547a') }} {{ ledgerTotal }} {{ t('ui.itemsf004f1d') }}
                </div>
                <RefreshButton
                  :loading="loadingLedger"
                  @click="loadLedger"
                />
              </div>
            </div>

            <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
              <div class="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{{ t('ui.time') }}</TableHead>
                      <TableHead>{{ t('ui.owner1e9b3dc') }}</TableHead>
                      <TableHead>{{ t('ui.typeba40014') }}</TableHead>
                      <TableHead>{{ t('ui.amount') }}</TableHead>
                      <TableHead>{{ t('ui.balanceChanges') }}</TableHead>
                      <TableHead>{{ t('ui.description') }}</TableHead>
                      <TableHead class="text-right">
                        {{ t('ui.actionsed31fbb') }}
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="tx in ledgerItems"
                      :key="tx.id"
                      class="hover:bg-muted/20"
                    >
                      <TableCell class="text-xs text-muted-foreground whitespace-nowrap">
                        {{ formatDateTime(tx.created_at) }}
                      </TableCell>
                      <TableCell class="min-w-[180px]">
                        <div class="font-medium text-sm">
                          {{ ownerDisplayName(tx.owner_name, tx.owner_type) }}
                        </div>
                        <div class="text-xs text-muted-foreground mt-1 flex items-center gap-2">
                          <span>{{ ownerTypeLabel(tx.owner_type) }}</span>
                          <Badge
                            v-if="tx.wallet_status"
                            variant="outline"
                            class="text-[10px]"
                          >
                            {{ walletStatusLabel(tx.wallet_status) }}
                          </Badge>
                        </div>
                      </TableCell>
                      <TableCell>
                        <div class="space-y-1">
                          <Badge
                            variant="outline"
                            class="font-mono"
                          >
                            {{ walletTransactionCategoryLabel(tx.category) }}
                          </Badge>
                          <div class="text-[11px] text-muted-foreground">
                            {{ walletTransactionReasonLabel(tx.reason_code) }}
                          </div>
                        </div>
                      </TableCell>
                      <TableCell :class="tx.amount >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'">
                        {{ tx.amount >= 0 ? '+' : '' }}{{ tx.amount.toFixed(4) }}
                      </TableCell>
                      <TableCell class="text-xs tabular-nums whitespace-nowrap">
                        <div>{{ tx.balance_before.toFixed(4) }} → {{ tx.balance_after.toFixed(4) }}</div>
                        <div
                          v-if="tx.recharge_balance_before !== null && tx.recharge_balance_before !== undefined && tx.gift_balance_before !== null && tx.gift_balance_before !== undefined"
                          class="text-[11px] text-muted-foreground mt-0.5"
                        >
                          {{ t('ui.paid') }} {{ Number(tx.recharge_balance_before).toFixed(4) }}→{{ Number(tx.recharge_balance_after ?? 0).toFixed(4) }} {{ t('ui.gift') }} {{ Number(tx.gift_balance_before).toFixed(4) }}→{{ Number(tx.gift_balance_after ?? 0).toFixed(4) }}
                        </div>
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground max-w-[280px] truncate">
                        {{ tx.description || '-' }}
                      </TableCell>
                      <TableCell class="text-right">
                        <Button
                          size="sm"
                          variant="outline"
                          @click="openLedgerDrawer(tx)"
                        >
                          {{ t('ui.details') }}
                        </Button>
                      </TableCell>
                    </TableRow>
                    <TableRow v-if="!loadingLedger && ledgerItems.length === 0">
                      <TableCell
                        colspan="7"
                        class="py-12"
                      >
                        <EmptyState
                          :title="t('ui.noWalletLedgerEntries')"
                          :description="t('ui.noWalletTransactionsMatchTheCurrentFilters')"
                        />
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </div>
            </div>

            <Pagination
              :current="ledgerPage"
              :total="ledgerTotal"
              :page-size="ledgerPageSize"
              @update:current="handleLedgerPageChange"
              @update:page-size="handleLedgerPageSizeChange"
            />
          </TabsContent>

          <TabsContent
            value="refunds"
            class="mt-5 space-y-4"
          >
            <div class="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
              <div class="flex flex-wrap items-center gap-2">
                <Select v-model="refundStatusFilter">
                  <SelectTrigger class="w-[170px]">
                    <SelectValue :placeholder="t('ui.refundStatus')" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">
                      {{ t('ui.allStatuses') }}
                    </SelectItem>
                    <SelectItem value="pending_approval">
                      {{ t('ui.pendingApproval') }}
                    </SelectItem>
                    <SelectItem value="approved">
                      {{ t('ui.approved') }}
                    </SelectItem>
                    <SelectItem value="processing">
                      {{ t('ui.processing694b71b') }}
                    </SelectItem>
                    <SelectItem value="succeeded">
                      {{ t('site.home.status.completed') }}
                    </SelectItem>
                    <SelectItem value="failed">
                      {{ t('ui.failed7e9ef4d') }}
                    </SelectItem>
                  </SelectContent>
                </Select>

                <Select v-model="refundOwnerFilter">
                  <SelectTrigger class="w-[170px]">
                    <SelectValue :placeholder="t('ui.ownerType')" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">
                      {{ t('ui.allOwnership') }}
                    </SelectItem>
                    <SelectItem value="user">
                      {{ t('ui.userWallet') }}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div class="flex items-center justify-between gap-3">
                <div class="text-sm text-muted-foreground">
                  {{ t('ui.total76e547a') }} {{ refundTotal }} {{ t('ui.itemsf004f1d') }}
                </div>
                <RefreshButton
                  :loading="loadingRefunds"
                  @click="loadRefunds"
                />
              </div>
            </div>

            <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
              <div class="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{{ t('ui.owner1e9b3dc') }}</TableHead>
                      <TableHead>{{ t('ui.refundId') }}</TableHead>
                      <TableHead>{{ t('ui.amount') }}</TableHead>
                      <TableHead>{{ t('ui.mode') }}</TableHead>
                      <TableHead>{{ t('ui.status6320b4a') }}</TableHead>
                      <TableHead>{{ t('ui.reasonf029d9c') }}</TableHead>
                      <TableHead>{{ t('ui.requestedAt') }}</TableHead>
                      <TableHead class="text-right">
                        {{ t('ui.actionsed31fbb') }}
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="refund in refundItems"
                      :key="refund.id"
                      class="hover:bg-muted/20"
                    >
                      <TableCell class="min-w-[180px]">
                        <div class="font-medium text-sm">
                          {{ ownerDisplayName(refund.owner_name, refund.owner_type) }}
                        </div>
                        <div class="text-xs text-muted-foreground mt-1 flex items-center gap-2">
                          <span>{{ ownerTypeLabel(refund.owner_type) }}</span>
                          <Badge
                            v-if="refund.wallet_status"
                            variant="outline"
                            class="text-[10px]"
                          >
                            {{ walletStatusLabel(refund.wallet_status) }}
                          </Badge>
                        </div>
                      </TableCell>
                      <TableCell class="font-mono text-xs whitespace-nowrap">
                        {{ refund.refund_no }}
                      </TableCell>
                      <TableCell class="tabular-nums whitespace-nowrap">
                        {{ formatCurrency(refund.amount_usd) }}
                      </TableCell>
                      <TableCell>
                        {{ refundModeLabel(refund.refund_mode) }}
                      </TableCell>
                      <TableCell>
                        <Badge :variant="refundStatusBadge(refund.status)">
                          {{ refundStatusLabel(refund.status) }}
                        </Badge>
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground max-w-[240px] truncate">
                        {{ refund.reason || refund.failure_reason || '-' }}
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground whitespace-nowrap">
                        {{ formatDateTime(refund.created_at) }}
                      </TableCell>
                      <TableCell class="text-right">
                        <div class="flex justify-end gap-2">
                          <Button
                            size="sm"
                            variant="outline"
                            @click="openRefundDrawer(refund)"
                          >
                            {{ t('ui.approve') }}
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                    <TableRow v-if="!loadingRefunds && refundItems.length === 0">
                      <TableCell
                        colspan="8"
                        class="py-12"
                      >
                        <EmptyState
                          :title="t('ui.noRefundRequests')"
                          :description="t('ui.noRefundsMatchTheSelectedFilters')"
                        />
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </div>
            </div>

            <Pagination
              :current="refundPage"
              :total="refundTotal"
              :page-size="refundPageSize"
              @update:current="handleRefundPageChange"
              @update:page-size="handleRefundPageSizeChange"
            />
          </TabsContent>

          <TabsContent
            value="orders"
            class="mt-5 space-y-4"
          >
            <div class="flex flex-wrap items-center gap-2">
              <Select v-model="orderStatusFilter">
                <SelectTrigger class="w-[180px]">
                  <SelectValue :placeholder="t('ui.orderStatus')" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">
                    {{ t('ui.allStatuses') }}
                  </SelectItem>
                  <SelectItem value="pending">
                    {{ t('ui.awaitingPayment') }}
                  </SelectItem>
                  <SelectItem value="paid">
                    {{ t('ui.paid0a52f8a') }}
                  </SelectItem>
                  <SelectItem value="credited">
                    {{ t('ui.credited') }}
                  </SelectItem>
                  <SelectItem value="failed">
                    {{ t('ui.paymentFailed') }}
                  </SelectItem>
                  <SelectItem value="expired">
                    {{ t('ui.expired') }}
                  </SelectItem>
                </SelectContent>
              </Select>

              <Select v-model="orderMethodFilter">
                <SelectTrigger class="w-[180px]">
                  <SelectValue :placeholder="t('ui.paymentMethod')" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">
                    {{ t('ui.allMethods') }}
                  </SelectItem>
                  <SelectItem value="alipay">
                    {{ t('ui.alipay') }}
                  </SelectItem>
                  <SelectItem value="wechat">
                    {{ t('ui.wechatPay') }}
                  </SelectItem>
                  <SelectItem value="admin_manual">
                    {{ t('ui.manualTopUp') }}
                  </SelectItem>
                  <SelectItem value="card_code">
                    {{ t('ui.topUpCard') }}
                  </SelectItem>
                  <SelectItem value="gift_code">
                    {{ t('ui.giftCard') }}
                  </SelectItem>
                  <SelectItem value="card_recharge">
                    {{ t('ui.codeTopUp') }}
                  </SelectItem>
                </SelectContent>
              </Select>

              <RefreshButton
                :loading="loadingOrders"
                @click="loadOrders"
              />
            </div>

            <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
              <div class="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{{ t('ui.orderNumber') }}</TableHead>
                      <TableHead>{{ t('ui.walletName') }}</TableHead>
                      <TableHead>{{ t('ui.amount') }}</TableHead>
                      <TableHead>{{ t('ui.paymentMethod') }}</TableHead>
                      <TableHead>{{ t('ui.status6320b4a') }}</TableHead>
                      <TableHead>{{ t('ui.createdAt') }}</TableHead>
                      <TableHead class="text-right">
                        {{ t('ui.actionsed31fbb') }}
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="order in orders"
                      :key="order.id"
                    >
                      <TableCell class="font-mono text-xs whitespace-nowrap">
                        {{ order.order_no }}
                      </TableCell>
                      <TableCell class="min-w-[180px]">
                        <div class="text-sm font-medium">
                          {{ orderWalletName(order.wallet_id) }}
                        </div>
                        <div class="text-xs text-muted-foreground mt-1">
                          {{ orderWalletTypeLabel(order.wallet_id) }}
                        </div>
                      </TableCell>
                      <TableCell class="tabular-nums">
                        {{ formatCurrency(order.amount_usd) }}
                      </TableCell>
                      <TableCell>{{ paymentMethodLabel(order.payment_method) }}</TableCell>
                      <TableCell>
                        <Badge :variant="paymentStatusBadge(order.status)">
                          {{ paymentStatusLabel(order.status) }}
                        </Badge>
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground whitespace-nowrap">
                        {{ formatDateTime(order.created_at) }}
                      </TableCell>
                      <TableCell class="text-right">
                        <div class="flex justify-end gap-2">
                          <Button
                            v-if="canCreditOrder(order.status)"
                            size="sm"
                            @click="openCreditDialog(order)"
                          >
                            {{ t('ui.credit') }}
                          </Button>
                          <Button
                            v-if="canExpireOrder(order.status)"
                            size="sm"
                            variant="outline"
                            :disabled="submittingOrderAction"
                            @click="expireOrder(order.id)"
                          >
                            {{ t('ui.expired97399bd') }}
                          </Button>
                          <Button
                            v-if="canFailOrder(order.status)"
                            size="sm"
                            variant="destructive"
                            :disabled="submittingOrderAction"
                            @click="failOrder(order.id)"
                          >
                            {{ t('ui.failed28384d7') }}
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                    <TableRow v-if="!loadingOrders && orders.length === 0">
                      <TableCell
                        colspan="7"
                        class="py-10"
                      >
                        <EmptyState
                          :title="t('ui.noPaymentOrders')"
                          :description="t('ui.noDataUnderTheCurrentFilters')"
                        />
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </div>
            </div>

            <Pagination
              :current="orderPage"
              :total="orderTotal"
              :page-size="orderPageSize"
              @update:current="handleOrderPageChange"
              @update:page-size="handleOrderPageSizeChange"
            />
          </TabsContent>

          <TabsContent
            value="callbacks"
            class="mt-5 space-y-4"
          >
            <div class="flex items-center gap-2">
              <Select v-model="callbackMethodFilter">
                <SelectTrigger class="w-[180px]">
                  <SelectValue :placeholder="t('ui.paymentMethod')" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">
                    {{ t('ui.allMethods') }}
                  </SelectItem>
                  <SelectItem value="alipay">
                    {{ t('ui.alipay') }}
                  </SelectItem>
                  <SelectItem value="wechat">
                    {{ t('ui.wechatPay') }}
                  </SelectItem>
                </SelectContent>
              </Select>
              <RefreshButton
                :loading="loadingCallbacks"
                @click="loadCallbacks"
              />
            </div>

            <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
              <div class="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{{ t('ui.callbackKey') }}</TableHead>
                      <TableHead>{{ t('ui.orderNumber') }}</TableHead>
                      <TableHead>{{ t('ui.method') }}</TableHead>
                      <TableHead>{{ t('ui.signatureVerification') }}</TableHead>
                      <TableHead>{{ t('ui.status6320b4a') }}</TableHead>
                      <TableHead>{{ t('ui.time') }}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="callback in callbacks"
                      :key="callback.id"
                    >
                      <TableCell class="font-mono text-xs">
                        {{ callback.callback_key }}
                      </TableCell>
                      <TableCell class="font-mono text-xs whitespace-nowrap">
                        {{ callback.order_no || '-' }}
                      </TableCell>
                      <TableCell>{{ paymentMethodLabel(callback.payment_method) }}</TableCell>
                      <TableCell>
                        <Badge :variant="callback.signature_valid ? 'success' : 'destructive'">
                          {{ callback.signature_valid ? t('ui.passed') : t('ui.failed28384d7') }}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <Badge :variant="callbackStatusBadge(callback.status)">
                          {{ callbackStatusLabel(callback.status) }}
                        </Badge>
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground whitespace-nowrap">
                        {{ formatDateTime(callback.created_at) }}
                      </TableCell>
                    </TableRow>
                    <TableRow v-if="!loadingCallbacks && callbacks.length === 0">
                      <TableCell
                        colspan="6"
                        class="py-10"
                      >
                        <EmptyState
                          :title="t('ui.noCallbackLogs')"
                          :description="t('ui.noDataUnderTheCurrentFilters')"
                        />
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </div>
            </div>

            <Pagination
              :current="callbackPage"
              :total="callbackTotal"
              :page-size="callbackPageSize"
              @update:current="handleCallbackPageChange"
              @update:page-size="handleCallbackPageSizeChange"
            />
          </TabsContent>

          <TabsContent
            value="redeem_codes"
            class="mt-5"
          >
            <WalletRedeemCodesTab ref="redeemCodesTab" />
          </TabsContent>
        </Tabs>
      </div>
    </Card>

    <WalletLedgerDetailDrawer
      :open="showLedgerDrawer"
      :ledger="currentLedger"
      @close="closeLedgerDrawer"
    />

    <WalletRefundReviewDrawer
      :open="showRefundDrawer"
      :refund="currentRefund"
      :submitting="submittingRefundAction"
      @close="closeRefundDrawer"
      @process="processRefund"
      @fail="submitFailRefund"
      @complete="submitCompleteRefund"
    />

    <WalletOrderCreditDialog
      :open="showCreditDialog"
      :order="currentOrder"
      :submitting="submittingOrderAction"
      @close="showCreditDialog = false"
      @submit="submitCreditOrder"
    />
  </div>
  <RefundReferralReview
    ref="referralReview"
    :active="showRefundDrawer"
    :context-key="currentRefund?.id || ''"
  />
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import RefundReferralReview from '@/features/wallet/components/RefundReferralReview.vue'
import WalletLedgerDetailDrawer from '@/features/wallet/components/WalletLedgerDetailDrawer.vue'
import WalletRefundReviewDrawer from '@/features/wallet/components/WalletRefundReviewDrawer.vue'
import type { WalletRefundCompletePayload } from '@/features/wallet/components/WalletRefundReviewDrawer.vue'
import WalletOrderCreditDialog from '@/features/wallet/components/WalletOrderCreditDialog.vue'
import type { WalletOrderCreditPayload } from '@/features/wallet/components/WalletOrderCreditDialog.vue'
import WalletRedeemCodesTab from '@/features/wallet/components/WalletRedeemCodesTab.vue'
import { useWalletLocation } from '@/features/wallet/useWalletLocation'
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import {
  Badge,
  Button,
  Card,
  Pagination,
  RefreshButton,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/components/ui'
import { EmptyState } from '@/components/common'
import {
  adminWalletApi,
  type AdminGlobalRefund,
  type AdminLedgerTransaction,
} from '@/api/admin-wallets'
import {
  adminPaymentsApi,
  type PaymentCallbackRecord,
} from '@/api/admin-payments'
import type { PaymentOrder, RefundRequest } from '@/api/wallet'
import { parseApiError } from '@/utils/errorParser'
import { useToast } from '@/composables/useToast'
import { log } from '@/utils/logger'
import {
  callbackStatusBadge,
  callbackStatusLabel,
  formatWalletCurrency as formatCurrency,
  paymentMethodLabel,
  paymentStatusBadge,
  paymentStatusLabel,
  refundModeLabel,
  refundStatusBadge,
  refundStatusLabel,
  walletStatusLabel,
  walletTransactionCategoryLabel,
  walletTransactionReasonLabel,
  ownerDisplayName,
  ownerTypeLabel,
} from '@/features/wallet/utils/walletDisplay'
import {
  canCreditOrder,
  canExpireOrder,
  canFailOrder,
  canFailRefund,
} from '@/features/wallet/utils/refundActions'
import { formatDateTime } from '@/utils/format'
import { useClipboard } from '@/composables/useClipboard'

type WalletManagementTab = 'ledger' | 'refunds' | 'orders' | 'callbacks' | 'redeem_codes'
type LedgerCategory = 'recharge' | 'gift' | 'adjust' | 'refund'
type LedgerReasonOption = {
  value: string
  label: string
  category: LedgerCategory
}

const referralReview = ref<InstanceType<typeof RefundReferralReview> | null>(null)
const redeemCodesTab = ref<InstanceType<typeof WalletRedeemCodesTab> | null>(null)

const LEDGER_REASON_OPTIONS: LedgerReasonOption[] = [
  { value: 'topup_admin_manual', get label() { return t('ui.manualTopUp') }, category: 'recharge' },
  { value: 'topup_gateway', get label() { return t('ui.paymentTopUp') }, category: 'recharge' },
  { value: 'topup_card_code', get label() { return t('ui.codeTopUp') }, category: 'recharge' },
  { value: 'gift_initial', get label() { return t('ui.initialGiftCredit') }, category: 'gift' },
  { value: 'gift_campaign', get label() { return t('ui.promotionalGiftCredit') }, category: 'gift' },
  { value: 'gift_expire_reclaim', get label() { return t('ui.giftCreditReversal') }, category: 'gift' },
  { value: 'adjust_admin', get label() { return t('ui.manualAdjustment') }, category: 'adjust' },
  { value: 'adjust_system', get label() { return t('ui.systemAdjustment') }, category: 'adjust' },
  { value: 'refund_out', get label() { return t('ui.refundDebit') }, category: 'refund' },
  { value: 'refund_revert', get label() { return t('ui.refundRecredit') }, category: 'refund' },
]

const { success, error: showError } = useToast()
const { copyToClipboard } = useClipboard()
const route = useRoute()

const activeTab = ref<WalletManagementTab>('ledger')
const activeTabLoadPromises = new Map<WalletManagementTab, Promise<void>>()

const loadingLedger = ref(false)
const loadingRefunds = ref(false)
const loadingOrders = ref(false)
const loadingCallbacks = ref(false)
const submittingRefundAction = ref(false)
const submittingOrderAction = ref(false)

const ledgerItems = ref<AdminLedgerTransaction[]>([])
const ledgerTotal = ref(0)
const ledgerPage = ref(1)
const ledgerPageSize = ref(20)
const ledgerCategoryFilter = ref('all')
const ledgerReasonFilter = ref('all')
const ledgerOwnerFilter = ref('all')
const ledgerReasonOptions = computed(() => {
  if (ledgerCategoryFilter.value === 'all') {
    return LEDGER_REASON_OPTIONS
  }
  return LEDGER_REASON_OPTIONS.filter((option) => option.category === ledgerCategoryFilter.value)
})

const refundItems = ref<AdminGlobalRefund[]>([])
const refundTotal = ref(0)
const refundPage = ref(1)
const refundPageSize = ref(20)
const refundStatusFilter = ref('all')
const refundOwnerFilter = ref('all')

const orders = ref<PaymentOrder[]>([])
const orderTotal = ref(0)
const orderPage = ref(1)
const orderPageSize = ref(20)
const orderStatusFilter = ref('all')
const orderMethodFilter = ref('all')

const callbacks = ref<PaymentCallbackRecord[]>([])
const callbackTotal = ref(0)
const callbackPage = ref(1)
const callbackPageSize = ref(20)
const callbackMethodFilter = ref('all')

const walletMetaMap = ref<Record<string, { ownerName: string; ownerType: 'user' | 'api_key' }>>({})

const showLedgerDrawer = ref(false)
const showRefundDrawer = ref(false)
const currentLedger = ref<AdminLedgerTransaction | null>(null)
const currentRefund = ref<AdminGlobalRefund | null>(null)

const showCreditDialog = ref(false)
const currentOrder = ref<PaymentOrder | null>(null)

const {
  target: locationTarget, loading: locationLoading, error: locationError,
  targetId: locationId, retry: retryLocation,
} = useWalletLocation(() => route.query, () => {
  closeLedgerDrawer()
  closeRefundDrawer()
  showCreditDialog.value = false
}, (target) => {
  if (target.kind === 'refunds') openRefundDrawer(target.item)
  if (target.kind === 'ledger') openLedgerDrawer(target.item)
})

function copyLocationId() {
  void copyToClipboard(locationId.value)
}

watch([ledgerCategoryFilter, ledgerReasonFilter, ledgerOwnerFilter], () => {
  ledgerPage.value = 1
  void loadLedger()
})

watch(ledgerCategoryFilter, () => {
  if (ledgerReasonFilter.value === 'all') {
    return
  }
  const valid = ledgerReasonOptions.value.some((option) => option.value === ledgerReasonFilter.value)
  if (!valid) {
    ledgerReasonFilter.value = 'all'
  }
})

watch([refundStatusFilter, refundOwnerFilter], () => {
  refundPage.value = 1
  void loadRefunds()
})

watch([orderStatusFilter, orderMethodFilter], () => {
  orderPage.value = 1
  void loadOrders()
})

watch(callbackMethodFilter, () => {
  callbackPage.value = 1
  void loadCallbacks()
})

watch(
  () => route.query.tab,
  (tab) => {
    const tabValue = Array.isArray(tab) ? tab[0] : tab
    if (tabValue === 'wallets') {
      activeTab.value = 'ledger'
    } else if (isValidTab(tabValue)) {
      activeTab.value = tabValue
    }
  },
  { immediate: true }
)

onMounted(async () => {
  await loadActiveTab()
})

watch(activeTab, () => {
  void loadActiveTab()
})

function isValidTab(tab: unknown): tab is WalletManagementTab {
  return tab === 'ledger' || tab === 'refunds' || tab === 'orders' || tab === 'callbacks' || tab === 'redeem_codes'
}

function loadActiveTab(): Promise<void> {
  const tab = activeTab.value
  const existing = activeTabLoadPromises.get(tab)
  if (existing) return existing

  const request = (async () => {
    switch (tab) {
      case 'refunds':
        return loadRefunds()
      case 'orders':
        await Promise.all([loadOrders(), loadWalletMetaMap()])
        return
      case 'callbacks':
        return loadCallbacks()
      case 'redeem_codes':
        return redeemCodesTab.value?.load() ?? Promise.resolve()
      default:
        return loadLedger()
    }
  })().finally(() => {
    if (activeTabLoadPromises.get(tab) === request) activeTabLoadPromises.delete(tab)
  })
  activeTabLoadPromises.set(tab, request)
  return request
}

async function loadWalletMetaMap() {
  try {
    const wallets = await adminWalletApi.listAllWallets(undefined, { cacheTtlMs: 30_000 })
    walletMetaMap.value = wallets.reduce<Record<string, { ownerName: string; ownerType: 'user' | 'api_key' }>>(
      (acc, wallet) => {
        const ownerName =
          wallet.owner_name || (wallet.owner_type === 'user' ? t('ui.unnamedUser') : t('ui.unnamedKey'))
        acc[wallet.id] = {
          ownerName,
          ownerType: wallet.owner_type,
        }
        return acc
      },
      {}
    )
  } catch (error) {
    log.error('加载钱包名称映射失败:', error)
  }
}

async function loadLedger() {
  loadingLedger.value = true
  try {
    const offset = (ledgerPage.value - 1) * ledgerPageSize.value
    const resp = await adminWalletApi.listLedger({
      category: ledgerCategoryFilter.value !== 'all' ? ledgerCategoryFilter.value : undefined,
      reason_code: ledgerReasonFilter.value !== 'all' ? ledgerReasonFilter.value : undefined,
      owner_type: ledgerOwnerFilter.value !== 'all' ? ledgerOwnerFilter.value : undefined,
      limit: ledgerPageSize.value,
      offset,
    })
    ledgerItems.value = resp.items
    ledgerTotal.value = resp.total
  } catch (error) {
    log.error('加载全局资金流水失败:', error)
    showError(parseApiError(error, '加载全局资金流水失败'))
  } finally {
    loadingLedger.value = false
  }
}

async function loadRefunds() {
  loadingRefunds.value = true
  try {
    const offset = (refundPage.value - 1) * refundPageSize.value
    const resp = await adminWalletApi.listGlobalRefunds({
      status: refundStatusFilter.value !== 'all' ? refundStatusFilter.value : undefined,
      owner_type: refundOwnerFilter.value === 'user' ? 'user' : undefined,
      limit: refundPageSize.value,
      offset,
    })
    refundItems.value = resp.items
    refundTotal.value = resp.total
    if (currentRefund.value) {
      syncCurrentRefund(currentRefund.value.id)
    }
  } catch (error) {
    log.error('加载全局退款列表失败:', error)
    showError(parseApiError(error, '加载全局退款列表失败'))
  } finally {
    loadingRefunds.value = false
  }
}

async function loadOrders() {
  loadingOrders.value = true
  try {
    const offset = (orderPage.value - 1) * orderPageSize.value
    const resp = await adminPaymentsApi.listOrders({
      status: orderStatusFilter.value !== 'all' ? orderStatusFilter.value : undefined,
      payment_method: orderMethodFilter.value !== 'all' ? orderMethodFilter.value : undefined,
      limit: orderPageSize.value,
      offset,
    })
    orders.value = resp.items
    orderTotal.value = resp.total
  } catch (error) {
    log.error('加载支付订单失败:', error)
    showError(parseApiError(error, '加载支付订单失败'))
  } finally {
    loadingOrders.value = false
  }
}

async function loadCallbacks() {
  loadingCallbacks.value = true
  try {
    const offset = (callbackPage.value - 1) * callbackPageSize.value
    const resp = await adminPaymentsApi.listCallbacks({
      payment_method: callbackMethodFilter.value !== 'all' ? callbackMethodFilter.value : undefined,
      limit: callbackPageSize.value,
      offset,
    })
    callbacks.value = resp.items
    callbackTotal.value = resp.total
  } catch (error) {
    log.error('加载支付回调失败:', error)
    showError(parseApiError(error, '加载支付回调失败'))
  } finally {
    loadingCallbacks.value = false
  }
}

function orderWalletName(walletId: string) {
  return walletMetaMap.value[walletId]?.ownerName || t('ui.unknownWallet')
}

function orderWalletTypeLabel(walletId: string) {
  const ownerType = walletMetaMap.value[walletId]?.ownerType
  if (!ownerType) return t('ui.unknownOwner')
  return ownerType === 'user' ? t('ui.userWallet') : t('ui.standaloneKeyWallet')
}

function openLedgerDrawer(tx: AdminLedgerTransaction) {
  currentLedger.value = tx
  showLedgerDrawer.value = true
}

function closeLedgerDrawer() {
  showLedgerDrawer.value = false
}

function openRefundDrawer(refund: AdminGlobalRefund) {
  currentRefund.value = refund
  showRefundDrawer.value = true
}

function closeRefundDrawer() {
  showRefundDrawer.value = false
}

function syncCurrentRefund(refundId: string) {
  if (currentRefund.value?.id !== refundId) return
  const latest = refundItems.value.find((item) => item.id === refundId)
  if (latest) {
    currentRefund.value = latest
  }
}

function applyRefundUpdate(refundId: string, refund: RefundRequest) {
  if (currentRefund.value?.id === refundId) currentRefund.value = { ...currentRefund.value, ...refund }
  const target = locationTarget.value
  if (target?.kind === 'refunds' && target.item.id === refundId) {
    locationTarget.value = { kind: 'refunds', item: { ...target.item, ...refund } }
  }
}

async function processRefund(refund: AdminGlobalRefund) {
  const walletId = refund.wallet_id
  const refundId = refund.id
  const canReject = canFailRefund(refund)
  submittingRefundAction.value = true
  try {
    const review = await referralReview.value?.review(walletId, refundId, 'process', canReject)
    if (review?.decision === 'reject') { await nextTick(); document.querySelector<HTMLInputElement>('#refund-fail-reason')?.focus(); return }
    if (review?.decision !== 'approve') return
    const response = await adminWalletApi.processRefund(walletId, refundId, { referral_shortfall_confirmation: review.token })
    applyRefundUpdate(refundId, response.refund)
    success(t('ui.refundIsNowProcessing'))
    await Promise.all([loadRefunds(), loadLedger()])
    syncCurrentRefund(refundId)
  } catch (error) {
    log.error('处理退款失败:', error)
    showError(parseApiError(error, '处理退款失败'))
  } finally {
    submittingRefundAction.value = false
  }
}

async function submitFailRefund(reason: string) {
  if (!currentRefund.value) return
  if (!reason) {
    showError(t('ui.enterAReasonForRejection'))
    return
  }

  const walletId = currentRefund.value.wallet_id
  const refundId = currentRefund.value.id
  submittingRefundAction.value = true
  try {
    const response = await adminWalletApi.failRefund(walletId, refundId, {
      reason,
    })
    applyRefundUpdate(refundId, response.refund)
    success(t('ui.refundRejected'))
    await Promise.all([loadRefunds(), loadLedger()])
    syncCurrentRefund(refundId)
  } catch (error) {
    log.error('驳回退款失败:', error)
    showError(parseApiError(error, '驳回退款失败'))
  } finally {
    submittingRefundAction.value = false
  }
}

async function submitCompleteRefund(payload: WalletRefundCompletePayload) {
  if (!currentRefund.value) return

  const walletId = currentRefund.value.wallet_id
  const refundId = currentRefund.value.id
  const canReject = canFailRefund(currentRefund.value)
  submittingRefundAction.value = true
  try {
    const review = await referralReview.value?.review(walletId, refundId, 'complete', canReject)
    if (review?.decision === 'reject') { await nextTick(); document.querySelector<HTMLInputElement>('#refund-fail-reason')?.focus(); return }
    if (review?.decision !== 'approve') return
    const response = await adminWalletApi.completeRefund(walletId, refundId, {
      referral_shortfall_confirmation: review.token,
      ...payload,
    })
    applyRefundUpdate(refundId, response.refund)
    success(response.referral_reversal
      ? `退款已完成；该订单累计冲回 $${response.referral_reversal.reversed_amount_usd.toFixed(2)}，当前待冲回 $${response.referral_reversal.pending_reversal_amount_usd.toFixed(2)}`
      : t('ui.refundCompleted'))
    await Promise.all([loadRefunds(), loadLedger()])
    syncCurrentRefund(refundId)
  } catch (error) {
    log.error('完成退款失败:', error)
    showError(parseApiError(error, '完成退款失败'))
  } finally {
    submittingRefundAction.value = false
  }
}

function openCreditDialog(order: PaymentOrder) {
  currentOrder.value = order
  showCreditDialog.value = true
}

async function submitCreditOrder(payload: WalletOrderCreditPayload) {
  if (!currentOrder.value) return
  submittingOrderAction.value = true
  try {
    await adminPaymentsApi.creditOrder(currentOrder.value.id, payload)
    success(t('ui.orderCreditedManually'))
    showCreditDialog.value = false
    await Promise.all([loadOrders(), loadLedger(), loadWalletMetaMap()])
  } catch (error) {
    log.error('手动到账失败:', error)
    showError(parseApiError(error, '手动到账失败'))
  } finally {
    submittingOrderAction.value = false
  }
}

async function expireOrder(orderId: string) {
  submittingOrderAction.value = true
  try {
    await adminPaymentsApi.expireOrder(orderId)
    success(t('ui.orderMarkedAsExpired'))
    await loadOrders()
  } catch (error) {
    log.error('标记过期失败:', error)
    showError(parseApiError(error, '标记过期失败'))
  } finally {
    submittingOrderAction.value = false
  }
}

async function failOrder(orderId: string) {
  submittingOrderAction.value = true
  try {
    await adminPaymentsApi.failOrder(orderId)
    success(t('ui.orderMarkedAsFailed'))
    await loadOrders()
  } catch (error) {
    log.error('标记失败失败:', error)
    showError(parseApiError(error, '标记失败失败'))
  } finally {
    submittingOrderAction.value = false
  }
}

function handleLedgerPageChange(page: number) {
  ledgerPage.value = page
  void loadLedger()
}

function handleLedgerPageSizeChange(size: number) {
  ledgerPageSize.value = size
  ledgerPage.value = 1
  void loadLedger()
}

function handleRefundPageChange(page: number) {
  refundPage.value = page
  void loadRefunds()
}

function handleRefundPageSizeChange(size: number) {
  refundPageSize.value = size
  refundPage.value = 1
  void loadRefunds()
}

function handleOrderPageChange(page: number) {
  orderPage.value = page
  void loadOrders()
}

function handleOrderPageSizeChange(size: number) {
  orderPageSize.value = size
  orderPage.value = 1
  void loadOrders()
}

function handleCallbackPageChange(page: number) {
  callbackPage.value = page
  void loadCallbacks()
}

function handleCallbackPageSizeChange(size: number) {
  callbackPageSize.value = size
  callbackPage.value = 1
  void loadCallbacks()
}

</script>

