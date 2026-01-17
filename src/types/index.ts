// ROM 相关类型
// ROM 相关类型
export interface Rom {
  file: string;
  name: string;
  description?: string;
  summary?: string;
  developer?: string;
  publisher?: string;
  genre?: string;
  players?: string;
  release?: string;
  rating?: string;
  boxart?: string;
  directory: string;
  system: string;
}

// 系统 ROM 列表（后端 get_roms 返回结构）
export interface SystemRoms {
  system: string;
  path: string;
  roms: Rom[];
}

// 废弃旧的 Metadata 接口，直接合并在 Rom 中
export interface MediaAsset {
  // 暂时保留用于兼容性，后续可能需要调整
  id: string;
  path: string;
  assetType: MediaAssetType;
}

export type MediaAssetType =
  | "boxfront"
  | "boxback"
  | "screenshot"
  | "video"
  | "logo"
  | "manual"
  | "hero"
  | "icon";

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
// 目录配置
export interface DirectoryConfig {
  path: string;
  isRootDirectory: boolean;
  metadataFormat: string;
  systemId?: string;
}

// 兼容别名
export type ScanDirectory = DirectoryConfig;

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
