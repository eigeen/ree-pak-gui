import { fetch } from '@tauri-apps/plugin-http'

export interface DownloadOptions {
  connectTimeout?: number
  // Minimal download speed (KB/s)
  minSpeed?: number
  // Speed check interval (ms)
  checkInterval?: number
  // Max stall time (ms)
  maxStallTime?: number
}

/**
 * 带下载速度检测的 fetch 请求
 * @param url - 下载地址
 * @param options - fetch 配置项
 * @param minSpeed - 最低速度要求 (KB/s)
 * @param checkInterval - 速度检测间隔 (ms)
 * @param maxStallTime - 最大允许卡顿时长 (ms)
 */
export async function fetchWithSpeedCheck(
  url: string,
  options: DownloadOptions = {},
  onEvent?: (event: ProgressEvent) => Promise<void>
): Promise<Blob> {
  const {
    connectTimeout = 10000,
    minSpeed = 50,
    checkInterval = 2000,
    maxStallTime = 10000
  } = options

  const controller = new AbortController()
  const signal = controller.signal

  // 初始化下载监控
  let downloadedBytes = 0
  let lastCheckBytes = 0
  let lastCheckTime = Date.now()
  let lastProgressTime = Date.now()

  // 超时监控定时器
  const speedMonitor = setInterval(() => {
    const currentTime = Date.now()
    const elapsed = currentTime - lastCheckTime

    // 计算当前下载速度 (KB/s)
    const speed = (((downloadedBytes - lastCheckBytes) / elapsed) * 1000) / 1024

    // 更新检测基准
    lastCheckBytes = downloadedBytes
    lastCheckTime = currentTime

    console.log(`Download speed: ${speed.toFixed(2)}KB/s`)

    // 速度检测逻辑
    const stallTime = currentTime - lastProgressTime
    if (speed < minSpeed && stallTime > maxStallTime) {
      clearInterval(speedMonitor)
      controller.abort(`Download speed too slow (${speed.toFixed(2)}KB/s < ${minSpeed}KB/s)`)
    }

    // 如果有下载进展，更新最后有效时间
    if (speed > 0) {
      lastProgressTime = currentTime
    }
  }, checkInterval)

  try {
    const response = await fetch(url, { method: 'GET', signal, connectTimeout })

    if (!response.ok) {
      throw new Error(`Http error: ${response.status}`)
    }

    const reader = response.body?.getReader()
    const contentLength = response.headers.get('Content-Length')
    const totalBytes = contentLength ? parseInt(contentLength) : null

    if (!reader || !totalBytes) {
      throw new Error('Response empty.')
    }

    await onEvent?.(new ProgressEvent('loadstart'))

    let chunks = []
    while (true) {
      const { done, value } = await reader.read()

      if (done) {
        await onEvent?.(new ProgressEvent('loadend'))
        break
      }

      // 更新下载字节数
      downloadedBytes += value.length
      chunks.push(value)
      await onEvent?.(
        new ProgressEvent('load', {
          loaded: downloadedBytes,
          total: totalBytes
        })
      )
    }

    clearInterval(speedMonitor)
    return new Blob(chunks)
  } catch (err: any) {
    clearInterval(speedMonitor)
    if (err.name === 'AbortError') {
      throw new Error(`Download timeout: ${err.message}`)
    }
    throw err
  }
}
