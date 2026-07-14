import { describe, it, expect, vi, beforeEach } from "vitest";
import type { ScraperProviderInfo } from "@/types";

vi.mock("@/lib/api", () => ({
  scraperApi: {
    getProviders: vi.fn(),
    configureProvider: vi.fn(),
    setProviderEnabled: vi.fn(),
    setProviderPriority: vi.fn(),
  },
}));

import { scraperApi } from "@/lib/api";
import { useScraperStore } from "./scraperStore";

const mockedApi = vi.mocked(scraperApi);

const makeProvider = (
  overrides: Partial<ScraperProviderInfo> = {},
): ScraperProviderInfo => ({
  id: "steamgriddb",
  name: "SteamGridDB",
  enabled: true,
  priority: 100,
  requires_credentials: true,
  has_credentials: false,
  has_saved_password: false,
  capabilities: ["search", "media"],
  rate_limit: 1,
  threads: 1,
  matched_count: 0,
  selected_count: 0,
  cache_hit_count: 0,
  error_count: 0,
  ...overrides,
});

beforeEach(() => {
  vi.clearAllMocks();
  useScraperStore.setState({ providers: [], isLoading: false });
});

describe("scraperStore", () => {
  it("fetchProviders 拉取动态 provider 列表", async () => {
    const providers = [
      makeProvider(),
      makeProvider({ id: "screenscraper", name: "ScreenScraper" }),
    ];
    mockedApi.getProviders.mockResolvedValue(providers);

    await useScraperStore.getState().fetchProviders();

    expect(useScraperStore.getState().providers).toEqual(providers);
    expect(useScraperStore.getState().isLoading).toBe(false);
  });

  it("setProviderEnabled 保存后重新拉取列表,状态与后端一致", async () => {
    useScraperStore.setState({ providers: [makeProvider({ enabled: true })] });
    mockedApi.setProviderEnabled.mockResolvedValue(undefined);
    // 后端保存后重新返回的权威状态
    mockedApi.getProviders.mockResolvedValue([makeProvider({ enabled: false })]);

    await useScraperStore.getState().setProviderEnabled("steamgriddb", false);

    expect(mockedApi.setProviderEnabled).toHaveBeenCalledWith("steamgriddb", false);
    expect(mockedApi.getProviders).toHaveBeenCalled();
    expect(useScraperStore.getState().providers[0].enabled).toBe(false);
  });

  it("setProviderEnabled 失败时回滚到后端状态", async () => {
    useScraperStore.setState({ providers: [makeProvider({ enabled: true })] });
    mockedApi.setProviderEnabled.mockRejectedValue(new Error("保存失败"));
    mockedApi.getProviders.mockResolvedValue([makeProvider({ enabled: true })]);

    await expect(
      useScraperStore.getState().setProviderEnabled("steamgriddb", false),
    ).rejects.toThrow();

    expect(useScraperStore.getState().providers[0].enabled).toBe(true);
  });

  it("configureProvider 保存凭证后重新拉取列表,凭证状态一致", async () => {
    useScraperStore.setState({
      providers: [makeProvider({ has_credentials: false })],
    });
    mockedApi.configureProvider.mockResolvedValue(undefined);
    mockedApi.getProviders.mockResolvedValue([
      makeProvider({ has_credentials: true }),
    ]);

    await useScraperStore
      .getState()
      .configureProvider("steamgriddb", { api_key: "key" });

    expect(mockedApi.configureProvider).toHaveBeenCalledWith("steamgriddb", {
      api_key: "key",
    });
    expect(useScraperStore.getState().providers[0].has_credentials).toBe(true);
  });
});
