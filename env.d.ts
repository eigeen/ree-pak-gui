/// <reference types="vite/client" />

declare module 'markdown-it/dist/index.cjs.js' {
  type MarkdownItOptions = {
    html?: boolean
    breaks?: boolean
    linkify?: boolean
  }

  export default class MarkdownIt {
    constructor(options?: MarkdownItOptions)
    render(source: string): string
  }
}
