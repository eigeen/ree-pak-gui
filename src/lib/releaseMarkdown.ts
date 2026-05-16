type MarkdownAttribute = [string, string]

type MarkdownToken = {
  attrs: MarkdownAttribute[]
  children: MarkdownToken[]
  content: string
}

type MarkdownInlineState = {
  env: unknown
  md: {
    inline: {
      parse(
        source: string,
        md: MarkdownInlineState['md'],
        env: unknown,
        tokens: MarkdownToken[]
      ): void
    }
  }
  pos: number
  posMax: number
  push(type: string, tag: string, nesting: number): MarkdownToken
  src: string
}

type MarkdownInlineRule = (state: MarkdownInlineState, silent: boolean) => boolean

type MarkdownRenderer = {
  inline: {
    ruler: {
      before(beforeName: string, ruleName: string, rule: MarkdownInlineRule): void
    }
  }
  render(source: string): string
}

const GITHUB_ATTACHMENT_HOST = 'github.com'
const GITHUB_ATTACHMENT_PATH_PREFIX = '/user-attachments/assets/'
const HTML_IMAGE_TAG_PATTERN = /^<img\b(?:"[^"]*"|'[^']*'|[^'">])*>/i
const HTML_ATTRIBUTE_PATTERN = /\s([^\s"'<>/=]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?/g
const IMAGE_DIMENSION_PATTERN = /^[1-9]\d{0,4}$/

/** Adds support for GitHub Release image tags without enabling arbitrary HTML. */
export function addGithubImageTagRule(markdown: MarkdownRenderer): void {
  markdown.inline.ruler.before('html_inline', 'github_image_tag', renderGithubImageTag)
}

function renderGithubImageTag(state: MarkdownInlineState, silent: boolean): boolean {
  const tag = readImageTag(state)
  if (!tag) {
    return false
  }

  const image = toGithubImageAttributes(parseHtmlAttributes(tag))
  if (!image) {
    return false
  }

  if (!silent) {
    const token = state.push('image', 'img', 0)
    const alt = image.get('alt') ?? ''
    token.attrs = buildMarkdownImageAttributes(image)
    token.children = parseInlineAltText(state, alt)
    token.content = alt
  }
  state.pos += tag.length
  return true
}

function readImageTag(state: MarkdownInlineState): string | null {
  if (state.src.charCodeAt(state.pos) !== 0x3c) {
    return null
  }

  const match = state.src.slice(state.pos, state.posMax).match(HTML_IMAGE_TAG_PATTERN)
  return match?.[0] ?? null
}

function parseHtmlAttributes(tag: string): Map<string, string> {
  const attributes = new Map<string, string>()
  for (const match of tag.matchAll(HTML_ATTRIBUTE_PATTERN)) {
    const name = match[1]
    if (name) {
      attributes.set(name.toLowerCase(), match[2] ?? match[3] ?? match[4] ?? '')
    }
  }
  return attributes
}

function toGithubImageAttributes(attributes: Map<string, string>): Map<string, string> | null {
  const src = normalizeGithubAttachmentUrl(attributes.get('src'))
  if (!src) {
    return null
  }

  const image = new Map<string, string>([['src', src]])
  addOptionalAttribute(image, 'alt', attributes.get('alt') ?? 'image')
  addOptionalDimension(image, 'width', attributes.get('width'))
  addOptionalDimension(image, 'height', attributes.get('height'))
  return image
}

function normalizeGithubAttachmentUrl(src?: string): string | null {
  try {
    const url = new URL(src ?? '')
    return isGithubAttachmentUrl(url) ? url.href : null
  } catch {
    return null
  }
}

function isGithubAttachmentUrl(url: URL): boolean {
  return (
    url.protocol === 'https:' &&
    url.hostname === GITHUB_ATTACHMENT_HOST &&
    url.pathname.startsWith(GITHUB_ATTACHMENT_PATH_PREFIX)
  )
}

function addOptionalAttribute(target: Map<string, string>, name: string, value: string): void {
  if (value) {
    target.set(name, value)
  }
}

function addOptionalDimension(target: Map<string, string>, name: string, value?: string): void {
  if (value && IMAGE_DIMENSION_PATTERN.test(value)) {
    target.set(name, value)
  }
}

function buildMarkdownImageAttributes(image: Map<string, string>): MarkdownAttribute[] {
  return [
    ['src', image.get('src') ?? ''],
    ['alt', image.get('alt') ?? ''],
    ...readOptionalAttribute(image, 'width'),
    ...readOptionalAttribute(image, 'height'),
    ['loading', 'lazy'],
    ['decoding', 'async']
  ]
}

function parseInlineAltText(state: MarkdownInlineState, alt: string): MarkdownToken[] {
  const tokens: MarkdownToken[] = []
  state.md.inline.parse(alt, state.md, state.env, tokens)
  return tokens
}

function readOptionalAttribute(image: Map<string, string>, name: string): MarkdownAttribute[] {
  const value = image.get(name)
  return value ? [[name, value]] : []
}
