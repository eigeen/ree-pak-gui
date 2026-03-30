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

export function getExtractRelativeRoot(path: string): string {
  const segments = splitNormalizedPath(path)
  const nativesIndex = segments.findIndex((segment) => segment.toLowerCase() === 'natives')
  const nextSegment = nativesIndex >= 0 ? segments[nativesIndex + 1] : undefined

  if (nativesIndex >= 0 && nextSegment && nextSegment.toLowerCase() === 'stm') {
    return segments.slice(0, nativesIndex + 2).join('/')
  }

  return ''
}
