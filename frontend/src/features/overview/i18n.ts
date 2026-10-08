import { useI18n } from '@/i18n'

export function useOverviewI18n() {
  const { locale } = useI18n()
  return { t: (zh: string, en: string) => locale.value === 'zh-CN' ? zh : en }
}
