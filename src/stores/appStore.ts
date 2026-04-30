import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
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
export const applyThemeToDOM = (theme: ThemeMode) => {
  const root = document.documentElement;
  // 移除所有主题类
  THEMES.forEach(t => root.classList.remove(t.id));
  // 添加当前主题类（light 是默认的 :root，不需要类）
  if (theme !== "light") {
    root.classList.add(theme);
  }
};

// 保存设置到后端
const saveSettingToBackend = async (key: string, value: string) => {
  try {
    await invoke("update_app_setting", { key, value });
  } catch (error) {
    console.error("Failed to save setting to backend:", error);
  }
};

interface AppSettings {
  theme: string;
  language: string;
  view_mode: string;
}

interface AppState {
  // 初始化状态
  initialized: boolean;
  initFromBackend: () => Promise<void>;
  
  // Theme
  theme: ThemeMode;
  setTheme: (theme: ThemeMode) => void;

  // UI 状态
  viewMode: ViewMode;
  setViewMode: (mode: ViewMode) => void;

  // 语言
  language: string;
  setLanguage: (lang: string) => void;

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

export const useAppStore = create<AppState>()((set) => ({
  // 初始化状态
  initialized: false,
  initFromBackend: async () => {
    try {
      const settings = await invoke<AppSettings>("get_app_settings");
      const theme = (settings.theme || "dark") as ThemeMode;
      const viewMode = (settings.view_mode || "grid") as ViewMode;
      const language = settings.language || "zh";
      
      applyThemeToDOM(theme);
      set({ 
        theme,
        viewMode,
        language,
        initialized: true 
      });
    } catch (error) {
      console.error("Failed to load settings from backend:", error);
      // 使用默认值
      applyThemeToDOM("dark");
      set({ initialized: true });
    }
  },

  // Theme - 默认暗色
  theme: "dark",
  setTheme: (theme) => {
    applyThemeToDOM(theme);
    set({ theme });
    saveSettingToBackend("theme", theme);
  },

  // UI 状态
  viewMode: "grid",
  setViewMode: (mode) => {
    set({ viewMode: mode });
    saveSettingToBackend("view_mode", mode);
  },

  // 语言
  language: "zh",
  setLanguage: (lang) => {
    set({ language: lang });
    saveSettingToBackend("language", lang);
  },

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
