import { create } from "zustand";
import { api, scraperApi, isTauri } from "@/lib/api";
import type { Rom, GameSystem, ScanDirectory, FilterOption, SystemRoms, ScraperGameMetadata, RomSystemSummary } from "@/types";
import { shouldIncludeInBatchScrape } from "@/lib/romScrapeStatus";

interface ScanProgress {
  current: number;
  total: number;
  system?: string;
  mode: "incremental" | "full";
  message: string;
  finished: boolean;
  changed: boolean;
  libraryId?: string;
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
  path: string;
  scrapedCount: number;
  totalSize: number;
}

export type BatchScrapeScope = "selection" | "platform" | "library";

let scanProgressListenerInstalled = false;
let systemRequestSequence = 0;
let libraryRequestSequence = 0;

function computeSummaryStats(summaries: RomSystemSummary[]) {
  return {
    totalRoms: summaries.reduce((sum, entry) => sum + entry.romCount, 0),
    scrapedRoms: summaries.reduce((sum, entry) => sum + entry.scrapedCount, 0),
    totalSize: summaries.reduce((sum, entry) => sum + entry.totalSize, 0),
  };
}

function mapSummaries(summaries: RomSystemSummary[]): SystemInfo[] {
  return summaries.map(entry => ({
    name: entry.system,
    path: entry.path,
    romCount: entry.romCount,
    scrapedCount: entry.scrapedCount,
    totalSize: entry.totalSize,
  }));
}

interface RomState {
  // ROM 列表
  roms: Rom[];
  systemRoms: SystemRoms[];
  availableSystems: SystemInfo[];
  selectedSystem: string | null;
  setSelectedSystem: (system: string | null) => void;
  fetchRoms: (filter?: FilterOption) => Promise<void>;
  fetchLibrarySummary: () => Promise<void>;
  fetchSystemRoms: (system: string) => Promise<void>;
  scanLibrary: (full?: boolean) => Promise<void>;
  initializeScanProgress: () => Promise<void>;
  isLoadingRoms: boolean;
  
  // 选中的 ROM
  selectedRomIds: Set<string>;
  toggleRomSelection: (id: string, multiSelect?: boolean) => void;
  selectAllRoms: (ids?: string[]) => void;
  clearSelection: () => void;

  // 批量 Scrape
  isBatchScraping: boolean;
  batchProgress: BatchProgress | null;
  startBatchScrape: (
    providerIds: string[],
    mediaTypes?: string[],
    scope?: BatchScrapeScope,
    forceRescrape?: boolean,
  ) => Promise<void>;
  cancelBatchScrape: () => Promise<void>;
  
  // 游戏系统
  systems: GameSystem[];
  fetchSystems: () => Promise<void>;

  // 扫描目录
  scanDirectories: ScanDirectory[];
  fetchScanDirectories: () => Promise<void>;
  addScanDirectory: (
    path: string,
    metadataFormat?: string,
    isRoot?: boolean,
    systemId?: string | null,
  ) => Promise<void>;
  removeScanDirectory: (id: string) => Promise<void>;
  activateLibrary: (id: string, fullScan?: boolean) => Promise<void>;
  renameLibrary: (id: string, name: string) => Promise<void>;

  // 扫描状态
  isScanning: boolean;
  scanProgress: ScanProgress | null;

  // 统计信息
  stats: {
    totalRoms: number;
    scrapedRoms: number;
    totalSize: number;
  };
  updateTempMetadata: (system: string, directory: string, rom_id: string, metadata: ScraperGameMetadata) => Promise<void>;
  deleteTempMedia: (system: string, rom_id: string, assetType: string) => Promise<void>;
  // 导出状态
  isExporting: boolean;
  exportProgress: { current: number; total: number; message: string; finished: boolean } | null;
  
  exportData: (system: string, directory: string, format?: string, targetDirectory?: string, nameMode?: "original" | "chinese") => Promise<void>;
  exportLibraryData: (libraryId: string, format?: string, targetDirectory?: string, nameMode?: "original" | "chinese") => Promise<void>;
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
      set({ roms: [] });
    } else {
      const filtered = systemRoms.find(s => s.system === system);
      set({ roms: filtered ? filtered.roms : [] });
    }
  },
  // 导出状态
  isExporting: false,
  exportProgress: null,

  fetchRoms: async (_filter?: FilterOption) => {
    const selected = get().selectedSystem;
    if (selected) {
      await Promise.all([get().fetchLibrarySummary(), get().fetchSystemRoms(selected)]);
    } else {
      await get().fetchLibrarySummary();
    }
  },

  fetchLibrarySummary: async () => {
    set({ isLoadingRoms: true });
    try {
      const summaries = await api.getRomLibrarySummary();
      set({
        availableSystems: mapSummaries(summaries),
        isLoadingRoms: false,
        stats: computeSummaryStats(summaries),
      });
    } catch (error) {
      console.error("Failed to fetch ROM library summary:", error);
      set({ isLoadingRoms: false });
    }
  },

  fetchSystemRoms: async (system: string) => {
    const requestId = ++systemRequestSequence;
    set({ isLoadingRoms: true });
    try {
      const loaded = await api.getSystemRoms(system);
      set((state) => ({
        systemRoms: [
          ...state.systemRoms.filter(entry => entry.system !== loaded.system),
          loaded,
        ],
        roms: state.selectedSystem === system ? loaded.roms : state.roms,
        isLoadingRoms: requestId === systemRequestSequence ? false : state.isLoadingRoms,
      }));
    } catch (error) {
      console.error(`Failed to fetch ROM system ${system}:`, error);
      if (requestId === systemRequestSequence) set({ isLoadingRoms: false });
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
  selectAllRoms: (ids?: string[]) => {
    // 暂时用文件路径作为 ID
    set((state) => ({ selectedRomIds: new Set(ids || state.roms.map(r => r.file)) }));
  },
  clearSelection: () => set({ selectedRomIds: new Set() }),

  // 批量 Scrape
  isBatchScraping: false,
  batchProgress: null,
  startBatchScrape: async (
    providerIds: string[],
    mediaTypes?: string[],
    scope = "selection",
    forceRescrape = false,
  ) => {
    const { selectedRomIds, selectedSystem } = get();

    if (!isTauri()) {
      console.warn("Batch scrape not supported in web mode");
      return;
    }

    const sourceSystems = scope === "library" ? await api.getRoms() : get().systemRoms;
    const selectedSystemInfo = sourceSystems.find(s => s.system === selectedSystem);
    const targetSystems = scope === "library"
      ? sourceSystems
      : selectedSystemInfo ? [selectedSystemInfo] : [];
    const targetRoms = targetSystems.flatMap(systemInfo =>
      systemInfo.roms
        .filter(rom => scope !== "selection" || selectedRomIds.has(rom.file))
        .filter(rom => shouldIncludeInBatchScrape(rom, forceRescrape))
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

      await scraperApi.batchScrape(
        targetRoms,
        "",
        "",
        providerIds,
        mediaTypes,
        forceRescrape,
      );
    } catch (error) {
      console.error("Failed to start batch scrape:", error);
      set({ isBatchScraping: false });
    }
  },

  initializeScanProgress: async () => {
    if (!isTauri() || scanProgressListenerInstalled) return;
    scanProgressListenerInstalled = true;
    const { listen } = await import("@tauri-apps/api/event");
    await listen<ScanProgress>("rom-scan-progress", ({ payload }) => {
      const activeLibraryId = get().scanDirectories.find(item => item.isActive)?.id;
      if (payload.libraryId && payload.libraryId !== activeLibraryId) return;
      set({
        isScanning: !payload.finished,
        scanProgress: payload,
      });
      if (payload.finished) {
        window.setTimeout(() => {
          if (get().scanProgress?.finished) set({ scanProgress: null });
        }, 2500);
      }
    });
  },

  scanLibrary: async (full = false) => {
    await get().initializeScanProgress();
    const libraryId = get().scanDirectories.find(item => item.isActive)?.id;
    set({ isScanning: true });
    try {
      const summaries = await api.scanRomLibrary(full, libraryId);
      const currentLibraryId = get().scanDirectories.find(item => item.isActive)?.id;
      if (libraryId && currentLibraryId !== libraryId) return;
      const { selectedSystem } = get();
      const selectedStillExists = selectedSystem
        ? summaries.some((entry) => entry.system === selectedSystem)
        : false;
      set({
        availableSystems: mapSummaries(summaries),
        stats: computeSummaryStats(summaries),
        systemRoms: [],
        roms: selectedStillExists ? get().roms : [],
        selectedSystem: selectedStillExists ? selectedSystem : null,
        isScanning: false,
        isLoadingRoms: false,
      });
      if (selectedStillExists && selectedSystem) await get().fetchSystemRoms(selectedSystem);
    } catch (error) {
      console.error("Failed to scan ROM library:", error);
      set({ isScanning: false, isLoadingRoms: false });
      throw error;
    }
  },
  cancelBatchScrape: async () => {
    await scraperApi.cancelBatchScrape();
  },

  exportData: async (system: string, directory: string, format = "auto", targetDirectory?: string, nameMode = "original") => {
    set({ isExporting: true, exportProgress: null });
    let unlisten: (() => void) | undefined;
    try {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<{ current: number; total: number; message: string; finished: boolean }>("export-progress", (event) => {
        set({ exportProgress: event.payload });
        if (event.payload.finished) {
          setTimeout(() => {
            set({ isExporting: false, exportProgress: null });
            get().fetchRoms();
          }, 1500);
          unlisten?.();
        }
      });

      await scraperApi.exportScrapedData(system, directory, format, targetDirectory, nameMode);
    } catch (error) {
      unlisten?.();
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
  addScanDirectory: async (
    path: string,
    metadataFormat = "none",
    isRoot = false,
    systemId = null,
  ) => {
    try {
      // 新目录由后端自动设为激活 Library；先清空旧库状态再开始首次全量扫描。
      await api.addDirectory(path, metadataFormat, isRoot, systemId);
      systemRequestSequence++;
      set({
        roms: [],
        systemRoms: [],
        availableSystems: [],
        selectedSystem: null,
        selectedRomIds: new Set(),
        stats: { totalRoms: 0, scrapedRoms: 0, totalSize: 0 },
        isLoadingRoms: true,
      });
      await get().fetchScanDirectories();

      // 新 ROM 库第一次加入时建立完整持久索引。
      await get().scanLibrary(true);
    } catch (error) {
      console.error("Failed to add directory:", error);
      throw error;
    }
  },
  removeScanDirectory: async (id: string) => {
    try {
      const wasActive = get().scanDirectories.some(item => item.id === id && item.isActive);
      await api.removeDirectory(id);
      await get().fetchScanDirectories();
      if (wasActive) {
        systemRequestSequence++;
        set({
          roms: [],
          systemRoms: [],
          availableSystems: [],
          selectedSystem: null,
          selectedRomIds: new Set(),
          stats: { totalRoms: 0, scrapedRoms: 0, totalSize: 0 },
          isLoadingRoms: true,
        });
        await get().scanLibrary(false);
      }
    } catch (error) {
      console.error("Failed to remove directory:", error);
      throw error;
    }
  },

  exportLibraryData: async (libraryId: string, format = "auto", targetDirectory?: string, nameMode = "original") => {
    set({ isExporting: true, exportProgress: null });
    let unlisten: (() => void) | undefined;
    try {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<{ current: number; total: number; message: string; finished: boolean }>("export-progress", (event) => {
        set({ exportProgress: event.payload });
        if (event.payload.finished) {
          setTimeout(() => {
            set({ isExporting: false, exportProgress: null });
            get().fetchRoms();
          }, 1500);
          unlisten?.();
        }
      });

      await scraperApi.exportLibraryScrapedData(libraryId, format, targetDirectory, nameMode);
    } catch (error) {
      unlisten?.();
      console.error("Failed to export library data:", error);
      set({ isExporting: false });
      throw error;
    }
  },

  activateLibrary: async (id: string, fullScan = false) => {
    const current = get().scanDirectories.find(item => item.isActive);
    if (current?.id === id && !fullScan) return;
    const requestId = ++libraryRequestSequence;
    try {
      if (current?.id !== id) await api.setActiveLibrary(id);
      systemRequestSequence++;
      set({
        roms: [],
        systemRoms: [],
        availableSystems: [],
        selectedSystem: null,
        selectedRomIds: new Set(),
        stats: { totalRoms: 0, scrapedRoms: 0, totalSize: 0 },
        isLoadingRoms: true,
      });
      await get().fetchScanDirectories();

      // 先从本地索引恢复平台列表，不等待慢速盘完成校验。
      const cached = await api.getLibraryRomSummary(id);
      if (requestId !== libraryRequestSequence) return;
      set({
        availableSystems: mapSummaries(cached),
        stats: computeSummaryStats(cached),
        isLoadingRoms: false,
      });

      if (fullScan) {
        await get().scanLibrary(true);
        return;
      }

      // 增量扫描在后台刷新；只允许当前切库请求把结果写回界面。
      await get().initializeScanProgress();
      set({ isScanning: true });
      void api.scanRomLibrary(false, id).then(async (summaries) => {
        if (requestId !== libraryRequestSequence) return;
        set({
          availableSystems: mapSummaries(summaries),
          stats: computeSummaryStats(summaries),
          isScanning: false,
          isLoadingRoms: false,
        });
      }).catch((error) => {
        console.error("Failed to refresh activated library:", error);
        if (requestId === libraryRequestSequence) {
          set({ isScanning: false, isLoadingRoms: false });
        }
      });
    } catch (error) {
      console.error("Failed to activate library:", error);
      throw error;
    }
  },

  renameLibrary: async (id: string, name: string) => {
    try {
      await api.renameLibrary(id, name);
      await get().fetchScanDirectories();
    } catch (error) {
      console.error("Failed to rename library:", error);
      throw error;
    }
  },

  // 扫描状态
  isScanning: false,
  scanProgress: null,

  // 统计信息
  stats: {
    totalRoms: 0,
    scrapedRoms: 0,
    totalSize: 0,
  },
}));
