import type { SystemRoms, GameSystem, ScanDirectory } from "@/types";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export const isTauri = (): boolean => {
  return typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;
};

const API_BASE = import.meta.env.VITE_API_URL || "/api";

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
