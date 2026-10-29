import { translate } from '@/i18n'
import { formatCompactNumber } from '@/utils/format'
import { getI18nLocale } from '@/i18n'

export function walletStatusLabel(status: string | null | undefined): string {
  const labels: Record<string, string> = {
    get active() { return translate('ui.normal') },
    suspended: '已冻结',
    closed: '已关闭',
  }
  if (!status) return translate('chart.unknown')
  if (getI18nLocale() === 'en-US') {
    const englishLabels: Record<string, string> = {
      active: 'Active',
      suspended: 'Frozen',
      closed: 'Closed',
    }
    return englishLabels[status] || status
  }
  return labels[status] || status
}

export function formatWalletCurrency(
  value: number | null | undefined,
  options?: { decimals?: number }
): string {
  const decimals = options?.decimals ?? 2
  const amount = Number(value ?? 0)
  return `$${amount.toFixed(decimals)}`
}

export function walletStatusBadge(status: string | null | undefined) {
  if (status === 'active') return 'success'
  if (status === 'suspended') return 'warning'
  if (status === 'closed') return 'destructive'
  return 'secondary'
}

export function walletTransactionCategoryLabel(category: string | null | undefined): string {
  const labels: Record<string, string> = {
    get recharge() { return translate('ui.topUp') },
    get gift() { return translate('ui.giftCredit') },
    get adjust() { return translate('ui.adjustment') },
    get refund() { return translate('ui.refundf1c91e1') },
  }
  if (!category) return translate('chart.unknown')
  if (getI18nLocale() === 'en-US') {
    const englishLabels: Record<string, string> = {
      recharge: 'Top-up',
      gift: 'Grant',
      adjust: 'Adjustment',
      refund: 'Refund',
    }
    return englishLabels[category] || category
  }
  return labels[category] || category
}

export function dailyUsageCategoryLabel(isToday = false): string {
  if (getI18nLocale() === 'en-US') return isToday ? 'Today usage' : 'Daily usage'
  return isToday ? '今日消费' : '每日消费'
}

export function formatTokenCount(value: number | null | undefined): string {
  const amount = Number(value ?? 0)
  if (!Number.isFinite(amount) || amount <= 0) return '0'
  return formatCompactNumber(Math.round(amount), { fractionDigits: 1 })
}

export function walletTransactionReasonLabel(reasonCode: string | null | undefined): string {
  const labels: Record<string, string> = {
    get topup_admin_manual() { return translate('ui.manualTopUp') },
    get topup_gateway() { return translate('ui.paymentTopUp') },
    get topup_card_code() { return translate('ui.codeTopUp') },
    get gift_initial() { return translate('ui.initialGiftCredit') },
    get gift_campaign() { return translate('ui.promotionalGiftCredit') },
    get gift_expire_reclaim() { return translate('ui.giftCreditReversal') },
    get adjust_admin() { return translate('ui.manualAdjustment') },
    get adjust_system() { return translate('ui.systemAdjustment') },
    referral_reward: '邀请返利发放',
    referral_reward_reversal: '邀请返利冲回',
    get refund_out() { return translate('ui.refundDebit') },
    get refund_revert() { return translate('ui.refundRecredit') },
  }
  if (!reasonCode) return translate('chart.unknown')
  if (getI18nLocale() === 'en-US') {
    const englishLabels: Record<string, string> = {
      topup_admin_manual: 'Manual top-up',
      topup_gateway: 'Payment top-up',
      topup_card_code: 'Card code top-up',
      gift_initial: 'Initial grant',
      gift_campaign: 'Campaign grant',
      gift_expire_reclaim: 'Grant reclaim',
      adjust_admin: 'Manual adjustment',
      adjust_system: 'System adjustment',
      referral_reward: 'Referral reward',
      referral_reward_reversal: 'Referral reward reversal',
      refund_out: 'Refund deduction',
      refund_revert: 'Refund reversal',
    }
    return englishLabels[reasonCode] || reasonCode
  }
  return labels[reasonCode] || reasonCode
}

export function paymentMethodLabel(method: string | null | undefined): string {
  const labels: Record<string, string> = {
    get alipay() { return translate('ui.alipay') },
    get wechat() { return translate('ui.wechatPay') },
    get wxpay() { return translate('ui.wechatPay') },
    get wechat_pay() { return translate('ui.wechatPay') },
    get epay() { return translate('ui.epay') },
    stripe: 'Stripe',
    get card() { return translate('ui.cardCreditCard') },
    link: 'Stripe Link',
    get admin_manual() { return translate('ui.manualTopUp') },
    get card_code() { return translate('ui.topUpCard') },
    get gift_code() { return translate('ui.giftCard') },
    get card_recharge() { return translate('ui.codeTopUp') },
    bank_transfer: '银行转账',
    offline: '线下转账',
  }
  if (!method) return '-'
  if (getI18nLocale() === 'en-US') {
    const englishLabels: Record<string, string> = {
      alipay: 'Alipay',
      wechat: 'WeChat Pay',
      wxpay: 'WeChat Pay',
      wechat_pay: 'WeChat Pay',
      epay: 'E-Pay',
      stripe: 'Stripe',
      card: 'Bank card / credit card',
      link: 'Stripe Link',
      admin_manual: 'Manual top-up',
      card_code: 'Top-up code',
      gift_code: 'Gift code',
      card_recharge: 'Card code top-up',
      bank_transfer: 'Bank transfer',
      offline: 'Offline transfer',
    }
    return englishLabels[method] || method
  }
  return labels[method] || method
}

export function paymentStatusLabel(status: string | null | undefined): string {
  const labels: Record<string, string> = {
    get pending() { return translate('ui.awaitingPayment') },
    get paid() { return translate('ui.paid0a52f8a') },
    get credited() { return translate('ui.credited') },
    get failed() { return translate('ui.paymentFailed') },
    get expired() { return translate('ui.expired') },
    refunding: '退款中',
    refunded: '已退款',
  }
  if (!status) return translate('chart.unknown')
  if (getI18nLocale() === 'en-US') {
    const englishLabels: Record<string, string> = {
      pending: 'Pending',
      paid: 'Paid',
      credited: 'Credited',
      failed: 'Failed',
      expired: 'Expired',
      refunding: 'Refunding',
      refunded: 'Refunded',
    }
    return englishLabels[status] || status
  }
  return labels[status] || status
}

export function walletLinkTypeLabel(type: string | null | undefined): string {
  const labels: Record<string, string> = {
    get payment_order() { return translate('ui.topUpOrder') },
    get refund_request() { return translate('ui.refundRequest') },
    referral_reward: '邀请返利记录',
    admin_action: '后台操作',
    system_task: '系统任务',
    campaign: '活动批次',
    usage: '用量记录',
  }
  if (!type) return '-'
  if (getI18nLocale() === 'en-US') {
    const englishLabels: Record<string, string> = {
      payment_order: 'Top-up order',
      refund_request: 'Refund request',
      referral_reward: 'Referral reward record',
      admin_action: 'Admin action',
      system_task: 'System task',
      campaign: 'Campaign batch',
      usage: 'Usage record',
    }
    return englishLabels[type] || 'Other'
  }
  return labels[type] || '其他'
}

export function paymentStatusBadge(status: string | null | undefined) {
  if (status === 'credited' || status === 'refunded') return 'success'
  if (status === 'paid' || status === 'refunding') return 'outline'
  if (status === 'pending') return 'secondary'
  if (status === 'expired') return 'warning'
  if (status === 'failed') return 'destructive'
  return 'secondary'
}

export function refundModeLabel(mode: string | null | undefined): string {
  const labels: Record<string, string> = {
    get original_channel() { return translate('ui.originalRouteRefund') },
    get offline_payout() { return translate('ui.offlineTransfer') },
  }
  if (!mode) return '-'
  if (getI18nLocale() === 'en-US') {
    const englishLabels: Record<string, string> = {
      original_channel: 'Original channel',
      offline_payout: 'Offline payout',
    }
    return englishLabels[mode] || mode
  }
  return labels[mode] || mode
}

export function refundStatusLabel(status: string | null | undefined): string {
  const labels: Record<string, string> = {
    get pending_approval() { return translate('ui.pendingApproval') },
    get approved() { return translate('ui.approved') },
    get processing() { return translate('ui.processing694b71b') },
    get succeeded() { return translate('site.home.status.completed') },
    get failed() { return translate('ui.failed7e9ef4d') },
    get cancelled() { return translate('ui.canceled') },
  }
  if (!status) return translate('chart.unknown')
  if (getI18nLocale() === 'en-US') {
    const englishLabels: Record<string, string> = {
      pending_approval: 'Pending approval',
      approved: 'Approved',
      processing: 'Processing',
      succeeded: 'Succeeded',
      failed: 'Failed',
      cancelled: 'Cancelled',
    }
    return englishLabels[status] || status
  }
  return labels[status] || status
}

export function refundStatusBadge(status: string | null | undefined) {
  if (status === 'succeeded') return 'success'
  if (status === 'processing') return 'outline'
  if (status === 'pending_approval' || status === 'approved') return 'secondary'
  if (status === 'failed' || status === 'cancelled') return 'destructive'
  return 'secondary'
}

export function callbackStatusLabel(status: string | null | undefined): string {
  const labels: Record<string, string> = {
    processed: '已处理',
    duplicate: '重复回调',
    ignored: '已忽略',
    invalid_signature: '验签失败',
    error: '处理失败',
  }
  if (!status) return translate('chart.unknown')
  if (getI18nLocale() === 'en-US') {
    const englishLabels: Record<string, string> = {
      processed: 'Processed',
      duplicate: 'Duplicate callback',
      ignored: 'Ignored',
      invalid_signature: 'Invalid signature',
      error: 'Failed',
    }
    return englishLabels[status] || status
  }
  return labels[status] || status
}

export function callbackStatusBadge(status: string | null | undefined) {
  if (status === 'processed') return 'success'
  if (status === 'duplicate' || status === 'ignored') return 'secondary'
  if (status === 'invalid_signature' || status === 'error') return 'destructive'
  return 'outline'
}

export function ownerTypeLabel(ownerType: 'user' | 'api_key') {
  return ownerType === 'user' ? translate('ui.userWallet') : translate('nav.standaloneKeys')
}

export function ownerDisplayName(name: string | null | undefined, ownerType: 'user' | 'api_key') {
  if (name) return name
  return ownerType === 'user' ? translate('ui.unnamedUser') : translate('ui.unnamedKey')
}

export function redeemCodeStatusLabel(status: string) {
  if (status === 'active') return translate('ui.available4d99c97')
  if (status === 'disabled') return translate('ui.disableda8c3698')
  if (status === 'redeemed') return translate('ui.redeemed')
  return status
}

export function redeemCodeStatusBadge(status: string) {
  if (status === 'active') return 'success'
  if (status === 'disabled') return 'secondary'
  if (status === 'redeemed') return 'outline'
  return 'secondary'
}
