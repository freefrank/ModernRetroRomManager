import { beforeEach, describe, expect, it } from "vitest";
import { cacheSearchResults, getCachedSearchResults } from "./scraperSearchCache";
import type { ScraperSearchResult } from "@/types";

const context = {
  query: "CT Special Forces 2",
  fileName: "ct2.gba",
  system: "GBA",
  directory: "D:\\Roms\\GBA",
};

const result: ScraperSearchResult = {
  provider: "screenscraper",
  source_id: "2",
  name: "CT Special Forces 2",
  confidence: 0.98,
};

describe("scraperSearchCache", () => {
  beforeEach(() => localStorage.clear());

  it("restores results for the same ROM and normalized query", () => {
    cacheSearchResults(context, [result]);
    expect(getCachedSearchResults({
      ...context,
      query: " ct special forces 2 ",
      directory: "d:/roms/gba",
    })).toEqual([result]);
  });

  it("does not reuse results for another query", () => {
    cacheSearchResults(context, [result]);
    expect(getCachedSearchResults({ ...context, query: "CT Special Forces" })).toBeNull();
  });
});

