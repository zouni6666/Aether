import type { App, InjectionKey, Ref } from 'vue'
import { computed, inject, readonly, ref } from 'vue'
import {
  defaultLocale,
  supportedLocales,
  translateMessage,
  type Locale,
  type MessageKey,
} from './messages'

type Params = Record<string, string | number>

interface I18nContext {
  locale: Ref<Locale>
  setLocale: (locale: Locale) => void
  t: (key: MessageKey, params?: Params) => string
}

const STORAGE_KEY = 'aether_locale'
const i18nKey: InjectionKey<I18nContext> = Symbol('aether-i18n')
const locale = ref<Locale>(readInitialLocale())

function isLocale(value: string | null | undefined): value is Locale {
  return !!value && supportedLocales.includes(value as Locale)
}

function readInitialLocale(): Locale {
  if (typeof window === 'undefined') return defaultLocale

  try {
    const stored = normalizeLocale(localStorage.getItem(STORAGE_KEY))
    if (stored) return stored
  } catch {
    // Browser storage may be unavailable; the in-memory preference still works.
  }

  const preferred = navigator.languages?.find(language => {
    const normalized = normalizeLocale(language)
    return isLocale(normalized)
  })
  const normalizedPreferred = normalizeLocale(preferred)
  return isLocale(normalizedPreferred) ? normalizedPreferred : defaultLocale
}

export function normalizeLocale(value: string | null | undefined): Locale | undefined {
  if (!value) return undefined
  const language = value.trim().toLowerCase().split(/[-_]/)[0]
  if (language === 'zh') return 'zh-CN'
  if (language === 'en') return 'en-US'
  return undefined
}

function setLocale(nextLocale: Locale): void {
  if (!isLocale(nextLocale)) return
  locale.value = nextLocale
  if (typeof document !== 'undefined') {
    document.documentElement.lang = nextLocale
  }
  if (typeof window !== 'undefined') {
    try {
      localStorage.setItem(STORAGE_KEY, nextLocale)
    } catch {
      // Persistence is optional; do not interrupt language switching.
    }
  }
}

export function translate(key: MessageKey, params?: Params): string {
  return translateMessage(key, locale.value, params)
}

const t = translate

const context: I18nContext = {
  locale,
  setLocale,
  t,
}

export function createI18n() {
  return {
    install(app: App) {
      app.provide(i18nKey, context)
      app.config.globalProperties.$t = t
      setLocale(locale.value)
    }
  }
}

export function useI18n() {
  return inject(i18nKey, context)
}

export function setI18nLocale(locale: Locale): void {
  setLocale(locale)
}

export function getI18nLocale(): Locale {
  return locale.value
}

export function useLocaleOptions() {
  const { locale: currentLocale, setLocale: applyLocale } = useI18n()
  const currentLocaleLabel = computed(() => {
    return currentLocale.value === 'zh-CN' ? t('common.chinese') : t('common.english')
  })

  return {
    locale: readonly(currentLocale),
    supportedLocales,
    currentLocaleLabel,
    setLocale: applyLocale,
  }
}

export type { Locale, MessageKey }
