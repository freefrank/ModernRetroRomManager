import { create } from "zustand";
import type { ViewMode, SortOption, FilterOption } from "@/types";

export type ThemeMode = "light" | "dark";

interface AppState {
  // Theme
  theme: ThemeMode;
  setTheme: (theme: ThemeMode) => void;

  // UI 状态
  viewMode: ViewMode;
  setViewMode: (mode: ViewMode) => void;

  // 排序
  sortOption: SortOption;
  setSortOption: (option: SortOption) => void;

  // 筛选
  filterOption: FilterOption;
  setFilterOption: (option: FilterOption) => void;

  // 全局搜索
  searchQuery: string;
  setSearchQuery: (query: string) => void;

  // 加载状态
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;

  // 任务进度
  taskProgress: {
    current: number;
    total: number;
    message: string;
  } | null;
  setTaskProgress: (progress: { current: number; total: number; message: string } | null) => void;
}

export const useAppStore = create<AppState>((set) => ({
  // Theme
  theme: "dark", // Default to dark
  setTheme: (theme) => set({ theme }),

  // UI 状态
  viewMode: "grid",
  setViewMode: (mode) => set({ viewMode: mode }),

  // 排序
  sortOption: { field: "name", direction: "asc" },
  setSortOption: (option) => set({ sortOption: option }),

  // 筛选
  filterOption: {},
  setFilterOption: (option) => set({ filterOption: option }),

  // 全局搜索
  searchQuery: "",
  setSearchQuery: (query) => set({ searchQuery: query }),

  // 加载状态
  isLoading: false,
  setIsLoading: (loading) => set({ isLoading: loading }),

  // 任务进度
  taskProgress: null,
  setTaskProgress: (progress) => set({ taskProgress: progress }),
}));
