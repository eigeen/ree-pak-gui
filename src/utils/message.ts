import { ElMessage } from "element-plus"

// global message invoker
export const ShowError = (message: any) => {
  ElMessage({
    showClose: true,
    message,
    type: 'error'
  })
  console.error('[ERROR]', message)
}

export const ShowWarn = (message: any) => {
  ElMessage({
    showClose: true,
    message,
    type: 'warning'
  })
  console.warn('[WARN]', message)
}

export const ShowInfo = (message: any) => {
  ElMessage({
    showClose: true,
    message,
    type: 'info'
  })
  console.info('[INFO]', message)
}
