# Changelog / 更新日志

All notable changes to this project are documented here. This file follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org/).

本文件记录项目的重要变更，遵循 Keep a Changelog，并使用语义化版本。

## [Unreleased]

## [1.1.0] - 2026-07-18

### English

#### Added

- Batch scraping gains an optional AI name-resolution fallback: ROMs whose English title cannot be resolved locally are matched by your configured OpenAI-compatible LLM against the built-in No-Intro title list. Only titles copied verbatim from the list are accepted, results (including misses) are cached locally, and the per-run switch lives in the batch scrape dialog.
- Sega Saturn libraries now index Alcohol 120% images (`.mds`/`.mdf`), so official Traditional-Chinese discs stored in that format appear in the library.

#### Fixed

- Cartridge-header identification no longer returns No-Intro comma-article names (`Lion King, The`) as scrape queries; they are restored to natural order so translated SFC ROMs with intact headers finally match.
- The built-in Chinese title database now resolves far more translated ROMs: folder names take priority over cryptic release-group filenames for disc games, full-width brackets and trailing disc markers (`CD1`, `Disc A`) are stripped, `MSU-1` markers are removed, and queries formed as “database title + extra subtitle” match via a normalized-prefix tier (e.g. `46亿年物语 向远方的伊甸` → E.V.O. - Search for Eden).
- Fuzzy matching validates digit signatures, eliminating confident mismatches such as `梦幻模拟战1&2` → Langrisser III or `光明力量3剧本2` → Marie no Atelier; exact multi-region hits now deterministically prefer informative variants (`D之食卓` → `D no Shokutaku` instead of the unsearchable `D`).
- Saturn/Dreamcast disc-header titles are only trusted when the Chinese database cannot resolve a name, and polluted headers (rewritten by translation tools) are rejected instead of producing garbled queries.
- Injected-disc PlayStation translations no longer mislead Providers: the carrier disc's serial is ignored once the Chinese database resolves the real title, and lettered disc markers (`Disc A`) are cleaned.
- Batch scrape no longer reports “0 games” for libraries whose `gamelist.xml` contains empty tags: empty `<desc/>`/`<image/>` elements no longer count as scraped metadata or produce phantom media paths.
- Disc-based platforms (PS1, Saturn, Dreamcast, Mega-CD, PCE-CD, 3DO) index a single bootable descriptor per disc — CUE-referenced tracks and same-name `bin`/`img`/`mdf`/`ccd`/`sub` companions are no longer listed as separate games.
- BIOS and system files (`*bios*`, `scph*`, `syscard*`, `neogeo.zip`, `pgm.zip`, `dc_boot`/`dc_flash`) are excluded from scanning on every platform.

### 简体中文

#### 新增

- 批量抓取新增可选的 AI 名称解析兜底：本地无法解析英文名的 ROM，交给你配置的 OpenAI 兼容 LLM 在内置 No-Intro 标题清单内匹配。只接受逐字来自清单的标题，结果（含未命中）本地缓存，开关位于批量抓取对话框、按次生效。
- 世嘉土星库现在会索引 Alcohol 120% 镜像（`.mds`/`.mdf`），官方繁体中文光盘不再缺席。

#### 修复

- 卡带头识别不再把 No-Intro 逗号冠词名（`Lion King, The`）直接用作抓取查询词，而是还原自然语序，卡带头完好的汉化 SFC ROM 终于能够命中。
- 内置中文标题库解析能力大幅增强：光盘游戏优先使用中文文件夹名而非汉化组代号文件名，清理全角括号与尾部盘号（`CD1`、`Disc A`），剥离 `MSU-1` 标记，“库标题 + 额外副标题”式查询经规范化前缀层命中（如 `46亿年物语 向远方的伊甸` → E.V.O. - Search for Eden）。
- 模糊匹配增加数字签名校验，消除“梦幻模拟战1&2 → Langrisser III”“光明力量3剧本2 → 玛莉的炼金工房”这类自信错配；多区域精确命中确定性优先取信息量足的变体（`D之食卓` → `D no Shokutaku` 而非检索不到的 `D`）。
- 土星/DC 光盘头标题仅在中文库解析失败时才采信，被汉化工具改写污染的头部会被拒绝，不再产生乱码查询。
- 注入式 PlayStation 汉化盘不再误导 Provider：中文库解析出真实标题后忽略底盘序列号，并清理字母盘号（`Disc A`）。
- 修复 `gamelist.xml` 空标签导致批量抓取显示“0 个游戏”的问题：空 `<desc/>`/`<image/>` 不再被计为已抓取，也不再产生指向库目录的伪媒体路径。
- 光盘平台（PS1、土星、DC、Mega-CD、PCE-CD、3DO）一张盘只索引一个可引导描述文件——CUE 引用轨与同名 `bin`/`img`/`mdf`/`ccd`/`sub` 不再被列为独立游戏。
- 全平台扫描排除 BIOS 与系统文件（`*bios*`、`scph*`、`syscard*`、`neogeo.zip`、`pgm.zip`、`dc_boot`/`dc_flash`）。

## [1.0.7] - 2026-07-18

### English

#### Added

- The ROM library toolbar now has a game-name / file-name toggle; switching to file name lists ROMs by their original filename across cover, grid, and list views, and the preference is remembered locally.

#### Fixed

- Scrape queries now strip leftover parenthetical markers — language tags (`简`, `繁`, `中`, …), version tags (`v1`, `v1.01`), checksums (`1d444`), and sizes (`16Mb`) — so translated ROMs no longer send noisy queries that caused SFC and other platforms to miss matches.

### 简体中文

#### 新增

- ROM 库工具栏新增“游戏名 / 文件名”切换开关；切换到文件名后，封面、卡片、列表三种视图均按原始文件名展示，偏好本地记忆。

#### 修复

- 抓取查询词现在会剥离残留括注标记——语言标记（`简`、`繁`、`中` 等）、版本号（`v1`、`v1.01`）、校验和（`1d444`）与容量（`16Mb`），避免汉化 ROM 发出带噪声的查询词导致 SFC 等平台匹配失败。

## [1.0.6] - 2026-07-17

### English

#### Fixed

- N64 cartridge-header matching now disambiguates the regular Ocarina of Time release from Master Quest using archive-name markers such as `Master Quest`, `Ura`, `里`, and `裏`.
- Ambiguous Ocarina of Time product codes now fall back to the base release when no Master Quest marker is present instead of sending untranslated filenames to Providers.
- Added a live ScreenScraper regression check confirming that the Master Quest query resolves its dedicated N64 entry.

### 简体中文

#### 修复

- N64 卡带头匹配现在会根据压缩包名中的 `Master Quest`、`Ura`、“里”和“裏”等标记，正确区分普通版《时之笛》与里篇。
- 当有歧义的《时之笛》产品码不含里篇标记时，现在会回退到普通版英文标题，不再把未翻译文件名发给 Provider。
- 增加 ScreenScraper 真实 API 回归检查，确认里篇查询会命中其 N64 专用条目。

## [1.0.5] - 2026-07-17

### English

#### Changed

- PlayStation batch scraping now extracts disc serials from BIN/CUE `SYSTEM.CNF` data and uses ScreenScraper serial lookup before title search.
- ScreenScraper now prefers US/World titles over the first regional alias and retries parenthetical aliases, sports titles without season suffixes, and longer trailing subtitles.
- Translated ROM filenames are cleaned before fallback searches, preserving meaningful aliases while removing localization and repair tags and extracting embedded English titles.

#### Fixed

- PlayStation libraries no longer index a CUE file, its referenced BIN tracks, and an accompanying `_BIOS.BIN` as separate games; standalone BIN images remain supported.
- Improved N64 batch matching for iQue, translated, encrypted, and modified ROM archives by combining reliable cartridge-header identification with cleaned filename fallbacks.
- Preserved explicit manual search terms while limiting PlayStation serial-based exact matching to automatic and batch scraping.

### 简体中文

#### 变更

- PlayStation 批量抓取现在会从 BIN/CUE 的 `SYSTEM.CNF` 数据提取光盘序列号，并优先使用 ScreenScraper 序列号匹配。
- ScreenScraper 现在优先采用美国/世界版标题，并支持括号别名、移除赛季后缀的体育游戏标题以及更长的末尾副标题回退。
- 汉化 ROM 文件名在回退搜索前会进行清理：保留有意义的别名，移除汉化与修复标签，并提取其中的英文标题。

#### 修复

- PlayStation 库不再把 CUE、其引用的 BIN 轨道和附带的 `_BIOS.BIN` 分别索引为游戏，同时继续支持没有 CUE 的独立 BIN 镜像。
- 结合可靠的卡带头识别与清理后的文件名回退，改善神游、汉化、加密及修改版 N64 ROM 的批量匹配。
- 单独抓取继续完整尊重用户输入，PlayStation 序列号精确匹配仅用于自动与批量抓取。

## [1.0.4] - 2026-07-16

### English

#### Added

- Added diagnostic Console entries for manual scraping, including the effective query, platform, file, Provider result count, and leading candidates.

#### Fixed

- ScreenScraper now retries subtitle-only variants for titles with separators and unseparated trailing subtitles, allowing searches such as `Front Mission - Gun Hazard` and `Front Mission Gun Hazard` to resolve correctly.
- Manual scraping now preserves the user's search text instead of silently replacing it with a ROM header or DAT identification result; automatic and batch scraping retain header-based identification.

### 简体中文

#### 新增

- 为单独抓取增加 Console 诊断信息，记录实际查询词、平台、文件、Provider 结果数量和主要候选。

#### 修复

- ScreenScraper 现在会对带分隔符的副标题以及未分隔的末尾副标题执行回退搜索，使 `Front Mission - Gun Hazard` 和 `Front Mission Gun Hazard` 等关键词能够正确匹配。
- 单独抓取不再用 ROM 头或 DAT 识别结果悄悄覆盖用户输入的搜索词；自动与批量抓取仍保留基于 ROM 头的识别。

## [1.0.3] - 2026-07-16

### English

#### Added

- Added multi-select export scopes and persistence for the last selected metadata format and game-name mode.

#### Changed

- Saving from the ROM list now writes both Pegasus metadata and EmulationStation `gamelist.xml`, while copying selected artwork only once.
- ROM metadata/media edits and Library indexed-folder changes now preserve the existing index and use incremental refreshes instead of triggering a full-library scan.
- Scraper asset previews and application now reuse the persistent downloaded candidate cache, including legacy cache files and assets whose credential-bearing URLs have changed.

#### Fixed

- Added a ScreenScraper alias fallback so `Ys 4` on Super Nintendo resolves the `Ys 4 - Mask Of The Sun` entry.

### 简体中文

#### 新增

- 导出范围支持多选，并记住上次选择的 Metadata 格式与游戏名称模式。

#### 变更

- ROM 列表中的“保存”现在会同时写入 Pegasus Metadata 与 EmulationStation `gamelist.xml`，所选美术资源只复制一次。
- 编辑 ROM Metadata/媒体以及修改 Library 索引文件夹时会保留已有索引并执行增量刷新，不再触发整库全量扫描。
- 抓取资源预览与应用会复用持久化的候选资源缓存，同时兼容旧缓存文件以及凭据参数变化后的资源 URL。

#### 修复

- 为 ScreenScraper 增加别名回退，使 Super Nintendo 平台的 `Ys 4` 能匹配 `Ys 4 - Mask Of The Sun` 条目。

## [1.0.2] - 2026-07-16

### English

#### Added

- Added indexed-folder selection while creating a Library and cancellation controls for active ROM scans.
- Added a safe per-Library cleanup tool that previews and removes artwork and media not referenced by valid Pegasus or EmulationStation metadata.

#### Changed

- Expanded ScreenScraper platform IDs and folder aliases across the supported Nintendo, Sega, Sony, NEC, SNK, Bandai, Atari, Microsoft, arcade, and computer systems.
- Completed console Logo resolution for mapping-only platforms and corrected missing or invalid Logo references across built-in systems and arcade categories.
- Packed the bundled DAT/CSV lookup data into compressed embedded archives, substantially reducing the standalone Windows portable executable size without extracting database files at runtime.

#### Fixed

- Normalized malformed, empty, legacy, and percentage-style ratings without repeatedly multiplying already normalized values.

### 简体中文

#### 新增

- 新建 Library 时可直接选择需要索引的文件夹，并为进行中的 ROM 扫描增加取消控制。
- 增加安全的逐 Library 未引用资源清理工具，可先预览，再删除未被有效 Pegasus 或 EmulationStation Metadata 引用的美术与媒体文件。

#### 变更

- 扩展 ScreenScraper 平台 ID 与目录别名，覆盖项目支持的 Nintendo、Sega、Sony、NEC、SNK、Bandai、Atari、Microsoft、街机及电脑平台。
- 补全仅存在于目录映射表的平台 Logo，并修复内置系统与街机分类中缺失或无效的 Logo 引用。
- 将内置 DAT/CSV 匹配数据压缩打包进嵌入式资源，在运行时不释放数据库文件的同时显著缩小 Windows 单文件 Portable 体积。

#### 修复

- 正确规范化异常、空值、旧格式及百分比评分，避免已经规范化的评分被重复放大。

## [1.0.1] - 2026-07-16

### English

#### Changed

- ScreenScraper retries progressively simplified title variants when long database-style names return no match.

#### Fixed

- Recognized ScreenScraper's real-world `[{}]` empty-search sentinel instead of treating it as a malformed response.
- Batch scraping now excludes Providers without usable credentials and stops before processing when none of the selected Providers are available.
- Missing Provider configuration and Provider request failures now produce actionable errors instead of reporting every ROM as unmatched.

### 简体中文

#### 变更

- ScreenScraper 遇到过长数据库式标题无结果时，会逐步简化标题并重试搜索。

#### 修复

- 正确识别 ScreenScraper 实际使用的 `[{}]` 空搜索结果，不再误判为响应结构损坏。
- 批量抓取会排除没有可用凭据的 Provider；所选来源均不可用时会在处理 ROM 前停止。
- Provider 配置缺失或请求失败时显示可操作错误，不再将每个 ROM 都报告为未匹配。

## [1.0.0] - 2026-07-16

### English

#### Added

- Added configurable AI metadata target language, following the interface language by default while allowing a custom value.
- Added file-level library export progress with byte throughput, cancellation, same-size file skipping, and an option to copy only ROM and artwork/media assets.

#### Changed

- Batch scrape completion remains visible until explicitly dismissed, and the Chinese dialog title no longer includes the redundant English label.
- ROM grid and cover layouts now enforce a minimum per-item padding across themes and viewport sizes.

#### Fixed

- Fixed GBA releases sharing a serial by disambiguating them with the internal ROM title, including `ALHJ` Love Hina Advance.
- Empty Provider searches are no longer persisted indefinitely, and total search failures now report actionable Provider errors or the exact query and platform.
- Folder-based ROMs such as PS3 remain complete when using filtered library exports, while unrelated source files are omitted.

### 简体中文

#### 新增

- 增加 AI Metadata 目标语言设置，默认跟随界面语言，同时允许用户自定义填写。
- 增加文件级游戏库导出进度、字节速度、停止、同大小文件跳过，以及“只复制 ROM 和美术/媒体资产”选项。

#### 变更

- 批量抓取完成结果会保持显示直至用户主动关闭，中文弹窗标题不再附带多余英文提示。
- ROM 网格与封面布局在不同主题和窗口尺寸下强制保留最小条目间距。

#### 修复

- 使用 ROM 内部标题消歧共享相同序列号的 GBA 游戏，包括 `ALHJ`《纯情房东俏房客 Advance》。
- Provider 空搜索结果不再被永久缓存；全部搜索失败时会显示可操作的 Provider 错误，或具体查询名称与平台。
- 筛选导出时完整保留 PS3 等文件夹型 ROM，同时忽略无关源文件。

## [0.9.0] - 2026-07-15

### English

#### Added

- Added a platform toolbar Save action that writes preview metadata and artwork directly into the ROM directory.
- Added per-library indexed-folder configuration with All, None, and individual immediate-folder selection.

#### Changed

- Renamed the platform toolbar actions to Select All, Translate, Scrape, and Save.
- Library export now copies the complete indexed ROM library to a user-selected destination while preserving platform folders, existing files, metadata, and artwork.
- Slow folder enumeration and export copying now run on background workers to keep the interface responsive on SD cards and network shares.

#### Fixed

- Fixed cover and card layouts using window width instead of the actual library viewport, which could cause cards to overlap after resizing or changing the card scale.
- Added stable content padding and row-height allowance across ROM library display modes.

### 简体中文

#### 新增

- 平台工具栏增加“保存”，可将预览 Metadata 和美术资源直接写入 ROM 目录。
- 每个 Library 增加索引文件夹配置，支持全选、None 及逐个选择直属文件夹。

#### 变更

- 平台工具栏操作精简为“全选、翻译、抓取、保存”。
- Library 导出改为将完整的已索引 ROM 库复制到用户选择的目标目录，并保留平台目录、现有文件、Metadata 和美术资源。
- 慢速目录枚举及导出复制改到后台工作线程，减少 SD 卡和网络共享上的界面阻塞。

#### 修复

- 修复封面及卡片布局按窗口宽度而非 ROM 库实际内容宽度计算，导致缩放窗口或调整卡片尺寸后卡片互相覆盖的问题。
- 为 ROM 库各展示模式补充稳定的内容间距和行高余量。

## [0.8.7] - 2026-07-16

### English

#### Added

- Added optional merged AI metadata translation with configurable 50K, 100K, and 200K context limits.
- Added configurable OpenAI-compatible reasoning effort and per-language translation markers so completed metadata can be skipped safely.

#### Changed

- Improved translation prompts to prioritize platform and ROM filename evidence, keep entries isolated, and favor accuracy over token efficiency.
- Merged translation batches use no more than one eighth of the selected context for input, reserve the remaining budget for visible output and hidden reasoning, and adaptively split failed or incomplete batches.

#### Fixed

- Translation dialogs can now be closed or stopped while processing; completed results are preserved and no further requests are started.

### 简体中文

#### 新增

- 增加可选的 AI Metadata 合并翻译，并可配置 50K、100K、200K 上下文限制。
- 增加 OpenAI Compatible 思考强度设置及按语言翻译标记，可安全跳过已完成的 Metadata。

#### 变更

- 优化翻译提示词，优先依据平台及 ROM 文件名确认游戏身份，隔离不同条目，并以准确性而非 Token 效率为优先目标。
- 合并翻译最多使用所选上下文的八分之一作为输入，为可见输出及隐藏推理预留预算，并对失败或不完整批次进行自适应拆分。

#### 修复

- 翻译过程中现在可以关闭弹窗或停止任务；已完成结果会保留，并且不会继续发送后续请求。

## [0.8.6] - 2026-07-15

### English

#### Added

- Added a dedicated Chinese ROM name field with legacy metadata migration and original/Chinese name selection for platform and whole-library exports.

#### Changed

- Simplified and Traditional Chinese interfaces now prefer the Chinese ROM name while other interface languages continue to display the original name.

#### Fixed

- Added regular JSON and SSE response handling for OpenAI-compatible metadata translation endpoints, including providers that default to streaming responses.
- Release CI now explicitly publishes the generated GitHub Release and marks it as latest instead of relying only on the release action's draft setting.
- Windows NSIS upgrades and same-version reinstalls now replace the existing application in place without asking users whether to uninstall it first.

### 简体中文

#### 新增

- 增加独立中文 ROM 名称字段、旧 Metadata 迁移，以及平台和整个游戏库导出时的原名/中文名选择。

#### 变更

- 简体中文和繁体中文界面优先显示中文 ROM 名称，其他界面语言继续显示原名。

#### 修复

- AI Metadata 翻译兼容 OpenAI Compatible 接口的普通 JSON 和 SSE 响应，包括默认返回流式内容的 Provider。
- Release CI 在生成 GitHub Release 后显式取消草稿并设为最新版本，不再只依赖 Release Action 的 draft 参数。
- Windows NSIS 升级及同版本重装改为直接覆盖现有程序，不再询问用户是否先卸载。

## [0.8.5] - 2026-07-15

### English

#### Added

- Added atomic settings backups and automatic recovery from the last valid configuration when the primary settings file is damaged.

#### Changed

- Library switching now restores the local index immediately and refreshes slow SD-card or Samba libraries in the background.
- Incremental scans now use shallow directory fingerprints instead of recursively reading and inspecting every ROM file.
- Windows library paths are consistently stored and displayed with native backslash separators.

#### Fixed

- Prevented background scans from blocking library and layout switching or applying results to the wrong active library.
- Isolated test configuration from the real user AppData directory and preserved legacy ROM indexes while migrating path and fingerprint formats.

### 简体中文

#### 新增

- 增加设置文件原子备份，并在主设置文件损坏时自动恢复最后一份有效配置。

#### 变更

- 切换游戏库时立即恢复本地索引，并在后台刷新慢速 SD 卡或 Samba 游戏库。
- 增量扫描改用浅层目录指纹，不再递归读取并检查每个 ROM 文件。
- Windows 游戏库路径统一使用原生反斜杠格式存储和显示。

#### 修复

- 修复后台扫描阻塞游戏库及布局切换，或将结果写入错误激活游戏库的问题。
- 隔离测试配置与真实用户 AppData，并在迁移路径和指纹格式时保留旧 ROM 索引。

## [0.8.4] - 2026-07-15

### English

#### Added

- Added a local library-artwork cache so covers and other images remain responsive when ROM libraries are stored on slow SD cards or network shares.

#### Changed

- Moved path validation, directory browsing, metadata detection, ROM scanning, storage statistics, and library-artwork reads off the asynchronous runtime's main worker threads.
- Increased confirmed console-platform matching weight from 20% to 30% and the penalty for conflicting platforms from 25% to 40%.

#### Fixed

- Prevented media-only providers such as SteamGridDB from overwriting game titles and metadata when applying selected artwork.

### 简体中文

#### 新增

- 增加本地游戏库美术资源缓存，使 ROM 库位于慢速 SD 卡或网络共享时，封面及其他图片仍能流畅显示。

#### 变更

- 将路径校验、目录浏览、Metadata 检测、ROM 扫描、存储统计和游戏库美术资源读取移出异步运行时的主工作线程。
- 将已确认主机平台的匹配权重从 20% 提高到 30%，跨平台冲突惩罚从 25% 提高到 40%。

#### 修复

- 修复 SteamGridDB 等纯媒体 Provider 在应用所选美术资源时错误覆盖游戏标题和 Metadata 的问题。

## [0.8.3] - 2026-07-15

### English

#### Added

- Added storage statistics for the configuration directory, scraper cache, pending export data, and incomplete downloads.
- Added settings actions to refresh storage usage, open the cache folder, remove interrupted `.part` downloads, and clear provider search and candidate-asset caches while preserving pending metadata and artwork.

#### Fixed

- Declared the `Microsoft.VCLibs.140.00.UWPDesktop` framework dependency in the Store MSIX so Microsoft Store installs the Visual C++ runtime required for `MSVCP140.dll`.

### 简体中文

#### 新增

- 增加配置目录、抓取缓存、待导出数据和中断下载的存储占用统计。
- 设置页增加刷新统计、打开缓存目录、清理中断 `.part` 下载和清空 Provider 搜索及候选资源缓存功能，并保留待导出的 Metadata 与美术资源。

#### 修复

- Store MSIX 增加 `Microsoft.VCLibs.140.00.UWPDesktop` framework dependency 声明，使 Microsoft Store 自动安装 `MSVCP140.dll` 所需的 Visual C++ 运行库。

## [0.8.2] - 2026-07-15

### English

#### Fixed

- Synchronized the MSIX manifest language declarations with all eight interface locales, allowing Microsoft Store to detect English, Simplified Chinese, Traditional Chinese, French, German, Italian, Spanish, and Russian support.

### 简体中文

#### 修复

- 将 MSIX manifest 的语言声明与全部八种界面语言同步，使 Microsoft Store 能正确识别英语、简体中文、繁体中文、法语、德语、意大利语、西班牙语和俄语支持。

## [0.8.1] - 2026-07-14

### English

#### Added

- Added a Microsoft Store MSIX build to the Windows release workflow using the Partner Center identity assigned to MRRM.
- Added a bilingual English and Simplified Chinese privacy policy covering local data, third-party scraper credentials, AI endpoints, retention, and deletion.

#### Fixed

- Corrected the complete MSIX package identity, publisher, publisher display name, and reserved Store display name to match the MRRM product in Partner Center.

#### Changed

- Upgraded every JavaScript-based GitHub Action in the release workflow to a Node.js 24 runtime, removing Node.js 20 deprecation warnings.

### 简体中文

#### 新增

- Windows 发布流程新增 Microsoft Store MSIX 构建，并使用 Partner Center 为 MRRM 分配的正式应用身份。
- 新增英文与简体中文双语隐私政策，说明本地数据、第三方抓取凭据、AI 端点、数据保留和删除方式。

#### 修复

- 修正完整的 MSIX 程序包标识、发布者、发布者显示名称和 Store 保留显示名称，使其与 Partner Center 中的 MRRM 产品一致。

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
