# RetroRomManager API 文档

## 概述

本文档描述 RetroRomManager 当前已实现的桌面端（Tauri）命令与 Web API。

---

## Tauri Commands（前端 ↔ 后端）

以下命令名已与 `src/lib/api.ts` 和 `src-tauri/src/commands/*` 对齐。

### ROM 与目录管理

```typescript
invoke('get_roms', { filter?: RomFilter }): Promise<SystemRoms[]>
invoke('get_rom_stats'): Promise<RomStats>
invoke('get_roms_for_single_directory', { path, metadataFormat, isRoot, systemId }): Promise<SystemRoms[]>

invoke('get_directories'): Promise<ScanDirectory[]>
invoke('add_directory', { path, metadataFormat, isRoot, systemId }): Promise<ScanDirectory>
invoke('remove_directory', { libraryId }): Promise<void>
invoke('set_active_library', { libraryId }): Promise<ScanDirectory>
invoke('rename_library', { libraryId, name }): Promise<ScanDirectory>
invoke('scan_directory', { path }): Promise<DirectoryScanResult>
```

`ScanDirectory` 中包含稳定的 `id`、可编辑的 `name` 和 `isActive` 状态。每条目录记录都是一个独立 Library，ROM 列表和持久索引只作用于当前激活项。

### 中文命名工具

```typescript
invoke('scan_directory_for_naming_check', { path }): Promise<NamingCheckResult[]>
invoke('get_naming_check_results', { path }): Promise<NamingCheckResult[]>
invoke('auto_fix_naming', { path, system? }): Promise<{ success: number; failed: number }>
invoke('update_english_name', { directory, file, englishName }): Promise<void>
invoke('update_extracted_cn_name', { directory, file, extractedCnName }): Promise<void>
invoke('set_extracted_cn_as_name', { directory }): Promise<AutoFixResult>
invoke('add_english_as_tag', { directory }): Promise<AutoFixResult>
invoke('export_cn_metadata', { directory, exportFormat }): Promise<void>
```

### Scraper

```typescript
invoke('get_scraper_providers'): Promise<ScraperProviderInfo[]>
invoke('configure_scraper_provider', { providerId, credentials }): Promise<void>
invoke('scraper_search', { name, fileName, system? }): Promise<ScraperSearchResult[]>
invoke('scraper_get_metadata', { providerId, sourceId }): Promise<ScraperGameMetadata>
invoke('scraper_get_media', { providerId, sourceId }): Promise<ScraperMediaAsset[]>
invoke('scraper_auto_scrape', { name, fileName, system? }): Promise<ScrapeResult>
invoke('scraper_set_provider_enabled', { providerId, enabled }): Promise<void>
invoke('scraper_set_provider_priority', { providerId, priority }): Promise<void>
invoke('apply_scraped_data', { options }): Promise<void>
invoke('batch_scrape', { romIds, system, directory, providerId }): Promise<void>
invoke('cancel_batch_scrape'): Promise<void>
invoke('save_temp_metadata', { system, directory, romId, metadata }): Promise<void>
invoke('get_temp_media_list', { system, romId, romDirectory }): Promise<{ asset_type: string; path: string }[]>
invoke('delete_temp_media', { system, romId, assetType }): Promise<void>
```

### 导出与保存

`export_scraped_data` 的 `targetDirectory` 省略或等于源目录时为**原地写回**(保存到 ROM 目录);`format` 默认 `both`(同时写 `gamelist.xml` 与 `metadata.pegasus.txt`)。

```typescript
invoke('export_scraped_data', {
  system, directory,
  format?, targetDirectory?, nameMode?, romAssetsOnly?, syncDelete?
}): Promise<void>
invoke('export_library_scraped_data', {
  libraryId, format?, targetDirectory?, nameMode?, romAssetsOnly?, systemPaths?, syncDelete?
}): Promise<void>
invoke('cancel_export'): Promise<boolean>
```

### 隐藏 / 删除 / 工具

```typescript
invoke('get_hidden_roms'): Promise<HiddenRom[]>
invoke('set_rom_hidden', { directory, file, hidden }): Promise<void>
invoke('delete_rom', { directory, file }): Promise<void>
invoke('open_rom_location', { directory, file }): Promise<void>
invoke('cancel_rom_scan'): Promise<void>

// 中文 ROM 数据库更新
invoke('update_cn_repo'): Promise<void>
// 解压整理压缩包 ROM
invoke('organize_rom_archives', { directory, system, password }): Promise<ArchiveOrganizeResult>
// 多碟整理:各碟归入 <基名>/ 子文件夹、生成 <基名>.m3u,临时元数据折叠成一条(保存时调用)
invoke('organize_multidisc_games', { directory, system }): Promise<OrganizeReport>
```

`OrganizeReport`: `{ changed: boolean; groups: number; filesMoved: number; m3uWritten: number }`。非光盘平台、无临时元数据或无多碟组时 `changed=false`(空操作,幂等)。

### 主题

```typescript
invoke('list_custom_themes'): Promise<CustomTheme[]>
invoke('import_theme_pack', { path }): Promise<CustomTheme>
invoke('delete_custom_theme', { id }): Promise<void>
```

### 其他命令

```typescript
invoke('get_systems'): Promise<GameSystem[]>
invoke('generate_ps3_boxart', { request: { romFile, romDirectory, system } }): Promise<{ success: boolean; boxartPath: string; error?: string }>
```

---

## Web API（Node.js）

当前已实现路由（`server/src/index.ts`）：

```http
GET /api/health
GET /api/roms
GET /api/media?path=...
```

鉴权要求：

- `/api/*` 路径统一校验请求头 `X-API-Key`
- 未携带或不匹配时返回 `401 Unauthorized`

---

## 外部 Scraper API

当前已集成的 Provider 为 **SteamGridDB**、**TheGamesDB**、**ScreenScraper**,外加本地中文数据库与搜索引擎 + AI 名称解析兜底。IGDB、MobyGames 尚未集成(见路线图)。

### SteamGridDB

- **Endpoint**: `https://www.steamgriddb.com/api/v2`
- **认证**: API Key
- **Rate Limit**: 无硬性限制
- **资源类型**: Grid (封面), Hero (横幅), Logo, Icon
- **文档**: https://www.steamgriddb.com/api/v2

### TheGamesDB

- **Endpoint**: `https://api.thegamesdb.net/v1`
- **认证**: API Key
- **Rate Limit**: 3000 req/day (免费)
- **文档**: https://thegamesdb.net/api/

### ScreenScraper

- **Endpoint**: `https://api.screenscraper.fr/api2`
- **应用认证**: 内置并轻量混淆的 `devid` / `devpassword`，`softname=ModernRetroRomManager`
- **会员认证**: 用户在设置中填写自己的 `ssid` / `sspassword`
- **Rate Limit**: 按账户级别
- **文档**: https://www.screenscraper.fr/webapi2.php

### 搜索引擎 + AI (兜底方案)

当传统 API 无法匹配时，使用搜索引擎抓取网页，AI 提取结构化数据。

- **搜索引擎**: Google / Bing / DuckDuckGo (可配置)
- **AI 提取**: 本地 LLM (Ollama) 或云端 API (OpenAI / Claude)
- **工作流程**:
  1. ROM 文件名清洗解析
  2. 搜索引擎查询
  3. 抓取搜索结果页面
  4. AI 提取结构化数据
  5. 用户确认后入库

---

## 数据类型定义

```typescript
interface Rom {
  id: string;
  filename: string;
  path: string;
  system: string;
  hash: {
    crc32: string;
    md5: string;
    sha1: string;
  };
  metadata?: RomMetadata;
}

interface RomMetadata {
  name: string;
  description?: string;
  releaseDate?: string;
  developer?: string;
  publisher?: string;
  genre?: string[];
  players?: number;
  rating?: number;
  region?: string;
  media?: MediaAssets;
}

interface MediaAssets {
  boxFront?: string;
  boxBack?: string;
  screenshot?: string[];
  video?: string;
  logo?: string;
  manual?: string;
}

type ScraperSource = 'steamgriddb' | 'thegamesdb' | 'screenscraper' | 'local_cn' | 'ai';

// 导出格式:both = 同时写 gamelist.xml 与 metadata.pegasus.txt(默认)
type ExportFormat = 'both' | 'pegasus' | 'emulationstation';

// 导出命名模式:保留原始名 / 使用中文名
type ExportNameMode = 'original' | 'chinese';
```

> 注:`Rom` / `RomMetadata` 的字段以 `src/types/index.ts` 为准(如 `box_front`、`titlescreen` 等 15 类资源字段直接平铺在 `Rom` 上),上方结构仅为示意。
