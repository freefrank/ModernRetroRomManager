// ROM 相关类型
export interface Rom {
  id: string;
  filename: string;
  path: string;
  systemId: string;
  size: number;
  crc32?: string;
  md5?: string;
  sha1?: string;
  createdAt: string;
  updatedAt: string;
  metadata?: RomMetadata;
  media?: MediaAsset[];
}

export interface RomMetadata {
  romId: string;
  name: string;
  description?: string;
  releaseDate?: string;
  developer?: string;
  publisher?: string;
  genre?: string[];
  players?: number;
  rating?: number;
  region?: string;
  scraperSource?: ScraperSource;
  scrapedAt?: string;
}

export interface MediaAsset {
  id: string;
  romId: string;
  assetType: MediaAssetType;
  path: string;
  width?: number;
  height?: number;
  fileSize?: number;
  sourceUrl?: string;
  downloadedAt: string;
}

export type MediaAssetType =
  | "boxfront"
  | "boxback"
  | "screenshot"
  | "video"
  | "logo"
  | "manual";

// 游戏系统/平台
export interface GameSystem {
  id: string;
  name: string;
  shortName: string;
  manufacturer?: string;
  releaseYear?: number;
  extensions: string[];
  igdbPlatformId?: number;
  thegamesdbPlatformId?: number;
}

// Scraper 相关
export type ScraperSource =
  | "igdb"
  | "steamgriddb"
  | "thegamesdb"
  | "mobygames"
  | "screenscraper"
  | "ai";

export interface ApiConfig {
  id: string;
  provider: ScraperSource;
  apiKey?: string;
  apiSecret?: string;
  username?: string;
  password?: string;
  enabled: boolean;
  priority: number;
}

// 扫描目录配置
export interface ScanDirectory {
  id: string;
  path: string;
  systemId?: string;
  recursive: boolean;
  enabled: boolean;
  lastScan?: string;
}

// 导入导出格式
export type ExportFormat =
  | "emulationstation"
  | "metadata"
  | "launchbox"
  | "retroarch";

// UI 状态
export type ViewMode = "grid" | "list";

export interface SortOption {
  field: "name" | "system" | "size" | "updatedAt";
  direction: "asc" | "desc";
}

export interface FilterOption {
  systemId?: string;
  hasMetadata?: boolean;
  hasMedia?: boolean;
  searchQuery?: string;
}
