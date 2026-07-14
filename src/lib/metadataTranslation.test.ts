import { describe, expect, it } from "vitest";
import { romMetadata } from "./metadataTranslation";
import type { Rom } from "@/types";

const base: Rom = {
  file: "game.gba",
  name: "Original",
  directory: "C:/roms/gba",
  system: "gba",
  has_temp_metadata: true,
  release: "2001-01-01",
  rating: "8.5",
  temp_data: { name: "Scraped", description: "Description", genre: "Action" },
};

describe("romMetadata", () => {
  it("prefers current preview metadata and preserves non-language fields", () => {
    expect(romMetadata(base)).toEqual(expect.objectContaining({
      name: "Scraped",
      description: "Description",
      genres: ["Action"],
      release_date: "2001-01-01",
      rating: 8.5,
    }));
  });
});
