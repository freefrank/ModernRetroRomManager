import retroArcade from "./builtin/retro-arcade.json";
import modernDark from "./builtin/modern-dark.json";
import cyberpunk from "./builtin/cyberpunk.json";
import violet from "./builtin/violet.json";
import type { LoadedTheme, ThemeManifest } from "./types";

export const DEFAULT_THEME_ID = "retro-arcade";
export const BUILTIN_THEMES: LoadedTheme[] = [retroArcade, modernDark, cyberpunk, violet]
  .map(m => ({ ...(m as ThemeManifest), builtin: true }));

/** 内置 + 已导入(导入列表由调用方传入,Wave 2 接后端) */
export function resolveTheme(id: string | undefined, custom: LoadedTheme[] = []): LoadedTheme {
  const all = [...BUILTIN_THEMES, ...custom];
  return all.find(t => t.id === id) ?? all.find(t => t.id === DEFAULT_THEME_ID)!;
}
