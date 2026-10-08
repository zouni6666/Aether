import type { OverviewAmount } from '@/api/overview'
import { getI18nLocale } from '@/i18n'

export function count(value: number | null | undefined): string {
  return value == null ? '-' : new Intl.NumberFormat(getI18nLocale(), { maximumFractionDigits: 2 }).format(value)
}

export function attributionLabel(kind: string | null | undefined): string {
  const zh = getI18nLocale() === 'zh-CN'
  if (kind === 'employee') return zh ? '成员账号' : 'Member account'
  if (kind === 'standalone') return zh ? '独立余额 Key' : 'Standalone balance key'
  return !kind || kind === 'unknown' ? (zh ? '未知' : 'Unknown') : kind
}

export function money(amount: OverviewAmount | null | undefined): string {
  if (amount?.value == null) return '-'
  const number = Number(amount.value)
  if (!Number.isFinite(number)) return '-'
  return new Intl.NumberFormat(getI18nLocale(), { style: 'currency', currency: amount.currency || 'USD', currencyDisplay: 'narrowSymbol', minimumFractionDigits: 2, maximumFractionDigits: 6 }).format(number)
}

export function percent(value: number | null | undefined): string {
  return value == null ? '-' : `${(value * 100).toFixed(2)}%`
}

export function timestamp(value: string | null | undefined, timezone?: string): string {
  return !value ? '-' : new Intl.DateTimeFormat(getI18nLocale(), { dateStyle: 'short', timeStyle: 'short', timeZone: timezone }).format(new Date(value))
}
