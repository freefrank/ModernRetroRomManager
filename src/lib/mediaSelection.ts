import type { ScraperMediaAsset } from "@/types";

export function normalizeMediaType(assetType: string): string {
  return assetType.trim().toLocaleLowerCase().replace(/[-_\s]/g, "");
}

export function selectOneAssetPerType(media: ScraperMediaAsset[]): Set<string> {
  const selectedUrls = new Set<string>();
  const selectedTypes = new Set<string>();
  for (const asset of media) {
    const assetType = normalizeMediaType(asset.asset_type);
    if (!selectedTypes.has(assetType)) {
      selectedTypes.add(assetType);
      selectedUrls.add(asset.url);
    }
  }
  return selectedUrls;
}

