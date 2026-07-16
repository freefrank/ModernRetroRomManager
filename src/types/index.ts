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
  directory: string;
  system: string;
  box_front?: string;
  box_back?: string;
  box_spine?: string;
  box_full?: string;
  cartridge?: string;
  logo?: string;
  marquee?: string;
  bezel?: string;
  gridicon?: string;
  flyer?: string;
  background?: string;
  music?: string;
  screenshot?: string;
  titlescreen?: string;
  video?: string;
  english_name?: string;
  chinese_name?: string;
  translated_languages?: string[];
  /** ROM 文件大小(bytes,文件夹型 ROM 或读取失败为空) */
  file_size?: number;
  /** ROM 文件修改时间(unix 秒,读取失败为空) */
  modified_at?: number;
  // 预览相关
  has_temp_metadata: boolean;
  temp_data?: Partial<Rom>;
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
  logo?: string;
  romType: "File" | "Folder";
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

// ScraperManager 统一类型
export interface ScraperProviderInfo {
  id: string;
  name: string;
  enabled: boolean;
  priority: number;
  requires_credentials: boolean;
  has_credentials: boolean;
  configured_username?: string;
  has_saved_password: boolean;
  capabilities: string[];
  rate_limit: number;
  threads: number;
  matched_count: number;
  selected_count: number;
  cache_hit_count: number;
  error_count: number;
  last_error?: string;
}

export interface ScraperCredentials {
  api_key?: string;
  username?: string;
  password?: string;
  rate_limit?: number;
  threads?: number;
}

export interface RomSystemSummary {
  system: string;
  path: string;
  romCount: number;
  scrapedCount: number;
  totalSize: number;
}

export interface ScraperSearchResult {
  provider: string;
  source_id: string;
  name: string;
  year?: string;
  system?: string;
  thumbnail?: string;
  asset_count?: number;
  confidence: number;
}

export interface ScraperGameMetadata {
  name: string;
  english_name?: string;
  chinese_name?: string;
  description?: string;
  release_date?: string;
  developer?: string;
  publisher?: string;
  genres: string[];
  players?: string;
  rating?: number;
  translated_languages?: string[];
}

export interface ScraperMediaAsset {
  provider: string;
  url: string;
  asset_type: string;
  width?: number;
  height?: number;
}

export interface ScrapeResult {
  metadata: ScraperGameMetadata;
  media: ScraperMediaAsset[];
}

export interface ApplyScrapedDataOptions {
  rom_id: string;
  directory: string;
  system: string;
  metadata: ScraperGameMetadata;
  selected_media: ScraperMediaAsset[];
  provider_id?: string;
}

export interface AiTranslationConfig {
  endpoint: string;
  model: string;
  target_language: string;
  has_api_key: boolean;
  merge_batch_requests: boolean;
  batch_token_limit: number;
  reasoning_effort: "auto" | "none" | "minimal" | "low" | "medium" | "high";
}

export interface AiTranslationConfigInput {
  endpoint: string;
  model: string;
  target_language: string;
  api_key?: string;
  merge_batch_requests: boolean;
  batch_token_limit: number;
  reasoning_effort: "auto" | "none" | "minimal" | "low" | "medium" | "high";
}

export interface TranslateMetadataRequest {
  system: string;
  file_name: string;
  metadata: ScraperGameMetadata;
}

export interface BatchTranslationResult {
  index: number;
  metadata: ScraperGameMetadata;
}


// 扫描目录配置
// 目录配置
export interface DirectoryConfig {
  id: string;
  name: string;
  path: string;
  isRootDirectory: boolean;
  metadataFormat: string;
  systemId?: string;
  /** null/undefined = all, [] = none, otherwise immediate child folder names */
  indexedFolders?: string[] | null;
  isActive: boolean;
}

export interface LibraryFolderInfo {
  name: string;
  path: string;
}

// 兼容别名
export type ScanDirectory = DirectoryConfig;

// 导入导出格式
export type ExportFormat =
  | "emulationstation"
  | "metadata"
  | "launchbox"
  | "retroarch";

// 已导入自定义主题信息(字段名与后端 snake_case 保持一致)
export interface CustomThemeInfo {
  /** theme.json 原文(前端解析后再走 validateManifest) */
  manifest_json: string;
  /** 解压目录绝对路径(assets 解析用) */
  dir: string;
  /** 清洗后的自定义 CSS(无则为 null) */
  custom_css?: string | null;
}

// UI 状态
export type ViewMode = "grid" | "list" | "cover";

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
