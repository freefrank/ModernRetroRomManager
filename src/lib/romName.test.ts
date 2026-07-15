import { describe, expect, it } from "vitest";
import { getRomDisplayName } from "@/lib/romName";
import type { Rom } from "@/types";

const rom: Rom = {
  file: "game.gba",
  name: "Original Game",
  chinese_name: "中文游戏",
  directory: "Y:/gba",
  system: "gba",
  has_temp_metadata: false,
};

describe("ROM localized display name", () => {
  it("prefers chinese_name for simplified and traditional Chinese", () => {
    expect(getRomDisplayName(rom, "zh-CN")).toBe("中文游戏");
    expect(getRomDisplayName(rom, "zh-TW")).toBe("中文游戏");
  });

  it("keeps the original name for other languages", () => {
    expect(getRomDisplayName(rom, "en")).toBe("Original Game");
    expect(getRomDisplayName(rom, "fr")).toBe("Original Game");
  });
});
