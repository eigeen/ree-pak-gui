import type { Read, BufRead } from './read'

const DEFAULT_BUF_SIZE = 8192

export class BufReader implements BufRead {
  private inner: Read
  private buf: Uint8Array
  private pos = 0
  private cap = 0
  private eof = false

  constructor(inner: Read, bufferSize = DEFAULT_BUF_SIZE) {
    this.inner = inner
    this.buf = new Uint8Array(bufferSize)
  }

  async fillBuf(): Promise<void> {
    if (this.pos >= this.cap && !this.eof) {
      const n = await this.inner.read(this.buf)
      if (n === null) {
        this.eof = true
      } else {
        this.cap = n
        this.pos = 0
      }
    }
  }

  get buffer(): Uint8Array {
    return this.buf.subarray(this.pos, this.cap)
  }

  consume(amount: number): void {
    this.pos = Math.min(this.pos + amount, this.cap)
  }

  async read(output: Uint8Array): Promise<number | null> {
    if (this.pos >= this.cap) {
      if (this.eof) return null
      await this.fillBuf()
      if (this.pos >= this.cap) return null
    }

    const n = Math.min(output.length, this.cap - this.pos)
    output.set(this.buf.subarray(this.pos, this.pos + n))
    this.pos += n
    return n
  }

  get position(): number {
    return this.inner.position - (this.cap - this.pos)
  }

  async seek(offset: number, whence = 0): Promise<number> {
    this.discardBuf()
    return this.inner.seek(offset, whence)
  }

  private discardBuf(): void {
    this.pos = 0
    this.cap = 0
  }

  // 扩展方法：行迭代器
  async *lines(): AsyncIterableIterator<string> {
    const decoder = new TextDecoder()
    let lineBuffer = new Uint8Array(1024)
    let linePos = 0

    while (true) {
      await this.fillBuf()
      const buf = this.buffer
      if (buf.length === 0) break

      const newlineIdx = buf.indexOf(10) // '\n'
      if (newlineIdx >= 0) {
        // 找到换行符
        const lineBytes = buf.subarray(0, newlineIdx)
        this.consume(newlineIdx + 1)

        // 处理可能的回车符
        const line = decoder.decode(lineBytes)
        yield line.endsWith('\r') ? line.slice(0, -1) : line
      } else {
        // 未找到换行符，追加到行缓冲区
        const newLineBuffer = new Uint8Array(linePos + buf.length)
        newLineBuffer.set(lineBuffer.subarray(0, linePos))
        newLineBuffer.set(buf, linePos)
        lineBuffer = newLineBuffer
        linePos += buf.length
        this.consume(buf.length)
      }
    }

    // 处理最后一行
    if (linePos > 0) {
      yield decoder.decode(lineBuffer.subarray(0, linePos))
    }
  }
}
