import type { AudioSourceRef } from '@/api/tauri/pak'
import type { ExplorerEntry } from '@/lib/unpackExplorer'
import { getFileName } from '@/utils/path'

const FALLBACK_BANK_DIRECTORY = 'sound-bank'

type AudioBankSourceEntry = Pick<ExplorerEntry, 'hash' | 'belongsTo'>

export function getAudioSourceRef(entry: AudioBankSourceEntry): AudioSourceRef | null {
  if (!entry.hash || !entry.belongsTo) return null
  return {
    hash: entry.hash,
    belongsTo: entry.belongsTo
  }
}

export function resolveAudioBankDirectoryName(
  sourcePath: string | undefined,
  fallbackName: string
) {
  const sourceName = getFileName(sourcePath || fallbackName).trim()
  return sanitizeAudioBankDirectoryName(sourceName || FALLBACK_BANK_DIRECTORY)
}

export function sanitizeAudioBankDirectoryName(value: string) {
  const sanitized = value
    .replace(/[<>:"/\\|?*]/g, '_')
    .split('')
    .map(replaceControlCharacter)
    .join('')
    .replace(/[. ]+$/g, '')
    .trim()

  return sanitized || FALLBACK_BANK_DIRECTORY
}

function replaceControlCharacter(value: string) {
  return value.charCodeAt(0) < 32 ? '_' : value
}
