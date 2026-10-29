<template>
  <div class="space-y-6 pb-8">
    <div
      v-if="loadingInitial"
      class="py-16"
    >
      <LoadingState :message="t('ui.loadingWallet')" />
    </div>

    <template v-else>
      <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4">
        <Card class="p-5 space-y-2">
          <div class="text-xs uppercase tracking-wider text-muted-foreground">
            {{ t('ui.totalAvailableBalance') }}
          </div>
          <div class="text-3xl font-bold tabular-nums">
            {{ walletBalance?.unlimited ? t('ui.unlimited9ee3c07') : formatCurrency(totalAvailableBalance) }}
          </div>
          <div class="text-xs text-muted-foreground">
            {{ t('ui.planCredit') }} {{ formatCurrency(packageBalance) }} {{ t('ui.walletBalance0970059') }} {{ formatCurrency(walletOnlyBalance) }}
          </div>
        </Card>

        <Card class="p-5 space-y-3">
          <div class="text-xs uppercase tracking-wider text-muted-foreground">
            {{ t('ui.planQuotaToday') }}
          </div>
          <div class="text-2xl font-bold tabular-nums">
            <template v-if="hasActiveDailyQuota">
              {{ formatCurrency(packageBalance) }}
            </template>
            <template v-else>
              {{ t('ui.notActivated') }}
            </template>
          </div>
          <div
            v-if="hasActiveDailyQuota"
            class="space-y-1.5"
          >
            <div class="h-1.5 overflow-hidden rounded-full bg-muted">
              <div
                class="h-full rounded-full bg-primary transition-all"
                :style="{ width: `${dailyQuotaRemainingPercent}%` }"
              />
            </div>
            <div class="text-xs text-muted-foreground">
              {{ t('ui.used') }} {{ formatCurrency(dailyQuotaUsed) }} {{ t('ui.day') }} {{ formatCurrency(dailyQuotaTotal) }}
            </div>
            <div class="text-xs text-muted-foreground">
              {{ dailyQuota?.allow_wallet_overage ? t('ui.walletBalanceIsEnabledWhenThePlanQuotaIsInsufficient') : t('ui.requestsAreRejectedWhenThePlanQuotaIsInsufficientEnable') }}
            </div>
          </div>
          <div
            v-else
            class="text-xs text-muted-foreground"
          >
            {{ t('ui.afterActivatingADailyQuotaPlanRequestsConsumeThisQuota') }}
          </div>
        </Card>

        <Card class="p-5 space-y-2">
          <div class="text-xs uppercase tracking-wider text-muted-foreground">
            {{ t('ui.walletBalance') }}
          </div>
          <div class="text-2xl font-semibold tabular-nums">
            {{ formatCurrency(walletOnlyBalance) }}
          </div>
          <div class="text-xs text-muted-foreground">
            {{ t('ui.topUpBalancec3896d4') }} {{ formatCurrency(walletBalance?.wallet?.recharge_balance) }} {{ t('ui.grantBalanceeb1a851') }} {{ formatCurrency(walletBalance?.wallet?.gift_balance) }}
          </div>
        </Card>

        <Card class="p-5 space-y-2">
          <div class="text-xs uppercase tracking-wider text-muted-foreground">
            {{ t('ui.walletStatus') }}
          </div>
          <div class="flex items-center gap-2">
            <Badge :variant="walletStatusBadge(walletBalance?.wallet?.status)">
              {{ walletStatusLabel(walletBalance?.wallet?.status) }}
            </Badge>
          </div>
          <div class="text-xs text-muted-foreground">
            {{ t('ui.cumulativeTopUpSpending') }} {{ formatCurrency(walletBalance?.wallet?.total_recharged) }}
            <span class="text-muted-foreground font-normal mx-1">/</span>
            {{ formatCurrency(walletBalance?.wallet?.total_consumed) }}
          </div>
          <div class="text-xs text-muted-foreground">
            {{ t('ui.totalRefunded') }} {{ formatCurrency(walletBalance?.wallet?.total_refunded) }} {{ t('ui.refundableBalancea3bea10') }} {{ formatCurrency(walletBalance?.wallet?.refundable_balance) }}
          </div>
          <div
            v-if="walletBalance?.unlimited"
            class="text-xs text-amber-600 dark:text-amber-400"
          >
            {{ t('ui.thisAccountHasUnlimitedUsageTheBalanceIsTrackedFor') }}
          </div>
          <div class="text-xs text-muted-foreground">
            {{ t('ui.pendingRefunds') }} {{ walletBalance?.pending_refund_count || 0 }}
          </div>
        </Card>
      </div>

      <Card class="p-5 space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <h3 class="text-base font-semibold">
              {{ t('ui.redeemCodeTopUp') }}
            </h3>
            <p class="text-xs text-muted-foreground mt-1">
              {{ t('ui.enterARedeemCodeToTopUpDirectlyIntoWallet') }}
            </p>
          </div>
          <RefreshButton
            :loading="loadingOrders || loadingTransactions"
            @click="refreshWallet"
          />
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-[1fr_auto] gap-3">
          <Input
            v-model="redeemForm.code"
            :placeholder="t('ui.enterARedemptionCodeEGAbcdEfghIjklMnop')"
            autocomplete="off"
          />
          <Button
            :disabled="submittingRedeem"
            @click="submitRedeem"
          >
            {{ submittingRedeem ? t('ui.redeeming') : t('ui.redeem1f52a1f') }}
          </Button>
        </div>

        <div
          v-if="latestRedeem"
          class="rounded-xl border border-border/60 bg-muted/20 p-3 text-xs text-muted-foreground space-y-1.5"
        >
          <div>
            {{ t('ui.redeemedBatch') }} <span class="font-medium text-foreground">{{ latestRedeem.batch_name }}</span>
          </div>
          <div>
            {{ t('ui.topUpAmount') }} <span class="font-medium text-foreground">{{ formatCurrency(latestRedeem.amount_usd) }}</span>
          </div>
          <div>
            {{ t('ui.linkedOrder') }} <span class="font-mono text-foreground">{{ latestRedeem.order.order_no }}</span>
          </div>
        </div>
      </Card>

      <!-- TODO(wallet): 充值/退款用户主动操作入口暂未启用，待支付链路联调完成后再开放 -->
      <div
        v-if="ENABLE_WALLET_ACTION_FORMS"
        class="grid grid-cols-1 xl:grid-cols-2 gap-4"
      >
        <Card class="p-5 space-y-4">
          <div class="flex items-center justify-between">
            <h3 class="text-base font-semibold">
              {{ t('ui.startTopUp') }}
            </h3>
            <RefreshButton
              :loading="loadingOrders"
              @click="loadOrders"
            />
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div class="space-y-1.5">
              <Label>{{ t('ui.topUpAmountUsd') }}</Label>
              <Input
                v-model.number="rechargeForm.amount_usd"
                type="number"
                min="0.01"
                step="0.01"
                placeholder="10"
              />
            </div>

            <div class="space-y-1.5">
              <Label>{{ t('ui.paymentMethod') }}</Label>
              <Select v-model="rechargeForm.payment_option_key">
                <SelectTrigger>
                  <SelectValue
                    :placeholder="rechargeOptionsWithKey.length ? t('ui.selectPaymentMethod') : t('ui.noPaymentMethodsAvailable')"
                  />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem
                    v-for="option in rechargeOptionsWithKey"
                    :key="option.key"
                    :value="option.key"
                  >
                    {{ option.display_name }}
                    <span
                      v-if="option.pay_currency && option.usd_exchange_rate"
                      class="text-xs text-muted-foreground"
                    >
                      · {{ option.pay_currency }}
                    </span>
                    <span
                      v-if="Number(option.fee_rate || 0) > 0"
                      class="text-xs text-muted-foreground"
                    > {{ t('ui.fee89d05b4') }} {{ Number(option.fee_rate || 0).toFixed(2) }}%
                    </span>
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          <div
            v-if="selectedRechargeOption?.usd_exchange_rate"
            class="rounded-xl border border-border/60 bg-muted/20 p-3 text-xs text-muted-foreground"
          >
            {{ t('ui.estimatedPayment') }} <span class="font-medium text-foreground">
              {{ estimatedRechargePayAmount }}
              {{ rechargePayCurrency }}
            </span>
            · 1 USD = {{ Number(selectedRechargeOption.usd_exchange_rate).toFixed(4) }}
            {{ rechargePayCurrency }}
            <template v-if="estimatedRechargeFeeAmount > 0">
              {{ t('ui.fee89d05b4') }} {{ estimatedRechargeFeeAmount.toFixed(2) }} {{ rechargePayCurrency }}
              ({{ estimatedRechargeFeeRate.toFixed(2) }}%)
            </template>
          </div>

          <Button
            class="w-full"
            :disabled="submittingRecharge || rechargeOptionsWithKey.length === 0"
            @click="submitRecharge"
          >
            {{ submittingRecharge ? t('ui.creatingOrder') : t('ui.createTopUpOrder') }}
          </Button>

          <div
            v-if="latestRecharge"
            class="rounded-xl border border-border/60 bg-muted/30 p-3 space-y-1.5"
          >
            <div class="text-xs text-muted-foreground">
              {{ t('ui.latestOrder8e30e64') }} <span class="font-medium text-foreground">{{ latestRecharge.order.order_no }}</span>
            </div>
            <div class="text-xs text-muted-foreground">
              {{ t('ui.status') }} <Badge
                :variant="paymentStatusBadge(latestRecharge.order.status)"
                class="ml-1"
              >
                {{ paymentStatusLabel(latestRecharge.order.status) }}
              </Badge>
            </div>
            <a
              v-if="latestRechargePaymentUrl"
              class="inline-flex text-xs text-primary hover:underline"
              :href="latestRechargePaymentUrl"
              target="_blank"
              rel="noopener noreferrer"
              @click.prevent="submitPaymentInstructions(latestRecharge.payment_instructions)"
            > {{ t('ui.openPaymentLink') }} </a>
            <button
              v-if="latestRechargeStripeInstructions"
              type="button"
              class="inline-flex text-xs text-primary hover:underline"
              @click="submitPaymentInstructions(latestRecharge?.payment_instructions)"
            >
              {{ t('ui.openStripeCheckout') }}
            </button>
            <div
              v-if="latestRecharge.payment_instructions?.qr_code"
              class="text-xs text-muted-foreground break-all"
            >
              {{ t('ui.qrCodeReference') }} {{ latestRecharge.payment_instructions.qr_code }}
            </div>
          </div>
        </Card>

        <Card class="p-5 space-y-4">
          <div class="flex items-center justify-between">
            <h3 class="text-base font-semibold">
              {{ t('ui.requestRefund') }}
            </h3>
            <RefreshButton
              :loading="loadingRefunds || loadingRefundEligibility"
              @click="refreshRefundPanel"
            />
          </div>

          <div
            v-if="!loadingRefundEligibility && refundableOrders.length === 0"
            class="rounded-xl border border-border/60 bg-muted/20 p-3 text-xs text-muted-foreground"
          >
            {{ t('ui.thereAreCurrentlyNoRefundableTopUpOrdersWithUser') }}
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div class="space-y-1.5">
              <Label>{{ t('ui.refundAmountUsd') }}</Label>
              <Input
                v-model.number="refundForm.amount_usd"
                type="number"
                min="0.01"
                step="0.01"
                placeholder="5"
              />
            </div>

            <div class="space-y-1.5">
              <Label>{{ t('ui.refundMode') }}</Label>
              <Select v-model="refundForm.refund_mode">
                <SelectTrigger>
                  <SelectValue :placeholder="t('ui.selectRefundMethod')" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="original_channel">
                    {{ t('ui.originalRouteRefund') }}
                  </SelectItem>
                  <SelectItem value="offline_payout">
                    {{ t('ui.offlineTransfer') }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          <div class="space-y-1.5">
            <Label>{{ t('ui.linkedTopUpOrder') }}</Label>
            <Select v-model="refundForm.payment_order_id">
              <SelectTrigger>
                <SelectValue :placeholder="t('ui.selectAnOrderEligibleForUserRefunds')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="order in refundableOrders"
                  :key="order.id"
                  :value="order.id"
                >
                  {{ order.order_no }} {{ t('ui.refundable') }} {{ formatCurrency(order.refundable_amount_usd) }})
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-1.5">
            <Label>{{ t('ui.refundReasonOptional') }}</Label>
            <Textarea
              v-model="refundForm.reason"
              :placeholder="t('ui.describeTheReasonForYourRefundRequest')"
              rows="3"
            />
          </div>

          <div class="rounded-xl border border-border/60 bg-muted/20 p-3 text-xs text-muted-foreground">
            {{ t('ui.onlyPaymentMethodsWithAllowUserRefundsEnabledCanBe') }}
          </div>

          <Button
            class="w-full"
            variant="outline"
            :disabled="submittingRefund || refundableOrders.length === 0"
            @click="submitRefund"
          >
            {{ submittingRefund ? t('ui.submitting') : t('ui.submitRefundRequest') }}
          </Button>
        </Card>
      </div>

      <Card class="overflow-hidden">
        <div class="px-5 pt-5 pb-2">
          <Tabs v-model="activeTab">
            <TabsList class="tabs-button-list grid grid-cols-3 w-full max-w-xl">
              <TabsTrigger value="transactions">
                {{ t('ui.walletLedger') }}
              </TabsTrigger>
              <TabsTrigger value="orders">
                {{ t('ui.topUpOrder') }}
              </TabsTrigger>
              <TabsTrigger value="refunds">
                {{ t('ui.refundHistory') }}
              </TabsTrigger>
            </TabsList>

            <TabsContent
              value="transactions"
              class="mt-4 space-y-4"
            >
              <div class="px-5 flex items-center justify-between">
                <div class="text-sm text-muted-foreground">
                  {{ t('ui.total76e547a') }} {{ txTotal }} {{ t('ui.itemsf004f1d') }}
                </div>
                <RefreshButton
                  :loading="loadingTransactions"
                  @click="loadTransactions"
                />
              </div>
              <div class="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{{ t('ui.time') }}</TableHead>
                      <TableHead>{{ t('ui.typeba40014') }}</TableHead>
                      <TableHead>{{ t('ui.change') }}</TableHead>
                      <TableHead>{{ t('ui.balanceChanges') }}</TableHead>
                      <TableHead>{{ t('ui.description') }}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow v-if="todayUsage">
                      <TableCell class="text-xs text-muted-foreground">
                        {{ todayUsage.date || '-' }}
                      </TableCell>
                      <TableCell>
                        <div class="space-y-1">
                          <div class="flex items-center gap-2">
                            <Badge
                              variant="outline"
                              class="font-mono border-amber-500/40 text-amber-700 dark:text-amber-300"
                            >
                              {{ dailyUsageCategoryLabel(true) }}
                            </Badge>
                            <span class="inline-flex h-2 w-2 rounded-full bg-emerald-500 animate-pulse" />
                            <span class="text-[11px] text-muted-foreground">
                              Live
                            </span>
                          </div>
                          <div class="text-[11px] text-muted-foreground">
                            {{ todayUsage.timezone || 'UTC' }}
                          </div>
                        </div>
                      </TableCell>
                      <TableCell class="text-rose-600 dark:text-rose-400">
                        -{{ todayUsage.total_cost.toFixed(4) }}
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground">
                        {{ t('ui.dailySummary') }}
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground">
                        {{ todayUsage.total_requests }} {{ t('ui.requests0de6ce2') }} {{ formatTokenCount(todayUsage.input_tokens) }} / {{ formatTokenCount(todayUsage.output_tokens) }} tokens
                      </TableCell>
                    </TableRow>
                    <template
                      v-for="item in flowItems"
                      :key="item.type === 'transaction' ? item.data.id : `daily-${item.data.id || item.data.date}`"
                    >
                      <TableRow v-if="item.type === 'transaction'">
                        <TableCell class="text-xs text-muted-foreground">
                          {{ formatDateTime(item.data.created_at) }}
                        </TableCell>
                        <TableCell>
                          <div class="space-y-1">
                            <Badge
                              variant="outline"
                              class="font-mono"
                            >
                              {{ walletTransactionCategoryLabel(item.data.category) }}
                            </Badge>
                            <div class="text-[11px] text-muted-foreground">
                              {{ walletTransactionReasonLabel(item.data.reason_code) }}
                            </div>
                          </div>
                        </TableCell>
                        <TableCell
                          :class="item.data.amount >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'"
                        >
                          {{ item.data.amount >= 0 ? '+' : '' }}{{ item.data.amount.toFixed(4) }}
                        </TableCell>
                        <TableCell class="text-xs tabular-nums">
                          {{ item.data.balance_before.toFixed(4) }} → {{ item.data.balance_after.toFixed(4) }}
                        </TableCell>
                        <TableCell class="text-xs text-muted-foreground">
                          {{ item.data.description || '-' }}
                        </TableCell>
                      </TableRow>
                      <TableRow v-else>
                        <TableCell class="text-xs text-muted-foreground">
                          {{ item.data.date || '-' }}
                        </TableCell>
                        <TableCell>
                          <div class="space-y-1">
                            <Badge
                              variant="outline"
                              class="font-mono border-amber-500/40 text-amber-700 dark:text-amber-300"
                            >
                              {{ dailyUsageCategoryLabel(false) }}
                            </Badge>
                            <div class="text-[11px] text-muted-foreground">
                              {{ item.data.timezone || '-' }}
                            </div>
                          </div>
                        </TableCell>
                        <TableCell class="text-rose-600 dark:text-rose-400">
                          -{{ item.data.total_cost.toFixed(4) }}
                        </TableCell>
                        <TableCell class="text-xs text-muted-foreground">
                          {{ t('ui.dailySummary') }}
                        </TableCell>
                        <TableCell class="text-xs text-muted-foreground">
                          {{ item.data.total_requests }} {{ t('ui.requests0de6ce2') }} {{ formatTokenCount(item.data.input_tokens) }} / {{ formatTokenCount(item.data.output_tokens) }} tokens
                        </TableCell>
                      </TableRow>
                    </template>
                    <TableRow v-if="!loadingTransactions && flowItems.length === 0">
                      <TableCell
                        colspan="5"
                        class="py-10"
                      >
                        <EmptyState
                          :title="t('ui.noWalletLedgerEntries')"
                          :description="t('ui.topUpsRefundsAndChargesWillAppearHere')"
                        />
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </div>
              <Pagination
                :current="txPage"
                :total="txTotal"
                :page-size="txPageSize"
                @update:current="handleTxPageChange"
                @update:page-size="handleTxPageSizeChange"
              />
            </TabsContent>

            <TabsContent
              value="orders"
              class="mt-4 space-y-4"
            >
              <div class="px-5 flex items-center justify-between">
                <div class="text-sm text-muted-foreground">
                  {{ t('ui.total76e547a') }} {{ orderTotal }} {{ t('ui.itemsf004f1d') }}
                </div>
                <RefreshButton
                  :loading="loadingOrders"
                  @click="loadOrders"
                />
              </div>
              <div class="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{{ t('ui.orderNumber') }}</TableHead>
                      <TableHead>{{ t('ui.amount') }}</TableHead>
                      <TableHead>{{ t('ui.paymentMethod') }}</TableHead>
                      <TableHead>{{ t('ui.status6320b4a') }}</TableHead>
                      <TableHead>{{ t('ui.refundableAmount') }}</TableHead>
                      <TableHead>{{ t('ui.createdAt') }}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="order in rechargeOrders"
                      :key="order.id"
                    >
                      <TableCell class="font-mono text-xs">
                        {{ order.order_no }}
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
                      <TableCell class="tabular-nums">
                        {{ formatCurrency(order.refundable_amount_usd) }}
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground">
                        {{ formatDateTime(order.created_at) }}
                      </TableCell>
                    </TableRow>
                    <TableRow v-if="!loadingOrders && rechargeOrders.length === 0">
                      <TableCell
                        colspan="6"
                        class="py-10"
                      >
                        <EmptyState
                          :title="t('ui.noTopUpOrders')"
                          :description="t('ui.yourTopUpOrdersWillAppearHere')"
                        />
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
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
              value="refunds"
              class="mt-4 space-y-4"
            >
              <div class="px-5 flex items-center justify-between">
                <div class="text-sm text-muted-foreground">
                  {{ t('ui.total76e547a') }} {{ refundTotal }} {{ t('ui.itemsf004f1d') }}
                </div>
                <RefreshButton
                  :loading="loadingRefunds"
                  @click="loadRefunds"
                />
              </div>
              <div class="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{{ t('ui.refundId') }}</TableHead>
                      <TableHead>{{ t('ui.amount') }}</TableHead>
                      <TableHead>{{ t('ui.mode') }}</TableHead>
                      <TableHead>{{ t('ui.status6320b4a') }}</TableHead>
                      <TableHead>{{ t('ui.reasonf029d9c') }}</TableHead>
                      <TableHead>{{ t('ui.requestedAt') }}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="refund in refunds"
                      :key="refund.id"
                    >
                      <TableCell class="font-mono text-xs">
                        {{ refund.refund_no }}
                      </TableCell>
                      <TableCell class="tabular-nums">
                        {{ formatCurrency(refund.amount_usd) }}
                      </TableCell>
                      <TableCell>{{ refundModeLabel(refund.refund_mode) }}</TableCell>
                      <TableCell>
                        <Badge :variant="refundStatusBadge(refund.status)">
                          {{ refundStatusLabel(refund.status) }}
                        </Badge>
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground max-w-[220px] truncate">
                        {{ refund.reason || refund.failure_reason || '-' }}
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground">
                        {{ formatDateTime(refund.created_at) }}
                      </TableCell>
                    </TableRow>
                    <TableRow v-if="!loadingRefunds && refunds.length === 0">
                      <TableCell
                        colspan="6"
                        class="py-10"
                      >
                        <EmptyState
                          :title="t('ui.noRefunds')"
                          :description="t('ui.yourRefundRequestsWillAppearHere')"
                        />
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </div>
              <Pagination
                :current="refundPage"
                :total="refundTotal"
                :page-size="refundPageSize"
                @update:current="handleRefundPageChange"
                @update:page-size="handleRefundPageSizeChange"
              />
            </TabsContent>
          </Tabs>
        </div>
      </Card>
    </template>

    <StripePaymentDialog
      v-model:open="stripeDialogOpen"
      :instructions="stripePaymentInstructions"
      :title="t('ui.walletTopUpViaStripe')"
      :description="t('ui.yourWalletWillBeCreditedAutomaticallyAfterStripeConfirmsPayment')"
      :confirm-text="t('ui.paymentTopUp')"
      @success="handleStripePaymentSuccess"
    />
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/i18n'
const { t } = useI18n()

import { getI18nLocale } from '@/i18n'
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import {
  Badge,
  Button,
  Card,
  Input,
  Label,
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
  Textarea,
} from '@/components/ui'
import { EmptyState, LoadingState } from '@/components/common'
import StripePaymentDialog from '@/features/wallet/components/StripePaymentDialog.vue'
import {
  walletApi,
  type DailyUsageRecord,
  type FlowItem,
  type PaymentOrder,
  type RefundRequest,
  type WalletBalanceResponse,
  type WalletRedeemResponse,
  type WalletRechargeOption,
} from '@/api/wallet'
import { useToast } from '@/composables/useToast'
import { parseApiError } from '@/utils/errorParser'
import { log } from '@/utils/logger'
import { safePaymentTargetUrl } from '@/features/wallet/utils/paymentUrl'
import {
  getPaymentInstructionString,
  getStripePaymentInstructions,
  type PaymentInstructionMap,
} from '@/features/wallet/utils/paymentInstructions'
import {
  dailyUsageCategoryLabel,
  formatTokenCount,
  formatWalletCurrency as formatCurrency,
  paymentMethodLabel,
  paymentStatusBadge,
  paymentStatusLabel,
  refundModeLabel,
  refundStatusBadge,
  refundStatusLabel,
  walletStatusBadge,
  walletStatusLabel,
  walletTransactionCategoryLabel,
  walletTransactionReasonLabel,
} from '@/features/wallet/utils/walletDisplay'

const { success, info, error: showError } = useToast()

const ENABLE_WALLET_ACTION_FORMS = true

const loadingInitial = ref(true)
const loadingTransactions = ref(false)
const loadingOrders = ref(false)
const loadingRefunds = ref(false)
const loadingRefundEligibility = ref(false)
const submittingRedeem = ref(false)
const submittingRecharge = ref(false)
const submittingRefund = ref(false)

const walletBalance = ref<WalletBalanceResponse | null>(null)
const latestRecharge = ref<{ order: PaymentOrder; payment_instructions: Record<string, unknown> } | null>(null)
const latestRedeem = ref<WalletRedeemResponse | null>(null)
const rechargeOptions = ref<WalletRechargeOption[]>([])
const stripeDialogOpen = ref(false)
const stripePaymentInstructions = ref<PaymentInstructionMap | null>(null)

const flowItems = ref<FlowItem[]>([])
const todayUsage = ref<DailyUsageRecord | null>(null)
const txTotal = ref(0)
const txPage = ref(1)
const txPageSize = ref(20)

const rechargeOrders = ref<PaymentOrder[]>([])
const orderTotal = ref(0)
const orderPage = ref(1)
const orderPageSize = ref(20)
const refundEligiblePaymentMethods = ref<Set<string>>(new Set())

const refunds = ref<RefundRequest[]>([])
const refundTotal = ref(0)
const refundPage = ref(1)
const refundPageSize = ref(20)

const activeTab = ref('transactions')
const loadedTabs = new Set<string>()
const tabLoadPromises = new Map<string, Promise<void>>()
let refundEligibilityLoaded = false
let todayCostPollTimer: ReturnType<typeof setInterval> | null = null
let orderPollTimer: ReturnType<typeof setTimeout> | null = null
let orderLoadVersion = 0
let unmounted = false
const pendingOrders = new Map<string, PaymentOrder>()

const rechargeForm = reactive({
  amount_usd: 10,
  payment_option_key: '',
  idempotency_key: '',
})
let rechargeIdempotencyFingerprint = ''

const refundForm = reactive({
  amount_usd: 0,
  payment_order_id: '',
  refund_mode: 'offline_payout',
  reason: '',
})

const redeemForm = reactive({
  code: '',
})

const refundableOrders = computed(() =>
  rechargeOrders.value.filter(order =>
    (order.refundable_amount_usd || 0) > 0
    && refundEligiblePaymentMethods.value.has(refundPaymentMethod(order))
  )
)

const rechargeOptionsWithKey = computed(() =>
  rechargeOptions.value.map((option, index) => ({
    ...option,
    key: [
      option.payment_provider || option.provider || option.payment_method,
      option.payment_method,
      option.payment_channel || '',
      index,
    ].join(':'),
  }))
)

const selectedRechargeOption = computed(() => {
  if (rechargeOptionsWithKey.value.length === 0) return null
  return rechargeOptionsWithKey.value.find(option => option.key === rechargeForm.payment_option_key)
    || rechargeOptionsWithKey.value[0]
})

function roundPayAmount(value: number): number {
  return Math.round(value * 100) / 100
}

const rechargePaymentBreakdown = computed(() => {
  const rate = Number(selectedRechargeOption.value?.usd_exchange_rate || 0)
  const amount = Number(rechargeForm.amount_usd || 0)
  if (!Number.isFinite(rate) || rate <= 0 || !Number.isFinite(amount) || amount <= 0) return null
  const rawFeeRate = Number(selectedRechargeOption.value?.fee_rate || 0)
  const feeRate = Number.isFinite(rawFeeRate) && rawFeeRate > 0 ? rawFeeRate : 0
  const basePayAmount = roundPayAmount(amount * rate)
  const feeAmount = roundPayAmount(basePayAmount * feeRate / 100)
  return {
    basePayAmount,
    feeAmount,
    feeRate,
    totalPayAmount: roundPayAmount(basePayAmount + feeAmount),
  }
})

const rechargePayCurrency = computed(() => selectedRechargeOption.value?.pay_currency || 'CNY')

const estimatedRechargePayAmount = computed(() => {
  if (!rechargePaymentBreakdown.value) return '-'
  return rechargePaymentBreakdown.value.totalPayAmount.toFixed(2)
})
const estimatedRechargeFeeAmount = computed(() =>
  rechargePaymentBreakdown.value?.feeAmount || 0
)
const estimatedRechargeFeeRate = computed(() =>
  rechargePaymentBreakdown.value?.feeRate || 0
)
const latestRechargePaymentUrl = computed(() => {
  const paymentUrl = getPaymentInstructionString(
    latestRecharge.value?.payment_instructions,
    'payment_url',
  )
  return paymentUrl ? safePaymentTargetUrl(paymentUrl) : null
})
const latestRechargeStripeInstructions = computed(() =>
  getStripePaymentInstructions(latestRecharge.value?.payment_instructions)
)

const dailyQuota = computed(() => walletBalance.value?.daily_quota ?? null)
const hasActiveDailyQuota = computed(() => Boolean(dailyQuota.value?.has_active))
const walletOnlyBalance = computed(() => {
  const explicitBalance = walletBalance.value?.wallet_balance
  if (typeof explicitBalance === 'number' && Number.isFinite(explicitBalance)) {
    return explicitBalance
  }
  return Number(walletBalance.value?.balance ?? 0)
})
const packageBalance = computed(() => {
  const quotaRemaining = dailyQuota.value?.remaining_usd
  if (hasActiveDailyQuota.value && typeof quotaRemaining === 'number' && Number.isFinite(quotaRemaining)) {
    return Math.max(0, quotaRemaining)
  }
  const explicitBalance = walletBalance.value?.package_balance
  if (typeof explicitBalance === 'number' && Number.isFinite(explicitBalance)) {
    return Math.max(0, explicitBalance)
  }
  return 0
})
const totalAvailableBalance = computed(() => {
  const explicitBalance = walletBalance.value?.total_available_balance
  if (typeof explicitBalance === 'number' && Number.isFinite(explicitBalance)) {
    return explicitBalance
  }
  return walletOnlyBalance.value + packageBalance.value
})
const dailyQuotaTotal = computed(() => {
  const value = dailyQuota.value?.total_usd
  return typeof value === 'number' && Number.isFinite(value) ? Math.max(0, value) : 0
})
const dailyQuotaUsed = computed(() => {
  const value = dailyQuota.value?.used_usd
  return typeof value === 'number' && Number.isFinite(value) ? Math.max(0, value) : 0
})
const dailyQuotaRemainingPercent = computed(() => {
  if (!hasActiveDailyQuota.value || dailyQuotaTotal.value <= 0) return 0
  return Math.min(100, Math.max(0, (packageBalance.value / dailyQuotaTotal.value) * 100))
})

onMounted(async () => {
  document.addEventListener('visibilitychange', handleVisibilityChange)
  try {
    await Promise.all([
      loadBalance(),
      loadTransactions(),
      loadRechargeOptions(),
    ])
    if (unmounted) return
    await loadOrders()
    syncTodayCostPolling()
  } finally {
    loadingInitial.value = false
  }
})

onBeforeUnmount(() => {
  unmounted = true
  orderLoadVersion += 1
  stopTodayCostPolling()
  stopOrderPolling()
  document.removeEventListener('visibilitychange', handleVisibilityChange)
})

watch(activeTab, (tab) => {
  syncTodayCostPolling()
  void loadActiveTab(tab)
})

watch(refundableOrders, () => {
  syncRefundOrderSelection()
})

async function refreshWallet() {
  await loadOrders()
  await Promise.all([loadBalance(), loadTransactions()])
}

async function loadBalance() {
  walletBalance.value = await walletApi.getBalance()
}

async function loadRechargeOptions() {
  try {
    const response = await walletApi.listRechargeOptions()
    rechargeOptions.value = response.items
    if (!rechargeForm.payment_option_key && rechargeOptionsWithKey.value.length > 0) {
      const preferred = rechargeOptionsWithKey.value.find(option => option.payment_provider === 'epay')
        || rechargeOptionsWithKey.value[0]
      rechargeForm.payment_option_key = preferred.key
    }
  } catch (error) {
    log.error('加载充值方式失败:', error)
    showError(parseApiError(error, t('ui.failedToLoadTopUpMethods')))
  }
}

async function loadTransactions() {
  loadingTransactions.value = true
  try {
    const offset = (txPage.value - 1) * txPageSize.value
    const resp = await walletApi.getFlow({ limit: txPageSize.value, offset })
    flowItems.value = resp.items
    txTotal.value = resp.total
    todayUsage.value = resp.today_entry
    loadedTabs.add('transactions')
  } catch (error) {
    log.error('加载钱包流水失败:', error)
    showError(parseApiError(error, t('ui.failedToLoadWalletTransactions')))
  } finally {
    loadingTransactions.value = false
  }
}

async function loadTodayCost() {
  try {
    todayUsage.value = await walletApi.getTodayCost()
  } catch (error) {
    log.error('加载今日消费失败:', error)
  }
}

function syncTodayCostPolling() {
  if (!unmounted && activeTab.value === 'transactions' && !document.hidden) {
    startTodayCostPolling()
  } else {
    stopTodayCostPolling()
  }
}

function startTodayCostPolling() {
  if (todayCostPollTimer) return
  todayCostPollTimer = setInterval(() => {
    void loadTodayCost()
  }, 20_000)
}

function stopTodayCostPolling() {
  if (!todayCostPollTimer) return
  clearInterval(todayCostPollTimer)
  todayCostPollTimer = null
}

function handleVisibilityChange() {
  syncTodayCostPolling()
  if (document.hidden) {
    stopOrderPolling()
  } else if (!loadingInitial.value) {
    void reloadOrders(true)
  }
}

function isAwaitingCredit(order: PaymentOrder): boolean {
  return order.status === 'paid'
    || (order.status === 'pending'
      && (!order.expires_at || new Date(order.expires_at).getTime() > Date.now()))
}

function stopOrderPolling() {
  if (orderPollTimer === null) return
  clearTimeout(orderPollTimer)
  orderPollTimer = null
}

function scheduleOrderPolling() {
  stopOrderPolling()
  if (unmounted || document.hidden || ![...pendingOrders.values()].some(isAwaitingCredit)) return
  orderPollTimer = setTimeout(() => {
    orderPollTimer = null
    void reloadOrders(true)
  }, 5_000)
}

function loadOrders() {
  return reloadOrders(false)
}

async function reloadOrders(silent: boolean) {
  if (unmounted) return
  stopOrderPolling()
  const version = ++orderLoadVersion
  loadingOrders.value = !silent
  try {
    const offset = (orderPage.value - 1) * orderPageSize.value
    const resp = await walletApi.listRechargeOrders({ limit: orderPageSize.value, offset })
    if (unmounted || version !== orderLoadVersion) return
    const orders = [...resp.items]
    const visibleIds = new Set(orders.map(order => order.id))
    const missingOrders = [...pendingOrders.values()].filter(order =>
      isAwaitingCredit(order) && !visibleIds.has(order.id)
    )
    const details = await Promise.allSettled(missingOrders.map(order => walletApi.getRechargeOrder(order.id)))
    if (unmounted || version !== orderLoadVersion) return
    for (const detail of details) {
      if (detail.status === 'fulfilled') orders.push(detail.value.order)
      else log.error('加载待到账订单失败:', detail.reason)
    }
    const newlyCredited = orders.filter(order =>
      order.status === 'credited' && pendingOrders.has(order.id)
    )
    for (const order of orders) {
      if (isAwaitingCredit(order)) pendingOrders.set(order.id, order)
      else if (order.status !== 'credited') pendingOrders.delete(order.id)
      if (latestRecharge.value?.order.id === order.id) latestRecharge.value.order = order
    }

    // Order lists include the wallet snapshot, but omit the daily package quota.
    const { items: _items, total: _total, limit: _limit, offset: _offset, ...balance } = resp
    const currentPackageBalance = packageBalance.value
    const currentWalletBalance = Number(balance.wallet?.balance ?? balance.balance ?? 0)
    walletBalance.value = {
      ...walletBalance.value,
      ...balance,
      wallet_balance: Math.max(0, currentWalletBalance),
      total_available_balance: balance.unlimited ? null : Math.max(0, currentWalletBalance + currentPackageBalance),
    }
    rechargeOrders.value = resp.items
    orderTotal.value = resp.total
    loadedTabs.add('orders')
    syncRefundOrderSelection()
    if (newlyCredited.length > 0) {
      // Read after the credited status so a pre-credit list snapshot cannot win.
      await Promise.all([loadBalance(), loadTransactions()])
      for (const order of newlyCredited) pendingOrders.delete(order.id)
      if (!unmounted && version === orderLoadVersion) success('充值已到账，余额已更新')
    }
  } catch (error) {
    if (unmounted || version !== orderLoadVersion) return
    log.error('加载充值订单失败:', error)
    if (!silent) showError(parseApiError(error, t('ui.failedToLoadTopUpOrders')))
  } finally {
    if (!unmounted && version === orderLoadVersion) {
      loadingOrders.value = false
      scheduleOrderPolling()
    }
  }
}

async function loadRefundEligibility() {
  loadingRefundEligibility.value = true
  try {
    const resp = await walletApi.listRefundEligibleProviders()
    refundEligiblePaymentMethods.value = new Set(
      (resp.payment_methods || [])
        .map(item => item.trim().toLowerCase())
        .filter(Boolean)
    )
    refundEligibilityLoaded = true
    syncRefundOrderSelection()
  } catch (error) {
    refundEligiblePaymentMethods.value = new Set()
    log.error('加载退款资格失败:', error)
  } finally {
    loadingRefundEligibility.value = false
  }
}

async function loadRefunds() {
  loadingRefunds.value = true
  try {
    const offset = (refundPage.value - 1) * refundPageSize.value
    const resp = await walletApi.listRefunds({ limit: refundPageSize.value, offset })
    refunds.value = resp.items
    refundTotal.value = resp.total
    loadedTabs.add('refunds')
  } catch (error) {
    log.error('加载退款记录失败:', error)
    showError(parseApiError(error, t('ui.failedToLoadRefunds')))
  } finally {
    loadingRefunds.value = false
  }
}

function loadActiveTab(tab: string): Promise<void> {
  const tabIsLoaded = tab === 'refunds'
    ? loadedTabs.has('refunds') && loadedTabs.has('orders') && refundEligibilityLoaded
    : loadedTabs.has(tab)
  if (tabIsLoaded) return Promise.resolve()
  const existing = tabLoadPromises.get(tab)
  if (existing) return existing

  const request = (async () => {
    if (tab === 'orders') {
      await loadOrders()
    } else if (tab === 'refunds') {
      const requests: Promise<void>[] = []
      if (!loadedTabs.has('refunds')) requests.push(loadRefunds())
      if (!refundEligibilityLoaded) requests.push(loadRefundEligibility())
      if (!loadedTabs.has('orders')) requests.push(loadOrders())
      await Promise.all(requests)
    }
  })().finally(() => {
    if (tabLoadPromises.get(tab) === request) tabLoadPromises.delete(tab)
  })
  tabLoadPromises.set(tab, request)
  return request
}

async function refreshRefundPanel() {
  await Promise.all([loadRefunds(), loadRefundEligibility(), loadOrders()])
}

async function submitRedeem() {
  if (!redeemForm.code.trim()) {
    showError(t('ui.enterARedemptionCode'))
    return
  }

  submittingRedeem.value = true
  try {
    latestRedeem.value = await walletApi.redeemCode({
      code: redeemForm.code.trim(),
    })
    redeemForm.code = ''
    success(t('ui.codeRedeemed'))
    await Promise.all([loadBalance(), loadOrders(), loadTransactions(), loadTodayCost()])
    activeTab.value = 'orders'
  } catch (error) {
    log.error('兑换码充值失败:', error)
    showError(parseApiError(error, t('ui.failedToRedeemCode')))
  } finally {
    submittingRedeem.value = false
  }
}

async function submitRecharge() {
  if (!rechargeForm.amount_usd || rechargeForm.amount_usd <= 0) {
    showError(t('ui.enterAValidTopUpAmount'))
    return
  }
  const option = selectedRechargeOption.value
  if (!option) {
    showError(t('ui.selectAPaymentMethod'))
    return
  }
  if (option.min_recharge_usd && rechargeForm.amount_usd < option.min_recharge_usd) {
    showError(`充值金额不能低于 ${formatCurrency(option.min_recharge_usd)}`)
    return
  }

  submittingRecharge.value = true
  try {
    // Keep a failed request retryable only while its business parameters stay
    // identical. Changing the amount or channel must get a fresh key.
    const requestFingerprint = JSON.stringify([
      Number(rechargeForm.amount_usd),
      option.payment_method || '',
      option.payment_provider || '',
      option.payment_channel || '',
    ])
    if (
      rechargeForm.idempotency_key
      && rechargeIdempotencyFingerprint
      && rechargeIdempotencyFingerprint !== requestFingerprint
    ) {
      rechargeForm.idempotency_key = ''
    }
    if (!rechargeForm.idempotency_key) {
      rechargeForm.idempotency_key = typeof globalThis.crypto?.randomUUID === 'function'
        ? globalThis.crypto.randomUUID()
        : `${Date.now()}-${Math.random().toString(36).slice(2)}`
    }
    rechargeIdempotencyFingerprint = requestFingerprint
    latestRecharge.value = await walletApi.createRechargeOrder({
      amount_usd: rechargeForm.amount_usd,
      payment_method: option.payment_method,
      payment_provider: option.payment_provider,
      payment_channel: option.payment_channel,
      idempotency_key: rechargeForm.idempotency_key,
    })
    if (isAwaitingCredit(latestRecharge.value.order)) {
      pendingOrders.set(latestRecharge.value.order.id, latestRecharge.value.order)
    }
    success(t('ui.topUpOrderCreated'))
    await loadOrders()
    activeTab.value = 'orders'
    submitPaymentInstructions(latestRecharge.value.payment_instructions)
    rechargeForm.idempotency_key = ''
    rechargeIdempotencyFingerprint = ''
  } catch (error) {
    log.error('创建充值订单失败:', error)
    showError(parseApiError(error, t('ui.failedToCreateTopUpOrder')))
  } finally {
    submittingRecharge.value = false
  }
}

function submitPaymentInstructions(instructions: Record<string, unknown> | null | undefined) {
  if (!instructions) return
  const stripeInstructions = getStripePaymentInstructions(instructions)
  if (stripeInstructions) {
    stripePaymentInstructions.value = instructions
    stripeDialogOpen.value = true
    return
  }
  const paymentUrl = getPaymentInstructionString(instructions, 'payment_url')
  if (!paymentUrl) return
  const safePaymentUrl = safePaymentTargetUrl(paymentUrl)
  if (!safePaymentUrl) {
    showError(t('ui.thePaymentGatewayReturnedAnUnsafePaymentUrl'))
    return
  }
  const paymentParams = instructions.payment_params
  if (paymentParams && typeof paymentParams === 'object' && !Array.isArray(paymentParams)) {
    submitPaymentForm(safePaymentUrl, paymentParams as Record<string, unknown>)
    return
  }
  const opened = window.open(safePaymentUrl, '_blank', 'noopener,noreferrer')
  if (!opened) {
    window.location.href = safePaymentUrl
  }
}

function submitPaymentForm(url: string, params: Record<string, unknown>) {
  const safeUrl = safePaymentTargetUrl(url)
  if (!safeUrl) {
    showError(t('ui.thePaymentGatewayReturnedAnUnsafePaymentUrl'))
    return
  }
  const form = document.createElement('form')
  form.action = safeUrl
  form.method = 'POST'
  if (!isSafariBrowser()) {
    form.target = '_blank'
  }
  Object.entries(params).forEach(([key, value]) => {
    if (value === null || value === undefined) return
    const input = document.createElement('input')
    input.type = 'hidden'
    input.name = key
    input.value = String(value)
    form.appendChild(input)
  })
  document.body.appendChild(form)
  form.submit()
  document.body.removeChild(form)
}

function isSafariBrowser(): boolean {
  return navigator.userAgent.includes('Safari') && !navigator.userAgent.includes('Chrome')
}

async function handleStripePaymentSuccess() {
  info('支付已提交，正在等待充值到账')
  await loadOrders()
  activeTab.value = 'orders'
}

async function submitRefund() {
  if (!refundForm.amount_usd || refundForm.amount_usd <= 0) {
    showError(t('ui.enterAValidRefundAmount'))
    return
  }
  const selectedOrder = refundableOrders.value.find(order => order.id === refundForm.payment_order_id)
  if (!selectedOrder) {
    showError(t('ui.selectATopUpOrderEligibleForUserRefunds'))
    return
  }
  if (refundForm.amount_usd > (selectedOrder.refundable_amount_usd || 0)) {
    showError(`退款金额超过该订单可退金额（当前可退 ${formatCurrency(selectedOrder.refundable_amount_usd || 0)}）`)
    return
  }
  const refundableBalance =
    walletBalance.value?.wallet?.refundable_balance ?? walletBalance.value?.refundable_balance ?? null
  if (refundableBalance !== null && refundForm.amount_usd > refundableBalance) {
    showError(`退款金额超过可退款余额（当前可退 ${formatCurrency(refundableBalance)}）`)
    return
  }

  submittingRefund.value = true
  try {
    await walletApi.createRefund({
      amount_usd: refundForm.amount_usd,
      payment_order_id: selectedOrder.id,
      reason: refundForm.reason || undefined,
      idempotency_key: `web_refund_${buildRefundIdempotencyKey()}`,
    })
    success(t('ui.refundRequestSubmitted'))
    refundForm.amount_usd = 0
    refundForm.reason = ''
    await Promise.all([loadRefunds(), loadBalance(), loadOrders(), loadRefundEligibility(), loadTransactions(), loadTodayCost()])
    activeTab.value = 'refunds'
  } catch (error) {
    log.error('提交退款申请失败:', error)
    showError(parseApiError(error, t('ui.failedToSubmitRefundRequest')))
  } finally {
    submittingRefund.value = false
  }
}

function refundPaymentMethod(order: PaymentOrder): string {
  return String(order.payment_provider || order.payment_method || '').trim().toLowerCase()
}

function syncRefundOrderSelection() {
  if (refundableOrders.value.some(order => order.id === refundForm.payment_order_id)) return
  refundForm.payment_order_id = refundableOrders.value[0]?.id || ''
}

function buildRefundIdempotencyKey(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID().replaceAll('-', '')
  }
  return `${Date.now()}_${Math.random().toString(16).slice(2, 10)}`
}

function handleTxPageChange(page: number) {
  txPage.value = page
  void loadTransactions()
}

function handleTxPageSizeChange(size: number) {
  txPageSize.value = size
  txPage.value = 1
  void loadTransactions()
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

function handleRefundPageChange(page: number) {
  refundPage.value = page
  void loadRefunds()
}

function handleRefundPageSizeChange(size: number) {
  refundPageSize.value = size
  refundPage.value = 1
  void loadRefunds()
}

function formatDateTime(value: string | null | undefined): string {
  if (!value) return '-'
  return new Date(value).toLocaleString(getI18nLocale(), {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

</script>
