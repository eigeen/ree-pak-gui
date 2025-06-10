import type { FileHandle } from "@tauri-apps/plugin-fs"

export interface Read {
  /**
   * 读取数据到缓冲区
   * @param buffer 目标缓冲区
   * @returns 实际读取的字节数，null表示EOF
   */
  read(buffer: Uint8Array): Promise<number | null>

  /**
   * 获取读取位置
   */
  get position(): number

  /**
   * 设置读取位置
   * @param offset 偏移量
   * @param whence 参考位置 (0=起始，1=当前，2=末尾)
   */
  seek(offset: number, whence?: number): Promise<number>
}

export interface BufRead extends Read {
  /**
   * 填充内部缓冲区
   */
  fillBuf(): Promise<void>

  /**
   * 获取缓冲区的可用数据
   */
  get buffer(): Uint8Array

  /**
   * 消费指定数量的字节
   * @param amount 要消费的字节数
   */
  consume(amount: number): void
}

export class FileHandleReader implements Read {
  private inner: FileHandle
  private pos = 0

  constructor(inner: FileHandle) {
    this.inner = inner
  }

  async read(buffer: Uint8Array): Promise<number | null> {
    const bytesRead = await this.inner.read(buffer)
    if (bytesRead !== null) {
      this.pos += bytesRead
    }
    return bytesRead
  }

  get position(): number {
    return this.pos
  }

  async seek(offset: number, whence = 0): Promise<number> {
    await this.inner.seek(offset, whence)
    this.pos = offset
    return this.pos
  }
}