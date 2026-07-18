import { describe, expect, it } from "vitest";
import { hasMetadataAndAsset, shouldIncludeInBatchScrape } from "./romScrapeStatus";
import type { Rom } from "@/types";

function rom(overrides: Partial<Rom> = {}): Rom {
  return {
    file: "game.gba",
    name: "Game",
    directory: "D:\\Roms\\GBA",
    system: "GBA",
    has_temp_metadata: false,
    ...overrides,
  };
}

describe("hasMetadataAndAsset", () => {
  it("only skips ROMs that have both metadata and an asset", () => {
    expect(hasMetadataAndAsset(rom({ description: "Description" }))).toBe(false);
    expect(hasMetadataAndAsset(rom({ box_front: "media/box.png" }))).toBe(false);
    expect(hasMetadataAndAsset(rom({ description: "Description", box_front: "media/box.png" }))).toBe(true);
  });

  it("combines exported and temporary scrape data", () => {
    expect(hasMetadataAndAsset(rom({
      developer: "Studio",
      temp_data: { screenshot: "cache/screenshot.png" },
    }))).toBe(true);
    expect(hasMetadataAndAsset(rom({
      has_temp_metadata: true,
      temp_data: { box_front: "cache/box.png" },
    }))).toBe(true);
  });

  it("includes already scraped ROMs when full rescrape is enabled", () => {
    const scraped = rom({
      description: "Description",
      box_front: "media/box.png",
    });

    expect(shouldIncludeInBatchScrape(scraped, false)).toBe(false);
    expect(shouldIncludeInBatchScrape(scraped, true)).toBe(true);
    expect(shouldIncludeInBatchScrape(rom(), false)).toBe(true);
  });

  it("re-includes scraped ROMs that are missing a selected media type", () => {
    const scraped = rom({
      description: "Description",
      box_front: "media/box.png",
    });

    // 已有元数据+封面,选中类型都齐全时不重复抓取
    expect(shouldIncludeInBatchScrape(scraped, false, ["boxfront"])).toBe(false);
    // 选了 video 但缺 video,应补抓
    expect(shouldIncludeInBatchScrape(scraped, false, ["boxfront", "video"])).toBe(true);
    // 已有 video(含 temp_data)则不再纳入
    const withVideo = rom({
      description: "Description",
      box_front: "media/box.png",
      temp_data: { video: "cache/clip.mp4" },
    });
    expect(shouldIncludeInBatchScrape(withVideo, false, ["boxfront", "video"])).toBe(false);
    // manual 无对应字段,不触发补抓
    expect(shouldIncludeInBatchScrape(scraped, false, ["manual"])).toBe(false);
  });
});
