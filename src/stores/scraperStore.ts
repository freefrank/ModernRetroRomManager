import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface ApiConfig {
  id: string;
  provider: string;
  apiKey?: string;
  clientId?: string;
  clientSecret?: string;
  enabled: boolean;
  priority: number;
}

interface UpdateApiConfig {
  provider: string;
  apiKey?: string;
  clientId?: string;
  clientSecret?: string;
  enabled?: boolean;
}

interface ScraperState {
  configs: Record<string, ApiConfig>; // Map provider -> config
  fetchConfigs: () => Promise<void>;
  saveConfig: (config: UpdateApiConfig) => Promise<void>;
}

export const useScraperStore = create<ScraperState>((set, get) => ({
  configs: {},
  fetchConfigs: async () => {
    try {
      const list = await invoke<ApiConfig[]>("get_api_configs");
      const configMap: Record<string, ApiConfig> = {};
      list.forEach((c) => {
        configMap[c.provider] = c;
      });
      set({ configs: configMap });
    } catch (error) {
      console.error("Failed to fetch api configs:", error);
    }
  },
  saveConfig: async (config) => {
    try {
      await invoke("save_api_config", { config });
      await get().fetchConfigs();
    } catch (error) {
      console.error("Failed to save api config:", error);
      throw error;
    }
  },
}));
