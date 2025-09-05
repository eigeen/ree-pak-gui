export default {
  messages: {
    menu: {
      slogan: 'REE Pak Tool',
      unpack: 'Unpack',
      repack: 'Repack'
    },
    unpack: {
      fileList: 'Path List',
      pakFiles: 'Pak Files',
      filterKeyword: 'Filter keyword',
      regex: 'Regex',
      applyFilter: 'Apply Filter',
      loadFileTree: 'Load File Tree',
      extract: 'Extract',
      extractingFiles: 'Extracting Files...',
      done: 'Done!',
      files: 'files',
      extracting: 'Extracting:',
      terminate: 'Terminate',
      close: 'Close',
      confirmTermination: 'Confirm Termination',
      confirmTerminationText:
        'Did you want to terminate the current extraction operation? The extracted files will be retained.',
      cancel: 'Cancel',
      confirm: 'Confirm'
    },
    fileNameTable: {
      manageFileList: 'Manage Path List',
      openLocalDir: 'Open Local Dir',
      fetchRemote: 'Fetch Remote',
      local: 'Local',
      refresh: 'Refresh',
      delete: 'Delete',
      downloadable: 'Downloadable',
      failedFetchRemote: 'Failed to fetch remote path list: {error}',
      failedDownloadRemote: 'Failed to download remote file: {error}',
      selectedFilesDeleted: 'Selected files deleted.',
      conflictDownloadTip1: 'You have a local file with same identifier.',
      conflictDownloadTip2: 'If you want to download, please rename or delete the local file first.'
    },
    pakFiles: {
      selectFileNameTable: 'Select a Path List first.',
      openPaks: 'Open Paks',
      closeAllPaks: 'Close all paks'
    },
    updateDialog: {
      updateAvailable: 'Update Available',
      version: 'Version',
      releaseDate: 'Release Date',
      notNow: 'Not Now',
      update: 'Update',
      willDownloadAndRestart: 'Will download and restart the application.'
    },
    pack: {
      addFolder: 'Add Folder',
      addPak: 'Add Pak',
      addPakTooltip: 'Can be used to merge multiple Pak files',
      removeAll: 'Remove All',
      fileList: 'File List',
      noFilesAdded: 'No files added yet',
      noFilesAddedDesc: 'Click the buttons above or drag files here to add',
      exportSettings: 'Export Settings',
      exportMode: 'Export Mode',
      exportModeIndividual: 'Export each file item as separate pak',
      exportModeSingle: 'Export all files as single pak',
      autoDetectRoot: 'Auto detect root directory',
      autoDetectRootTooltip: 'Auto detect the first natives/STM/** path as root directory',
      fastMode: 'Fast Mode',
      fastModeTooltipL1:
        'Automatically export to specified directory after importing files, no confirmation needed.',
      fastModeTooltipL2:
        'If no directory is specified, export to the same directory as input files.',
      exportDirectory: 'Export Directory',
      exportDirectoryPlaceholder: 'Export directory',
      export: 'Export',
      cancelExport: 'Cancel Export',
      exportSuccess: 'Export Success',
      exportFailed: 'Export Failed',
      exporting: 'Exporting:',
      fileStructure: 'File Structure:',
      filesCount: 'files',
      fileConflictTitle: 'Handle File Conflicts',
      cancel: 'Cancel',
      confirm: 'Confirm',
      // Packer error messages
      noFiles: 'No files',
      failedGetParentPath: 'Failed to get parent path from input file',
      exportDirRequired: 'Export directory is required for merge export',
      failedCancelExport: 'Failed to cancel export operation: {error}'
    },
    preview: {
      title: 'Preview',
      previewLoadFailed: 'Preview load failed',
      unsupportedFileType: 'Unsupported file type for preview',
      selectFileToPreview: 'Select a file to preview content'
    },
    global: {
      failedLoadSettings: 'Failed to load settings: {error}',
      useDefaultSettings: 'Will use default settings',
      failedDownloadUpdate: 'Failed to download update: {error}',
      failedCheckUpdate: 'Failed to check for updates: {error}',
      updateAvailable: 'Update available. Click the button on the top right to download.',
      extractionTerminated: 'Extraction terminated.'
    }
  }
}
