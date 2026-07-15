# Changelog / 更新日志

All notable changes to this project are documented here. This file follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org/).

本文件记录项目的重要变更，遵循 Keep a Changelog，并使用语义化版本。

## [Unreleased]

## [0.8.1] - 2026-07-14

### English

#### Added

- Added a Microsoft Store MSIX build to the Windows release workflow, with configurable Partner Center identity and publisher values.
- Added a bilingual English and Simplified Chinese privacy policy covering local data, third-party scraper credentials, AI endpoints, retention, and deletion.

#### Fixed

- Corrected the default MSIX publisher display name to `dotSlashZ` so packages match the Partner Center account identity.

#### Changed

- Upgraded every JavaScript-based GitHub Action in the release workflow to a Node.js 24 runtime, removing Node.js 20 deprecation warnings.

### 简体中文

#### 新增

- Windows 发布流程新增 Microsoft Store MSIX 构建，并支持配置 Partner Center 分配的应用身份和发布者信息。
- 新增英文与简体中文双语隐私政策，说明本地数据、第三方抓取凭据、AI 端点、数据保留和删除方式。

#### 修复

- 将 MSIX 默认发布者显示名称修正为 `dotSlashZ`，确保程序包与 Partner Center 账户身份一致。

#### 变更

- 将发布流程中所有 JavaScript GitHub Action 升级到 Node.js 24 runtime，消除 Node.js 20 弃用警告。

## [0.8.0] - 2026-07-14

### English

#### Added

- Added complete French, German, Italian, Spanish, Russian, and Traditional Chinese interface translations, expanding the application to eight languages.
- Added automated locale parity tests covering all 400 translation keys and interpolation placeholders in every language.

#### Fixed

- Restored the saved interface language during startup instead of reverting to Simplified Chinese until the user changed it again.

#### Changed

- Release CI now rejects tags that do not match the application version before starting platform builds.

### 简体中文

#### 新增

- 增加完整的法语、德语、意大利语、西班牙语、俄语和繁体中文界面翻译，应用现支持八种语言。
- 增加多语言一致性自动测试，覆盖每种语言的全部 400 个翻译键和插值占位符。

#### 修复

- 修复启动时未恢复已保存界面语言、必须再次手动切换才能生效的问题。

#### 变更

- Release CI 在开始各平台构建前校验 tag 与应用版本一致，阻止错误版本发布。

## [0.7.3] - 2026-07-14

### English

#### Added

- Added a Windows NSIS installer alongside the self-contained portable EXE; Linux continues to ship as AppImage.

#### Changed

- An existing `config` folder beside the executable now has first priority; otherwise configuration falls back to the current user's application-data directory.
- Bundled ROM matching data and system logos are embedded in the application and extracted into the selected configuration directory when required.
- Release CI now builds with the latest Node.js LTS and publishes non-draft GitHub Releases containing only the current version's bilingual notes.
- Converted the complete changelog to Keep a Changelog with English and Simplified Chinese sections for every version.

### 简体中文

#### 新增

- 在单文件 Windows portable EXE 之外增加 NSIS 安装程序；Linux 继续使用 AppImage。

#### 变更

- 程序旁已存在的 `config` 文件夹现在拥有最高读取优先级；否则回退到当前用户的 AppData 应用数据目录。
- ROM 匹配数据和系统 Logo 改为嵌入应用，并在需要时释放到所选配置目录。
- Release CI 改用最新 Node.js LTS，并直接正式发布 GitHub Release，Release notes 仅包含当前版本的中英文内容。
- 将完整更新日志转换为 Keep a Changelog，每个版本均包含英文和简体中文部分。

## [0.7.2] - 2026-07-14

### English

#### Removed

- Removed non-functional play buttons from ROM cover and card views; selecting a ROM now consistently opens its detail panel.

### 简体中文

#### 移除

- 移除封面视图和卡片视图中无实际运行逻辑的播放按钮，点击 ROM 条目统一打开详情面板。

## [0.7.1] - 2026-07-14

### English

#### Changed

- Intercepted context-menu events at the application root to suppress the WebView browser menu and reserve a unified entry point for future app menus.

### 简体中文

#### 变更

- 在应用根层接管右键事件，不再显示 WebView 默认浏览器菜单，并为后续业务右键菜单保留统一入口。

## [0.7.0] - 2026-07-14

### English

#### Added

- Added configurable OpenAI-compatible metadata translation with endpoint, API key, model, and target-language settings.
- Added single-ROM and batch translation with cost warnings, progress feedback, and metadata previews.

#### Changed

- Translation prompts now include platform, ROM filename, and existing metadata for disambiguation while preserving non-language fields and preventing instruction injection or fabricated facts.

#### Fixed

- Added explicit timeout, authentication, rate-limit, malformed-response, and JSON parsing error handling.

### 简体中文

#### 新增

- 增加可配置端点、API Key、模型和目标语言的 OpenAI-compatible Metadata 翻译。
- ROM 详情支持单项翻译，平台页支持批量翻译，并提供费用提示、进度反馈和 Metadata 预览。

#### 变更

- 翻译提示词结合平台、ROM 文件名和现有 Metadata 消歧，保留非语言字段，并限制指令注入和资料杜撰。

#### 修复

- 增加超时、鉴权、限流、无效响应和 JSON 解析错误处理。

## [0.6.3] - 2026-07-14

### English

#### Fixed

- “Add Library” now opens General Settings, focuses the Library section, and opens the directory picker.
- Settings tabs are driven by URL parameters so navigation also works when the page is already mounted.

### 简体中文

#### 修复

- “添加 Library”现在会打开常规设置、定位 Library 管理区域并打开目录选择器。
- 设置选项卡改由 URL 参数驱动，修复页面已挂载时无法切换目标选项卡的问题。

## [0.6.2] - 2026-07-14

### English

#### Fixed

- Hardened ScreenScraper authentication, search, and game-detail parsing against polymorphic and malformed fields without failing the whole provider.
- Added connection/request timeouts and safe handling for authentication, client-version, rate-limit, and daily-quota errors.
- EmulationStation metadata now prefers real box art, uses marquee images as logos, and avoids severe cropping for landscape artwork.

### 简体中文

#### 修复

- 强化 ScreenScraper 鉴权、搜索和详情解析，兼容多形态及异常字段，单条坏数据不再拖垮整个 Provider。
- 增加连接与请求超时，安全处理鉴权、客户端版本、限流和每日额度错误。
- EmulationStation Metadata 优先使用真实 Boxart，以 Marquee 作为 Logo，并避免横向美术资源被严重裁切。

## [0.6.1] - 2026-07-14

### English

#### Added

- Added full re-scrape mode, current/whole Library export, provider-priority display, Retro Zpix font, offline ScreenScraper notes, and cross-platform build CI.

#### Fixed

- Fixed ScreenScraper credential persistence and result parsing, normalized Windows export paths, and allowed remaining providers to continue after one provider fails.

### 简体中文

#### 新增

- 增加全量重新抓取、当前或整个 Library 导出、Provider 优先级显示、Retro Zpix 字体、ScreenScraper 离线文档和跨平台构建 CI。

#### 修复

- 修复 ScreenScraper 凭据保存与结果解析，统一 Windows 导出路径，并在单个 Provider 失败后继续尝试其他来源。

## [0.6.0] - 2026-07-13

### English

#### Added

- Added multiple independently named and persisted Libraries, active-Library switching, per-Library full scans, and a Library selector in the ROM sidebar.

#### Fixed

- Reset platform, ROM, and selection state when switching Libraries to prevent cross-library data mixing.

### 简体中文

#### 新增

- 增加可独立命名和持久化的多 Library、激活 Library 切换、单库全量扫描及 ROM 侧栏 Library 选择器。

#### 修复

- 切换 Library 时清空平台、ROM 和选择状态，避免不同游戏库的数据混合。

## [0.5.1] - 2026-07-13

### English

#### Changed

- Embedded the application-level ScreenScraper developer credentials so users only configure their member username and password.
- Removed custom developer credential fields and prevented sensitive query parameters from being logged.

### 简体中文

#### 变更

- 内置应用级 ScreenScraper Developer 凭据，用户只需配置会员用户名和密码。
- 移除自定义 Developer 凭据字段，并防止敏感查询参数进入日志。

## [0.5.0] - 2026-07-13

### English

#### Added

- Added persistent ROM indexes, incremental/full scans with progress, a level-filtered console, cross-Library scrape caches, EmulationStation/Pegasus export, and provider diagnostics.

#### Changed

- Optimized a 14,366-ROM full scan from about 15.6 seconds to about 3 seconds and an unchanged incremental scan to about 1.1 seconds.

### 简体中文

#### 新增

- 增加持久 ROM 索引、带进度的增量/全量扫描、分级 Console、跨 Library 抓取缓存、EmulationStation/Pegasus 导出和 Provider 诊断。

#### 变更

- 14,366 个 ROM 的全量扫描由约 15.6 秒优化到约 3 秒，无变化增量扫描约 1.1 秒。

## [0.4.6] - 2026-07-13

### English

#### Added

- Added multi-provider batch scraping, default assets per media type, local search-result caching, and select-all for the current platform.

#### Fixed

- Ranked duplicate candidates by asset completeness, skipped complete ROMs, fixed cancellation persistence, improved sequel matching, and removed unimplemented run-game actions.

### 简体中文

#### 新增

- 增加多 Provider 批量抓取、各资源类型默认资产、本地搜索结果缓存和平台全选。

#### 修复

- 按资产完整度排列重复候选，跳过完整 ROM，修复停止后的结果保存和续作匹配，并移除未实现的运行游戏入口。

## [0.4.5] - 2026-07-13

### English

#### Added

- Added platform-wide and whole-library scraping, with the English README as the default and a linked Simplified Chinese edition.

#### Fixed

- Lowered confidence for candidates without box art, reused downloaded assets, and restored box art display in the ROM library.

### 简体中文

#### 新增

- 增加整平台和整个 ROM 库抓取；默认展示英文 README，并链接简体中文版。

#### 修复

- 降低无 Boxart 候选的置信度，复用已下载资产，并修复 ROM 库封面显示。

## [0.4.4] - 2026-07-13

### English

#### Fixed

- Improved platform-confidence weighting, filtered empty assets, used GBA headers/Game Codes for English queries, and corrected applied metadata/asset paths.

### 简体中文

#### 修复

- 改进平台置信度权重，过滤空资产，使用 GBA Header/Game Code 生成英文查询，并修正应用后的 Metadata 与资产路径。

## [0.3.2] - 2026-07-12

### English

#### Fixed

- Fixed Retro-theme scrolling jank caused by opaque custom scrollbar thumbs forcing Chromium/WebView2 main-thread repainting.
- Stopped a hidden drag-overlay icon animation from running continuously.

### 简体中文

#### 修复

- 修复不透明自绘滚动条导致 Chromium/WebView2 主线程重绘引起的 Retro 主题滚动掉帧。
- 修复隐藏拖拽遮罩的图标动画持续运行问题。

## [0.3.1] - 2026-07-12

### English

#### Fixed

- Replaced the full-screen animated scanline overlay with an isolated static texture to eliminate Retro/Cyber theme frame drops.

### 简体中文

#### 修复

- 将全屏动态扫描线改为隔离的静态纹理，消除 Retro/Cyber 主题掉帧。

## [0.3.0] - 2026-07-12

### English

#### Added

- Introduced importable theme packs, four built-in themes, a redesigned system-shelf/library UI, a reusable UI component set, and local pixel fonts.
- Added GBA header validation and dual-source Chinese ROM matching with editable English names and Pegasus metadata support.

#### Fixed

- Fixed production white screens, broken spacing utilities, missing versions, scraper setting persistence, and Chinese-ROM scrape/export paths.

### 简体中文

#### 新增

- 引入可导入主题包、四套内置主题、系统货架与 ROM 库新界面、统一 UI 组件和本地像素字体。
- 增加 GBA Header 验证、双数据源中文 ROM 匹配、英文名编辑和 Pegasus Metadata 支持。

#### 修复

- 修复生产白屏、间距工具失效、版本号缺失、Scraper 设置不保存以及中文 ROM 抓取/导出路径问题。

## [0.2.0] - 2026-01-18

### English

#### Added

- Added confidence visualization, inline English-name editing, region-tag cleanup, confidence sorting, and resizable columns to the Chinese ROM tool.

#### Fixed

- Fixed saving edited English names after sorting and a type-inference error in name cleanup.

### 简体中文

#### 新增

- 中文 ROM 工具增加置信度可视化、英文名行内编辑、区域标签清理、置信度排序和可调整列宽。

#### 修复

- 修复排序后英文名编辑无法保存及名称清理类型推断错误。

## [0.1.0] - 2026-01-17

### English

#### Changed

- Replaced SQLite/Diesel with metadata-file directory scanning and temporarily disabled legacy import/export during migration.

#### Fixed

- Aligned frontend ROM fields with backend responses and normalized i18n/directory prompts.

### 简体中文

#### 变更

- 移除 SQLite/Diesel，改用 Metadata 文件目录扫描，并在迁移期间暂时停用旧导入/导出流程。

#### 修复

- 对齐前端 ROM 字段与后端返回结构，并统一 i18n 和目录提示。

[Unreleased]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.8.0...HEAD
[0.8.0]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.7.3...v0.8.0
[0.7.3]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.7.2...v0.7.3
[0.7.2]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.6.3...v0.7.0
[0.6.3]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.4.6...v0.5.0
[0.4.6]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.4.5...v0.4.6
[0.4.5]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.3.2...v0.4.4
[0.3.2]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/freefrank/ModernRetroRomManager/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/freefrank/ModernRetroRomManager/releases/tag/v0.1.0
