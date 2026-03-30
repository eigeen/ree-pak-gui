export const APP_LOCALE_OPTIONS = [
  { value: 'zh_CN', label: '简体中文' },
  { value: 'en', label: 'English' }
] as const

export type AppLocale = (typeof APP_LOCALE_OPTIONS)[number]['value']

export function isAppLocale(value: string): value is AppLocale {
  return APP_LOCALE_OPTIONS.some((option) => option.value === value)
}

export function sanitizeStoredLocale(value?: string | null): AppLocale | undefined {
  if (!value) {
    return undefined
  }

  return isAppLocale(value) ? value : undefined
}

export function detectLocale(): AppLocale {
  const lang = navigator.language.toLowerCase()

  if (lang === 'zh-cn' || lang.startsWith('zh')) {
    return 'zh_CN'
  }

  if (lang.startsWith('en')) {
    return 'en'
  }

  return 'en'
}

export function resolveLocale(value?: string | null): AppLocale {
  return sanitizeStoredLocale(value) ?? detectLocale()
}
