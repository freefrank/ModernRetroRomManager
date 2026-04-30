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
invoke('add_directory', { path, metadataFormat, isRoot, systemId }): Promise<void>
invoke('remove_directory', { path }): Promise<void>
invoke('scan_directory', { path }): Promise<DirectoryScanResult>
```

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
invoke('export_scraped_data', { system, directory }): Promise<void>
invoke('save_temp_metadata', { system, directory, romId, metadata }): Promise<void>
invoke('get_temp_media_list', { system, romId, romDirectory }): Promise<{ asset_type: string; path: string }[]>
invoke('delete_temp_media', { system, romId, assetType }): Promise<void>
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

### IGDB

- **Endpoint**: `https://api.igdb.com/v4`
- **认证**: Twitch OAuth
- **Rate Limit**: 4 req/sec
- **文档**: https://api-docs.igdb.com/

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

### MobyGames

- **Endpoint**: `https://api.mobygames.com/v1`
- **认证**: API Key
- **Rate Limit**: 100 req/day (免费)
- **文档**: https://www.mobygames.com/info/api

### ScreenScraper

- **Endpoint**: `https://www.screenscraper.fr/api2`
- **认证**: 用户名/密码
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

type ScraperSource = 'igdb' | 'steamgriddb' | 'thegamesdb' | 'mobygames' | 'screenscraper' | 'ai';

type ExportFormat = 'emulationstation' | 'metadata' | 'launchbox' | 'retroarch';
```
