import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { ViewMode, SortOption, FilterOption } from "@/types";
import type { LoadedTheme, MotionLevel } from "@/theme/types";
import { DEFAULT_THEME_ID, resolveTheme } from "@/theme/registry";
import { applyTheme } from "@/theme/apply";

// 保存设置到后端
const saveSettingToBackend = async (key: string, value: string) => {
  try {
    await invoke("update_app_setting", { key, value });
  } catch (error) {
    console.error("Failed to save setting to backend:", error);
  }
};

// 无存储值时的动效默认档:尊重系统"减弱动态效果"偏好
const defaultMotionLevel = (): MotionLevel =>
  window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "low" : "full";

const isMotionLevel = (v: unknown): v is MotionLevel =>
  v === "off" || v === "low" || v === "full";

interface AppSettings {
  theme: string;
  language: string;
  view_mode: string;
  motion_level?: string;
}

interface AppState {
  // 初始化状态
  initialized: boolean;
  initFromBackend: () => Promise<void>;

  // 主题
  themeId: string;
  setTheme: (id: string) => void;

  // 动效档位
  motion: MotionLevel;
  setMotion: (level: MotionLevel) => void;

  // 导入的主题(Wave 2 接后端,本阶段恒为空)
  customThemes: LoadedTheme[];

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

export const useAppStore = create<AppState>()((set, get) => ({
  // 初始化状态
  initialized: false,
  initFromBackend: async () => {
    try {
      const settings = await invoke<AppSettings>("get_app_settings");
      const { customThemes } = get();
      // 旧值(dark/ocean 等)与未知 id 经 resolveTheme 自然回退默认主题
      const theme = resolveTheme(settings.theme, customThemes);
      const motion = isMotionLevel(settings.motion_level)
        ? settings.motion_level
        : defaultMotionLevel();
      const viewMode = (settings.view_mode || "grid") as ViewMode;
      const language = settings.language || "zh";

      applyTheme(theme, motion);
      set({
        themeId: theme.id,
        motion,
        viewMode,
        language,
        initialized: true,
      });
    } catch (error) {
      console.error("Failed to load settings from backend:", error);
      // 使用默认值
      const motion = defaultMotionLevel();
      applyTheme(resolveTheme(DEFAULT_THEME_ID), motion);
      set({ motion, initialized: true });
    }
  },

  // 主题 - 默认复古游戏厅
  themeId: DEFAULT_THEME_ID,
  setTheme: (id) => {
    const { customThemes, motion } = get();
    const theme = resolveTheme(id, customThemes);
    applyTheme(theme, motion);
    set({ themeId: theme.id });
    saveSettingToBackend("theme", theme.id);
  },

  // 动效档位
  motion: "full",
  setMotion: (level) => {
    const { themeId, customThemes } = get();
    applyTheme(resolveTheme(themeId, customThemes), level);
    set({ motion: level });
    saveSettingToBackend("motion_level", level);
  },

  // 导入的主题
  customThemes: [],

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
