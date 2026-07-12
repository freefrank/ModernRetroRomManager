import { create } from "zustand";
import { scraperApi } from "@/lib/api";
import type { ScraperProviderInfo, ScraperCredentials } from "@/types";

interface ScraperState {
  providers: ScraperProviderInfo[];
  isLoading: boolean;
  fetchProviders: () => Promise<void>;
  configureProvider: (providerId: string, credentials: ScraperCredentials) => Promise<void>;
  setProviderEnabled: (providerId: string, enabled: boolean) => Promise<void>;
  setProviderPriority: (providerId: string, priority: number) => Promise<void>;
}

export const useScraperStore = create<ScraperState>((set, get) => ({
  providers: [],
  isLoading: false,
  fetchProviders: async () => {
    set({ isLoading: true });
    try {
      const providers = await scraperApi.getProviders();
      set({ providers, isLoading: false });
    } catch (error) {
      console.error("Failed to fetch scraper providers:", error);
      set({ isLoading: false });
    }
  },
  configureProvider: async (providerId, credentials) => {
    try {
      await scraperApi.configureProvider(providerId, credentials);
      await get().fetchProviders();
    } catch (error) {
      console.error(`Failed to configure provider ${providerId}:`, error);
      throw error;
    }
  },
  setProviderEnabled: async (providerId, enabled) => {
    // 乐观更新,保存成功后重新拉取列表与后端状态对齐
    set({
      providers: get().providers.map((p) =>
        p.id === providerId ? { ...p, enabled } : p
      ),
    });
    try {
      await scraperApi.setProviderEnabled(providerId, enabled);
      await get().fetchProviders();
    } catch (error) {
      console.error(`Failed to set provider ${providerId} enabled:`, error);
      await get().fetchProviders(); // 失败时回滚
      throw error;
    }
  },
  setProviderPriority: async (providerId, priority) => {
    // 乐观更新,保存成功后重新拉取列表与后端状态对齐
    set({
      providers: get().providers.map((p) =>
        p.id === providerId ? { ...p, priority } : p
      ),
    });
    try {
      await scraperApi.setProviderPriority(providerId, priority);
      await get().fetchProviders();
    } catch (error) {
      console.error(`Failed to set provider ${providerId} priority:`, error);
      await get().fetchProviders(); // 失败时回滚
      throw error;
    }
  },
}));

