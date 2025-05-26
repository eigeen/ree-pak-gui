// import { writeBinaryFile, removeFile } from '@tauri-apps/api/fs';

import { remove, writeFile } from "@tauri-apps/plugin-fs";

type DownloadOptions = {
  overwrite?: boolean;
};

class DownloadController {
  private abortController: AbortController | null = null;
  private isDownloading = false;
  private progressCallbacks: Array<(loaded: number, total: number | null) => void> = [];

  async download(url: string, filePath: string, options: DownloadOptions = {}) {
    const { overwrite = true } = options;

    if (this.isDownloading) {
      throw new Error('A download is already in progress.');
    }

    this.isDownloading = true;
    this.abortController = new AbortController();

    try {
      if (overwrite) {
        try {
          await remove(filePath);
        } catch (error) {
          if (!(error instanceof Error && error.message.includes('os error 2'))) {
            throw error;
          }
        }
      }

      const response = await fetch(url, { signal: this.abortController.signal });
      if (!response.ok) {
        throw new Error(`Failed to download: ${response.status} ${response.statusText}`);
      }

      const total = parseInt(response.headers.get('Content-Length') || '0', 10) || null;
      let loaded = 0;

      const reader = response.body?.getReader();
      if (!reader) throw new Error('Failed to get response body reader');

      while (this.isDownloading) {
        const { done, value } = await reader.read();
        if (done) break;

        loaded += value.length;
        await writeFile(filePath, value, { append: true });
        this.progressCallbacks.forEach(callback => callback(loaded, total));
      }

      this.isDownloading = false;
      this.abortController = null;
    } catch (error) {
      this.isDownloading = false;
      this.abortController = null;

      if ((error as Error).name === 'AbortError') {
        console.log('Download aborted');
      } else {
        throw error;
      }
    }
  }

  abort() {
    if (this.isDownloading && this.abortController) {
      this.abortController.abort();
      this.isDownloading = false;
    }
  }

  onProgress(callback: (loaded: number, total: number | null) => void) {
    this.progressCallbacks.push(callback);
  }

  offProgress(callback: (loaded: number, total: number | null) => void) {
    this.progressCallbacks = this.progressCallbacks.filter(cb => cb !== callback);
  }
}

// 使用示例
// const downloader = new DownloadController();
// downloader.onProgress((loaded, total) => {
//   console.log(`Downloaded ${loaded} bytes of ${total ?? 'unknown'}`);
// });
// 
// downloader.download('https://example.com/file.zip', 'file.zip')
//   .then(() => console.log('Download completed'))
//   .catch(err => console.error('Download failed:', err));
// 
// // 要中止下载：
// downloader.abort();