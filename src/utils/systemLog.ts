import { useSystemLogStore, type SystemLogLevel } from '@/store/system'

export function pushSystemLog(level: SystemLogLevel, message: string) {
  const systemLogStore = useSystemLogStore()
  systemLogStore.append(level, message)
}

export function pushSystemInfo(message: string) {
  pushSystemLog('info', message)
}

export function pushSystemWarn(message: string) {
  pushSystemLog('warn', message)
}

export function pushSystemError(message: string) {
  pushSystemLog('error', message)
}

export function pushSystemDebug(message: string) {
  pushSystemLog('debug', message)
}
