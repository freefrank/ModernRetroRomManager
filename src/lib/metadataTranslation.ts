import type { Rom, ScraperGameMetadata } from "@/types";

export function effectiveRom(rom: Rom): Rom {
  return rom.temp_data ? { ...rom, ...rom.temp_data } : rom;
}

export function romMetadata(rom: Rom): ScraperGameMetadata {
  const source = effectiveRom(rom);
  const rating = typeof source.rating === "string" ? Number.parseFloat(source.rating) : source.rating;
  return {
    name: source.name || rom.name,
    english_name: source.english_name,
    chinese_name: source.chinese_name,
    description: source.description || source.summary,
    release_date: source.release,
    developer: source.developer,
    publisher: source.publisher,
    genres: source.genre ? [source.genre] : [],
    players: source.players,
    rating: typeof rating === "number" && Number.isFinite(rating) ? rating : undefined,
    translated_languages: source.translated_languages,
  };
}
