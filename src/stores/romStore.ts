import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Rom, GameSystem, ScanDirectory, FilterOption } from "@/types";

interface ScanProgress {
  current: number;
  total?: number;
  message: string;
  finished: boolean;
}

interface BatchProgress {
  current: number;
  total: number;
  message: string;
  finished: boolean;
}

interface RomState {
  // ROM 列表
  roms: Rom[];
  fetchRoms: (filter?: FilterOption) => Promise<void>;
  
  // 选中的 ROM
  selectedRomIds: Set<string>;
  toggleRomSelection: (id: string, multiSelect?: boolean) => void;
  selectAllRoms: () => void;
  clearSelection: () => void;

  // 批量 Scrape
  isBatchScraping: boolean;
  batchProgress: BatchProgress | null;
  startBatchScrape: (provider: string) => Promise<void>;
  
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
  fetchRoms: async (filter?: FilterOption) => {
    try {
      const roms = await invoke<Rom[]>("get_roms", { filter });
      set({ roms });
      get().fetchStats();
    } catch (error) {
      console.error("Failed to fetch roms:", error);
    }
  },

  // 选中的 ROM
  selectedRomIds: new Set(),
  toggleRomSelection: (id: string, multiSelect = false) => {
    set((state) => {
      if (multiSelect) {
        const newSet = new Set(state.selectedRomIds);
        if (newSet.has(id)) newSet.delete(id);
        else newSet.add(id);
        return { selectedRomIds: newSet };
      } else {
        return { selectedRomIds: new Set([id]) };
      }
    });
  },
  selectAllRoms: () => {
    set((state) => ({ selectedRomIds: new Set(state.roms.map(r => r.id)) }));
  },
  clearSelection: () => set({ selectedRomIds: new Set() }),

  // 批量 Scrape
  isBatchScraping: false,
  batchProgress: null,
  startBatchScrape: async (provider: string) => {
    const { selectedRomIds } = get();
    if (selectedRomIds.size === 0) return;

    set({ isBatchScraping: true, batchProgress: null });
    try {
      const unlisten = await listen<BatchProgress>("batch-scrape-progress", (event) => {
        set({ batchProgress: event.payload });
        if (event.payload.finished) {
          set({ isBatchScraping: false });
          get().fetchRoms();
          unlisten();
        }
      });

      await invoke("batch_scrape", { romIds: Array.from(selectedRomIds), provider });
    } catch (error) {
      console.error("Failed to start batch scrape:", error);
      set({ isBatchScraping: false });
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
      // 添加目录并获取新目录的信息
      const newDir = await invoke<ScanDirectory>("add_scan_directory", { path, systemId: null });
      await get().fetchScanDirectories();
      // 自动开始扫描新添加的目录
      await get().startScan(newDir.id);
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
