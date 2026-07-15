import type { Rom } from "@/types";

export function isChineseInterface(language: string | undefined): boolean {
  return language?.toLowerCase().startsWith("zh") ?? false;
}

export function getRomDisplayName(rom: Rom, language: string | undefined): string {
  if (isChineseInterface(language)) {
    const chineseName = rom.temp_data?.chinese_name?.trim() || rom.chinese_name?.trim();
    if (chineseName) return chineseName;
  }
  return rom.temp_data?.name?.trim() || rom.name;
}
