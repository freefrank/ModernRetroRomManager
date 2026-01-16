import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { ViewMode, SortOption, FilterOption } from "@/types";

export type ThemeMode = "light" | "dark" | "cyberpunk" | "ocean" | "forest" | "sunset" | "rose" | "nord";

// 所有可用主题
export const THEMES: { id: ThemeMode; name: string; icon: string }[] = [
  { id: "light", name: "Light", icon: "☀️" },
  { id: "dark", name: "Dark", icon: "🌙" },
  { id: "cyberpunk", name: "Cyberpunk", icon: "🌆" },
  { id: "ocean", name: "Ocean", icon: "🌊" },
  { id: "forest", name: "Forest", icon: "🌲" },
  { id: "sunset", name: "Sunset", icon: "🌅" },
  { id: "rose", name: "Rose", icon: "🌹" },
  { id: "nord", name: "Nord", icon: "❄️" },
];

// 同步主题到 DOM
const applyThemeToDOM = (theme: ThemeMode) => {
  const root = document.documentElement;
  // 移除所有主题类
  THEMES.forEach(t => root.classList.remove(t.id));
  // 添加当前主题类（light 是默认的 :root，不需要类）
  if (theme !== "light") {
    root.classList.add(theme);
  }
};

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

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      // Theme - 默认暗色
      theme: "dark",
      setTheme: (theme) => {
        applyThemeToDOM(theme);
        set({ theme });
      },

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
    }),
    {
      name: "app-settings",
      partialize: (state) => ({ theme: state.theme, viewMode: state.viewMode }),
      onRehydrateStorage: () => (state) => {
        // 从存储中恢复后立即应用主题
        if (state) {
          applyThemeToDOM(state.theme);
        }
      },
    }
  )
);

// 初始化时立即应用主题（避免闪烁）
const initTheme = () => {
  const stored = localStorage.getItem("app-settings");
  if (stored) {
    try {
      const { state } = JSON.parse(stored);
      if (state?.theme) {
        applyThemeToDOM(state.theme);
      }
    } catch {
      applyThemeToDOM("dark");
    }
  } else {
    applyThemeToDOM("dark");
  }
};
initTheme();

