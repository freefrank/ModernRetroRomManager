import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Rom, GameSystem, ScanDirectory } from "@/types";

interface ScanProgress {
  current: number;
  total?: number;
  message: string;
  finished: boolean;
}

interface RomState {
  // ROM 列表
  roms: Rom[];
  fetchRoms: () => Promise<void>;
  
  // 游戏系统
  systems: GameSystem[];
  fetchSystems: () => Promise<void>;

  // 扫描目录
  scanDirectories: ScanDirectory[];
  fetchScanDirectories: () => Promise<void>;
  addScanDirectory: (path: string) => Promise<void>;
  removeScanDirectory: (id: string) => Promise<void>;

  // 扫描状态
  isScanning: boolean;
  scanProgress: ScanProgress | null;
  startScan: (dirId: string) => Promise<void>;

  // 统计信息
  stats: {
    totalRoms: number;
    scrapedRoms: number;
    totalSize: number;
  };
  fetchStats: () => Promise<void>;
}

export const useRomStore = create<RomState>((set, get) => ({
  // ROM 列表
  roms: [],
  fetchRoms: async () => {
    try {
      const roms = await invoke<Rom[]>("get_roms", {});
      set({ roms });
      get().fetchStats();
    } catch (error) {
      console.error("Failed to fetch roms:", error);
    }
  },

  // 游戏系统
  systems: [],
  fetchSystems: async () => {
    try {
      const systems = await invoke<GameSystem[]>("get_systems");
      set({ systems });
    } catch (error) {
      console.error("Failed to fetch systems:", error);
    }
  },

  // 扫描目录
  scanDirectories: [],
  fetchScanDirectories: async () => {
    try {
      const dirs = await invoke<ScanDirectory[]>("get_scan_directories");
      set({ scanDirectories: dirs });
    } catch (error) {
      console.error("Failed to fetch scan directories:", error);
    }
  },
  addScanDirectory: async (path: string) => {
    try {
      await invoke("add_scan_directory", { path, systemId: null });
      await get().fetchScanDirectories();
    } catch (error) {
      console.error("Failed to add scan directory:", error);
      throw error;
    }
  },
  removeScanDirectory: async (id: string) => {
    try {
      await invoke("remove_scan_directory", { id });
      await get().fetchScanDirectories();
    } catch (error) {
      console.error("Failed to remove scan directory:", error);
      throw error;
    }
  },

  // 扫描状态
  isScanning: false,
  scanProgress: null,
  startScan: async (dirId: string) => {
    set({ isScanning: true, scanProgress: null });
    try {
      // 监听进度事件
      const unlisten = await listen<ScanProgress>("scan-progress", (event) => {
        set({ scanProgress: event.payload });
        if (event.payload.finished) {
          set({ isScanning: false });
          get().fetchRoms(); // 扫描完成后刷新列表
          get().fetchScanDirectories(); // 更新最后扫描时间
          unlisten(); // 移除监听
        }
      });

      await invoke("start_scan", { dirId });
    } catch (error) {
      console.error("Failed to start scan:", error);
      set({ isScanning: false });
    }
  },

  // 统计信息
  stats: {
    totalRoms: 0,
    scrapedRoms: 0,
    totalSize: 0,
  },
  fetchStats: async () => {
    try {
      const stats = await invoke<{ total_roms: number; scraped_roms: number; total_size: number }>("get_rom_stats");
      set({
        stats: {
          totalRoms: stats.total_roms,
          scrapedRoms: stats.scraped_roms,
          totalSize: stats.total_size,
        },
      });
    } catch (error) {
      console.error("Failed to fetch stats:", error);
    }
  },
}));
