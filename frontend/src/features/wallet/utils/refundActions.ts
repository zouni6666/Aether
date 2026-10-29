import type { RefundRequest } from '@/api/wallet'

// 退款与充值订单在各状态下允许的管理员操作

export function canProcessRefund(status: string) {
  return status === 'pending_approval' || status === 'approved'
}

export function canFailRefund(refund: Pick<RefundRequest, 'status' | 'refund_mode' | 'gateway_refund_id' | 'payout_proof'>) {
  if (refund.status === 'pending_approval' || refund.status === 'approved') return true
  return refund.status === 'processing'
    && refund.refund_mode === 'offline_payout'
    && !refund.gateway_refund_id?.trim()
    && !refund.payout_proof
}

export function canCompleteRefund(status: string) {
  return status === 'processing'
}

export function canCreditOrder(status: string) {
  return status === 'pending' || status === 'paid'
}

export function canExpireOrder(status: string) {
  return status === 'pending'
}

export function canFailOrder(status: string) {
  return status !== 'credited' && status !== 'refunded'
}
