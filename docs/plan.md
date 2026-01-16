# ModernRetroManager - 现代化 Retro ROM 管理软件

## 🎯 项目愿景

打造一款**现代化、跨平台、开源**的 Retro ROM 管理软件，替代老旧的 ARRM 和 Skraper，摆脱对 screenscraper.fr 的过度依赖。

### 核心目标
- 🌐 **双模式部署**：可 Self-host 也可打包成 Native App (Win/Mac/Linux)
- 🎨 **现代化 UI**：使用最新前端技术，美观且高效
- 🔌 **多源 Scraping**：整合多个 API 和爬虫源
- 📦 **兼容性强**：支持导入现有 metadata.txt、playlist.xml 等格式

---

## 🏗️ 技术架构

### 技术栈

```
┌─────────────────────────────────────────────────────────────┐
│                      Frontend Layer                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │         React + TypeScript + TailwindCSS            │   │
│  │              (Vite 构建工具)                         │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                      Backend Layer                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Rust (Tauri Framework)                 │   │
│  │     - 轻量级 (~5MB vs Electron ~150MB)              │   │
│  │     - 高性能文件/ROM处理                            │   │
│  │     - 跨平台编译 (Win/Mac/Linux)                    │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                      Data Layer                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │    SQLite (本地) / PostgreSQL (Self-host)           │   │
│  │              ORM: Diesel (Rust)                     │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 📋 功能模块

### 1. ROM 库管理
- 扫描本地 ROM 目录
- 自动识别 ROM 系统/平台
- 支持多种 ROM 格式 (.zip, .7z, .iso, .bin/.cue, etc.)
- ROM 文件校验 (CRC32, MD5, SHA1)

### 2. 元数据导入/导出
- EmulationStation gamelist.xml
- metadata.txt (Pegasus/Recalbox/Batocera)
- LaunchBox XML
- RetroArch playlist (.lpl)

### 3. Scraper 引擎

| 源 | API类型 | 优先级 | 说明 |
|----|---------|--------|------|
| IGDB | REST | ⭐⭐⭐⭐⭐ | Twitch 旗下，数据全面 |
| SteamGridDB | REST | ⭐⭐⭐⭐⭐ | 高质量封面/Logo/图标 |
| TheGamesDB | REST | ⭐⭐⭐⭐ | 社区驱动，免费 |
| MobyGames | REST | ⭐⭐⭐⭐ | 老游戏数据丰富 |
| ScreenScraper | REST | ⭐⭐⭐ | 需注册，媒体资源多 |
| LaunchBox | 本地DB | ⭐⭐⭐ | 离线可用 |
| 搜索引擎 + AI | 混合 | ⭐⭐⭐ | 兜底方案，处理冷门游戏 |

#### AI Scraper 工作流程
```
ROM 文件名 → 清洗/解析 → 搜索引擎查询 → 抓取搜索结果页面
                                              ↓
                              AI 提取结构化数据 (名称、简介、发行日期等)
                                              ↓
                                     用户确认 → 入库
```

- **搜索引擎**：Google/Bing/DuckDuckGo（可配置）
- **AI 模型**：本地 LLM (Ollama) 或云端 API (OpenAI/Claude)
- **使用场景**：当传统 API 无法匹配时的兜底方案

### 4. 媒体资产管理
- Box Art, Screenshot, Video, Logo, Manual
- 本地存储管理
- 图片压缩/格式转换

---

## 📁 项目结构

```
ModernRetroManager/
├── docs/                    # 文档
│   ├── plan.md             # 项目规划
│   ├── api.md              # API 文档
│   └── user-guide.md       # 用户指南
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── main.rs         # 入口
│   │   ├── commands/       # Tauri Commands
│   │   │   ├── mod.rs
│   │   │   ├── rom.rs      # ROM 管理命令
│   │   │   ├── scraper.rs  # Scraper 命令
│   │   │   └── import.rs   # 导入导出命令
│   │   ├── db/             # 数据库层
│   │   │   ├── mod.rs
│   │   │   ├── models.rs   # 数据模型
│   │   │   ├── schema.rs   # 表结构
│   │   │   └── migrations/ # 数据库迁移
│   │   ├── scraper/        # Scraper 引擎
│   │   │   ├── mod.rs
│   │   │   ├── igdb.rs
│   │   │   ├── thegamesdb.rs
│   │   │   ├── mobygames.rs
│   │   │   └── screenscraper.rs
│   │   ├── scanner/        # ROM 扫描器
│   │   │   ├── mod.rs
│   │   │   ├── hash.rs     # 文件哈希计算
│   │   │   └── detect.rs   # 系统检测
│   │   └── utils/          # 工具函数
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                     # React 前端
│   ├── components/         # 通用组件
│   │   ├── ui/             # 基础 UI 组件
│   │   ├── rom/            # ROM 相关组件
│   │   └── layout/         # 布局组件
│   ├── pages/              # 页面
│   │   ├── Library.tsx     # 库管理
│   │   ├── Scraper.tsx     # Scraper 设置
│   │   ├── Settings.tsx    # 系统设置
│   │   └── Import.tsx      # 导入导出
│   ├── hooks/              # 自定义 Hooks
│   ├── stores/             # 状态管理 (Zustand)
│   ├── types/              # TypeScript 类型
│   ├── utils/              # 工具函数
│   ├── App.tsx
│   └── main.tsx
├── package.json
├── vite.config.ts
├── tailwind.config.js
├── tsconfig.json
└── README.md
```

---

## 🗄️ 数据库设计

### 核心表结构

```sql
-- ROM 表
CREATE TABLE roms (
  id TEXT PRIMARY KEY,
  filename TEXT NOT NULL,
  path TEXT NOT NULL,
  system_id TEXT NOT NULL REFERENCES systems(id),
  size INTEGER NOT NULL,
  crc32 TEXT,
  md5 TEXT,
  sha1 TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 游戏系统/平台表
CREATE TABLE systems (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,           -- 显示名称
  short_name TEXT NOT NULL,     -- 简称 (如 nes, snes, psx)
  manufacturer TEXT,            -- 制造商
  release_year INTEGER,
  extensions TEXT NOT NULL,     -- 支持的扩展名 JSON 数组
  igdb_platform_id INTEGER,     -- IGDB 平台 ID 映射
  thegamesdb_platform_id INTEGER
);

-- ROM 元数据表
CREATE TABLE rom_metadata (
  rom_id TEXT PRIMARY KEY REFERENCES roms(id),
  name TEXT NOT NULL,
  description TEXT,
  release_date TEXT,
  developer TEXT,
  publisher TEXT,
  genre TEXT,                   -- JSON 数组
  players INTEGER,
  rating REAL,
  region TEXT,
  scraper_source TEXT,          -- 数据来源
  scraped_at DATETIME
);

-- 媒体资产表
CREATE TABLE media_assets (
  id TEXT PRIMARY KEY,
  rom_id TEXT NOT NULL REFERENCES roms(id),
  asset_type TEXT NOT NULL,     -- boxfront, boxback, screenshot, video, logo, manual
  path TEXT NOT NULL,
  width INTEGER,
  height INTEGER,
  file_size INTEGER,
  source_url TEXT,
  downloaded_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- API 密钥配置表
CREATE TABLE api_configs (
  id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,       -- igdb, thegamesdb, mobygames, screenscraper
  api_key TEXT,
  api_secret TEXT,
  username TEXT,
  password TEXT,
  enabled INTEGER DEFAULT 1,
  priority INTEGER DEFAULT 0
);

-- 扫描目录配置表
CREATE TABLE scan_directories (
  id TEXT PRIMARY KEY,
  path TEXT NOT NULL,
  system_id TEXT REFERENCES systems(id),
  recursive INTEGER DEFAULT 1,
  enabled INTEGER DEFAULT 1,
  last_scan DATETIME
);

-- 索引
CREATE INDEX idx_roms_system ON roms(system_id);
CREATE INDEX idx_roms_hash ON roms(crc32, md5, sha1);
CREATE INDEX idx_media_rom ON media_assets(rom_id);
CREATE INDEX idx_media_type ON media_assets(asset_type);
```

### 预置系统数据

支持的游戏系统（初始版本）：

| 系统 | Short Name | 扩展名 |
|------|------------|--------|
| Nintendo Entertainment System | nes | .nes, .zip, .7z |
| Super Nintendo | snes | .sfc, .smc, .zip, .7z |
| Nintendo 64 | n64 | .n64, .z64, .v64, .zip |
| Game Boy | gb | .gb, .zip |
| Game Boy Color | gbc | .gbc, .zip |
| Game Boy Advance | gba | .gba, .zip |
| Nintendo DS | nds | .nds, .zip |
| Sega Genesis/Mega Drive | genesis | .md, .bin, .gen, .zip |
| Sega Saturn | saturn | .iso, .cue, .bin |
| Sega Dreamcast | dreamcast | .cdi, .gdi, .iso |
| PlayStation | psx | .iso, .bin, .cue, .pbp |
| PlayStation 2 | ps2 | .iso, .bin |
| PlayStation Portable | psp | .iso, .cso |
| Arcade (MAME) | arcade | .zip |
| Neo Geo | neogeo | .zip |
| PC Engine | pce | .pce, .zip |
| Atari 2600 | atari2600 | .a26, .bin, .zip |

---

## 🎨 UI/UX 设计规范

### 设计原则

1. **暗色主题优先**：黑底配色，减少视觉疲劳
2. **信息密度适中**：一屏展示足够信息，避免过多滚动
3. **操作高效**：支持批量操作、拖拽、快捷键
4. **响应式设计**：适配不同窗口尺寸

### 配色方案

```css
:root {
  /* 主色调 */
  --bg-primary: #0d0d0d;        /* 主背景 */
  --bg-secondary: #1a1a1a;      /* 次级背景 */
  --bg-tertiary: #262626;       /* 卡片背景 */

  /* 强调色 */
  --accent-primary: #6366f1;    /* 主强调色 (Indigo) */
  --accent-secondary: #8b5cf6;  /* 次强调色 (Purple) */
  --accent-success: #22c55e;    /* 成功 */
  --accent-warning: #f59e0b;    /* 警告 */
  --accent-error: #ef4444;      /* 错误 */

  /* 文字 */
  --text-primary: #f5f5f5;      /* 主文字 */
  --text-secondary: #a3a3a3;    /* 次级文字 */
  --text-muted: #525252;        /* 弱化文字 */

  /* 边框 */
  --border-default: #333333;
  --border-hover: #525252;
}
```

### 主要页面布局

```
┌────────────────────────────────────────────────────────────┐
│  Logo    [搜索栏]                    [设置] [通知]         │
├──────────┬─────────────────────────────────────────────────┤
│          │                                                 │
│  导航栏   │              主内容区                           │
│          │                                                 │
│  📚 库    │   ┌─────────────────────────────────────────┐  │
│  🔍 Scrape│   │  工具栏: [筛选] [排序] [视图切换]       │  │
│  📥 导入  │   ├─────────────────────────────────────────┤  │
│  📤 导出  │   │                                         │  │
│  ⚙️ 设置  │   │           ROM 列表/网格视图             │  │
│          │   │                                         │  │
│          │   │                                         │  │
│          │   └─────────────────────────────────────────┘  │
│          │                                                 │
├──────────┴─────────────────────────────────────────────────┤
│  状态栏: ROM数量 | 已Scrape | 存储占用 | 任务进度          │
└────────────────────────────────────────────────────────────┘
```

---

## 🚀 开发路线图

### Phase 1: 基础框架 (MVP)

#### 1.1 项目初始化
- [x] 项目规划文档
- [ ] Tauri + React + TypeScript 项目搭建
- [ ] TailwindCSS 配置
- [ ] 基础路由配置 (React Router)

#### 1.2 数据库层
- [ ] SQLite + Diesel 集成
- [ ] 数据库迁移脚本
- [ ] 预置系统数据导入
- [ ] CRUD 基础操作

#### 1.3 ROM 扫描器
- [ ] 目录递归扫描
- [ ] 文件扩展名过滤
- [ ] CRC32/MD5/SHA1 计算
- [ ] 系统自动识别（基于目录名/扩展名）

#### 1.4 基础 UI
- [ ] 侧边栏导航
- [ ] ROM 列表视图（表格）
- [ ] ROM 网格视图（封面）
- [ ] ROM 详情面板
- [ ] 全局搜索

### Phase 2: Scraper 核心

#### 2.1 API 集成
- [ ] IGDB API 客户端
  - Twitch OAuth 认证
  - 游戏搜索
  - 封面/截图获取
- [ ] SteamGridDB API 客户端
  - Grid/Hero/Logo/Icon 获取
  - 多尺寸资源支持
- [ ] TheGamesDB API 客户端
- [ ] MobyGames API 客户端
- [ ] ScreenScraper API 客户端
- [ ] 搜索引擎 + AI Scraper
  - 搜索引擎集成 (Google/Bing/DuckDuckGo)
  - 网页内容抓取
  - AI 结构化提取 (Ollama/OpenAI/Claude)

#### 2.2 智能匹配
- [ ] ROM 文件名解析（No-Intro 命名规范）
- [ ] Hash 精确匹配
- [ ] 模糊搜索 + 用户确认
- [ ] 多源数据聚合（优先级合并）

#### 2.3 媒体下载
- [ ] 并发下载队列
- [ ] 断点续传
- [ ] 图片格式转换/压缩
- [ ] 本地缓存管理

### Phase 3: 导入导出

#### 3.1 导入功能
- [ ] EmulationStation gamelist.xml 解析
- [ ] metadata.txt 解析
- [ ] LaunchBox XML 解析
- [ ] RetroArch .lpl 解析
- [ ] 媒体资产关联

#### 3.2 导出功能
- [ ] gamelist.xml 生成
- [ ] metadata.txt 生成
- [ ] 自定义导出模板
- [ ] 批量导出

### Phase 4: 高级功能

#### 4.1 用户体验优化
- [ ] 拖拽添加 ROM
- [ ] 批量编辑元数据
- [ ] 快捷键系统
- [ ] 主题切换（暗/亮）

#### 4.2 高级 Scraper
- [ ] 自定义爬虫规则
- [ ] 代理设置
- [ ] 速率限制配置

#### 4.3 插件系统（远期）
- [ ] 插件 API 设计
- [ ] 自定义 Scraper 源
- [ ] 自定义导出格式

---

## 🧪 测试策略

### 单元测试
- **Rust 后端**：使用 `cargo test`
  - 数据库操作测试
  - 文件扫描测试
  - Hash 计算测试

- **React 前端**：使用 Vitest
  - 组件单元测试
  - 工具函数测试

### 集成测试
- Tauri Command 端到端测试
- API Mock 测试

### 手动测试 Checklist
- [ ] Windows 10/11 安装运行
- [ ] macOS (Intel/Apple Silicon) 安装运行
- [ ] Linux (Ubuntu/Arch) 安装运行
- [ ] 大规模 ROM 库扫描 (1000+ 文件)
- [ ] 各 Scraper 源连通性

---

## 📦 发布与部署

### Native App 发布

#### Windows
- 输出格式：`.msi`, `.exe`
- 签名：可选 (Windows SmartScreen)
- 更新：Tauri Updater

#### macOS
- 输出格式：`.dmg`, `.app`
- 签名：需要 Apple Developer 账号（可选）
- 公证：可选

#### Linux
- 输出格式：`.deb`, `.AppImage`, `.tar.gz`
- 发布渠道：GitHub Releases, AUR

### Self-Host 部署（远期）

- Docker 镜像
- PostgreSQL 支持
- 多用户认证

---

## 💡 与 ARRM/Skraper 的差异化

| 特性 | ModernRetroManager | ARRM | Skraper |
|------|-----------------|------|---------|
| 开源 | ✅ MIT | ❌ | ❌ |
| 跨平台 | ✅ Win/Mac/Linux | ⚠️ Win Only | ✅ |
| Self-Host | ✅ | ❌ | ❌ |
| 多 Scraper 源 | ✅ 5+ | ✅ | ⚠️ |
| 现代 UI | ✅ | ⚠️ | ⚠️ |
| 性能 | ⚡ Rust 原生 | .NET | .NET |
| 安装包大小 | ~5-10MB | ~50MB | ~50MB |
| 离线使用 | ✅ | ✅ | ⚠️ |

---

## 🔗 参考资源

### API 文档
- [IGDB API](https://api-docs.igdb.com/)
- [TheGamesDB API](https://thegamesdb.net/api/)
- [MobyGames API](https://www.mobygames.com/info/api)
- [ScreenScraper API](https://www.screenscraper.fr/webapi2.php)

### 技术框架
- [Tauri](https://tauri.app/)
- [React](https://react.dev/)
- [TailwindCSS](https://tailwindcss.com/)
- [Diesel ORM](https://diesel.rs/)

### 参考项目
- [EmulationStation](https://emulationstation.org/)
- [Skraper](https://www.skraper.net/)
- [ARRM](https://github.com/cosmo0/retrogaming-tools)

---

## 📄 许可证

MIT License
