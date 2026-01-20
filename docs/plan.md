# ModernRetroManager - 现代化 Retro ROM 管理软件

## 📝 最近更新 (2026-01-20)

### 本次会话完成的功能

| 修复/优化项 | 文件 | 说明 |
|------------|------|------|
| **扫描结果去重** | `naming_check.rs` | 基于 `file` 字段 HashMap 去重，保留更完整的条目 |
| **Pegasus 模块统一** | `pegasus.rs`, `persistence.rs`, `naming_check.rs` | 将分散的 Pegasus metadata 生成逻辑统一到 `pegasus.rs` |
| **增强 Pegasus 导出** | `pegasus.rs` | 新增 `PegasusExportOptions`、`write_pegasus_file()` 支持合并模式 |
| **匹配英文名优化** | `naming_check.rs` | 不再重复扫描文件夹，直接读取临时 metadata |
| **移除无用弹窗** | `CnRomTools.tsx` | 移除匹配英文名的确认弹窗和完成提示 |
| **统一游戏名提取** | `naming_check.rs` | 合并 `parse_cn_name_from_filename` 和 `clean_folder_name` 为 `extract_game_name` |
| **查询名优先级** | `naming_check.rs` | 匹配时优先使用已生成的 `name` 字段，而非重新提取 |

### 架构改进

#### Pegasus Metadata 模块统一化
```
scraper/pegasus.rs (唯一入口)
├── PegasusExportOptions     # 导出配置（collection header、assets 等）
├── export_to_pegasus()      # 生成 metadata 字符串
├── write_pegasus_file()     # 文件写入 + 合并逻辑
├── write_multiline_field()  # 多行值处理（符合官方规范）
└── write_asset_field()      # 资源路径字段

调用方:
├── persistence.rs::save_metadata_pegasus()  # 使用新模块
└── naming_check.rs::export_pegasus_format() # 使用新模块
```

#### 游戏名提取逻辑统一化
```rust
extract_game_name(name: &str, is_filename: bool) -> Option<String>
// - 子文件夹 ROM → 从文件夹名提取 (is_filename=false)
// - 平台文件夹 ROM → 从文件名提取 (is_filename=true)
// - 统一清理：括号、汉化组、版本号、全角字符
```

---

## 🎯 项目愿景

打造一款**现代化、跨平台、开源**的 Retro ROM 管理软件，替代老旧的 ARRM 和 Skraper，摆脱对 screenscraper.fr 的过度依赖。

### 核心目标
- 🌐 **双模式部署**：可 Self-host 也可打包成 Native App (Win/Mac/Linux)
- 🎨 **现代化 UI**：使用最新前端技术，美观且高效 (Cyberpunk 风格)
- 🔌 **多源 Scraping**：整合多个 API 和爬虫源
- 📦 **兼容性强**：支持导入现有 metadata.txt、playlist.xml 等格式

---

## 🏗️ 技术架构

### 技术栈

```
┌─────────────────────────────────────────────────────────────┐
│                      Frontend Layer                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │    React 19 + TypeScript + TailwindCSS v4           │   │
│  │   (Vite + Framer Motion + Lucide React)             │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                      Backend Layer                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │     桌面版: Rust (Tauri Framework v2)               │   │
│  │     - 轻量级 (无内嵌浏览器开销)                     │   │
│  │     - Metadata 驱动 (直接读写 XML/TXT)              │   │
│  │     - 跨平台编译 (Win/Mac/Linux)                    │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │     Web版: Node.js (Express + TypeScript)           │   │
│  │     - Docker 容器部署                               │   │
│  │     - Volume 映射 ROM 目录                          │   │
│  │     - 媒体文件代理 API                              │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                      Storage Layer                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │             File System (Metadata Files)            │   │
│  │           pegasus.txt / gamelist.xml                │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 详细架构图

#### 后端架构 (Rust)

```
src-tauri/src/
├── main.rs (App Entry)
├── lib.rs (Tauri Setup)
├── config.rs (Path/Config Mgmt)
├── system_mapping.rs (Platform Mapping)
├── rom_service.rs (Core Service)
│
├── commands/ (Tauri APIs)
│   ├── mod.rs
│   ├── rom.rs          # ROM listing/scanning
│   ├── scraper.rs      # Scraper interactions
│   ├── naming_check.rs # CN naming tools
│   ├── ps3.rs          # PS3 tools
│   └── ...
│
├── scraper/ (Scraper Engine)
│   ├── mod.rs
│   ├── manager.rs      # Provider orchestration
│   ├── types.rs        # Shared structs
│   ├── pegasus.rs      # Metadata parser/writer
│   ├── persistence.rs  # File saving
│   ├── matcher.rs      # Fuzzy matching
│   └── providers/      # Implementations
│       ├── steamgriddb.rs
│       └── screenscraper.rs
│
└── ps3/ (PS3 Module)
    ├── mod.rs
    ├── sfo.rs          # PARAM.SFO parser
    └── boxart.rs       # Boxart generator
```

---

## 📚 代码库详解 (Function Reference)

### 1. 核心服务 (`src-tauri/src/rom_service.rs`)

核心业务逻辑层，负责协调文件扫描和元数据应用。

- `struct RomInfo`: 核心数据结构，表示一个 ROM 及其所有元数据（描述、开发者、媒体路径等）。
- `struct SystemRoms`: 按系统分组的 ROM 列表。
- `get_all_roms() -> Result<Vec<SystemRoms>>`: 获取所有配置目录下的 ROM，自动检测 EmulationStation 或 Pegasus 格式。
- `get_roms_for_directory(config) -> Vec<SystemRoms>`: 扫描单个目录。支持“根目录模式”（包含多个系统子文件夹）和“单系统模式”。
- `scan_rom_files(dir, system) -> Result<Vec<RomInfo>>`: 底层扫描函数，根据扩展名过滤文件。
- `apply_temp_metadata(roms, library_path, system)`: 将临时目录 (`config/temp/...`) 中的元数据合并到文件扫描结果中。优先显示临时数据。
- `try_load_from_temp_metadata(...)`: 尝试直接从临时 metadata 加载 ROM 列表，避免重复扫描文件系统（性能优化）。
- `create_or_update_metadata(...)`: 在 temp 目录初始化 metadata 文件。
- `update_rom_media_in_metadata(...)`: 更新 metadata 文件中特定 ROM 的媒体路径。

### 2. 配置管理 (`src-tauri/src/config.rs`)

负责所有路径解析和目录管理。

- `get_config_dir() -> PathBuf`: 获取配置根目录 (优先 `CONFIG_DIR` 环境变量，否则 `exe/config`).
- `get_temp_dir() -> PathBuf`: 获取 `config/temp`。
- `get_media_dir() -> PathBuf`: 获取 `config/media`。
- `normalize_path_to_dirname(path) -> String`: 将绝对路径（如 `D:\Games`）转换为合法目录名（`d_games`），用于多库隔离。
- `get_temp_dir_for_library(lib_path, system) -> PathBuf`: 获取特定库+系统的临时目录，如 `config/temp/z/gba/`.

### 3. Pegasus 解析器 (`src-tauri/src/scraper/pegasus.rs`)

Pegasus 前端格式 (`metadata.txt`) 的读写引擎。

- `struct PegasusGame / PegasusCollection`: 对应文件结构的 Rust 结构体。
- `struct PegasusExportOptions`: 导出配置（是否包含 assets，header 等）。
- `parse_pegasus_file(path) -> Result<PegasusMetadata>`: 读取并解析文件，支持自动检测编码（UTF-8/GBK）。
- `export_to_pegasus(games, options) -> String`: 将游戏列表序列化为 Pegasus 格式字符串。
- `write_pegasus_file(path, games, options, merge) -> Result<()>`:
  - **核心功能**：写入文件。
  - **Merge 模式**：如果 `merge=true`，先读取现有文件，合并新旧数据（新数据覆盖旧数据，保留已有但未更新的字段），然后写回。
- `write_multiline_field(...)`: 处理多行文本格式（Pegasus 规范）。

### 4. 数据持久化 (`src-tauri/src/scraper/persistence.rs`)

负责将内存中的 `GameMetadata` 保存到磁盘。

- `save_metadata_pegasus(rom, metadata, is_temp)`: 将通用元数据保存为 Pegasus 格式。调用 `pegasus::write_pegasus_file`。
- `save_metadata_emulationstation(...)`: 保存为 `gamelist.xml`。使用 `quick-xml` 进行反序列化->修改->序列化，确保格式稳健。
- `download_media(rom, assets, is_temp) -> Result<Vec<(MediaType, PathBuf)>>`: 下载网络图片到本地 `media` 目录。

### 5. Scraper 引擎 (`src-tauri/src/scraper/manager.rs` & `types.rs`)

- `struct ScraperManager`: 管理多个 Provider (SteamGridDB, ScreenScraper)。
- `scrape(query) -> ScrapeResult`: 智能抓取流程：
  1. Hash 查找 (精确)
  2. 名字搜索 (模糊)
  3. 聚合多个 Provider 的 Metadata (按优先级覆盖)
  4. 并行下载 Media
- `search(query) -> Vec<SearchResult>`: 并发调用所有 Provider 的搜索接口。
- `aggregate_metadata(...)`: 合并不同来源的元数据（例如：IGDB 的描述 + SteamGridDB 的封面）。

### 6. 中文 ROM 工具 (`src-tauri/src/commands/naming_check.rs`)

专为中文 ROM 整理设计的工具集。

- `scan_directory_for_naming_check(path)`: 扫描目录，生成 `NamingCheckResult`。
  - 自动识别子文件夹中的 ROM。
  - 读取临时 Metadata 状态。
  - 返回：文件名、当前显示名、已匹配的英文名、置信度。
- `auto_fix_naming(path, system)`: **一键修复**。
  - 从 `rom-name-cn` CSV 数据库中查找匹配项。
  - 使用 `fast_match` 算法（内存中匹配）。
  - 将匹配结果写入临时 Metadata。
- `extract_game_name(name, is_filename) -> Option<String>`: **核心清洗逻辑**。
  - 去除括号 `(USA)`, `[汉化]`。
  - 去除版本号 `v1.0`。
  - 处理全角字符。
  - 用于从文件名或文件夹名提取纯净的游戏标题。
- `scan_directory_with_folders(path)`: 增强版扫描，支持识别 `ROM/子文件夹/game.iso` 结构。
- `save_temp_cn_metadata / load_temp_cn_metadata`: 读写 `temp/cn_metadata/{dir}/metadata.json`，用于持久化用户的整理进度。

### 7. 前端 Store (`src/stores/*.ts`)

- `romStore.ts`:
  - `fetchRoms()`: 调用后端 `get_roms`。
  - `addScanDirectory()`: 添加新目录并刷新。
  - `updateTempMetadata()`: 更新前端的临时修改。
- `scraperStore.ts`:
  - 管理 Provider 的开启状态、优先级和凭证。
- `cnRomToolsStore.ts`:
  - 管理中文工具页面的状态（扫描进度、匹配进度、结果列表）。

---

## 📋 功能模块状态

### 1. ROM 库管理
- [x] 目录递归扫描
- [x] 多 metadata 格式支持 (Pegasus / ES)
- [x] 临时元数据覆盖机制 (Non-destructive editing)
- [x] PS3 专用支持 (SFO 解析, 混合目录)

### 2. Scraper
- [x] 多源聚合 (Manager 模式)
- [x] SteamGridDB 实现
- [x] ScreenScraper 实现
- [x] 并行搜索与下载
- [x] 优先级配置

### 3. 中文 ROM 整理
- [x] CSV 数据库集成 (rom-name-cn)
- [x] 智能命名提取 (去除标签/版本号)
- [x] 批量自动匹配
- [x] 结果导出 (Pegasus / Gamelist)
- [x] 手动修正与锁定 (Confidence=100)

### 4. UI/UX
- [x] 虚拟列表 (React Window)
- [x] 拖拽调整列宽
- [x] 进度条反馈
- [x] 国际化 (i18n)

---

## 🚀 开发路线图

### Phase 1: 基础框架 (MVP)

#### 1.1 项目初始化
- [x] 项目规划文档
- [x] Tauri v2 + React 19 + TypeScript 项目搭建
- [x] TailwindCSS v4 + 多主题配置 (8种主题: Light/Dark/Cyberpunk/Ocean/Forest/Sunset/Rose/Nord)
- [x] 基础路由配置 (React Router 7)

#### 1.2 数据服务层 (Refactored)
- [x] 移除 SQLite/Diesel 依赖
- [x] 实现 Metadata 文件解析器 (Pegasus / EmulationStation)
- [x] 预置 18 种游戏系统数据 (Config file)
- [x] 系统名称映射配置 (60+ 平台，统一 CSV/Logo 映射)
- [x] 基础 Tauri Commands (get_roms, get_stats)
- [x] 目录扫描替代旧导入流程
- [x] 前端 ROM 列表字段对齐

#### 1.3 基础 UI
- [x] 现代化 Cyberpunk 风格布局
- [x] Glassmorphism 侧边栏导航
- [x] 国际化支持 (i18n)
- [x] ROM 列表视图（表格）
- [x] ROM 列表视图（网格）
- [x] ROM 详情面板
- [x] 全局搜索 (Spotlight 风格)
- [x] ROM 网格视图（封面）

#### 1.4 ROM 扫描器
- [x] 目录递归扫描 (Backend)
- [x] 文件扩展名过滤 (Backend)
- [x] CRC32/MD5/SHA1 计算 (Backend)
- [x] 系统自动识别 (Backend)
- [x] 扫描目录管理 UI (Frontend)
- [x] 扫描进度展示 (Frontend)

### Phase 2: Scraper 核心

#### 2.1 ScraperManager 统一调度层
- [x] ScraperManager 核心实现
  - [x] Provider 注册/管理 (HashMap<String, Box<dyn Scraper>>)
  - [x] 统一搜索接口 (并行查询多 provider)
  - [x] 统一元数据/媒体获取接口
  - [x] 智能 scrape (自动匹配 + 聚合)
  - [ ] 批量 scrape (进度回调)
- [x] 标准化数据结构
  - [x] ScrapeQuery (name, system, hash, file_name)
  - [x] SearchResult (provider, source_id, name, confidence)
  - [x] GameMetadata (name, description, developer, publisher, genres, rating)
  - [x] MediaAsset (provider, url, asset_type, dimensions)
  - [x] MediaType 枚举 (BoxFront, Screenshot, Logo, Video, etc.)
- [x] Provider trait (可扩展接口)
  - [x] id() + display_name() -> 标识符
  - [x] capabilities() -> 支持的功能 (search, hash_lookup, metadata, media)
  - [x] search(query) -> Vec<SearchResult>
  - [x] get_metadata(source_id) -> GameMetadata
  - [x] get_media(source_id) -> Vec<MediaAsset>
  - [x] lookup_by_hash() -> 可选实现

#### 2.2 内置 Provider 实现
- [x] SteamGridDB (媒体为主，适配新 trait)
- [x] ScreenScraper (元数据+媒体，支持 Hash 查找)
- [ ] IGDB (元数据为主)
- [ ] TheGamesDB (免费，社区驱动)
- [ ] MobyGames (老游戏数据丰富)
- [ ] LaunchBox 本地数据库 (离线可用)
- [ ] 搜索引擎 + AI Scraper (兜底方案)

#### 2.3 智能匹配引擎
- [x] ROM 文件名解析（No-Intro 命名规范）
- [x] Hash 精确匹配 (CRC32/MD5/SHA1 → ScreenScraper)
- [x] 名称模糊匹配 (Jaro-Winkler 相似度算法)
- [x] 置信度评分 (名称+系统综合评估)
- [x] 多源数据聚合（优先级合并规则）
  - [x] 并行获取所有 provider 的元数据
  - [x] 按优先级合并元数据（优先级高的数据优先）
  - [x] 空字段由其他 provider 自动补充
  - [x] genres 字段自动去重合并
  - [x] 用户可配置 provider 优先级

#### 2.4 媒体下载
- [x] 并发下载队列 (Batch Scraper)
- [x] 断点续传 (Basic Implementation)
- [ ] 图片格式转换/压缩
- [x] 本地缓存管理

#### 2.6 中文数据库集成
- [x] 本地 rom-name-cn 仓库管理 (Git Clone/Pull)
- [x] CSV 解析与双路径搜索 (User Data + Resources)
- [x] 智能匹配算法 (Jaro-Winkler)
- [x] 独立管理页面与 Sidebar 入口
- [x] 目录命名检查工具 (Scan & Report)
- [x] 一键自动修复功能 (Auto-fix & Persistence)

### Phase 3: 导入导出

#### 3.1 导入功能 (即时读取)
- [x] EmulationStation gamelist.xml 解析
- [x] metadata.txt 解析
- [x] 临时元数据合并预览 (Temp metadata merging)
- [ ] LaunchBox XML 解析
- [ ] RetroArch .lpl 解析
- [x] 媒体资产关联 (Support local & temp media)

#### 3.2 导出功能
- [x] gamelist.xml 生成 (支持 <english-name>)
- [x] metadata.txt 生成 (Pegasus 格式，支持 Block 级替换)
- [ ] 自定义导出模板
- [x] 异步导出任务 (Support media synchronization)
- [x] 导出进度回调 (Tauri Emitter)

### Phase 4: 高级功能

#### 4.1 用户体验优化
- [x] 拖拽添加 ROM
- [ ] 批量编辑元数据
- [ ] 快捷键系统
- [x] 主题切换（暗/亮）
- [x] 统一视图组件 (Cover/Grid/List 合并为 RomView.tsx)
- [x] 视图切换平滑动画 (CSS transition，保持滚动位置)
- [x] 动态行高计算 (根据容器宽度和 aspect-ratio 自适应)
- [x] 启动 Splash Screen (HTML 内联，防止白屏闪烁)
- [x] 封面预加载 (启动时预加载前 50 个 ROM 封面)
- [x] Splash 加载步骤显示 (支持 i18n)
- [x] 中文 ROM 工具 UI 优化
  - [x] 响应式 Flex 布局
  - [x] 表格列宽平均分布
  - [x] 内容区域占满页面宽度
  - [x] 修复内容被 footer 遮挡问题
  - [x] 选择目录后自动扫描
- [x] 中文 ROM 工具增强功能
  - [x] 置信度可视化显示（背景色渐变：低分红色→高分透明）
  - [x] 点击英文名可编辑（Enter确认/ESC取消）
  - [x] 手动编辑实时保存到临时metadata
  - [x] 用户编辑的英文名自动设置为满分（100分）
  - [x] 自动去除英文名中的区域标签（如 (USA)）
  - [x] 按置信度排序（点击列头切换：降序→升序→取消）
  - [x] 表格列宽拖拽调整（鼠标拖拽列头分隔线）
- [x] i18n 合规性修复
  - [x] Settings.tsx - API 配置相关文字（13个翻译键）
  - [x] Scraper.tsx - 未配置凭证警告（3个翻译键）
  - [x] CnRomTools.tsx - 所有硬编码中文文字（50+个翻译键）
  - [x] 更新翻译文件（zh-CN.json 和 en.json）


#### 4.2 Settings & Configuration Management
- [x] API 配置管理
  - [x] 将 API 配置从 Scraper 页面移至 Settings 页面
  - [x] Provider 列表展示（SteamGridDB、ScreenScraper）
  - [x] 启用/禁用开关（支持未配置凭证时的状态保存）
  - [x] 凭证配置面板（用户名/密码/API Key）
  - [x] 配置持久化到 settings.json
  - [x] 修复未注册 provider 的开关状态保存问题
- [x] ChineseROMDB 架构调整
  - [x] 从 scraper provider 列表中移除
  - [x] 保留为独立的中文 ROM 工具
  - [x] update_cn_repo 命令移至 tools 模块
- [x] Provider 优先级管理
  - [x] ScraperConfig 添加 priority 字段
  - [x] 后端 set_priority() 方法和 API
  - [x] 前端 setProviderPriority 方法
  - [x] 优先级持久化到 settings.json
- [x] Provider 拖拽排序 UI
  - [x] 拖拽手柄图标和视觉反馈
  - [x] HTML5 drag and drop 实现
  - [x] 按 priority 排序显示
  - [x] 拖拽后自动重新计算优先级
  - [x] 乐观更新和错误回滚

#### 4.3 高级 Scraper
- [x] 批量 Scraper (Backend Queue & Auto-Match)
- [x] 批量操作 UI (Frontend)
- [ ] 自定义爬虫规则
- [ ] 代理设置
- [ ] 速率限制配置

#### 4.4 PS3 平台增强
- [x] PS3 模块架构重构
  - [x] 创建 ps3/ 目录统一管理 PS3 功能
  - [x] ps3/sfo.rs - PARAM.SFO 解析模块
  - [x] ps3/boxart.rs - Boxart/Logo 生成模块
  - [x] ps3/iso.rs - ISO9660 文件系统提取模块
  - [x] ps3/mod.rs - 模块入口和接口导出
- [x] PARAM.SFO 解析
  - [x] 从 PS3_GAME 文件夹解析游戏信息
  - [x] 从 ISO 文件解析游戏信息（ISO9660 文件系统）
  - [x] 提取游戏标题、ID、版本等元数据
- [x] ROM 扫描增强
  - [x] 自动识别 PS3_GAME 目录结构
  - [x] 混合目录支持（ISO 和文件夹混合扫描）
  - [x] 异步扫描避免 UI 阻塞
  - [x] 根目录模式下正确分组 PS3 游戏
- [x] Boxart 自动生成
  - [x] 图像合成引擎（image crate）
  - [x] PIC1.PNG 背景居中裁切（512x512）
  - [x] ICON0.PNG 图标叠加（左下角，128x128）
  - [x] Tauri command 接口（generate_ps3_boxart）
  - [x] 生成结果保存到 temp 目录
  - [x] 同时生成 Logo（直接提取 ICON0.PNG）
  - [x] 生成后自动刷新 ROM 库和详情页
- [ ] 批量 Boxart 生成
  - [ ] 为目录下所有 PS3 ROM 批量生成
  - [ ] 进度回调和取消支持

### Phase 5: 临时元数据架构 (Temp Metadata)

#### 5.1 目录结构设计
- [x] 统一临时数据目录结构
  ```
  {config_dir}/temp/{library_normalized}/{system}/
  ├── metadata.txt            # 临时 Pegasus 元数据文件
  ├── gamelist.xml            # 临时 EmulationStation 元数据文件
  └── media/
      └── {rom_file_stem}/    # 每个 ROM 独立媒体目录
          ├── boxfront.png    # 封面 (scraper/PS3 生成)
          ├── logo.png        # Logo (PS3 ICON0.PNG)
          ├── screenshot.png  # 截图
          └── video.mp4       # 视频预览
  ```
- [x] library_path 计算
  - [x] `rom.directory` 是 ROM 所在目录 (如 `Z:\ps3`)
  - [x] `library_path` = `rom.directory.parent()` (如 `Z:\`)
  - [x] 在 `persistence.rs`, `ps3.rs`, `naming_check.rs` 统一实现
- [x] 路径规范化 (`config.rs::normalize_path_to_dirname`)
  - [x] `Z:\` → `z`
  - [x] `D:\games\` → `d_games`
- [x] 支持多库隔离（不同驱动器/路径的 ROM 库独立存储）

#### 5.2 后端实现

##### 5.2.1 配置模块 (`src-tauri/src/config.rs`)
```rust
// 核心函数
get_config_dir()           // 配置根目录 (环境变量 CONFIG_DIR 或 exe/config/)
get_temp_dir()             // 临时目录 (config/temp/)
get_temp_dir_for_library() // 特定库的临时目录 (temp/{library}/{system}/)
normalize_path_to_dirname() // 路径规范化 (Z:\ → z)
```

##### 5.2.2 持久化模块 (`src-tauri/src/scraper/persistence.rs`)
- [x] `download_media()` - 下载媒体到 `media/{file_stem}/asset_type.ext`
- [x] `save_metadata_pegasus()` - 写入 Pegasus 格式元数据
- [x] `save_metadata_emulationstation()` - 写入 EmulationStation 格式元数据
- [x] 所有函数使用 `rom.directory.parent()` 计算 library_path

##### 5.2.3 Pegasus 解析器 (`src-tauri/src/scraper/pegasus.rs`)
- [x] 大小写不敏感键名匹配
  - [x] `assets.boxFront` / `assets.boxfront` / `assets.box_front` 统一处理
  - [x] 使用 `key.to_lowercase()` 进行匹配
- [x] 支持相对路径解析为绝对路径

##### 5.2.4 PS3 命令 (`src-tauri/src/commands/ps3.rs`)
```rust
#[tauri::command]
async fn generate_ps3_boxart(request: GenerateBoxartRequest) -> Result<GenerateBoxartResponse>

// Response 包含:
// - boxart_path / relative_boxart_path  (PIC1+ICON0 合成)
// - logo_path / relative_logo_path      (ICON0 直接提取)
```
- [x] 支持文件夹 ROM (PS3_GAME 目录)
- [x] 支持 ISO ROM (ISO9660 文件系统提取)
- [x] 输出到 `temp/{library}/{system}/media/{file_stem}/boxfront.png`
- [x] 同时生成 `logo.png` (ICON0.PNG)
- [x] 自动更新 metadata.txt 中的 assets 路径

##### 5.2.5 中文 ROM 工具 (`src-tauri/src/commands/naming_check.rs`)
- [x] `auto_fix_naming()` 合并逻辑
  ```rust
  // 1. 加载现有临时数据
  let existing = parse_existing_temp_metadata();
  // 2. 合并新数据，保留用户编辑
  for (key, new_entry) in new_entries {
      if existing[key].confidence == 100 {
          continue; // 跳过用户手动编辑的条目
      }
      merged.insert(key, new_entry);
  }
  // 3. 写入合并后的数据
  ```
- [x] `clean_english_name()` - 去除区域标签 `(USA)`, `[Europe]` 等

#### 5.3 前端实现

##### 5.3.1 封面优先级 (`src/components/rom/RomView.tsx`)
```typescript
// 获取 ROM 封面，优先使用 temp_data
function getRomCover(rom: Rom): string | undefined {
  return rom.temp_data?.box_front || rom.box_front || rom.gridicon;
}
```

##### 5.3.2 媒体 URL 预加载 (`src/lib/api.ts`)
```typescript
export async function preloadMediaUrls(roms: Rom[]): Promise<void> {
  const paths = roms.slice(0, PRELOAD_LIMIT).flatMap((rom) => {
    // 优先检查 temp_data
    const cover = rom.temp_data?.box_front || rom.box_front;
    return cover ? [cover] : [];
  });
  // 并发解析所有路径
  await Promise.all(paths.map(resolveMediaUrlAsync));
}
```

##### 5.3.3 生成后刷新 (`src/components/rom/RomDetail.tsx`)
```typescript
const handleGenerateBoxart = async () => {
  const result = await toolsApi.generatePs3Boxart(request);
  if (result.success) {
    // 刷新临时媒体列表
    await scraperApi.getTempMediaList(selectedLibrary.path);
    // 刷新 ROM 列表以更新封面
    await fetchRoms();
  }
};
```

##### 5.3.4 Rom 类型定义 (`src/types/index.ts`)
```typescript
interface Rom {
  // ... 基础字段
  temp_data?: {
    box_front?: string;
    logo?: string;
    screenshot?: string;
    video?: string;
    name?: string;
    english_name?: string;
    confidence?: number;
    [key: string]: any;
  };
}
```

#### 5.4 数据流图

```
┌─────────────────────────────────────────────────────────────┐
│                 Temp Metadata Data Flow                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. 生成/抓取阶段:                                            │
│     User Action (Scrape / Generate Boxart / Auto-fix CN)    │
│       → Backend Command (generate_ps3_boxart, etc.)         │
│       → library_path = rom.directory.parent()               │
│       → temp_dir = get_temp_dir_for_library(library_path)   │
│       → 写入 temp_dir/media/{file_stem}/boxfront.png        │
│       → 更新 temp_dir/metadata.txt                          │
│                                                              │
│  2. 加载阶段:                                                 │
│     scan_directory() / fetchRoms()                          │
│       → apply_temp_metadata(roms, library_path)             │
│       → 解析 temp_dir/metadata.txt                          │
│       → 填充 rom.temp_data (box_front, logo, etc.)          │
│       → 相对路径解析为绝对路径                                │
│                                                              │
│  3. 显示阶段:                                                 │
│     RomView.tsx                                              │
│       → getRomCover(rom) 获取封面路径                        │
│       → useMediaUrl(path) 转换为可显示的 URL                 │
│       → 显示图片                                             │
│                                                              │
│  4. 导入阶段 (TODO):                                          │
│     import_temp_data()                                       │
│       → 将 temp 数据复制到 ROM 目录                          │
│       → 合并 metadata 到 ROM 目录的 metadata.txt             │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

#### 5.5 关键文件清单

| 文件 | 职责 |
|------|------|
| `src-tauri/src/config.rs` | 配置目录管理、路径规范化 |
| `src-tauri/src/scraper/persistence.rs` | 媒体下载、元数据写入 |
| `src-tauri/src/scraper/pegasus.rs` | Pegasus 格式解析 |
| `src-tauri/src/commands/ps3.rs` | PS3 boxart/logo 生成命令 |
| `src-tauri/src/commands/scraper.rs` | get_temp_media_list API |
| `src-tauri/src/commands/naming_check.rs` | 中文 ROM 工具 |
| `src-tauri/src/rom_service.rs` | ROM 扫描、临时数据应用 |
| `src/components/rom/RomView.tsx` | 封面显示组件 |
| `src/components/rom/RomDetail.tsx` | ROM 详情面板 |
| `src/lib/api.ts` | 媒体 URL 解析、预加载 |
| `src/types/index.ts` | Rom 接口定义 |

### Phase 6: 配置架构重构 (本地/Docker 双模式)

#### 5.1 配置目录结构
- [x] 统一配置目录到 `./config/`
  - `config/settings.json` - 应用配置
  - `config/media/` - 媒体资产缓存
- [x] 环境变量支持 (`CONFIG_DIR` 覆盖默认路径)
- [x] Docker volume 挂载支持

#### 5.2 目录选择 UI 重构
- [x] 移除 Tauri dialog 依赖（Web 端不可用）
- [x] 新增手动输入路径 UI
- [x] 路径验证 API（后端验证目录是否存在/可读）
- [x] 目录浏览 API（后端返回目录列表供选择）

#### 5.3 部署模式支持
- [x] 本地模式：使用相对路径 `./config/`
- [x] Docker 模式：挂载 `/roms` volume
- [ ] 配置热重载支持

### Phase 6: Web 版本 (Docker 部署)

#### 6.1 Node.js 后端服务
- [x] Express + TypeScript 服务端
- [x] ROM 数据 API (`/api/roms`)
- [x] 媒体文件代理 API (`/api/media`)
- [x] Pegasus metadata 解析器 (移植自 Rust)
- [x] Media 目录自动扫描

#### 6.2 Docker 支持
- [x] 多阶段 Dockerfile (前端构建 + 后端构建 + 生产镜像)
- [x] docker-compose.yml 配置
- [x] 环境变量配置 (`ROMS_DIR`, `PORT`)
- [x] Volume 映射文档

#### 6.3 前端适配
- [x] 环境检测 (Tauri vs Web)
- [x] API 调用适配层 (`src/lib/api.ts`)
- [x] 媒体 URL 转换 (convertFileSrc vs HTTP URL)

---

## 🔗 参考资源

### API 文档
- [SteamGridDB API](https://www.steamgriddb.com/api/v2)
- [IGDB API](https://api-docs.igdb.com/)
- [TheGamesDB API](https://thegamesdb.net/api/)
- [MobyGames API](https://www.mobygames.com/info/api)
- [ScreenScraper API](https://www.screenscraper.fr/webapi2.php)

### 技术框架
- [Tauri](https://tauri.app/)
- [React](https://react.dev/)
- [TailwindCSS](https://tailwindcss.com/)
- [Express](https://expressjs.com/)
- [Docker](https://www.docker.com/)
