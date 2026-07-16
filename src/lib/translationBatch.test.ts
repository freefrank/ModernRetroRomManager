import { describe, expect, it } from "vitest";
import type { TranslateMetadataRequest } from "@/types";
import { groupTranslationRequests } from "./translationBatch";

const request = (name: string, description: string): TranslateMetadataRequest => ({
  system: "gba",
  file_name: `${name}.zip`,
  metadata: { name, description, genres: [] },
});

describe("groupTranslationRequests", () => {
  it("uses the selected context budget instead of a fixed item count", () => {
    const items = Array.from({ length: 12 }, (_, index) => request(`Game ${index}`, "中文".repeat(3000)));
    const small = groupTranslationRequests(items, item => item, 50000);
    const large = groupTranslationRequests(items, item => item, 200000);
    expect(small.length).toBeGreaterThan(large.length);
    expect(large.flat()).toEqual(items);
  });

  it("keeps an oversized entry intact for individual fallback", () => {
    const oversized = request("Large", "中文".repeat(30000));
    expect(groupTranslationRequests([oversized], item => item, 50000)).toEqual([[oversized]]);
  });
});
