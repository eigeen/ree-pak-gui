export default {
  messages: {
    menu: {
      slogan: 'REE Pak Tool',
      unpack: '解包',
      repack: '打包'
    },
    unpack: {
      fileList: '文件列表',
      pakFiles: 'Pak 文件',
      filterKeyword: '过滤关键字',
      regex: '正则表达式',
      applyFilter: '应用过滤器',
      loadFileTree: '加载文件树',
      extract: '提取',
      extractingFiles: '正在提取文件...',
      done: '完成！',
      files: '个文件',
      extracting: '正在提取：',
      terminate: '终止',
      close: '关闭',
      confirmTermination: '确认终止',
      confirmTerminationText: '你确定要终止当前的提取操作吗？已提取的文件将被保留。',
      cancel: '取消',
      confirm: '确认'
    },
    fileNameTable: {
      manageFileList: '管理文件列表',
      openLocalDir: '打开本地目录',
      fetchRemote: '获取远程文件',
      local: '本地',
      refresh: '刷新',
      delete: '删除',
      downloadable: '可下载',
      failedFetchRemote: '获取远程文件列表失败: {error}',
      failedDownloadRemote: '下载远程文件失败: {error}',
      selectedFilesDeleted: '已删除所选文件。',
      conflictDownloadTip1: '存在同名本地文件。',
      conflictDownloadTip2: '如需下载，请先重命名或删除本地文件。'
    },
    pakFiles: {
      selectFileNameTable: '请先选择一个文件名列表。',
      openPaks: '打开 Pak 文件',
      closeAllPaks: '关闭所有 Pak 文件'
    },
    updateDialog: {
      updateAvailable: '有可用更新',
      version: '版本',
      releaseDate: '发布日期',
      notNow: '暂不更新',
      update: '更新',
      willDownloadAndRestart: '将下载并重启应用。'
    },
    pack: {
      addFolder: '添加文件夹',
      addPak: '添加 Pak',
      addPakTooltip: '可用于合并多个 Pak 文件',
      removeAll: '移除全部',
      fileList: '文件列表',
      noFilesAdded: '尚未添加文件',
      noFilesAddedDesc: '点击上方按钮或拖拽文件到此处添加',
      exportSettings: '导出设置',
      exportMode: '导出模式',
      exportModeIndividual: '每个文件项单独导出 pak',
      exportModeSingle: '所有文件导出为单个 pak',
      autoDetectRoot: '自动检测根目录',
      autoDetectRootTooltip: '自动检测第一个 natives/STM/** 路径作为根目录',
      fastMode: '快速模式',
      fastModeTooltipL1: '导入文件后会自动导出到指定目录，无需确认。',
      fastModeTooltipL2: '如未指定目录，则导出到输入文件相同目录。',
      exportDirectory: '导出目录',
      exportDirectoryPlaceholder: '导出目录',
      export: '导出',
      cancelExport: '取消导出',
      exportSuccess: '导出成功',
      exportFailed: '导出失败',
      exporting: '正在导出：',
      fileStructure: '文件结构:',
      filesCount: '个文件',
      fileConflictTitle: '处理文件冲突',
      cancel: '取消',
      confirm: '确定'
    },
    global: {
      failedLoadSettings: '加载设置失败: {error}',
      useDefaultSettings: '将使用默认设置',
      failedDownloadUpdate: '下载更新失败: {error}',
      failedCheckUpdate: '检查更新失败: {error}',
      updateAvailable: '有新版本可用，点击右上角按钮下载。',
      extractionTerminated: '已终止提取。'
    }
  }
}
