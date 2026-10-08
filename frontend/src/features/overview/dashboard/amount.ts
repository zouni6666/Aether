import type { OverviewAmount } from '@/api/overview'

export function amountValue(amount: OverviewAmount | null | undefined): number | null {
  if (amount?.value == null || amount.status === 'unknown') return null
  const value = Number(amount.value)
  return Number.isFinite(value) ? value : null
}

export function amountStatus(amount: OverviewAmount | null | undefined, t: (zh: string, en: string) => string): string {
  if (amountValue(amount) === null) return t('金额未知', 'Amount unknown')
  if (amount?.status === 'known_subtotal') return t('已知小计', 'Known subtotal')
  if (amount?.status === 'estimated_subtotal') return t('已知范围估算', 'Estimate within known coverage')
  if (amount?.status === 'estimated') return t('估算', 'Estimated')
  return ''
}
