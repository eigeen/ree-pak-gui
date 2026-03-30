import { createI18n } from 'vue-i18n'

import en_US from '@/i18n/en_US'
import zh_CN from '@/i18n/zh_CN'
import { detectLocale, resolveLocale, type AppLocale } from '@/lib/language'

const i18n = createI18n({
  legacy: false,
  locale: detectLocale(),
  fallbackLocale: 'en',
  messages: {
    en: en_US.messages,
    zh_CN: zh_CN.messages
  }
})

export function setAppLocale(value?: string | null): AppLocale {
  const locale = resolveLocale(value)
  i18n.global.locale.value = locale
  return locale
}

export default i18n
