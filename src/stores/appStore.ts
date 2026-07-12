import { create } from "zustand";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import type { ViewMode, SortOption, FilterOption } from "@/types";
import type { LoadedTheme, MotionLevel } from "@/theme/types";
import { DEFAULT_THEME_ID, resolveTheme } from "@/theme/registry";
import { applyTheme } from "@/theme/apply";
import { validateManifest } from "@/theme/validate";
import { isTauri, themeApi } from "@/lib/api";

// 保存设置到后端
const saveSettingToBackend = async (key: string, value: string) => {
  try {
    await invoke("update_app_setting", { key, value });
  } catch (error) {
    console.error("Failed to save setting to backend:", error);
  }
};

/**
 * 主题包资产相对路径安全校验:
 * 拒绝空值、以 `/` 开头(绝对路径)、含 `\`(Windows 分隔符)或 `:`(盘符/协议)的路径,
 * 并按 `/` 分段归一化,拒绝 `.`、`..` 与空段(路径穿越防护)。
 */
export function isSafeAssetPath(path: string): boolean {
  const p = path.trim();
  if (!p || p.startsWith("/") || p.includes("\\") || p.includes(":")) return false;
  return p.split("/").every((seg) => seg !== "" && seg !== "." && seg !== "..");
}

// 纵深防御:后端清洗存在 EOF 处不带分号的极窄逃逸,这里把残留 @import 再剥一遍
const RESIDUAL_IMPORT = /@import[^;]*(;|$)/gi;
// __RR_ASSET__/<path> 占位符(后端将合法相对 url 重写为该形式)
const ASSET_URL = /url\(\s*(['"]?)__RR_ASSET__\/([^)'"]*)\1\s*\)/gi;

/**
 * 清洗导入主题的自定义 CSS:剥残留 @import;
 * `__RR_ASSET__/<path>` 占位符先做路径安全校验,不合法的直接置为空 url(),
 * 合法的替换为 convertFileSrc(<dir>/assets/<path>)。
 */
export function sanitizeCustomCss(
  css: string,
  dir: string,
  convert: (path: string) => string = convertFileSrc,
): string {
  return css
    .replace(RESIDUAL_IMPORT, "")
    .replace(ASSET_URL, (_match, _quote, rawPath: string) => {
      const rel = rawPath.trim();
      if (!isSafeAssetPath(rel)) return "url()";
      return `url("${convert(`${dir}/assets/${rel}`)}")`;
    });
}

/** 从后端装载已导入主题;仅 Tauri 模式;任何失败静默降级为空列表 */
async function loadCustomThemes(): Promise<LoadedTheme[]> {
  if (!isTauri()) return [];
  try {
    const infos = await themeApi.listCustomThemes();
    const loaded: LoadedTheme[] = [];
    for (const info of infos) {
      try {
        const result = validateManifest(JSON.parse(info.manifest_json));
        if (!result.ok || !result.manifest) continue;
        loaded.push({
          ...result.manifest,
          builtin: false,
          dir: info.dir,
          customCss: info.custom_css
            ? sanitizeCustomCss(info.custom_css, info.dir)
            : undefined,
        });
      } catch {
        // 单个损坏主题静默跳过
      }
    }
    return loaded;
  } catch {
    return [];
  }
}

// 侧边栏面板收起态持久化:后端 update_app_setting 为显式 match 且无对应分支
// (未知 key 被静默丢弃),故沿用 localStorage 持久化(旧 key sidebar_collapsed 已废弃)
const SIDEBAR_PANEL_COLLAPSED_KEY = "sidebar_panel_collapsed";

const loadSidebarPanelCollapsed = (): boolean => {
  try {
    return localStorage.getItem(SIDEBAR_PANEL_COLLAPSED_KEY) === "true";
  } catch {
    return false;
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

  // 导入的主题(从后端 config/themes/ 装载)
  customThemes: LoadedTheme[];
  refreshCustomThemes: () => Promise<void>;

  // 侧边栏面板收起态(localStorage 持久化;收起 = rail-only)
  sidebarPanelCollapsed: boolean;
  setSidebarPanelCollapsed: (collapsed: boolean) => void;

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
      const [settings, customThemes] = await Promise.all([
        invoke<AppSettings>("get_app_settings"),
        loadCustomThemes(),
      ]);
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
        customThemes,
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
  refreshCustomThemes: async () => {
    const customThemes = await loadCustomThemes();
    const { themeId, motion } = get();
    // 当前主题被删除时经 resolveTheme 回退默认;被覆盖导入时重新应用新内容
    const theme = resolveTheme(themeId, customThemes);
    applyTheme(theme, motion);
    set({ customThemes, themeId: theme.id });
  },

  // 侧边栏面板收起态
  sidebarPanelCollapsed: loadSidebarPanelCollapsed(),
  setSidebarPanelCollapsed: (collapsed) => {
    set({ sidebarPanelCollapsed: collapsed });
    try {
      localStorage.setItem(SIDEBAR_PANEL_COLLAPSED_KEY, String(collapsed));
    } catch {
      // 存储不可用时静默降级为会话内状态
    }
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
