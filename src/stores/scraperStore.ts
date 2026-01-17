import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface ScraperConfig {
  enabled: boolean;
  api_key?: string;
  client_id?: string;
  client_secret?: string;
  username?: string;
  password?: string;
}

interface ScraperState {
  configs: Record<string, ScraperConfig>; // Map provider -> config
  fetchConfigs: () => Promise<void>;
  saveConfig: (provider: string, config: Partial<ScraperConfig>) => Promise<void>;
}

export const useScraperStore = create<ScraperState>((set, get) => ({
  configs: {},
  fetchConfigs: async () => {
    try {
      const configMap = await invoke<Record<string, ScraperConfig>>("get_scraper_configs");
      set({ configs: configMap });
    } catch (error) {
      console.error("Failed to fetch scraper configs:", error);
    }
  },
  saveConfig: async (provider, config) => {
    try {
      // 合并现有配置
      const existing = get().configs[provider] || { enabled: false };
      const merged: ScraperConfig = { ...existing, ...config };
      await invoke("save_scraper_config", { provider, config: merged });
      await get().fetchConfigs();
    } catch (error) {
      console.error("Failed to save scraper config:", error);
      throw error;
    }
  },
}));

