import type { ScraperSearchResult } from "@/types";

const STORAGE_KEY = "mrrm.scraper-search-cache.v1";
const MAX_ENTRIES = 100;

interface CacheEntry {
  results: ScraperSearchResult[];
  updatedAt: number;
}

type SearchCache = Record<string, CacheEntry>;

export interface SearchCacheContext {
  query: string;
  fileName: string;
  system?: string;
  directory?: string;
}

function normalize(value?: string): string {
  return (value || "").trim().replace(/\\/g, "/").toLocaleLowerCase();
}

function cacheKey(context: SearchCacheContext): string {
  return JSON.stringify([
    normalize(context.system),
    normalize(context.query),
  ]);
}

function readCache(): SearchCache {
  if (typeof localStorage === "undefined") return {};
  try {
    const parsed = JSON.parse(localStorage.getItem(STORAGE_KEY) || "{}");
    return parsed && typeof parsed === "object" ? parsed as SearchCache : {};
  } catch {
    return {};
  }
}

export function getCachedSearchResults(context: SearchCacheContext): ScraperSearchResult[] | null {
  const entry = readCache()[cacheKey(context)];
  return Array.isArray(entry?.results) ? entry.results : null;
}

export function cacheSearchResults(
  context: SearchCacheContext,
  results: ScraperSearchResult[],
): void {
  if (typeof localStorage === "undefined") return;
  const cache = readCache();
  cache[cacheKey(context)] = { results, updatedAt: Date.now() };
  const entries = Object.entries(cache)
    .sort(([, left], [, right]) => right.updatedAt - left.updatedAt)
    .slice(0, MAX_ENTRIES);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(Object.fromEntries(entries)));
}

export function clearCachedSearchResults(): void {
  if (typeof localStorage === "undefined") return;
  localStorage.removeItem(STORAGE_KEY);
}
