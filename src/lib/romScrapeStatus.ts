import type { Rom } from "@/types";

const METADATA_FIELDS: (keyof Rom)[] = [
  "description", "summary", "developer", "publisher", "genre",
  "players", "release", "rating", "english_name",
];

const ASSET_FIELDS: (keyof Rom)[] = [
  "box_front", "box_back", "box_spine", "box_full", "cartridge",
  "logo", "marquee", "bezel", "gridicon", "flyer", "background",
  "music", "screenshot", "titlescreen", "video",
];

// 抓取媒体类型 ID → ROM 字段。与后端 persistence.rs 的 MediaType 映射保持一致。
// manual/other 无对应字段,不参与缺失判断。
const MEDIA_TYPE_FIELDS: Record<string, keyof Rom> = {
  boxfront: "box_front",
  boxback: "box_back",
  box3d: "box_full",
  screenshot: "screenshot",
  titlescreen: "titlescreen",
  logo: "logo",
  icon: "gridicon",
  hero: "background",
  banner: "marquee",
  video: "video",
};

function hasValue(value: unknown): boolean {
  return typeof value === "string" && value.trim().length > 0;
}

export function hasMetadataAndAsset(rom: Rom): boolean {
  const records: Partial<Rom>[] = [rom, rom.temp_data || {}];
  const hasMetadata = rom.has_temp_metadata || records.some(record =>
    METADATA_FIELDS.some(field => hasValue(record[field])),
  );
  const hasAsset = records.some(record =>
    ASSET_FIELDS.some(field => hasValue(record[field])),
  );
  return hasMetadata && hasAsset;
}

/** ROM 是否已具备所有选中媒体类型对应的资源(导出与临时数据合并判断)。 */
export function hasSelectedAssets(rom: Rom, mediaTypes: string[]): boolean {
  const records: Partial<Rom>[] = [rom, rom.temp_data || {}];
  return mediaTypes.every(type => {
    const field = MEDIA_TYPE_FIELDS[type];
    if (!field) return true; // 无对应字段的类型(manual 等)不参与缺失判断
    return records.some(record => hasValue(record[field]));
  });
}

/**
 * 是否纳入批量抓取。
 * - 全量重抓:全部纳入。
 * - 未抓取(缺元数据或资产):纳入。
 * - 已抓取但缺少任一选中媒体类型:纳入以补抓缺失资源(命中缓存,不重复下载已有资源)。
 */
export function shouldIncludeInBatchScrape(
  rom: Rom,
  forceRescrape: boolean,
  mediaTypes: string[] = [],
): boolean {
  if (forceRescrape) return true;
  if (!hasMetadataAndAsset(rom)) return true;
  if (mediaTypes.length > 0 && !hasSelectedAssets(rom, mediaTypes)) return true;
  return false;
}
