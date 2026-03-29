const reAssetSuffixTags = new Set([
  'x64',
  'stm',
  'nsw',
  'msg',
  'ja',
  'en',
  'fr',
  'it',
  'de',
  'es',
  'ru',
  'pl',
  'nl',
  'pt',
  'ptbr',
  'ko',
  'zhtw',
  'zhcn',
  'fi',
  'sv',
  'da',
  'no',
  'cs',
  'hu',
  'sk',
  'ar',
  'tr',
  'bu',
  'gr',
  'ro',
  'th',
  'uk',
  'vi',
  'id',
  'fc',
  'hi',
  'es419'
])

type LastDotSegment = {
  segment: string
  nextEnd: number
}

export function getReAssetExtension(
  pathOrName: string,
  knownExtensions?: ReadonlySet<string>
): string {
  if (typeof pathOrName !== 'string') {
    return ''
  }

  const fileName = getLastPathSegment(pathOrName)
  if (!fileName) {
    return ''
  }

  const rawExtension = getLastDotSegment(fileName)
  if (!rawExtension) {
    return ''
  }

  const rawExtensionLower = rawExtension.segment.toLowerCase()
  if (isAsciiDigits(rawExtension.segment)) {
    return getLastDotSegment(fileName, rawExtension.nextEnd)?.segment.toLowerCase() ?? ''
  }

  let cursor = rawExtension.nextEnd
  let strippedTag = false
  let candidate = rawExtension

  while (reAssetSuffixTags.has(candidate.segment.toLowerCase())) {
    strippedTag = true
    if (candidate.nextEnd <= 0) {
      break
    }

    const previous = getLastDotSegment(fileName, candidate.nextEnd)
    if (!previous) {
      break
    }

    candidate = previous
    cursor = previous.nextEnd
  }

  if (strippedTag) {
    if (isAsciiDigits(candidate.segment)) {
      return getLastDotSegment(fileName, cursor)?.segment.toLowerCase() ?? ''
    }

    const candidateLower = candidate.segment.toLowerCase()
    if (knownExtensions?.has(candidateLower)) {
      return candidateLower
    }
  }

  return rawExtensionLower
}

function getLastPathSegment(pathOrName: string): string {
  let end = pathOrName.length
  while (end > 0) {
    const char = pathOrName.charCodeAt(end - 1)
    if (char !== 47 && char !== 92) {
      break
    }
    end -= 1
  }

  if (end === 0) {
    return ''
  }

  let start = end - 1
  while (start > 0) {
    const char = pathOrName.charCodeAt(start - 1)
    if (char === 47 || char === 92) {
      break
    }
    start -= 1
  }

  return pathOrName.slice(start, end)
}

function getLastDotSegment(value: string, end = value.length): LastDotSegment | null {
  if (end <= 0) {
    return null
  }

  const dot = value.lastIndexOf('.', end - 1)
  if (dot < 0) {
    return null
  }

  return {
    segment: value.slice(dot + 1, end),
    nextEnd: dot
  }
}

function isAsciiDigits(value: string): boolean {
  if (value.length === 0) {
    return false
  }

  for (let i = 0; i < value.length; i += 1) {
    const code = value.charCodeAt(i)
    if (code < 48 || code > 57) {
      return false
    }
  }

  return true
}
