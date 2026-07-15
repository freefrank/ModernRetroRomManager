import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
const convertFileSrc = vi.fn((path: string) => `asset://${path}`);

vi.mock("@tauri-apps/api/core", () => ({
  invoke,
  convertFileSrc,
}));

import { clearCachedMediaUrls, resolveMediaUrlAsync } from "./api";

describe("library media URL cache", () => {
  beforeEach(() => {
    invoke.mockReset();
    convertFileSrc.mockClear();
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });
  });

  it("deduplicates concurrent cache requests and reuses the resolved URL", async () => {
    invoke.mockResolvedValue("C:\\cache\\cover.png");
    const source = "Y:\\gba\\media\\cover-unique.png";

    const [first, second] = await Promise.all([
      resolveMediaUrlAsync(source),
      resolveMediaUrlAsync(source),
    ]);
    const third = await resolveMediaUrlAsync(source);

    expect(first).toBe("asset://C:\\cache\\cover.png");
    expect(second).toBe(first);
    expect(third).toBe(first);
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("resolve_cached_library_asset", {
      path: source,
    });

    clearCachedMediaUrls();
    await resolveMediaUrlAsync(source);
    expect(invoke).toHaveBeenCalledTimes(2);
  });
});
