import { ElMessage } from "element-plus"

// global message invoker
export const ShowError = (message: any) => {
  ElMessage({
    showClose: true,
    message,
    type: 'error',
    duration: 5000
  })
  console.error('[ERROR]', message)
}

export const ShowWarn = (message: any) => {
  ElMessage({
    showClose: true,
    message,
    type: 'warning',
    duration: 5000
  })
  console.warn('[WARN]', message)
}

export const ShowInfo = (message: any) => {
  ElMessage({
    showClose: true,
    message,
    type: 'success',
    duration: 5000
  })
  console.info('[INFO]', message)
}
