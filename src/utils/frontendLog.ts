import { pushSystemDebug, pushSystemError, pushSystemInfo, pushSystemWarn } from '@/utils/systemLog'

type FrontendLogLevel = 'debug' | 'info' | 'warn' | 'error'

function formatElapsed(startedAt: number) {
  return `${Math.round(performance.now() - startedAt)} ms`
}

function stringifyError(error: unknown) {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}

function writeLog(level: FrontendLogLevel, scope: string, message: string) {
  const text = `[frontend:${scope}] ${message}`

  switch (level) {
    case 'debug':
      pushSystemDebug(text)
      console.debug(text)
      break
    case 'warn':
      pushSystemWarn(text)
      console.warn(text)
      break
    case 'error':
      pushSystemError(text)
      console.error(text)
      break
    default:
      pushSystemInfo(text)
      console.info(text)
      break
  }
}

export function logFrontendDebug(scope: string, message: string) {
  writeLog('debug', scope, message)
}

export function logFrontendInfo(scope: string, message: string) {
  writeLog('info', scope, message)
}

export function logFrontendWarn(scope: string, message: string) {
  writeLog('warn', scope, message)
}

export function logFrontendError(scope: string, message: string, error?: unknown) {
  writeLog(
    'error',
    scope,
    error === undefined ? message : `${message} error=${stringifyError(error)}`
  )
}

export async function runLoggedTask<T>(
  scope: string,
  task: () => Promise<T>,
  options: {
    start?: string
    success?: string | ((result: T) => string)
    failure?: string | ((error: unknown) => string)
    successLevel?: Exclude<FrontendLogLevel, 'error'>
  } = {}
): Promise<T> {
  const startedAt = performance.now()
  const { start = 'start', success = 'done', failure = 'failed', successLevel = 'info' } = options

  logFrontendInfo(scope, start)

  try {
    const result = await task()
    const successMessage = typeof success === 'function' ? success(result) : success
    writeLog(successLevel, scope, `${successMessage} elapsed=${formatElapsed(startedAt)}`)
    return result
  } catch (error) {
    const failureMessage = typeof failure === 'function' ? failure(error) : failure
    logFrontendError(scope, `${failureMessage} elapsed=${formatElapsed(startedAt)}`, error)
    throw error
  }
}
