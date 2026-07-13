import { describe, expect, it } from "vitest";
import { normalizeMediaType, selectOneAssetPerType } from "./mediaSelection";
import type { ScraperMediaAsset } from "@/types";

function asset(assetType: string, url: string): ScraperMediaAsset {
  return { provider: "test", asset_type: assetType, url };
}

describe("mediaSelection", () => {
  it("selects only the first asset of each type", () => {
    const selected = selectOneAssetPerType([
      asset("box_front", "box-1"),
      asset("boxfront", "box-2"),
      asset("screenshot", "screen-1"),
      asset("screenshot", "screen-2"),
      asset("logo", "logo-1"),
    ]);
    expect([...selected]).toEqual(["box-1", "screen-1", "logo-1"]);
  });

  it("normalizes backend and legacy media type spellings", () => {
    expect(normalizeMediaType("box_front")).toBe("boxfront");
    expect(normalizeMediaType("box-Front")).toBe("boxfront");
  });
});

