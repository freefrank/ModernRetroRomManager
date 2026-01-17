import { create } from "zustand";
import { api, isTauri } from "@/lib/api";
import type { Rom, GameSystem, ScanDirectory, FilterOption, SystemRoms } from "@/types";

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

interface SystemInfo {
  name: string;
  romCount: number;
}

interface RomState {
  // ROM 列表
  roms: Rom[];
  systemRoms: SystemRoms[];
  availableSystems: SystemInfo[];
  selectedSystem: string | null;
  setSelectedSystem: (system: string | null) => void;
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
  addScanDirectory: (path: string, metadataFormat?: string) => Promise<void>;
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
  roms: [],
  systemRoms: [],
  availableSystems: [],
  selectedSystem: null,
  setSelectedSystem: (system: string | null) => {
    set({ selectedSystem: system });
    const { systemRoms } = get();
    if (system === null) {
      set({ roms: systemRoms.flatMap(s => s.roms) });
    } else {
      const filtered = systemRoms.find(s => s.system === system);
      set({ roms: filtered ? filtered.roms : [] });
    }
  },
  fetchRoms: async (_filter?: FilterOption) => {
    try {
      const systemRoms = await api.getRoms();
      const availableSystems = systemRoms.map(s => ({
        name: s.system,
        romCount: s.roms.length,
      }));
      const { selectedSystem } = get();
      let roms: Rom[];
      if (selectedSystem) {
        const filtered = systemRoms.find(s => s.system === selectedSystem);
        roms = filtered ? filtered.roms : [];
      } else {
        roms = systemRoms.flatMap(s => s.roms);
      }
      set({ systemRoms, availableSystems, roms });
      get().fetchStats();
    } catch (error) {
      console.error("Failed to fetch roms:", error);
    }
  },

  // 选中的 ROM
  selectedRomIds: new Set(),
  toggleRomSelection: (id: string, multiSelect = false) => {
    set((state) => {
      // 暂时用文件路径作为 ID
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
    // 暂时用文件路径作为 ID
    set((state) => ({ selectedRomIds: new Set(state.roms.map(r => r.file)) }));
  },
  clearSelection: () => set({ selectedRomIds: new Set() }),

  // 批量 Scrape
  isBatchScraping: false,
  batchProgress: null,
  startBatchScrape: async (provider: string) => {
    const { selectedRomIds } = get();
    if (selectedRomIds.size === 0) return;

    if (!isTauri()) {
      console.warn("Batch scrape not supported in web mode");
      return;
    }

    set({ isBatchScraping: true, batchProgress: null });
    try {
      const { listen } = await import("@tauri-apps/api/event");
      const { invoke } = await import("@tauri-apps/api/core");
      
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

  // 游戏系统 - 暂时保留，后续可能需要完全移除，直接从 SystemRoms 获取系统列表
  systems: [],
  fetchSystems: async () => {
    try {
      const systems = await api.getSystems();
      set({ systems });
    } catch (error) {
      console.error("Failed to fetch systems:", error);
    }
  },

  // 目录列表
  scanDirectories: [],
  fetchScanDirectories: async () => {
    try {
      const dirs = await api.getDirectories();
      set({ scanDirectories: dirs });
    } catch (error) {
      console.error("Failed to fetch directories:", error);
    }
  },
  addScanDirectory: async (path: string, metadataFormat="none") => {
    try {
      await api.addDirectory(path, metadataFormat, false, null);
      await get().fetchScanDirectories();
      await get().fetchRoms();
    } catch (error) {
      console.error("Failed to add directory:", error);
      throw error;
    }
  },
  removeScanDirectory: async (path: string) => {
    try {
      await api.removeDirectory(path);
      await get().fetchScanDirectories();
      await get().fetchRoms();
    } catch (error) {
      console.error("Failed to remove directory:", error);
      throw error;
    }
  },

  // 扫描状态 - 本地无数据库，扫描其实很快，可能不再需要复杂的进度状态
  isScanning: false,
  scanProgress: null,
  startScan: async () => Promise.resolve(),

  // 统计信息
  stats: {
    totalRoms: 0,
    scrapedRoms: 0,
    totalSize: 0,
  },
  fetchStats: async () => {
    try {
      const stats = await api.getStats();
      set({
        stats: {
          totalRoms: stats.total_roms,
          scrapedRoms: 0,
          totalSize: 0,
        },
      });
    } catch (error) {
      console.error("Failed to fetch stats:", error);
    }
  },
}));
