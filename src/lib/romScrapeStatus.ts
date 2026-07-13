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
