# Changelog

## v0.3.1

### Features

- I18n support, now supports English and Chinese.

### Improvements

- Auto fetch remote filelist data when open Filelsit Manage UI at first time.

## v0.3.0

### Features

- Refactored UI based on Vuetify library
- Implemented self-update functionality
- Added filename list service supporting management and downloading/updating list files from cloud
- Added persistent storage, now supporting workspace data persistence
- Added application settings related components (not implemented yet)

### Improvements

- Added drag-and-drop sorting for Pak list
- Added regex pattern support for filename filtering
- Improved filter interaction
- Added extraction progress display and cancel extraction function
- Improved file tree rendering interaction
- Added file type icons in file tree
- Unified frontend font to MiSans to address system font differences across regions. This increased file size by ~5MB

### Bug Fixes

- Fixed incorrect file size display in file tree

### Refactors

- Migrated file list management service to frontend
- Replaced yarn with pnpm

### Update Plan

Due to this update taking much longer than expected, some milestone features were not completed and will be postponed until before v0.4.0.

- Implement application settings related components
- Improve Pak list display to prevent truncation of long filenames
- Support quick drag-and-drop export. Support flat export mode
- Auto-reminder for same files in selected streaming directory

Features that may be completed in v0.4.0 or future versions:

- i18n support
- GUI packer
- Integrated CLI tool

## v0.3.0 (Chinese Version)

### 特性

- 重构UI，以Vuetify组件库为基础
- 实现自更新功能
- 新增文件名列表服务，支持管理和从云端下载、更新列表文件
- 新增持久化存储，现已支持工作区数据持久化
- 增加应用设置相关组件（未实装）

### 改进

- Pak列表支持拖拽排序
- 文件名过滤支持正则表达式模式
- 优化过滤器交互
- 新增解压进度显示，终止解压功能
- 优化文件树渲染交互
- 文件树根据文件类型显示小图标
- 前端字体统一更换为 MiSans 以应对不同地区系统字体差异。此项导致文件体积增加5MB左右

### Bug Fixes

- 修复文件树中错误文件大小显示

### Refactors

- 迁移文件列表管理服务到前端
- 使用pnpm替代yarn

### 更新计划

由于此次更新时间远超预期，部分里程碑计划功能未完成，将推迟到v0.4.0以前完成。

- 实现应用设置相关组件
- 优化Pak列表显示，防止文件名过长导致显示不全
- 支持快速拖拽导出。支持平铺导出模式
- 自动提醒选中streaming目录的相同文件

可能会在v0.4.0中及未来版本完成的功能：

- i18n支持
- GUI打包器
- 集成CLI工具
