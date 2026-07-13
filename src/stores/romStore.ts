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
  cancelled?: boolean;
}

interface SystemInfo {
  name: string;
  romCount: number;
}

export type BatchScrapeScope = "selection" | "platform" | "library";

// 判定 ROM 是否已刮削:已有封面或描述(含待导出的 temp_data)即视为已刮削
function isRomScraped(rom: Rom): boolean {
  return Boolean(
    rom.box_front || rom.description || rom.temp_data?.box_front || rom.temp_data?.description
  );
}

// 从 systemRoms 聚合统计信息(totalSize 由后端扫描的 file_size 聚合,缺失按 0 计)
function computeStats(systemRoms: SystemRoms[]) {
  const allRoms = systemRoms.flatMap((s) => s.roms);
  return {
    totalRoms: allRoms.length,
    scrapedRoms: allRoms.filter(isRomScraped).length,
    totalSize: allRoms.reduce((sum, rom) => sum + (rom.file_size ?? 0), 0),
  };
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
  startBatchScrape: (providerIds: string[], mediaTypes?: string[], scope?: BatchScrapeScope) => Promise<void>;
  cancelBatchScrape: () => Promise<void>;
  
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
      // 直接从 systemRoms 计算 stats，避免额外的后端调用
      set({
        systemRoms,
        availableSystems,
        roms,
        isLoadingRoms: false,
        stats: computeStats(systemRoms),
      });
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
  startBatchScrape: async (providerIds: string[], mediaTypes?: string[], scope = "selection") => {
    const { selectedRomIds, selectedSystem, systemRoms } = get();

    if (!isTauri()) {
      console.warn("Batch scrape not supported in web mode");
      return;
    }

    const selectedSystemInfo = systemRoms.find(s => s.system === selectedSystem);
    const targetSystems = scope === "library"
      ? systemRoms
      : selectedSystemInfo ? [selectedSystemInfo] : [];
    const targetRoms = targetSystems.flatMap(systemInfo =>
      systemInfo.roms
        .filter(rom => scope !== "selection" || selectedRomIds.has(rom.file))
        .map(rom => ({
          file_name: rom.file,
          search_name: rom.english_name?.trim() || rom.name || rom.file,
          system: systemInfo.system,
          directory: systemInfo.path || "",
        })),
    );

    if (targetRoms.length === 0) return;

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

      await scraperApi.batchScrape(targetRoms, "", "", providerIds, mediaTypes);
    } catch (error) {
      console.error("Failed to start batch scrape:", error);
      set({ isBatchScraping: false });
    }
  },
  cancelBatchScrape: async () => {
    await scraperApi.cancelBatchScrape();
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
      // 先添加目录到配置
      await api.addDirectory(path, metadataFormat, false, null);
      await get().fetchScanDirectories();
      
      // 只扫描新添加的目录，而不是全部目录
      const newSystems = await api.getRomsForDirectory(path, metadataFormat, false, null);
      
      // 合并到现有的 systemRoms
      const { systemRoms, selectedSystem } = get();
      const updatedSystemRoms = [...systemRoms];
      
      for (const newSystem of newSystems) {
        const existingIndex = updatedSystemRoms.findIndex(s => s.system === newSystem.system);
        if (existingIndex >= 0) {
          // 系统已存在，合并 ROMs（避免重复）
          const existingFiles = new Set(updatedSystemRoms[existingIndex].roms.map(r => r.file));
          const uniqueRoms = newSystem.roms.filter(r => !existingFiles.has(r.file));
          updatedSystemRoms[existingIndex].roms.push(...uniqueRoms);
        } else {
          // 新系统
          updatedSystemRoms.push(newSystem);
        }
      }
      
      const availableSystems = updatedSystemRoms.map(s => ({
        name: s.system,
        romCount: s.roms.length,
      }));
      
      let roms: Rom[];
      if (selectedSystem) {
        const filtered = updatedSystemRoms.find(s => s.system === selectedSystem);
        roms = filtered ? filtered.roms : [];
      } else {
        roms = updatedSystemRoms.flatMap(s => s.roms);
      }
      
      set({
        systemRoms: updatedSystemRoms,
        availableSystems,
        roms,
        stats: computeStats(updatedSystemRoms),
      });
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
      // 后端 stats 仅含总数;scrapedRoms 从已加载的 systemRoms 计算,避免被清零
      set({
        stats: {
          ...computeStats(get().systemRoms),
          totalRoms: stats.total_roms,
        },
      });
    } catch (error) {
      console.error("Failed to fetch stats:", error);
    }
  },
}));
