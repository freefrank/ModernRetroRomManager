# RetroRomManager API 文档

## 概述

本文档描述 RetroRomManager 的内部 API 架构和外部 Scraper API 集成。

---

## Tauri Commands (Frontend ↔ Backend)

### ROM 管理

```typescript
// 扫描 ROM 目录
invoke('scan_rom_directory', { path: string }): Promise<ScanResult>

// 获取 ROM 列表
invoke('get_roms', { filter?: RomFilter }): Promise<Rom[]>

// 获取单个 ROM 详情
invoke('get_rom', { id: string }): Promise<RomDetail>

// 更新 ROM 元数据
invoke('update_rom', { id: string, data: Partial<RomData> }): Promise<void>
```

### Scraper

```typescript
// 搜索游戏信息
invoke('scrape_search', { query: string, system: string }): Promise<SearchResult[]>

// 获取游戏详情
invoke('scrape_game', { gameId: string, source: ScraperSource }): Promise<GameData>

// 批量 Scrape
invoke('scrape_batch', { romIds: string[], options: ScrapeOptions }): Promise<BatchResult>
```

### 导入/导出

```typescript
// 导入 gamelist.xml
invoke('import_gamelist', { path: string }): Promise<ImportResult>

// 导出 gamelist.xml
invoke('export_gamelist', { romIds: string[], path: string, format: ExportFormat }): Promise<void>
```

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
