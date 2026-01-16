import { create } from "zustand";
import type { Rom, GameSystem, ScanDirectory } from "@/types";

interface RomState {
  // ROM 列表
  roms: Rom[];
  setRoms: (roms: Rom[]) => void;
  addRom: (rom: Rom) => void;
  updateRom: (id: string, data: Partial<Rom>) => void;
  removeRom: (id: string) => void;

  // 当前选中的 ROM
  selectedRomIds: string[];
  setSelectedRomIds: (ids: string[]) => void;
  toggleRomSelection: (id: string) => void;
  clearSelection: () => void;

  // 游戏系统
  systems: GameSystem[];
  setSystems: (systems: GameSystem[]) => void;

  // 扫描目录
  scanDirectories: ScanDirectory[];
  setScanDirectories: (dirs: ScanDirectory[]) => void;
  addScanDirectory: (dir: ScanDirectory) => void;
  removeScanDirectory: (id: string) => void;

  // 统计信息
  stats: {
    totalRoms: number;
    scrapedRoms: number;
    totalSize: number;
  };
  updateStats: () => void;
}

export const useRomStore = create<RomState>((set, get) => ({
  // ROM 列表
  roms: [],
  setRoms: (roms) => {
    set({ roms });
    get().updateStats();
  },
  addRom: (rom) => {
    set((state) => ({ roms: [...state.roms, rom] }));
    get().updateStats();
  },
  updateRom: (id, data) => {
    set((state) => ({
      roms: state.roms.map((rom) => (rom.id === id ? { ...rom, ...data } : rom)),
    }));
  },
  removeRom: (id) => {
    set((state) => ({
      roms: state.roms.filter((rom) => rom.id !== id),
      selectedRomIds: state.selectedRomIds.filter((romId) => romId !== id),
    }));
    get().updateStats();
  },

  // 当前选中的 ROM
  selectedRomIds: [],
  setSelectedRomIds: (ids) => set({ selectedRomIds: ids }),
  toggleRomSelection: (id) => {
    set((state) => ({
      selectedRomIds: state.selectedRomIds.includes(id)
        ? state.selectedRomIds.filter((romId) => romId !== id)
        : [...state.selectedRomIds, id],
    }));
  },
  clearSelection: () => set({ selectedRomIds: [] }),

  // 游戏系统
  systems: [],
  setSystems: (systems) => set({ systems }),

  // 扫描目录
  scanDirectories: [],
  setScanDirectories: (dirs) => set({ scanDirectories: dirs }),
  addScanDirectory: (dir) => {
    set((state) => ({ scanDirectories: [...state.scanDirectories, dir] }));
  },
  removeScanDirectory: (id) => {
    set((state) => ({
      scanDirectories: state.scanDirectories.filter((dir) => dir.id !== id),
    }));
  },

  // 统计信息
  stats: {
    totalRoms: 0,
    scrapedRoms: 0,
    totalSize: 0,
  },
  updateStats: () => {
    const { roms } = get();
    set({
      stats: {
        totalRoms: roms.length,
        scrapedRoms: roms.filter((rom) => rom.metadata).length,
        totalSize: roms.reduce((sum, rom) => sum + rom.size, 0),
      },
    });
  },
}));
