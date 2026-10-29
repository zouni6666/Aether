import { translate } from '@/i18n'
import { getI18nLocale } from '@/i18n'
import type { ReferralRewardRecord } from '@/api/referrals'

export function usd(value: number): string {
  return `$${Number(value || 0).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 8 })}`
}
export function date(value?: number | null): string {
  return value ? new Date(value * 1000).toLocaleString(getI18nLocale()) : translate('ui.notRecorded')
}
export function trigger(value: string): string {
  return ({ registration: translate('auth.register.submit'), email_verified: translate('ui.emailVerification'), first_paid_order: translate('ui.firstActualPayment'), payment: translate('ui.actualPayment'), paid_order: translate('ui.actualPayment') } as Record<string, string>)[value] || value
}
export function rewardType(value: string): string {
  return value === 'percent' ? translate('ui.percentageReward') : value === 'headcount' ? translate('ui.perUserReward') : value
}
export function rewardStatus(value: string): string {
  return ({ pending: translate('ui.pendingIssue'), applying: translate('ui.processing694b71b'), failed: translate('ui.grantFailed'), applied: translate('ui.issued'), reversed: translate('ui.issued'), voided: translate('ui.voided') } as Record<string, string>)[value] || value
}
export function statusVariant(value: string): 'success' | 'warning' | 'destructive' | 'secondary' | 'outline' {
  if (value === 'applied' || value === 'reversed') return 'success'
  if (value === 'failed') return 'destructive'
  if (value === 'pending' || value === 'applying') return 'warning'
  return 'secondary'
}
export function reversalStatus(item: ReferralRewardRecord): string {
  if (item.pending_reversal_amount_usd > 0) return translate('ui.outstandingReversal')
  if (item.status === 'reversed' || (item.amount_usd > 0 && item.reversed_amount_usd >= item.amount_usd)) return translate('ui.fullyReversed')
  return item.reversed_amount_usd > 0 ? translate('ui.partiallyReversed') : translate('ui.noReversal')
}
export function source(item: ReferralRewardRecord): string {
  if (item.source_order_id) return item.source_order_no || item.source_order_id
  if (item.trigger_point === 'registration') return translate('ui.registrationReward')
  if (item.trigger_point === 'email_verified') return translate('ui.emailVerificationReward')
  return translate('ui.sourceOrderWasNotRecorded')
}
export function orderStatus(value: string): string {
  return ({ pending: '待付款', paid: '已付款', credited: '已入账', refunded: '已退款', failed: translate('ui.failed28384d7'), cancelled: translate('ui.canceled'), expired: translate('ui.expired') } as Record<string, string>)[value] || value
}
export function refundStatus(value: string): string {
  return ({ pending: translate('ui.pendingApproval'), processing: translate('ui.processing694b71b'), succeeded: '已成功', failed: translate('ui.failed28384d7'), cancelled: translate('ui.canceled') } as Record<string, string>)[value] || value
}
