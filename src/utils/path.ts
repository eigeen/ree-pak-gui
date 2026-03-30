export function getParentPath(path: string): string | null {
  // '/' or '\'
  const lastSlashIndex = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'))
  if (lastSlashIndex === -1) {
    return null
  }
  return path.substring(0, lastSlashIndex)
}

export function getFileName(path: string): string {
  let lastSlashIndex = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'))
  return path.substring(lastSlashIndex + 1)
}

export function getFileStem(path: string): string {
  const fileName = getFileName(path)
  const firstDotIndex = fileName.indexOf('.')
  if (firstDotIndex === -1) {
    return fileName
  }
  return fileName.substring(0, firstDotIndex)
}

export function normalizeDisplayPath(path: string): string {
  return path.replace(/\\/g, '/').replace(/\s*\/\s*/g, '/').trim()
}

export function normalizePathForSegments(path: string): string {
  return normalizeDisplayPath(path).replace(/\/+$/g, '')
}

export function splitNormalizedPath(path: string): string[] {
  return normalizePathForSegments(path).split('/').filter(Boolean)
}

export function getSelectedItemRelativeRoot(path: string): string {
  const normalizedPath = normalizePathForSegments(path)
  if (!normalizedPath) {
    return ''
  }

  return getParentPath(normalizedPath) ?? ''
}
