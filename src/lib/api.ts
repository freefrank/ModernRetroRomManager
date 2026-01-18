import type { SystemRoms, GameSystem, ScanDirectory, Rom, ScraperProviderInfo, ScraperCredentials, ScraperSearchResult, ScraperGameMetadata, ScraperMediaAsset, ScrapeResult } from "@/types";

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

export function getCachedMediaUrl(path: string | undefined): string | null {
  if (!path) return null;
  return mediaUrlCache.get(path) ?? null;
}

export async function preloadMediaUrls(roms: Rom[], limit = 50): Promise<void> {
  const paths = roms
    .slice(0, limit)
    .map(rom => rom.box_front || rom.gridicon)
    .filter((path): path is string => !!path && !mediaUrlCache.has(path));

  await Promise.all(
    paths.map(async (path) => {
      const url = await resolveMediaUrlAsync(path);
      if (url) {
        mediaUrlCache.set(path, url);
      }
    })
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

  async addDirectory(path: string, metadataFormat: string, isRoot: boolean, systemId: string | null): Promise<void> {
    if (isTauri()) {
      await tauriInvoke("add_directory", { path, metadataFormat, isRoot, systemId });
    }
  },

  async removeDirectory(path: string): Promise<void> {
    if (isTauri()) {
      await tauriInvoke("remove_directory", { path });
    }
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
};

export function resolveMediaUrl(path: string | undefined): string | null {
  if (!path) return null;
  if (path.startsWith("http") || path.startsWith("data:")) return path;

  if (isTauri()) {
    return import("@tauri-apps/api/core").then(({ convertFileSrc }) => convertFileSrc(path)) as unknown as string;
  }

  return `${API_BASE}/media?path=${encodeURIComponent(path)}`;
}

// Normalize path separators for Windows compatibility
function normalizePath(path: string): string {
  // Convert forward slashes to backslashes on Windows paths
  if (path.match(/^[A-Za-z]:/)) {
    return path.replace(/\//g, '\\');
  }
  return path;
}

export async function resolveMediaUrlAsync(path: string | undefined): Promise<string | null> {
  if (!path) return null;
  if (path.startsWith("http") || path.startsWith("data:")) return path;

  const normalizedPath = normalizePath(path);

  if (isTauri()) {
    const { convertFileSrc } = await import("@tauri-apps/api/core");
    return convertFileSrc(normalizedPath);
  }

  return `${API_BASE}/media?path=${encodeURIComponent(normalizedPath)}`;
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
  async search(name: string, fileName: string, system?: string): Promise<ScraperSearchResult[]> {
    if (isTauri()) {
      return tauriInvoke<ScraperSearchResult[]>("scraper_search", { name, fileName, system });
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
  async getMedia(providerId: string, sourceId: string): Promise<ScraperMediaAsset[]> {
    if (isTauri()) {
      return tauriInvoke<ScraperMediaAsset[]>("scraper_get_media", { providerId, sourceId });
    }
    return [];
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
};
