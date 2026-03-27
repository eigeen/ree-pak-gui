/**
 * Generates shadcn-vue component docs from GitHub repository
 * Run: npx -y tsx skills/shadcn-vue/scripts/generate-shadcn-components.ts
 *
 * Creates:
 *   - references/components.md (index)
 *   - components/<component>.md (per-component details)
 */

import { mkdirSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const REPO = 'unovue/shadcn-vue'
const BRANCH = 'dev'
const API_URL = `https://api.github.com/repos/${REPO}/contents/apps/v4/content/docs/components?ref=${BRANCH}`
const BASE_RAW_URL = `https://raw.githubusercontent.com/${REPO}/${BRANCH}`

interface ComponentMeta {
  name: string
  title: string
  description: string
  component: boolean
  links?: {
    doc?: string
    api?: string
  }
  content: string
}

interface GitHubFile {
  name: string
  path: string
  sha: string
  size: number
  url: string
  html_url: string
  git_url: string
  download_url: string
  type: string
  _links: {
    self: string
    git: string
    html: string
  }
}

async function fetchComponentList(): Promise<GitHubFile[]> {
  try {
    const res = await fetch(API_URL)
    if (!res.ok)
      throw new Error(`Failed to fetch component list: ${res.statusText}`)
    const data = await res.json()
    return data.filter((f: GitHubFile) => f.type === 'file' && f.name.endsWith('.md'))
  }
  catch (error) {
    console.error('Error fetching component list:', error)
    return []
  }
}

function parseFrontmatter(content: string): { meta: any, content: string } {
  const frontmatterRegex = /^---\n([\s\S]*?)\n---\n/
  const match = content.match(frontmatterRegex)
  
  if (!match)
    return { meta: {}, content }
  
  const frontmatter = match[1]
  const meta: any = {}
  const lines = frontmatter.split('\n')
  
  for (const line of lines) {
    if (line.trim() === '')
      continue
    const colonIndex = line.indexOf(':')
    if (colonIndex === -1)
      continue
    const key = line.slice(0, colonIndex).trim()
    let value = line.slice(colonIndex + 1).trim()
    
    if (value === 'true')
      value = true
    else if (value === 'false')
      value = false
    
    if (key === 'links') {
      meta[key] = {}
      continue
    }
    
    if (key.startsWith('  ') && meta.links) {
      const linkKey = key.trim()
      meta.links[linkKey] = value
    }
    else {
      meta[key] = value
    }
  }
  
  const remainingContent = content.slice(match[0].length)
  return { meta, content: remainingContent }
}

async function fetchComponent(file: GitHubFile): Promise<ComponentMeta | null> {
  try {
    const res = await fetch(file.download_url)
    if (!res.ok)
      return null
    const content = await res.text()
    const { meta, content: markdownContent } = parseFrontmatter(content)
    
    const componentName = file.name.replace('.md', '')
    
    return {
      name: componentName,
      title: meta.title || componentName,
      description: meta.description || '',
      component: meta.component || false,
      links: meta.links,
      content: markdownContent,
    }
  }
  catch {
    return null
  }
}

function escapeMarkdown(str: string): string {
  return str.replace(/\|/g, '\\|').replace(/\n/g, ' ')
}

async function main() {
  const __dirname = dirname(fileURLToPath(import.meta.url))
  const baseDir = join(__dirname, '..')
  const componentsDir = join(baseDir, 'components')
  mkdirSync(componentsDir, { recursive: true })

  console.log('Generating shadcn-vue component docs...')
  
  const files = await fetchComponentList()
  console.log(`Found ${files.length} component files`)

  const components: ComponentMeta[] = []
  
  for (const file of files) {
    console.log(`Fetching ${file.name}...`)
    const component = await fetchComponent(file)
    if (component)
      components.push(component)
  }

  console.log(`Successfully parsed ${components.length} components`)

  const sortedComponents = components.sort((a, b) => a.name.localeCompare(b.name))

  const index: string[] = []
  index.push('# Shadcn Vue Components')
  index.push('')
  index.push(`> Total components: ${components.length}`)
  index.push('')
  index.push('| Component | Description | File |')
  index.push('|-----------|-------------|------|')

  for (const comp of sortedComponents) {
    const file = `components/${comp.name}.md`
    const badge = comp.component ? '`component`' : ''
    const links = comp.links ? `[doc](${comp.links.doc})` : ''
    index.push(`| **${comp.title}** | ${escapeMarkdown(comp.description)} ${badge} ${links} | \`${file}\` |`)
  }

  index.push('')

  writeFileSync(join(baseDir, 'references/components.md'), index.join('\n'))
  console.log('✓ Generated references/components.md (index)')

  for (const comp of sortedComponents) {
    const lines: string[] = []
    lines.push(`# ${comp.title}`)
    lines.push('')
    lines.push(`**Description:** ${comp.description}`)
    lines.push('')
    
    if (comp.links) {
      const links: string[] = []
      if (comp.links.doc)
        links.push(`[Documentation](${comp.links.doc})`)
      if (comp.links.api)
        links.push(`[API Reference](${comp.links.api})`)
      if (links.length > 0) {
        lines.push(`**Links:** ${links.join(' | ')}`)
        lines.push('')
      }
    }
    
    lines.push('---')
    lines.push('')
    lines.push(comp.content)
    
    const filename = `${comp.name}.md`
    writeFileSync(join(componentsDir, filename), lines.join('\n'))
    console.log(`✓ Generated components/${filename}`)
  }

  console.log(`\nDone! Generated ${sortedComponents.length + 1} files.`)
}

main().catch(console.error)
