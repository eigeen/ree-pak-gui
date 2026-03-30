import { reactive, readonly } from 'vue'
import { ShowWarn } from '@/utils/message'

export type TaskProgressStatus = 'idle' | 'running' | 'success' | 'error'

export type TaskProgressStartPayload = {
  taskId: string
  title: string
  progressLabel: string
  runningDescription: string
  successDescription: string
  terminatedDescription: string
  closeLabel: string
  terminateLabel: string
  confirmTitle: string
  confirmDescription: string
  busyMessage: string
  placeholder?: string
  onTerminate?: () => Promise<void> | void
}

export type TaskProgressUpdatePayload = {
  taskId: string
  currentFile?: string
  finishFileCount?: number
  totalFileCount?: number
  description?: string
}

export type TaskProgressFinishPayload = {
  taskId: string
  status: Exclude<TaskProgressStatus, 'idle' | 'running'>
  currentFile?: string
  finishFileCount?: number
  totalFileCount?: number
  description?: string
  errorMessage?: string
}

export type TaskProgressEvent =
  | { type: 'start'; payload: TaskProgressStartPayload }
  | { type: 'update'; payload: TaskProgressUpdatePayload }
  | { type: 'finish'; payload: TaskProgressFinishPayload }
  | { type: 'reset'; payload?: { taskId?: string } }

type TaskProgressState = {
  visible: boolean
  taskId: string
  title: string
  progressLabel: string
  runningDescription: string
  successDescription: string
  terminatedDescription: string
  description: string
  currentFile: string
  finishFileCount: number
  totalFileCount: number
  working: boolean
  status: TaskProgressStatus
  errorMessage: string
  closeLabel: string
  terminateLabel: string
  confirmTitle: string
  confirmDescription: string
  placeholder: string
  busyMessage: string
  onTerminate?: () => Promise<void> | void
}

const EMPTY_STATE: TaskProgressState = {
  visible: false,
  taskId: '',
  title: '',
  progressLabel: '',
  runningDescription: '',
  successDescription: '',
  terminatedDescription: '',
  description: '',
  currentFile: '',
  finishFileCount: 0,
  totalFileCount: 0,
  working: false,
  status: 'idle',
  errorMessage: '',
  closeLabel: 'Close',
  terminateLabel: 'Terminate',
  confirmTitle: 'Terminate',
  confirmDescription: '',
  placeholder: '—',
  busyMessage: ''
}

const state = reactive<TaskProgressState>({ ...EMPTY_STATE })

let nextTaskId = 0

export function useTaskProgressState() {
  return readonly(state)
}

export function createTaskProgressId(prefix = 'task') {
  nextTaskId += 1
  return `${prefix}-${nextTaskId}`
}

export function isTaskProgressRunning() {
  return state.working
}

export function ensureTaskProgressIdle(message?: string) {
  if (!state.working) {
    return true
  }

  ShowWarn(message || state.busyMessage || '已有任务正在进行，请稍候。')
  return false
}

export function dispatchTaskProgressEvent(event: TaskProgressEvent) {
  switch (event.type) {
    case 'start':
      if (state.working) {
        return false
      }

      Object.assign(state, {
        ...EMPTY_STATE,
        visible: true,
        taskId: event.payload.taskId,
        title: event.payload.title,
        progressLabel: event.payload.progressLabel,
        runningDescription: event.payload.runningDescription,
        successDescription: event.payload.successDescription,
        terminatedDescription: event.payload.terminatedDescription,
        description: event.payload.runningDescription,
        working: true,
        status: 'running',
        closeLabel: event.payload.closeLabel,
        terminateLabel: event.payload.terminateLabel,
        confirmTitle: event.payload.confirmTitle,
        confirmDescription: event.payload.confirmDescription,
        placeholder: event.payload.placeholder ?? EMPTY_STATE.placeholder,
        busyMessage: event.payload.busyMessage,
        onTerminate: event.payload.onTerminate
      })
      return true

    case 'update':
      if (event.payload.taskId !== state.taskId) {
        return false
      }

      if (event.payload.currentFile !== undefined) {
        state.currentFile = event.payload.currentFile
      }
      if (event.payload.finishFileCount !== undefined) {
        state.finishFileCount = event.payload.finishFileCount
      }
      if (event.payload.totalFileCount !== undefined) {
        state.totalFileCount = event.payload.totalFileCount
      }
      if (event.payload.description !== undefined) {
        state.description = event.payload.description
      }
      return true

    case 'finish':
      if (event.payload.taskId !== state.taskId) {
        return false
      }

      state.visible = true
      state.working = false
      state.status = event.payload.status
      if (event.payload.currentFile !== undefined) {
        state.currentFile = event.payload.currentFile
      }
      if (event.payload.finishFileCount !== undefined) {
        state.finishFileCount = event.payload.finishFileCount
      }
      if (event.payload.totalFileCount !== undefined) {
        state.totalFileCount = event.payload.totalFileCount
      }
      state.errorMessage = event.payload.errorMessage ?? ''
      state.description =
        event.payload.description ??
        (event.payload.status === 'success'
          ? state.successDescription
          : state.terminatedDescription)
      state.onTerminate = undefined
      return true

    case 'reset':
      if (event.payload?.taskId && event.payload.taskId !== state.taskId) {
        return false
      }

      Object.assign(state, { ...EMPTY_STATE })
      return true
  }
}

export function tryStartTaskProgress(
  payload: Omit<TaskProgressStartPayload, 'taskId'> & {
    taskId?: string
  }
) {
  const taskId = payload.taskId ?? createTaskProgressId()
  const accepted = dispatchTaskProgressEvent({
    type: 'start',
    payload: {
      ...payload,
      taskId
    }
  })

  return accepted ? taskId : null
}

export function updateTaskProgress(
  taskId: string,
  payload: Omit<TaskProgressUpdatePayload, 'taskId'>
) {
  return dispatchTaskProgressEvent({
    type: 'update',
    payload: {
      taskId,
      ...payload
    }
  })
}

export function finishTaskProgress(
  taskId: string,
  payload: Omit<TaskProgressFinishPayload, 'taskId'>
) {
  return dispatchTaskProgressEvent({
    type: 'finish',
    payload: {
      taskId,
      ...payload
    }
  })
}

export function resetTaskProgress(taskId?: string) {
  return dispatchTaskProgressEvent({
    type: 'reset',
    payload: taskId ? { taskId } : undefined
  })
}

export function closeTaskProgressPanel() {
  state.visible = false
}

export async function terminateTaskProgress() {
  if (!state.working || !state.onTerminate) {
    return
  }

  await state.onTerminate()
}
