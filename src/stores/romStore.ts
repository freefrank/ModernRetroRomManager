import { create } from "zustand";
import { api, scraperApi, isTauri } from "@/lib/api";
import type { Rom, GameSystem, ScanDirectory, FilterOption, SystemRoms, ScraperGameMetadata } from "@/types";

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
  isLoadingRoms: boolean;
  
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
  updateTempMetadata: (system: string, directory: string, rom_id: string, metadata: ScraperGameMetadata) => Promise<void>;
  deleteTempMedia: (system: string, rom_id: string, assetType: string) => Promise<void>;
  // 导出状态
  isExporting: boolean;
  exportProgress: { current: number; total: number; message: string; finished: boolean } | null;
  
  exportData: (system: string, directory: string) => Promise<void>;
}

export const useRomStore = create<RomState>((set, get) => ({
  roms: [],
  systemRoms: [],
  availableSystems: [],
  selectedSystem: null,
  isLoadingRoms: false,
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
  // 导出状态
  isExporting: false,
  exportProgress: null,

  fetchRoms: async (_filter?: FilterOption) => {
    set({ isLoadingRoms: true });
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
      set({ systemRoms, availableSystems, roms, isLoadingRoms: false });
      get().fetchStats();
    } catch (error) {
      console.error("Failed to fetch roms:", error);
      set({ isLoadingRoms: false });
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
  startBatchScrape: async (providerId: string) => {
    const { selectedRomIds, selectedSystem, systemRoms } = get();
    if (selectedRomIds.size === 0) return;

    if (!isTauri()) {
      console.warn("Batch scrape not supported in web mode");
      return;
    }

    // 获取当前系统的目录信息
    const systemInfo = systemRoms.find(s => s.system === selectedSystem);
    const directory = systemInfo?.path || "";
    const system = selectedSystem || "";

    set({ isBatchScraping: true, batchProgress: null });
    try {
      const { listen } = await import("@tauri-apps/api/event");
      
      const unlisten = await listen<BatchProgress>("batch-scrape-progress", (event) => {
        set({ batchProgress: event.payload });
        if (event.payload.finished) {
          setTimeout(() => {
            set({ isBatchScraping: false });
            get().fetchRoms();
          }, 1000);
          unlisten();
        }
      });

      await scraperApi.batchScrape(Array.from(selectedRomIds), system, directory, providerId);
    } catch (error) {
      console.error("Failed to start batch scrape:", error);
      set({ isBatchScraping: false });
    }
  },

  exportData: async (system: string, directory: string) => {
    set({ isExporting: true, exportProgress: null });
    try {
      const { listen } = await import("@tauri-apps/api/event");
      const unlisten = await listen<{ current: number; total: number; message: string; finished: boolean }>("export-progress", (event) => {
        set({ exportProgress: event.payload });
        if (event.payload.finished) {
          setTimeout(() => {
            set({ isExporting: false, exportProgress: null });
            get().fetchRoms();
          }, 1500);
          unlisten();
        }
      });

      await scraperApi.exportScrapedData(system, directory);
    } catch (error) {
      console.error("Failed to export data:", error);
      set({ isExporting: false });
      throw error;
    }
  },

  updateTempMetadata: async (system: string, directory: string, rom_id: string, metadata: ScraperGameMetadata) => {
    try {
      await scraperApi.saveTempMetadata(system, directory, rom_id, metadata);
      await get().fetchRoms(); // 刷新以获取最新 temp_data
    } catch (error) {
      console.error("Failed to update temp metadata:", error);
      throw error;
    }
  },

  deleteTempMedia: async (system: string, rom_id: string, assetType: string) => {
    try {
      await scraperApi.deleteTempMedia(system, rom_id, assetType);
      await get().fetchRoms();
    } catch (error) {
      console.error("Failed to delete temp media:", error);
      throw error;
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
