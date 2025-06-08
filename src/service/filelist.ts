type SourceType = 'local' | 'remote'

interface FileListSource {
  identifier: string
  sourceType: SourceType
  filePath: string
}

interface FileListMetadata {
  identifier: string
  file_name: string
  game: string
  platform: string
  tags: string[]
  description: string
}

export class FileListService {
  private fileList: string[] = []

//   public async getLocalFileList(): FileListSource[] {
//     return this.fileList
//   }

//   public async loadLocalFileList(): Promise<void> {

//   }
}
