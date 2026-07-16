import type { SystemRoms, RomSystemSummary, GameSystem, ScanDirectory, Rom, ScraperProviderInfo, ScraperCredentials, ScraperSearchResult, ScraperGameMetadata, ScraperMediaAsset, ScrapeResult, ApplyScrapedDataOptions, CustomThemeInfo, AiTranslationConfig, AiTranslationConfigInput, TranslateMetadataRequest, BatchTranslationResult } from "@/types";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export const isTauri = (): boolean => {
  return typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;
};

const API_BASE = import.meta.env.VITE_API_URL || "/api";

// ============ Media URL Cache ============
const mediaUrlCache = new Map<string, string>();
const mediaUrlRequests = new Map<string, Promise<string | null>>();
let mediaUrlGeneration = 0;

export function getCachedMediaUrl(path: string | undefined): string | null {
  if (!path) return null;
  return mediaUrlCache.get(path) ?? null;
}

export function clearCachedMediaUrls(): void {
  mediaUrlGeneration += 1;
  mediaUrlCache.clear();
  mediaUrlRequests.clear();
}

export async function preloadMediaUrls(roms: Rom[], limit = 50): Promise<void> {
  const paths = roms
    .slice(0, limit)
    .map(rom => rom.temp_data?.box_front || rom.box_front || rom.gridicon)
    .filter((path): path is string => !!path && !mediaUrlCache.has(path));

  await Promise.all(
    paths.map((path) => resolveMediaUrlAsync(path))
  );
}

// ============ API Functions ============

async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

async function httpFetch<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${endpoint}`, {
    headers: { "Content-Type": "application/json" },
    ...options,
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}: ${res.statusText}`);
  return res.json();
}

export const api = {
  async getRoms(): Promise<SystemRoms[]> {
    if (isTauri()) {
      return tauriInvoke<SystemRoms[]>("get_roms", {});
    }
    return httpFetch<SystemRoms[]>("/roms");
  },

  async getRomLibrarySummary(): Promise<RomSystemSummary[]> {
    if (isTauri()) {
      return tauriInvoke<RomSystemSummary[]>("get_rom_library_summary");
    }
    const systems = await this.getRoms();
    return systems.map((entry) => ({
      system: entry.system,
      path: entry.path,
      romCount: entry.roms.length,
      scrapedCount: 0,
      totalSize: entry.roms.reduce((sum, rom) => sum + (rom.file_size || 0), 0),
    }));
  },

  async getLibraryRomSummary(libraryId: string): Promise<RomSystemSummary[]> {
    if (isTauri()) {
      return tauriInvoke<RomSystemSummary[]>("get_library_rom_summary", { libraryId });
    }
    return this.getRomLibrarySummary();
  },

  async getSystemRoms(system: string): Promise<SystemRoms> {
    if (isTauri()) {
      return tauriInvoke<SystemRoms>("get_system_roms", { system });
    }
    const systems = await this.getRoms();
    const found = systems.find((entry) => entry.system === system);
    if (!found) throw new Error(`System not found: ${system}`);
    return found;
  },

  async scanRomLibrary(full: boolean, libraryId?: string): Promise<RomSystemSummary[]> {
    if (isTauri()) {
      return tauriInvoke<RomSystemSummary[]>("scan_rom_library", { full, libraryId });
    }
    return this.getRomLibrarySummary();
  },

  async getSystems(): Promise<GameSystem[]> {
    if (isTauri()) {
      return tauriInvoke<GameSystem[]>("get_systems");
    }
    return [];
  },

  async getDirectories(): Promise<ScanDirectory[]> {
    if (isTauri()) {
      return tauriInvoke<ScanDirectory[]>("get_directories");
    }
    return [];
  },

  async addDirectory(path: string, metadataFormat: string, isRoot: boolean, systemId: string | null): Promise<ScanDirectory | null> {
    if (isTauri()) {
      return tauriInvoke<ScanDirectory>("add_directory", { path, metadataFormat, isRoot, systemId });
    }
    return null;
  },

  async removeDirectory(libraryId: string): Promise<void> {
    if (isTauri()) {
      await tauriInvoke("remove_directory", { libraryId });
    }
  },

  async setActiveLibrary(libraryId: string): Promise<ScanDirectory | null> {
    if (isTauri()) {
      return tauriInvoke<ScanDirectory>("set_active_library", { libraryId });
    }
    return null;
  },

  async renameLibrary(libraryId: string, name: string): Promise<ScanDirectory | null> {
    if (isTauri()) {
      return tauriInvoke<ScanDirectory>("rename_library", { libraryId, name });
    }
    return null;
  },

  async getRomsForDirectory(path: string, metadataFormat: string, isRoot: boolean, systemId: string | null): Promise<SystemRoms[]> {
    if (isTauri()) {
      return tauriInvoke<SystemRoms[]>("get_roms_for_single_directory", { path, metadataFormat, isRoot, systemId });
    }
    return [];
  },

  async getStats(): Promise<{ total_roms: number; total_systems: number }> {
    if (isTauri()) {
      return tauriInvoke("get_rom_stats");
    }
    const roms = await this.getRoms();
    return {
      total_roms: roms.reduce((acc, s) => acc + s.roms.length, 0),
      total_systems: roms.length,
    };
  },

/** 扫描目录进行命名检查 */
  async scanDirectoryForNamingCheck(path: string): Promise<{ file: string; name: string; english_name?: string }[]> {
    if (isTauri()) {
      return await tauriInvoke("scan_directory_for_naming_check", { path });
    }
    return [];
  },

  /** 获取命名检查结果（只读取临时元数据，不扫描文件系统） */
  async getNamingCheckResults(path: string): Promise<{ file: string; name: string; english_name?: string }[]> {
    if (isTauri()) {
      return await tauriInvoke("get_naming_check_results", { path });
    }
    return [];
  },

  /** 自动修复目录命名 */
  async autoFixNaming(path: string, system?: string): Promise<{ success: number; failed: number }> {
    if (isTauri()) {
      return await tauriInvoke("auto_fix_naming", { path, system });
    }
    return { success: 0, failed: 0 };
  },
};

// Normalize path separators for Windows compatibility
function normalizePath(path: string): string {
  // Convert forward slashes to backslashes on Windows paths
  if (path.match(/^[A-Za-z]:/)) {
    return path.replace(/\//g, '\\');
  }
  return path;
}

/**
 * 解析系统 logo(打包资源目录 logo/<文件名>)为可显示 URL。
 * 非 Tauri 环境或解析失败返回 null,由调用方回退占位图标。
 */
export async function resolveSystemLogoUrl(logoFileName: string | undefined): Promise<string | null> {
  if (!logoFileName || !isTauri()) return null;
  try {
    const { convertFileSrc } = await import("@tauri-apps/api/core");
    const resourcePath = await tauriInvoke<string>("get_system_logo_path", { fileName: logoFileName });
    return convertFileSrc(resourcePath);
  } catch (error) {
    console.error("Failed to resolve system logo:", logoFileName, error);
    return null;
  }
}

export async function resolveMediaUrlAsync(path: string | undefined): Promise<string | null> {
  if (!path) return null;
  if (path.startsWith("http") || path.startsWith("data:")) return path;

  const cached = mediaUrlCache.get(path);
  if (cached) return cached;
  const pending = mediaUrlRequests.get(path);
  if (pending) return pending;

  const normalizedPath = normalizePath(path);
  const requestGeneration = mediaUrlGeneration;
  const request = (async () => {
    if (isTauri()) {
      const { convertFileSrc } = await import("@tauri-apps/api/core");
      let displayPath = normalizedPath;
      try {
        displayPath = await tauriInvoke<string>("resolve_cached_library_asset", {
          path: normalizedPath,
        });
      } catch (error) {
        console.warn("Failed to cache library asset, using source path:", error);
      }
      const url = convertFileSrc(displayPath);
      if (requestGeneration === mediaUrlGeneration) mediaUrlCache.set(path, url);
      return url;
    }

    const url = `${API_BASE}/media?path=${encodeURIComponent(normalizedPath)}`;
    if (requestGeneration === mediaUrlGeneration) mediaUrlCache.set(path, url);
    return url;
  })();
  mediaUrlRequests.set(path, request);
  try {
    return await request;
  } finally {
    if (mediaUrlRequests.get(path) === request) mediaUrlRequests.delete(path);
  }
}

// ============ Scraper API ============

export const scraperApi = {
  /** 获取所有可用的 provider */
  async getProviders(): Promise<ScraperProviderInfo[]> {
    if (isTauri()) {
      return tauriInvoke<ScraperProviderInfo[]>("get_scraper_providers");
    }
    return [];
  },

  /** 配置 provider 凭证 */
  async configureProvider(providerId: string, credentials: ScraperCredentials): Promise<void> {
    if (isTauri()) {
      await tauriInvoke("configure_scraper_provider", { providerId, credentials });
    }
  },

  /** 搜索游戏 */
  async search(name: string, fileName: string, system?: string, directory?: string): Promise<ScraperSearchResult[]> {
    if (isTauri()) {
      return tauriInvoke<ScraperSearchResult[]>("scraper_search", { name, fileName, system, directory });
    }
    return [];
  },

  /** 获取游戏元数据 */
  async getMetadata(providerId: string, sourceId: string): Promise<ScraperGameMetadata> {
    if (isTauri()) {
      return tauriInvoke<ScraperGameMetadata>("scraper_get_metadata", { providerId, sourceId });
    }
    throw new Error("Not available in web mode");
  },

  /** 获取媒体资产 */
  async getMedia(
    providerId: string,
    sourceId: string,
    context?: { romDirectory: string; system: string; romId: string; mediaTypes?: string[] },
  ): Promise<ScraperMediaAsset[]> {
    if (isTauri()) {
      return tauriInvoke<ScraperMediaAsset[]>("scraper_get_media", {
        providerId,
        sourceId,
        romDirectory: context?.romDirectory,
        system: context?.system,
        romId: context?.romId,
        mediaTypes: context?.mediaTypes,
      });
    }
    return [];
  },

  async getMediaTypes(): Promise<string[]> {
    return isTauri() ? tauriInvoke<string[]>("get_scraper_media_types") : [];
  },

  async setMediaTypes(mediaTypes: string[]): Promise<void> {
    if (isTauri()) await tauriInvoke("set_scraper_media_types", { mediaTypes });
  },

  /** 智能 scrape - 自动匹配并聚合数据 */
  async autoScrape(name: string, fileName: string, system?: string): Promise<ScrapeResult> {
    if (isTauri()) {
      return tauriInvoke<ScrapeResult>("scraper_auto_scrape", { name, fileName, system });
    }
    throw new Error("Not available in web mode");
  },

  /** 启用/禁用 provider */
  async setProviderEnabled(providerId: string, enabled: boolean): Promise<void> {
    if (isTauri()) {
      await tauriInvoke("scraper_set_provider_enabled", { providerId, enabled });
    }
  },

  /** 设置 provider 优先级 */
  async setProviderPriority(providerId: string, priority: number): Promise<void> {
    if (isTauri()) {
      await tauriInvoke("scraper_set_provider_priority", { providerId, priority });
    }
  },

  async testProvider(providerId: string): Promise<string> {
    if (isTauri()) return tauriInvoke<string>("test_scraper_provider", { providerId });
    throw new Error("Not available in web mode");
  },

  /** 应用抓取到的数据 */
  async applyScrapedData(options: ApplyScrapedDataOptions): Promise<void> {
    if (isTauri()) {
      await tauriInvoke("apply_scraped_data", { options });
    }
  },

  /** 批量抓取 */
  async batchScrape(
    roms: { file_name: string; search_name: string; system?: string; directory?: string }[],
    system: string,
    directory: string,
    providerIds: string[],
    mediaTypes?: string[],
    forceRescrape = false,
  ): Promise<void> {
    if (isTauri()) {
      await tauriInvoke("batch_scrape", {
        roms,
        system,
        directory,
        providerIds,
        mediaTypes,
        forceRescrape,
      });
    }
  },

  async cancelBatchScrape(): Promise<boolean> {
    return isTauri() ? tauriInvoke<boolean>("cancel_batch_scrape") : false;
  },

  /** 导出临时数据到库 */
  async exportScrapedData(
    system: string,
    directory: string,
    format = "auto",
    targetDirectory?: string,
    nameMode?: "original" | "chinese",
  ): Promise<void> {
    if (isTauri()) {
      await tauriInvoke("export_scraped_data", { system, directory, format, targetDirectory, nameMode });
    }
  },

  /** 导出指定 Library 中所有已抓取平台 */
  async exportLibraryScrapedData(
    libraryId: string,
    format = "auto",
    targetDirectory?: string,
    nameMode?: "original" | "chinese",
  ): Promise<void> {
    if (isTauri()) {
      await tauriInvoke("export_library_scraped_data", { libraryId, format, targetDirectory, nameMode });
    }
  },

  /** 保存手动编辑的临时元数据 */
  async saveTempMetadata(system: string, directory: string, romId: string, metadata: ScraperGameMetadata): Promise<void> {
    if (isTauri()) {
      await tauriInvoke("save_temp_metadata", { system, directory, romId, metadata });
    }
  },

  /** 获取临时媒体列表 */
  async getTempMediaList(system: string, romId: string, romDirectory: string): Promise<{ asset_type: string, path: string }[]> {
    if (isTauri()) {
      return tauriInvoke("get_temp_media_list", { system, romId, romDirectory });
    }
    return [];
  },

  /** 删除临时媒体 */
  async deleteTempMedia(system: string, romId: string, assetType: string): Promise<void> {
    if (isTauri()) {
      await tauriInvoke("delete_temp_media", { system, romId, assetType });
    }
  },
};

export const aiTranslationApi = {
  async getConfig(): Promise<AiTranslationConfig> {
    if (isTauri()) return tauriInvoke<AiTranslationConfig>("get_ai_translation_config");
    return { endpoint: "https://api.openai.com/v1", model: "", target_language: "简体中文（zh-CN）", has_api_key: false, merge_batch_requests: false, batch_token_limit: 50000 };
  },

  async saveConfig(input: AiTranslationConfigInput): Promise<void> {
    if (isTauri()) await tauriInvoke("save_ai_translation_config", { input });
  },

  async translateMetadata(request: TranslateMetadataRequest): Promise<ScraperGameMetadata> {
    if (isTauri()) return tauriInvoke<ScraperGameMetadata>("translate_metadata", { request });
    throw new Error("Not available in web mode");
  },

  async translateMetadataBatch(requests: TranslateMetadataRequest[]): Promise<BatchTranslationResult[]> {
    if (isTauri()) return tauriInvoke<BatchTranslationResult[]>("translate_metadata_batch", { requests });
    throw new Error("Not available in web mode");
  },
};

// ============ Theme API ============

export const themeApi = {
  /** 列出已导入的自定义主题 */
  async listCustomThemes(): Promise<CustomThemeInfo[]> {
    if (isTauri()) {
      return tauriInvoke<CustomThemeInfo[]>("list_custom_themes");
    }
    throw new Error("主题包管理仅桌面版可用");
  },

  /** 导入 .rrtheme 主题包 */
  async importThemePack(path: string): Promise<CustomThemeInfo> {
    if (isTauri()) {
      return tauriInvoke<CustomThemeInfo>("import_theme_pack", { filePath: path });
    }
    throw new Error("主题包管理仅桌面版可用");
  },

  /** 删除已导入的自定义主题 */
  async deleteCustomTheme(id: string): Promise<void> {
    if (isTauri()) {
      await tauriInvoke("delete_custom_theme", { id });
      return;
    }
    throw new Error("主题包管理仅桌面版可用");
  },
};

// ============ PS3 API ============

export const ps3Api = {
  /** 为单个 PS3 ROM 生成 boxart */
  async generateBoxart(romFile: string, romDirectory: string, system: string): Promise<{ success: boolean; boxartPath: string; error?: string }> {
    if (isTauri()) {
      return tauriInvoke("generate_ps3_boxart", {
        request: {
          romFile,
          romDirectory,
          system
        }
      });
    }
    return { success: false, boxartPath: "", error: "Not supported in web mode" };
  },
};
