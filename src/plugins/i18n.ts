import { createI18n } from 'vue-i18n'

import en_US from '@/i18n/en_US'
import zh_CN from '@/i18n/zh_CN'

function detectLocale() {
  const lang = navigator.language
  if (lang === 'zh-CN') {
    return 'zh_CN'
  } else if (lang.startsWith('en')) {
    return 'en'
  }

  return 'en'
}

export default createI18n({
  locale: detectLocale(),
  fallbackLocale: 'en',
  messages: {
    en: en_US.messages,
    zh_CN: zh_CN.messages
  }
})
